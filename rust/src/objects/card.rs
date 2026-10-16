//! Access card (objects/card.cc).

use crate::engine::*;
use crate::icache;
use crate::objects::player::Player;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Card {
    pub base: Base,
    card: i32,
}

impl Card {
    pub fn new(x: i32, y: i32, z: i32, card: i32) -> Card {
        let name = match card {
            CARD_BLUE => "items/bluecard.pcx",
            CARD_GREEN => "items/greencar.pcx",
            CARD_RED => "items/redcard.pcx",
            _ => {
                eprintln!("Unknown Card");
                std::process::exit(1);
            }
        };
        let mut b = Base::new(x, y, z);
        b.images = vec![icache::get(name), icache::get("mine2.pcx")];
        b.current_image = 0;
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.x += TILE_WIDTH / 2;
        b.x -= b.width / 2;
        b.y -= b.height;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.energy = 1;
        // Fall to the floor
        b.speed_y = 5;
        b.friends = FRIEND_PLAYER;
        b.enemies = !b.friends;
        b.me = FRIEND_CARD;
        Card { base: b, card }
    }
}

#[typetag::serde]
impl Obj for Card {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        if b.counter != 0 {
            b.counter -= 1;
        }
        if b.counter == 0 {
            b.current_image = if b.current_image == 0 { 1 } else { 0 };
            b.counter = 10;
        }
        if g.check_floor(b.x, b.y + b.height, b.z) || g.check_floor(b.x + b.width, b.y + b.height, b.z) {
            b.speed_y = 0;
        } else {
            b.y += b.speed_y;
        }
        // Delete if already used
        if b.energy == 0 {
            return Upd::Delete;
        }
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, g: &mut Game) {
        if self.base.energy == 0 || self.base.friends & other.base().me == 0 {
            return;
        }
        self.base.energy = 0;
        if let Some(p) = other.downcast_mut::<Player>() {
            p.give_card(self.card);
        }
        match self.card {
            CARD_GREEN => g.push_message("Green access granted"),
            CARD_RED => g.push_message("Red access granted"),
            CARD_BLUE => g.push_message("Blue access granted"),
            _ => {}
        }
    }
}
