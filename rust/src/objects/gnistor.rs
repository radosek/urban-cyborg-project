//! STUB — replaced by the port of ref/objects/gnistor.cc
#![allow(unused)]

use crate::engine::*;

pub struct Gnistor {
    pub base: Base,
}

impl Gnistor {
    pub fn new(x: i32, y: i32, z: i32, speed_x: i32, speed_y: i32, speed_z: i32, direction: i32) -> Gnistor {
        todo!("Gnistor")
    }
}

impl Obj for Gnistor {
    crate::obj_boilerplate!();
}

