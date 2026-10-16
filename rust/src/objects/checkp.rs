//! Checkpoint (objects/checkp.cc): touching it saves the player's respawn position.

use crate::engine::*;
use crate::icache;

pub struct Checkpoint {
    pub base: Base,
}

impl Checkpoint {
    pub fn new(x: i32, y: i32, z: i32) -> Checkpoint {
        let mut b = Base::new(x, y, z);
        b.images = vec![icache::get("items/check.pcx")];
        b.current_image = 0;
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
        b.friends = FRIEND_PLAYER;
        b.enemies = !b.friends;
        b.me = FRIEND_CHECKPOINT;
        Checkpoint { base: b }
    }
}

impl Obj for Checkpoint {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        if g.check_floor(b.x, b.y + b.height, b.z) || g.check_floor(b.x + b.width, b.y + b.height, b.z) {
            b.speed_y = 0;
        } else {
            b.y += b.speed_y;
        }
        // Delete if already used
        if b.energy == 0 {
            return Upd::Delete;
        }
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, g: &mut Game) {
        if self.base.energy == 0 || self.base.friends & other.base().me == 0 {
            return;
        }
        self.base.energy = 0;
        g.push_message("Checkpoint");
    }
}
