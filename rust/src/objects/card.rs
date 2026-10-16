//! STUB — replaced by the port of ref/objects/card.cc
#![allow(unused)]

use crate::engine::*;

pub struct Card {
    pub base: Base,
}

impl Card {
    pub fn new(x: i32, y: i32, z: i32, card: i32) -> Card {
        todo!("Card")
    }
}

impl Obj for Card {
    crate::obj_boilerplate!();
}

