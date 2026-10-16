//! STUB — replaced by the port of ref/objects/tank.cc
#![allow(unused)]

use crate::engine::*;

pub struct Tank {
    pub base: Base,
}

impl Tank {
    pub fn new(x: i32, y: i32, z: i32) -> Tank {
        todo!("Tank")
    }
}

impl Obj for Tank {
    crate::obj_boilerplate!();
}

