//! Port of ref/objects/elevwire.cc

use super::elevstat::ElevatorStation;
use crate::engine::*;
use crate::icache;

const STATE_NONE: i32 = 0;
const STATE_DESTROY: i32 = 2;

pub struct ElevatorWire {
    pub base: Base,
    elev_controller: Option<ObjId>,
    wire_up: Option<ObjId>,
    wire_down: Option<ObjId>,
}

impl ElevatorWire {
    pub fn new(x: i32, y: i32, z: i32) -> ElevatorWire {
        let mut b = Base::new(x, y, z);
        b.images = vec![icache::get("connect.pcx")];
        b.height = b.images[0].h;
        b.width = b.images[0].w;
        b.x += TILE_WIDTH;
        b.y -= b.height;
        b.coll_width = b.width;
        b.coll_height = b.height;
        b.energy = 1;
        b.me = FRIEND_ELEVWIRE;
        b.state = STATE_NONE;
        ElevatorWire { base: b, elev_controller: None, wire_up: None, wire_down: None }
    }

    /// Walks down the wire chain to the next station and links it with station `w`.
    pub fn find_elevator_station(&mut self, w: ObjId, sender: ObjId, g: &mut Game) {
        match (self.elev_controller, self.wire_down) {
            (Some(ec), _) if ec != sender => {
                if let Some(st) = g.find::<ElevatorStation>(ec) {
                    st.set_elevator_station(w);
                }
            }
            (_, Some(wd)) => {
                let me = self.base.id;
                with_wire(g, wd, |next, g| next.find_elevator_station(w, me, g));
            }
            _ => {}
        }
        self.base.state = STATE_DESTROY;
    }
}

/// Runs `f` on the wire `id` with it taken out of the innerlayer list, so it can reach `g`.
pub(crate) fn with_wire(g: &mut Game, id: ObjId, f: impl FnOnce(&mut ElevatorWire, &mut Game)) {
    let Some(i) = g.innerlayer.iter().position(|s| matches!(s, Slot::Obj(o) if o.base().id == id)) else {
        return;
    };
    let Slot::Obj(mut o) = std::mem::replace(&mut g.innerlayer[i], Slot::Taken) else { unreachable!() };
    if let Some(wire) = o.downcast_mut::<ElevatorWire>() {
        f(wire, g);
    }
    g.innerlayer[i] = Slot::Obj(o);
}

impl Obj for ElevatorWire {
    crate::obj_boilerplate!();

    fn update(&mut self, _g: &mut Game) -> Upd {
        if self.base.state == STATE_DESTROY {
            return Upd::Delete;
        }
        Upd::Keep
    }

    fn collision(&mut self, other: &mut dyn Obj, _g: &mut Game) {
        let o = other.base();
        if o.me == FRIEND_ELEVSTAT {
            self.elev_controller = Some(o.id);
        }
        if o.me == FRIEND_ELEVWIRE {
            if o.y > self.base.y {
                self.wire_down = Some(o.id);
            } else if o.y < self.base.y {
                self.wire_up = Some(o.id);
            }
        }
    }
}
