//! STUB — replaced by the port of ref/objects/missile.cc
#![allow(unused)]

use crate::engine::*;

pub struct Missile {
    pub base: Base,
}

impl Missile {
    pub fn new(x: i32, y: i32, z: i32, speed: i32) -> Missile {
        todo!("Missile")
    }
}

impl Obj for Missile {
    crate::obj_boilerplate!();
}

