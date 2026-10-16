//! Guard (port of ref/objects/vaktm.cc).

use crate::engine::*;
use crate::icache;
use crate::objects::ai_sci::*;
use crate::objects::anim::Anim;
use crate::objects::burning::BurningBody;
use crate::objects::explode::Explosion;
use crate::rng::random;

const PROB_WALK_END: i32 = 80;
const PROB_TURN_START: i32 = 81;
const PROB_TURN_END: i32 = 100;
const PROB_NUMBER: i32 = 100;
const MAX_NUM: i32 = 15;

const STATE_NONE: i32 = 0;
const STATE_WALK: i32 = 1;
const STATE_TURN: i32 = 2;

const MAX_X_SPEED: i32 = 1;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Vaktm {
    pub base: Base,
    anim: Anim,
}

impl Vaktm {
    pub fn new(x: i32, y: i32, z: i32) -> Vaktm {
        let mut b = Base::new(x, y, z);
        b.images.extend(icache::seq("vaktm/v/", 1, 4, ".pcx"));
        b.images.extend(icache::seq("vaktm/h/", 1, 4, ".pcx"));
        stand_on_feet(&mut b);
        b.energy = 1;
        b.direction = LEFT_DIR;
        b.state = STATE_NONE;
        b.score = 50;
        b.friends = !FRIEND_PLAYER;
        b.enemies = ENEMY_PLAYER;
        b.me = FRIEND_VAKTM;
        Vaktm { base: b, anim: Anim::default() }
    }
}

#[typetag::serde]
impl Obj for Vaktm {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let anim = &mut self.anim;
        let b = &mut self.base;
        if b.counter != 0 {
            b.counter -= 1;
        }
        friction(b);

        if b.counter == 0 {
            let r = random() % PROB_NUMBER;
            if r <= PROB_WALK_END {
                b.state = STATE_WALK;
            } else if (PROB_TURN_START..=PROB_TURN_END).contains(&r) {
                b.state = STATE_TURN;
            }
            b.counter = random() % MAX_NUM + 15;
        }

        match b.state {
            STATE_NONE => {
                b.current_image = if b.direction == RIGHT_DIR { 4 } else { 0 };
                b.counter = 0;
                anim.reset();
            }
            STATE_WALK => walk_lr(b, anim, g, MAX_X_SPEED, 4, 0, 3),
            STATE_TURN => {
                b.direction = if b.direction == RIGHT_DIR { LEFT_DIR } else { RIGHT_DIR };
                b.speed_x = 0;
                b.counter = 0;
                b.state = STATE_WALK;
            }
            _ => {}
        }

        walker_tail(b, g);

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
        let o = other.base();
        let b = &mut self.base;
        if o.strength > 0 && o.friends & b.me == 0 {
            g.create_object(Explosion::new(b.x, b.y + b.height - 75, b.z, 0, 0, 0, 0));
            g.create_object(Explosion::new(b.x + 15, b.y + b.height - 50, b.z, 0, 0, 0, 0));
            g.create_object(Explosion::new(b.x - 15, b.y + b.height - 50, b.z, 0, 0, 0, 0));
            g.create_effect(BurningBody::new(b.x, b.y + b.height, b.z, 1));
            b.counter = 0;
            b.energy = 0;
        }
    }
}
