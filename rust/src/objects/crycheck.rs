//! Inspector Crycheck (port of ref/objects/crycheck.cc).

use crate::engine::*;
use crate::icache;
use crate::objects::ai_sci::*;
use crate::objects::anim::Anim;
use crate::objects::bdyparts::Bodyparts;
use crate::objects::burning::BurningBody;
use crate::objects::frosen::FrosenBody;
use crate::objects::hs_bulle::HighSpeedBullet;
use crate::rng::random;

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
const STATE_FIRE: i32 = 6;
const STATE_FIRING: i32 = 7;
const STATE_WAITING: i32 = 9;
const STATE_DEAD: i32 = 10;
const STATE_COLLAPS: i32 = 11;
const STATE_DYING: i32 = 12;

const FIRE_RANGE: i32 = 150;
const FIRE_STRENGTH: i32 = 20;
const MAX_X_SPEED: i32 = 2;
const MAX_Y_SPEED: i32 = 16;
const X_ACCEL: i32 = 2;
const Y_ACCEL: i32 = 1;
const FIRE_DELAY: i32 = 60;

const FOOT_LEFT: i32 = 39;
const FOOT_WIDTH: i32 = 17;
const FOOT_RIGHT: i32 = FOOT_LEFT + FOOT_WIDTH;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Crycheck {
    pub base: Base,
    anim: Anim,
}

impl Crycheck {
    pub fn new(x: i32, y: i32, z: i32) -> Crycheck {
        let mut b = Base::new(x, y, z);
        b.images.extend(icache::seq("inspecto/inspv/", 1, 5, ".pcx"));
        b.images.extend(icache::seq("inspecto/insph/", 1, 5, ".pcx"));
        b.images.extend(icache::seq("inspecto/inspv/shot", 1, 4, ".pcx"));
        b.images.extend(icache::seq("inspecto/insph/shot", 1, 4, ".pcx"));
        b.images.extend(icache::seq("inspecto/insph/dead", 1, 12, ".pcx"));
        b.images.extend(icache::seq("inspecto/inspv/dead", 1, 12, ".pcx"));
        b.current_image = 1;
        stand_on_feet(&mut b);
        b.energy = 1;
        b.direction = RIGHT_DIR;
        b.state = STATE_NONE;
        b.score = 500;
        b.friends = !FRIEND_PLAYER;
        b.enemies = ENEMY_PLAYER;
        b.me = FRIEND_AGENT;
        Crycheck { base: b, anim: Anim::default() }
    }

    fn fire(b: &mut Base, anim: &mut Anim, g: &mut Game, px: i32, player_layer: i32) {
        let right = b.direction == RIGHT_DIR;
        if b.counter2 == 0 {
            let r = anim.next_frame(2, 20);
            b.current_image = (if right { 14 } else { 10 }) + r;
            if r < 2 {
                return;
            }
            b.counter2 = 1;
            anim.reset();
        }
        if b.counter2 == 1 || b.counter2 == 2 {
            let mut r = anim.next_frame(2, 10);
            if r == 1 && b.counter2 == 1 {
                b.counter2 = 2;
                g.create_object(HighSpeedBullet::new(
                    b.x + if right { 100 } else { 12 },
                    b.y + 13,
                    b.z,
                    b.direction,
                    (!FRIEND_PLAYER) as i32,
                    FIRE_STRENGTH,
                ));
                if right {
                    let sx = -(random() % 5);
                    let sy = -(random() % 8);
                    g.create_effect(Bodyparts::new(b.x + 90, b.y + 22, b.z, "soldier/hylsa", 1, sx, sy, 0));
                } else {
                    let sx = random() % 5;
                    let sy = -(random() % 8);
                    g.create_effect(Bodyparts::new(b.x + 23, b.y + 22, b.z, "soldier/hylsa", 1, sx, sy, 0));
                }
            } else if r == 2 {
                b.counter2 = 1;
                if b.direction == LEFT_DIR && px < b.x && px > b.x - FIRE_RANGE && player_layer == b.layer {
                    r = 0;
                } else if right && px > b.x && px < b.x + FIRE_RANGE && player_layer == b.layer {
                    r = 0;
                }
            }
            b.current_image = (if right { 16 } else { 12 }) + r;
            if r < 2 {
                return;
            }
            b.counter2 = 3;
            anim.reset();
        }
        if b.counter2 == 3 {
            let r = anim.next_frame(2, 20);
            b.current_image = (if right { 15 } else { 11 }) - r;
            if r == 2 {
                b.state = STATE_NONE;
                b.current_image = if right { 4 } else { 9 };
            }
        }
    }
}

#[typetag::serde]
impl Obj for Crycheck {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let (px, player_layer) = (g.pbase().x, g.pbase().layer);
        let anim = &mut self.anim;
        let b = &mut self.base;

        if b.state == STATE_DEAD {
            return Upd::Delete;
        }
        if b.counter != 0 {
            b.counter -= 1;
        }

        if b.counter == 0
            && b.state != STATE_COLLAPS
            && b.state != STATE_WAITING
            && b.state != STATE_DYING
            && b.state != STATE_FIRING
        {
            let r = random() % PROB_NUMBER;
            if r <= PROB_WALK_END {
                b.state = STATE_WALK;
                b.direction = random() % 4;
            } else if (PROB_STOP_START..=PROB_STOP_END).contains(&r) || b.state == STATE_STOP || b.state == STATE_TURN {
                b.state = STATE_STOP;
            } else if (PROB_TURN_START..=PROB_TURN_END).contains(&r) {
                b.state = STATE_TURN;
            }
            b.counter = random() % MAX_NUM + 15;
        }

        if ![STATE_FIRE, STATE_FIRING, STATE_TURN, STATE_COLLAPS, STATE_DYING, STATE_WAITING].contains(&b.state) {
            let d = b.direction;
            if px < b.x
                && px > b.x - FIRE_RANGE
                && (d == LEFT_DIR || d == UP_DIR || d == DOWN_DIR)
                && player_layer == b.layer
            {
                b.state = STATE_FIRE;
            } else if px > b.x
                && px < b.x + FIRE_RANGE
                && (d == RIGHT_DIR || d == UP_DIR || d == DOWN_DIR)
                && player_layer == b.layer
            {
                b.state = STATE_FIRE;
            }
        }

        if b.speed_y == 0 {
            match b.state {
                STATE_COLLAPS | STATE_WAITING => {
                    let mut waiting = b.state == STATE_WAITING;
                    if !waiting {
                        b.direction = if px > b.x { RIGHT_DIR } else { LEFT_DIR };
                        b.speed_x = 0;
                        b.speed_z = 0;
                        let r = anim.next_frame(4, 5);
                        b.current_image = (if b.direction == RIGHT_DIR { 18 } else { 30 }) + r;
                        if r == 4 {
                            b.state = STATE_WAITING;
                            anim.reset();
                            waiting = true;
                        }
                    }
                    if waiting {
                        b.speed_x = 0;
                        b.speed_z = 0;
                        b.current_image = (if b.direction == RIGHT_DIR { 22 } else { 34 }) + anim.next_frame(1, 10);
                    }
                }
                STATE_DYING => {
                    if b.direction == -100 {
                        b.direction = if px > b.x { RIGHT_DIR } else { LEFT_DIR };
                    }
                    let r = anim.next_frame(6, 5);
                    b.current_image = (if b.direction == RIGHT_DIR { 23 } else { 35 }) + r;
                    if r == 6 {
                        return Upd::ToDekoration;
                    }
                }
                STATE_WALK => match b.direction {
                    RIGHT_DIR => {
                        if b.speed_x < MAX_X_SPEED {
                            b.speed_x += X_ACCEL;
                        }
                        while g.check_wall(b.x + b.coll_x + b.coll_width + b.speed_x, b.y + b.coll_y + b.coll_height, b.z)
                            && b.speed_x != 0
                        {
                            b.speed_x -= 1;
                        }
                        b.current_image = 5 + anim.next_frame(4, 5);
                        b.speed_z = 0;
                    }
                    LEFT_DIR => {
                        if b.speed_x > -MAX_X_SPEED {
                            b.speed_x -= X_ACCEL;
                        }
                        while g.check_wall(b.x + b.coll_x + b.speed_x, b.y + b.coll_y + b.coll_height, b.z) && b.speed_x != 0 {
                            b.speed_x += 1;
                        }
                        b.current_image = anim.next_frame(4, 5);
                        b.speed_z = 0;
                    }
                    _ => b.direction = LEFT_DIR,
                },
                STATE_STOP => {
                    b.speed_x = 0;
                    b.speed_z = 0;
                }
                STATE_TURN => {
                    if b.direction == RIGHT_DIR || b.direction == LEFT_DIR {
                        b.direction = if b.direction == RIGHT_DIR { LEFT_DIR } else { RIGHT_DIR };
                        b.speed_x = 0;
                        b.counter = random() % MAX_NUM + 15;
                        b.state = STATE_WALK;
                    }
                }
                STATE_NONE => {
                    if b.direction == RIGHT_DIR || b.direction == LEFT_DIR {
                        b.current_image = if b.direction == RIGHT_DIR { 4 } else { 9 };
                    }
                    b.counter = 0;
                    anim.reset();
                }
                STATE_FIRE | STATE_FIRING => {
                    if b.state == STATE_FIRE {
                        b.direction = if px < b.x { LEFT_DIR } else { RIGHT_DIR };
                        b.counter = 2 * FIRE_DELAY;
                        b.state = STATE_FIRING;
                        anim.reset();
                        b.speed_x = 0;
                        b.speed_z = 0;
                        b.counter2 = 0;
                    }
                    Crycheck::fire(b, anim, g, px, player_layer);
                }
                _ => {}
            }
        }

        if b.speed_y == 0 {
            let foot = match b.direction {
                LEFT_DIR => Some(FOOT_LEFT),
                RIGHT_DIR => Some(FOOT_RIGHT),
                _ => None,
            };
            if let Some(f) = foot {
                if !g.check_floor(b.x + f + b.speed_x, b.y + b.height, b.z)
                    && !g.check_floor(b.x + f + b.speed_x, b.y + b.height + TILE_SIDE_HEIGHT, b.z)
                {
                    b.speed_x = 0;
                    b.state = STATE_TURN;
                }
            }
        }

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

        if b.speed_x == 0
            && b.speed_z == 0
            && b.state != STATE_FIRING
            && b.state != STATE_COLLAPS
            && b.state != STATE_WAITING
            && b.state != STATE_DYING
            && (b.direction == LEFT_DIR || b.direction == RIGHT_DIR)
        {
            b.current_image = if b.direction == RIGHT_DIR { 4 } else { 9 };
        }
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, g: &mut Game) {
        if self.base.state == STATE_DEAD {
            return;
        }
        default_collision(&mut self.base, other.base());
        let o = other.base();
        let b = &mut self.base;
        if o.strength > 0 && o.friends & b.me == 0 && b.state != STATE_DYING {
            // Instant kills only once the hit has used up the energy (the boss has 30).
            let spent = b.energy == 0;
            if o.me & ENEMY_EXPLOSION != 0 {
                if spent {
                    b.state = STATE_DEAD;
                    death_by_explosion(b, g);
                    return;
                }
            } else if o.me & (ENEMY_FIREBALL | ENEMY_BEAM) != 0 {
                if spent {
                    g.create_effect(BurningBody::new(b.x, b.y + b.height, b.z, 1));
                    b.state = STATE_DEAD;
                }
            } else if o.me & ENEMY_ICEBALL != 0 {
                if spent {
                    g.create_object(FrosenBody::new(b.x, b.y + b.height, b.z, b.direction));
                    b.state = STATE_DEAD;
                }
            } else if o.me & (ENEMY_HS_BULLET | ENEMY_PLASMA) != 0 {
                if b.state == STATE_COLLAPS {
                    b.state = STATE_DEAD;
                    death_by_explosion(b, g);
                    return;
                } else if b.state == STATE_WAITING {
                    b.direction = -100;
                    b.state = STATE_DYING;
                    self.anim.reset();
                    return;
                }
                blood_spray(b, g);
                if spent {
                    b.state = STATE_COLLAPS;
                    b.score += 50;
                    self.anim.reset();
                }
            }
            b.counter = 0;
        }
    }
}

crate::boss_with_card!(BossCrycheck, Crycheck);

impl BossCrycheck {
    pub fn new(x: i32, y: i32, z: i32) -> BossCrycheck {
        let mut inner = Crycheck::new(x, y, z);
        inner.base.score = 1500;
        inner.base.energy = 30;
        BossCrycheck { inner }
    }
}
