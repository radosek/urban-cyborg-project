//! STUB — replaced by the port of ref/objects/iceball.cc
#![allow(unused)]

use crate::engine::*;

pub struct IceBall {
    pub base: Base,
}

impl IceBall {
    pub fn new(x: i32, y: i32, z: i32, friends: i32, speedx: i32, speedy: i32, speedz: i32) -> IceBall {
        todo!("IceBall")
    }
}

impl Obj for IceBall {
    crate::obj_boilerplate!();
}

