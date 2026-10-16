//! STUB — replaced by the port of ref/objects/cluster.cc
#![allow(unused)]

use crate::engine::*;

pub struct Cluster {
    pub base: Base,
}

impl Cluster {
    pub fn new(x: i32, y: i32, z: i32, speedx: i32, speedy: i32, speedz: i32, friends: i32) -> Cluster {
        todo!("Cluster")
    }
}

impl Obj for Cluster {
    crate::obj_boilerplate!();
}

