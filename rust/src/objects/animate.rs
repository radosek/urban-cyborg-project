//! Port of objects/animate.cc: a falling one-shot animation that may leave a decoration frame.

use super::anim::Anim;
use crate::engine::*;
use crate::icache;

const MAX_Y_SPEED: i32 = 4;
const Y_ACCEL: i32 = 1;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Animation {
    pub base: Base,
    anim: Anim,
    dekoration_frame: i32,
}

impl Animation {
    pub fn new(x: i32, y: i32, z: i32, name: &str, num_pics: i32, dek_frame: i32, speed_x: i32, speed_y: i32, speed_z: i32) -> Animation {
        let mut b = Base::new(x, y, z);
        b.images = icache::seq(name, 1, num_pics, ".pcx");
        b.width = b.images[0].w;
        b.height = b.images[0].h;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.y = y - b.height;
        b.speed_x = speed_x;
        b.speed_y = speed_y;
        b.speed_z = speed_z;
        b.direction = if speed_x > 0 { RIGHT_DIR } else { LEFT_DIR };
        Animation { base: b, anim: Anim::default(), dekoration_frame: dek_frame }
    }
}

#[typetag::serde]
impl Obj for Animation {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        if b.state != 0 {
            return Upd::Delete;
        }
        if g.check_floor(b.x, b.y + b.height, b.z) || g.check_floor(b.x + b.width, b.y + b.height, b.z) {
            b.speed_y = 0;
        } else {
            b.y += b.speed_y;
            b.speed_y += Y_ACCEL;
            if b.speed_y > MAX_Y_SPEED {
                b.speed_y = MAX_Y_SPEED;
            }
        }
        let n = b.num_images();
        b.current_image = self.anim.next_frame(n, 8);
        if b.current_image == n {
            if self.dekoration_frame != -1 {
                // The original inserted itself into the decoration list and stayed in the
                // effect list for one more frame (drawn twice), then left the effects.
                b.current_image = self.dekoration_frame;
                b.state = 1;
                g.create_dekoration(self.clone());
                return Upd::Keep;
            }
            return Upd::Delete;
        }
        Upd::Keep
    }
}
