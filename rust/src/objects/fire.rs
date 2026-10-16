//! STUB — replaced by the port of ref/objects/fire.cc
#![allow(unused)]

use crate::engine::*;

pub struct Fire {
    pub base: Base,
}

impl Fire {
    pub fn new(x: i32, y: i32, z: i32, size: i32) -> Fire {
        todo!("Fire")
    }
}

impl Obj for Fire {
    crate::obj_boilerplate!();
}

