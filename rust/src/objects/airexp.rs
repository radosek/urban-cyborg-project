//! STUB — replaced by the port of ref/objects/airexp.cc
#![allow(unused)]

use crate::engine::*;

pub struct AirExplosion {
    pub base: Base,
}

impl AirExplosion {
    pub fn new(x: i32, y: i32, z: i32, speed_x: i32, speed_y: i32, speed_z: i32) -> AirExplosion {
        todo!("AirExplosion")
    }
}

impl Obj for AirExplosion {
    crate::obj_boilerplate!();
}

