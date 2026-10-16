//! STUB — replaced by the port of ref/objects/rambo.cc
#![allow(unused)]

use crate::engine::*;

pub struct BossRambo {
    pub base: Base,
}

impl BossRambo {
    pub fn new(x: i32, y: i32, z: i32) -> BossRambo {
        todo!("BossRambo")
    }
}

impl Obj for BossRambo {
    crate::obj_boilerplate!();
}

pub struct Rambo {
    pub base: Base,
}

impl Rambo {
    pub fn new(x: i32, y: i32, z: i32) -> Rambo {
        todo!("Rambo")
    }
}

impl Obj for Rambo {
    crate::obj_boilerplate!();
}

