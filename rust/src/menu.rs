//! Main menu, load game and the front-end main loop (meny/meny.cc).

use crate::engine::{Game, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::font::{UrbanFont, LARGE_FONT, SMALL_FONT};
use crate::keys::*;
use crate::{config_ui, credits, flags, game, gfx, highscore, icache, intro, sound, sys};

pub const MENU_MUSIC: &str = "snd/modules/nole.xm";
const CREDITS_MUSIC: &str = "snd/modules/extrem.xm";
const FONT_H: i32 = 26;

/// SOUND.PlayMusic: starts a module at the configured music volume.
pub fn play_music(path: &str) {
    sound::play_music(path);
    sound::set_music_volume(crate::config::get().music_vol);
}

fn menu_music() {
    if flags::sound_on() {
        play_music(MENU_MUSIC);
    }
}

pub fn blit_full(bmp: &gfx::Bitmap) {
    sys::with_screen(|s| gfx::blit(bmp, s, 0, 0, 0, 0, bmp.w, bmp.h));
}

/// Shows `text` centred over the menu background with a pointer; returns the 1-based choice,
/// or 0 on ESC.
pub fn do_menu(text: &str, num_items: i32, mut pos: i32) -> i32 {
    let mut m = UrbanFont::new(LARGE_FONT);
    m.set_scale(65);
    let choice = icache::get("pekare.pcx");
    let (backg, pal) = icache::get_pal("ibild.pcx");
    sys::set_palette(&pal);
    let textbmp = m.render(text);
    let text_x = (SCREEN_WIDTH - textbmp.w) / 2;
    let text_y = (SCREEN_HEIGHT - textbmp.h) / 2;

    sys::with_screen(|s| {
        gfx::blit(&backg, s, 0, 0, 0, 0, backg.w, backg.h);
        gfx::masked_blit(&textbmp, s, 0, 0, text_x, text_y, textbmp.w, textbmp.h);
    });
    sys::clear_keybuf();

    let px = text_x - 10 - choice.w;
    loop {
        sys::with_screen(|s| {
            gfx::blit(&backg, s, px, text_y - 20, px, text_y - 20, choice.w, choice.h * 10);
            gfx::masked_blit(&choice, s, 0, 0, px, text_y + FONT_H * (pos - 1), choice.w, choice.h);
        });
        sys::present(true);
        match (sys::readkey() >> 8) as usize {
            KEY_ENTER | KEY_SPACE => break,
            KEY_ESC => {
                pos = 0;
                break;
            }
            KEY_UP => pos = if pos > 1 { pos - 1 } else { num_items },
            KEY_DOWN => pos = if pos < num_items { pos + 1 } else { 1 },
            _ => {}
        }
    }
    sys::clear_keybuf();
    pos
}

fn load_game(g: &mut Game) {
    let slots = game::load_slots();
    let slot = do_menu(&game::slot_menu_text(&slots), 5, 1);
    if slot > 0 && slots[(slot - 1) as usize].level != 0 {
        game::new_game(g, slot as usize);
    }
}

fn menu_choice(g: &mut Game, alternative: i32) -> bool {
    match alternative {
        1 => {
            game::new_game(g, 0);
            menu_music();
        }
        2 => {
            load_game(g);
            menu_music();
            sys::clear_keybuf();
        }
        3 => {
            highscore::show();
            sys::readkey();
        }
        4 => {
            if flags::sound_on() {
                play_music(CREDITS_MUSIC);
            }
            sys::fade_out(5);
            credits::show_credits();
            menu_music();
        }
        5 => config_ui::start(),
        6 => return false,
        _ => {}
    }
    true
}

/// The part of main() after the engine exists: intro, menu music, menu loop, homepage screen.
pub fn run(g: &mut Game) {
    let meny_text = "START GAME\nLOAD GAME\nHIGHSCORE\nCREDITS\nCONFIG\nQUIT";
    sys::set_palette(&gfx::BLACK_PALETTE);

    if flags::intro_on() {
        intro::run_intro();
    }
    sys::clear_keybuf();

    menu_music();

    let mut pos = 1;
    loop {
        pos = do_menu(meny_text, 6, pos);
        if pos == 0 || pos == 6 {
            break;
        }
        menu_choice(g, pos);
    }

    let l = UrbanFont::new(LARGE_FONT);
    let s = UrbanFont::new(SMALL_FONT);
    let backg = icache::get("ibild.pcx");
    blit_full(&backg);
    sys::present(true);
    sys::rest(500);
    l.print_centre_screen("THE URBAN", SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2 - 50);
    l.print_centre_screen("HOMEPAGE", SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2);
    s.print_centre_screen("urban.home.dhs.org", SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2 + 40);
    sys::present(true);
    let mut i = 600;
    while !sys::keypressed() && i > 0 {
        sys::rest(100);
        i -= 1;
    }
    sys::fade_out(6);
}
