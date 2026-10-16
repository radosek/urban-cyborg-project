//! Persistent control/volume configuration (ctrl.dat, 12 little-endian i32s — same layout as DOS).

use crate::keys::*;
use std::cell::RefCell;

pub const QUALITY_NORMAL: i32 = 1;
pub const QUALITY_HIGH: i32 = 2;

pub const CONTROLLER_NONE: i32 = 0x00;
pub const CONTROLLER_KEYBOARD: i32 = 0x02;
pub const CONTROLLER_JOYSTICK: i32 = 0x03;
pub const CONTROLLER_GAMEPAD2: i32 = 0x06;
pub const CONTROLLER_GAMEPAD4: i32 = 0x07;
pub const CONTROLLER_GAMEPAD6: i32 = 0x08;
pub const CONTROLLER_GAMEPAD8: i32 = 0x09;

#[derive(Clone, Copy, Debug)]
pub struct KeyConf {
    pub key_up: i32,
    pub key_down: i32,
    pub key_left: i32,
    pub key_right: i32,
    pub key_jump: i32,
    pub key_fire: i32,
    pub key_prevweapon: i32,
    pub key_nextweapon: i32,
    pub ctrl_type: i32,
    pub music_vol: i32,
    pub sfx_vol: i32,
    pub gfx_quality: i32,
}

impl Default for KeyConf {
    fn default() -> Self {
        KeyConf {
            key_up: KEY_UP as i32,
            key_down: KEY_DOWN as i32,
            key_left: KEY_LEFT as i32,
            key_right: KEY_RIGHT as i32,
            key_jump: KEY_SPACE as i32,
            key_fire: KEY_LCONTROL as i32,
            key_prevweapon: KEY_LSHIFT as i32,
            key_nextweapon: KEY_RSHIFT as i32,
            ctrl_type: CONTROLLER_KEYBOARD,
            music_vol: 90,
            sfx_vol: 128,
            gfx_quality: QUALITY_NORMAL,
        }
    }
}

impl KeyConf {
    fn to_bytes(self) -> Vec<u8> {
        [
            self.key_up, self.key_down, self.key_left, self.key_right, self.key_jump, self.key_fire,
            self.key_prevweapon, self.key_nextweapon, self.ctrl_type, self.music_vol, self.sfx_vol, self.gfx_quality,
        ]
        .iter()
        .flat_map(|v| v.to_le_bytes())
        .collect()
    }

    fn from_bytes(b: &[u8]) -> Option<KeyConf> {
        if b.len() < 48 {
            return None;
        }
        let v: Vec<i32> = b[..48].chunks(4).map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
        let valid_key = |k: i32| (0..128).contains(&k);
        let c = KeyConf {
            key_up: v[0], key_down: v[1], key_left: v[2], key_right: v[3], key_jump: v[4], key_fire: v[5],
            key_prevweapon: v[6], key_nextweapon: v[7], ctrl_type: v[8], music_vol: v[9].clamp(0, 255),
            sfx_vol: v[10].clamp(0, 255), gfx_quality: v[11],
        };
        if ![c.key_up, c.key_down, c.key_left, c.key_right, c.key_jump, c.key_fire, c.key_prevweapon, c.key_nextweapon]
            .iter()
            .all(|&k| valid_key(k))
        {
            return None;
        }
        Some(c)
    }
}

thread_local! {
    static CONF: RefCell<KeyConf> = RefCell::new(KeyConf::default());
}

fn path() -> std::path::PathBuf {
    crate::sys::user_dir().join("ctrl.dat")
}

pub fn load() {
    if let Some(c) = std::fs::read(path()).ok().and_then(|b| KeyConf::from_bytes(&b)) {
        CONF.with(|k| *k.borrow_mut() = c);
    }
}

pub fn save() {
    let _ = std::fs::write(path(), get().to_bytes());
}

pub fn get() -> KeyConf {
    CONF.with(|k| *k.borrow())
}

pub fn set(c: KeyConf) {
    CONF.with(|k| *k.borrow_mut() = c);
}
