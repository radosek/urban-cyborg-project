//! STUB — replaced by the port of ref/objects/player.cc
#![allow(unused)]

use crate::engine::*;

pub struct Player {
    pub base: Base,
}

impl Player {
    pub fn new(x: i32, y: i32, z: i32, controls: i32) -> Player { todo!("Player") }
    pub fn move_to(&mut self, x: i32, y: i32, z: i32) { todo!() }
    pub fn set_ammo(&mut self, t: usize, n: i32) { todo!() }
    pub fn add_ammo(&mut self, t: usize, n: i32) { todo!() }
    pub fn enable_weapon(&mut self, t: usize) { todo!() }
    pub fn disable_weapon(&mut self, t: usize) { todo!() }
    pub fn ammo(&self) -> i32 { todo!() }
    pub fn ammo_of(&self, t: usize) -> i32 { todo!() }
    pub fn weapon(&self) -> usize { todo!() }
    pub fn have_weapon(&self, t: usize) -> bool { todo!() }
    pub fn set_life(&mut self, n: i32) { todo!() }
    pub fn add_life(&mut self) { todo!() }
    pub fn remove_life(&mut self) { todo!() }
    pub fn lives(&self) -> i32 { todo!() }
    pub fn give_card(&mut self, c: i32) { todo!() }
    pub fn have_card(&self, c: i32) -> bool { todo!() }
    pub fn kill_player(&mut self, g: &mut Game) { todo!() }
    pub fn new_level(&mut self) -> i32 { todo!() }
}

impl Obj for Player {
    crate::obj_boilerplate!();
}
