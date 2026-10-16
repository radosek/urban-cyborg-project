//! STUB — replaced by the port of ref/objects/exptile.cc
#![allow(unused)]

use crate::engine::*;

pub struct ExplosiveTile {
    pub base: Base,
}

impl ExplosiveTile {
    pub fn new(x: i32, y: i32, z: i32, type_: i32) -> ExplosiveTile {
        todo!("ExplosiveTile")
    }
}

impl Obj for ExplosiveTile {
    crate::obj_boilerplate!();
}

