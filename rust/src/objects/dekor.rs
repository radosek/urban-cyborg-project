//! Port of objects/dekor.cc: a static decoration sprite.

use crate::engine::*;
use crate::icache;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Dekor {
    pub base: Base,
}

impl Dekor {
    pub fn new(x: i32, y: i32, z: i32, filename: &str) -> Dekor {
        let mut b = Base::new(x, y, z);
        b.images = vec![icache::get(filename)];
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.x += TILE_WIDTH / 2;
        b.x -= b.width / 2;
        b.y -= b.height;
        b.y += TILE_TOP_HEIGHT;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.energy = 1;
        b.speed_y = 5;
        b.me = FRIEND_DEKOR;
        Dekor { base: b }
    }
}

#[typetag::serde]
impl Obj for Dekor {
    crate::obj_boilerplate!();

    fn update(&mut self, _g: &mut Game) -> Upd {
        Upd::Keep
    }

    fn collision(&mut self, _other: &mut dyn Obj, _g: &mut Game) {}
}
