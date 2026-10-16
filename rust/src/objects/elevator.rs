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
    pub fn activate(&mut self, dir: i32, g: &mut Game) { todo!() }
    pub fn deactivate(&mut self, g: &mut Game) { todo!() }
    pub fn send_elevator(&mut self, dir: i32, g: &mut Game) { todo!() }
}

impl Obj for Elevator {
    crate::obj_boilerplate!();
}

