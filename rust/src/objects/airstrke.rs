//! STUB — replaced by the port of ref/objects/airstrke.cc
#![allow(unused)]

use crate::engine::*;

pub struct Airstrike {
    pub base: Base,
}

impl Airstrike {
    pub fn new(x: i32, y: i32, z: i32) -> Airstrike {
        todo!("Airstrike")
    }
}

impl Obj for Airstrike {
    crate::obj_boilerplate!();
}

