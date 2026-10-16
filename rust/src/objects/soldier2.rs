//! Port of ref/objects/soldier2.cc: the grenade-throwing soldier and its boss variant.
//! soldier4.cc is the same soldier throwing cluster bombs; it reuses this code (`cluster`).

use crate::engine::*;
use crate::icache;
use crate::objects::ai_soldier::soldier_hit;
use crate::objects::anim::Anim;
use crate::objects::bdyparts::Bodyparts;
use crate::objects::card::Card;
use crate::objects::cluster::Cluster;
use crate::objects::grenade::Grenade;
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

const MAX_X_SPEED: i32 = 2;
const MIN_X_SPEED: i32 = -MAX_X_SPEED;
const MAX_Y_SPEED: i32 = 16;
const X_ACCEL: i32 = 2;
const Y_ACCEL: i32 = 1;
const Z_ACCEL: i32 = 2;
const MAX_Z_SPEED: i32 = 2;
const MIN_Z_SPEED: i32 = -MAX_Z_SPEED;

const ALARM_DELAY: i32 = 30;
const FIRE_DELAY: i32 = 60;
const FIRING_DELAY: i32 = 5;
const HIT_DELAY: i32 = 10;

const FOOT_LEFT: i32 = 20;
const FOOT_WIDTH: i32 = 22;
const FOOT_RIGHT: i32 = FOOT_LEFT + FOOT_WIDTH;

pub struct Soldier2 {
    pub base: Base,
    anim: Anim,
    /// soldier4: cluster bombs, longer range, fires across layers.
    cluster: bool,
}

impl Soldier2 {
    pub fn new(x: i32, y: i32, z: i32) -> Soldier2 {
        Soldier2::with_kind(x, y, z, false)
    }

    pub(crate) fn with_kind(x: i32, y: i32, z: i32, cluster: bool) -> Soldier2 {
        let mut b = Base::new(x, y, z);
        b.images.extend(icache::seq("soldier2/v/", 1, 5, ".pcx"));
        b.images.extend(icache::seq("soldier2/h/", 1, 5, ".pcx"));
        b.images.extend(icache::seq("soldier2/f/", 1, 5, ".pcx"));
        b.images.extend(icache::seq("soldier2/b/", 1, 5, ".pcx"));
        b.images.extend(icache::seq("soldier2/v/shot", 1, 2, ".pcx"));
        b.images.extend(icache::seq("soldier2/h/shot", 1, 2, ".pcx"));
        b.images.push(icache::get("soldier2/h/hit.pcx"));
        b.images.push(icache::get("soldier2/v/hit.pcx"));
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
        b.me = FRIEND_SOLDIER2;
        Soldier2 { base: b, anim: Anim::default(), cluster }
    }

    fn throw(&self, g: &mut Game) {
        let b = &self.base;
        let right = b.direction == RIGHT_DIR;
        let (tx, sx) = if right { (b.x + b.width + 5, 1) } else { (b.x - 5, -1) };
        let sy = -(2 + random() % 5);
        if self.cluster {
            g.create_object(Cluster::new(tx, b.y - 10, b.z, 2 * sx, sy, 0, 0));
        } else {
            g.create_object(Grenade::new(tx, b.y - 10, b.z, 4 * sx, sy, 0));
        }
        g.create_object(Bodyparts::new(b.x, b.y, b.z, "soldier2/", 1, 0, 0, 0));
    }

    /// STATE_FIRING: wind up, throw, recover, then turn. Returns true when it ends in STATE_TURN.
    fn firing(&mut self, g: &mut Game) -> bool {
        let base_img = if self.base.direction == RIGHT_DIR { 22 } else { 20 };
        let mut stage = self.base.counter2;
        if stage == 0 {
            let r = self.anim.next_frame(1, ALARM_DELAY);
            self.base.current_image = base_img + r;
            if r == 0 {
                return false;
            }
            self.base.counter2 = 1;
            self.anim.reset();
            stage = 1;
        }
        if stage == 1 || stage == 2 {
            let r = self.anim.next_frame(1, FIRING_DELAY);
            self.base.current_image = base_img + r;
            if r == 0 && self.base.counter2 == 1 {
                self.throw(g);
                self.base.counter2 = 2;
            }
            if r == 0 {
                return false;
            }
            self.base.counter2 = 3;
            self.anim.reset();
            stage = 3;
        }
        if stage == 3 {
            let r = self.anim.next_frame(1, ALARM_DELAY);
            self.base.current_image = base_img + r;
            if r == 0 {
                return false;
            }
        }
        self.base.state = STATE_TURN;
        true
    }

    fn fire_check(&mut self, px: i32, player_layer: i32) {
        let b = &mut self.base;
        let d = b.direction;
        let vertical = d == UP_DIR || d == DOWN_DIR;
        let range = if self.cluster { 250 } else { 150 };
        if px < b.x && px > b.x - range && (d == LEFT_DIR || vertical) {
            if player_layer == b.layer || (self.cluster && px < b.x - 150) {
                b.state = STATE_FIRE;
            }
        } else if px > b.x && px < b.x + range && (d == RIGHT_DIR || vertical) {
            if player_layer == b.layer || (self.cluster && px > b.x + 150) {
                b.state = STATE_FIRE;
            }
        }
    }

    fn do_update(&mut self, g: &mut Game) -> Upd {
        let (px, player_layer) = (g.pbase().x, g.pbase().layer);
        if self.base.energy <= 0 {
            return Upd::Delete;
        }
        {
            let b = &mut self.base;
            if b.counter != 0 {
                b.counter -= 1;
            }
            if b.counter == 0 && b.state != STATE_FIRING {
                let r = random() % PROB_NUMBER;
                if (PROB_WALK_START..=PROB_WALK_END).contains(&r) {
                    b.state = STATE_WALK;
                    // pick a random direction
                    b.direction = random() % 4;
                } else if (PROB_STOP_START..=PROB_STOP_END).contains(&r) || b.state == STATE_STOP || b.state == STATE_TURN {
                    b.state = STATE_STOP;
                } else if (PROB_TURN_START..=PROB_TURN_END).contains(&r) {
                    b.state = STATE_TURN;
                }
                b.counter = random() % MAX_NUM + 15;
            }
        }

        let s = self.base.state;
        if s != STATE_FIRE && s != STATE_FIRING && s != STATE_TURN && s != STATE_ALARM && s != STATE_HIT && s != STATE_HIT_COUNTDOWN {
            self.fire_check(px, player_layer);
        }

        if self.base.speed_y == 0 {
            let mut state = self.base.state;
            if state == STATE_FIRE {
                let b = &mut self.base;
                b.direction = if px < b.x { LEFT_DIR } else { RIGHT_DIR };
                b.counter = FIRE_DELAY;
                b.state = STATE_FIRING;
                self.anim.reset();
                b.speed_x = 0;
                b.speed_z = 0;
                b.counter2 = 0;
                state = STATE_FIRING;
            }
            if state == STATE_FIRING && self.firing(g) {
                state = STATE_TURN;
            }
            if state == STATE_HIT {
                let b = &mut self.base;
                if b.direction == UP_DIR || b.direction == DOWN_DIR {
                    b.direction = if px < b.x { LEFT_DIR } else { RIGHT_DIR };
                }
                b.state = STATE_HIT_COUNTDOWN;
                b.counter = HIT_DELAY;
                state = STATE_HIT_COUNTDOWN;
            }
            let b = &mut self.base;
            match state {
                STATE_WALK => match b.direction {
                    RIGHT_DIR => {
                        if b.speed_x < MAX_X_SPEED {
                            b.speed_x += X_ACCEL;
                        }
                        while g.check_wall(b.x + b.coll_x + b.coll_width + b.speed_x, b.y + b.coll_y + b.coll_height, b.z) && b.speed_x != 0 {
                            b.speed_x -= 1;
                        }
                        b.current_image = 5 + self.anim.next_frame(4, 5);
                        b.speed_z = 0;
                    }
                    LEFT_DIR => {
                        if b.speed_x > MIN_X_SPEED {
                            b.speed_x -= X_ACCEL;
                        }
                        while g.check_wall(b.x + b.coll_x + b.speed_x, b.y + b.coll_y + b.coll_height, b.z) && b.speed_x != 0 {
                            b.speed_x += 1;
                        }
                        b.current_image = self.anim.next_frame(4, 5);
                        b.speed_z = 0;
                    }
                    UP_DIR => {
                        if b.speed_z > MIN_Z_SPEED {
                            b.speed_z -= Z_ACCEL;
                        }
                        let cy = b.y + b.coll_y + b.coll_height;
                        while (g.check_wall(b.x + b.coll_x, cy, b.z + b.speed_z - 5)
                            || g.check_wall(b.x + b.coll_x + b.coll_width, cy, b.z + b.speed_z - 5))
                            && b.speed_z != 0
                        {
                            b.speed_z += 1;
                        }
                        b.current_image = 15 + self.anim.next_frame(4, 5);
                        b.speed_x = 0;
                    }
                    DOWN_DIR => {
                        if b.speed_z < MAX_Z_SPEED {
                            b.speed_z += Z_ACCEL;
                        }
                        let cy = b.y + b.coll_y + b.coll_height;
                        while (g.check_wall(b.x + b.coll_x, cy, b.z + b.speed_z + 5)
                            || g.check_wall(b.x + b.coll_x + b.coll_width, cy, b.z + b.speed_z + 5))
                            && b.speed_z != 0
                        {
                            b.speed_z -= 1;
                        }
                        b.current_image = 10 + self.anim.next_frame(4, 5);
                        b.speed_x = 0;
                    }
                    _ => b.direction = LEFT_DIR,
                },
                STATE_STOP => {
                    b.speed_x = 0;
                    b.speed_z = 0;
                }
                STATE_NONE => {
                    b.current_image = if b.direction == RIGHT_DIR || b.direction == LEFT_DIR {
                        if b.direction == RIGHT_DIR { 4 } else { 9 }
                    } else if b.direction == UP_DIR {
                        14
                    } else {
                        19
                    };
                    b.counter = 0;
                    self.anim.reset();
                }
                STATE_TURN => {
                    if b.direction == RIGHT_DIR || b.direction == LEFT_DIR {
                        b.direction = if b.direction == RIGHT_DIR { LEFT_DIR } else { RIGHT_DIR };
                        b.speed_x = 0;
                        b.counter = random() % MAX_NUM + 15;
                        b.state = STATE_WALK;
                    }
                }
                STATE_HIT_COUNTDOWN => {
                    b.current_image = if b.direction == RIGHT_DIR { 24 } else { 25 };
                }
                _ => {}
            }
        }

        let b = &mut self.base;
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
                UP_DIR | DOWN_DIR => {
                    if !g.check_floor(b.x + FOOT_RIGHT, b.y + b.height, b.z + b.speed_z)
                        && !g.check_floor(b.x + FOOT_RIGHT, b.y + b.height + TILE_SIDE_HEIGHT, b.z + b.speed_z)
                    {
                        b.speed_z = 0;
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
        b.z += b.speed_z;
        if b.x < 0 {
            b.x = 0;
        }
        if b.z > MIN_Z {
            b.z = MIN_Z;
            b.speed_z = 0;
            b.state = STATE_STOP;
            b.counter = 0;
        }
        if b.z < MAX_Z {
            b.z = MAX_Z;
            b.speed_z = 0;
            b.state = STATE_STOP;
            b.counter = 0;
        }
        b.layer = b.z / TILE_TOP_HEIGHT;

        if b.speed_x == 0 && b.speed_z == 0 && b.state != STATE_HIT_COUNTDOWN && b.state != STATE_FIRING && b.state != STATE_ALARM {
            if b.direction == LEFT_DIR || b.direction == RIGHT_DIR {
                b.current_image = if b.direction == RIGHT_DIR { 4 } else { 9 };
            } else if b.direction == UP_DIR || b.direction == DOWN_DIR {
                b.current_image = if b.direction == UP_DIR { 14 } else { 19 };
            }
        }
        Upd::Keep
    }
}

impl Obj for Soldier2 {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        self.do_update(g)
    }

    fn collision(&mut self, other: &mut dyn Obj, g: &mut Game) {
        if self.base.energy == 0 {
            return;
        }
        default_collision(&mut self.base, other.base());
        soldier_hit(&self.base, other.base(), g, "soldier2/h/dead", "soldier2/v/dead");
    }
}

pub struct BossSoldier2 {
    inner: Soldier2,
}

impl BossSoldier2 {
    pub fn new(x: i32, y: i32, z: i32) -> BossSoldier2 {
        let mut inner = Soldier2::new(x, y, z);
        inner.base.score = 1500;
        inner.base.energy = 30;
        BossSoldier2 { inner }
    }
}

impl Obj for BossSoldier2 {
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
