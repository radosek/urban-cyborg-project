//! Port of ref/objects/airexp.cc

use super::anim::Anim;
use crate::engine::*;
use crate::icache;

const EXPLODE_SPEED: i32 = 4;
const EXPLODE_DAMAGE: i32 = 30;
const EXPLODE_FRAMES: i32 = 4;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AirExplosion {
    pub base: Base,
    anim: Anim,
}

impl AirExplosion {
    /// The original ignores the speed arguments.
    pub fn new(x: i32, y: i32, z: i32, _speed_x: i32, _speed_y: i32, _speed_z: i32) -> AirExplosion {
        let mut b = Base::new(x, y, z);
        b.images = icache::seq("expl/", 1, 10, ".pcx");
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.y -= b.height / 2;
        b.x -= b.width / 2;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.energy = 1;
        b.strength = EXPLODE_DAMAGE;
        b.friends = FRIEND_EXPLOSION;
        b.enemies = !b.friends;
        b.counter = EXPLODE_FRAMES;
        b.me = FRIEND_EXPLOSION;
        AirExplosion { base: b, anim: Anim::default() }
    }
}

#[typetag::serde]
impl Obj for AirExplosion {
    crate::obj_boilerplate!();

    fn spawned(&mut self, g: &mut Game) {
        g.enable_earthquake();
    }

    fn update(&mut self, _g: &mut Game) -> Upd {
        let b = &mut self.base;
        if b.counter != 0 {
            b.counter -= 1;
        }
        if b.counter == 0 {
            b.strength = 0;
        }
        b.current_image = self.anim.next_frame(10, EXPLODE_SPEED);
        if b.current_image >= 10 {
            return Upd::Delete;
        }
        Upd::Keep
    }

    fn collision(&mut self, _other: &mut dyn Obj, _g: &mut Game) {}
}
