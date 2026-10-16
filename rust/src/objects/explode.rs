//! STUB — replaced by the port of ref/objects/explode.cc
#![allow(unused)]

use crate::engine::*;

pub struct Explosion {
    pub base: Base,
}

impl Explosion {
    pub fn new(x: i32, y: i32, z: i32, speed_x: i32, speed_y: i32, speed_z: i32, friends: i32) -> Explosion {
        todo!("Explosion")
    }
}

impl Obj for Explosion {
    crate::obj_boilerplate!();
}

