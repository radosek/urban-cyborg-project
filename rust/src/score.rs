//! HUD: health bar, score counter, lives, ammo, key cards, face and boss health (engine/score.cc).

use crate::engine::{Obj, CARD_BLUE, CARD_GREEN, CARD_RED, SCREEN_WIDTH};
use crate::font::{UrbanFont, SMALL_FONT};
use crate::gfx::{self, Bitmap};
use crate::icache::{self, Img};
use crate::objects::player::Player;

const BOSSHEALTH_WIDTH: i32 = SCREEN_WIDTH - 2 * 20;

pub struct ScoreBoard {
    score: i32,
    tmp_score: i32,
    boss_health: i32,
    boss_max: i32,
    face: Vec<Img>,
    cards: [Img; 3],
    board: Img,
    fnt: UrbanFont,
}

impl ScoreBoard {
    pub fn new() -> ScoreBoard {
        ScoreBoard {
            score: 0,
            tmp_score: 0,
            boss_health: 0,
            boss_max: -1,
            face: icache::seq("head/head", 1, 12, ".pcx"),
            cards: [icache::get("items/bluecard.pcx"), icache::get("items/greencar.pcx"), icache::get("items/redcard.pcx")],
            board: icache::get("scoreb.pcx"),
            fnt: UrbanFont::new(SMALL_FONT),
        }
    }

    /// Draws the HUD; `ox` shifts it horizontally (centred in widescreen).
    pub fn display(&mut self, bmp: &mut Bitmap, p: &Player, ox: i32) {
        if self.score - 99 > self.tmp_score {
            self.tmp_score += 99;
        }
        if self.score - 4 > self.tmp_score {
            self.tmp_score += 4;
        }
        if self.score > self.tmp_score {
            self.tmp_score += 1;
        }
        let energy = p.base().energy;
        bmp.rectfill(ox + 26 + 44 * energy / 300, 23, ox + 27 + 44, 34, 19);
        bmp.rectfill(ox + 26, 23, ox + 27 + 44 * energy / 300, 34, 36);
        gfx::draw(bmp, &self.board, ox, 0);
        self.fnt.print(&format!("Score:{:6}", self.tmp_score), ox + 212, 10, bmp);
        for (card, img, x) in [(CARD_BLUE, 0, 237), (CARD_GREEN, 1, 212), (CARD_RED, 2, 262)] {
            if p.have_card(card) {
                let c = &self.cards[img];
                gfx::blit(c, bmp, 0, 0, ox + x, 40, c.w, c.h);
            }
        }
        self.fnt.print(&p.lives().to_string(), ox + 32, 40, bmp);
        let ammo = p.ammo();
        self.fnt.print(&if ammo < 0 { "Inf".to_string() } else { ammo.to_string() }, ox + 27, 4, bmp);
        // Clamped: the original indexed past the 12 faces for energy outside 0..=300.
        let mut head = (10 * (300 - energy) / 300).clamp(0, 10) as usize;
        if p.base().is_immortal() {
            head = 11;
        }
        gfx::draw(bmp, &self.face[head], ox + SCREEN_WIDTH / 2 - 27, 8);
        if self.boss_max > 0 {
            bmp.rectfill(ox + 20 - 1, 65 - 1, ox + 20 + BOSSHEALTH_WIDTH + 1, 65 + 7 + 1, 0);
            bmp.rectfill(ox + 20, 65, ox + 20 + self.boss_health.max(0) * BOSSHEALTH_WIDTH / self.boss_max, 65 + 7, 36);
        }
    }

    pub fn save_state(&self) -> [i32; 4] {
        [self.score, self.tmp_score, self.boss_health, self.boss_max]
    }

    pub fn restore_state(&mut self, v: [i32; 4]) {
        [self.score, self.tmp_score, self.boss_health, self.boss_max] = v;
    }

    pub fn add_score(&mut self, s: i32) {
        self.score += s;
    }
    pub fn get_score(&self) -> i32 {
        self.score
    }
    pub fn reset_score(&mut self) {
        self.score = 0;
        self.tmp_score = 0;
    }
    pub fn set_score(&mut self, s: i32) {
        self.score = s;
    }
    /// -1 hides the boss bar.
    pub fn init_boss_health(&mut self, max: i32) {
        self.boss_max = max;
    }
    pub fn set_boss_health(&mut self, h: i32) {
        self.boss_health = h;
    }
}
