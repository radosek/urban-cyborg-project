//! Port of objects/fire.cc: a burning flame that hurts everything.

use super::anim::Anim;
use crate::engine::*;
use crate::icache;

const FRAME_DELAY: i32 = 4;
const BURN_DELAY: i32 = 2000;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Fire {
    pub base: Base,
    anim: Anim,
}

impl Fire {
    pub fn new(x: i32, y: i32, z: i32, size: i32) -> Fire {
        let mut b = Base::new(x, y, z);
        let dir = if size == 1 { "fire/" } else { "smallfir/" };
        b.images = icache::seq(dir, 1, 4, ".pcx");
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.coll_width = b.width;
        b.coll_height = b.height;
        // Stand on its feet
        b.y -= b.height / 2;
        b.energy = 1;
        b.strength = 2;
        b.friends = !0;
        b.enemies = 0;
        b.counter = BURN_DELAY;
        Fire { base: b, anim: Anim::default() }
    }
}

#[typetag::serde]
impl Obj for Fire {
    crate::obj_boilerplate!();

    fn update(&mut self, _g: &mut Game) -> Upd {
        let b = &mut self.base;
        if b.counter != 0 {
            b.counter -= 1;
        } else {
            return Upd::Delete;
        }
        b.current_image = self.anim.next_frame(3, FRAME_DELAY);
        Upd::Keep
    }
}
