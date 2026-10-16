//! STUB — replaced by the port of ref/objects/water.cc
#![allow(unused)]

use crate::engine::*;

pub struct Water {
    pub base: Base,
}

impl Water {
    pub fn new(x: i32, y: i32, z: i32) -> Water {
        todo!("Water")
    }
}

impl Obj for Water {
    crate::obj_boilerplate!();
}

