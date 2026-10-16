//! Port of objects/iceball.cc: ice-maker projectile.

use super::anim::Anim;
use crate::engine::*;
use crate::icache;

const X_SPEED: i32 = 1;
const FRAME_DELAY: i32 = 4;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct IceBall {
    pub base: Base,
    anim: Anim,
}

impl IceBall {
    pub fn new(x: i32, y: i32, z: i32, friends: i32, speedx: i32, speedy: i32, speedz: i32) -> IceBall {
        let mut b = Base::new(x, y, z);
        b.images = icache::seq("ice/v/", 1, 8, ".pcx");
        b.images.extend(icache::seq("ice/h/", 1, 8, ".pcx"));
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.direction = if speedx < 0 { LEFT_DIR } else { RIGHT_DIR };
        b.current_image = if b.direction == LEFT_DIR { 0 } else { 8 };
        // Stand on its feet
        b.y -= b.height / 2;
        if b.direction == LEFT_DIR {
            b.x -= b.width;
        }
        b.energy = 10000000;
        b.strength = 4;
        b.speed_x = speedx + if b.direction == LEFT_DIR { -X_SPEED } else { X_SPEED };
        b.speed_y = speedy;
        b.speed_z = speedz;
        b.friends = friends as u32 | FRIEND_POWERUP | FRIEND_ICEBALL | FRIEND_HS_BULLET | FRIEND_DEKOR;
        b.enemies = !b.friends;
        b.me = FRIEND_ICEBALL;
        IceBall { base: b, anim: Anim::default() }
    }
}

#[typetag::serde]
impl Obj for IceBall {
    crate::obj_boilerplate!();

    fn update(&mut self, _g: &mut Game) -> Upd {
        let b = &mut self.base;
        let first = if b.direction == LEFT_DIR { 0 } else { 8 };
        b.current_image = first + self.anim.next_frame(8, FRAME_DELAY);
        b.x += b.speed_x;
        b.y += b.speed_y;
        b.z += b.speed_z;
        if b.current_image >= first + 8 {
            return Upd::Delete;
        }
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
