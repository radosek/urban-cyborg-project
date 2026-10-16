//! STUB — replaced by the port of ref/objects/tankwhel.cc
#![allow(unused)]

use crate::engine::*;

pub struct TankWheel {
    pub base: Base,
}

impl TankWheel {
    pub fn new(x: i32, y: i32, z: i32) -> TankWheel {
        todo!("TankWheel")
    }
}

impl Obj for TankWheel {
    crate::obj_boilerplate!();
}

