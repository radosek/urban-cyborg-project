//! STUB — replaced by the port of ref/objects/elevstat.cc
#![allow(unused)]

use crate::engine::*;

pub struct ElevatorStation {
    pub base: Base,
}

impl ElevatorStation {
    pub fn new(x: i32, y: i32, z: i32, dir: i32, card: i32) -> ElevatorStation {
        todo!("ElevatorStation")
    }
}

impl Obj for ElevatorStation {
    crate::obj_boilerplate!();
}

