//! STUB — replaced by the port of ref/objects/science.cc
#![allow(unused)]

use crate::engine::*;

pub struct BossScientist {
    pub base: Base,
}

impl BossScientist {
    pub fn new(x: i32, y: i32, z: i32) -> BossScientist {
        todo!("BossScientist")
    }
}

impl Obj for BossScientist {
    crate::obj_boilerplate!();
}

pub struct Scientist {
    pub base: Base,
}

impl Scientist {
    pub fn new(x: i32, y: i32, z: i32) -> Scientist {
        todo!("Scientist")
    }
}

impl Obj for Scientist {
    crate::obj_boilerplate!();
}

