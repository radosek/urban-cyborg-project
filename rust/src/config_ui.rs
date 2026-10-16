//! Config menus: controls, gamepad, music/SFX volume, graphics quality (config/config.cc).
//!
//! Keyboard and gamepad setup edit a copy of the config that is only committed once every
//! binding has been chosen, so ESC part-way leaves the old mapping intact.

use crate::config::{self, KeyConf, CONTROLLER_GAMEPAD2, CONTROLLER_GAMEPAD4, CONTROLLER_GAMEPAD6, CONTROLLER_GAMEPAD8, CONTROLLER_KEYBOARD};
use crate::ctrls::*;
use crate::engine::SCREEN_WIDTH;
use crate::font::{UrbanFont, LARGE_FONT, SMALL_FONT2};
use crate::keys::*;
use crate::menu::{blit_full, do_menu};
use crate::{icache, sound, sys};

const KEYINFO: &[(&str, usize)] = &[
    ("-", 12), ("=", 13), ("BACKSPACE", 14), ("TAB", 15), ("Q", 16), ("W", 17), ("E", 18), ("R", 19),
    ("T", 20), ("Y", 21), ("U", 22), ("I", 23), ("O", 24), ("P", 25), ("[", 26), ("]", 27),
    ("ENTER", 28), ("LCONTROL", 29), ("A", 30), ("S", 31), ("D", 32), ("F", 33), ("G", 34), ("H", 35),
    ("J", 36), ("K", 37), ("L", 38), (":", 39), ("\"", 40), ("~", 41), ("LSHIFT", 42), ("\\", 43),
    ("Z", 44), ("X", 45), ("C", 46), ("V", 47), ("B", 48), ("N", 49), ("M", 50), (",", 51),
    ("/", 53), ("RSHIFT", 54), ("*", 55), ("ALT", 56), ("SPACE", 57), ("HOME", 71), ("UP", 72),
    ("PGUP", 73), ("PAD -", 74), ("LEFT", 75), ("PAD 5", 76), ("RIGHT", 77), ("PAD +", 78),
    ("END", 79), ("DOWN", 80), ("PGDN", 81), ("INSERT", 82), ("DEL", 83), ("RCONTROL", 120),
    ("ALTGR", 121),
];

fn background() -> crate::icache::Img {
    let (bmp, pal) = icache::get_pal("ibild.pcx");
    sys::set_palette(&pal);
    bmp
}

/// Waits for a key in the KEYINFO table; any other key (ESC included) aborts with None.
fn get_key(label: &str) -> Option<i32> {
    let fnt = UrbanFont::new(SMALL_FONT2);
    fnt.print_centre_screen(&format!("Please press your key for '{}'", label), 320 / 2, 90);
    sys::present(true);
    sys::clear_keybuf();
    let temp = (sys::readkey() >> 8) as usize;
    let &(desc, _) = KEYINFO.iter().find(|k| k.1 == temp)?;
    fnt.print_centre_screen(desc, 320 / 2, 120);
    sys::present(true);
    while sys::key(temp) {
        sys::rest(5);
    }
    sys::rest(500);
    Some(temp as i32)
}

/// Waits for a gamepad direction or button; a keypress aborts with None.
fn get_button(label: &str) -> Option<i32> {
    let fnt = UrbanFont::new(SMALL_FONT2);
    fnt.print_centre_screen(&format!("Please press your button for '{}'", label), 320 / 2, 90);
    sys::present(true);
    loop {
        if sys::keypressed() {
            while sys::keypressed() {
                sys::readkey();
            }
            return None;
        }
        let j = sys::joy();
        let mut hit: Option<(String, i32, Box<dyn Fn(&sys::Joy) -> bool>)> = None;
        if j.left {
            hit = Some(("LEFT".into(), JOYSTICK_LEFT, Box::new(|j| j.left)));
        } else if j.right {
            hit = Some(("RIGHT".into(), JOYSTICK_RIGHT, Box::new(|j| j.right)));
        } else if j.up {
            hit = Some(("UP".into(), JOYSTICK_UP, Box::new(|j| j.up)));
        } else if j.down {
            hit = Some(("DOWN".into(), JOYSTICK_DOWN, Box::new(|j| j.down)));
        } else if let Some(b) = j.b.iter().position(|&b| b) {
            hit = Some((format!("BUTTON {}", b + 1), JOYSTICK_B1 + b as i32, Box::new(move |j| j.b[b])));
        }
        if let Some((name, code, held)) = hit {
            fnt.print_centre_screen(&name, 320 / 2, 120);
            sys::present(true);
            sys::rest(500);
            while held(&sys::joy()) {
                sys::rest(5);
            }
            return Some(code);
        }
        sys::rest(5);
    }
}

fn configure(get: fn(&str) -> Option<i32>, with_weapons: bool, c: &mut KeyConf) -> bool {
    let bmp = background();
    blit_full(&bmp);
    let mut slots: Vec<(&str, &mut i32)> = vec![
        ("UP", &mut c.key_up),
        ("DOWN", &mut c.key_down),
        ("LEFT", &mut c.key_left),
        ("RIGHT", &mut c.key_right),
        ("FIRE", &mut c.key_fire),
        ("JUMP", &mut c.key_jump),
    ];
    if with_weapons {
        slots.push(("Prev Weapon", &mut c.key_prevweapon));
        slots.push(("Next Weapon", &mut c.key_nextweapon));
    }
    for (label, field) in slots {
        match get(label) {
            Some(v) => *field = v,
            None => return false,
        }
        blit_full(&bmp);
    }
    sys::present(true);
    true
}

fn configure_keyboard() {
    let mut c = config::get();
    if configure(get_key, true, &mut c) {
        c.ctrl_type = CONTROLLER_KEYBOARD;
        config::set(c);
    }
}

fn configure_gamepad() {
    let bmp = background();
    let fnt = UrbanFont::new(SMALL_FONT2);
    blit_full(&bmp);
    fnt.print_centre_screen("Gamepad type:", 160, 100);
    fnt.print_centre_screen("  1. 2 buttons\n  2. 4 buttons\n  3. 6 buttons\n  4. 8 buttons", 160, 140);
    sys::present(true);

    let mut c = config::get();
    let choice;
    loop {
        let key = sys::readkey();
        if (key >> 8) as usize == KEY_ESC {
            return;
        }
        let ch = (key & 0xff) as u8;
        if (b'1'..=b'4').contains(&ch) {
            c.ctrl_type = match ch {
                b'1' => CONTROLLER_GAMEPAD2,
                b'2' => CONTROLLER_GAMEPAD4,
                b'3' => CONTROLLER_GAMEPAD6,
                _ => CONTROLLER_GAMEPAD8,
            };
            choice = ch;
            break;
        }
    }
    let with_weapons = choice > b'1';
    if !configure(get_button, with_weapons, &mut c) {
        return;
    }
    if !with_weapons {
        c.key_prevweapon = 0;
        c.key_nextweapon = 0;
    }
    config::set(c);
}

fn volume_screen(title: &str, initial: i32, mut apply: impl FnMut(i32)) {
    let backg = icache::get("ibild.pcx");
    let mut vol = initial;
    blit_full(&backg);
    let mut m = UrbanFont::new(LARGE_FONT);
    m.set_scale(65);
    m.print_centre_screen(title, SCREEN_WIDTH / 2, 50);
    let x0 = (SCREEN_WIDTH - 256) / 2;
    loop {
        sys::with_screen(|s| {
            crate::gfx::blit(&backg, s, 0, 100, 0, 100, backg.w, 23);
            s.rectfill(x0 + 2, 102, vol + x0 + 2, 122, 207);
            s.rectfill(x0, 100, vol + x0, 120, 192);
        });
        sys::vsync();
        match (sys::readkey() >> 8) as usize {
            KEY_LEFT | KEY_DOWN => {
                if vol > 1 {
                    vol -= 2;
                }
                apply(vol);
            }
            KEY_UP | KEY_RIGHT => {
                if vol < 254 {
                    vol += 2;
                }
                apply(vol);
            }
            _ => break,
        }
    }
    config::save();
}

fn change_music_vol() {
    volume_screen("MUSIC VOLUME", config::get().music_vol, |v| {
        sound::set_music_volume(v);
        let mut c = config::get();
        c.music_vol = v;
        config::set(c);
    });
}

fn change_sfx_vol() {
    volume_screen("SFX VOLUME", config::get().sfx_vol, |v| {
        let mut c = config::get();
        c.sfx_vol = v;
        config::set(c);
    });
}

fn change_gfx_quality() {
    let mut c = config::get();
    let tmp = do_menu("NORMAL\nHIGH", 2, c.gfx_quality);
    if tmp > 0 && tmp < 3 {
        c.gfx_quality = tmp;
        config::set(c);
        config::save();
    }
}

fn start_controls() {
    if sys::num_joysticks() == 0 {
        configure_keyboard();
        config::save();
        return;
    }
    match do_menu("KEYBOARD\nGAMEPAD\nBACK", 3, 1) {
        1 => configure_keyboard(),
        2 => configure_gamepad(),
        _ => return,
    }
    config::save();
}

/// Display upgrades: filter, CRT, widescreen, fullscreen. ENTER on a line changes it.
fn graphics_menu() {
    let mut pos = 1;
    loop {
        let v = sys::video();
        let on = |b: bool| if b { "ON" } else { "OFF" };
        let text = format!("{}\nCRT {}\nWIDE {}\nFULL {}\nBACK", v.filter.name(), on(v.crt), on(v.widescreen), on(v.fullscreen));
        pos = do_menu(&text, 5, pos);
        let mut c = v;
        match pos {
            1 => c.filter = c.filter.next(),
            2 => c.crt = !c.crt,
            3 => c.widescreen = !c.widescreen,
            4 => c.fullscreen = !c.fullscreen,
            _ => break,
        }
        sys::set_video(c);
    }
}

/// Config::Start
pub fn start() {
    let mut pos = 1;
    loop {
        pos = do_menu("CONTROLS\nGRAPHICS\nMUSIC VOL\nSFX VOL\nBACK", 5, pos);
        match pos {
            0 | 5 => break,
            1 => start_controls(),
            // Replay tests compare menus with the C++ original, which only had NORMAL/HIGH here.
            2 if sys::replaying() => change_gfx_quality(),
            2 => graphics_menu(),
            3 => change_music_vol(),
            4 => change_sfx_vol(),
            _ => {}
        }
    }
}
