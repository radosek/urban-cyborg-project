//! Port of objects/smallexp.cc: small explosion effect.

use super::anim::Anim;
use crate::engine::*;
use crate::{icache, rng};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SmallExplosion {
    pub base: Base,
    anim: Anim,
}

impl SmallExplosion {
    pub fn new(x: i32, y: i32, z: i32) -> SmallExplosion {
        let mut b = Base::new(x, y, z);
        b.images = icache::seq("smallexp/", 1, 10, ".pcx");
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.y -= b.height / 2;
        b.x -= b.width / 2;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.energy = 1;
        b.friends = FRIEND_EXPLOSION;
        b.enemies = !b.friends;
        b.counter2 = rng::random() % 10;
        b.me = FRIEND_EXPLOSION;
        SmallExplosion { base: b, anim: Anim::default() }
    }
}

#[typetag::serde]
impl Obj for SmallExplosion {
    crate::obj_boilerplate!();

    fn update(&mut self, _g: &mut Game) -> Upd {
        let b = &mut self.base;
        b.counter += 1;
        if b.counter2 == 0 {
            b.counter2 = 30;
            b.strength = 4;
        } else {
            b.strength = 0;
            b.counter2 -= 1;
        }
        b.current_image = self.anim.next_frame(10, 3);
        if b.current_image > 5 {
            b.strength = 0;
        }
        if b.current_image >= 10 {
            return Upd::Delete;
        }
        Upd::Keep
    }

    fn collision(&mut self, _other: &mut dyn Obj, _g: &mut Game) {}
}
