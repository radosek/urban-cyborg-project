//! STUB — replaced by the port of ref/objects/seagal.cc
#![allow(unused)]

use crate::engine::*;

pub struct BossSeagal {
    pub base: Base,
}

impl BossSeagal {
    pub fn new(x: i32, y: i32, z: i32) -> BossSeagal {
        todo!("BossSeagal")
    }
}

impl Obj for BossSeagal {
    crate::obj_boilerplate!();
}

pub struct Seagal {
    pub base: Base,
}

impl Seagal {
    pub fn new(x: i32, y: i32, z: i32) -> Seagal {
        todo!("Seagal")
    }
}

impl Obj for Seagal {
    crate::obj_boilerplate!();
}

