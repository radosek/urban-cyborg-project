//! STUB — replaced by the port of ref/objects/frosen.cc
#![allow(unused)]

use crate::engine::*;

pub struct FrosenBody {
    pub base: Base,
}

impl FrosenBody {
    pub fn new(x: i32, y: i32, z: i32, dir: i32) -> FrosenBody {
        todo!("FrosenBody")
    }
}

impl Obj for FrosenBody {
    crate::obj_boilerplate!();
}

