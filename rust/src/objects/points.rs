//! Floating score label (objects/points.cc).

use crate::engine::*;
use crate::icache;

const EXIST_DELAY: i32 = 100;
const MOVE_DELAY: i32 = 2;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Points {
    pub base: Base,
}

impl Points {
    pub fn new(x: i32, y: i32, z: i32, score: i32) -> Points {
        let mut b = Base::new(x, y, z);
        b.images = (1..=10).map(|i| icache::get(&format!("points/{}00.pcx", i))).collect();
        b.current_image = (score / 100 - 1).clamp(0, 9);
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.energy = 1;
        b.counter = EXIST_DELAY;
        Points { base: b }
    }
}

#[typetag::serde]
impl Obj for Points {
    crate::obj_boilerplate!();

    fn update(&mut self, _g: &mut Game) -> Upd {
        let b = &mut self.base;
        if b.counter != 0 {
            b.counter -= 1;
        }
        if b.counter == 0 {
            return Upd::Delete;
        }
        if b.counter % MOVE_DELAY == 0 {
            b.y -= 1;
        }
        Upd::Keep
    }

    fn collision(&mut self, _other: &mut dyn Obj, _g: &mut Game) {}
}
