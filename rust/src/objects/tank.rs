//! Port of ref/objects/tank.cc (end boss).

use super::cluster::Cluster;
use super::explode::Explosion;
use super::hs_bulle::HighSpeedBullet;
use super::smallexp::SmallExplosion;
use super::tankwhel::TankWheel;
use crate::engine::*;
use crate::{icache, rng::random, sound};

const WARNING_SAMPLE: &str = "samples/airnuke.wav";
const MAX_CLUSTER_COUNTER: i32 = 5;
const MAX_ENERGY: i32 = 1000;
const NUM_SKADA: i32 = 4;

const COMMENTS: [&str; 5] = [
    "samples/oldare.wav",
    "samples/olneverd.wav",
    "samples/olskills.wav",
    "samples/olquit.wav",
    "samples/olvictor.wav",
];

const STATE_NONE: i32 = 0;
const STATE_MOVE_LEFT: i32 = 1;
const STATE_MOVE_RIGHT: i32 = 2;
const STATE_FIRE_MINIGUN: i32 = 4;
const STATE_FIRE_CLUSTER: i32 = 8;
const STATE_DEAD: i32 = 16;

const FIRE_REP: i32 = 2;
const FIRE_DELAY: i32 = 1;
const CLUSTER_REP: i32 = 4;
const CLUSTER_DELAY: i32 = 20;

const MAX_Y_SPEED: i32 = 16;
const Y_ACCEL: i32 = 1;

const MOVE_LEFT: i32 = 1;
const MOVE_RIGHT: i32 = 2;
const MOVE_STOP: i32 = 3;
const FIRE_MINIGUN: i32 = 4;
const FIRE_CLUSTER: i32 = 5;
const FIRE_STOP: i32 = 6;

const AIMOVE: [(i32, i32); 6] = [
    (MOVE_LEFT, 300),
    (MOVE_STOP, 200),
    (MOVE_RIGHT, 100),
    (MOVE_STOP, 50),
    (MOVE_RIGHT, 200),
    (MOVE_STOP, 200),
];

const AIFIRE: [(i32, i32); 7] = [
    (FIRE_MINIGUN, 200),
    (FIRE_STOP, 200),
    (FIRE_CLUSTER, 100),
    (FIRE_STOP, 100),
    (FIRE_CLUSTER, 80),
    (FIRE_MINIGUN, 170),
    (FIRE_STOP, 200),
];

pub struct Tank {
    pub base: Base,
    wheel1: Option<ObjId>,
    wheel2: Option<ObjId>,
    aim: usize,
    aif: usize,
    cluster_counter: i32,
    cmc: usize,
}

impl Tank {
    pub fn new(x: i32, y: i32, z: i32) -> Tank {
        let mut b = Base::new(x, y, z);
        for side in ["v", "h"] {
            for s in 0..4 {
                b.images.extend(icache::seq(&format!("boss/{}/skada{}/", side, s), 1, 3, ".pcx"));
            }
        }
        b.images.push(icache::get("boss/v/dead.pcx"));
        b.images.push(icache::get("boss/h/dead.pcx"));
        b.height = b.images[0].h - 19;
        b.width = b.images[0].w;
        b.coll_x = 21;
        b.coll_y = 0;
        b.coll_width = b.width - b.coll_x;
        b.coll_height = b.height;
        // Stand on the feet
        b.y -= b.images[0].h;
        b.energy = MAX_ENERGY;
        b.strength = 10;
        b.direction = LEFT_DIR;
        b.state = STATE_NONE;
        // counter3: aimove timer, counter4: aifire timer, counter5: started yet?
        b.score = 0;
        b.friends = FRIEND_TANK;
        b.enemies = !0;
        b.me = FRIEND_TANK;
        Tank { base: b, wheel1: None, wheel2: None, aim: 0, aif: 0, cluster_counter: 0, cmc: 1 }
    }

    /// Damage stage 0..3 (the original's __URK).
    fn urk(&self) -> i32 {
        (NUM_SKADA - 1) - self.base.energy.min(MAX_ENERGY - 1) / (MAX_ENERGY / NUM_SKADA)
    }

    fn image(&self, x: i32) -> i32 {
        x + 3 * self.urk().min(3)
    }

    fn wheels(&self) -> [Option<ObjId>; 2] {
        [self.wheel1, self.wheel2]
    }
}

impl Obj for Tank {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        if self.base.counter5 == 0 {
            // The original creates the wheels in its constructor.
            let (x, y, z) = (self.base.x, self.base.y, self.base.z);
            let w1 = TankWheel::new(x, y, z - TILE_TOP_HEIGHT);
            self.wheel1 = Some(w1.base.id);
            g.create_object(w1);
            let w2 = TankWheel::new(x, y, z + TILE_TOP_HEIGHT);
            self.wheel2 = Some(w2.base.id);
            g.create_object(w2);

            g.score.init_boss_health(MAX_ENERGY);
            g.score.set_boss_health(MAX_ENERGY);
            self.base.counter5 = 1;
            g.set_max_update_x(2000);
            sound::play_music("snd/modules/boss.xm");
            sound::play_sfx_critical(COMMENTS[0]);
            return Upd::ToAlwaysUpdate;
        } else {
            self.base.score = 50000;
        }

        if self.base.counter != 0 {
            self.base.counter -= 1;
        }

        if self.base.counter == 0 && self.base.state & STATE_DEAD != 0 {
            for w in self.wheels().into_iter().flatten() {
                if let Some(w) = g.find::<TankWheel>(w) {
                    w.base.energy = 0;
                }
            }
            // Hide the health meter
            g.score.init_boss_health(-1);
            g.clear_level();
            return Upd::ToDekoration;
        }

        let b = &mut self.base;
        if b.state & STATE_DEAD != 0 {
            g.create_effect(SmallExplosion::new(b.x + random() % b.width, b.y + random() % b.height, b.z));
            if random() % 14 == 0 {
                g.create_object(Explosion::new(b.x + random() % b.width, b.y + random() % b.height, b.z, 0, 0, 0, 0));
            }
            if b.counter == 200 {
                g.push_message("Airstrike alert!");
                sound::play_sfx_critical(WARNING_SAMPLE);
            }
            if b.counter < 25 {
                g.create_object(Explosion::new(b.x + random() % b.width, b.y + random() % b.height, b.z, 0, 0, 0, 0));
                g.enable_earthquake();
                g.enable_earthquake();
                g.enable_earthquake();
            }
            return Upd::Keep;
        }

        if b.energy <= 0 {
            b.strength = 0;
            b.state = STATE_DEAD;
            b.counter = 300;
            b.current_image = if b.direction == LEFT_DIR { 24 } else { 25 };
            return Upd::Keep;
        }

        if b.counter2 != 0 {
            b.counter2 -= 1;
        }
        if b.counter3 != 0 {
            b.counter3 -= 1;
        }
        if b.counter4 != 0 {
            b.counter4 -= 1;
        }

        if self.base.counter3 == 0 {
            self.base.state &= !(STATE_MOVE_LEFT | STATE_MOVE_RIGHT);
            let (what, num) = AIMOVE[self.aim];
            match what {
                MOVE_LEFT => self.base.state |= STATE_MOVE_LEFT,
                MOVE_RIGHT => self.base.state |= STATE_MOVE_RIGHT,
                _ => {}
            }
            self.base.counter3 = num;
            self.aim += 1;
            if self.aim >= AIMOVE.len() {
                self.aim = 0;
            }
        }
        if self.base.counter4 == 0 {
            self.base.state &= !(STATE_FIRE_MINIGUN | STATE_FIRE_CLUSTER);
            self.base.current_image = self.image(0);
            let (what, num) = AIFIRE[self.aif];
            match what {
                FIRE_MINIGUN => self.base.state |= STATE_FIRE_MINIGUN,
                FIRE_CLUSTER => self.base.state |= STATE_FIRE_CLUSTER,
                _ => {}
            }
            self.base.counter4 = num;
            self.aif += 1;
            if self.aif >= AIFIRE.len() {
                self.aif = 0;
            }
        }

        let state = self.base.state;
        if state & (STATE_MOVE_RIGHT | STATE_MOVE_LEFT) != 0 {
            self.base.current_image = self.image(0);
            if self.base.direction == LEFT_DIR {
                let right = state & STATE_MOVE_RIGHT != 0;
                self.base.x += if right { 1 } else { -1 };
                let x = self.base.x;
                for w in self.wheels().into_iter().flatten() {
                    if let Some(w) = g.find::<TankWheel>(w) {
                        if right {
                            w.move_right(x);
                        } else {
                            w.move_left(x);
                        }
                    }
                }
            }
        }

        let img0 = self.image(0);
        let img1 = self.image(1);
        let img2 = self.image(2);
        let b = &mut self.base;
        if state & STATE_FIRE_MINIGUN != 0 {
            if b.direction == LEFT_DIR {
                if b.counter2 == 0 && b.current_image == img0 {
                    b.current_image = img2;
                    b.counter2 = FIRE_REP;
                    g.create_alwaysupdate(HighSpeedBullet::new(b.x + 20, b.y + 54, b.z, b.direction, b.me as i32, 5));
                } else if b.counter2 == 0 && b.current_image == img2 {
                    b.counter2 = FIRE_DELAY;
                    b.current_image = img0;
                }
            }
        } else if state & STATE_FIRE_CLUSTER != 0 {
            if b.direction == LEFT_DIR {
                if b.counter2 == 0 && b.current_image == img0 {
                    b.current_image = img1;
                    b.counter2 = CLUSTER_REP;
                    let s = -(3 + self.cluster_counter);
                    g.create_alwaysupdate(Cluster::new(b.x + 67, b.y + 5, b.z, s, s, 0, b.me as i32));
                    self.cluster_counter += 1;
                    if self.cluster_counter > MAX_CLUSTER_COUNTER {
                        self.cluster_counter = 0;
                    }
                } else if b.counter2 == 0 && b.current_image == img1 {
                    b.current_image = img0;
                    b.counter2 = CLUSTER_DELAY;
                }
            }
        }

        // Fall or stop
        if g.check_floor(b.x, b.y + b.height, b.z) || g.check_floor(b.x + b.width, b.y + b.height, b.z) {
            b.speed_y = 0;
        } else if b.speed_y < MAX_Y_SPEED {
            b.speed_y += Y_ACCEL;
            b.speed_z = 0;
        }
        b.x += b.speed_x;
        b.y += b.speed_y;
        b.z += b.speed_z;
        if b.x < 0 {
            b.x = 0;
        }
        if b.z > MIN_Z {
            b.z = MIN_Z;
            b.speed_z = 0;
            b.counter = 0;
        }
        if b.z < MAX_Z {
            b.z = MAX_Z;
            b.speed_z = 0;
            b.counter = 0;
        }
        b.layer = b.z / TILE_TOP_HEIGHT;
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, g: &mut Game) {
        if self.base.energy == 0 {
            return;
        }
        let o = other.base();
        if o.me & (ENEMY_FIREBALL | ENEMY_BEAM | ENEMY_ICEBALL | ENEMY_PLASMA) != 0 {
            return;
        }
        if o.enemies & self.base.me != 0 {
            if o.me & ENEMY_EXPLOSION != 0 {
                self.base.energy -= o.strength / 2;
            } else {
                self.base.energy -= o.strength;
            }
        }
        if self.base.energy < 0 {
            self.base.energy = 0;
        }
        g.score.set_boss_health(self.base.energy);
        let cur = self.base.current_image;
        let new = self.image(cur % 3);
        if new != cur {
            sound::play_sfx_critical(COMMENTS[self.cmc.min(COMMENTS.len() - 1)]);
            self.cmc += 1;
        }
        self.base.current_image = new;
    }
}
