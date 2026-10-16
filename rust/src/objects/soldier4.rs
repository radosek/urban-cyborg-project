//! Port of ref/objects/soldier4.cc. It is soldier2.cc with cluster bombs instead of grenades,
//! a 250px fire range, and firing across layers beyond 150px; the shared code is in soldier2.rs.

use crate::engine::*;
use crate::objects::soldier2::Soldier2;

pub struct Soldier4 {
    inner: Soldier2,
}

impl Soldier4 {
    pub fn new(x: i32, y: i32, z: i32) -> Soldier4 {
        Soldier4 { inner: Soldier2::with_kind(x, y, z, true) }
    }
}

impl Obj for Soldier4 {
    fn base(&self) -> &Base {
        &self.inner.base
    }
    fn base_mut(&mut self) -> &mut Base {
        &mut self.inner.base
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }

    fn update(&mut self, g: &mut Game) -> Upd {
        self.inner.update(g)
    }

    fn collision(&mut self, other: &mut dyn Obj, g: &mut Game) {
        self.inner.collision(other, g);
    }
}
