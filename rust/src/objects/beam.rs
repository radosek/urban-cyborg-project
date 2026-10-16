//! STUB — replaced by the port of ref/objects/beam.cc
#![allow(unused)]

use crate::engine::*;

pub struct Beam {
    pub base: Base,
}

impl Beam {
    pub fn new(x: i32, y: i32, z: i32, friends: i32, speedx: i32, speedy: i32) -> Beam {
        todo!("Beam")
    }
}

impl Obj for Beam {
    crate::obj_boilerplate!();
}

