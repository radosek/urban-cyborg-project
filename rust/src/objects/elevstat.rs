//! Port of ref/objects/elevstat.cc

use super::anim::Anim;
use super::elevator::Elevator;
use super::elevwire::with_wire;
use super::player::Player;
use crate::engine::*;
use crate::icache;

const STATE_NONE: i32 = 0;
const STATE_ACTIVATE: i32 = 1;
const STATE_ACTIVATED: i32 = 2;
const STATE_STOP: i32 = 3;
const STATE_SEND: i32 = 4;
const STATE_LOCATESTATION: i32 = 5;

const SEND_DELAY: i32 = 100;
const FRAME_DELAY: i32 = 2;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ElevatorStation {
    pub base: Base,
    anim: Anim,
    card: i32,
    direction_to_send: i32,
    elevator: Option<ObjId>,
    wire_up: Option<ObjId>,
    wire_down: Option<ObjId>,
    elev_stat: Option<ObjId>,
}

impl ElevatorStation {
    pub fn new(x: i32, y: i32, z: i32, dir: i32, card: i32) -> ElevatorStation {
        let mut b = Base::new(x, y, z);
        b.images = icache::seq("elepan", 0, 10, ".pcx");
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.x += TILE_WIDTH;
        b.y -= b.height;
        b.coll_x = -TILE_WIDTH;
        b.coll_y = 0;
        b.coll_width = b.width + TILE_WIDTH;
        b.coll_height = b.height;
        b.energy = 1;
        b.me = FRIEND_ELEVSTAT;
        b.state = STATE_LOCATESTATION;
        b.direction = dir;
        ElevatorStation {
            base: b,
            anim: Anim::default(),
            card,
            direction_to_send: 0,
            elevator: None,
            wire_up: None,
            wire_down: None,
            elev_stat: None,
        }
    }

    /// StartElevator, using `g.player` for the card check. While the player updates itself,
    /// `g.player` is empty: call `start_elevator_for` from there instead.
    pub fn start_elevator(&mut self, g: &mut Game) -> i32 {
        let card = self.card;
        let have = g.player.as_deref().map(|p| p.have_card(card)).unwrap_or(false);
        self.start(have, g)
    }

    /// StartElevator with an explicit player (for use inside the player's own update).
    pub fn start_elevator_for(&mut self, player: &Player, g: &mut Game) -> i32 {
        let have = player.have_card(self.card);
        self.start(have, g)
    }

    fn start(&mut self, have_card: bool, g: &mut Game) -> i32 {
        if self.base.counter != 0 {
            return 0;
        }
        self.base.counter = SEND_DELAY;
        if self.base.state == STATE_NONE || self.base.state == STATE_STOP {
            if self.card == CARD_NONE || have_card {
                self.base.state = STATE_ACTIVATE;
            } else {
                let msg = match self.card {
                    CARD_GREEN => "Green access denied",
                    CARD_BLUE => "Blue access denied",
                    CARD_RED => "Red access denied",
                    CARD_YELLOW => "Yellow access denied",
                    _ => "elevstat.cc: unknown card",
                };
                g.push_message(msg);
            }
        }
        if self.base.state == STATE_ACTIVATE {
            1
        } else {
            0
        }
    }

    pub fn send_elevator(&mut self, dir: i32, _g: &mut Game) {
        self.send(dir);
    }

    fn send(&mut self, dir: i32) {
        self.direction_to_send = dir;
        self.base.state = STATE_SEND;
    }

    /// Sets the partner station if none yet. The original then calls back
    /// `station->SetElevatorStation(this)`; the caller does that half (see `update`).
    pub fn set_elevator_station(&mut self, station: ObjId) {
        if self.elev_stat.is_none() {
            self.elev_stat = Some(station);
        }
    }

    fn elevator_y(&self, g: &mut Game) -> Option<i32> {
        self.elevator.and_then(|id| g.find::<Elevator>(id)).map(|e| e.base.y)
    }
}

#[typetag::serde]
impl Obj for ElevatorStation {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        if self.base.counter != 0 {
            self.base.counter -= 1;
        }

        match self.base.state {
            STATE_LOCATESTATION => {
                if self.base.direction == DOWN_DIR {
                    if let Some(wd) = self.wire_down {
                        let me = self.base.id;
                        with_wire(g, wd, |w, g| w.find_elevator_station(me, me, g));
                        // Reciprocal link: the station that just took us as partner becomes ours.
                        if self.elev_stat.is_none() {
                            let ids = g.ids_where(List::Innerlayer, |b| b.me == FRIEND_ELEVSTAT);
                            for id in ids {
                                if g.find::<ElevatorStation>(id).map(|s| s.elev_stat == Some(me)).unwrap_or(false) {
                                    self.elev_stat = Some(id);
                                    break;
                                }
                            }
                        }
                        self.base.state = STATE_NONE;
                    }
                } else {
                    self.base.state = STATE_NONE;
                }
            }
            STATE_SEND => {
                if let Some(e) = self.elevator.and_then(|id| g.find::<Elevator>(id)) {
                    e.start(self.direction_to_send);
                    self.base.state = STATE_NONE;
                    g.push_message("Elevator on its way");
                }
            }
            STATE_ACTIVATE => {
                if let Some(e) = self.elevator.and_then(|id| g.find::<Elevator>(id)) {
                    e.start(self.base.direction);
                    self.base.state = STATE_ACTIVATED;
                } else {
                    if let Some(s) = self.elev_stat.and_then(|id| g.find::<ElevatorStation>(id)) {
                        s.send(if self.base.direction == UP_DIR { DOWN_DIR } else { UP_DIR });
                    }
                    self.base.state = STATE_STOP;
                }
            }
            STATE_ACTIVATED => {
                if self.elevator.is_none() {
                    self.base.state = STATE_NONE;
                }
            }
            STATE_NONE => {
                if self.elevator.is_none() {
                    self.base.state = STATE_STOP;
                }
            }
            STATE_STOP => {
                if let Some(ey) = self.elevator_y(g) {
                    let bottom = self.base.y + self.base.height;
                    let arrived = (self.base.direction == UP_DIR && ey > bottom - 9)
                        || (self.base.direction == DOWN_DIR && ey < bottom - 7);
                    if arrived {
                        self.base.state = STATE_NONE;
                        if let Some(e) = self.elevator.and_then(|id| g.find::<Elevator>(id)) {
                            e.stop();
                        }
                    }
                }
            }
            _ => {}
        }

        self.elevator = None;
        self.base.current_image = self.anim.next_frame(self.base.num_images() - 1, FRAME_DELAY);
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, _g: &mut Game) {
        let o = other.base();
        if o.me == FRIEND_ELEVATOR {
            self.elevator = Some(o.id);
        }
        if o.me == FRIEND_ELEVWIRE {
            if self.base.direction == DOWN_DIR {
                self.wire_down = Some(o.id);
            } else {
                self.wire_up = Some(o.id);
            }
        }
    }
}
