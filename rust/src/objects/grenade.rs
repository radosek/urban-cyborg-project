//! STUB — replaced by the port of ref/objects/grenade.cc
#![allow(unused)]

use crate::engine::*;

pub struct Grenade {
    pub base: Base,
}

impl Grenade {
    pub fn new(x: i32, y: i32, z: i32, speedx: i32, speedy: i32, speedz: i32) -> Grenade {
        todo!("Grenade")
    }
}

impl Obj for Grenade {
    crate::obj_boilerplate!();
}

