//! Platform layer: window, 8-bit screen, palette, keyboard, gamepad, timing and audio mixing.
//!
//! Exposes an Allegro-like free-function API so the ported game code keeps the original
//! blocking structure (menus that `rest()` and poll `keypressed()`). All rendering of the
//! game goes through `present()`, which is the single place to swap in upgraded graphics.

use crate::gfx::{self, Bitmap, Palette, Rgb, BLACK_PALETTE};
use crate::keys::*;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::controller::{Axis, Button, GameController};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod, Scancode};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{FullscreenType, Window, WindowContext};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const SCREEN_W: i32 = 320;
pub const SCREEN_H: i32 = 240;
pub const MIX_RATE: i32 = 44100;
const SFX_VOICES: usize = 4;

#[derive(Default, Clone, Copy)]
pub struct Joy {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub b: [bool; 8],
}

struct Display {
    canvas: Canvas<Window>,
    creator: &'static TextureCreator<WindowContext>,
    texture: Texture<'static>,
    tex_w: u32,
    tex_h: u32,
    linear: bool,
}

fn open_display(video: &sdl2::VideoSubsystem) -> Display {
    let mut window = video
        .window("URBAN - The Cyborg Project", (SCREEN_W * 3) as u32, (SCREEN_H * 3) as u32)
        .position_centered()
        .resizable()
        .allow_highdpi()
        .build()
        .expect("SDL window");
    if crate::video::VideoConf::load().fullscreen {
        let _ = window.set_fullscreen(FullscreenType::Desktop);
    }
    let mut canvas = window.into_canvas().accelerated().build().expect("SDL renderer");
    canvas.set_logical_size(SCREEN_W as u32, SCREEN_H as u32).ok();
    canvas.set_integer_scale(true).ok();
    let creator: &'static TextureCreator<WindowContext> = Box::leak(Box::new(canvas.texture_creator()));
    let texture = creator
        .create_texture_streaming(PixelFormatEnum::ARGB8888, SCREEN_W as u32, SCREEN_H as u32)
        .expect("SDL texture");
    Display { canvas, creator, texture, tex_w: SCREEN_W as u32, tex_h: SCREEN_H as u32, linear: false }
}

struct Sys {
    _sdl: sdl2::Sdl,
    _video: sdl2::VideoSubsystem,
    timer: sdl2::TimerSubsystem,
    events: sdl2::EventPump,
    controller_sys: sdl2::GameControllerSubsystem,
    pad: Option<GameController>,
    /// None in headless test runs (URBAN_HEADLESS with URBAN_REPLAY).
    display: Option<Display>,
    screen: Bitmap,
    pal: Palette,
    pal32: [u32; 256],
    video: crate::video::VideoConf,
    crt: Option<crate::video::Crt>,
    buf_a: Vec<u32>,
    buf_b: Vec<u32>,
    buf_c: Vec<u32>,
    /// One-shot capture of the final filtered frame (test/screenshot hook).
    rgb_shot: Option<PathBuf>,
    keys: [bool; 128],
    keybuf: VecDeque<i32>,
    joy: Joy,
    last_present: Instant,
    next_frame: Option<Instant>,
    shot_dir: Option<PathBuf>,
    last_shot: Instant,
    shot_n: u32,
    autoplay: bool,
    start: Instant,
    last_auto_tick: u64,
    audio: Option<AudioDevice<Mixer>>,
    mixer: Arc<Mutex<MixState>>,
    replay: Option<Replay>,
    menu_keys: Option<MenuKeys>,
}

/// Deterministic replay for diffing against the C++ build (tools/difftest.sh).
/// Mirrors the hooks in port/sdl_allegro.cc exactly: same script format, hash and timing.
struct Replay {
    steps: Vec<(u64, [bool; 128])>,
    cur: usize,
    frame: u64,
    stop: Option<u64>,
    dump: Option<u64>,
    dump_dir: Option<PathBuf>,
    trace: Option<std::io::BufWriter<std::fs::File>>,
}

/// Menu replay (URBAN_MENU_KEYS): scripted keys are handed out whenever the game reads one,
/// and the screen is fingerprinted at each read (mirrors port/sdl_allegro.cc).
struct MenuKeys {
    keys: Vec<i32>,
    cur: usize,
    trace: Option<std::fs::File>,
    dump: Option<usize>,
    dump_dir: Option<PathBuf>,
}

impl MenuKeys {
    fn load() -> Option<MenuKeys> {
        let path = std::env::var_os("URBAN_MENU_KEYS")?;
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("URBAN_MENU_KEYS: {}", e);
            std::process::exit(1)
        });
        let nums: Vec<i32> = text.split_whitespace().filter_map(|t| t.parse().ok()).collect();
        let keys = nums.chunks(2).filter(|c| c.len() == 2).map(|c| (c[0] << 8) | c[1]).collect();
        Some(MenuKeys {
            keys,
            cur: 0,
            trace: std::env::var_os("URBAN_MENU_TRACE").and_then(|p| std::fs::File::create(p).ok()),
            dump: std::env::var("URBAN_DUMP").ok().and_then(|v| v.parse().ok()),
            dump_dir: std::env::var_os("URBAN_DUMP_DIR").map(PathBuf::from),
        })
    }
}

pub fn dump_screen(path: &std::path::Path) {
    with(|s| write_bmp(path, &s.screen, &s.pal));
}

/// FNV-1a hash of the screen and palette (replay tests).
pub fn current_screen_hash() -> u64 {
    with(|s| screen_hash(s))
}

fn screen_hash(s: &Sys) -> u64 {
    let mut h: u64 = 1469598103934665603;
    for &c in &s.screen.data {
        h ^= c as u64;
        h = h.wrapping_mul(1099511628211);
    }
    for c in s.pal.iter() {
        h ^= c.r as u64 | (c.g as u64) << 8 | (c.b as u64) << 16;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

impl Replay {
    fn empty() -> Replay {
        let stop = std::env::var("URBAN_FRAMES").ok().and_then(|v| v.parse().ok());
        Replay { steps: Vec::new(), cur: 0, frame: 0, stop, dump: None, dump_dir: None, trace: None }
    }

    fn load() -> Option<Replay> {
        let path = std::env::var_os("URBAN_REPLAY")?;
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("URBAN_REPLAY: {}", e);
            std::process::exit(1)
        });
        let mut steps = Vec::new();
        for line in text.lines() {
            let mut it = line.split_whitespace();
            let Some(first) = it.next() else { continue };
            if first.starts_with('#') {
                continue;
            }
            let mut keys = [false; 128];
            for k in it.filter_map(|t| t.parse::<usize>().ok()) {
                if k > 0 && k < 128 {
                    keys[k] = true;
                }
            }
            steps.push((first.parse().unwrap_or(0), keys));
        }
        let env_u64 = |n: &str| std::env::var(n).ok().and_then(|v| v.parse::<u64>().ok());
        if let Some(seed) = env_u64("URBAN_SEED") {
            crate::rng::seed(seed);
        }
        Some(Replay {
            steps,
            cur: 0,
            frame: 0,
            stop: env_u64("URBAN_FRAMES"),
            dump: env_u64("URBAN_DUMP"),
            dump_dir: std::env::var_os("URBAN_DUMP_DIR").map(PathBuf::from),
            trace: std::env::var_os("URBAN_TRACE_OUT").and_then(|p| std::fs::File::create(p).ok()).map(std::io::BufWriter::new),
        })
    }
}

pub fn set_replay_frame(f: u64) {
    with(|s| {
        if let Some(r) = s.replay.as_mut() {
            r.frame = f;
            r.cur = r.steps.iter().take_while(|st| st.0 <= f).count();
            if r.cur > 0 {
                s.keys = r.steps[r.cur - 1].1;
            }
        }
    });
}

pub fn replay_frame() -> u64 {
    with(|s| s.replay.as_ref().map(|r| r.frame).unwrap_or(0))
}

pub fn replaying() -> bool {
    with(|s| s.replay.is_some())
}

fn replay_tick(s: &mut Sys) {
    use std::io::Write;
    let Some(r) = s.replay.as_mut() else { return };
    let mut h: u64 = 1469598103934665603;
    for &c in &s.screen.data {
        h ^= c as u64;
        h = h.wrapping_mul(1099511628211);
    }
    for c in s.pal.iter() {
        h ^= c.r as u64 | (c.g as u64) << 8 | (c.b as u64) << 16;
        h = h.wrapping_mul(1099511628211);
    }
    if let Some(t) = r.trace.as_mut() {
        let _ = writeln!(t, "{} {:016x} {}", r.frame, h, crate::rng::calls());
    }
    if std::env::var("URBAN_SHOT_FRAME").ok().and_then(|v| v.parse::<u64>().ok()) == Some(r.frame) {
        if let Some(d) = &r.dump_dir {
            s.rgb_shot = Some(d.join(format!("final_{:06}.bmp", r.frame)));
        }
    }
    if Some(r.frame) == r.dump {
        if let Some(d) = &r.dump_dir {
            write_bmp(&d.join(format!("rust_{:06}.bmp", r.frame)), &s.screen, &s.pal);
        }
    }
    r.frame += 1;
    while r.cur < r.steps.len() && r.steps[r.cur].0 <= r.frame {
        s.keys = r.steps[r.cur].1;
        r.cur += 1;
    }
    if let Some(stop) = r.stop {
        if r.frame >= stop {
            if let Some(t) = r.trace.as_mut() {
                let _ = t.flush();
            }
            std::process::exit(0);
        }
    }
}

thread_local! {
    static SYS: RefCell<Option<Sys>> = const { RefCell::new(None) };
}

fn with<R>(f: impl FnOnce(&mut Sys) -> R) -> R {
    SYS.with(|s| f(s.borrow_mut().as_mut().expect("sys::init not called")))
}

/// Opens the window and audio. Must be called once before anything else in this module.
pub fn init() {
    let sdl = sdl2::init().expect("SDL init");
    let video = sdl.video().expect("SDL video");
    let timer = sdl.timer().expect("SDL timer");
    let controller_sys = sdl.game_controller().expect("SDL gamecontroller");
    sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "0");
    let headless = std::env::var_os("URBAN_HEADLESS").is_some();
    let display = if headless { None } else { Some(open_display(&video)) };
    sdl.mouse().show_cursor(false);
    let events = sdl.event_pump().expect("SDL events");

    let mixer = Arc::new(Mutex::new(MixState::default()));
    let audio = sdl.audio().ok().and_then(|a| {
        let spec = AudioSpecDesired { freq: Some(MIX_RATE), channels: Some(2), samples: Some(1024) };
        let m = mixer.clone();
        match a.open_playback(None, &spec, |_| Mixer { state: m }) {
            Ok(dev) => {
                dev.resume();
                Some(dev)
            }
            Err(e) => {
                eprintln!("Could not initialize sound: {}", e);
                None
            }
        }
    });
    if audio.is_some() {
        mixer.lock().unwrap().enabled = true;
    }

    let sys = Sys {
        _sdl: sdl,
        _video: video,
        timer,
        events,
        controller_sys,
        pad: None,
        display,
        screen: Bitmap::new(SCREEN_W, SCREEN_H),
        pal: BLACK_PALETTE,
        pal32: [0xff000000; 256],
        video: crate::video::VideoConf::load(),
        crt: None,
        buf_a: Vec::new(),
        buf_b: Vec::new(),
        buf_c: Vec::new(),
        rgb_shot: None,
        keys: [false; 128],
        keybuf: VecDeque::new(),
        joy: Joy::default(),
        last_present: Instant::now(),
        next_frame: None,
        shot_dir: std::env::var_os("URBAN_SHOTS").map(PathBuf::from),
        last_shot: Instant::now(),
        shot_n: 0,
        autoplay: std::env::var_os("URBAN_AUTOPLAY").is_some(),
        start: Instant::now(),
        last_auto_tick: u64::MAX,
        audio,
        mixer,
        replay: Replay::load()
            .or_else(|| std::env::var_os("URBAN_MENU_KEYS").map(|_| Replay::empty()))
            .or_else(|| std::env::var_os("URBAN_KEYPRESSES").map(|_| Replay::empty())),
        menu_keys: MenuKeys::load(),
    };
    SYS.with(|s| *s.borrow_mut() = Some(sys));
    open_pad();
}

fn open_pad() {
    with(|s| {
        if s.pad.is_some() {
            return;
        }
        let n = s.controller_sys.num_joysticks().unwrap_or(0);
        for i in 0..n {
            if s.controller_sys.is_game_controller(i) {
                if let Ok(p) = s.controller_sys.open(i) {
                    s.pad = Some(p);
                    break;
                }
            }
        }
    });
}

pub fn num_joysticks() -> i32 {
    with(|s| s.pad.is_some() as i32)
}

pub fn joy() -> Joy {
    with(|s| s.joy)
}

// ---------------------------------------------------------------------------
// Screen and palette

/// Runs `f` with mutable access to the visible 320x240 screen bitmap.
pub fn with_screen<R>(f: impl FnOnce(&mut Bitmap) -> R) -> R {
    with(|s| f(&mut s.screen))
}

/// Copies a full frame to the screen and shows it (the game's `blit(buffer, screen, ...)`).
pub fn show(buf: &Bitmap, dx: i32, dy: i32) {
    // A shaken frame is blitted over the previous one without clearing, like the original.
    with(|s| gfx::blit(buf, &mut s.screen, 0, 0, dx, dy, buf.w, buf.h));
    pump();
    present(true);
}

/// `blit(src, screen, ...)` followed by a present.
pub fn blit_to_screen(src: &Bitmap, sx: i32, sy: i32, dx: i32, dy: i32, w: i32, h: i32) {
    with(|s| gfx::blit(src, &mut s.screen, sx, sy, dx, dy, w, h));
    pump();
    present(true);
}

pub fn masked_blit_to_screen(src: &Bitmap, sx: i32, sy: i32, dx: i32, dy: i32, w: i32, h: i32) {
    with(|s| gfx::masked_blit(src, &mut s.screen, sx, sy, dx, dy, w, h));
    present(false);
}

pub fn screen_copy() -> Bitmap {
    with(|s| s.screen.clone())
}

pub fn set_color(i: usize, c: Rgb) {
    with(|s| {
        s.pal[i] = c;
        let r = (c.r as u32 & 63) * 255 / 63;
        let g = (c.g as u32 & 63) * 255 / 63;
        let b = (c.b as u32 & 63) * 255 / 63;
        s.pal32[i] = 0xff000000 | (r << 16) | (g << 8) | b;
    });
}

pub fn set_palette(p: &Palette) {
    for (i, c) in p.iter().enumerate() {
        set_color(i, *c);
    }
    present(true);
}

pub fn get_palette() -> Palette {
    with(|s| s.pal)
}

pub fn fade_from(src: &Palette, dst: &Palette, speed: i32) {
    let speed = speed.max(1);
    let mut tmp = BLACK_PALETTE;
    let mut c = 0;
    while c < 64 {
        gfx::fade_interpolate(src, dst, &mut tmp, c, 0, 255);
        set_palette(&tmp);
        pump();
        std::thread::sleep(Duration::from_millis(14));
        c += speed;
    }
    set_palette(dst);
}

pub fn fade_in(p: &Palette, speed: i32) {
    fade_from(&BLACK_PALETTE, p, speed);
}

pub fn fade_out(speed: i32) {
    let now = get_palette();
    fade_from(&now, &BLACK_PALETTE, speed);
}

/// Uploads the 8-bit screen through the palette and presents it.
/// `force == false` throttles to roughly 120 Hz for code that draws straight to the screen.
pub fn present(force: bool) {
    with(|s| {
        let now = Instant::now();
        if !force && now.duration_since(s.last_present) < Duration::from_millis(8) {
            return;
        }
        s.last_present = now;
        if s.display.is_some() || s.rgb_shot.is_some() {
            render(s);
        }
        if let Some(dir) = &s.shot_dir {
            if now.duration_since(s.last_shot) >= Duration::from_secs(1) {
                s.last_shot = now;
                let path = dir.join(format!("shot{:04}.bmp", s.shot_n));
                s.shot_n += 1;
                write_bmp(&path, &s.screen, &s.pal);
            }
        }
    });
}

/// Palette lookup, optional upscaling filter and CRT pass, then upload and present.
/// This is the single place where the 8-bit game frame becomes pixels on screen.
fn render(s: &mut Sys) {
    use crate::video::{self, Filter};
    let (w, h) = (s.screen.w as usize, s.screen.h as usize);
    s.buf_a.resize(w * h, 0);
    for (o, &c) in s.buf_a.iter_mut().zip(s.screen.data.iter()) {
        *o = s.pal32[c as usize];
    }
    let v = s.video;
    let n = v.scale();
    let (ow, oh) = (w * n, h * n);
    let mut out: &[u32] = &s.buf_a;
    if n > 1 {
        s.buf_b.resize(ow * oh, 0);
        match v.filter {
            Filter::Hqx => hqx::hq4x(&s.buf_a, &mut s.buf_b, w as u32, h as u32),
            Filter::ScaleX => video::scale3x(&s.buf_a, w, h, &mut s.buf_b),
            _ => video::nearest(&s.buf_a, w, h, n, &mut s.buf_b),
        }
        out = &s.buf_b;
    }
    if v.crt {
        if !s.crt.as_ref().map(|c| c.fits(ow, oh, n)).unwrap_or(false) {
            s.crt = Some(video::Crt::new(ow, oh, n));
        }
        s.buf_c.resize(ow * oh, 0);
        s.crt.as_ref().unwrap().apply(out, &mut s.buf_c);
        out = &s.buf_c;
    }
    let linear = matches!(v.filter, Filter::Smooth | Filter::Hqx) || v.crt;
    if let Some(path) = s.rgb_shot.take() {
        write_bmp32(&path, out, ow, oh);
    }
    let Some(d) = s.display.as_mut() else { return };
    if d.tex_w != ow as u32 || d.tex_h != oh as u32 || d.linear != linear {
        if let Ok(mut t) = d.creator.create_texture_streaming(PixelFormatEnum::ARGB8888, ow as u32, oh as u32) {
            t.set_scale_mode(if linear { sdl2::render::ScaleMode::Linear } else { sdl2::render::ScaleMode::Nearest });
            d.texture = t;
            d.tex_w = ow as u32;
            d.tex_h = oh as u32;
            d.linear = linear;
        }
    }
    let (lw, lh) = d.canvas.logical_size();
    if lw != w as u32 || lh != h as u32 {
        d.canvas.set_logical_size(w as u32, h as u32).ok();
    }
    d.canvas.set_integer_scale(!linear).ok();
    d.texture.update(None, bytemuck::cast_slice(out), ow * 4).ok();
    d.canvas.clear();
    d.canvas.copy(&d.texture, None, None).ok();
    d.canvas.present();
}

/// Current display settings.
pub fn video() -> crate::video::VideoConf {
    with(|s| s.video)
}

/// Applies and saves display settings.
pub fn set_video(c: crate::video::VideoConf) {
    with(|s| {
        if let Some(d) = s.display.as_mut() {
            let w = d.canvas.window_mut();
            let want = if c.fullscreen { FullscreenType::Desktop } else { FullscreenType::Off };
            if w.fullscreen_state() != want {
                let _ = w.set_fullscreen(want);
            }
        }
        s.video = c;
    });
    c.save();
    present(true);
}

/// Width the game view should use: 427 in widescreen, else the original 320.
/// Replay tests always use the original width.
pub fn gameplay_width() -> i32 {
    let tests_allow = std::env::var_os("URBAN_TEST_VIDEO").is_some();
    with(|s| if s.video.widescreen && (s.replay.is_none() || tests_allow) { crate::video::WIDE_W } else { SCREEN_W })
}

/// Resizes the visible screen (menus use 320, widescreen gameplay 427).
pub fn set_screen_width(w: i32) {
    with(|s| {
        if s.screen.w != w {
            s.screen = Bitmap::new(w, SCREEN_H);
        }
    });
}

pub fn screen_width() -> i32 {
    with(|s| s.screen.w)
}

/// Requests that the next presented frame is saved after filters/CRT (24-bit BMP).
pub fn capture_next_frame(path: PathBuf) {
    with(|s| s.rgb_shot = Some(path));
}

fn write_bmp32(path: &std::path::Path, px: &[u32], w: usize, h: usize) {
    let row = (w * 3 + 3) & !3;
    let size = 54 + row * h;
    let mut out = Vec::with_capacity(size);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(size as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(w as i32).to_le_bytes());
    out.extend_from_slice(&(h as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&[0u8; 24]);
    for y in (0..h).rev() {
        for &c in &px[y * w..(y + 1) * w] {
            out.extend_from_slice(&[c as u8, (c >> 8) as u8, (c >> 16) as u8]);
        }
        out.resize(out.len() + row - w * 3, 0);
    }
    let _ = std::fs::write(path, out);
}

fn write_bmp(path: &std::path::Path, b: &Bitmap, pal: &Palette) {
    let row = ((b.w + 3) & !3) as usize;
    let size = 54 + 1024 + row * b.h as usize;
    let mut out = Vec::with_capacity(size);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(size as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&(54u32 + 1024).to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&b.w.to_le_bytes());
    out.extend_from_slice(&b.h.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&8u16.to_le_bytes());
    out.extend_from_slice(&[0u8; 24]);
    for c in pal.iter() {
        out.extend_from_slice(&[c.b << 2, c.g << 2, c.r << 2, 0]);
    }
    for y in (0..b.h).rev() {
        let start = (y * b.w) as usize;
        out.extend_from_slice(&b.data[start..start + b.w as usize]);
        out.resize(out.len() + row - b.w as usize, 0);
    }
    let _ = std::fs::write(path, out);
}

// ---------------------------------------------------------------------------
// Input

fn sdl_to_allegro(s: Scancode) -> usize {
    use Scancode::*;
    (match s {
        Escape => KEY_ESC, Num1 => KEY_1, Num2 => KEY_2, Num3 => KEY_3, Num4 => KEY_4, Num5 => KEY_5,
        Num6 => KEY_6, Num7 => KEY_7, Num8 => KEY_8, Num9 => KEY_9, Num0 => KEY_0, Minus => KEY_MINUS,
        Equals => KEY_EQUALS, Backspace => KEY_BACKSPACE, Tab => KEY_TAB, Q => KEY_Q, W => KEY_W,
        E => KEY_E, R => KEY_R, T => KEY_T, Y => KEY_Y, U => KEY_U, I => KEY_I, O => KEY_O, P => KEY_P,
        LeftBracket => KEY_OPENBRACE, RightBracket => KEY_CLOSEBRACE, Return | KpEnter => KEY_ENTER,
        LCtrl => KEY_LCONTROL, A => KEY_A, S => KEY_S, D => KEY_D, F => KEY_F, G => KEY_G, H => KEY_H,
        J => KEY_J, K => KEY_K, L => KEY_L, Semicolon => KEY_COLON, Apostrophe => KEY_QUOTE,
        Grave => KEY_TILDE, LShift => KEY_LSHIFT, Backslash => KEY_BACKSLASH, Z => KEY_Z, X => KEY_X,
        C => KEY_C, V => KEY_V, B => KEY_B, N => KEY_N, M => KEY_M, Comma => KEY_COMMA,
        Period => KEY_STOP, Slash => KEY_SLASH, RShift => KEY_RSHIFT, KpMultiply => KEY_ASTERISK,
        LAlt => KEY_ALT, Space => KEY_SPACE, CapsLock => KEY_CAPSLOCK, F1 => KEY_F1, F2 => KEY_F2,
        F3 => KEY_F3, F4 => KEY_F4, F5 => KEY_F5, F6 => KEY_F6, F7 => KEY_F7, F8 => KEY_F8,
        F9 => KEY_F9, F10 => KEY_F10, NumLockClear => KEY_NUMLOCK, ScrollLock => KEY_SCRLOCK,
        Home => KEY_HOME, Up => KEY_UP, PageUp => KEY_PGUP, KpMinus => KEY_MINUS_PAD, Left => KEY_LEFT,
        Kp5 => KEY_5_PAD, Right => KEY_RIGHT, KpPlus => KEY_PLUS_PAD, End => KEY_END, Down => KEY_DOWN,
        PageDown => KEY_PGDN, Insert => KEY_INSERT, Delete => KEY_DEL, PrintScreen => KEY_PRTSCR,
        F11 => KEY_F11, F12 => KEY_F12, LGui => KEY_LWIN, RGui => KEY_RWIN, Application => KEY_MENU,
        RCtrl => KEY_RCONTROL, RAlt => KEY_ALTGR, KpDivide => KEY_SLASH2, Pause => KEY_PAUSE,
        _ => 0,
    }) as usize
}

fn ascii_for(k: Keycode, m: Mod) -> i32 {
    let code = k.into_i32();
    if (b'a' as i32..=b'z' as i32).contains(&code) {
        if m.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD | Mod::CAPSMOD) {
            return code - 32;
        }
        return code;
    }
    match k {
        Keycode::Return | Keycode::KpEnter => 13,
        Keycode::Backspace => 8,
        Keycode::Escape => 27,
        _ if (32..127).contains(&code) => code,
        _ => 0,
    }
}

enum VideoKey {
    Fullscreen,
    Filter,
    Crt,
    Widescreen,
}

/// Processes window/input events. Called implicitly by every waiting function.
/// F6 cycles the filter, F7 toggles CRT, F8 widescreen, Alt/Cmd+Enter fullscreen.
pub fn pump() {
    let mut reopen = false;
    let mut video_change: Option<VideoKey> = None;
    let quit = with(|s| {
        let mut quit = false;
        let evs: Vec<Event> = s.events.poll_iter().collect();
        for e in evs {
            match e {
                Event::Quit { .. } => quit = true,
                Event::KeyDown { .. } | Event::KeyUp { .. } if s.replay.is_some() => {}
                Event::KeyDown { scancode: Some(sc), keycode, keymod, repeat, .. } => {
                    if sc == Scancode::Return && keymod.intersects(Mod::LALTMOD | Mod::RALTMOD | Mod::LGUIMOD | Mod::RGUIMOD) {
                        if !repeat {
                            video_change = Some(VideoKey::Fullscreen);
                        }
                        continue;
                    }
                    if !repeat {
                        match sc {
                            Scancode::F6 => video_change = Some(VideoKey::Filter),
                            Scancode::F7 => video_change = Some(VideoKey::Crt),
                            Scancode::F8 => video_change = Some(VideoKey::Widescreen),
                            _ => {}
                        }
                    }
                    let k = sdl_to_allegro(sc);
                    if k != 0 {
                        s.keys[k] = true;
                        if s.keybuf.len() < 64 {
                            let a = keycode.map(|kc| ascii_for(kc, keymod)).unwrap_or(0);
                            s.keybuf.push_back(((k as i32) << 8) | a);
                        }
                    }
                }
                Event::KeyUp { scancode: Some(sc), .. } => {
                    let k = sdl_to_allegro(sc);
                    if k != 0 {
                        s.keys[k] = false;
                    }
                }
                Event::ControllerDeviceAdded { .. } => reopen = true,
                Event::ControllerDeviceRemoved { .. } => {
                    if s.pad.as_ref().map(|p| !p.attached()).unwrap_or(false) {
                        s.pad = None;
                        reopen = true;
                    }
                }
                _ => {}
            }
        }
        if let Some(p) = &s.pad {
            const DEAD: i16 = 12000;
            let ax = p.axis(Axis::LeftX);
            let ay = p.axis(Axis::LeftY);
            s.joy = Joy {
                left: ax < -DEAD || p.button(Button::DPadLeft),
                right: ax > DEAD || p.button(Button::DPadRight),
                up: ay < -DEAD || p.button(Button::DPadUp),
                down: ay > DEAD || p.button(Button::DPadDown),
                b: [
                    p.button(Button::A),
                    p.button(Button::B),
                    p.button(Button::X),
                    p.button(Button::Y),
                    p.button(Button::LeftShoulder),
                    p.button(Button::RightShoulder),
                    p.button(Button::Back),
                    p.button(Button::Start),
                ],
            };
        } else {
            s.joy = Joy::default();
        }
        if s.autoplay {
            autoplay(s);
        }
        quit
    });
    if reopen {
        open_pad();
    }
    if let Some(k) = video_change {
        let mut c = video();
        match k {
            VideoKey::Fullscreen => c.fullscreen = !c.fullscreen,
            VideoKey::Filter => c.filter = c.filter.next(),
            VideoKey::Crt => c.crt = !c.crt,
            VideoKey::Widescreen => c.widescreen = !c.widescreen,
        }
        set_video(c);
    }
    if quit {
        std::process::exit(0);
    }
}

/// Test hook: presses Enter every 3 s, holds right, pulses fire, jumps occasionally.
fn autoplay(s: &mut Sys) {
    let t = s.start.elapsed().as_millis() as u64 / 100;
    let phase = t % 30;
    if phase == 0 && t != s.last_auto_tick {
        s.keybuf.push_back(((KEY_ENTER as i32) << 8) | 13);
        s.keys[KEY_ENTER] = true;
    } else if phase == 1 {
        s.keys[KEY_ENTER] = false;
    }
    s.keys[KEY_RIGHT] = true;
    s.keys[KEY_LCONTROL] = phase % 4 < 2;
    s.keys[KEY_SPACE] = phase == 10;
    s.last_auto_tick = t;
}

pub fn key(k: usize) -> bool {
    with(|s| s.keys.get(k).copied().unwrap_or(false))
}

pub fn set_key(k: usize, v: bool) {
    with(|s| s.keys[k] = v);
}

pub fn keyboard_reset() {
    with(|s| s.keys = [false; 128]);
}

/// Sequence test (URBAN_KEYPRESSES="1,5"): keypressed() is true only on those call numbers,
/// rest() fingerprints the screen, and the first readkey() ends the run (mirrors the C++ shim).
struct Seq {
    keypressed_calls: u64,
    rest_calls: u64,
    want: Vec<u64>,
    trace: Option<std::fs::File>,
}

thread_local! {
    static SEQ: RefCell<Option<Seq>> = RefCell::new(std::env::var("URBAN_KEYPRESSES").ok().map(|v| Seq {
        keypressed_calls: 0,
        rest_calls: 0,
        want: v.split(',').filter_map(|t| t.trim().parse().ok()).collect(),
        trace: std::env::var_os("URBAN_SEQ_TRACE").and_then(|p| std::fs::File::create(p).ok()),
    }));
}

fn seq_active() -> bool {
    SEQ.with(|q| q.borrow().is_some())
}

fn seq_rest_tick() {
    use std::io::Write;
    if !seq_active() {
        return;
    }
    let h = current_screen_hash();
    SEQ.with(|q| {
        let mut q = q.borrow_mut();
        let q = q.as_mut().unwrap();
        if let Some(t) = q.trace.as_mut() {
            let _ = writeln!(t, "{} {:016x}", q.rest_calls, h);
        }
        if std::env::var("URBAN_DUMP").ok().and_then(|v| v.parse::<u64>().ok()) == Some(q.rest_calls) {
            if let Some(d) = std::env::var_os("URBAN_DUMP_DIR") {
                dump_screen(&PathBuf::from(d).join(format!("rust_{:06}.bmp", q.rest_calls)));
            }
        }
        q.rest_calls += 1;
    });
}

pub fn keypressed() -> bool {
    present(false);
    pump();
    if let Some(r) = SEQ.with(|q| {
        q.borrow_mut().as_mut().map(|q| {
            q.keypressed_calls += 1;
            q.want.contains(&q.keypressed_calls)
        })
    }) {
        return r;
    }
    with(|s| s.menu_keys.is_some() || !s.keybuf.is_empty())
}

/// Blocks for a key; returns `(scancode << 8) | ascii` like Allegro.
pub fn readkey() -> i32 {
    if seq_active() {
        std::process::exit(0);
    }
    if let Some(k) = with(|s| {
        use std::io::Write;
        let h = screen_hash(s);
        let m = s.menu_keys.as_mut()?;
        if let Some(t) = m.trace.as_mut() {
            let _ = writeln!(t, "{} {:016x}", m.cur, h);
        }
        if m.dump == Some(m.cur) {
            if let Some(d) = &m.dump_dir {
                write_bmp(&d.join(format!("rust_{:06}.bmp", m.cur)), &s.screen, &s.pal);
            }
        }
        if m.cur >= m.keys.len() {
            std::process::exit(0);
        }
        m.cur += 1;
        Some(m.keys[m.cur - 1])
    }) {
        return k;
    }
    while !keypressed() {
        std::thread::sleep(Duration::from_millis(5));
    }
    with(|s| s.keybuf.pop_front().unwrap_or(0))
}

pub fn clear_keybuf() {
    pump();
    with(|s| s.keybuf.clear());
}

pub fn simulate_keypress(k: i32) {
    with(|s| s.keybuf.push_back(k));
}

// ---------------------------------------------------------------------------
// Timing

pub fn rest(ms: u64) {
    seq_rest_tick();
    present(false);
    pump();
    if !replaying() {
        std::thread::sleep(Duration::from_millis(ms));
    }
}

pub fn vsync() {
    present(true);
    pump();
}

/// Sleeps until the next frame boundary at `fps` (replaces the original 60 Hz timer interrupt).
pub fn wait_frame(fps: u32) {
    if replaying() {
        with(replay_tick);
        return;
    }
    let step = Duration::from_nanos(1_000_000_000 / fps as u64);
    let next = with(|s| {
        let now = Instant::now();
        let mut next = s.next_frame.unwrap_or(now);
        if now > next + step * 4 {
            next = now;
        }
        next += step;
        s.next_frame = Some(next);
        next
    });
    loop {
        let now = Instant::now();
        if now >= next {
            break;
        }
        let left = next - now;
        if left > Duration::from_millis(2) {
            std::thread::sleep(left - Duration::from_millis(1));
        } else {
            std::hint::spin_loop();
        }
    }
}

pub fn ticks_ms() -> u64 {
    with(|s| s.timer.ticks64())
}

// ---------------------------------------------------------------------------
// Paths

/// Per-user writable directory for config, saves and highscores.
pub fn user_dir() -> PathBuf {
    let p = match std::env::var_os("URBAN_USER_DIR") {
        Some(d) => PathBuf::from(d),
        None => sdl2::filesystem::pref_path("", "urban").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from(".")),
    };
    let _ = std::fs::create_dir_all(&p);
    p
}

/// Directory holding urban.dat, levels.dat, intro.dat and snd/.
pub fn data_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("URBAN_DATA") {
        return PathBuf::from(d);
    }
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("data"));
            for up in dir.ancestors().take(5) {
                candidates.push(up.join("ref").join("data"));
            }
        }
    }
    candidates.push(PathBuf::from("data"));
    candidates.push(PathBuf::from("../ref/data"));
    candidates.into_iter().find(|d| d.join("urban.dat").exists()).unwrap_or_else(|| PathBuf::from("data"))
}

// ---------------------------------------------------------------------------
// Audio

pub struct Sample {
    pub data: Vec<i16>,
    pub rate: u32,
}

struct Voice {
    sample: Arc<Sample>,
    pos: u64,
    step: u64,
    vol: i32,
    critical: bool,
    started: u64,
}

#[derive(Default)]
struct MixState {
    enabled: bool,
    music: Option<xmrsplayer::xmrsplayer::XmrsPlayer<'static>>,
    music_vol: i32,
    voices: Vec<Voice>,
    counter: u64,
}

struct Mixer {
    state: Arc<Mutex<MixState>>,
}

impl AudioCallback for Mixer {
    type Channel = i16;

    fn callback(&mut self, out: &mut [i16]) {
        let mut st = self.state.lock().unwrap();
        let mvol = st.music_vol;
        for frame in out.chunks_mut(2) {
            let (mut l, mut r) = (0i32, 0i32);
            if let Some(p) = st.music.as_mut() {
                let a = p.next().unwrap_or(0) as i32;
                let b = p.next().unwrap_or(0) as i32;
                l += a * mvol / 128;
                r += b * mvol / 128;
            }
            for v in st.voices.iter_mut() {
                let idx = (v.pos >> 16) as usize;
                if idx < v.sample.data.len() {
                    let s = v.sample.data[idx] as i32 * v.vol / 255;
                    l += s;
                    r += s;
                    v.pos += v.step;
                }
            }
            frame[0] = l.clamp(-32768, 32767) as i16;
            if frame.len() > 1 {
                frame[1] = r.clamp(-32768, 32767) as i16;
            }
        }
        st.voices.retain(|v| ((v.pos >> 16) as usize) < v.sample.data.len());
    }
}

pub fn audio_enabled() -> bool {
    with(|s| s.audio.is_some())
}

/// Starts looping music; `module` must live for the rest of the program (callers cache it).
pub fn play_music(module: &'static xmrs::prelude::Module) {
    with(|s| {
        let mut p = xmrsplayer::xmrsplayer::XmrsPlayer::new(module, MIX_RATE as u32, 0);
        p.set_max_loop_count(0);
        s.mixer.lock().unwrap().music = Some(p);
    });
}

pub fn stop_music() {
    with(|s| s.mixer.lock().unwrap().music = None);
}

/// Music volume on the original 0..=255 config scale.
pub fn set_music_volume(v: i32) {
    with(|s| s.mixer.lock().unwrap().music_vol = v.clamp(0, 255) / 2);
}

pub fn music_volume() -> i32 {
    with(|s| s.mixer.lock().unwrap().music_vol * 2)
}

/// Plays a sound effect on one of four voices (MikMod_SetNumVoices(12, 4) in the original).
/// Non-critical sounds steal the oldest non-critical voice; critical ones are never stolen.
pub fn play_sample(sample: &Arc<Sample>, vol: i32, critical: bool) {
    with(|s| {
        let mut st = s.mixer.lock().unwrap();
        st.counter += 1;
        let v = Voice {
            sample: sample.clone(),
            pos: 0,
            step: ((sample.rate as u64) << 16) / MIX_RATE as u64,
            vol: vol.clamp(0, 255),
            critical,
            started: st.counter,
        };
        if st.voices.len() < SFX_VOICES {
            st.voices.push(v);
            return;
        }
        let victim = st
            .voices
            .iter()
            .enumerate()
            .filter(|(_, x)| !x.critical)
            .min_by_key(|(_, x)| x.started)
            .map(|(i, _)| i);
        if let Some(i) = victim {
            st.voices[i] = v;
        }
    });
}
