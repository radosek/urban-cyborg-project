//! STUB — replaced by the port of ref/objects/smallexp.cc
#![allow(unused)]

use crate::engine::*;

pub struct SmallExplosion {
    pub base: Base,
}

impl SmallExplosion {
    pub fn new(x: i32, y: i32, z: i32) -> SmallExplosion {
        todo!("SmallExplosion")
    }
}

impl Obj for SmallExplosion {
    crate::obj_boilerplate!();
}

