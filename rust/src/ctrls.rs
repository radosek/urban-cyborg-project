//! Player controllers (keyboard / gamepad), driven by the saved KeyConf.

use crate::config::{self, *};
use crate::keys::*;
use crate::sys;

pub const JOYSTICK_NONE: i32 = 0x00;
pub const JOYSTICK_UP: i32 = 0x01;
pub const JOYSTICK_DOWN: i32 = 0x02;
pub const JOYSTICK_LEFT: i32 = 0x03;
pub const JOYSTICK_RIGHT: i32 = 0x04;
pub const JOYSTICK_B1: i32 = 0x05;
pub const JOYSTICK_B8: i32 = 0x0C;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    None,
    Keyboard,
    Gamepad,
}

/// Edge detector for the weapon-cycle buttons (the original used function statics).
#[derive(Default)]
struct Edge(bool);

impl Edge {
    fn hit(&mut self, down: bool) -> bool {
        let r = down && !self.0;
        self.0 = down;
        r
    }
}

pub struct Controls {
    kind: Kind,
    conf: KeyConf,
    next: Edge,
    prev: Edge,
}

fn joy_value(code: i32) -> bool {
    let j = sys::joy();
    match code {
        JOYSTICK_UP => j.up,
        JOYSTICK_DOWN => j.down,
        JOYSTICK_LEFT => j.left,
        JOYSTICK_RIGHT => j.right,
        c if (JOYSTICK_B1..=JOYSTICK_B8).contains(&c) => j.b[(c - JOYSTICK_B1) as usize],
        _ => false,
    }
}

impl Controls {
    /// `ctrl_type`: CONTROLLER_KEYBOARD, a CONTROLLER_GAMEPADn, or CONTROLLER_NONE (demo/dummy).
    /// Unknown types fall back to the keyboard (the original left the controller uninitialised).
    pub fn new(ctrl_type: i32) -> Controls {
        let conf = config::get();
        let kind = match ctrl_type {
            CONTROLLER_NONE => Kind::None,
            CONTROLLER_GAMEPAD2 | CONTROLLER_GAMEPAD4 | CONTROLLER_GAMEPAD6 | CONTROLLER_GAMEPAD8 | CONTROLLER_JOYSTICK => Kind::Gamepad,
            _ => Kind::Keyboard,
        };
        Controls { kind, conf, next: Edge::default(), prev: Edge::default() }
    }

    fn k(&self, code: i32) -> bool {
        sys::key(code as usize)
    }

    fn get(&self, code: i32) -> bool {
        match self.kind {
            Kind::None => false,
            Kind::Keyboard => self.k(code),
            Kind::Gamepad => joy_value(code),
        }
    }

    pub fn up(&self) -> bool { self.get(self.conf.key_up) }
    pub fn down(&self) -> bool { self.get(self.conf.key_down) }
    pub fn left(&self) -> bool { self.get(self.conf.key_left) }
    pub fn right(&self) -> bool { self.get(self.conf.key_right) }
    pub fn jump(&self) -> bool { self.get(self.conf.key_jump) }
    pub fn fire(&self) -> bool { self.get(self.conf.key_fire) }

    /// Number keys 1..9 select weapons directly regardless of controller.
    pub fn weapon(&self, n: usize) -> bool {
        self.kind != Kind::None && sys::key(KEY_1 + n - 1)
    }

    /// Next/previous weapon. The original swapped these two keyboard bindings; fixed here.
    /// Gamepads with unset bindings fall back to INSERT/DEL like the keyboard path.
    pub fn next_weapon(&mut self) -> bool {
        let down = match self.kind {
            Kind::None => false,
            Kind::Keyboard => {
                let k = if self.conf.key_nextweapon != 0 { self.conf.key_nextweapon } else { KEY_DEL as i32 };
                self.k(k)
            }
            Kind::Gamepad => joy_value(self.conf.key_nextweapon) || self.k(KEY_DEL as i32),
        };
        self.next.hit(down)
    }

    pub fn prev_weapon(&mut self) -> bool {
        let down = match self.kind {
            Kind::None => false,
            Kind::Keyboard => {
                let k = if self.conf.key_prevweapon != 0 { self.conf.key_prevweapon } else { KEY_INSERT as i32 };
                self.k(k)
            }
            Kind::Gamepad => joy_value(self.conf.key_prevweapon) || self.k(KEY_INSERT as i32),
        };
        self.prev.hit(down)
    }
}
