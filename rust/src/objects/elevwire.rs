//! STUB — replaced by the port of ref/objects/elevwire.cc
#![allow(unused)]

use crate::engine::*;

pub struct ElevatorWire {
    pub base: Base,
}

impl ElevatorWire {
    pub fn new(x: i32, y: i32, z: i32) -> ElevatorWire {
        todo!("ElevatorWire")
    }
}

impl Obj for ElevatorWire {
    crate::obj_boilerplate!();
}

