//! Scientist (port of ref/objects/science.cc).

use crate::engine::*;
use crate::icache;
use crate::objects::ai_sci::*;
use crate::objects::anim::Anim;
use crate::objects::burning::BurningBody;
use crate::objects::frosen::FrosenBody;
use crate::rng::random;

const PROB_WALK_END: i32 = 40;
const PROB_STOP_START: i32 = 41;
const PROB_STOP_END: i32 = 80;
const PROB_TURN_START: i32 = 81;
const PROB_TURN_END: i32 = 100;
const PROB_NUMBER: i32 = 100;
const MAX_NUM: i32 = 15;

const STATE_NONE: i32 = 0;
const STATE_STOP: i32 = 1;
const STATE_WALK: i32 = 2;
const STATE_TURN: i32 = 3;
const STATE_COLLAPS: i32 = 6;
const STATE_DYING: i32 = 7;
const STATE_WAITING: i32 = 8;
const STATE_DEAD: i32 = 9;
const STATE_WRITE: i32 = 10;
const STATE_WRITING: i32 = 11;

const MAX_X_SPEED: i32 = 2;
const WRITE_DELAY: i32 = 30;

pub struct Scientist {
    pub base: Base,
    anim: Anim,
}

impl Scientist {
    pub fn new(x: i32, y: i32, z: i32) -> Scientist {
        let mut b = Base::new(x, y, z);
        b.images.extend(icache::seq("profh/", 1, 5, ".pcx"));
        b.images.extend(icache::seq("profv/", 1, 5, ".pcx"));
        b.images.extend(icache::seq("profh/dead", 1, 12, ".pcx"));
        b.images.extend(icache::seq("profv/dead", 1, 12, ".pcx"));
        b.images.extend(icache::seq("profh/write", 1, 3, ".pcx"));
        b.images.extend(icache::seq("profv/write", 1, 3, ".pcx"));
        b.current_image = 5;
        stand_on_feet(&mut b);
        b.energy = 2;
        b.direction = LEFT_DIR;
        b.state = STATE_NONE;
        b.score = 100;
        b.friends = !FRIEND_PLAYER;
        b.enemies = ENEMY_PLAYER;
        b.me = FRIEND_SCIENTIST;
        Scientist { base: b, anim: Anim::default() }
    }
}

impl Obj for Scientist {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let px = g.pbase().x;
        let anim = &mut self.anim;
        let b = &mut self.base;

        if b.state == STATE_DEAD {
            crate::sound::play_sfx("samples/scream.wav");
            return Upd::Delete;
        }

        if b.counter != 0 {
            b.counter -= 1;
        }
        friction(b);

        if b.counter == 0 && b.state != STATE_COLLAPS && b.state != STATE_WAITING && b.state != STATE_DYING {
            let r = random() % PROB_NUMBER;
            if r <= PROB_WALK_END {
                b.state = STATE_WALK;
            } else if (PROB_STOP_START..=PROB_STOP_END).contains(&r) || b.state == STATE_STOP {
                b.state = STATE_STOP;
            } else if (PROB_TURN_START..=PROB_TURN_END).contains(&r) {
                b.state = STATE_TURN;
            }
            b.counter = random() % MAX_NUM + 15;
        }

        match b.state {
            STATE_WRITE | STATE_WRITING => {
                if b.state == STATE_WRITE {
                    b.state = STATE_WRITING;
                    anim.reset();
                    b.counter = 3 * WRITE_DELAY;
                }
                b.current_image = (if b.direction == RIGHT_DIR { 34 } else { 37 }) + anim.next_frame(2, WRITE_DELAY);
            }
            STATE_COLLAPS | STATE_WAITING => {
                let mut waiting = b.state == STATE_WAITING;
                if !waiting {
                    b.speed_x = 0;
                    b.speed_z = 0;
                    b.direction = if px > b.x { RIGHT_DIR } else { LEFT_DIR };
                    let r = anim.next_frame(4, 5);
                    b.current_image = (if b.direction == RIGHT_DIR { 10 } else { 22 }) + r;
                    if r == 4 {
                        b.state = STATE_WAITING;
                        anim.reset();
                        waiting = true;
                    }
                }
                if waiting {
                    b.speed_x = 0;
                    b.speed_z = 0;
                    b.current_image = (if b.direction == RIGHT_DIR { 14 } else { 26 }) + anim.next_frame(1, 10);
                }
            }
            STATE_DYING => {
                if b.direction == -100 {
                    b.direction = if px > b.x { RIGHT_DIR } else { LEFT_DIR };
                }
                let r = anim.next_frame(6, 5);
                b.current_image = (if b.direction == RIGHT_DIR { 15 } else { 27 }) + r;
                if r == 6 {
                    return Upd::ToDekoration;
                }
            }
            STATE_NONE => {
                b.current_image = if b.direction == RIGHT_DIR { 0 } else { 5 };
                b.counter = 0;
                anim.reset();
            }
            STATE_WALK => walk_lr(b, anim, g, MAX_X_SPEED, 0, 5, 4),
            STATE_TURN => {
                b.direction = if b.direction == RIGHT_DIR { LEFT_DIR } else { RIGHT_DIR };
                b.speed_x = 0;
                b.counter = 0;
                b.state = STATE_WALK;
            }
            STATE_STOP => {
                anim.reset();
                b.speed_x = 0;
            }
            _ => {}
        }

        walker_tail(b, g);

        if b.speed_x == 0
            && b.state != STATE_COLLAPS
            && b.state != STATE_WAITING
            && b.state != STATE_DYING
            && b.state != STATE_WRITE
            && b.state != STATE_WRITING
        {
            b.current_image = if b.direction == RIGHT_DIR { 4 } else { 9 };
            b.state = STATE_WRITE;
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
            if o.me & ENEMY_EXPLOSION != 0 {
                b.state = STATE_DEAD;
                death_by_explosion(b, g);
                b.score += 50;
                return;
            } else if o.me & (ENEMY_FIREBALL | ENEMY_BEAM) != 0 {
                g.create_effect(BurningBody::new(b.x, b.y + b.height, b.z, 1));
                b.state = STATE_DEAD;
            } else if o.me & ENEMY_ICEBALL != 0 {
                g.create_object(FrosenBody::new(b.x, b.y + b.height, b.z, b.direction));
                b.state = STATE_DEAD;
            } else if o.me & (ENEMY_HS_BULLET | ENEMY_PLASMA) != 0 {
                if b.state == STATE_COLLAPS {
                    b.state = STATE_DEAD;
                    death_by_explosion2(b, g);
                    return;
                } else if b.state == STATE_WAITING {
                    b.direction = -100;
                    b.state = STATE_DYING;
                    self.anim.reset();
                    b.score += 100;
                    return;
                }
                blood_spray(b, g);
                b.state = STATE_COLLAPS;
                b.score += 50;
                self.anim.reset();
            }
            b.counter = 0;
        }
    }
}

crate::boss_with_card!(BossScientist, Scientist);

impl BossScientist {
    pub fn new(x: i32, y: i32, z: i32) -> BossScientist {
        BossScientist { inner: Scientist::new(x, y, z) }
    }
}
