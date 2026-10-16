//! STUB — replaced by the port of ref/objects/plasma.cc
#![allow(unused)]

use crate::engine::*;

pub struct Plasma {
    pub base: Base,
}

impl Plasma {
    pub fn new(x: i32, y: i32, z: i32, dir: i32, evilness: i32, damage: i32) -> Plasma {
        todo!("Plasma")
    }
}

impl Obj for Plasma {
    crate::obj_boilerplate!();
}

