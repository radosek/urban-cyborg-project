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
    pub fn find_elevator_station(&mut self, w: ObjId, sender: ObjId, g: &mut Game) { todo!() }
}

impl Obj for ElevatorWire {
    crate::obj_boilerplate!();
}

