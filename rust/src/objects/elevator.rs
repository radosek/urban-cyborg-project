//! STUB — replaced by the port of ref/objects/elevator.cc
#![allow(unused)]

use crate::engine::*;

pub struct Elevator {
    pub base: Base,
}

impl Elevator {
    pub fn new(x: i32, y: i32, z: i32) -> Elevator {
        todo!("Elevator")
    }
}

impl Obj for Elevator {
    crate::obj_boilerplate!();
}

