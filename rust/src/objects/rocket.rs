//! STUB — replaced by the port of ref/objects/rocket.cc
#![allow(unused)]

use crate::engine::*;

pub struct Rocket {
    pub base: Base,
}

impl Rocket {
    pub fn new(x: i32, y: i32, z: i32, speed: i32, mode: i32) -> Rocket {
        todo!("Rocket")
    }
}

impl Obj for Rocket {
    crate::obj_boilerplate!();
}

