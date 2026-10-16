//! STUB — replaced by the port of ref/objects/soldier4.cc
#![allow(unused)]

use crate::engine::*;

pub struct Soldier4 {
    pub base: Base,
}

impl Soldier4 {
    pub fn new(x: i32, y: i32, z: i32) -> Soldier4 {
        todo!("Soldier4")
    }
}

impl Obj for Soldier4 {
    crate::obj_boilerplate!();
}

