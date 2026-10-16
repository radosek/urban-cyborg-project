//! STUB — replaced by the port of ref/objects/vaktm.cc
#![allow(unused)]

use crate::engine::*;

pub struct Vaktm {
    pub base: Base,
}

impl Vaktm {
    pub fn new(x: i32, y: i32, z: i32) -> Vaktm {
        todo!("Vaktm")
    }
}

impl Obj for Vaktm {
    crate::obj_boilerplate!();
}

