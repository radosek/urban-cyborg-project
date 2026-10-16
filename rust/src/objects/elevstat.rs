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
    pub fn start_elevator(&mut self, g: &mut Game) -> i32 { todo!() }
    pub fn send_elevator(&mut self, dir: i32, g: &mut Game) { todo!() }
    pub fn set_elevator_station(&mut self, station: ObjId) { todo!() }
}

impl Obj for ElevatorStation {
    crate::obj_boilerplate!();
}

