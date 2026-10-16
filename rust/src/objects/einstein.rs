//! Einstein (port of ref/objects/einstein.cc).

use crate::engine::*;
use crate::icache;
use crate::objects::ai_sci::*;
use crate::objects::anim::Anim;
use crate::objects::animate::Animation;
use crate::objects::burning::BurningBody;
use crate::objects::frosen::FrosenBody;
use crate::rng::random;

const PROB_WALK_END: i32 = 60;
const PROB_STOP_START: i32 = 61;
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Einstein {
    pub base: Base,
    anim: Anim,
}

impl Einstein {
    pub fn new(x: i32, y: i32, z: i32) -> Einstein {
        let mut b = Base::new(x, y, z);
        b.images.extend(icache::seq("prof2/prof2h/", 1, 5, ".pcx"));
        b.images.extend(icache::seq("prof2/prof2v/", 1, 5, ".pcx"));
        b.images.extend(icache::seq("prof2/prof2h/dead", 1, 7, ".pcx"));
        b.images.extend(icache::seq("prof2/prof2v/dead", 1, 7, ".pcx"));
        b.images.extend(icache::seq("prof2/prof2h/write", 1, 3, ".pcx"));
        b.images.extend(icache::seq("prof2/prof2v/write", 1, 3, ".pcx"));
        b.current_image = 5;
        stand_on_feet(&mut b);
        b.energy = 2;
        b.direction = LEFT_DIR;
        b.state = STATE_NONE;
        b.score = 100;
        b.friends = !FRIEND_PLAYER;
        b.enemies = ENEMY_PLAYER;
        b.me = FRIEND_SCIENTIST;
        Einstein { base: b, anim: Anim::default() }
    }
}

#[typetag::serde]
impl Obj for Einstein {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
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
                b.current_image = (if b.direction == RIGHT_DIR { 24 } else { 27 }) + anim.next_frame(2, WRITE_DELAY);
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
                let dir = if b.direction == RIGHT_DIR { "prof2/prof2h/dead" } else { "prof2/prof2v/dead" };
                g.create_effect(Animation::new(b.x, b.y + b.height, b.z, dir, 7, 6, 0, 0, 0));
                blood_spray(b, g);
                b.state = STATE_DEAD;
            }
            b.counter = 0;
        }
    }
}

crate::boss_with_card!(BossEinstein, Einstein);

impl BossEinstein {
    pub fn new(x: i32, y: i32, z: i32) -> BossEinstein {
        BossEinstein { inner: Einstein::new(x, y, z) }
    }
}
