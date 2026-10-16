//! Shared helpers for the scientist-family objects: include/death.h gibbing, the blood spray, and
//! the walk/fall code repeated in science.cc, einstein.cc, drgreen.cc, vaktm.cc and burning.cc.

use crate::engine::*;
use crate::objects::anim::Anim;
use crate::objects::bdyparts::Bodyparts;
use crate::objects::blood::Blood;
use crate::rng::random;

pub const SPLAT_SOUND: &str = "samples/gutsplat.wav";

fn gibs(b: &Base, g: &mut Game, px: i32, py: i32, sound: bool) {
    let (x, y, z) = (b.x, b.y, b.z);
    let r = random() % 3;
    let bx = x + b.width / 2;
    for _ in 0..10 {
        let sx = random() % 8 + 2;
        let sy = -(random() % 4) - 2;
        g.create_effect(Blood::new(bx, y, z, sx, sy, 0));
    }
    for _ in 0..10 {
        let sx = -(random() % 8) - 2;
        let sy = -(random() % 4) - 2;
        g.create_effect(Blood::new(bx, y, z, sx, sy, 0));
    }
    let part = |g: &mut Game, name: &str, n: i32, sx: i32, sy: i32| {
        g.create_effect(Bodyparts::new(px, py, z, name, n, sx, sy, 0));
    };
    match r {
        0 => {
            part(g, "meat/arm", 5, -4, -6);
            part(g, "meat/arm", 5, 4, -6);
            part(g, "meat/head", 5, 2, -4);
            part(g, "meat/leg", 5, -1, -2);
            part(g, "meat/leg", 5, 3, -4);
            part(g, "meat/ribs", 5, 0, 0);
            for _ in 0..5 {
                let sx = random() % 5;
                let sy = -(random() % 6);
                part(g, "meat/meat", 3, sx, sy);
            }
        }
        1 => {
            part(g, "meat/head", 5, 2, -4);
            part(g, "meat/ribs", 5, 0, 0);
            for _ in 0..5 {
                let sx = random() % 4;
                let sy = -(random() % 4) - 2;
                part(g, "meat/meat", 3, sx, sy);
            }
            for _ in 0..5 {
                let sx = -(random() % 4);
                let sy = -(random() % 4) - 2;
                part(g, "meat/meat", 3, sx, sy);
            }
        }
        _ => {
            for _ in 0..3 {
                let sx = random() % 5;
                let sy = -(random() % 4) - 3;
                part(g, "meat/bigmeat", 5, sx, sy);
            }
            for _ in 0..3 {
                let sx = -(random() % 5);
                let sy = -(random() % 4) - 3;
                part(g, "meat/bigmeat", 5, sx, sy);
            }
        }
    }
    if sound {
        crate::sound::play_sfx(SPLAT_SOUND);
    }
}

/// DEATH_BY_EXPLOSION (include/death.h).
pub fn death_by_explosion(b: &Base, g: &mut Game) {
    gibs(b, g, b.x + b.width / 2, b.y + b.height / 2, true);
}

/// DEATH_BY_EXPLOSION2 (science.cc): parts spawn at the object's origin, no splat sound.
pub fn death_by_explosion2(b: &Base, g: &mut Game) {
    gibs(b, g, b.x, b.y, false);
}

/// Seven blood drops when shot; they fly away from the facing direction.
pub fn blood_spray(b: &Base, g: &mut Game) {
    let h = b.height.max(1);
    for _ in 0..7 {
        let by = b.y + random() % h;
        let sx = if b.direction == RIGHT_DIR { -(2 + random() % 4) } else { 2 + random() % 4 };
        g.create_effect(Blood::new(b.x + b.width / 2, by, b.z, sx, 0, 0));
    }
}

pub fn friction(b: &mut Base) {
    if b.speed_x < 0 {
        b.speed_x += 1;
    }
    if b.speed_x > 0 {
        b.speed_x -= 1;
    }
}

/// STATE_WALK for left/right walkers that test walls at their bounding box.
pub fn walk_lr(b: &mut Base, anim: &mut Anim, g: &Game, max_speed: i32, right_img: i32, left_img: i32, frames: i32) {
    const X_ACCEL: i32 = 2;
    match b.direction {
        RIGHT_DIR => {
            if b.speed_x < max_speed {
                b.speed_x += X_ACCEL;
            }
            while g.check_wall(b.x + b.width + b.speed_x, b.y + b.height, b.z) && b.speed_x != 0 {
                b.speed_x -= 1;
            }
            b.current_image = right_img + anim.next_frame(frames, 5);
        }
        LEFT_DIR => {
            if b.speed_x > -max_speed {
                b.speed_x -= X_ACCEL;
            }
            while g.check_wall(b.x + b.speed_x, b.y + b.height, b.z) && b.speed_x != 0 {
                b.speed_x += 1;
            }
            b.current_image = left_img + anim.next_frame(frames, 5);
        }
        _ => {}
    }
}

/// Stop at ledges, fall or land, then move.
pub fn walker_tail(b: &mut Base, g: &Game) {
    const MAX_Y_SPEED: i32 = 16;
    if b.speed_y == 0
        && (!g.check_floor(b.x + b.speed_x, b.y + b.height, b.z)
            || !g.check_floor(b.x + b.width + b.speed_x, b.y + b.height, b.z))
    {
        b.speed_x = 0;
    }
    if g.check_floor(b.x + 14, b.y + b.height, b.z) || g.check_floor(b.x + b.width - 14, b.y + b.height, b.z) {
        b.speed_y = 0;
    } else if b.speed_y < MAX_Y_SPEED {
        b.speed_y += 1;
    }
    b.x += b.speed_x;
    b.y += b.speed_y;
    if b.x < 0 {
        b.x = 0;
    }
}

/// Sets width/height/collision box from image 0 and stands the object on its feet.
pub fn stand_on_feet(b: &mut Base) {
    let (w, h) = (b.images[0].w, b.images[0].h);
    b.width = w;
    b.height = h;
    b.coll_x = 0;
    b.coll_y = 0;
    b.coll_width = w;
    b.coll_height = h;
    b.y -= h;
}

/// Boss_X_o: X that drops a green key card when it is removed.
#[macro_export]
macro_rules! boss_with_card {
    ($boss:ident, $inner:ty) => {
        pub struct $boss {
            inner: $inner,
        }

        impl $crate::engine::Obj for $boss {
            fn base(&self) -> &$crate::engine::Base {
                &self.inner.base
            }
            fn base_mut(&mut self) -> &mut $crate::engine::Base {
                &mut self.inner.base
            }
            fn update(&mut self, g: &mut $crate::engine::Game) -> $crate::engine::Upd {
                let ret = self.inner.update(g);
                if ret != $crate::engine::Upd::Keep {
                    let b = &self.inner.base;
                    g.create_object($crate::objects::card::Card::new(b.x, b.y, b.z, $crate::engine::CARD_GREEN));
                }
                ret
            }
            fn collision(&mut self, other: &mut dyn $crate::engine::Obj, g: &mut $crate::engine::Game) {
                self.inner.collision(other, g)
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
        }
    };
}
