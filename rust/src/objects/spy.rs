//! STUB — replaced by the port of ref/objects/spy.cc
#![allow(unused)]

use crate::engine::*;

pub struct Spy {
    pub base: Base,
}

impl Spy {
    pub fn new(x: i32, y: i32, z: i32) -> Spy {
        todo!("Spy")
    }
    pub fn reset_spy(&mut self) { todo!() }
    pub fn get_spy(&self) -> i32 { todo!() }
}

impl Obj for Spy {
    crate::obj_boilerplate!();
}

