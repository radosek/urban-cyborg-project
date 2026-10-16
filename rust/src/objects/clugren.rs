//! Port of objects/clugren.cc: one bomblet of a cluster burst.

use super::explode::Explosion;
use crate::engine::*;
use crate::{icache, sound};

const EXPLOSION_SAMPLE: &str = "samples/heart_1.wav";
const ACCEL_COUNTER: i32 = 3;

pub struct ClusterGrenade {
    pub base: Base,
}

impl ClusterGrenade {
    pub fn new(x: i32, y: i32, z: i32, speedx: i32, speedy: i32, speedz: i32, friends: i32) -> ClusterGrenade {
        let mut b = Base::new(x, y, z);
        b.images = vec![icache::get("soldier2/kula.pcx")];
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.friends = friends as u32 | FRIEND_GRENADE | FRIEND_EXPLOSION | FRIEND_DOOR | FRIEND_DEKOR;
        b.enemies = !b.friends;
        b.me = FRIEND_GRENADE;
        b.speed_x = speedx;
        b.speed_z = speedz;
        b.speed_y = speedy;
        b.energy = 10000000;
        ClusterGrenade { base: b }
    }
}

impl Obj for ClusterGrenade {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        b.x += b.speed_x;
        b.y += b.speed_y;
        b.z += b.speed_z;
        b.z = b.z.clamp(MAX_Z, MIN_Z);
        b.layer = b.z / TILE_TOP_HEIGHT;
        if b.counter < 0 {
            b.speed_y += 1;
            b.counter = ACCEL_COUNTER;
        }
        b.counter -= 1;
        if b.energy <= 0 {
            return Upd::Delete;
        }
        if g.check_wall(b.x, b.y + TILE_SIDE_HEIGHT, b.z) || g.check_wall(b.x + b.width, b.y + TILE_SIDE_HEIGHT, b.z) {
            sound::play_sfx(EXPLOSION_SAMPLE);
            g.create_alwaysupdate(Explosion::new(b.x, b.y - TILE_SIDE_HEIGHT + 15, b.z, 0, 0, 0, b.friends as i32));
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
        sound::play_sfx(EXPLOSION_SAMPLE);
        g.create_alwaysupdate(Explosion::new(b.x, b.y - TILE_SIDE_HEIGHT + 15, b.z, 0, 0, 0, b.friends as i32));
    }
}
