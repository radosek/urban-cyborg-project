//! STUB — replaced by the port of ref/objects/bubblmak.cc
#![allow(unused)]

use crate::engine::*;

pub struct BubbleMaker {
    pub base: Base,
}

impl BubbleMaker {
    pub fn new(x: i32, y: i32, z: i32) -> BubbleMaker {
        todo!("BubbleMaker")
    }
}

impl Obj for BubbleMaker {
    crate::obj_boilerplate!();
}

