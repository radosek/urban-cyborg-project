//! STUB — replaced by the port of ref/objects/dolph.cc
#![allow(unused)]

use crate::engine::*;

pub struct BossDolph {
    pub base: Base,
}

impl BossDolph {
    pub fn new(x: i32, y: i32, z: i32) -> BossDolph {
        todo!("BossDolph")
    }
}

impl Obj for BossDolph {
    crate::obj_boilerplate!();
}

pub struct Dolph {
    pub base: Base,
}

impl Dolph {
    pub fn new(x: i32, y: i32, z: i32) -> Dolph {
        todo!("Dolph")
    }
}

impl Obj for Dolph {
    crate::obj_boilerplate!();
}

