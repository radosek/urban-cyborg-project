//! Port of ref/objects/hs_bulle.cc: the fast bullet fired by soldiers and floor guns.

use crate::engine::*;
use crate::icache;
use crate::objects::gnistor::Gnistor;

const HS_SPEED: i32 = 7;
const FIRE_SAMPLE: &str = "samples/gun2.wav";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HighSpeedBullet {
    pub base: Base,
}

impl HighSpeedBullet {
    /// `evilness` is OR-ed into the friend mask (callers pass e.g. `!FRIEND_PLAYER`).
    pub fn new(x: i32, y: i32, z: i32, dir: i32, evilness: i32, damage: i32) -> HighSpeedBullet {
        let mut b = Base::new(x, y, z);
        b.width = 1;
        b.height = 1;
        b.strength = damage;
        b.energy = 1;
        b.direction = dir;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.images.push(icache::get("hsbullet.pcx"));
        b.me = FRIEND_HS_BULLET;
        b.friends = (evilness as u32
            | b.me
            | FRIEND_DOOR
            | FRIEND_FIREBALL
            | FRIEND_ICEBALL
            | FRIEND_DEKOR
            | FRIEND_GRENADE
            | FRIEND_MINE)
            & !ENEMY_SCIENTIST;
        b.enemies = !b.friends;
        b.speed_x = if dir == RIGHT_DIR { HS_SPEED } else { -HS_SPEED };
        crate::sound::play_sfx(FIRE_SAMPLE);
        HighSpeedBullet { base: b }
    }
}

#[typetag::serde]
impl Obj for HighSpeedBullet {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        if b.energy == 0 {
            return Upd::Delete;
        }
        b.x += b.speed_x;
        b.counter += HS_SPEED;
        if b.counter > 600 {
            return Upd::Delete;
        }
        let wx = if b.direction == LEFT_DIR { b.x } else { b.x + b.coll_width };
        if g.check_wall(wx, b.y, b.z) {
            // gnistor_o(x, y + height, z, direction): the direction lands in Speed_X.
            g.create_effect(Gnistor::new(b.x, b.y + b.height, b.z, b.direction, 0, 0, 0));
            if crate::rng::random() % 3 == 0 {
                crate::sound::play_sfx("samples/ricochet.wav");
            }
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
