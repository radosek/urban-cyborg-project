//! STUB — replaced by the port of ref/objects/bubble.cc
#![allow(unused)]

use crate::engine::*;

pub struct Bubble {
    pub base: Base,
}

impl Bubble {
    pub fn new(x: i32, y: i32, z: i32) -> Bubble {
        todo!("Bubble")
    }
}

impl Obj for Bubble {
    crate::obj_boilerplate!();
}

