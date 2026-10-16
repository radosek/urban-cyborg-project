//! STUB — replaced by the port of ref/objects/airgren.cc
#![allow(unused)]

use crate::engine::*;

pub struct AirGrenade {
    pub base: Base,
}

impl AirGrenade {
    pub fn new(x: i32, y: i32, z: i32, speedx: i32, speedy: i32, speedz: i32) -> AirGrenade {
        todo!("AirGrenade")
    }
}

impl Obj for AirGrenade {
    crate::obj_boilerplate!();
}

