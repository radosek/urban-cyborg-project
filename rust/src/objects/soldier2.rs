//! STUB — replaced by the port of ref/objects/soldier2.cc
#![allow(unused)]

use crate::engine::*;

pub struct BossSoldier2 {
    pub base: Base,
}

impl BossSoldier2 {
    pub fn new(x: i32, y: i32, z: i32) -> BossSoldier2 {
        todo!("BossSoldier2")
    }
}

impl Obj for BossSoldier2 {
    crate::obj_boilerplate!();
}

pub struct Soldier2 {
    pub base: Base,
}

impl Soldier2 {
    pub fn new(x: i32, y: i32, z: i32) -> Soldier2 {
        todo!("Soldier2")
    }
}

impl Obj for Soldier2 {
    crate::obj_boilerplate!();
}

