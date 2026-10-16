//! STUB — replaced by the port of ref/objects/einstein.cc
#![allow(unused)]

use crate::engine::*;

pub struct BossEinstein {
    pub base: Base,
}

impl BossEinstein {
    pub fn new(x: i32, y: i32, z: i32) -> BossEinstein {
        todo!("BossEinstein")
    }
}

impl Obj for BossEinstein {
    crate::obj_boilerplate!();
}

pub struct Einstein {
    pub base: Base,
}

impl Einstein {
    pub fn new(x: i32, y: i32, z: i32) -> Einstein {
        todo!("Einstein")
    }
}

impl Obj for Einstein {
    crate::obj_boilerplate!();
}

