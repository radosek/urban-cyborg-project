//! STUB — replaced by the port of ref/objects/bdyparts.cc
#![allow(unused)]

use crate::engine::*;

pub struct Bodyparts {
    pub base: Base,
}

impl Bodyparts {
    pub fn new(x: i32, y: i32, z: i32, name: &str, num_pics: i32, speed_x: i32, speed_y: i32, speed_z: i32) -> Bodyparts {
        todo!("Bodyparts")
    }
}

impl Obj for Bodyparts {
    crate::obj_boilerplate!();
}

