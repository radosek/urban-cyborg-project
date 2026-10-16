//! STUB — replaced by the port of ref/objects/hs_bulle.cc
#![allow(unused)]

use crate::engine::*;

pub struct HighSpeedBullet {
    pub base: Base,
}

impl HighSpeedBullet {
    pub fn new(x: i32, y: i32, z: i32, dir: i32, evilness: i32, damage: i32) -> HighSpeedBullet {
        todo!("HighSpeedBullet")
    }
}

impl Obj for HighSpeedBullet {
    crate::obj_boilerplate!();
}

