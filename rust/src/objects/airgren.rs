//! Port of ref/objects/airgren.cc

use super::airexp::AirExplosion;
use crate::engine::*;
use crate::icache;

const ACCEL_COUNTER: i32 = 3;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AirGrenade {
    pub base: Base,
}

impl AirGrenade {
    pub fn new(x: i32, y: i32, z: i32, speedx: i32, speedy: i32, speedz: i32) -> AirGrenade {
        let mut b = Base::new(x, y, z);
        b.images = vec![icache::get("airgren.pcx")];
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.friends = FRIEND_GRENADE | FRIEND_EXPLOSION | FRIEND_DOOR | FRIEND_DEKOR;
        b.enemies = !b.friends;
        b.me = FRIEND_GRENADE;
        b.speed_x = speedx;
        b.speed_z = speedz;
        b.speed_y = speedy;
        b.energy = 10000000;
        b.strength = 0;
        AirGrenade { base: b }
    }
}

#[typetag::serde]
impl Obj for AirGrenade {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        b.x += b.speed_x;
        b.y += b.speed_y;
        b.z += b.speed_z;
        if b.z > MIN_Z {
            b.z = MIN_Z;
        }
        if b.z < MAX_Z {
            b.z = MAX_Z;
        }
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
            g.create_alwaysupdate(AirExplosion::new(b.x, b.y - TILE_SIDE_HEIGHT + 15, b.z, 0, 0, 0));
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
        g.create_alwaysupdate(AirExplosion::new(b.x, b.y - TILE_SIDE_HEIGHT + 15, b.z, 0, 0, 0));
    }
}
