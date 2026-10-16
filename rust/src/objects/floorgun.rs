//! STUB — replaced by the port of ref/objects/floorgun.cc
#![allow(unused)]

use crate::engine::*;

pub struct FloorGun {
    pub base: Base,
}

impl FloorGun {
    pub fn new(x: i32, y: i32, z: i32) -> FloorGun {
        todo!("FloorGun")
    }
}

impl Obj for FloorGun {
    crate::obj_boilerplate!();
}

