//! Port of ref/objects/soldier.cc: the basic rifle soldier and its card-dropping boss variant.

use crate::engine::*;
use crate::icache;
use crate::objects::ai_soldier::soldier_hit;
use crate::objects::anim::Anim;
use crate::objects::bdyparts::Bodyparts;
use crate::objects::card::Card;
use crate::objects::hs_bulle::HighSpeedBullet;
use crate::rng::random;

const PROB_WALK_START: i32 = 0;
const PROB_WALK_END: i32 = 40;
const PROB_STOP_START: i32 = 41;
const PROB_STOP_END: i32 = 70;
const PROB_TURN_START: i32 = 71;
const PROB_TURN_END: i32 = 100;
const PROB_NUMBER: i32 = 100;

const MAX_NUM: i32 = 15;
const STATE_NONE: i32 = 0;
const STATE_STOP: i32 = 1;
const STATE_WALK: i32 = 2;
const STATE_TURN: i32 = 3;
const STATE_HIT: i32 = 4;
const STATE_FIRE: i32 = 6;
const STATE_FIRING: i32 = 7;
const STATE_ALARM: i32 = 8;

const FIRE_RANGE: i32 = 100;

const MAX_X_SPEED: i32 = 2;
const X_FRICTION: i32 = 1;
const MIN_X_SPEED: i32 = -MAX_X_SPEED;
const MAX_Y_SPEED: i32 = 16;
const X_ACCEL: i32 = 2;
const Y_ACCEL: i32 = 1;

const FIRE_DELAY: i32 = 4;
const NUM_ROUNDS: i32 = 6;
const ALARM_DELAY: i32 = 120;

pub struct Soldier {
    pub base: Base,
    anim: Anim,
}

impl Soldier {
    pub fn new(x: i32, y: i32, z: i32) -> Soldier {
        let mut b = Base::new(x, y, z);
        b.images.extend(icache::seq("soldier/soldh/", 1, 5, ".pcx"));
        b.images.extend(icache::seq("soldier/soldh/shot", 1, 3, ".pcx"));
        b.images.extend(icache::seq("soldier/soldv/", 1, 5, ".pcx"));
        b.images.extend(icache::seq("soldier/soldv/shot", 1, 3, ".pcx"));
        b.current_image = 4;
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.coll_x = 20;
        b.coll_y = 0;
        b.coll_width = 30;
        b.coll_height = b.height;
        // Stand on the feet
        b.y -= b.images[0].h;
        b.energy = 5;
        b.strength = 3;
        b.direction = RIGHT_DIR;
        b.state = STATE_NONE;
        b.score = 300;
        b.friends = !FRIEND_PLAYER;
        b.enemies = ENEMY_PLAYER;
        b.me = FRIEND_SOLDIER;
        Soldier { base: b, anim: Anim::default() }
    }
}

impl Obj for Soldier {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let (px, player_layer) = (g.pbase().x, g.pbase().layer);
        let b = &mut self.base;
        if b.counter != 0 {
            b.counter -= 1;
        }
        if b.speed_x < 0 {
            b.speed_x += X_FRICTION;
        }
        if b.speed_x > 0 {
            b.speed_x -= X_FRICTION;
        }

        if b.counter == 0 {
            if b.state == STATE_FIRE || b.state == STATE_FIRING {
                b.state = STATE_ALARM;
            } else if b.state == STATE_HIT {
                b.direction = if px < b.x { LEFT_DIR } else { RIGHT_DIR };
                b.state = STATE_FIRE;
            } else {
                let r = random() % PROB_NUMBER;
                if (PROB_WALK_START..=PROB_WALK_END).contains(&r) {
                    b.state = STATE_WALK;
                } else if (PROB_STOP_START..=PROB_STOP_END).contains(&r) || b.state == STATE_STOP {
                    b.state = STATE_STOP;
                } else if (PROB_TURN_START..=PROB_TURN_END).contains(&r) {
                    b.state = STATE_TURN;
                }
                b.counter = random() % MAX_NUM + 15;
            }
        }

        if b.state != STATE_FIRE && b.state != STATE_ALARM && b.state != STATE_FIRING {
            if px < b.x && px > b.x - FIRE_RANGE && b.direction == LEFT_DIR && player_layer == b.layer {
                b.state = STATE_FIRE;
            } else if px > b.x && px < b.x + FIRE_RANGE && b.direction == RIGHT_DIR && player_layer == b.layer {
                b.state = STATE_FIRE;
            }
        }

        match b.state {
            STATE_FIRE | STATE_FIRING => {
                if b.state == STATE_FIRE {
                    b.counter = NUM_ROUNDS * FIRE_DELAY * 3;
                    b.state = STATE_FIRING;
                    self.anim.reset();
                    b.speed_x = 0;
                    b.counter2 = 0;
                }
                let r = self.anim.next_frame(2, FIRE_DELAY);
                b.current_image = if b.direction == RIGHT_DIR { 5 } else { 13 } + r;
                if r == 1 && b.counter2 == 0 {
                    g.create_object(HighSpeedBullet::new(b.x, b.y + 24, b.z, b.direction, !FRIEND_PLAYER as i32, 1));
                    b.counter2 = 1;
                }
                if r == 2 && b.counter2 == 1 {
                    if b.direction == RIGHT_DIR {
                        let sx = -(random() % 5);
                        let sy = -(random() % 8);
                        g.create_effect(Bodyparts::new(b.x + 36, b.y + 24, b.z, "soldier/hylsa", 1, sx, sy, 0));
                    } else {
                        let sx = random() % 5;
                        let sy = -(random() % 8);
                        g.create_effect(Bodyparts::new(b.x + 28, b.y + 24, b.z, "soldier/hylsa", 1, sx, sy, 0));
                    }
                    b.counter2 = 0;
                }
            }
            STATE_ALARM => {
                if b.counter == 0 {
                    b.current_image = if b.direction == RIGHT_DIR { 5 } else { 13 };
                    b.counter = ALARM_DELAY;
                }
            }
            STATE_NONE => {
                b.current_image = if b.direction == RIGHT_DIR { 4 } else { 12 };
                b.counter = 0;
                self.anim.reset();
            }
            STATE_WALK => match b.direction {
                RIGHT_DIR => {
                    if b.speed_x < MAX_X_SPEED {
                        b.speed_x += X_ACCEL;
                    }
                    while g.check_wall(b.x + b.width + b.speed_x, b.y + b.height, b.z) && b.speed_x != 0 {
                        b.speed_x -= 1;
                    }
                    b.current_image = self.anim.next_frame(4, 5);
                }
                LEFT_DIR => {
                    if b.speed_x > MIN_X_SPEED {
                        b.speed_x -= X_ACCEL;
                    }
                    while g.check_wall(b.x + b.speed_x, b.y + b.height, b.z) && b.speed_x != 0 {
                        b.speed_x += 1;
                    }
                    b.current_image = 8 + self.anim.next_frame(4, 5);
                }
                _ => {}
            },
            STATE_TURN => {
                b.direction = if b.direction == RIGHT_DIR { LEFT_DIR } else { RIGHT_DIR };
                b.speed_x = 0;
                b.counter = 0;
            }
            STATE_STOP => b.speed_x = 0,
            _ => {}
        }

        if b.speed_y == 0
            && (!g.check_floor(b.x + b.speed_x, b.y + b.height, b.z)
                || !g.check_floor(b.x + b.width + b.speed_x, b.y + b.height, b.z))
        {
            b.speed_x = 0;
        }

        // Fall or stop
        if g.check_floor(b.x + 14, b.y + b.height, b.z) || g.check_floor(b.x + b.width - 14, b.y + b.height, b.z) {
            b.speed_y = 0;
        } else if b.speed_y < MAX_Y_SPEED {
            b.speed_y += Y_ACCEL;
        }

        b.x += b.speed_x;
        b.y += b.speed_y;
        if b.x < 0 {
            b.x = 0;
        }

        if b.speed_x == 0 && b.state != STATE_FIRE && b.state != STATE_FIRING && b.state != STATE_ALARM {
            b.current_image = if b.direction == RIGHT_DIR { 4 } else { 12 };
        }

        if b.energy <= 0 {
            return Upd::Delete;
        }
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, g: &mut Game) {
        if self.base.energy == 0 {
            return;
        }
        default_collision(&mut self.base, other.base());
        if soldier_hit(&self.base, other.base(), g, "soldier/deadh/dead", "soldier/deadv/dead") {
            self.base.state = STATE_HIT;
            self.base.counter = 0;
        }
    }
}

pub struct BossSoldier {
    inner: Soldier,
}

impl BossSoldier {
    pub fn new(x: i32, y: i32, z: i32) -> BossSoldier {
        BossSoldier { inner: Soldier::new(x, y, z) }
    }
}

impl Obj for BossSoldier {
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
        let ret = self.inner.update(g);
        if ret == Upd::Delete {
            let b = &self.inner.base;
            g.create_object(Card::new(b.x, b.y, b.z, CARD_GREEN));
        }
        ret
    }

    fn collision(&mut self, other: &mut dyn Obj, g: &mut Game) {
        self.inner.collision(other, g);
    }
}
