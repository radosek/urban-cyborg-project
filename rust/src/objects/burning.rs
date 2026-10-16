//! Burning body running around until it collapses (port of ref/objects/burning.cc).

use crate::engine::*;
use crate::icache;
use crate::objects::ai_sci::*;
use crate::objects::anim::Anim;
use crate::rng::random;

const PROB_WALK_END: i32 = 80;
const PROB_TURN_START: i32 = 81;
const PROB_TURN_END: i32 = 100;
const PROB_NUMBER: i32 = 100;
const MAX_NUM: i32 = 10;

const MAX_X_SPEED: i32 = 1;

const STATE_WALK: i32 = 0;
const STATE_TURN: i32 = 1;
const STATE_DEAD: i32 = 2;
const STATE_NONE: i32 = 3;
const STATE_BURN: i32 = 4;
const STATE_BURNING: i32 = 5;

const BURN_DELAY: i32 = 300;

pub struct BurningBody {
    pub base: Base,
    anim: Anim,
}

impl BurningBody {
    pub fn new(x: i32, y: i32, z: i32, move_: i32) -> BurningBody {
        let mut b = Base::new(x, y, z);
        b.images.extend(icache::seq("burning/burnh/", 1, 4, ".pcx"));
        b.images.extend(icache::seq("burning/burnv/", 1, 4, ".pcx"));
        b.images.extend(icache::seq("burning/", 1, 3, ".pcx"));
        b.current_image = if move_ != 0 { 0 } else { 10 };
        stand_on_feet(&mut b);
        b.friends = !0;
        b.enemies = 0;
        b.energy = 1;
        b.direction = RIGHT_DIR;
        b.state = STATE_NONE;
        b.counter2 = BURN_DELAY;
        BurningBody { base: b, anim: Anim::default() }
    }
}

impl Obj for BurningBody {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let anim = &mut self.anim;
        let b = &mut self.base;
        if b.counter != 0 {
            b.counter -= 1;
        }
        if b.counter2 != 0 {
            b.counter2 -= 1;
        }
        friction(b);

        if b.counter == 0 {
            if b.state == STATE_BURNING {
                b.state = STATE_DEAD;
            } else {
                let r = random() % PROB_NUMBER;
                if r <= PROB_WALK_END {
                    b.state = STATE_WALK;
                } else if (PROB_TURN_START..=PROB_TURN_END).contains(&r) {
                    b.state = STATE_TURN;
                }
                b.counter = random() % MAX_NUM + 15;
            }
        }

        if b.counter2 == 0 && b.state != STATE_DEAD && b.state != STATE_BURNING {
            b.state = STATE_BURN;
        }

        match b.state {
            STATE_NONE => {
                b.current_image = if b.direction == RIGHT_DIR { 0 } else { 4 };
                b.counter = 0;
                anim.reset();
            }
            STATE_WALK => walk_lr(b, anim, g, MAX_X_SPEED, 0, 4, 3),
            STATE_TURN => {
                b.direction = if b.direction == RIGHT_DIR { LEFT_DIR } else { RIGHT_DIR };
                b.speed_x = 0;
                b.counter = 0;
                b.state = STATE_WALK;
            }
            STATE_BURN | STATE_BURNING => {
                if b.state == STATE_BURN {
                    b.counter = 150;
                    b.state = STATE_BURNING;
                    anim.reset();
                }
                b.current_image = 8 + anim.next_frame(1, 5);
            }
            STATE_DEAD => {
                b.current_image = 10;
                return Upd::ToDekoration;
            }
            _ => {}
        }

        walker_tail(b, g);

        if b.speed_x == 0 && b.state != STATE_BURNING {
            b.state = STATE_TURN;
        }
        if b.energy <= 0 {
            return Upd::Delete;
        }
        Upd::Keep
    }
}
