//! STUB — replaced by the port of ref/objects/tankwhel.cc
#![allow(unused)]

use crate::engine::*;

pub struct TankWheel {
    pub base: Base,
}

impl TankWheel {
    pub fn new(x: i32, y: i32, z: i32) -> TankWheel {
        todo!("TankWheel")
    }
    pub fn move_left(&mut self, x: i32) { todo!() }
    pub fn move_right(&mut self, x: i32) { todo!() }
    pub fn destroy_tank(&mut self, g: &mut Game) { todo!() }
}

impl Obj for TankWheel {
    crate::obj_boilerplate!();
}

