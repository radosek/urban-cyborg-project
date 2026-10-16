//! Port of objects/bdyparts.cc: bouncing gibs and shells that settle as decorations.

use super::anim::Anim;
use crate::engine::*;
use crate::icache;

const MAX_Y_SPEED: i32 = 4;
const Y_ACCEL: i32 = 1;

const STATE_NONE: i32 = 0;
const STATE_DESTROY: i32 = 1;

pub struct Bodyparts {
    pub base: Base,
    anim: Anim,
}

impl Bodyparts {
    pub fn new(x: i32, y: i32, z: i32, name: &str, num_pics: i32, speed_x: i32, speed_y: i32, speed_z: i32) -> Bodyparts {
        let mut b = Base::new(x, y, z);
        b.images = icache::seq(name, 1, num_pics, ".pcx");
        b.width = b.images[0].w;
        b.height = b.images[0].h;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.y = y - b.height;
        b.me = 2;
        b.speed_x = speed_x;
        b.speed_y = speed_y;
        b.speed_z = speed_z;
        b.direction = if speed_x > 0 { RIGHT_DIR } else { LEFT_DIR };
        b.state = STATE_NONE;
        Bodyparts { base: b, anim: Anim::default() }
    }
}

impl Obj for Bodyparts {
    crate::obj_boilerplate!();

    fn spawned(&mut self, g: &mut Game) {
        let b = &mut self.base;
        if g.check_floor(b.x, b.y + b.height, b.z)
            || g.check_floor(b.x + b.width, b.y + b.height, b.z)
            || g.check_wall(b.x, b.y + b.height, b.z)
            || g.check_wall(b.x + b.width, b.y + b.height, b.z)
        {
            b.state = STATE_DESTROY;
        }
    }

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        if b.state == STATE_DESTROY {
            return Upd::Delete;
        }
        if g.check_wall(b.x + b.speed_x, b.y + b.height, b.z) || g.check_wall(b.x + b.width + b.speed_x, b.y + b.height, b.z) {
            b.speed_x = -(b.speed_x / 2);
        }
        if g.check_floor(b.x, b.y + b.height + b.speed_y, b.z) || g.check_floor(b.x + b.width, b.y + b.height + b.speed_y, b.z) {
            b.speed_y = -(b.speed_y / 2);
            if b.speed_y == 0 {
                return Upd::ToDekoration;
            }
        }
        if b.speed_y < MAX_Y_SPEED {
            b.speed_y += Y_ACCEL;
        }
        b.current_image = self.anim.next_frame(b.num_images() - 1, 4);
        if b.speed_y != 0 {
            b.x += b.speed_x;
        }
        b.y += b.speed_y;
        Upd::Keep
    }
}
