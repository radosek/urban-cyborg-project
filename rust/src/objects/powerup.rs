//! STUB — replaced by the port of ref/objects/powerup.cc
#![allow(unused)]

use crate::engine::*;

pub struct Powerup {
    pub base: Base,
}

impl Powerup {
    pub fn new(x: i32, y: i32, z: i32, type_: i32) -> Powerup {
        todo!("Powerup")
    }
}

impl Obj for Powerup {
    crate::obj_boilerplate!();
}

