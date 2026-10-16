//! Port of ref/objects/spy.cc: an invisible-in-effect trip sensor that a floor gun polls.

use crate::engine::*;
use crate::icache;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Spy {
    pub base: Base,
    victim: i32,
}

impl Spy {
    pub fn new(x: i32, y: i32, z: i32) -> Spy {
        let mut b = Base::new(x, y, z);
        b.images.push(icache::get("mine2.pcx"));
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.x += TILE_WIDTH / 2;
        b.x -= b.width / 2;
        b.y -= b.height;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.energy = 1;
        // Fall to the floor
        b.speed_y = 5;
        b.me = FRIEND_MINE;
        // The original never initialised `victim` (BUGS.md #22).
        Spy { base: b, victim: 0 }
    }
    pub fn reset_spy(&mut self) {
        self.victim = 0;
    }
    pub fn get_spy(&self) -> i32 {
        self.victim
    }
}

#[typetag::serde]
impl Obj for Spy {
    crate::obj_boilerplate!();

    fn collision(&mut self, _other: &mut dyn Obj, _g: &mut Game) {
        self.victim = 1;
    }
}
