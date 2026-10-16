//! STUB — replaced by the port of ref/objects/crycheck.cc
#![allow(unused)]

use crate::engine::*;

pub struct BossCrycheck {
    pub base: Base,
}

impl BossCrycheck {
    pub fn new(x: i32, y: i32, z: i32) -> BossCrycheck {
        todo!("BossCrycheck")
    }
}

impl Obj for BossCrycheck {
    crate::obj_boilerplate!();
}

pub struct Crycheck {
    pub base: Base,
}

impl Crycheck {
    pub fn new(x: i32, y: i32, z: i32) -> Crycheck {
        todo!("Crycheck")
    }
}

impl Obj for Crycheck {
    crate::obj_boilerplate!();
}

