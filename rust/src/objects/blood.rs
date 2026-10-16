//! STUB — replaced by the port of ref/objects/blood.cc
#![allow(unused)]

use crate::engine::*;

pub struct Blood {
    pub base: Base,
}

impl Blood {
    pub fn new(x: i32, y: i32, z: i32, speed_x: i32, speed_y: i32, speed_z: i32) -> Blood {
        todo!("Blood")
    }
}

impl Obj for Blood {
    crate::obj_boilerplate!();
}

