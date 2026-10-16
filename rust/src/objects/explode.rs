//! Port of objects/explode.cc: the big explosion.

use super::anim::Anim;
use crate::engine::*;
use crate::{icache, sound};

const EXPLODE_SAMPLE: &str = "samples/ex12.wav";
const EXPLODE_SPEED: i32 = 4;
const EXPLODE_DAMAGE: i32 = 10;
const EXPLODE_FRAMES: i32 = 2;

pub struct Explosion {
    pub base: Base,
    anim: Anim,
    quake_started: bool,
}

impl Explosion {
    pub fn new(x: i32, y: i32, z: i32, speed_x: i32, speed_y: i32, speed_z: i32, friends: i32) -> Explosion {
        // The speed arguments are ignored, as in the C++.
        let _ = (speed_x, speed_y, speed_z);
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
        b.friends = FRIEND_EXPLOSION | friends as u32;
        b.enemies = !b.friends;
        b.counter = EXPLODE_FRAMES;
        b.me = FRIEND_EXPLOSION;
        sound::play_sfx(EXPLODE_SAMPLE);
        Explosion { base: b, anim: Anim::default(), quake_started: false }
    }
}

impl Obj for Explosion {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        // The C++ constructor enabled the earthquake; it needs the engine, so it happens here.
        if !self.quake_started {
            self.quake_started = true;
            g.enable_earthquake();
        }
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
