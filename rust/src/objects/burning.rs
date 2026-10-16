//! STUB — replaced by the port of ref/objects/burning.cc
#![allow(unused)]

use crate::engine::*;

pub struct BurningBody {
    pub base: Base,
}

impl BurningBody {
    pub fn new(x: i32, y: i32, z: i32, move_: i32) -> BurningBody {
        todo!("BurningBody")
    }
}

impl Obj for BurningBody {
    crate::obj_boilerplate!();
}

