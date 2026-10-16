//! The player (objects/player.cc).

use crate::ctrls::Controls;
use crate::engine::*;
use crate::icache;
use crate::objects::anim::Anim;
use crate::objects::bdyparts::Bodyparts;
use crate::objects::beam::Beam;
use crate::objects::blood::Blood;
use crate::objects::bubble::Bubble;
use crate::objects::burning::BurningBody;
use crate::objects::elevstat::ElevatorStation;
use crate::objects::fireball::FireBall;
use crate::objects::frosen::FrosenBody;
use crate::objects::hs_bulle::HighSpeedBullet;
use crate::objects::iceball::IceBall;
use crate::objects::plasma::Plasma;
use crate::objects::rocket::Rocket;
use crate::rng::random;
use crate::sound;

const FIRE_SAMPLE: &str = "samples/flame.wav";
const SPLAT_SOUND: &str = "samples/gutsplat.wav";

const FALL_DAMAGE: i32 = 20;
const FALL_DAMAGE_BEGIN: i32 = 30;

const X_FRICTION: i32 = 1;
const Z_FRICTION: i32 = X_FRICTION;
const Y_ACCEL: i32 = 1;

const DEATH_EXPLOSION: i32 = 0x01;
const DEATH_ICEBALL: i32 = 0x02;
const DEATH_FIREBALL: i32 = 0x03;
const DEATH_BULLET: i32 = 0x04;

const STATE_NONE: i32 = 0x00;
const STATE_WALK: i32 = 0x01;
const STATE_FIRE: i32 = 0x04;
const STATE_DEAD: i32 = 0x20;

const IMMORTAL_DELAY: i32 = 200;
const TOUCH_DELAY: i32 = 10;
const EL_LENGTH: i32 = 5;

const NUM_WEAPONS: usize = PLAYER_N_WEAPONS;
const NUM_LIFES: i32 = 3;
const MAX_ENERGY: i32 = 300;
const EMPTY_IMAGE: i32 = 240;

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
struct Weapon {
    walk_left: [i32; 6],
    walk_right: [i32; 6],
    walk_up: [i32; 6],
    walk_down: [i32; 6],
    fire_left: [i32; 6],
    fire_right: [i32; 6],
    fire_rep: i32,
    fire_delay: i32,
    /// Negative ammo is infinite (the shotgun starts at -1).
    ammo: i32,
    availible: bool,
    waterproof: bool,
    max_ammo: i32,
}

const fn r6(a: i32) -> [i32; 6] {
    [a, a + 1, a + 2, a + 3, a + 4, a + 5]
}

#[allow(clippy::too_many_arguments)]
const fn w(
    wl: i32, wr: i32, wu: i32, wd: i32, fl: i32, fr: i32,
    fire_rep: i32, fire_delay: i32, ammo: i32, availible: bool, waterproof: bool, max_ammo: i32,
) -> Weapon {
    Weapon {
        walk_left: r6(wl),
        walk_right: r6(wr),
        walk_up: r6(wu),
        walk_down: r6(wd),
        fire_left: r6(fl),
        fire_right: r6(fr),
        fire_rep,
        fire_delay,
        ammo,
        availible,
        waterproof,
        max_ammo,
    }
}

const WEAPONS: [Weapon; NUM_WEAPONS] = [
    // Shotgun
    w(6, 0, 18, 12, 30, 24, 3, 10, -1, true, true, -1),
    // Flame thrower
    w(54, 48, 18, 60, 42, 36, 5, 8, 0, false, false, 500),
    // IceMaker(tm)
    w(84, 78, 18, 90, 72, 66, 5, 8, 0, false, false, 500),
    // Grenade launcher
    w(102, 96, 114, 108, 126, 120, 3, 60, 0, false, true, 32),
    // Plasma weapon
    w(138, 132, 150, 144, 162, 156, 3, 5, 0, false, false, 100),
    // Minigun
    w(174, 168, 186, 180, 198, 192, 1, 1, 0, false, true, 1000),
    // Electric
    w(210, 204, 222, 216, 234, 228, 6, 6, 0, false, false, 100),
];

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Player {
    pub base: Base,
    anim: Anim,
    #[serde(skip)]
    ctrl: Controls,
    weapon: [Weapon; NUM_WEAPONS],
    elevator: Option<ObjId>,
    elev_station: Option<ObjId>,
    current_weapon: usize,
    lives: i32,
    save_x: i32,
    save_y: i32,
    save_z: i32,
    control_type: i32,
    card: i32,
    jumping: bool,
    jump_keypressed: bool,
    saved_mode: i32,
    fall_counter: i32,
}

fn load_images() -> Vec<icache::Img> {
    let groups: [(&str, &str); 40] = [
        ("urban/h/", ".pcx"),
        ("urban/v/", ".pcx"),
        ("urban/f/", ".pcx"),
        ("urban/b/", ".pcx"),
        // Firing frames: shotgun
        ("urban/h/", "s.pcx"),
        ("urban/v/", "s.pcx"),
        // Flame thrower
        ("urban/flame_h/", "s.pcx"),
        ("urban/flame_v/", "s.pcx"),
        ("urban/flame_h/", ".pcx"),
        ("urban/flame_v/", ".pcx"),
        ("urban/flame_f/", ".pcx"),
        // IceMaker(tm)
        ("urban/ice/urbhoger/", "s.pcx"),
        ("urban/ice/urbvanst/", "s.pcx"),
        ("urban/ice/urbhoger/", ".pcx"),
        ("urban/ice/urbvanst/", ".pcx"),
        ("urban/ice/urbfram/", ".pcx"),
        // Grenade
        ("urban/granadel/urbhoger/", ".pcx"),
        ("urban/granadel/urbvanst/", ".pcx"),
        ("urban/granadel/urbfram/", ".pcx"),
        ("urban/granadel/urbbak/", ".pcx"),
        ("urban/granadel/urbhoger/", "s.pcx"),
        ("urban/granadel/urbvanst/", "s.pcx"),
        // Plasma weapon
        ("plasmal/urbhoger/", ".pcx"),
        ("plasmal/urbvanst/", ".pcx"),
        ("plasmal/urbfram/", ".pcx"),
        ("plasmal/urbbak/", ".pcx"),
        ("plasmal/urbhoger/", "s.pcx"),
        ("plasmal/urbvanst/", "s.pcx"),
        // Minigun
        ("urban/minigun/urbhoger/", ".pcx"),
        ("urban/minigun/urbvanst/", ".pcx"),
        ("urban/minigun/urbfram/", ".pcx"),
        ("urban/minigun/urbbak/", ".pcx"),
        ("urban/minigun/urbhoger/", "s.pcx"),
        ("urban/minigun/urbvanst/", "s.pcx"),
        // Electric
        ("urban/electric/urbhoger/", ".pcx"),
        ("urban/electric/urbvanst/", ".pcx"),
        ("urban/electric/urbfram/", ".pcx"),
        ("urban/electric/urbbak/", ".pcx"),
        ("urban/electric/urbhoger/", "s.pcx"),
        ("urban/electric/urbvanst/", "s.pcx"),
    ];
    let mut v = Vec::with_capacity(241);
    for (prefix, suffix) in groups {
        v.extend(icache::seq(prefix, 1, 6, suffix));
    }
    v.push(icache::get("urban/empty.pcx"));
    v
}

/// Runs `f` on the object `id` of type T with the object taken out of its list, so `f` can use
/// the whole Game (`g.find` would keep `g` borrowed).
fn with_obj<T: Obj, R>(g: &mut Game, id: ObjId, f: impl FnOnce(&mut T, &mut Game) -> R) -> Option<R> {
    fn list(g: &mut Game, k: usize) -> &mut Vec<Slot> {
        match k {
            0 => &mut g.objects,
            1 => &mut g.innerlayer,
            2 => &mut g.alwaysupdate,
            3 => &mut g.effects,
            4 => &mut g.dekorations,
            5 => &mut g.map_effect,
            _ => &mut g.map_dekor,
        }
    }
    for k in 0..7 {
        let pos = list(g, k).iter().position(|s| matches!(s, Slot::Obj(o) if o.base().id == id));
        if let Some(i) = pos {
            let Slot::Obj(mut o) = std::mem::replace(&mut list(g, k)[i], Slot::Taken) else { unreachable!() };
            let r = o.as_mut().downcast_mut::<T>().map(|t| f(t, g));
            list(g, k)[i] = Slot::Obj(o);
            return r;
        }
    }
    None
}

impl Player {
    pub fn new(x: i32, y: i32, z: i32, controls: i32) -> Player {
        let mut base = Base::new(x, y, z);
        base.images = load_images();
        base.current_image = 5;
        base.speed_y = 1;
        base.direction = RIGHT_DIR;
        base.height = base.images[5].h;
        base.width = base.images[5].w;
        base.coll_x = 31;
        base.coll_y = 0;
        base.coll_width = 20;
        base.coll_height = base.height;
        // Stand on the feet
        base.y -= 66;
        base.energy = MAX_ENERGY;
        base.strength = 0;
        base.friends = FRIEND_PLAYER;
        base.enemies = !FRIEND_PLAYER;
        base.me = FRIEND_PLAYER;
        base.state = STATE_NONE;
        let saved_mode = base.mode;
        let (sx, sy, sz) = (base.x, base.y, base.z);
        Player {
            base,
            anim: Anim::default(),
            ctrl: Controls::new(controls),
            weapon: WEAPONS,
            elevator: None,
            elev_station: None,
            current_weapon: SHOTGUN,
            lives: NUM_LIFES,
            save_x: sx,
            save_y: sy,
            save_z: sz,
            control_type: controls,
            card: 0,
            jumping: false,
            jump_keypressed: false,
            saved_mode,
            fall_counter: 0,
        }
    }

    /// Respawn position (changes when a checkpoint is reached).
    pub fn checkpoint(&self) -> (i32, i32, i32) {
        (self.save_x, self.save_y, self.save_z)
    }

    pub fn move_to(&mut self, x: i32, y: i32, z: i32) {
        let b = &mut self.base;
        b.x = x;
        b.y = y - 66;
        b.z = z;
        self.save_x = b.x;
        self.save_y = b.y;
        self.save_z = b.z;
        self.card = 0;
        b.speed_x = 0;
        b.speed_y = 0;
        b.speed_z = 0;
    }

    pub fn set_ammo(&mut self, t: usize, n: i32) {
        self.weapon[t].ammo = n;
    }

    pub fn add_ammo(&mut self, t: usize, n: i32) {
        let wp = &mut self.weapon[t];
        wp.ammo += n;
        if wp.ammo > wp.max_ammo {
            wp.ammo = wp.max_ammo;
        }
    }

    pub fn enable_weapon(&mut self, t: usize) {
        self.weapon[t].availible = true;
        if t > self.current_weapon {
            self.current_weapon = t;
        }
    }

    pub fn disable_weapon(&mut self, t: usize) {
        self.weapon[t].availible = false;
    }

    pub fn ammo(&self) -> i32 {
        self.weapon[self.current_weapon].ammo
    }

    pub fn ammo_of(&self, t: usize) -> i32 {
        self.weapon[t].ammo
    }

    pub fn weapon(&self) -> usize {
        self.current_weapon
    }

    pub fn have_weapon(&self, t: usize) -> bool {
        self.weapon[t].availible
    }

    pub fn set_life(&mut self, n: i32) {
        self.lives = n;
    }

    pub fn add_life(&mut self) {
        self.lives += 1;
    }

    pub fn remove_life(&mut self) {
        self.lives -= 1;
    }

    pub fn lives(&self) -> i32 {
        self.lives
    }

    pub fn give_card(&mut self, c: i32) {
        self.card |= c;
    }

    pub fn have_card(&self, c: i32) -> bool {
        self.card & c != 0
    }

    pub fn kill_player(&mut self, g: &mut Game) {
        self.perform_death(0, g);
    }

    pub fn new_level(&mut self) -> i32 {
        self.save_x = self.base.x;
        self.save_y = self.base.y;
        self.save_z = self.base.z;
        self.card = 0;
        0
    }

    pub fn control_type(&self) -> i32 {
        self.control_type
    }

    fn normal(&self) -> bool {
        self.base.mode == MODE_NORMAL
    }
    fn max_x_speed(&self) -> i32 {
        if self.normal() { 3 } else { 2 }
    }
    fn max_y_speed(&self) -> i32 {
        if self.normal() { 16 } else { 8 }
    }
    fn min_y_speed(&self) -> i32 {
        if self.normal() { -12 } else { -6 }
    }
    fn max_z_speed(&self) -> i32 {
        if self.normal() { 2 } else { 1 }
    }
    fn x_accel(&self) -> i32 {
        if self.normal() { 2 } else { 1 }
    }
    fn z_accel(&self) -> i32 {
        if self.normal() { 2 } else { 1 }
    }
    fn frame_delay(&self) -> i32 {
        if self.normal() { 4 } else { 8 }
    }

    fn cur(&mut self) -> &mut Weapon {
        &mut self.weapon[self.current_weapon]
    }

    fn weapon_availible(&self) -> bool {
        let wp = &self.weapon[self.current_weapon];
        wp.ammo != 0 && (self.normal() || wp.waterproof)
    }

    fn death_by_explosion(&self, g: &mut Game) {
        let b = &self.base;
        let (y, z) = (b.y, b.z);
        let cx = b.x + b.width / 2;
        let cy = b.y + b.height / 2;
        let r = random() % 3;
        for _ in 0..10 {
            let sx = random() % 8 + 2;
            let sy = -(random() % 4) - 2;
            g.create_effect(Blood::new(cx, y, z, sx, sy, 0));
        }
        for _ in 0..10 {
            let sx = -(random() % 8) - 2;
            let sy = -(random() % 4) - 2;
            g.create_effect(Blood::new(cx, y, z, sx, sy, 0));
        }
        let part = |g: &mut Game, name: &str, n: i32, sx: i32, sy: i32| {
            g.create_effect(Bodyparts::new(cx, cy, z, name, n, sx, sy, 0));
        };
        match r {
            0 => {
                part(g, "meat/arm", 5, -4, -6);
                part(g, "meat/arm", 5, 4, -6);
                part(g, "meat/head", 5, 2, -4);
                part(g, "meat/leg", 5, -1, -2);
                part(g, "meat/leg", 5, 3, -4);
                part(g, "meat/ribs", 5, 0, 0);
                for _ in 0..5 {
                    let sx = random() % 5;
                    let sy = -(random() % 6);
                    part(g, "meat/meat", 3, sx, sy);
                }
                sound::play_sfx(SPLAT_SOUND);
            }
            1 => {
                part(g, "meat/head", 5, 2, -4);
                part(g, "meat/ribs", 5, 0, 0);
                for _ in 0..5 {
                    let sx = random() % 4;
                    let sy = -(random() % 4) - 2;
                    part(g, "meat/meat", 3, sx, sy);
                }
                for _ in 0..5 {
                    let sx = -(random() % 4);
                    let sy = -(random() % 4) - 2;
                    part(g, "meat/meat", 3, sx, sy);
                }
                sound::play_sfx(SPLAT_SOUND);
            }
            _ => {
                for _ in 0..3 {
                    let sx = random() % 5;
                    let sy = -(random() % 4) - 3;
                    part(g, "meat/bigmeat", 5, sx, sy);
                }
                for _ in 0..3 {
                    let sx = -(random() % 5);
                    let sy = -(random() % 4) - 3;
                    part(g, "meat/bigmeat", 5, sx, sy);
                }
                sound::play_sfx(SPLAT_SOUND);
            }
        }
    }

    fn perform_death(&mut self, typ: i32, g: &mut Game) {
        let b = &self.base;
        match typ {
            DEATH_ICEBALL => g.create_object(FrosenBody::new(b.x, b.y + b.height, b.z, b.direction)),
            DEATH_FIREBALL => g.create_effect(BurningBody::new(b.x, b.y + b.height, b.z, 1)),
            _ => self.death_by_explosion(g),
        }
        let b = &mut self.base;
        b.counter = 60;
        self.lives -= 1;
        b.energy = MAX_ENERGY;
        b.state |= STATE_DEAD;
        b.current_image = EMPTY_IMAGE;
    }

    fn handle_cheats(&mut self, g: &mut Game) -> Option<Upd> {
        let c = g.cheat_codes_active;
        if c & CHEAT_GIVE_PLAYER_PLASMA != 0 {
            self.weapon[PLASMA_GUN].availible = true;
        }
        if c & CHEAT_GIVE_PLAYER_FIRE != 0 {
            self.weapon[FLAME_THROWER].availible = true;
        }
        if c & CHEAT_GIVE_PLAYER_ICE != 0 {
            self.weapon[ICEMAKER].availible = true;
        }
        if c & CHEAT_GIVE_PLAYER_GRENADE != 0 {
            self.weapon[GRENADE_LAUNCHER].availible = true;
        }
        if c & CHEAT_GIVE_PLAYER_MINIGUN != 0 {
            self.weapon[MINIGUN].availible = true;
        }
        if c & CHEAT_GIVE_PLAYER_ELECTRIC != 0 {
            self.weapon[ELECTRIC].availible = true;
        }
        if c & CHEAT_MK_PLAYER_IMMORTAL != 0 {
            self.base.state |= STATE_IMMORTAL;
            self.base.counter4 = 5000;
            sound::play_sfx_critical("samples/pathetic.wav");
        }
        if c & CHEAT_GIVE_PLAYER_1K_OF_AMMO != 0 {
            self.cur().ammo += 1000;
        }
        if c & CHEAT_GIVE_PLAYER_CARDS != 0 {
            self.card = CARD_RED | CARD_GREEN | CARD_BLUE;
        }
        const ALL: [usize; 6] = [PLASMA_GUN, FLAME_THROWER, ICEMAKER, GRENADE_LAUNCHER, MINIGUN, ELECTRIC];
        if c & CHEAT_GIVE_PLAYER_1K_AMMO_2_ALL != 0 {
            for t in ALL {
                self.weapon[t].ammo += 1000;
            }
        }
        if c & CHEAT_GIVE_PLAYER_MAX_AMMO != 0 {
            for t in ALL {
                self.weapon[t].ammo = self.weapon[t].max_ammo;
            }
        }
        if c & CHEAT_GIVE_PLAYER_HEALTH != 0 {
            self.base.energy = MAX_ENERGY;
        }
        if c & CHEAT_GIVE_PLAYER_ALL_WEAPONS != 0 {
            for t in ALL {
                self.weapon[t].availible = true;
            }
        }
        if c & CHEAT_SET_PLAYER_DEADLY != 0 {
            self.base.strength = 10;
        }
        if c & CHEAT_KILL_PLAYER != 0 {
            self.base.energy = 0;
            self.perform_death(DEATH_EXPLOSION, g);
            return Some(if self.lives < 1 { Upd::Delete } else { Upd::Keep });
        }
        None
    }

    fn select_weapon(&mut self) {
        for n in 1..=NUM_WEAPONS {
            if self.ctrl.weapon(n) && self.weapon[n - 1].availible {
                self.current_weapon = n - 1;
                break;
            }
        }
        // The original looped forever when no weapon at all was available.
        if self.ctrl.next_weapon() {
            for _ in 0..NUM_WEAPONS {
                self.current_weapon = (self.current_weapon + 1) % NUM_WEAPONS;
                if self.weapon[self.current_weapon].availible {
                    break;
                }
            }
        } else if self.ctrl.prev_weapon() {
            for _ in 0..NUM_WEAPONS {
                self.current_weapon = (self.current_weapon + NUM_WEAPONS - 1) % NUM_WEAPONS;
                if self.weapon[self.current_weapon].availible {
                    break;
                }
            }
        }
    }

    fn fire(&mut self, g: &mut Game) {
        let b = &self.base;
        let (x, y, z, dir, friends) = (b.x, b.y, b.z, b.direction, b.friends as i32);
        let right = dir == RIGHT_DIR;
        let front = x + if right { 67 } else { 20 };
        let ball_speed = if b.speed_x == 0 { if right { 1 } else { -1 } } else { b.speed_x };
        match self.current_weapon {
            ELECTRIC => {
                let wp = self.cur();
                wp.ammo -= 4;
                sound::play_sfx("samples/el.wav");
                if wp.ammo < 0 {
                    wp.ammo = 0;
                }
                for i in 0..EL_LENGTH {
                    let tmp = if right { x + 67 + 32 * i } else { x - 32 * (i + 1) };
                    if g.check_wall(tmp, y + 32, z) {
                        break;
                    }
                    g.create_object(Beam::new(tmp, y + 32, z, friends, 0, 0));
                }
            }
            MINIGUN | SHOTGUN => {
                let h = self.base.height;
                if right {
                    let sx = -(random() % 5);
                    let sy = -(random() % 8);
                    g.create_effect(Bodyparts::new(x + 36, y + 24, z, "soldier/hylsa", 1, sx, sy, 0));
                    g.create_object(HighSpeedBullet::new(x + 60, y + h / 2, z, dir, friends, 1));
                } else {
                    g.create_object(HighSpeedBullet::new(x, y + h / 2, z, dir, friends, 1));
                    let sx = random() % 5;
                    let sy = -(random() % 8);
                    g.create_effect(Bodyparts::new(x + 28, y + 24, z, "soldier/hylsa", 1, sx, sy, 0));
                }
            }
            FLAME_THROWER => {
                sound::play_sfx(FIRE_SAMPLE);
                g.create_object(FireBall::new(front, y + 35, z, friends, ball_speed, 0, 0, 2));
            }
            ICEMAKER => {
                sound::play_sfx(FIRE_SAMPLE);
                g.create_object(IceBall::new(front, y + 35, z, friends, ball_speed, 0, 0));
            }
            GRENADE_LAUNCHER => {
                g.create_object(Rocket::new(front, y + 34, z, if right { 5 } else { -5 }, self.base.mode));
            }
            PLASMA_GUN => {
                sound::play_sfx("samples/plasma3.wav");
                g.create_object(Plasma::new(front, y + 28, z, dir, friends, 5));
            }
            _ => {}
        }
    }

    fn set_frame_image(&mut self, frame: usize) {
        let wp = self.weapon[self.current_weapon];
        let firing = self.base.state & STATE_FIRE != 0;
        let img = match self.base.direction {
            RIGHT_DIR => if firing { wp.fire_right[frame] } else { wp.walk_right[frame] },
            LEFT_DIR => if firing { wp.fire_left[frame] } else { wp.walk_left[frame] },
            UP_DIR => wp.walk_up[frame],
            DOWN_DIR => wp.walk_down[frame],
            _ => return,
        };
        self.base.current_image = img;
    }

    fn clip_x(&mut self, g: &Game, y: i32) {
        let b = &mut self.base;
        if b.speed_x > 0 {
            while g.check_wall(b.x + b.width - 31 + b.speed_x, y, b.z) && b.speed_x != 0 {
                b.speed_x -= 1;
            }
        } else if b.speed_x < 0 {
            while g.check_wall(b.x + 31 + b.speed_x, y, b.z) && b.speed_x != 0 {
                b.speed_x += 1;
            }
        }
    }

    fn clip_z(&mut self, g: &Game, y_pos: i32, y_neg: i32) {
        let b = &mut self.base;
        let l = b.x + 31 + b.speed_x;
        let r = b.x + b.width - 31 + b.speed_x;
        if b.speed_z > 0 {
            while (g.check_wall(l, y_pos, b.z + b.speed_z + 5) || g.check_wall(r, y_pos, b.z + b.speed_z + 5)) && b.speed_z != 0 {
                b.speed_z -= 1;
            }
        } else if b.speed_z < 0 {
            while (g.check_wall(l, y_neg, b.z + b.speed_z - 5) || g.check_wall(r, y_neg, b.z + b.speed_z - 5)) && b.speed_z != 0 {
                b.speed_z += 1;
            }
        }
    }
}

#[typetag::serde]
impl Obj for Player {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        if self.base.counter5 != 0 {
            self.base.counter5 += 1;
            // Fade in if the player has been reborn
            if self.base.counter5 == 20 {
                fade_in_map(g, 2);
                self.base.counter5 = 0;
            }
        }

        if self.base.energy > MAX_ENERGY {
            self.base.energy = MAX_ENERGY;
        }

        if self.base.counter != 0 {
            self.base.counter -= 1;
        }
        if self.base.state & STATE_DEAD != 0 {
            if self.base.counter == 0 {
                let b = &mut self.base;
                b.state |= STATE_IMMORTAL;
                b.state &= !STATE_DEAD;
                b.counter4 = IMMORTAL_DELAY;
                b.counter2 = 0;
                b.x = self.save_x;
                b.y = self.save_y;
                b.z = self.save_z;
                b.counter5 = 1;
                crate::sys::fade_out(4);
            }
            return Upd::Keep;
        }
        if self.base.counter2 != 0 {
            self.base.counter2 -= 1;
        }
        if self.base.counter4 != 0 {
            self.base.counter4 -= 1;
        }

        {
            let b = &mut self.base;
            if b.speed_x > 0 {
                b.speed_x -= X_FRICTION;
            } else if b.speed_x < 0 {
                b.speed_x += X_FRICTION;
            }
            if b.speed_z > 0 {
                b.speed_z -= Z_FRICTION;
            } else if b.speed_z < 0 {
                b.speed_z += Z_FRICTION;
            }
        }

        if let Some(r) = self.handle_cheats(g) {
            return r;
        }

        self.select_weapon();

        if self.base.counter == 0 {
            let dir = self.base.direction;
            if self.base.state & STATE_FIRE != 0 {
                self.base.state &= !STATE_FIRE;
                self.base.counter = self.weapon[self.current_weapon].fire_delay;
            } else if self.ctrl.fire() && self.weapon_availible() && dir != UP_DIR && dir != DOWN_DIR {
                self.base.state |= STATE_FIRE;
                self.base.counter = self.weapon[self.current_weapon].fire_rep;
                self.cur().ammo -= 1;
                self.fire(g);
            } else if self.ctrl.fire() && dir == UP_DIR {
                if let Some(id) = self.elev_station {
                    let me: &Player = self;
                    with_obj::<ElevatorStation, _>(g, id, |s, g| s.start_elevator_for(me, g));
                }
            }
        }

        if self.ctrl.right() {
            if self.base.speed_x < self.max_x_speed() {
                self.base.speed_x += self.x_accel();
            }
            self.base.direction = RIGHT_DIR;
            self.base.state |= STATE_WALK;
        }
        if self.ctrl.left() {
            if self.base.speed_x > -self.max_x_speed() {
                self.base.speed_x -= self.x_accel();
            }
            self.base.direction = LEFT_DIR;
            self.base.state |= STATE_WALK;
        }
        if self.ctrl.up() {
            if self.base.speed_z > -self.max_z_speed() {
                self.base.speed_z -= self.z_accel();
            }
            self.base.direction = UP_DIR;
            self.base.state |= STATE_WALK;
            self.base.state &= !STATE_FIRE;
        }
        if self.ctrl.down() {
            if self.base.speed_z < self.max_z_speed() {
                self.base.speed_z += self.z_accel();
            }
            self.base.direction = DOWN_DIR;
            self.base.state |= STATE_WALK;
            self.base.state &= !STATE_FIRE;
        }

        let jump = self.ctrl.jump();
        if jump && !self.jumping && !self.jump_keypressed {
            self.base.speed_y = self.min_y_speed();
            self.jumping = true;
            self.jump_keypressed = true;
            self.saved_mode = self.base.mode;
        } else if !jump && !self.jumping {
            self.jump_keypressed = false;
        }

        if self.base.state & STATE_WALK != 0 {
            let fd = self.frame_delay();
            let f = self.anim.next_frame(4, fd) as usize;
            self.set_frame_image(f);
        }

        if self.base.speed_x == 0 && self.base.speed_z == 0 {
            self.base.state &= !STATE_WALK;
            self.set_frame_image(5);
        }

        // Fall or stop
        let (x, y, z, w, sy) = (self.base.x, self.base.y, self.base.z, self.base.width, self.base.speed_y);
        if (self.elevator.is_some() && sy > 0)
            || g.check_floor(x + 31, y + 66 + sy, z)
            || g.check_floor(x + w - 31, y + 66 + sy, z)
        {
            if self.normal() && self.fall_counter > FALL_DAMAGE_BEGIN {
                self.base.energy -= FALL_DAMAGE;
            }
            self.fall_counter = 0;
            self.base.speed_y = 0;
            self.jumping = false;
            self.saved_mode = self.base.mode;
            if self.base.energy <= 0 {
                self.base.energy = 0;
                self.perform_death(DEATH_EXPLOSION, g);
            }
        } else if sy < self.max_y_speed() {
            if self.saved_mode == MODE_NORMAL {
                self.base.speed_y += Y_ACCEL;
            } else if self.saved_mode == MODE_WATER {
                if self.base.grav_counter == 4 {
                    self.base.speed_y += 1;
                    self.base.grav_counter = 0;
                } else {
                    self.base.grav_counter += 1;
                }
            }
        }

        if self.base.speed_y > 0 && self.elevator.is_some() {
            self.base.speed_y = 0;
            self.jumping = false;
        }

        // Count how far we have fallen
        if self.base.speed_y > 0 {
            self.fall_counter += 1;
        }

        {
            let b = &mut self.base;
            if b.speed_y < 0 && (g.check_floor(b.x + 31, b.y + b.speed_y, b.z) || g.check_floor(b.x + b.width - 31, b.y + b.speed_y, b.z)) {
                b.speed_y = 0;
            }
        }

        let y = self.base.y;
        // Feet, belly, head
        self.clip_x(g, y + 3 * TILE_SIDE_HEIGHT);
        self.clip_x(g, y + 2 * TILE_SIDE_HEIGHT);
        self.clip_x(g, y + TILE_SIDE_HEIGHT);
        // Head, belly, feet
        self.clip_z(g, y + TILE_SIDE_HEIGHT, y + TILE_SIDE_HEIGHT);
        self.clip_z(g, y + 2 * TILE_SIDE_HEIGHT, y + 2 * TILE_SIDE_HEIGHT);
        self.clip_z(g, y + 3 * TILE_SIDE_HEIGHT, y + 66);

        {
            let b = &mut self.base;
            b.x += b.speed_x;
            b.z += b.speed_z;
            b.y += b.speed_y;

            while g.check_floor(b.x + 31, b.y + 66, b.z) || g.check_floor(b.x + b.width - 31, b.y + 66, b.z) {
                b.y -= 1;
            }

            if b.x < 0 {
                b.x = 0;
            }
            if b.z > MIN_Z {
                b.z = MIN_Z;
            }
            if b.z < MAX_Z {
                b.z = MAX_Z;
            }
            b.layer = b.z / TILE_TOP_HEIGHT;

            if b.state & STATE_IMMORTAL != 0 && b.counter4 == 0 {
                b.state &= !STATE_IMMORTAL;
            }

            if b.state & STATE_IMMORTAL != 0 {
                // Blink while immortal
                if b.counter3 != 0 {
                    b.current_image = EMPTY_IMAGE;
                }
                if b.counter2 == 0 {
                    b.counter3 = if b.counter3 == 0 { 1 } else { 0 };
                    b.counter2 = 10;
                }
            }
        }

        if self.base.mode == MODE_WATER && random() % 75 == 0 {
            g.create_object(Bubble::new(self.base.x, self.base.y, self.base.z));
        }

        self.elevator = None;
        self.elev_station = None;

        if self.lives < 1 { Upd::Delete } else { Upd::Keep }
    }

    fn collision(&mut self, other: &mut dyn Obj, g: &mut Game) {
        let o = other.base();
        if o.me == FRIEND_ELEVATOR {
            self.elevator = Some(o.id);
            return;
        } else if o.me == FRIEND_ELEVSTAT {
            self.elev_station = Some(o.id);
            return;
        }

        if self.base.state & STATE_DEAD != 0 {
            return;
        }

        if o.me == FRIEND_CHECKPOINT {
            self.save_x = self.base.x;
            self.save_y = self.base.y;
            self.save_z = self.base.z;
        }
        // Powerups (medipaks) also heal while immortal; the original consumed them without effect.
        if o.me & FRIEND_POWERUP != 0 {
            if o.enemies & self.base.me != 0 {
                self.base.energy -= o.strength;
            }
            return;
        }
        if self.base.state & STATE_IMMORTAL == 0 {
            if o.enemies & self.base.me != 0 {
                self.base.energy -= o.strength;
            }

            if o.me & ENEMY_HS_BULLET != 0 {
                let b = &self.base;
                let by = b.y + random() % b.height.max(1);
                let s = 2 + random() % 4;
                let s = if o.direction == RIGHT_DIR { s } else { -s };
                g.create_effect(Blood::new(b.x + b.width / 2, by, b.z, s, 0, 0));
            }

            if MAKE_PLAYER_IMMORTAL & o.me != 0 {
                self.base.state |= STATE_IMMORTAL;
                self.base.counter4 = TOUCH_DELAY;
            }
        }
        if self.base.energy <= 0 {
            let tmp = if o.me & ENEMY_FIREBALL != 0 && o.me & ENEMY_EXPLOSION == 0 {
                DEATH_FIREBALL
            } else if o.me & ENEMY_ICEBALL != 0 && o.me & (ENEMY_EXPLOSION | ENEMY_FIREBALL) == 0 {
                DEATH_ICEBALL
            } else {
                DEATH_EXPLOSION
            };
            self.perform_death(tmp, g);
        }
    }
}
