//! Port of objects/gnistor.cc: sparks where a bullet hits a wall.

use super::anim::Anim;
use crate::engine::*;
use crate::icache;

pub struct Gnistor {
    pub base: Base,
    anim: Anim,
}

impl Gnistor {
    pub fn new(x: i32, y: i32, z: i32, speed_x: i32, speed_y: i32, speed_z: i32, direction: i32) -> Gnistor {
        let _ = (speed_x, speed_y, speed_z);
        let mut b = Base::new(x, y, z);
        let dir = if direction == LEFT_DIR { "gnist/v/" } else { "gnist/h/" };
        b.images = icache::seq(dir, 1, 3, ".pcx");
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.y -= b.height / 2;
        b.x -= b.width / 2;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.strength = 10;
        b.friends = !0;
        b.enemies = 0;
        b.me = 3;
        Gnistor { base: b, anim: Anim::default() }
    }
}

impl Obj for Gnistor {
    crate::obj_boilerplate!();

    fn update(&mut self, _g: &mut Game) -> Upd {
        let b = &mut self.base;
        b.counter += 1;
        b.current_image = self.anim.next_frame(3, 3);
        if b.current_image >= 3 {
            return Upd::Delete;
        }
        Upd::Keep
    }

    fn collision(&mut self, _other: &mut dyn Obj, _g: &mut Game) {}
}
