//! Port of ref/objects/elevator.cc

use crate::engine::*;
use crate::icache;

const STATE_NONE: i32 = 0;
const STATE_MOVE: i32 = 1;
const STATE_STOP: i32 = 2;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Elevator {
    pub base: Base,
    /// Set when the player touched the elevator since the last update.
    passenger: bool,
}

impl Elevator {
    pub fn new(x: i32, y: i32, z: i32) -> Elevator {
        let mut b = Base::new(x, y, z);
        b.images = vec![icache::get("elevator.pcx")];
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.x += TILE_WIDTH;
        b.y -= b.height;
        b.y += 8; // hack from the original
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.energy = 1;
        b.me = FRIEND_ELEVATOR;
        b.state = STATE_NONE;
        b.direction = UP_DIR;
        Elevator { base: b, passenger: false }
    }

    pub fn activate(&mut self, dir: i32, _g: &mut Game) {
        self.start(dir);
    }

    pub fn deactivate(&mut self, _g: &mut Game) {
        self.stop();
    }

    /// Declared but never defined in the original; behaves like `activate`.
    pub fn send_elevator(&mut self, dir: i32, _g: &mut Game) {
        self.start(dir);
    }

    /// `activate` without a Game, for callers that reach the elevator through `g.find`.
    pub(crate) fn start(&mut self, dir: i32) {
        self.base.direction = dir;
        self.base.state = STATE_MOVE;
    }

    pub(crate) fn stop(&mut self) {
        self.base.state = STATE_STOP;
    }
}

#[typetag::serde]
impl Obj for Elevator {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        if b.state == STATE_MOVE {
            match b.direction {
                UP_DIR => b.speed_y = -1,
                DOWN_DIR => b.speed_y = 1,
                LEFT_DIR => b.speed_x = -1,
                RIGHT_DIR => b.speed_x = 1,
                _ => {}
            }
            b.counter += 1;
        } else {
            b.speed_y = 0;
            b.speed_x = 0;
            b.counter = 0;
        }

        if self.passenger && b.state == STATE_MOVE {
            if let Some(p) = g.player.as_deref_mut() {
                let pb = p.base_mut();
                pb.y += b.speed_y;
                pb.x += b.speed_x;
            }
        }

        b.y += b.speed_y;
        b.x += b.speed_x;
        self.passenger = false;
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, _g: &mut Game) {
        if other.base().me != FRIEND_ELEVSTAT && other.is_player() {
            self.passenger = true;
        }
    }
}
