//! Port of objects/plasma.cc: plasma gun shot.

use crate::engine::*;
use crate::icache;

const PLASMA_SPEED: i32 = 10;

pub struct Plasma {
    pub base: Base,
}

impl Plasma {
    pub fn new(x: i32, y: i32, z: i32, dir: i32, evilness: i32, damage: i32) -> Plasma {
        let mut b = Base::new(x, y, z);
        b.width = 1;
        b.height = 1;
        b.strength = damage;
        b.energy = 1;
        b.direction = dir;
        b.coll_width = b.width;
        b.coll_height = b.height;
        let name = if dir == LEFT_DIR { "plasmal/skottv.pcx" } else { "plasmal/skotth.pcx" };
        b.images = vec![icache::get(name)];
        b.me = FRIEND_PLASMA;
        b.friends = (evilness as u32 | b.me | FRIEND_DOOR | FRIEND_FIREBALL | FRIEND_ICEBALL | FRIEND_DEKOR) & !ENEMY_SCIENTIST;
        b.enemies = !b.friends;
        b.speed_x = if b.direction == RIGHT_DIR { PLASMA_SPEED } else { -PLASMA_SPEED };
        Plasma { base: b }
    }
}

impl Obj for Plasma {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        if b.energy == 0 {
            return Upd::Delete;
        }
        b.x += b.speed_x;
        b.counter += PLASMA_SPEED;
        if b.counter > 600 {
            return Upd::Delete;
        }
        let wx = if b.direction == LEFT_DIR { b.x } else { b.x + b.coll_width };
        if g.check_wall(wx, b.y, b.z) {
            b.energy = 0;
        }
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, _g: &mut Game) {
        if other.base().me & self.base.enemies != 0 {
            self.base.energy = 0;
        }
    }
}
