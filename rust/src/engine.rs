//! Game world: object model, object lists, tile queries, collisions and the per-frame loop
//! (port of engine/engine.cc).
//!
//! Objects live in `Slot`s. While an object updates (or two objects collide) it is taken out of
//! its slot, so it can receive `&mut Game` and still reach every other object, including the
//! player, without aliasing. Iteration order and removal semantics match the original arrays.

use crate::font::{UrbanFont, SMALL_FONT, SMALL_FONT2};
use crate::gfx::{self, Bitmap, Palette, Rgb, BLACK_PALETTE};
use crate::icache::{self, Img};
use crate::keys::*;
use crate::map::{Map, BG_TILE_HEIGHT, BG_TILE_WIDTH};
use crate::objects::player::Player;
use crate::score::ScoreBoard;
use crate::{flags, rng, sound, sys};
use std::any::Any;
use std::sync::atomic::{AtomicU32, Ordering};

pub const SCREEN_WIDTH: i32 = 320;
pub const SCREEN_HEIGHT: i32 = 240;
pub const TILE_WIDTH: i32 = 26;
pub const TILE_HEIGHT: i32 = 40;
pub const TILE_SIDE_HEIGHT: i32 = 24;
pub const TILE_TOP_HEIGHT: i32 = TILE_HEIGHT - TILE_SIDE_HEIGHT;
pub const MAX_Z: i32 = TILE_TOP_HEIGHT >> 1;
pub const MIN_Z: i32 = 3 * TILE_TOP_HEIGHT - 5;
pub const SCORE_BOARD_HEIGHT: i32 = 61 - 2 * TILE_SIDE_HEIGHT;
pub const FRAMESPERSEC: i32 = 60;
pub const PLAYER_N_WEAPONS: usize = 7;

pub const MAX_OBJECTS: usize = 1000;
pub const MAX_EFFECTS_OBJECTS: usize = 400;
pub const MAX_DEKORATION_OBJECTS: usize = 400;

const LIGHTENING_MAX: i32 = 58;
const LIGHTENING_STEP: i32 = 5;
const SHAKE_DELAY: i32 = 50;
const MSG_FRAME_DELAY: i32 = 120;

pub const fn seconds_to_frames(x: i32) -> i32 {
    x * FRAMESPERSEC
}
pub const fn frames_to_seconds(x: i32) -> i32 {
    x / FRAMESPERSEC
}

// Directions
pub const RIGHT_DIR: i32 = 0;
pub const LEFT_DIR: i32 = 1;
pub const UP_DIR: i32 = 2;
pub const DOWN_DIR: i32 = 3;

pub const EVIL_PLAYER: i32 = 0;
pub const EVIL_ENEMY: i32 = 1;

pub const MODE_NORMAL: i32 = 0x01;
pub const MODE_WATER: i32 = 0x02;

// Weapons
pub const SHOTGUN: usize = 0;
pub const FLAME_THROWER: usize = 1;
pub const ICEMAKER: usize = 2;
pub const GRENADE_LAUNCHER: usize = 3;
pub const PLASMA_GUN: usize = 4;
pub const MINIGUN: usize = 5;
pub const ELECTRIC: usize = 6;

// Key cards (enum Cardtype)
pub const CARD_NONE: i32 = 0;
pub const CARD_GREEN: i32 = 1;
pub const CARD_RED: i32 = 2;
pub const CARD_BLUE: i32 = 4;
pub const CARD_YELLOW: i32 = 8;
pub const CARD_EXTRA_LEVEL: i32 = 16;

// Object identity bits: FRIEND_x and ENEMY_x share values.
pub const FRIEND_PLAYER: u32 = 0x01;
pub const FRIEND_SCIENTIST: u32 = 0x02;
pub const FRIEND_SOLDIER: u32 = 0x04;
pub const FRIEND_GRENADE: u32 = 0x08;
pub const FRIEND_HS_BULLET: u32 = 0x10;
pub const FRIEND_MINE: u32 = 0x20;
pub const FRIEND_POWERUP: u32 = 0x40;
pub const FRIEND_CARD: u32 = 0x80;
pub const FRIEND_SOLDIER2: u32 = 0x100;
pub const FRIEND_EXPLOSION: u32 = 0x200;
pub const FRIEND_FIREBALL: u32 = 0x400;
pub const FRIEND_VAKTM: u32 = 0x800;
pub const FRIEND_ICEBALL: u32 = 0x1000;
pub const FRIEND_FROZENBODY: u32 = 0x2000;
pub const FRIEND_RAMBO: u32 = 0x4000;
pub const FRIEND_CHECKPOINT: u32 = 0x8000;
pub const FRIEND_DOOR: u32 = 0x10000;
pub const FRIEND_AGENT: u32 = 0x20000;
pub const FRIEND_DEKOR: u32 = 0x40000;
pub const FRIEND_ELEVATOR: u32 = 0x80000;
pub const FRIEND_ELEVSTAT: u32 = 0x100000;
pub const FRIEND_ELEVWIRE: u32 = 0x200000;
pub const FRIEND_WATER: u32 = 0x400000;
pub const FRIEND_BUBBLE: u32 = 0x800000;
pub const FRIEND_BEAM: u32 = 0x1000000;
pub const FRIEND_TANK: u32 = 0x2000000;
pub const FRIEND_PLASMA: u32 = 0x4000000;

pub const ENEMY_PLAYER: u32 = FRIEND_PLAYER;
pub const ENEMY_SCIENTIST: u32 = FRIEND_SCIENTIST;
pub const ENEMY_SOLDIER: u32 = FRIEND_SOLDIER;
pub const ENEMY_GRENADE: u32 = FRIEND_GRENADE;
pub const ENEMY_HS_BULLET: u32 = FRIEND_HS_BULLET;
pub const ENEMY_MINE: u32 = FRIEND_MINE;
pub const ENEMY_POWERUP: u32 = FRIEND_POWERUP;
pub const ENEMY_CARD: u32 = FRIEND_CARD;
pub const ENEMY_SOLDIER2: u32 = FRIEND_SOLDIER2;
pub const ENEMY_EXPLOSION: u32 = FRIEND_EXPLOSION;
pub const ENEMY_FIREBALL: u32 = FRIEND_FIREBALL;
pub const ENEMY_VAKTM: u32 = FRIEND_VAKTM;
pub const ENEMY_ICEBALL: u32 = FRIEND_ICEBALL;
pub const ENEMY_FROZENBODY: u32 = FRIEND_FROZENBODY;
pub const ENEMY_RAMBO: u32 = FRIEND_RAMBO;
pub const ENEMY_CHECKPOINT: u32 = FRIEND_CHECKPOINT;
pub const ENEMY_DOOR: u32 = FRIEND_DOOR;
pub const ENEMY_AGENT: u32 = FRIEND_AGENT;
pub const ENEMY_DEKOR: u32 = FRIEND_DEKOR;
pub const ENEMY_ELEVATOR: u32 = FRIEND_ELEVATOR;
pub const ENEMY_ELEVSTAT: u32 = FRIEND_ELEVSTAT;
pub const ENEMY_ELEVWIRE: u32 = FRIEND_ELEVWIRE;
pub const ENEMY_WATER: u32 = FRIEND_WATER;
pub const ENEMY_BUBBLE: u32 = FRIEND_BUBBLE;
pub const ENEMY_BEAM: u32 = FRIEND_BEAM;
pub const ENEMY_TANK: u32 = FRIEND_TANK;
pub const ENEMY_PLASMA: u32 = FRIEND_PLASMA;
pub const MAKE_PLAYER_IMMORTAL: u32 = ENEMY_SOLDIER | ENEMY_SOLDIER2 | ENEMY_RAMBO;

/// `state` bit the scoreboard and Object::IsImmortal test.
pub const STATE_IMMORTAL: i32 = 0x80;

// Cheats (include/cheat.h)
pub const CHEAT_MK_PLAYER_IMMORTAL: u32 = 0x0001;
pub const CHEAT_GIVE_PLAYER_HEALTH: u32 = 0x0002;
pub const CHEAT_GIVE_PLAYER_FIRE: u32 = 0x0004;
pub const CHEAT_GIVE_PLAYER_ICE: u32 = 0x0008;
pub const CHEAT_GIVE_PLAYER_GRENADE: u32 = 0x0010;
pub const CHEAT_GIVE_PLAYER_PLASMA: u32 = 0x0020;
pub const CHEAT_GIVE_PLAYER_MINIGUN: u32 = 0x0040;
pub const CHEAT_GIVE_PLAYER_ELECTRIC: u32 = 0x0080;
pub const CHEAT_GIVE_PLAYER_1K_OF_AMMO: u32 = 0x0100;
pub const CHEAT_GIVE_PLAYER_1K_AMMO_2_ALL: u32 = 0x0200;
pub const CHEAT_GIVE_PLAYER_MAX_AMMO: u32 = 0x0400;
pub const CHEAT_GIVE_PLAYER_ALL_WEAPONS: u32 = 0x0800;
pub const CHEAT_GIVE_PLAYER_CARDS: u32 = 0x1000;
pub const CHEAT_CLEAR_LEVEL: u32 = 0x2000;
pub const CHEAT_SET_PLAYER_DEADLY: u32 = 0x4000;
pub const CHEAT_KILL_PLAYER: u32 = 0x8000;
pub const CHEAT_AIRSTRIKE: u32 = 0x10000;

const CHEAT_CODES: &[(&str, u32)] = &[
    ("LINUSISGOD", CHEAT_MK_PLAYER_IMMORTAL),
    ("LINUXRULES", CHEAT_GIVE_PLAYER_HEALTH),
    ("VOLATILE", CHEAT_GIVE_PLAYER_FIRE),
    ("ILIKETOSKATE", CHEAT_GIVE_PLAYER_ICE),
    ("KABLAM", CHEAT_GIVE_PLAYER_GRENADE),
    ("STARTREKFOREVER", CHEAT_GIVE_PLAYER_PLASMA),
    ("LETSROCK", CHEAT_GIVE_PLAYER_MINIGUN),
    ("SITINTHECHAIR", CHEAT_GIVE_PLAYER_ELECTRIC),
    ("FIREAWAY", CHEAT_GIVE_PLAYER_1K_OF_AMMO),
    ("TALKINGTODEATH", CHEAT_GIVE_PLAYER_1K_AMMO_2_ALL),
    ("THENIDIE", CHEAT_GIVE_PLAYER_MAX_AMMO),
    ("NVTRULES", CHEAT_GIVE_PLAYER_ALL_WEAPONS),
    ("LETMEIN", CHEAT_GIVE_PLAYER_CARDS),
    ("THERECANBEONLYONE", CHEAT_CLEAR_LEVEL),
    ("TIMETODIE", CHEAT_SET_PLAYER_DEADLY),
    ("MICROSOFT", CHEAT_KILL_PLAYER),
    ("RETURNOFMRA", CHEAT_AIRSTRIKE),
];

const LETTER_KEYS: [(usize, u8); 26] = [
    (KEY_A, b'A'), (KEY_B, b'B'), (KEY_C, b'C'), (KEY_D, b'D'), (KEY_E, b'E'), (KEY_F, b'F'), (KEY_G, b'G'),
    (KEY_H, b'H'), (KEY_I, b'I'), (KEY_J, b'J'), (KEY_K, b'K'), (KEY_L, b'L'), (KEY_M, b'M'), (KEY_N, b'N'),
    (KEY_O, b'O'), (KEY_P, b'P'), (KEY_Q, b'Q'), (KEY_R, b'R'), (KEY_S, b'S'), (KEY_T, b'T'), (KEY_U, b'U'),
    (KEY_V, b'V'), (KEY_W, b'W'), (KEY_X, b'X'), (KEY_Y, b'Y'), (KEY_Z, b'Z'),
];

// ---------------------------------------------------------------------------
// Object model

pub type ObjId = u32;
static NEXT_ID: AtomicU32 = AtomicU32::new(1);

/// Fields of the C++ `Object` base class. All start at zero like the (usually zeroed) heap memory
/// the original relied on, except those its constructor set.
#[derive(Clone)]
pub struct Base {
    pub id: ObjId,
    pub images: Vec<Img>,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub layer: i32,
    pub speed_x: i32,
    pub speed_y: i32,
    pub speed_z: i32,
    pub current_image: i32,
    pub width: i32,
    pub height: i32,
    pub mode: i32,
    pub coll_x: i32,
    pub coll_y: i32,
    pub coll_width: i32,
    pub coll_height: i32,
    pub direction: i32,
    pub state: i32,
    pub counter: i32,
    pub counter2: i32,
    pub counter3: i32,
    pub counter4: i32,
    pub counter5: i32,
    pub fcounter: f32,
    pub fcounter2: f32,
    pub fcounter3: f32,
    pub grav_counter: i32,
    pub score: i32,
    pub friends: u32,
    pub enemies: u32,
    pub me: u32,
    pub energy: i32,
    pub strength: i32,
}

impl Base {
    pub fn new(x: i32, y: i32, z: i32) -> Base {
        Base::with_mode(x, y, z, MODE_NORMAL)
    }

    pub fn with_mode(x: i32, y: i32, z: i32, mode: i32) -> Base {
        Base {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            images: Vec::new(),
            x,
            y,
            z,
            layer: z / TILE_TOP_HEIGHT,
            speed_x: 0,
            speed_y: 0,
            speed_z: 0,
            current_image: 0,
            width: 0,
            height: 0,
            mode,
            coll_x: 0,
            coll_y: 0,
            coll_width: 0,
            coll_height: 0,
            direction: 0,
            state: 0,
            counter: 0,
            counter2: 0,
            counter3: 0,
            counter4: 0,
            counter5: 0,
            fcounter: 0.0,
            fcounter2: 0.0,
            fcounter3: 0.0,
            grav_counter: 0,
            score: 0,
            friends: 0,
            enemies: 0,
            me: 0,
            energy: 0,
            strength: 0,
        }
    }

    pub fn num_images(&self) -> i32 {
        self.images.len() as i32
    }

    /// Object::GetImage: falls back to image 0 when `current_image` is out of range.
    pub fn image(&self) -> &Img {
        let i = self.current_image;
        if i >= 0 && (i as usize) < self.images.len() {
            &self.images[i as usize]
        } else {
            &self.images[0]
        }
    }

    pub fn collision_x(&self) -> i32 {
        self.x + self.coll_x
    }
    pub fn collision_y(&self) -> i32 {
        self.y + self.coll_y
    }
    pub fn is_immortal(&self) -> bool {
        self.state & STATE_IMMORTAL != 0
    }
}

/// Object::Collision: take damage from anything whose enemy mask includes us.
pub fn default_collision(me: &mut Base, other: &Base) {
    if other.enemies & me.me != 0 {
        me.energy -= other.strength;
    }
    if me.energy < 0 {
        me.energy = 0;
    }
}

/// Result of `Obj::update` (the original's int return codes).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Upd {
    /// 0: keep the object and draw it.
    Keep,
    /// DELETE_ME (-1): remove and drop.
    Delete,
    /// `create_dekoration(this); return REMOVE_ME;`
    ToDekoration,
    /// `create_alwaysupdate(this); return REMOVE_ME;`
    ToAlwaysUpdate,
    /// REMOVE_ME (-2) without re-inserting anywhere.
    Remove,
}

impl Upd {
    pub fn removed(self) -> bool {
        self != Upd::Keep
    }
}

pub trait Obj: Any {
    fn base(&self) -> &Base;
    fn base_mut(&mut self) -> &mut Base;
    fn update(&mut self, _g: &mut Game) -> Upd {
        Upd::Keep
    }
    /// Runs when the object is handed to the engine, i.e. where the C++ constructor had
    /// engine side effects (earthquakes, spawning child objects, map checks).
    fn spawned(&mut self, _g: &mut Game) {}
    /// Called for each overlapping pair. When `other` is the player, `g.player` is unavailable;
    /// use `other` (or `player_of(other, g)`) instead.
    fn collision(&mut self, other: &mut dyn Obj, _g: &mut Game) {
        default_collision(self.base_mut(), other.base());
    }
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

/// Implements the boilerplate `Obj` accessors for a struct with a `base: Base` field.
#[macro_export]
macro_rules! obj_boilerplate {
    () => {
        fn base(&self) -> &$crate::engine::Base {
            &self.base
        }
        fn base_mut(&mut self) -> &mut $crate::engine::Base {
            &mut self.base
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
            self
        }
    };
}

impl dyn Obj {
    pub fn downcast_mut<T: Obj>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut::<T>()
    }
    pub fn downcast_ref<T: Obj>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
    pub fn is_player(&self) -> bool {
        self.as_any().is::<Player>()
    }
}

/// `((player_o *)PLAYER)` inside a collision: the player is either `other` or still in `g`.
pub fn player_of<'a>(other: &'a mut dyn Obj, g: &'a mut Game) -> Option<&'a mut Player> {
    if other.is_player() {
        other.downcast_mut::<Player>()
    } else {
        g.player.as_deref_mut()
    }
}

pub enum Slot {
    Obj(Box<dyn Obj>),
    /// Position of the player in the objects list; the player itself lives in `Game::player`.
    Player,
    Taken,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum List {
    Objects,
    AlwaysUpdate,
    Effects,
    Dekorations,
    Innerlayer,
    MapEffect,
    MapDekor,
}

struct Message {
    msg: Bitmap,
    delay: i32,
}

#[derive(Clone, Copy, Default)]
pub struct PlayerData {
    pub num_lives: i32,
    pub weapon_ammo: [i32; 10],
    pub weapon_avail: [i32; 10],
    pub health: i32,
    pub score: i32,
}

pub struct Game {
    pub map: Map,
    pub score: ScoreBoard,
    pub player: Option<Box<Player>>,
    pub objects: Vec<Slot>,
    pub alwaysupdate: Vec<Slot>,
    pub effects: Vec<Slot>,
    pub dekorations: Vec<Slot>,
    dekor_pos: usize,
    pub innerlayer: Vec<Slot>,
    pub map_effect: Vec<Slot>,
    pub map_dekor: Vec<Slot>,
    pub map_x: i32,
    pub map_y: i32,
    pub do_shake: i32,
    always_shake: bool,
    level_complete: i32,
    lightening: i32,
    max_time: i32,
    pub max_update_area_x: i32,
    pub max_update_area_y: i32,
    pub min_update_area_x: i32,
    pub min_update_area_y: i32,
    pub control_type: i32,
    pub cheat_codes_active: u32,
    cheat_letters: Vec<u8>,
    message: Option<Message>,
    buffer: Bitmap,
    background_available: bool,
    msg_font: UrbanFont,
    fps_n: i32,
    fps_n2: i32,
    fps_t: u64,
}

impl Game {
    pub fn new() -> Game {
        Game {
            map: Map::new(),
            score: ScoreBoard::new(),
            player: None,
            objects: Vec::new(),
            alwaysupdate: Vec::new(),
            effects: Vec::new(),
            dekorations: Vec::new(),
            dekor_pos: 0,
            innerlayer: Vec::new(),
            map_effect: Vec::new(),
            map_dekor: Vec::new(),
            map_x: 0,
            map_y: 0,
            do_shake: 0,
            always_shake: false,
            level_complete: 0,
            lightening: 0,
            max_time: -1,
            max_update_area_x: 500,
            max_update_area_y: 500,
            min_update_area_x: 500,
            min_update_area_y: 500,
            control_type: crate::config::CONTROLLER_KEYBOARD,
            cheat_codes_active: 0,
            cheat_letters: Vec::new(),
            message: None,
            buffer: Bitmap::new(SCREEN_WIDTH, SCREEN_HEIGHT),
            background_available: false,
            msg_font: UrbanFont::new(SMALL_FONT),
            fps_n: 0,
            fps_n2: 0,
            fps_t: 0,
        }
    }

    // --- player access ------------------------------------------------------

    /// PLAYER. Panics if called from the player's own update/collision (use `self` there).
    pub fn player(&mut self) -> &mut Player {
        self.player.as_deref_mut().expect("player is not available here (it is self or the collision partner)")
    }

    pub fn player_ref(&self) -> &Player {
        self.player.as_deref().expect("player is not available here (it is self or the collision partner)")
    }

    pub fn has_player(&self) -> bool {
        self.player.is_some()
    }

    /// PLAYER's base fields (x, y, z, layer, energy, ...).
    pub fn pbase(&self) -> &Base {
        self.player_ref().base()
    }

    // --- object creation ----------------------------------------------------

    pub fn create_object<T: Obj>(&mut self, o: T) {
        self.create_object_box(Box::new(o));
    }

    pub fn create_object_box(&mut self, mut o: Box<dyn Obj>) {
        o.spawned(self);
        if self.objects.len() < MAX_OBJECTS {
            self.objects.push(Slot::Obj(o));
        }
    }

    pub fn create_effect<T: Obj>(&mut self, o: T) {
        self.create_effect_box(Box::new(o));
    }

    pub fn create_effect_box(&mut self, mut o: Box<dyn Obj>) {
        o.spawned(self);
        if self.effects.len() < MAX_EFFECTS_OBJECTS {
            self.effects.push(Slot::Obj(o));
        }
    }

    pub fn create_alwaysupdate<T: Obj>(&mut self, o: T) {
        self.create_alwaysupdate_box(Box::new(o));
    }

    pub fn create_alwaysupdate_box(&mut self, mut o: Box<dyn Obj>) {
        o.spawned(self);
        if self.alwaysupdate.len() < MAX_OBJECTS {
            self.alwaysupdate.push(Slot::Obj(o));
        }
    }

    /// Decorations are a 400-entry ring buffer: once full, the oldest is replaced.
    pub fn create_dekoration<T: Obj>(&mut self, o: T) {
        self.create_dekoration_box(Box::new(o));
    }

    pub fn create_dekoration_box(&mut self, mut o: Box<dyn Obj>) {
        o.spawned(self);
        self.insert_dekoration(o);
    }

    fn insert_dekoration(&mut self, o: Box<dyn Obj>) {
        if self.dekorations.len() < MAX_DEKORATION_OBJECTS {
            self.dekorations.push(Slot::Obj(o));
        } else {
            if self.dekor_pos >= MAX_DEKORATION_OBJECTS {
                self.dekor_pos = 0;
            }
            self.dekorations[self.dekor_pos] = Slot::Obj(o);
            self.dekor_pos += 1;
        }
    }

    // --- lookup of linked objects (elevators, tank wheels, spies) -----------

    pub fn find_mut(&mut self, id: ObjId) -> Option<&mut dyn Obj> {
        if self.player.as_ref().map(|p| p.base().id == id).unwrap_or(false) {
            return self.player.as_deref_mut().map(|p| p as &mut dyn Obj);
        }
        for list in [
            &mut self.objects,
            &mut self.innerlayer,
            &mut self.alwaysupdate,
            &mut self.effects,
            &mut self.dekorations,
            &mut self.map_effect,
            &mut self.map_dekor,
        ] {
            for s in list.iter_mut() {
                if let Slot::Obj(o) = s {
                    if o.base().id == id {
                        return Some(o.as_mut());
                    }
                }
            }
        }
        None
    }

    pub fn find<T: Obj>(&mut self, id: ObjId) -> Option<&mut T> {
        self.find_mut(id).and_then(|o| o.downcast_mut::<T>())
    }

    /// Ids of objects in `list` matching `pred` (used where the original scanned arrays).
    pub fn ids_where(&self, list: List, pred: impl Fn(&Base) -> bool) -> Vec<ObjId> {
        let l = self.list(list);
        let mut out = Vec::new();
        for s in l {
            match s {
                Slot::Obj(o) if pred(o.base()) => out.push(o.base().id),
                Slot::Player => {
                    if let Some(p) = &self.player {
                        if pred(p.base()) {
                            out.push(p.base().id);
                        }
                    }
                }
                _ => {}
            }
        }
        out
    }

    fn list(&self, l: List) -> &Vec<Slot> {
        match l {
            List::Objects => &self.objects,
            List::AlwaysUpdate => &self.alwaysupdate,
            List::Effects => &self.effects,
            List::Dekorations => &self.dekorations,
            List::Innerlayer => &self.innerlayer,
            List::MapEffect => &self.map_effect,
            List::MapDekor => &self.map_dekor,
        }
    }

    fn list_mut(&mut self, l: List) -> &mut Vec<Slot> {
        match l {
            List::Objects => &mut self.objects,
            List::AlwaysUpdate => &mut self.alwaysupdate,
            List::Effects => &mut self.effects,
            List::Dekorations => &mut self.dekorations,
            List::Innerlayer => &mut self.innerlayer,
            List::MapEffect => &mut self.map_effect,
            List::MapDekor => &mut self.map_dekor,
        }
    }

    fn base_at(&self, l: List, i: usize) -> Option<&Base> {
        match &self.list(l)[i] {
            Slot::Obj(o) => Some(o.base()),
            Slot::Player => self.player.as_deref().map(|p| p.base()),
            Slot::Taken => None,
        }
    }

    fn take(&mut self, l: List, i: usize) -> (Box<dyn Obj>, bool) {
        match std::mem::replace(&mut self.list_mut(l)[i], Slot::Taken) {
            Slot::Obj(o) => (o, false),
            Slot::Player => (self.player.take().expect("player slot without player") as Box<dyn Obj>, true),
            Slot::Taken => panic!("slot {:?}[{}] taken twice", l, i),
        }
    }

    fn put(&mut self, l: List, i: usize, o: Box<dyn Obj>, was_player: bool) {
        if was_player {
            self.player = Some(o.into_any().downcast::<Player>().expect("player slot"));
            self.list_mut(l)[i] = Slot::Player;
        } else {
            self.list_mut(l)[i] = Slot::Obj(o);
        }
    }

    // --- engine services used by objects -----------------------------------

    pub fn check_floor(&self, x: i32, y: i32, z: i32) -> bool {
        let bx = x / (TILE_WIDTH - 1);
        let by = y / TILE_SIDE_HEIGHT;
        let bz = (z / (TILE_TOP_HEIGHT - 1)).min(2);
        let cell = self.map.get_bg(bx, by, bz);
        cell != 0 && !self.map.is_fake(cell)
    }

    pub fn check_wall(&self, x: i32, y: i32, z: i32) -> bool {
        let y = y - TILE_TOP_HEIGHT;
        let bx = x / (TILE_WIDTH - 1);
        let by = y / TILE_SIDE_HEIGHT;
        let bz = z / (TILE_TOP_HEIGHT - 1);
        let cell = self.map.get_bg(bx, by, bz);
        cell != 0 && !self.map.is_fake(cell)
    }

    pub fn clear_tile(&mut self, x: i32, y: i32, z: i32) {
        self.set_tile(x, y, z, 0);
    }

    pub fn set_tile(&mut self, x: i32, y: i32, z: i32, t: i32) {
        self.map.set_bg(x / (TILE_WIDTH - 1), y / TILE_SIDE_HEIGHT, z / TILE_TOP_HEIGHT, t);
    }

    pub fn clear_level(&mut self) {
        self.level_complete = 1;
    }

    pub fn play_extra_level(&mut self) {
        self.level_complete = 2;
    }

    pub fn enable_earthquake(&mut self) {
        self.do_shake = SHAKE_DELAY;
        self.enable_lightening();
    }

    pub fn enable_lightening(&mut self) {
        if self.lightening < LIGHTENING_MAX {
            self.lightening += 10;
        }
        if self.lightening > LIGHTENING_MAX {
            self.lightening = LIGHTENING_MAX;
        }
    }

    pub fn set_max_update_x(&mut self, m: i32) {
        self.max_update_area_x = m;
    }

    pub fn push_message(&mut self, msg: &str) {
        self.message = Some(Message { msg: self.msg_font.render(msg), delay: 0 });
    }

    fn update_lightening(&mut self) {
        if self.lightening != 0 {
            if self.lightening % LIGHTENING_STEP == 0 || self.lightening == LIGHTENING_MAX {
                let white = [Rgb { r: 63, g: 63, b: 63 }; 256];
                let mut out = BLACK_PALETTE;
                gfx::fade_interpolate(self.map.pal(), &white, &mut out, self.lightening, 0, 255);
                sys::set_palette(&out);
            }
            self.lightening -= 4;
            if self.lightening <= 0 {
                self.lightening = 0;
                sys::set_palette(self.map.pal());
            }
        }
    }

    fn obj_on_screen(&self, b: &Base) -> bool {
        b.x > self.map_x - 500 && b.x < self.map_x + self.max_update_area_x && b.y > self.map_y - 500 && b.y < self.map_y + 500
    }

    fn obj_off_screen(&self, b: &Base) -> bool {
        b.x < self.map_x - 500 || b.x > self.map_x + self.max_update_area_x || b.y < self.map_y - 500 || b.y > self.map_y + 500
    }

    fn dump_state(&self, l: List, tag: &str) {
        for i in 0..self.list(l).len() {
            if let Some(o) = self.base_at(l, i) {
                eprintln!(
                    "S {} {} me={:x} x={} y={} z={} l={} cx={} cy={} cw={} ch={} e={} str={} sx={} sy={} img={}x{} f={}",
                    tag, i, o.me, o.x, o.y, o.z, o.layer, o.collision_x(), o.collision_y(), o.coll_width, o.coll_height,
                    o.energy, o.strength, o.speed_x, o.speed_y,
                    if o.images.is_empty() { 0 } else { o.image().w },
                    if o.images.is_empty() { 0 } else { o.image().h },
                    o.current_image
                );
            }
        }
    }

    // --- collisions ----------------------------------------------------------

    fn overlap(a: &Base, b: &Base) -> bool {
        let x_dist = (a.collision_x() + (a.coll_width >> 1) - (b.collision_x() + (b.coll_width >> 1))).abs();
        if x_dist >= (a.coll_width + b.coll_width) >> 1 {
            return false;
        }
        let y_dist = (a.collision_y() + (a.coll_height >> 1) - (b.collision_y() + (b.coll_height >> 1))).abs();
        y_dist < (a.coll_height + b.coll_height) >> 1
    }

    fn collide_pair(&mut self, la: List, i: usize, lb: List, j: usize) {
        if let Some(f) = std::env::var("URBAN_STATE_FRAME").ok().and_then(|v| v.parse::<u64>().ok()) {
            if sys::replay_frame() == f {
                if let (Some(a), Some(b)) = (self.base_at(la, i), self.base_at(lb, j)) {
                    let tag = match la { List::AlwaysUpdate => "always", List::Innerlayer => "inner", _ => "obj" };
                    eprintln!("C {} {} {} {:x} {:x}", tag, i, j, a.me, b.me);
                }
            }
        }
        let (mut a, pa) = self.take(la, i);
        let (mut b, pb) = self.take(lb, j);
        a.collision(b.as_mut(), self);
        b.collision(a.as_mut(), self);
        self.put(lb, j, b, pb);
        self.put(la, i, a, pa);
    }

    /// Player vs. an innerlayer object, when the player is not itself in that list.
    fn collide_player_with(&mut self, l: List, i: usize) {
        let Some(mut p) = self.player.take() else { return };
        let (mut o, was_player) = self.take(l, i);
        p.collision(o.as_mut(), self);
        o.collision(p.as_mut(), self);
        self.put(l, i, o, was_player);
        self.player = Some(p);
    }

    /// Loop bounds are re-read every iteration: collisions can spawn objects (explosions),
    /// and the original's `j < num_objects` checks included them in the same pass.
    fn check_collision(&mut self) {
        if self.player.as_ref().map(|p| p.base().layer == 0).unwrap_or(false) {
            let mut i = 0;
            while i < self.innerlayer.len() {
                let hit = match (self.player.as_deref(), self.base_at(List::Innerlayer, i)) {
                    (Some(p), Some(b)) => Self::overlap(p.base(), b),
                    _ => false,
                };
                if hit {
                    self.collide_player_with(List::Innerlayer, i);
                }
                i += 1;
            }
        }
        let mut i = 0;
        while i < self.innerlayer.len() {
            let mut j = i + 1;
            while j < self.innerlayer.len() {
                let hit = match (self.base_at(List::Innerlayer, i), self.base_at(List::Innerlayer, j)) {
                    (Some(a), Some(b)) => Self::overlap(b, a),
                    _ => false,
                };
                if hit {
                    self.collide_pair(List::Innerlayer, j, List::Innerlayer, i);
                }
                j += 1;
            }
            i += 1;
        }
        let mut i = 0;
        while i < self.objects.len() {
            let skip = match self.base_at(List::Objects, i) {
                Some(a) => self.obj_off_screen(a),
                None => true,
            };
            if !skip {
                let mut j = i + 1;
                while j < self.objects.len() {
                    if let (Some(a), Some(b)) = (self.base_at(List::Objects, i), self.base_at(List::Objects, j)) {
                        if a.layer == b.layer && !self.obj_off_screen(b) && Self::overlap(a, b) {
                            self.collide_pair(List::Objects, i, List::Objects, j);
                        }
                    }
                    j += 1;
                }
            }
            i += 1;
        }
        let mut i = 0;
        while i < self.alwaysupdate.len() {
            let mut j = 0;
            while j < self.objects.len() {
                if let (Some(a), Some(b)) = (self.base_at(List::AlwaysUpdate, i), self.base_at(List::Objects, j)) {
                    if a.layer == b.layer && Self::overlap(a, b) {
                        self.collide_pair(List::AlwaysUpdate, i, List::Objects, j);
                    }
                }
                j += 1;
            }
            i += 1;
        }
    }

    // --- frame ---------------------------------------------------------------

    fn draw_obj(buffer: &mut Bitmap, b: &Base, map_x: i32, map_y: i32, k: i32) {
        if b.images.is_empty() {
            return;
        }
        let img = b.image();
        gfx::masked_blit(img, buffer, 0, 0, b.x - map_x, b.y - map_y + b.z - k + SCORE_BOARD_HEIGHT, img.w, img.h);
    }

    /// Updates and draws one list for layer `k`. `scored` awards points for removed objects.
    /// Returns false if the player died (its update asked for removal).
    fn run_list(&mut self, l: List, k: i32, update: bool, scored: bool, on_screen_only: bool, draw_player: &mut bool) -> bool {
        let mut i = 0;
        while i < self.list(l).len() {
            let Some(b) = self.base_at(l, i) else {
                i += 1;
                continue;
            };
            if b.layer != k || (on_screen_only && !self.obj_on_screen(b)) {
                i += 1;
                continue;
            }
            if !update {
                let b = b.clone();
                Self::draw_obj(&mut self.buffer, &b, self.map_x, self.map_y, k);
                i += 1;
                continue;
            }
            let (mut o, was_player) = self.take(l, i);
            let r = o.update(self);
            if r.removed() {
                if scored {
                    let s = o.base().score;
                    self.score.add_score(s);
                    if s != 0 {
                        let b = o.base();
                        self.create_effect(crate::objects::points::Points::new(b.x, b.y, b.z, s));
                    }
                }
                if was_player {
                    self.put(l, i, o, true);
                    return false;
                }
                self.list_mut(l).remove(i);
                match r {
                    Upd::ToDekoration => self.insert_dekoration(o),
                    Upd::ToAlwaysUpdate => {
                        if self.alwaysupdate.len() < MAX_OBJECTS {
                            self.alwaysupdate.push(Slot::Obj(o));
                        }
                    }
                    _ => {}
                }
            } else {
                if was_player {
                    *draw_player = true;
                } else {
                    Self::draw_obj(&mut self.buffer, o.base(), self.map_x, self.map_y, k);
                }
                self.put(l, i, o, was_player);
                i += 1;
            }
        }
        true
    }

    fn display_map(&mut self) -> bool {
        let startx = self.map_x / (TILE_WIDTH - 1);
        let starty = self.map_y / TILE_SIDE_HEIGHT;
        let offsetx = startx * TILE_WIDTH - self.map_x;
        let offsety = starty * TILE_SIDE_HEIGHT - self.map_y;
        let bgstartx = self.map_x / BG_TILE_WIDTH;
        let bgstarty = self.map_y / BG_TILE_HEIGHT - 1;
        let bgoffsetx = bgstartx * BG_TILE_WIDTH - self.map_x;
        let bgoffsety = bgstarty * BG_TILE_HEIGHT - self.map_y;

        self.map.update_pal();
        self.update_lightening();
        sys::pump();

        if !self.background_available || !flags::background_on() {
            self.buffer.clear_to_color(1);
        }
        if let Some(f) = std::env::var("URBAN_STATE_FRAME").ok().and_then(|v| v.parse::<u64>().ok()) {
            if sys::replay_frame() == f {
                self.dump_state(List::Objects, "obj");
                self.dump_state(List::Innerlayer, "inner");
                self.dump_state(List::Effects, "eff");
                self.dump_state(List::AlwaysUpdate, "always");
                self.dump_state(List::Dekorations, "dekor");
            }
        }
        self.check_collision();
        if self.do_shake != 0 && !self.always_shake {
            self.do_shake -= 1;
        }
        self.fps_n2 += 1;

        if flags::background_on() {
            for i in bgstartx..bgstartx + 4 {
                for j in (bgstarty..=bgstarty + 3).rev() {
                    if j < 0 {
                        continue;
                    }
                    let t = self.map.get_background(i, j);
                    if t != 0 {
                        let tile = self.map.bg_tile(t - 1).clone();
                        gfx::blit(
                            &tile,
                            &mut self.buffer,
                            0,
                            0,
                            (i - bgstartx) * BG_TILE_WIDTH + bgoffsetx,
                            (j - bgstarty) * BG_TILE_HEIGHT + bgoffsety + SCORE_BOARD_HEIGHT,
                            BG_TILE_WIDTH,
                            BG_TILE_HEIGHT,
                        );
                        self.background_available = true;
                    }
                }
            }
        }

        for k in 0..3 {
            let ki = TILE_TOP_HEIGHT * k - k;
            let mut draw_player = false;
            for i in startx..startx + 14 {
                for j in (starty..=starty + 10).rev() {
                    let t = self.map.get_bg(i, j, k);
                    if t != 0 {
                        let tile = self.map.tile(t - 1).clone();
                        let dx = (i - startx) * TILE_WIDTH + offsetx - i;
                        let dy = (j - starty) * TILE_SIDE_HEIGHT + offsety + ki + SCORE_BOARD_HEIGHT;
                        if self.map.is_transparent(t) {
                            gfx::masked_blit(&tile, &mut self.buffer, 0, 0, dx, dy, TILE_WIDTH, TILE_HEIGHT);
                        } else {
                            gfx::blit(&tile, &mut self.buffer, 0, 0, dx, dy, TILE_WIDTH, TILE_HEIGHT);
                        }
                    }
                }
            }
            self.run_list(List::Innerlayer, k, true, false, false, &mut draw_player);
            self.run_list(List::MapDekor, k, false, false, false, &mut draw_player);
            self.run_list(List::MapEffect, k, true, false, false, &mut draw_player);
            self.run_list(List::Dekorations, k, false, false, false, &mut draw_player);
            self.run_list(List::Effects, k, true, false, false, &mut draw_player);
            self.run_list(List::AlwaysUpdate, k, true, true, false, &mut draw_player);
            if !self.run_list(List::Objects, k, true, true, true, &mut draw_player) {
                return false;
            }
            if draw_player {
                if let Some(p) = &self.player {
                    let b = p.base().clone();
                    Self::draw_obj(&mut self.buffer, &b, self.map_x, self.map_y, k);
                }
            }
        }

        if let Some(p) = self.player.as_deref() {
            self.score.display(&mut self.buffer, p);
        }

        if sys::key(KEY_P) {
            self.pause();
        }

        let now = sys::ticks_ms();
        if now - self.fps_t >= 1000 {
            self.fps_n = self.fps_n2;
            self.fps_n2 = 0;
            self.fps_t = now;
        }
        if sys::key(KEY_F11) {
            crate::text::textout(&mut self.buffer, &format!("Frames/s {}", self.fps_n), 0, 0, 36, 0);
        }

        if let Some(m) = &mut self.message {
            gfx::masked_blit(&m.msg, &mut self.buffer, 0, 0, (SCREEN_WIDTH - m.msg.w) / 2, 60, m.msg.w, m.msg.h);
            m.delay += 1;
            if m.delay > MSG_FRAME_DELAY {
                self.message = None;
            }
        }

        if self.max_time != -1 {
            self.max_time -= 1;
            let s = format!("Time left: {}", frames_to_seconds(self.max_time));
            self.msg_font.print(&s, 10, 60, &mut self.buffer);
        }
        if self.max_time == 0 {
            if let Some(mut p) = self.player.take() {
                p.kill_player(self);
                self.player = Some(p);
            }
            self.max_time = seconds_to_frames(self.map.option("TIME").and_then(|t| t.trim().parse().ok()).unwrap_or(0));
        }

        sys::wait_frame(FRAMESPERSEC as u32);
        if self.do_shake != 0 {
            let dx = rng::random() % 3 - 1;
            let dy = rng::random() % 3 - 1;
            if std::env::var_os("URBAN_SHAKE_TRACE").is_some() {
                eprintln!("SHAKE {} {} {}", sys::replay_frame(), dx, dy);
            }
            sys::show(&self.buffer, dx, dy);
        } else {
            sys::show(&self.buffer, 0, 0);
        }

        self.handle_cheats();
        true
    }

    fn pause(&mut self) {
        while sys::keypressed() {
            sys::readkey();
        }
        let mut buf = sys::screen_copy();
        let mut x = 0;
        while x < 320 {
            let mut y = 0;
            while y < 240 {
                buf.putpixel(x, y, 0);
                buf.putpixel(x + 1, y + 1, 0);
                y += 2;
            }
            x += 2;
        }
        self.msg_font.print_centre("PAUSE", SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, &mut buf);
        sys::show(&buf, 0, 0);
        while !sys::keypressed() {
            sys::rest(10);
        }
        sys::clear_keybuf();
        // The game clock must not try to catch up on the paused time.
        sys::wait_frame(FRAMESPERSEC as u32);
    }

    fn handle_cheats(&mut self) {
        self.cheat_codes_active = 0;
        for &(k, letter) in LETTER_KEYS.iter() {
            if sys::key(k) && self.cheat_letters.len() < 63 {
                if self.cheat_letters.last() == Some(&letter) {
                    break;
                }
                self.cheat_letters.push(letter);
                let typed = std::str::from_utf8(&self.cheat_letters).unwrap_or("").to_string();
                let mut keep = false;
                for &(code, cheat) in CHEAT_CODES {
                    if code.starts_with(&typed) {
                        if code == typed {
                            self.cheat_codes_active |= cheat;
                            self.push_message("Cheat activated");
                            keep = false;
                            break;
                        }
                        keep = true;
                    }
                }
                if !keep {
                    self.cheat_letters.clear();
                }
                break;
            }
        }
        if self.cheat_codes_active & CHEAT_CLEAR_LEVEL != 0 {
            self.clear_level();
        }
        if self.cheat_codes_active & CHEAT_AIRSTRIKE != 0 {
            let (x, y, z) = {
                let b = self.pbase();
                (b.x, b.y, b.z)
            };
            if y - 200 > 50 {
                self.create_alwaysupdate(crate::objects::airstrke::Airstrike::new(x, y - 200, z));
            } else {
                self.push_message("Unable to issue airstrike");
            }
        }
    }

    fn scroll(&mut self) {
        let Some(p) = self.player.as_deref() else { return };
        let (px, py) = (p.base().x, p.base().y);
        self.map_x = px - 100;
        if py - 65 < self.map_y {
            self.map_y = py - 65;
        } else if py - 110 + SCORE_BOARD_HEIGHT > self.map_y {
            self.map_y = py - 110 + SCORE_BOARD_HEIGHT;
        }
        let max_y = self.map.height() * TILE_SIDE_HEIGHT - SCREEN_HEIGHT + 25;
        if self.map_y > max_y {
            self.map_y = max_y;
        }
        if self.map_y < 0 {
            self.map_y = 0;
        }
        let max_x = self.map.width() * (TILE_WIDTH - 1) - 330;
        if self.map_x > max_x {
            self.map_x = max_x;
        }
        if self.map_x < 0 {
            self.map_x = 0;
        }
    }

    /// Returns -1 when the player is dead for good, 0 on ESC, 1 level cleared, 2 extra level.
    fn play_loop(&mut self) -> i32 {
        self.map_y = 0;
        sys::set_palette(&BLACK_PALETTE);
        let mut first_frame = true;
        loop {
            self.scroll();
            if !self.display_map() {
                return -1;
            }
            if first_frame {
                for i in 0..65 {
                    let mut out = BLACK_PALETTE;
                    gfx::fade_interpolate(&BLACK_PALETTE, self.map.pal(), &mut out, i, 0, 255);
                    self.scroll();
                    sys::set_palette(&out);
                    if !self.display_map() {
                        return -1;
                    }
                }
                first_frame = false;
            }
            if sys::key(KEY_ESC) {
                return 0;
            }
            if self.level_complete != 0 {
                return self.level_complete;
            }
        }
    }

    /// Clears per-level state. Keeps the player when `keep_player`.
    fn reset_lists(&mut self, keep_player: bool) {
        self.objects.clear();
        if keep_player && self.player.is_some() {
            self.objects.push(Slot::Player);
        } else {
            self.player = None;
        }
        self.alwaysupdate.clear();
        self.effects.clear();
        self.innerlayer.clear();
        self.map_dekor.clear();
        self.map_effect.clear();
        self.dekorations.clear();
        self.dekor_pos = 0;
        self.do_shake = 0;
        self.map_x = 0;
        self.map_y = 0;
        self.level_complete = 0;
        self.lightening = 0;
        self.max_update_area_x = 500;
        self.message = None;
        self.background_available = false;
    }

    /// Engine::NewGame: forget the player and score.
    pub fn new_game(&mut self) {
        self.reset_lists(false);
        self.score.reset_score();
    }

    pub fn play_level(&mut self, map_name: &str, p_dat: Option<&mut PlayerData>, controls: i32) -> i32 {
        if !sys::replaying() || std::env::var_os("URBAN_SEED").is_none() {
            rng::seed(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1));
        }
        self.control_type = controls;
        if let Err(e) = self.map.load(map_name) {
            eprintln!("Can't open map '{}': {}", map_name, e);
            std::process::exit(1);
        }
        self.map.pal_mut()[0] = Rgb { r: 0, g: 0, b: 0 };
        self.reset_lists(true);
        self.create_objects();

        if let (Some(pd), Some(p)) = (p_dat.as_deref(), self.player.as_deref_mut()) {
            self.score.set_score(pd.score);
            p.base_mut().energy = pd.health;
            p.set_life(pd.num_lives);
            for i in 0..PLAYER_N_WEAPONS {
                p.set_ammo(i, pd.weapon_ammo[i]);
                if pd.weapon_avail[i] != 0 {
                    p.enable_weapon(i);
                } else {
                    p.disable_weapon(i);
                }
            }
        }
        sys::clear_keybuf();
        self.cheat_letters.clear();
        let ret = self.play_loop();
        sys::clear_keybuf();
        if let (Some(pd), Some(p)) = (p_dat, self.player.as_deref()) {
            pd.health = p.base().energy;
            pd.num_lives = p.lives();
            for i in 0..PLAYER_N_WEAPONS {
                pd.weapon_ammo[i] = p.ammo_of(i);
                pd.weapon_avail[i] = p.have_weapon(i) as i32;
            }
            pd.score = self.score.get_score();
        }
        ret
    }

    fn create_objects(&mut self) {
        let (ibild, pal) = icache::get_pal("ibild.pcx");
        let fnt = UrbanFont::new(SMALL_FONT2);
        let episode = self.map.option("EPISODE_NAME").map(str::to_string);
        let level = self.map.option("LEVEL_NAME").map(str::to_string);
        sys::set_palette(&pal);
        sys::blit_to_screen(&ibild, 0, 0, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);
        let draw_titles = |fnt: &UrbanFont| {
            for (text, row) in [(&level, 1), (&episode, 2)] {
                if let Some(t) = text {
                    let b = fnt.render(t);
                    sys::masked_blit_to_screen(&b, 0, 0, 160 - b.w / 2, SCREEN_HEIGHT / 2 - row * b.h, b.w, b.h);
                }
            }
        };

        let (w, h) = (self.map.width(), self.map.height());
        for k in 0..3 {
            for i in 0..w {
                let procent = 100 * (k * w + i) / (3 * w);
                let b = fnt.render(&format!("Loading {}%", procent));
                sys::blit_to_screen(&ibild, 160 - b.w / 2, SCREEN_HEIGHT - b.h, 160 - b.w / 2, SCREEN_HEIGHT - b.h, b.w, b.h);
                sys::masked_blit_to_screen(&b, 0, 0, 160 - b.w / 2, SCREEN_HEIGHT - b.h, b.w, b.h);
                draw_titles(&fnt);
                for j in 0..h {
                    let t = self.map.get_obj(i, j, k);
                    if t == 0 {
                        continue;
                    }
                    let x = i * (TILE_WIDTH - 1);
                    let y = j * TILE_SIDE_HEIGHT;
                    match crate::objects::spawn(t, x, y, k, self.control_type) {
                        Some(crate::objects::Spawn::Player(p)) => {
                            if let Some(existing) = self.player.as_deref_mut() {
                                // MoveTo takes the raw tile position (it applies the -66 itself).
                                existing.move_to(x, y, k * TILE_TOP_HEIGHT + (TILE_TOP_HEIGHT >> 1));
                            } else {
                                self.player = Some(p);
                                self.objects.push(Slot::Player);
                            }
                        }
                        Some(crate::objects::Spawn::Object(mut o)) => {
                            o.spawned(self);
                            self.objects.push(Slot::Obj(o));
                        }
                        Some(crate::objects::Spawn::Inner(mut o)) => {
                            o.spawned(self);
                            // No cap: level 6-3 has 114 entries; the original overran its
                            // 100-entry array here.
                            self.innerlayer.push(Slot::Obj(o));
                        }
                        Some(crate::objects::Spawn::Always(mut o)) => {
                            o.spawned(self);
                            self.alwaysupdate.push(Slot::Obj(o));
                        }
                        None => {}
                    }
                }
            }
        }
        // Elevators go to the end of the innerlayer list, using the original's swap loop
        // (not a stable partition: the resulting order decides update order).
        let is_elev = |s: &Slot| matches!(s, Slot::Obj(o) if o.base().me == FRIEND_ELEVATOR);
        let n = self.innerlayer.len();
        if n > 0 {
            let mut j = n as i64 - 1;
            let mut i = 0i64;
            while i < j {
                if is_elev(&self.innerlayer[i as usize]) {
                    while j > i {
                        if !is_elev(&self.innerlayer[j as usize]) {
                            self.innerlayer.swap(i as usize, j as usize);
                            j -= 1;
                            break;
                        }
                        j -= 1;
                    }
                }
                i += 1;
            }
        }

        let vol = sound::music_volume();
        for v in (1..=vol).rev() {
            sound::set_music_volume(v);
            sys::rest(10);
        }
        self.always_shake = self.map.option("ALWAYS_SHAKE") == Some("ON");
        self.max_time = match self.map.option("TIME") {
            Some(t) => match seconds_to_frames(t.trim().parse().unwrap_or(0)) {
                0 => -1,
                n => n,
            },
            None => -1,
        };
        if self.always_shake {
            self.do_shake = 1;
        }
        let music = match self.map.option("MUSIC") {
            Some(m) => m.to_string(),
            None => {
                const SONGS: [&str; 8] = ["blazer", "looser", "madness2", "fett", "basesolo", "nole", "demise", "slayers"];
                format!("snd/modules/{}.xm", SONGS[(rng::random() % 8) as usize])
            }
        };
        if flags::sound_on() {
            sound::play_music(&music);
        }
        for v in 0..vol {
            sound::set_music_volume(v);
            sys::rest(10);
        }
        sound::set_music_volume(vol);
        sys::blit_to_screen(&ibild, 0, 0, 0, 0, ibild.w, ibild.h);
        draw_titles(&fnt);
        if level.is_some() || episode.is_some() {
            for _ in 0..100 {
                sys::rest(50);
                if sys::keypressed() {
                    break;
                }
            }
        }
    }
}

/// Palette helper for objects that fade (player death).
pub fn fade_in_map(g: &Game, speed: i32) {
    let p: Palette = *g.map.pal();
    sys::fade_in(&p, speed);
}

// Powerup kinds (object.h)
pub const POWERUP_ENERGY: i32 = 1;
pub const POWERUP_AMMO_SHOTGUN: i32 = 2;
pub const POWERUP_AMMO_ICEMAKER: i32 = 3;
pub const POWERUP_AMMO_FLAME_THROWER: i32 = 4;
pub const POWERUP_AMMO_GRENADE_LAUNCHER: i32 = 5;
pub const POWERUP_AMMO_GRENADE_LAUNCHER6: i32 = 6;
pub const POWERUP_WEAPON_FLAME_THROWER: i32 = 7;
pub const POWERUP_WEAPON_ICEMAKER: i32 = 8;
pub const POWERUP_WEAPON_GRENADE_LAUNCHER: i32 = 9;
pub const POWERUP_EXTRA_LIFE: i32 = 10;
pub const POWERUP_AMMO_PLASMA: i32 = 11;
pub const POWERUP_WEAPON_PLASMA: i32 = 12;
pub const POWERUP_AMMO_MINIGUN: i32 = 13;
pub const POWERUP_WEAPON_MINIGUN: i32 = 14;
pub const POWERUP_AMMO_ELECTRIC: i32 = 15;
pub const POWERUP_WEAPON_ELECTRIC: i32 = 16;
