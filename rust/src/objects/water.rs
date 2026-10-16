//! Port of objects/water.cc: water surface; switches objects into water mode.

use super::anim::Anim;
use crate::engine::*;
use crate::icache;
use crate::map::{PAL_AIR, PAL_WATER};

pub struct Water {
    pub base: Base,
    anim: Anim,
}

impl Water {
    pub fn new(x: i32, y: i32, z: i32) -> Water {
        let mut b = Base::new(x, y, z);
        b.images = icache::seq("water", 1, 2, ".pcx");
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.y -= b.height;
        b.y += 10;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.energy = 1;
        b.me = FRIEND_WATER;
        Water { base: b, anim: Anim::default() }
    }
}

impl Obj for Water {
    crate::obj_boilerplate!();

    fn update(&mut self, _g: &mut Game) -> Upd {
        self.base.current_image = self.anim.next_frame(1, 60);
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, g: &mut Game) {
        let y = self.base.y;
        let o = other.base_mut();
        let is_player = o.me == FRIEND_PLAYER;
        if o.speed_y > 0 && o.y > y {
            o.mode = MODE_WATER;
            if is_player {
                g.map.set_pal(PAL_WATER);
            }
        } else if o.mode == MODE_WATER && o.y <= y {
            o.mode = MODE_NORMAL;
            if is_player {
                g.map.set_pal(PAL_AIR);
            }
        }
    }
}
