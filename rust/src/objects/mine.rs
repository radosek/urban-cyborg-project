//! Port of objects/mine.cc: blinking floor mine that goes off in three blasts.

use super::explode::Explosion;
use crate::engine::*;
use crate::{icache, rng};

const STRENGTH_MINE: i32 = 100;
const STATE_NONE: i32 = 0x00;
const STATE_EXPLODE: i32 = 0x01;

pub struct Mine {
    pub base: Base,
}

impl Mine {
    pub fn new(x: i32, y: i32, z: i32) -> Mine {
        let mut b = Base::new(x, y, z);
        b.images = vec![icache::get("mine1.pcx"), icache::get("mine2.pcx")];
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.x += TILE_WIDTH / 2;
        b.x -= b.width / 2;
        b.y -= b.height;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.energy = 1;
        b.strength = STRENGTH_MINE;
        // Fall to the floor
        b.speed_y = 5;
        b.counter = rng::random() % 50;
        b.me = FRIEND_MINE;
        b.friends = !FRIEND_PLAYER;
        b.enemies = !b.friends;
        b.state = STATE_NONE;
        Mine { base: b }
    }
}

impl Obj for Mine {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        if b.counter != 0 {
            b.counter -= 1;
        }
        if b.counter == 0 {
            match b.state {
                STATE_NONE => {
                    b.current_image = if b.current_image == 0 { 1 } else { 0 };
                    b.counter = if b.current_image != 0 { 100 } else { 10 };
                }
                STATE_EXPLODE => {
                    b.strength = 0;
                    b.counter = 10;
                    b.counter2 += 1;
                    if b.counter2 >= 3 {
                        b.energy = 0;
                    }
                    match b.counter2 {
                        1 => g.create_object(Explosion::new(b.x, b.y - TILE_SIDE_HEIGHT, b.z, 0, 0, 0, 0)),
                        2 => g.create_object(Explosion::new(b.x - TILE_WIDTH, b.y - TILE_SIDE_HEIGHT, b.z, 0, 0, 0, 0)),
                        3 => g.create_object(Explosion::new(b.x + TILE_WIDTH, b.y - TILE_SIDE_HEIGHT, b.z, 0, 0, 0, 0)),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        // Fall or stop
        if !(g.check_floor(b.x, b.y + b.height, b.z) || g.check_floor(b.x + b.width, b.y + b.height, b.z)) {
            b.y += b.speed_y;
        }
        // Delete if already used
        if b.energy == 0 {
            return Upd::Delete;
        }
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, _g: &mut Game) {
        let b = &mut self.base;
        if b.energy == 0 {
            return;
        }
        let who = other.base().me;
        if who == 0 || b.friends & who != 0 {
            return;
        }
        if b.state != STATE_EXPLODE {
            b.state = STATE_EXPLODE;
            b.counter = 0;
            b.energy = 100000;
        }
    }
}
