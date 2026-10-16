//! STUB — replaced by the port of ref/objects/compscrn.cc
#![allow(unused)]

use crate::engine::*;

pub struct ComputerScreen {
    pub base: Base,
}

impl ComputerScreen {
    pub fn new(x: i32, y: i32, z: i32, type_: i32) -> ComputerScreen {
        todo!("ComputerScreen")
    }
}

impl Obj for ComputerScreen {
    crate::obj_boilerplate!();
}

