//! STUB — replaced by the port of ref/objects/points.cc
#![allow(unused)]

use crate::engine::*;

pub struct Points {
    pub base: Base,
}

impl Points {
    pub fn new(x: i32, y: i32, z: i32, score: i32) -> Points {
        todo!("Points")
    }
}

impl Obj for Points {
    crate::obj_boilerplate!();
}

