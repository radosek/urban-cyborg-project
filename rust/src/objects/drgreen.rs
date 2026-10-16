//! STUB — replaced by the port of ref/objects/drgreen.cc
#![allow(unused)]

use crate::engine::*;

pub struct BossDrGreen {
    pub base: Base,
}

impl BossDrGreen {
    pub fn new(x: i32, y: i32, z: i32) -> BossDrGreen {
        todo!("BossDrGreen")
    }
}

impl Obj for BossDrGreen {
    crate::obj_boilerplate!();
}

pub struct DrGreen {
    pub base: Base,
}

impl DrGreen {
    pub fn new(x: i32, y: i32, z: i32) -> DrGreen {
        todo!("DrGreen")
    }
}

impl Obj for DrGreen {
    crate::obj_boilerplate!();
}

