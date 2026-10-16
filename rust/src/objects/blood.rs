//! Port of objects/blood.cc: falling blood drop that leaves a stain.

use crate::engine::*;
use crate::{icache, rng};

const MAX_Y_SPEED: i32 = 8;
const Y_ACCEL: i32 = 1;

pub struct Blood {
    pub base: Base,
}

impl Blood {
    pub fn new(x: i32, y: i32, z: i32, speed_x: i32, speed_y: i32, speed_z: i32) -> Blood {
        let mut b = Base::new(x, y, z);
        b.images = icache::seq("dead/blod", 1, 3, ".pcx");
        b.width = b.images[0].w;
        b.height = b.images[0].h;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.speed_x = speed_x;
        b.speed_y = speed_y;
        b.speed_z = speed_z;
        b.current_image = if speed_x != 0 { 0 } else { rng::random() % 2 + 1 };
        b.direction = if speed_x > 0 { RIGHT_DIR } else { LEFT_DIR };
        b.me = 1;
        Blood { base: b }
    }
}

impl Obj for Blood {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        if g.check_floor(b.x, b.y + b.height, b.z) || g.check_floor(b.x + b.width, b.y + b.height, b.z) {
            b.speed_y = 0;
        } else if b.speed_y < MAX_Y_SPEED {
            b.speed_y += Y_ACCEL;
        }
        b.x += b.speed_x;
        b.y += b.speed_y;
        if g.check_floor(b.x, b.y + b.height, b.z) && g.check_floor(b.x + b.images[1].w, b.y + b.height, b.z) {
            g.create_dekoration(Blood::new(b.x, b.y, b.z, 0, 0, 0));
            return Upd::Delete;
        }
        Upd::Keep
    }
}
