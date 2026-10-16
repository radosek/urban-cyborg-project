//! STUB — replaced by the port of ref/objects/arnold.cc
#![allow(unused)]

use crate::engine::*;

pub struct Arnold {
    pub base: Base,
}

impl Arnold {
    pub fn new(x: i32, y: i32, z: i32) -> Arnold {
        todo!("Arnold")
    }
}

impl Obj for Arnold {
    crate::obj_boilerplate!();
}

pub struct BossArnold {
    pub base: Base,
}

impl BossArnold {
    pub fn new(x: i32, y: i32, z: i32) -> BossArnold {
        todo!("BossArnold")
    }
}

impl Obj for BossArnold {
    crate::obj_boilerplate!();
}

