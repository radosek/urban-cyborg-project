//! Shared death effects for the soldier family (include/death.h and the common Collision code).

use crate::engine::*;
use crate::objects::animate::Animation;
use crate::objects::bdyparts::Bodyparts;
use crate::objects::blood::Blood;
use crate::objects::burning::BurningBody;
use crate::objects::frosen::FrosenBody;
use crate::rng::random;

const SPLAT_SOUND: &str = "samples/gutsplat.wav";

fn part(g: &mut Game, b: &Base, name: &str, n: i32, sx: i32, sy: i32) {
    let (cx, cy) = (b.x + b.width / 2, b.y + b.height / 2);
    g.create_effect(Bodyparts::new(cx, cy, b.z, name, n, sx, sy, 0));
}

/// DEATH_BY_EXPLOSION
pub fn death_by_explosion(g: &mut Game, b: &Base) {
    let r = random() % 3;
    let cx = b.x + b.width / 2;
    for _ in 0..10 {
        let sx = random() % 8 + 2;
        let sy = -(random() % 4) - 2;
        g.create_effect(Blood::new(cx, b.y, b.z, sx, sy, 0));
    }
    for _ in 0..10 {
        let sx = -(random() % 8) - 2;
        let sy = -(random() % 4) - 2;
        g.create_effect(Blood::new(cx, b.y, b.z, sx, sy, 0));
    }
    match r {
        0 => {
            part(g, b, "meat/arm", 5, -4, -6);
            part(g, b, "meat/arm", 5, 4, -6);
            part(g, b, "meat/head", 5, 2, -4);
            part(g, b, "meat/leg", 5, -1, -2);
            part(g, b, "meat/leg", 5, 3, -4);
            part(g, b, "meat/ribs", 5, 0, 0);
            for _ in 0..5 {
                let sx = random() % 5;
                let sy = -(random() % 6);
                part(g, b, "meat/meat", 3, sx, sy);
            }
        }
        1 => {
            part(g, b, "meat/head", 5, 2, -4);
            part(g, b, "meat/ribs", 5, 0, 0);
            for _ in 0..5 {
                let sx = random() % 4;
                let sy = -(random() % 4) - 2;
                part(g, b, "meat/meat", 3, sx, sy);
            }
            for _ in 0..5 {
                let sx = -(random() % 4);
                let sy = -(random() % 4) - 2;
                part(g, b, "meat/meat", 3, sx, sy);
            }
        }
        _ => {
            for _ in 0..3 {
                let sx = random() % 5;
                let sy = -(random() % 4) - 3;
                part(g, b, "meat/bigmeat", 5, sx, sy);
            }
            for _ in 0..3 {
                let sx = -(random() % 5);
                let sy = -(random() % 4) - 3;
                part(g, b, "meat/bigmeat", 5, sx, sy);
            }
        }
    }
    crate::sound::play_sfx(SPLAT_SOUND);
}

/// The hit/death part of every soldier's Collision (after Object::Collision). Returns true when
/// the hit counted.
pub fn soldier_hit(b: &Base, other: &Base, g: &mut Game, dead_right: &str, dead_left: &str) -> bool {
    if !(other.strength > 0 && other.friends & b.me == 0) {
        return false;
    }
    let who = other.me;
    if b.energy <= 0 {
        if who & ENEMY_EXPLOSION != 0 {
            death_by_explosion(g, b);
        } else if who & (ENEMY_FIREBALL | ENEMY_BEAM) != 0 {
            g.create_effect(BurningBody::new(b.x, b.y + b.height, b.z, 1));
        } else if who & ENEMY_ICEBALL != 0 {
            g.create_object(FrosenBody::new(b.x, b.y + b.height, b.z, b.direction));
        } else {
            let name = if b.direction == RIGHT_DIR { dead_right } else { dead_left };
            g.create_effect(Animation::new(b.x, b.y + b.height, b.z, name, 4, 3, 0, 0, 0));
        }
    }
    if who & (ENEMY_FIREBALL | ENEMY_ICEBALL | ENEMY_BEAM) == 0 {
        for _ in 0..3 {
            let y = b.y + random() % b.height;
            let sx = -2 + random() % 4;
            g.create_effect(Blood::new(b.x + b.width / 2, y, b.z, sx, 0, 0));
        }
    }
    true
}
