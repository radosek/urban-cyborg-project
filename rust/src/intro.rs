//! Intro sequence (intro/intro.cc). Shared helpers are used by the outtro too.

use crate::dat::Datfile;
use crate::flc::play_memory_fli;
use crate::font::{UrbanFont, SMALL_FONT};
use crate::{flags, icache, sound, sys};

const FRAME_DELAY: u64 = 100;

pub struct Player {
    pub dat: Datfile,
    pub lg: UrbanFont,
    pub quit: bool,
    pub key_quits: bool,
}

impl Player {
    pub fn new(key_quits: bool) -> Player {
        let path = sys::data_dir().join("intro.dat");
        let dat = Datfile::open(&path).unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1)
        });
        Player { dat, lg: UrbanFont::new(SMALL_FONT), quit: false, key_quits }
    }

    /// PLAY_FLC: loops the animation while `text` types out at (20,170). Returns false to quit.
    pub fn play_flc(&mut self, file: &str, text: &str) -> bool {
        let Some(buf) = self.dat.get(file) else {
            eprintln!("Can't load \"{}\"", file);
            std::process::exit(1);
        };
        let num_frames = text.len();
        let mut count = 0usize;
        let lg = &self.lg;
        let key_quits = self.key_quits;
        let quit = &mut self.quit;
        play_memory_fli(buf, true, || {
            let shown = &text[..(count + 1).min(text.len())];
            lg.print_screen(shown, 20, 170);
            sys::rest(FRAME_DELAY);
            if key_quits && sys::keypressed() {
                *quit = true;
            }
            if count + 1 == num_frames || *quit {
                return true;
            }
            count += 1;
            false
        });
        if self.quit {
            sys::clear_keybuf();
            return false;
        }
        true
    }

    /// ANIM_TEXT: types `text` at (50,170). Returns false to quit.
    pub fn anim_text(&mut self, text: &str) -> bool {
        sys::clear_keybuf();
        for i in 0..text.len() {
            self.lg.print_screen(&text[..i + 1], 50, 170);
            sys::rest(FRAME_DELAY);
            if self.key_quits && sys::keypressed() {
                sys::clear_keybuf();
                return false;
            }
        }
        true
    }
}

pub fn display_image(name: &str) {
    let (bmp, pal) = icache::get_pal(name);
    sys::set_palette(&pal);
    sys::blit_to_screen(&bmp, 0, 0, 0, 0, 320, 240);
}

/// PLAY_MOD: the intro and outtro play at a fixed volume of 64.
pub fn play_mod(path: &str) {
    if flags::sound_on() {
        sound::play_music(path);
        sound::set_music_volume(64);
    }
}

pub fn run_intro() {
    let mut p = Player::new(true);

    play_mod("snd/modules/warning.mod");
    display_image("warning.pcx");
    while !sys::keypressed() {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    sys::fade_out(5);

    play_mod("snd/modules/blazer.xm");
    display_image("intro/1.pcx");
    sys::rest(2000);

    macro_rules! flc {
        ($f:expr, $t:expr) => {
            if !p.play_flc($f, $t) {
                return;
            }
        };
    }
    macro_rules! text {
        ($t:expr) => {
            if !p.anim_text($t) {
                return;
            }
        };
    }

    text!("Container terminal 3B\nLos Angeles Harbour, USA\n11:45 am... Present Day");
    sys::rest(1000);
    flc!("gfx/intro/2.flc", "                              ");
    flc!("gfx/intro/3.flc", "Zee lab und all our equipment\niz ready. Zee only thing we need now\niz ein object. ");
    sys::rest(1000);
    flc!("gfx/intro/4.flc", "Sure, I'll take care of it.\nDoes he need any special abilities?          ");
    flc!("gfx/intro/3.flc", "Of course, here you have eine list...     ");
    flc!("gfx/intro/5.flc", "                                   ");
    flc!("gfx/intro/6.flc", "Aah... I think I just found\nthe perfect victim!          ");
    sys::rest(1000);
    display_image("intro/id.pcx");
    sys::rest(2000);
    text!("Yes, He will be perfect!\n       He He He He!");
    sys::rest(1000);
    display_image("intro/1.pcx");
    sys::rest(3000);
    sys::fade_out(4);
    sys::rest(2000);
    display_image("intro/nevada.pcx");
    sys::rest(1000);
    text!("Three months later...\nLocation: Top secret military\nfacility for weapon development.\nSomewhere in the Nevada desert.");
    sys::rest(3000);
    flc!("gfx/intro/7.flc", "                      ");
    flc!("gfx/intro/8.flc", "This iz zee last stage\nin zee process.   ");
    sys::rest(1000);
    flc!("gfx/intro/9.flc", "Great! \nSo we can soon begin\nthe battle testing?   ");
    flc!("gfx/intro/8.flc", "Yes, in about zree days\nfrom now.    ");
    flc!("gfx/intro/7.flc", "Zhen you can play with it\nas much as you like.\n      \nPerfect....    ");
    flc!("gfx/intro/11.flc", "                             \nWHAT?!!      ");
    flc!("gfx/intro/12.flc", "He iz not suppozed to\nwake up now.     \nThiz can become eine...\n...grosse problem.");
    sys::rest(2000);
}
