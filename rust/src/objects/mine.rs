//! STUB — replaced by the port of ref/objects/mine.cc
#![allow(unused)]

use crate::engine::*;

pub struct Mine {
    pub base: Base,
}

impl Mine {
    pub fn new(x: i32, y: i32, z: i32) -> Mine {
        todo!("Mine")
    }
}

impl Obj for Mine {
    crate::obj_boilerplate!();
}

