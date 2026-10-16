//! Port of ref/objects/tankwhel.cc

use super::anim::Anim;
use crate::engine::*;
use crate::icache;

const STATE_NONE: i32 = 0;
const MAX_Y_SPEED: i32 = 16;
const Y_ACCEL: i32 = 1;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TankWheel {
    pub base: Base,
    anim: Anim,
}

impl TankWheel {
    pub fn new(x: i32, y: i32, z: i32) -> TankWheel {
        let mut b = Base::new(x, y, z);
        b.images = icache::seq("boss/v/band", 1, 3, ".pcx");
        b.images.extend(icache::seq("boss/h/band", 1, 3, ".pcx"));
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.coll_width = b.width;
        b.coll_height = b.height;
        // Stand on the feet
        b.y -= b.images[0].h;
        b.energy = 10;
        b.strength = 10;
        b.direction = RIGHT_DIR;
        b.state = STATE_NONE;
        b.friends = !FRIEND_PLAYER;
        b.enemies = ENEMY_PLAYER;
        b.me = 0;
        TankWheel { base: b, anim: Anim::default() }
    }

    pub fn move_left(&mut self, x: i32) {
        self.base.x = x;
        self.base.current_image = self.anim.next_frame(2, 4);
    }

    pub fn move_right(&mut self, x: i32) {
        self.base.x = x;
        self.base.current_image = 2 - self.anim.next_frame(2, 4);
    }

    pub fn destroy_tank(&mut self, _g: &mut Game) {
        self.base.energy = 0;
    }
}

#[typetag::serde]
impl Obj for TankWheel {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        if b.energy <= 0 {
            return Upd::Delete;
        }
        if b.counter != 0 {
            b.counter -= 1;
        }
        // Fall or stop
        if g.check_floor(b.x, b.y + b.height, b.z) || g.check_floor(b.x + b.width, b.y + b.height, b.z) {
            b.speed_y = 0;
        } else if b.speed_y < MAX_Y_SPEED {
            b.speed_y += Y_ACCEL;
            b.speed_z = 0;
        }
        b.x += b.speed_x;
        b.y += b.speed_y;
        b.z += b.speed_z;
        if b.x < 0 {
            b.x = 0;
        }
        if b.z > MIN_Z {
            b.z = MIN_Z;
            b.speed_z = 0;
            b.counter = 0;
        }
        if b.z < MAX_Z {
            b.z = MAX_Z;
            b.speed_z = 0;
            b.counter = 0;
        }
        b.layer = b.z / TILE_TOP_HEIGHT;
        Upd::Keep
    }

    fn collision(&mut self, _other: &mut dyn Obj, _g: &mut Game) {}
}
