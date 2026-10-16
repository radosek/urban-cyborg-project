//! Port of ref/objects/rocket.cc

use super::bubble::Bubble;
use super::explode::Explosion;
use crate::engine::*;
use crate::{icache, rng::random, sound};

const EXPLOSION_SAMPLE: &str = "samples/heart_1.wav";
const FIRE_SAMPLE: &str = "samples/grenade.wav";

pub struct Rocket {
    pub base: Base,
}

impl Rocket {
    pub fn new(x: i32, y: i32, z: i32, speed: i32, mode: i32) -> Rocket {
        let mut b = Base::with_mode(x, y, z, mode);
        sound::play_sfx(FIRE_SAMPLE);
        b.images = vec![icache::get(if speed > 0 { "grenade.pcx" } else { "grenade2.pcx" })];
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.me = FRIEND_GRENADE;
        b.friends = b.me | FRIEND_GRENADE | FRIEND_FIREBALL | FRIEND_EXPLOSION | FRIEND_DOOR;
        b.enemies = !b.friends;
        b.speed_x = if b.mode == MODE_NORMAL { speed } else { speed / 2 };
        b.direction = if speed < 0 { LEFT_DIR } else { RIGHT_DIR };
        b.energy = 10000000;
        b.strength = 0;
        Rocket { base: b }
    }
}

impl Obj for Rocket {
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
        if g.check_wall(b.x, b.y, b.z) || g.check_wall(b.x + b.width, b.y, b.z) {
            sound::play_sfx(EXPLOSION_SAMPLE);
            g.create_object(Explosion::new(b.x, b.y - TILE_SIDE_HEIGHT + 15, b.z, 0, 0, 0, 0));
            return Upd::Delete;
        }
        if b.mode == MODE_WATER {
            let off = if b.speed_x < 0 { 20 } else { 30 };
            if random() % 10 == 0 {
                g.create_object(Bubble::new(b.x - off, b.y, b.z));
            }
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
        g.create_object(Explosion::new(b.x, b.y - TILE_SIDE_HEIGHT + 15, b.z, 0, 0, 0, 0));
    }
}
