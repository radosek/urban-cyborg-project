//! STUB — replaced by the port of ref/objects/checkp.cc
#![allow(unused)]

use crate::engine::*;

pub struct Checkpoint {
    pub base: Base,
}

impl Checkpoint {
    pub fn new(x: i32, y: i32, z: i32) -> Checkpoint {
        todo!("Checkpoint")
    }
}

impl Obj for Checkpoint {
    crate::obj_boilerplate!();
}

