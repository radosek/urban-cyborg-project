//! STUB — replaced by the port of ref/objects/door.cc
#![allow(unused)]

use crate::engine::*;

pub struct Door {
    pub base: Base,
}

impl Door {
    pub fn new(x: i32, y: i32, z: i32, card: i32) -> Door {
        todo!("Door")
    }
}

impl Obj for Door {
    crate::obj_boilerplate!();
}

