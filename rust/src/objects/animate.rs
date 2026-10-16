//! STUB — replaced by the port of ref/objects/animate.cc
#![allow(unused)]

use crate::engine::*;

pub struct Animation {
    pub base: Base,
}

impl Animation {
    pub fn new(x: i32, y: i32, z: i32, name: &str, num_pics: i32, dek_frame: i32, speed_x: i32, speed_y: i32, speed_z: i32) -> Animation {
        todo!("Animation")
    }
}

impl Obj for Animation {
    crate::obj_boilerplate!();
}

