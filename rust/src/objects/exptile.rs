//! Port of objects/exptile.cc: a tile destroyed by explosions.

use super::explode::Explosion;
use crate::engine::*;
use crate::icache;

pub struct ExplosiveTile {
    pub base: Base,
    #[allow(dead_code)]
    type_: i32,
}

impl ExplosiveTile {
    pub fn new(x: i32, y: i32, z: i32, type_: i32) -> ExplosiveTile {
        let mut b = Base::new(x, y, z);
        b.images = vec![icache::get("xbarrel.pcx")];
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.x += TILE_WIDTH / 2;
        b.x -= b.width / 2;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.energy = 1;
        ExplosiveTile { base: b, type_ }
    }
}

impl Obj for ExplosiveTile {
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
        if other.base().me & FRIEND_EXPLOSION == 0 {
            return;
        }
        b.energy = 0;
        g.clear_tile(b.x + b.width / 2, b.y + b.height / 2, b.z);
        g.create_object(Explosion::new(b.x, b.y, b.z, 0, 0, 0, 0));
    }
}
