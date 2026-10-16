//! STUB — replaced by the port of ref/objects/dekor.cc
#![allow(unused)]

use crate::engine::*;

pub struct Dekor {
    pub base: Base,
}

impl Dekor {
    pub fn new(x: i32, y: i32, z: i32, filename: &str) -> Dekor {
        todo!("Dekor")
    }
}

impl Obj for Dekor {
    crate::obj_boilerplate!();
}

