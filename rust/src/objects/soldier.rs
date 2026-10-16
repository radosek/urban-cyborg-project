//! STUB — replaced by the port of ref/objects/soldier.cc
#![allow(unused)]

use crate::engine::*;

pub struct BossSoldier {
    pub base: Base,
}

impl BossSoldier {
    pub fn new(x: i32, y: i32, z: i32) -> BossSoldier {
        todo!("BossSoldier")
    }
}

impl Obj for BossSoldier {
    crate::obj_boilerplate!();
}

pub struct Soldier {
    pub base: Base,
}

impl Soldier {
    pub fn new(x: i32, y: i32, z: i32) -> Soldier {
        todo!("Soldier")
    }
}

impl Obj for Soldier {
    crate::obj_boilerplate!();
}

