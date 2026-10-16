//! STUB — replaced by the port of ref/objects/soldier3.cc
#![allow(unused)]

use crate::engine::*;

pub struct Soldier3 {
    pub base: Base,
}

impl Soldier3 {
    pub fn new(x: i32, y: i32, z: i32) -> Soldier3 {
        todo!("Soldier3")
    }
}

impl Obj for Soldier3 {
    crate::obj_boilerplate!();
}

