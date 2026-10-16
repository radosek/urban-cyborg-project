//! Port of objects/beam.cc: one segment of the electric weapon.

use super::anim::Anim;
use crate::engine::*;
use crate::icache;

const X_SPEED: i32 = 1;
const FRAME_DELAY: i32 = 2;
const LIFE_FRAMES: i32 = 9;

pub struct Beam {
    pub base: Base,
    anim: Anim,
}

impl Beam {
    pub fn new(x: i32, y: i32, z: i32, friends: i32, speedx: i32, speedy: i32) -> Beam {
        let mut b = Base::new(x, y, z);
        b.images = icache::seq("beam/", 1, 5, ".pcx");
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.direction = if speedx < 0 { LEFT_DIR } else { RIGHT_DIR };
        b.current_image = if b.direction == LEFT_DIR { 0 } else { 8 };
        if b.direction == LEFT_DIR {
            b.x -= b.width;
        }
        b.energy = 10000000;
        b.strength = 10;
        b.speed_x = speedx + if b.direction == LEFT_DIR { -X_SPEED } else { X_SPEED };
        b.speed_y = speedy;
        b.friends = friends as u32 | FRIEND_FIREBALL | FRIEND_HS_BULLET | FRIEND_DEKOR;
        b.enemies = !b.friends;
        b.counter = LIFE_FRAMES;
        b.me = FRIEND_BEAM;
        Beam { base: b, anim: Anim::default() }
    }
}

impl Obj for Beam {
    crate::obj_boilerplate!();

    fn update(&mut self, _g: &mut Game) -> Upd {
        let b = &mut self.base;
        if b.counter != 0 {
            b.counter -= 1;
        }
        if b.counter == 0 {
            return Upd::Delete;
        }
        b.current_image = self.anim.next_frame(4, FRAME_DELAY);
        b.x += b.speed_x;
        b.y += b.speed_y;
        b.z += b.speed_z;
        if b.energy == 0 {
            return Upd::Delete;
        }
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, _g: &mut Game) {
        if other.base().me & self.base.friends == 0 {
            self.base.strength = 0;
        }
    }
}
