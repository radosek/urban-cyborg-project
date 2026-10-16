//! Port of objects/bubblmak.cc: spawns bubbles at random intervals.

use super::bubble::Bubble;
use crate::engine::*;
use crate::{icache, rng};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BubbleMaker {
    pub base: Base,
}

impl BubbleMaker {
    pub fn new(x: i32, y: i32, z: i32) -> BubbleMaker {
        let mut b = Base::new(x, y, z);
        b.images = vec![icache::get("mine2.pcx")];
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.x += TILE_WIDTH;
        b.y -= b.height;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.energy = 1;
        b.me = FRIEND_BUBBLE;
        BubbleMaker { base: b }
    }
}

#[typetag::serde]
impl Obj for BubbleMaker {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        if b.counter != 0 {
            b.counter -= 1;
        }
        if b.counter == 0 {
            b.counter = 30 + rng::random() % 100;
            g.create_object(Bubble::new(b.x, b.y, b.z));
        }
        Upd::Keep
    }

    fn collision(&mut self, _other: &mut dyn Obj, _g: &mut Game) {}
}
