//! Port of objects/bubble.cc: rising air bubble.

use crate::engine::*;
use crate::{icache, rng};

const STATE_DIE: i32 = 0x01;
const STATE_NONE: i32 = 0x00;
const MAX_LIFE: i32 = 200;
const MIN_LIFE: i32 = 50;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Bubble {
    pub base: Base,
}

impl Bubble {
    pub fn new(x: i32, y: i32, z: i32) -> Bubble {
        let mut b = Base::new(x, y, z);
        let name = if rng::random() % 2 == 0 { "bubble.pcx" } else { "bubbles.pcx" };
        b.images = vec![icache::get(name)];
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.x += TILE_WIDTH;
        b.y -= b.height;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.energy = 1;
        b.me = FRIEND_BUBBLE;
        b.counter4 = MIN_LIFE + rng::random() % MAX_LIFE;
        b.state = STATE_NONE;
        Bubble { base: b }
    }
}

#[typetag::serde]
impl Obj for Bubble {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        if b.state == STATE_DIE {
            return Upd::Delete;
        }
        if b.counter4 != 0 {
            b.counter4 -= 1;
        } else {
            b.state = STATE_DIE;
        }
        if b.counter != 0 {
            b.counter -= 1;
        }
        if b.counter2 != 0 {
            b.counter2 -= 1;
        }
        if b.counter == 0 {
            b.y -= 1;
            b.counter = 2;
        }
        if b.counter2 == 0 {
            b.x += -1 + rng::random() % 3;
            b.counter2 = 8;
        }
        if g.check_floor(b.x, b.y, b.z) {
            return Upd::Delete;
        }
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, _g: &mut Game) {
        if other.base().me == FRIEND_WATER {
            self.base.state = STATE_DIE;
        }
    }
}
