//! Hall of fame (highscor/highscor.cc).

use crate::font::{UrbanFont, SMALL_FONT2};
use crate::gfx::Bitmap;
use crate::keys::*;
use crate::{icache, sys};
use std::cell::RefCell;

pub const NUM_HIGHSCORES: usize = 10;
const NAME_MAX: usize = 63;

#[derive(Clone, Default)]
struct Score {
    name: String,
    score: i32,
    level: i32,
}

// hs.dat: "URBH", u32 version, then per entry: u8 name length, NAME_MAX name bytes
// (zero padded), i32 score, i32 level. Little-endian.
const MAGIC: &[u8; 4] = b"URBH";
const VERSION: u32 = 1;
const ENTRY: usize = 1 + NAME_MAX + 8;

fn path() -> std::path::PathBuf {
    sys::user_dir().join("hs.dat")
}

fn open() -> Vec<Score> {
    let mut hs = vec![Score::default(); NUM_HIGHSCORES];
    let Ok(b) = std::fs::read(path()) else { return hs };
    if b.len() != 8 + NUM_HIGHSCORES * ENTRY || &b[..4] != MAGIC || b[4..8] != VERSION.to_le_bytes() {
        return hs;
    }
    let rd = |o: usize| i32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
    for (i, e) in hs.iter_mut().enumerate() {
        let o = 8 + i * ENTRY;
        let len = (b[o] as usize).min(NAME_MAX);
        e.name = b[o + 1..o + 1 + len].iter().filter(|c| c.is_ascii() && !c.is_ascii_control()).map(|&c| c as char).collect();
        e.score = rd(o + 1 + NAME_MAX);
        e.level = rd(o + 5 + NAME_MAX).clamp(0, crate::game::NUM_LEVELS);
    }
    hs
}

fn save(hs: &[Score]) {
    let mut b = Vec::with_capacity(8 + NUM_HIGHSCORES * ENTRY);
    b.extend_from_slice(MAGIC);
    b.extend_from_slice(&VERSION.to_le_bytes());
    for e in hs {
        let n = &e.name.as_bytes()[..e.name.len().min(NAME_MAX)];
        b.push(n.len() as u8);
        b.extend_from_slice(n);
        b.resize(b.len() + NAME_MAX - n.len(), 0);
        b.extend_from_slice(&e.score.to_le_bytes());
        b.extend_from_slice(&e.level.to_le_bytes());
    }
    let _ = std::fs::write(path(), b);
}

/// HighScore(): draws the table over the current screen.
pub fn show() {
    let hs = open();
    let fnt = UrbanFont::new(SMALL_FONT2);
    let mut textbmp = Bitmap::new(220, 210);
    sys::with_screen(|s| {
        for x in (0..320).step_by(2) {
            for y in (0..240).step_by(2) {
                s.putpixel(x, y, 0);
                s.putpixel(x + 1, y + 1, 0);
            }
        }
        s.rectfill(45, 25, 320 - 45, 233, 27);
        s.rectfill(50, 30, 320 - 50, 228, 25);
        s.rectfill(50, 30, 320 - 50, 63, 23);
    });
    fnt.print("***** HALL OF FAME *****", 60, 13, &mut textbmp);
    fnt.print("  Name          Score Level", 0, 30, &mut textbmp);
    for (i, e) in hs.iter().enumerate() {
        let y = 50 + i as i32 * 16;
        fnt.print(&format!("{:2}", i + 1), 15, y, &mut textbmp);
        let line = if e.level != 0 {
            format!(" {:<10}{:6}   {}:{}", e.name, e.score, (e.level - 1) / 3 + 1, (e.level - 1) % 3 + 1)
        } else {
            format!(" {:<10}{:6}   0:0", "Empty", 0)
        };
        fnt.print(&line, 31, y, &mut textbmp);
    }
    sys::masked_blit_to_screen(&textbmp, 0, 0, 50, 20, 220, 210);
    sys::present(true);
}

thread_local! {
    static NAME: RefCell<String> = const { RefCell::new(String::new()) };
}

fn get_name() -> String {
    let (backg, pal) = icache::get_pal("ibild.pcx");
    let mut textbmp = Bitmap::new(220, 55);
    let fnt = UrbanFont::new(SMALL_FONT2);
    NAME.with(|n| {
        let mut name = n.borrow_mut();
        if name.is_empty() {
            *name = std::env::var("USER").ok().filter(|u| !u.is_empty()).unwrap_or_else(|| "Unknown".to_string());
            let cut = name.char_indices().nth(NAME_MAX).map(|(i, _)| i).unwrap_or(name.len());
            name.truncate(cut);
        }
        sys::set_palette(&pal);
        sys::blit_to_screen(&backg, 0, 0, 0, 0, 320, 240);
        sys::clear_keybuf();
        loop {
            sys::with_screen(|s| {
                s.rectfill(45, 55, 320 - 45, 240 - 120, 27);
                s.rectfill(50, 60, 320 - 50, 240 - 125, 25);
                s.rectfill(50, 60, 320 - 50, 240 - 165, 23);
            });
            textbmp.clear();
            fnt.print_centre("HighScore", 110, 8, &mut textbmp);
            fnt.print_centre("Please Enter Your Name:", 110, 24, &mut textbmp);
            fnt.print_centre(&name, 110, 43, &mut textbmp);
            sys::masked_blit_to_screen(&textbmp, 0, 0, 50, 60, 220, 55);
            sys::present(true);

            let key = sys::readkey();
            let sc = (key >> 8) as usize;
            let ch = (key & 0xff) as u8;
            if sc == KEY_ENTER {
                return name.clone();
            }
            if sc == KEY_ESC {
                return "Unknown".to_string();
            }
            if sc == KEY_BACKSPACE {
                name.pop();
            } else if name.chars().count() < 10 && ((b'A'..=b'z').contains(&ch) || ch == b' ') {
                name.push(ch as char);
            }
        }
    })
}

/// HighScore(score, level): asks for a name if the score makes the table, then saves.
pub fn add(score: i32, level: i32) {
    let mut hs = open();
    for i in 0..NUM_HIGHSCORES {
        if score > hs[i].score {
            hs.insert(i, Score { name: get_name(), score, level });
            hs.truncate(NUM_HIGHSCORES);
            break;
        }
    }
    save(&hs);
}
