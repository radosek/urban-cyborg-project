//! Level sequence, save slots and slot-name entry (engine/game.cc).

use crate::engine::{Game, PlayerData};
use crate::font::{UrbanFont, LARGE_FONT, SMALL_FONT2};
use crate::gfx::Bitmap;
use crate::keys::*;
use crate::menu::do_menu;
use crate::{config, credits, highscore, icache, outtro, sound, sys};

pub const NUM_LEVELS: i32 = 18;
pub const NUM_SLOTS: usize = 5;
const NAME_MAX: usize = 31;

pub const MAPS: [&str; NUM_LEVELS as usize] = [
    "gamemaps/level11.map",
    "gamemaps/level12.map",
    "gamemaps/level13.map",
    "gamemaps/level21.map",
    "gamemaps/level22.map",
    "gamemaps/level23.map",
    "gamemaps/level31.map",
    "gamemaps/level32.map",
    "gamemaps/level33.map",
    "gamemaps/level41.map",
    "gamemaps/level42.map",
    "gamemaps/level43.map",
    "gamemaps/level51.map",
    "gamemaps/level52.map",
    "gamemaps/level53.map",
    "gamemaps/level61.map",
    "gamemaps/level62.map",
    "gamemaps/level63.map",
];

/// No level has an extra map; looked up with `get`, so any level falls back to "".
const EXTRA_MAPS: [&str; 6] = ["", "", "", "", "", ""];

pub const DEFAULT_PDATA: PlayerData = PlayerData {
    num_lives: 3,
    weapon_ammo: [-1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    weapon_avail: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    health: 300,
    score: 0,
};

#[derive(Clone)]
pub struct SaveGameData {
    pub level: i32,
    pub name: String,
    pub dat: PlayerData,
}

impl Default for SaveGameData {
    fn default() -> Self {
        SaveGameData { level: 0, name: String::new(), dat: DEFAULT_PDATA }
    }
}

pub type Slots = [SaveGameData; NUM_SLOTS];

// savegame.dat: "URBS", u32 version, then per slot: i32 level, u8 name length,
// NAME_MAX name bytes (zero padded), i32 lives, i32 ammo[10], i32 avail[10], i32 health, i32 score.
// All integers little-endian.
const SAVE_MAGIC: &[u8; 4] = b"URBS";
const SAVE_VERSION: u32 = 1;
const SLOT_BYTES: usize = 4 + 1 + NAME_MAX + 4 * (1 + 10 + 10 + 2);

fn save_path() -> std::path::PathBuf {
    sys::user_dir().join("savegames.v1")
}

fn encode(slots: &Slots) -> Vec<u8> {
    let mut b = Vec::with_capacity(8 + NUM_SLOTS * SLOT_BYTES);
    b.extend_from_slice(SAVE_MAGIC);
    b.extend_from_slice(&SAVE_VERSION.to_le_bytes());
    for s in slots {
        b.extend_from_slice(&s.level.to_le_bytes());
        let name = &s.name.as_bytes()[..s.name.len().min(NAME_MAX)];
        b.push(name.len() as u8);
        b.extend_from_slice(name);
        b.resize(b.len() + NAME_MAX - name.len(), 0);
        let d = &s.dat;
        let ints = std::iter::once(d.num_lives)
            .chain(d.weapon_ammo)
            .chain(d.weapon_avail)
            .chain([d.health, d.score]);
        for v in ints {
            b.extend_from_slice(&v.to_le_bytes());
        }
    }
    b
}

fn decode(b: &[u8]) -> Option<Slots> {
    if b.len() != 8 + NUM_SLOTS * SLOT_BYTES || &b[..4] != SAVE_MAGIC || b[4..8] != SAVE_VERSION.to_le_bytes() {
        return None;
    }
    let rd = |o: usize| i32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
    let mut slots: Slots = Default::default();
    for (i, s) in slots.iter_mut().enumerate() {
        let o = 8 + i * SLOT_BYTES;
        let level = rd(o);
        let len = (b[o + 4] as usize).min(NAME_MAX);
        let name: String = b[o + 5..o + 5 + len].iter().filter(|c| c.is_ascii() && !c.is_ascii_control()).map(|&c| c as char).collect();
        let mut q = o + 5 + NAME_MAX;
        let mut next = || {
            let v = rd(q);
            q += 4;
            v
        };
        let mut d = PlayerData { num_lives: next(), ..DEFAULT_PDATA };
        for a in d.weapon_ammo.iter_mut() {
            *a = next();
        }
        for a in d.weapon_avail.iter_mut() {
            *a = next();
        }
        d.health = next();
        d.score = next();
        if (1..=NUM_LEVELS).contains(&level) {
            *s = SaveGameData { level, name, dat: d };
        }
    }
    Some(slots)
}

/// Reads savegame.dat; a missing or unreadable file is replaced by five empty slots.
pub fn load_slots() -> Slots {
    if let Some(s) = std::fs::read(save_path()).ok().and_then(|b| decode(&b)) {
        return s;
    }
    let slots: Slots = Default::default();
    write_slots(&slots);
    slots
}

pub fn write_slots(slots: &Slots) {
    let _ = std::fs::write(save_path(), encode(slots));
}

pub fn slot_menu_text(slots: &Slots) -> String {
    let mut buffer = String::new();
    for s in slots {
        if s.level != 0 {
            buffer.push_str(&s.name);
        } else {
            buffer.push_str("EMPTY");
        }
        buffer.push('\n');
    }
    buffer
}

fn save_game(slots: &Slots, name: &mut String) -> usize {
    if do_menu("CONTINUE\nSAVE GAME", 2, 1) != 2 {
        return 0;
    }
    let pos = do_menu(&slot_menu_text(slots), 5, 1);
    if pos == 0 {
        return 0;
    }
    let pos = pos as usize;
    if slots[pos - 1].level != 0 {
        *name = slots[pos - 1].name.clone();
    }
    if get_level_name(name) { pos } else { 0 }
}

fn draw_grid() {
    sys::with_screen(|s| {
        for x in (0..320).step_by(2) {
            for y in (0..240).step_by(2) {
                s.putpixel(x, y, 0);
                s.putpixel(x + 1, y + 1, 0);
            }
        }
    });
}

pub fn display_level_info(level: i32) {
    let (bmp, pal) = icache::get_pal("ibild.pcx");
    sys::with_screen(|s| crate::gfx::blit(&bmp, s, 0, 0, 0, 0, bmp.w, bmp.h));
    sys::present(true);
    let mut fnt = UrbanFont::new(LARGE_FONT);
    let message;
    if level == -1 {
        message = "GAME OVER";
    } else if level > NUM_LEVELS {
        outtro::run_outtro();
        credits::show_credits();
        // The original printed an uninitialised buffer here; restore its commented-out text.
        sys::with_screen(|s| crate::gfx::blit(&bmp, s, 0, 0, 0, 0, bmp.w, bmp.h));
        fnt.set_scale(65);
        fnt.print_centre_screen("MISSION", 320 / 2, 190 / 2);
        message = "COMPLETED";
    } else {
        return;
    }
    fnt.print_centre_screen(message, 320 / 2, 240 / 2);
    sys::fade_in(&pal, 2);
    sys::rest(2000);
    sys::clear_keybuf();
}

/// Slot name entry; returns false on ESC.
pub fn get_level_name(text: &mut String) -> bool {
    let fnt = UrbanFont::new(SMALL_FONT2);
    if text.is_empty() {
        *text = "MYLEVEL".to_string();
    }
    draw_grid();
    let mut textbmp = Bitmap::new(220, 55);
    sys::clear_keybuf();
    let message = "Slot Name";
    loop {
        sys::with_screen(|s| {
            s.rectfill(45, 55, 320 - 45, 240 - 120, 27);
            s.rectfill(50, 60, 320 - 50, 240 - 125, 25);
        });
        textbmp.clear();
        fnt.print_centre(message, 110, 20, &mut textbmp);
        fnt.print_centre(text, 110, 40, &mut textbmp);
        sys::masked_blit_to_screen(&textbmp, 0, 0, 50, 60, 220, 55);
        sys::present(true);

        let key = sys::readkey();
        let sc = (key >> 8) as usize;
        let ch = (key & 0xff) as u8;
        if sc == KEY_ENTER {
            return true;
        }
        if sc == KEY_ESC {
            return false;
        }
        if sc == KEY_BACKSPACE {
            if text.pop().is_none() {
                continue;
            }
        }
        if sc == KEY_SPACE {
            if text.len() >= 9 {
                continue;
            }
            text.push(' ');
        } else if text.len() < 9 && ((b'A'..=b'z').contains(&ch) || ch == b' ') {
            text.push(ch.to_ascii_uppercase() as char);
        }
    }
}

/// NewGame(slot): slot 0 starts from level 1, otherwise resumes that save slot.
pub fn new_game(g: &mut Game, slot: usize) -> i32 {
    g.new_game();
    // The original shows the level 1 screen here even when a later slot is loaded.
    display_level_info(1);
    let slots = load_slots();
    let mut pdat = DEFAULT_PDATA;
    let mut level: i32 = 1;
    if slot != 0 && slots[slot - 1].level != 0 {
        pdat = slots[slot - 1].dat;
        level = slots[slot - 1].level.clamp(1, NUM_LEVELS);
    }
    run_campaign(g, level, pdat, None)
}

/// Main menu CONTINUE: resumes the most recent quick save or autosave.
pub fn continue_game(g: &mut Game) -> bool {
    let Some(path) = crate::saves::latest() else { return false };
    let Ok(st) = crate::engine::SaveState::read(&path) else { return false };
    g.new_game();
    let level = if st.level > 0 { st.level } else { 1 };
    let pdat = st.level_start;
    run_campaign(g, level, pdat, Some(st));
    true
}

/// The level sequence (game.cc NewGame), optionally starting from a mid-level save.
fn run_campaign(g: &mut Game, mut level: i32, mut pdat: PlayerData, mut resume: Option<crate::engine::SaveState>) -> i32 {
    let mut name = String::new();
    let mut slots = load_slots();
    let mut map_name: String = MAPS[(level - 1) as usize].to_string();
    let mut quit = false;
    while !quit {
        g.campaign_level = level;
        let start_pdat = pdat;
        let r = match resume.take() {
            Some(st) => {
                map_name = st.map_name.clone();
                g.play_saved(st, &mut pdat)
            }
            None => g.play_level(&map_name, Some(&mut pdat), config::get().ctrl_type),
        };
        // A quick load can switch to another campaign level.
        if g.campaign_level > 0 {
            level = g.campaign_level;
        }
        match r {
            -1 => {
                sound::play_sfx("samples/haha.wav");
                quit = true;
                display_level_info(-1);
                highscore::add(g.score.get_score(), level);
            }
            0 => {
                sys::fade_out(5);
                quit = true;
            }
            3 => {
                // RESTART LEVEL from the pause menu: same map, the player data it began with.
                pdat = g.level_start;
                let _ = start_pdat;
                map_name = g.map_name.clone();
                sys::fade_out(5);
            }
            1 | 2 => {
                let mut extra = false;
                if r == 2 {
                    sys::fade_out(5);
                    let m = EXTRA_MAPS.get((level - 1) as usize).copied().unwrap_or("");
                    if !m.is_empty() {
                        map_name = m.to_string();
                        extra = true;
                    }
                }
                if !extra {
                    level += 1;
                    sys::fade_out(5);
                    display_level_info(level);
                    if level > NUM_LEVELS {
                        quit = true;
                        level -= 1;
                        highscore::add(g.score.get_score(), level);
                    } else {
                        let s = save_game(&slots, &mut name);
                        if s != 0 {
                            slots[s - 1] = SaveGameData { level, name: name.clone(), dat: pdat };
                            write_slots(&slots);
                        }
                    }
                    map_name = MAPS[(level - 1) as usize].to_string();
                }
            }
            _ => {}
        }
    }
    1
}
