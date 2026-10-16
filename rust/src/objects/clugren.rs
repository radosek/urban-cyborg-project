//! STUB — replaced by the port of ref/objects/clugren.cc
#![allow(unused)]

use crate::engine::*;

pub struct ClusterGrenade {
    pub base: Base,
}

impl ClusterGrenade {
    pub fn new(x: i32, y: i32, z: i32, speedx: i32, speedy: i32, speedz: i32, friends: i32) -> ClusterGrenade {
        todo!("ClusterGrenade")
    }
}

impl Obj for ClusterGrenade {
    crate::obj_boilerplate!();
}

