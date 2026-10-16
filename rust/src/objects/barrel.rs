//! STUB — replaced by the port of ref/objects/barrel.cc
#![allow(unused)]

use crate::engine::*;

pub struct Barrel {
    pub base: Base,
}

impl Barrel {
    pub fn new(x: i32, y: i32, z: i32, type_: i32) -> Barrel {
        todo!("Barrel")
    }
}

impl Obj for Barrel {
    crate::obj_boilerplate!();
}

