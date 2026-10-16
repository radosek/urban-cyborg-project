//! STUB — replaced by the port of ref/objects/fireball.cc
#![allow(unused)]

use crate::engine::*;

pub struct FireBall {
    pub base: Base,
}

impl FireBall {
    pub fn new(x: i32, y: i32, z: i32, friends: i32, speedx: i32, speedy: i32, speedz: i32, strength: i32) -> FireBall {
        todo!("FireBall")
    }
}

impl Obj for FireBall {
    crate::obj_boilerplate!();
}

