//! Port of ref/objects/soldier3.cc: a soldier that patrols left/right on its layer and shoots.

use crate::engine::*;
use crate::icache;
use crate::objects::ai_soldier::soldier_hit;
use crate::objects::anim::Anim;
use crate::objects::hs_bulle::HighSpeedBullet;
use crate::rng::random;

const PROB_WALK_START: i32 = 0;
const PROB_WALK_END: i32 = 50;
const PROB_STOP_START: i32 = 51;
const PROB_STOP_END: i32 = 60;
const PROB_TURN_START: i32 = 61;
const PROB_TURN_END: i32 = 100;
const PROB_NUMBER: i32 = 100;

const MAX_NUM: i32 = 15;
const STATE_NONE: i32 = 0;
const STATE_STOP: i32 = 1;
const STATE_WALK: i32 = 2;
const STATE_TURN: i32 = 3;
const STATE_HIT: i32 = 4;
const STATE_HIT_COUNTDOWN: i32 = 5;
const STATE_FIRE: i32 = 6;
const STATE_FIRING: i32 = 7;
const STATE_ALARM: i32 = 8;

const FIRE_RANGE: i32 = 150;
const FIRE_STRENGTH: i32 = 5;
const MAX_X_SPEED: i32 = 2;
const MIN_X_SPEED: i32 = -MAX_X_SPEED;
const MAX_Y_SPEED: i32 = 16;
const X_ACCEL: i32 = 2;
const Y_ACCEL: i32 = 1;

const FIRE_DELAY: i32 = 60;
const FIRING_DELAY: i32 = 5;
const HIT_DELAY: i32 = 10;
const MOVE_DELAY: i32 = 5;

const FOOT_LEFT: i32 = 20;
const FOOT_WIDTH: i32 = 22;
const FOOT_RIGHT: i32 = FOOT_LEFT + FOOT_WIDTH;

pub struct Soldier3 {
    pub base: Base,
    anim: Anim,
}

impl Soldier3 {
    pub fn new(x: i32, y: i32, z: i32) -> Soldier3 {
        let mut b = Base::new(x, y, z);
        b.images.extend(icache::seq("soldier3/v/", 1, 5, ".pcx"));
        b.images.extend(icache::seq("soldier3/h/", 1, 5, ".pcx"));
        b.images.extend(icache::seq("soldier3/v/shot", 1, 3, ".pcx"));
        b.images.extend(icache::seq("soldier3/h/shot", 1, 3, ".pcx"));
        b.current_image = 1;
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.coll_x = 20;
        b.coll_y = 0;
        b.coll_width = 45;
        b.coll_height = b.height - 1;
        // Stand on the feet
        b.y -= b.images[0].h;
        b.energy = 20;
        b.strength = 10;
        b.direction = RIGHT_DIR;
        b.state = STATE_NONE;
        b.score = 500;
        b.friends = !FRIEND_PLAYER;
        b.enemies = ENEMY_PLAYER;
        b.me = FRIEND_SOLDIER;
        Soldier3 { base: b, anim: Anim::default() }
    }
}

impl Obj for Soldier3 {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let (px, player_layer) = (g.pbase().x, g.pbase().layer);
        let b = &mut self.base;
        if b.energy <= 0 {
            return Upd::Delete;
        }
        if b.counter != 0 {
            b.counter -= 1;
        }
        if b.counter == 0 && b.state != STATE_FIRING {
            let r = random() % PROB_NUMBER;
            if (PROB_WALK_START..=PROB_WALK_END).contains(&r) {
                b.state = STATE_WALK;
                // pick a random direction
                b.direction = random() % 2;
            } else if (PROB_STOP_START..=PROB_STOP_END).contains(&r) || b.state == STATE_STOP || b.state == STATE_TURN {
                b.state = STATE_STOP;
            } else if (PROB_TURN_START..=PROB_TURN_END).contains(&r) {
                b.state = STATE_TURN;
            }
            b.counter = random() % MAX_NUM + 15;
        }

        let s = b.state;
        if s != STATE_FIRE && s != STATE_FIRING && s != STATE_TURN && s != STATE_ALARM && s != STATE_HIT && s != STATE_HIT_COUNTDOWN {
            if px < b.x && px > b.x - FIRE_RANGE && b.direction == LEFT_DIR && player_layer == b.layer {
                b.state = STATE_FIRE;
            } else if px > b.x && px < b.x + FIRE_RANGE && b.direction == RIGHT_DIR && player_layer == b.layer {
                b.state = STATE_FIRE;
            }
        }

        if b.speed_y == 0 {
            if b.state == STATE_HIT {
                b.state = STATE_HIT_COUNTDOWN;
                b.counter = HIT_DELAY;
            }
            match b.state {
                STATE_WALK => match b.direction {
                    RIGHT_DIR => {
                        if b.speed_x < MAX_X_SPEED {
                            b.speed_x += X_ACCEL;
                        }
                        while g.check_wall(b.x + b.coll_x + b.coll_width + b.speed_x, b.y + b.coll_y + b.coll_height, b.z) && b.speed_x != 0 {
                            b.speed_x -= 1;
                        }
                        b.current_image = 5 + self.anim.next_frame(4, MOVE_DELAY);
                        b.speed_z = 0;
                    }
                    LEFT_DIR => {
                        if b.speed_x > MIN_X_SPEED {
                            b.speed_x -= X_ACCEL;
                        }
                        while g.check_wall(b.x + b.coll_x + b.speed_x, b.y + b.coll_y + b.coll_height, b.z) && b.speed_x != 0 {
                            b.speed_x += 1;
                        }
                        b.current_image = self.anim.next_frame(4, MOVE_DELAY);
                        b.speed_z = 0;
                    }
                    _ => b.direction = LEFT_DIR,
                },
                STATE_STOP => {
                    b.speed_x = 0;
                    b.speed_z = 0;
                }
                STATE_NONE => {
                    b.current_image = if b.direction == RIGHT_DIR { 4 } else { 9 };
                    b.counter = 0;
                    self.anim.reset();
                }
                // The original left the fire state as a commented-out stub (BUGS.md #32). Face the
                // player and fire a burst for FIRE_DELAY frames, like soldier.cc.
                STATE_FIRE | STATE_FIRING => {
                    if b.state == STATE_FIRE {
                        b.direction = if px < b.x { LEFT_DIR } else { RIGHT_DIR };
                        b.counter = FIRE_DELAY;
                        b.state = STATE_FIRING;
                        self.anim.reset();
                        b.speed_x = 0;
                        b.speed_z = 0;
                        b.counter2 = 0;
                    }
                    let r = self.anim.next_frame(2, FIRING_DELAY);
                    b.current_image = if b.direction == RIGHT_DIR { 13 } else { 10 } + r;
                    if r == 1 && b.counter2 == 0 {
                        let bx = if b.direction == RIGHT_DIR { b.x + b.coll_x + b.coll_width } else { b.x + b.coll_x };
                        g.create_object(HighSpeedBullet::new(bx, b.y + 24, b.z, b.direction, !FRIEND_PLAYER as i32, FIRE_STRENGTH));
                        b.counter2 = 1;
                    }
                    if r == 2 {
                        b.counter2 = 0;
                    }
                    if b.counter == 0 {
                        b.state = STATE_STOP;
                    }
                }
                STATE_TURN => {
                    b.direction = if b.direction == RIGHT_DIR { LEFT_DIR } else { RIGHT_DIR };
                    b.speed_x = 0;
                    b.counter = random() % MAX_NUM + 15;
                    b.state = STATE_WALK;
                }
                STATE_HIT_COUNTDOWN => {
                    b.current_image = if b.direction == RIGHT_DIR { 24 } else { 25 };
                }
                _ => {}
            }
        }

        if b.speed_y == 0 {
            match b.direction {
                LEFT_DIR => {
                    if !g.check_floor(b.x + FOOT_LEFT + b.speed_x, b.y + b.height, b.z)
                        && !g.check_floor(b.x + FOOT_LEFT + b.speed_x, b.y + b.height + TILE_SIDE_HEIGHT, b.z)
                    {
                        b.speed_x = 0;
                        b.state = STATE_TURN;
                    }
                }
                RIGHT_DIR => {
                    if !g.check_floor(b.x + FOOT_RIGHT + b.speed_x, b.y + b.height, b.z)
                        && !g.check_floor(b.x + FOOT_RIGHT + b.speed_x, b.y + b.height + TILE_SIDE_HEIGHT, b.z)
                    {
                        b.speed_x = 0;
                        b.state = STATE_TURN;
                    }
                }
                _ => {}
            }
        }

        // Fall or stop
        if g.check_floor(b.x + FOOT_LEFT, b.y + b.height, b.z) || g.check_floor(b.x + FOOT_RIGHT, b.y + b.height, b.z) {
            b.speed_y = 0;
        } else if b.speed_y < MAX_Y_SPEED {
            b.speed_y += Y_ACCEL;
            b.speed_z = 0;
        }

        b.x += b.speed_x;
        b.y += b.speed_y;
        if b.x < 0 {
            b.x = 0;
        }

        if b.speed_x == 0 && b.state != STATE_HIT_COUNTDOWN && b.state != STATE_FIRING && b.state != STATE_ALARM {
            b.current_image = if b.direction == RIGHT_DIR { 4 } else { 9 };
        }
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, g: &mut Game) {
        if self.base.energy == 0 {
            return;
        }
        default_collision(&mut self.base, other.base());
        soldier_hit(&self.base, other.base(), g, "soldier2/h/dead", "soldier2/v/dead");
    }
}
