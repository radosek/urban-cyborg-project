//! Port of objects/compscrn.cc: a breakable computer screen.

use super::smallexp::SmallExplosion;
use crate::engine::*;
use crate::icache;

pub struct ComputerScreen {
    pub base: Base,
}

impl ComputerScreen {
    pub fn new(x: i32, y: i32, z: i32, type_: i32) -> ComputerScreen {
        let mut b = Base::new(x, y, z);
        // The C++ left the image unset for any type other than 0 or 1.
        let name = if type_ == 0 { "dekor/skarm1.pcx" } else { "dekor/skarm2.pcx" };
        b.images = vec![icache::get(name)];
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.x += TILE_WIDTH / 2;
        b.x -= b.width / 2;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.energy = 1;
        ComputerScreen { base: b }
    }
}

impl Obj for ComputerScreen {
    crate::obj_boilerplate!();

    fn update(&mut self, _g: &mut Game) -> Upd {
        if self.base.energy == 0 {
            return Upd::Delete;
        }
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, g: &mut Game) {
        let b = &mut self.base;
        if b.energy == 0 {
            return;
        }
        if other.base().me & (FRIEND_HS_BULLET | FRIEND_EXPLOSION | FRIEND_BEAM) == 0 {
            return;
        }
        b.energy = 0;
        g.create_effect(SmallExplosion::new(b.x, b.y, b.z));
    }
}
