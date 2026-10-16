//! Bump-mapped credits (credits/credits.cc).

use crate::font::{UrbanFont, LARGE_FONT};
use crate::gfx::{self, Bitmap};
use crate::{icache, sound, sys};
use std::sync::OnceLock;

const SCREEN_HEIGHT: i32 = 240;
const LIGHT_SIZE: f32 = 128.0;

const LIGHT_POS: [(i32, i32); 5] = [(25, 50), (250, 150), (250, 15), (50, 175), (160, 120)];
const NUM_LIGHTS: usize = LIGHT_POS.len();

struct CreditsInfo {
    title: &'static str,
    name: &'static str,
    erase_title: bool,
}

const CI: [CreditsInfo; 8] = [
    CreditsInfo { title: "PROGRAMMING", name: "THOMAS\nNYBERG", erase_title: false },
    CreditsInfo { title: "PROGRAMMING", name: "  JONAS\nBORGSTROM", erase_title: true },
    CreditsInfo { title: "GRAPHICS", name: "  DICK\nADOLFSSON", erase_title: true },
    CreditsInfo { title: "DESIGN", name: " PETTER\nHOLMBERG", erase_title: false },
    CreditsInfo { title: "DESIGN", name: "MARKUS\nBOMAN", erase_title: true },
    CreditsInfo { title: "SOUND", name: "  EMIL\nERIKSSON", erase_title: false },
    CreditsInfo { title: "SOUND", name: "RICHARD\nBERGMARK", erase_title: true },
    CreditsInfo { title: "MUSIC", name: "SAMUEL\nPERSSON", erase_title: true },
];

const SPECIAL_THANKS: &str = "\n SPECIAL THANKS\n\n PER JONNY KACK\n  TOR SANDEN\nMARKUS WILLANDER\n    UFFEMAN\n    HASSEMAN\n  ANNA NORBERG\n\n    DESCENT\n\n AND OURSELVES\n\n      NOW\n  PLAY URBAN";

fn lightmap() -> &'static [u8] {
    static LM: OnceLock<Vec<u8>> = OnceLock::new();
    LM.get_or_init(|| {
        let mut lm = vec![0u8; 256 * 256];
        for y in 0..256 {
            for x in 0..256 {
                let nx = (x - 128) as f32 / LIGHT_SIZE;
                let ny = (y - 128) as f32 / LIGHT_SIZE;
                let mut nz = 1.0 - (nx * nx + ny * ny).sqrt();
                if nz < 0.0 {
                    nz = 0.0;
                }
                let v = nz * 191.0 + nz.powi(9) * 64.0;
                lm[(x + y * 256) as usize] = if 255.0 < v { 255 } else { v as u8 };
            }
        }
        lm
    })
}

struct Credits {
    buffer: Bitmap,
    enabled: [bool; NUM_LIGHTS],
    lm: &'static [u8],
}

impl Credits {
    fn do_bump(&mut self, hmap: &Bitmap, mut starty: i32, mut stopy: i32) {
        if starty < 1 {
            starty = 1;
        }
        if stopy > SCREEN_HEIGHT - 2 {
            stopy = SCREEN_HEIGHT - 2;
        }
        for y in starty..stopy {
            for x in 1..319 {
                let nx = px(hmap, x + 1, y) - px(hmap, x - 1, y);
                let ny = px(hmap, x, y + 1) - px(hmap, x, y - 1);
                let mut lval = 0;
                for (i, &(lx, ly)) in LIGHT_POS.iter().enumerate() {
                    if !self.enabled[i] {
                        continue;
                    }
                    let mut nx1 = nx - (x - lx - 128);
                    let mut ny1 = ny - (y - ly - 128);
                    if !(0..=255).contains(&nx1) {
                        nx1 = 255;
                    }
                    if !(0..=255).contains(&ny1) {
                        ny1 = 255;
                    }
                    lval += self.lm[(nx1 + ny1 * 256) as usize] as i32;
                }
                if lval > 255 {
                    lval = 255;
                }
                self.buffer.putpixel(x, y, lval);
            }
        }
    }

    fn show(&self, starty: i32, h: i32) {
        sys::blit_to_screen(&self.buffer, 0, starty, 0, starty, self.buffer.w, h);
    }

    fn morf(&mut self, source: &Bitmap, dest: &Bitmap, starty: i32, stopy: i32, numframes: i32) -> Bitmap {
        let mut tmp = Bitmap::new(source.w, source.h);
        for k in 1..=numframes {
            for j in starty..stopy {
                for i in 0..320 {
                    let s = px(source, i, j);
                    let d = px(dest, i, j);
                    tmp.putpixel(i, j, s + ((d - s) as f32 * (k as f32 / numframes as f32)) as i32);
                }
            }
            self.do_bump(&tmp, starty + 1, stopy - 1);
            self.show(starty, stopy - starty + 1);
            if sys::keypressed() {
                break;
            }
        }
        tmp
    }

    /// Switches the lights off one by one from `from` down to 0.
    fn lights_out(&mut self, from: usize, hmap: &Bitmap) {
        for i in (0..=from).rev() {
            sys::rest(500);
            sound::play_sfx("samples/spots.wav");
            self.enabled[i] = false;
            self.do_bump(hmap, 0, SCREEN_HEIGHT);
            self.show(0, self.buffer.h);
        }
    }
}

fn px(b: &Bitmap, x: i32, y: i32) -> i32 {
    b.getpixel(x, y).max(0)
}

fn delay(ms: i32) {
    for _ in 0..ms / 10 {
        sys::rest(10);
        if sys::keypressed() {
            break;
        }
    }
}

fn cblit(bmp: &Bitmap, dest: &mut Bitmap, x: i32, y: i32) {
    gfx::masked_blit(bmp, dest, 0, 0, x - bmp.w / 2, y, bmp.w, bmp.h);
}

pub fn show_credits() {
    let mut c = Credits { buffer: Bitmap::new(320, SCREEN_HEIGHT), enabled: [false; NUM_LIGHTS], lm: lightmap() };
    let mut ufont = UrbanFont::new(LARGE_FONT);

    let hmap = icache::get("credits/bump.pcx");
    let (_, pal) = icache::get_pal("credits/palette.pcx");
    sys::with_screen(|s| s.clear());
    sys::set_palette(&pal);

    let mut heightmap = Bitmap::new(320, 240);
    gfx::blit(&hmap, &mut heightmap, 0, 0, 0, 0, hmap.w, hmap.h);

    ufont.set_scale(180);
    cblit(&ufont.render("URBAN"), &mut heightmap, 160, 30);
    ufont.set_scale(60);
    cblit(&ufont.render("THE"), &mut heightmap, 160, 120);
    cblit(&ufont.render("CYBORG PROJECT"), &mut heightmap, 160, 150);

    for i in 0..NUM_LIGHTS {
        delay(500);
        sound::play_sfx("samples/spots.wav");
        c.enabled[i] = true;
        c.do_bump(&heightmap, 0, SCREEN_HEIGHT);
        c.show(0, c.buffer.h);
        if sys::keypressed() {
            c.lights_out(i, &heightmap);
            sys::fade_out(10);
            return;
        }
    }
    delay(1200);
    c.morf(&heightmap, &hmap, 0, SCREEN_HEIGHT, 10);

    for i in 0..CI.len() {
        gfx::blit(&hmap, &mut heightmap, 0, 0, 0, 0, hmap.w, hmap.h);
        ufont.set_scale(80);
        cblit(&ufont.render(CI[i].title), &mut heightmap, 160, 20);
        ufont.set_scale(100);
        let txt = ufont.render(CI[i].name);
        cblit(&txt, &mut heightmap, 160, 80);

        let starty = if i == 0 || CI[i - 1].erase_title { 10 } else { 70 };
        let stopy = 80 + txt.h;

        c.morf(&hmap, &heightmap, starty, stopy, 10);
        delay(900);

        if sys::keypressed() {
            let tmp = c.morf(&hmap, &heightmap, 0, SCREEN_HEIGHT, 1);
            c.lights_out(NUM_LIGHTS - 1, &tmp);
            return;
        }
    }

    ufont.set_scale(60);
    let txt = ufont.render(SPECIAL_THANKS);
    gfx::blit(&hmap, &mut heightmap, 0, 0, 0, 0, hmap.w, hmap.h);
    cblit(&txt, &mut heightmap, 160, SCREEN_HEIGHT - 60);
    c.morf(&hmap, &heightmap, 0, SCREEN_HEIGHT, 10);
    c.show(0, c.buffer.h);
    let mut tmp = Bitmap::new(320, SCREEN_HEIGHT - 20);
    let mut y = SCREEN_HEIGHT - 60;
    while y > -txt.h {
        gfx::blit(&hmap, &mut heightmap, 0, 0, 0, 0, hmap.w, hmap.h);
        tmp.clear();
        cblit(&txt, &mut tmp, 160, y - 10);
        gfx::masked_blit(&tmp, &mut heightmap, 0, 0, 0, 10, tmp.w, tmp.h);
        c.do_bump(&heightmap, 0, SCREEN_HEIGHT);
        c.show(0, c.buffer.h);
        if sys::keypressed() || (y + txt.h < 100 && y % 20 == 0) {
            c.lights_out(NUM_LIGHTS - 1, &heightmap);
            break;
        }
        y -= 1;
    }

    sys::fade_out(10);
}
