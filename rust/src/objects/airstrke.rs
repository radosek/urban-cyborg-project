//! Port of ref/objects/airstrke.cc

use super::airgren::AirGrenade;
use crate::engine::*;
use crate::{icache, rng::random, sound};

const STATE_NONE: i32 = 0;
const STATE_WAIT: i32 = 1;
const STATE_STRIKE: i32 = 2;
const STATE_DONE: i32 = 3;

const NUM_NUKES: i32 = 90;
const AREA_START: i32 = -300;
const AREA_WIDTH: i32 = 600;
const AREA_MUL: i32 = AREA_WIDTH / NUM_NUKES;
const STRIKE_DELAY: i32 = 600;
const STRIKE_START: i32 = -20;
const STRIKE_END: i32 = 20;
const STRIKE_SAMPLE: &str = "samples/hellfire.wav";
const WARNING_SAMPLE: &str = "samples/airnuke.wav";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Airstrike {
    pub base: Base,
}

impl Airstrike {
    pub fn new(x: i32, y: i32, z: i32) -> Airstrike {
        let mut b = Base::new(x, y, z);
        b.images = vec![icache::get("mine2.pcx")];
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.x += TILE_WIDTH / 2;
        b.x -= b.width / 2;
        b.y -= b.height;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.energy = 1;
        b.me = FRIEND_DEKOR;
        b.state = STATE_NONE;
        Airstrike { base: b }
    }
}

#[typetag::serde]
impl Obj for Airstrike {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        if b.counter != 0 {
            b.counter -= 1;
        }
        if b.counter2 != 0 {
            b.counter2 -= 1;
        }
        if b.state == STATE_DONE {
            return Upd::Delete;
        }
        match b.state {
            STATE_NONE => {
                let px = g.pbase().x;
                if px > b.x + STRIKE_START && px < b.x + STRIKE_END {
                    b.state = STATE_WAIT;
                    b.counter = STRIKE_DELAY;
                    g.push_message("Airstrike alert!");
                    sound::play_sfx_critical(WARNING_SAMPLE);
                }
            }
            STATE_WAIT => {
                if b.counter == 0 {
                    b.state = STATE_STRIKE;
                }
            }
            _ => {
                if b.state == STATE_STRIKE {
                    sound::play_sfx_critical(STRIKE_SAMPLE);
                    for i in 0..NUM_NUKES {
                        g.create_alwaysupdate(AirGrenade::new(
                            b.x + AREA_START + i * AREA_MUL,
                            b.y - i * 15,
                            (random() % 3) * TILE_TOP_HEIGHT,
                            2,
                            5,
                            0,
                        ));
                    }
                }
                b.state = STATE_DONE;
            }
        }
        Upd::Keep
    }

    fn collision(&mut self, _other: &mut dyn Obj, _g: &mut Game) {}
}
