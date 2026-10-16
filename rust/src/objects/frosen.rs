//! Frozen body that shatters when hit (port of ref/objects/frosen.cc).

use crate::engine::*;
use crate::icache;
use crate::objects::ai_sci::stand_on_feet;
use crate::objects::animate::Animation;

const STATE_FROSEN: i32 = 0;

pub struct FrosenBody {
    pub base: Base,
}

impl FrosenBody {
    pub fn new(x: i32, y: i32, z: i32, dir: i32) -> FrosenBody {
        let mut b = Base::new(x, y, z);
        b.images.push(icache::get(if dir == LEFT_DIR { "iced/v/6.pcx" } else { "iced/h/6.pcx" }));
        stand_on_feet(&mut b);
        b.friends = 0;
        b.enemies = !0;
        b.energy = 1;
        b.direction = dir;
        b.me = FRIEND_FROZENBODY;
        b.state = STATE_FROSEN;
        b.score = 200;
        FrosenBody { base: b }
    }
}

impl Obj for FrosenBody {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &self.base;
        if b.energy <= 0 {
            let name = if b.direction == LEFT_DIR { "iced/v/" } else { "iced/h/" };
            g.create_effect(Animation::new(b.x, b.y + b.height, b.z, name, 5, 4, 0, 0, 0));
            return Upd::Delete;
        }
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, _g: &mut Game) {
        const SHATTER: u32 = ENEMY_HS_BULLET | ENEMY_GRENADE | ENEMY_EXPLOSION | ENEMY_FIREBALL | ENEMY_PLASMA;
        if other.base().me & SHATTER != 0 {
            self.base.energy = 0;
        }
    }
}
