//! Port of ref/objects/missile.cc

use super::anim::Anim;
use super::explode::Explosion;
use crate::engine::*;
use crate::icache;

const FIRE_SAMPLE: &str = "samples/missile1.wav";

pub struct Missile {
    pub base: Base,
    anim: Anim,
}

impl Missile {
    pub fn new(x: i32, y: i32, z: i32, speed: i32) -> Missile {
        let mut b = Base::new(x, y, z);
        crate::sound::play_sfx(FIRE_SAMPLE);
        let dir = if speed > 0 { "skotth" } else { "skottv" };
        b.images = (0..2).map(|i| icache::get(&format!("{}/{}.pcx", dir, i + 1))).collect();
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.friends = FRIEND_GRENADE | FRIEND_EXPLOSION | FRIEND_DOOR | FRIEND_HS_BULLET;
        b.enemies = !0;
        b.me = FRIEND_GRENADE;
        b.speed_x = speed;
        b.direction = if speed < 0 { LEFT_DIR } else { RIGHT_DIR };
        b.energy = 10000000;
        b.strength = 0;
        Missile { base: b, anim: Anim::default() }
    }
}

impl Obj for Missile {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        b.x += b.speed_x;
        if b.x < 0 {
            return Upd::Delete;
        }
        if b.energy <= 0 {
            return Upd::Delete;
        }
        b.counter += b.speed_x.abs();
        if b.counter > 600 {
            return Upd::Delete;
        }
        b.current_image = self.anim.next_frame(1, 5);
        if g.check_wall(b.x, b.y, b.z) || g.check_wall(b.x + b.width, b.y, b.z) {
            g.create_object(Explosion::new(b.x, b.y - TILE_SIDE_HEIGHT + 15, b.z, 0, 0, 0, 0));
            return Upd::Delete;
        }
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, g: &mut Game) {
        let b = &mut self.base;
        if b.energy == 0 {
            return;
        }
        if b.friends & other.base().me != 0 {
            return;
        }
        b.energy = 0;
        g.create_object(Explosion::new(b.x, b.y - TILE_SIDE_HEIGHT + 15, b.z, 0, 0, 0, 0));
    }
}
