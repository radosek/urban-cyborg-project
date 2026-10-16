//! Level exit door (objects/door.cc).

use crate::engine::*;
use crate::icache;
use crate::objects::anim::Anim;
use crate::objects::player::Player;
use crate::sound;

const FRAME_DELAY: i32 = 5;

pub struct Door {
    pub base: Base,
    card: i32,
    anim: Anim,
}

impl Door {
    pub fn new(x: i32, y: i32, z: i32, card: i32) -> Door {
        let _ = z;
        let mut b = Base::new(x, y, 0);
        b.images = icache::seq("new-door/", 1, 5, ".pcx");
        b.current_image = 0;
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.x += TILE_WIDTH / 2;
        b.x -= b.width / 2;
        b.y -= b.height - 2;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.energy = 1;
        // Fall to the floor
        b.speed_y = 5;
        b.friends = FRIEND_PLAYER;
        b.enemies = !b.friends;
        b.me = FRIEND_DOOR;
        Door { base: b, card, anim: Anim::default() }
    }
}

impl Obj for Door {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        if g.check_floor(b.x, b.y + b.height, b.z) || g.check_floor(b.x + b.width, b.y + b.height, b.z) {
            b.speed_y = 0;
        } else {
            b.y += b.speed_y;
        }

        while g.check_floor(b.x + b.width / 2, b.y + b.height - 2, b.z) {
            b.y -= 1;
        }

        if b.counter2 == 0 || b.counter2 == 2 {
            return Upd::Keep;
        }

        b.current_image = self.anim.next_frame(5, FRAME_DELAY);

        if b.current_image == 5 {
            b.current_image = 4;
            b.counter2 = 2;
            if self.card == CARD_EXTRA_LEVEL {
                g.play_extra_level();
            } else {
                g.clear_level();
            }
        }

        // Delete if already used
        if self.base.energy == 0 {
            return Upd::Delete;
        }
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, g: &mut Game) {
        if self.base.energy == 0 || self.base.friends & other.base().me == 0 {
            return;
        }
        let has_card = other.downcast_ref::<Player>().map(|p| p.have_card(self.card)).unwrap_or(false);
        if has_card && self.base.counter2 == 0 {
            g.push_message("Level Clear");
            sound::play_sfx("samples/fire4.wav");
            self.base.counter2 = 1;
            self.base.counter3 = 1;
        } else if self.base.counter3 == 0 {
            let msg = match self.card {
                CARD_GREEN => "Green access denied",
                CARD_BLUE => "Blue access denied",
                CARD_RED => "Red access denied",
                CARD_EXTRA_LEVEL => {
                    g.push_message("Extra level found");
                    sound::play_sfx("samples/fire4.wav");
                    self.base.counter2 = 1;
                    return;
                }
                // The original pushed an uninitialised buffer here.
                _ => return,
            };
            g.push_message(msg);
        }
    }
}
