//! Shared AI for the "hero" enemies (rambo.cc, arnold.cc, dolph.cc, seagal.cc), which are
//! copies of one another with different sprites and weapons, plus DEATH_BY_EXPLOSION
//! (include/death.h).

use crate::engine::*;
use crate::icache::{self, Img};
use crate::objects::anim::Anim;
use crate::objects::animate::Animation;
use crate::objects::bdyparts::Bodyparts;
use crate::objects::blood::Blood;
use crate::objects::burning::BurningBody;
use crate::objects::card::Card;
use crate::objects::fireball::FireBall;
use crate::objects::frosen::FrosenBody;
use crate::objects::hs_bulle::HighSpeedBullet;
use crate::objects::missile::Missile;
use crate::rng::random;

const SPLAT_SOUND: &str = "samples/gutsplat.wav";

/// DEATH_BY_EXPLOSION from include/death.h.
pub fn death_by_explosion(b: &Base, g: &mut Game) {
    let (x, y, z) = (b.x, b.y, b.z);
    let cx = x + b.width / 2;
    let cy = y + b.height / 2;
    let r = random() % 3;
    for _ in 0..10 {
        g.create_effect(Blood::new(cx, y, z, random() % 8 + 2, -(random() % 4) - 2, 0));
    }
    for _ in 0..10 {
        g.create_effect(Blood::new(cx, y, z, -(random() % 8) - 2, -(random() % 4) - 2, 0));
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
            crate::sound::play_sfx(SPLAT_SOUND);
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
            crate::sound::play_sfx(SPLAT_SOUND);
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
            crate::sound::play_sfx(SPLAT_SOUND);
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Rambo,
    Arnold,
    Dolph,
    Seagal,
}

const PROB_WALK_START: i32 = 0;
const PROB_WALK_END: i32 = 50;
const PROB_STOP_START: i32 = 51;
const PROB_STOP_END: i32 = 60;
const PROB_TURN_START: i32 = 61;
const PROB_TURN_END: i32 = 100;
const PROB_NUMBER: i32 = 100;

const MAX_NUM: i32 = 15;
const STATE_NONE: i32 = 0;
const STATE_STOP: i32 = 1;
const STATE_WALK: i32 = 2;
const STATE_TURN: i32 = 3;
const STATE_HIT: i32 = 4;
const STATE_HIT_COUNTDOWN: i32 = 5;
const STATE_FIRE: i32 = 6;
const STATE_FIRING: i32 = 7;
const STATE_ALARM: i32 = 8;
const STATE_JUMP: i32 = 10;
const STATE_JUMPING: i32 = 11;
const STATE_LAND: i32 = 12;

const MAX_X_SPEED: i32 = 2;
const MIN_X_SPEED: i32 = -MAX_X_SPEED;
const MAX_Y_SPEED: i32 = 16;
const MIN_Y_SPEED: i32 = -12;
const X_ACCEL: i32 = 2;
const Y_ACCEL: i32 = 1;
const Z_ACCEL: i32 = 2;
const MAX_Z_SPEED: i32 = 2;
const MIN_Z_SPEED: i32 = -MAX_Z_SPEED;

const ALARM_DELAY: i32 = 5;
const HIT_DELAY: i32 = 10;
const JUMP_DELAY: i32 = 20;
const LAND_DELAY: i32 = 10;

const FOOT_LEFT: i32 = 39;
const FOOT_WIDTH: i32 = 17;
const FOOT_RIGHT: i32 = FOOT_LEFT + FOOT_WIDTH;

const NO_BOX_DISTANCE: i32 = 20;
const BOX_DISTANCE: i32 = 50;

const FLAME_RANGE: i32 = 100;
const FLAME_STRENGTH: i32 = 5;

#[derive(serde::Serialize, serde::Deserialize)]
struct Cfg {
    dir: &'static str,
    shots: i32,
    fire_range: i32,
    fire_strength: i32,
    fire_delay: i32,
    fire_frames: i32,
    firing_delay: i32,
    jumps: bool,
    energy: i32,
    score: i32,
}

fn cfg(k: Kind) -> Cfg {
    match k {
        Kind::Rambo => Cfg { dir: "specf", shots: 2, fire_range: 150, fire_strength: 3, fire_delay: 60, fire_frames: 1, firing_delay: 3, jumps: true, energy: 20, score: 500 },
        Kind::Arnold => Cfg { dir: "specf2", shots: 5, fire_range: 150, fire_strength: 15, fire_delay: 60, fire_frames: 4, firing_delay: 10, jumps: true, energy: 20, score: 500 },
        Kind::Dolph => Cfg { dir: "specf3", shots: 3, fire_range: 200, fire_strength: 5, fire_delay: 60, fire_frames: 1, firing_delay: 5, jumps: true, energy: 20, score: 500 },
        Kind::Seagal => Cfg { dir: "specf4", shots: 4, fire_range: 300, fire_strength: 5, fire_delay: 25, fire_frames: 3, firing_delay: 5, jumps: false, energy: 30, score: 900 },
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Hero {
    anim: Anim,
    kind: Kind,
    // Jump bookkeeping kept apart from counter2, which the firing logic also uses.
    jump_armed: bool,
    landed: bool,
}

impl Hero {
    pub fn new(kind: Kind, x: i32, y: i32, z: i32) -> (Hero, Base) {
        let c = cfg(kind);
        let d = c.dir;
        let mut images: Vec<Img> = Vec::new();
        images.extend(icache::seq(&format!("{}/specv/", d), 1, 5, ".pcx"));
        images.extend(icache::seq(&format!("{}/spech/", d), 1, 5, ".pcx"));
        images.extend(icache::seq(&format!("{}/specfram/", d), 1, 5, ".pcx"));
        images.extend(icache::seq(&format!("{}/specbak/", d), 1, 5, ".pcx"));
        images.extend(icache::seq(&format!("{}/specv/shot", d), 1, c.shots, ".pcx"));
        images.extend(icache::seq(&format!("{}/spech/shot", d), 1, c.shots, ".pcx"));
        images.push(icache::get(&format!("{}/spech/hit.pcx", d)));
        images.push(icache::get(&format!("{}/specv/hit.pcx", d)));
        if c.jumps {
            images.extend(icache::seq(&format!("{}/specv/hopp", d), 1, 2, ".pcx"));
            images.extend(icache::seq(&format!("{}/spech/hopp", d), 1, 2, ".pcx"));
        }

        let mut b = Base::new(x, y, z);
        b.current_image = 1;
        b.height = images[0].h;
        b.width = images[0].w;
        b.coll_x = 33;
        b.coll_y = 0;
        b.coll_width = 31;
        b.coll_height = b.height - 1;
        // stand on the feet
        b.y -= images[0].h;
        b.images = images;
        b.energy = c.energy;
        b.strength = 10;
        b.direction = RIGHT_DIR;
        b.state = STATE_NONE;
        b.counter = 0;
        b.score = c.score;
        b.friends = !FRIEND_PLAYER;
        b.enemies = ENEMY_PLAYER;
        b.me = FRIEND_RAMBO;
        (Hero { anim: Anim::default(), kind, jump_armed: false, landed: false }, b)
    }

    fn shot_l(&self) -> i32 {
        20
    }
    fn shot_r(&self) -> i32 {
        20 + cfg(self.kind).shots
    }
    fn hit_r(&self) -> i32 {
        20 + 2 * cfg(self.kind).shots
    }
    fn jump_l(&self) -> i32 {
        22 + 2 * cfg(self.kind).shots
    }
    fn jump_r(&self) -> i32 {
        24 + 2 * cfg(self.kind).shots
    }

    pub fn update(&mut self, b: &mut Base, g: &mut Game) -> Upd {
        let c = cfg(self.kind);
        let kind = self.kind;
        let (px, player_layer) = {
            let p = g.pbase();
            (p.x, p.layer)
        };
        let jump_l = self.jump_l();
        let jump_r = self.jump_r();
        let shot_l = self.shot_l();
        let shot_r = self.shot_r();
        let hit_r = self.hit_r();
        let coll_xr = b.coll_x + b.coll_width;
        let coll_y = b.coll_y + b.coll_height;
        let r_dir = RIGHT_DIR;

        if b.energy <= 0 {
            return Upd::Delete;
        }

        if b.counter != 0 {
            b.counter -= 1;
        }

        if b.counter == 0 {
            if c.jumps && b.state == STATE_JUMP {
                b.state = STATE_JUMPING;
            } else if c.jumps && b.state == STATE_JUMPING {
                // Airborne: keep the jump until the landing check below.
            } else {
                let r = random() % PROB_NUMBER;
                if r >= PROB_WALK_START && r <= PROB_WALK_END {
                    b.state = STATE_WALK;
                    b.direction = random() % 4;
                } else if (r >= PROB_STOP_START && r <= PROB_STOP_END) || b.state == STATE_STOP || b.state == STATE_TURN {
                    b.state = STATE_STOP;
                } else if r >= PROB_TURN_START && r <= PROB_TURN_END {
                    b.state = STATE_TURN;
                }
                b.counter = random() % MAX_NUM + 15;
            }
        }

        let busy = matches!(
            b.state,
            STATE_FIRE | STATE_FIRING | STATE_TURN | STATE_ALARM | STATE_HIT | STATE_HIT_COUNTDOWN
        ) || (c.jumps && matches!(b.state, STATE_JUMP | STATE_JUMPING));
        if !busy {
            let d = b.direction;
            let left = px < b.x && px > b.x - c.fire_range && (d == LEFT_DIR || d == UP_DIR || d == DOWN_DIR);
            let right = !left && px > b.x && px < b.x + c.fire_range && (d == RIGHT_DIR || d == UP_DIR || d == DOWN_DIR);
            if left || right {
                if player_layer == b.layer {
                    b.state = STATE_FIRE;
                } else if kind == Kind::Seagal {
                    b.state = STATE_WALK;
                    b.direction = if player_layer > b.layer { DOWN_DIR } else { UP_DIR };
                    b.counter = 15;
                }
            }
        }

        if b.speed_y == 0 {
            let st = b.state;
            match st {
                STATE_LAND if c.jumps => {
                    b.speed_x = 0;
                    b.current_image = if b.direction == r_dir { jump_r } else { jump_l };
                    if !self.landed {
                        self.landed = true;
                        b.counter = LAND_DELAY;
                    }
                }
                STATE_JUMP if c.jumps => {
                    b.speed_x = 0;
                    b.current_image = if b.direction == r_dir { jump_r } else { jump_l };
                    if !self.jump_armed {
                        self.jump_armed = true;
                        self.landed = false;
                        b.counter = JUMP_DELAY;
                    }
                }
                STATE_JUMPING if c.jumps => {
                    if self.jump_armed {
                        b.speed_y = MIN_Y_SPEED;
                        b.speed_x = if b.direction == r_dir { 5 } else { -5 };
                        b.y += b.speed_y;
                        self.jump_armed = false;
                    }
                    b.counter = 5;
                    b.current_image = if b.direction == r_dir { jump_r + 1 } else { jump_l + 1 };
                }
                STATE_WALK => match b.direction {
                    RIGHT_DIR => {
                        if b.speed_x < MAX_X_SPEED {
                            b.speed_x += X_ACCEL;
                        }
                        let x0 = b.x + b.coll_x + b.speed_x;
                        if c.jumps
                            && g.check_wall(x0 + BOX_DISTANCE, b.y + b.height, b.z)
                            && !g.check_wall(x0 + NO_BOX_DISTANCE, b.y + b.height, b.z)
                            && !g.check_wall(x0 + BOX_DISTANCE, b.y + b.height - TILE_SIDE_HEIGHT, b.z)
                        {
                            b.state = STATE_JUMP;
                        } else {
                            while g.check_wall(b.x + coll_xr + b.speed_x, b.y + coll_y, b.z) && b.speed_x != 0 {
                                b.speed_x -= 1;
                            }
                        }
                        b.current_image = 5 + self.anim.next_frame(4, 5);
                        b.speed_z = 0;
                    }
                    LEFT_DIR => {
                        if b.speed_x > MIN_X_SPEED {
                            b.speed_x -= X_ACCEL;
                        }
                        // Jump when there is no box NO_BOX_DISTANCE ahead, a box BOX_DISTANCE
                        // away, and nothing above that box.
                        let x0 = b.x + b.coll_x + b.speed_x;
                        if c.jumps
                            && g.check_wall(x0 - BOX_DISTANCE, b.y + b.height, b.z)
                            && !g.check_wall(x0 - NO_BOX_DISTANCE, b.y + b.height, b.z)
                            && !g.check_wall(x0 - BOX_DISTANCE, b.y + b.height - TILE_SIDE_HEIGHT, b.z)
                        {
                            b.state = STATE_JUMP;
                        } else {
                            while g.check_wall(b.x + b.coll_x + b.speed_x, b.y + coll_y, b.z) && b.speed_x != 0 {
                                b.speed_x += 1;
                            }
                        }
                        b.current_image = self.anim.next_frame(4, 5);
                        b.speed_z = 0;
                    }
                    UP_DIR => {
                        if b.speed_z > MIN_Z_SPEED {
                            b.speed_z -= Z_ACCEL;
                        }
                        while (g.check_wall(b.x + b.coll_x, b.y + coll_y, b.z + b.speed_z - 5)
                            || g.check_wall(b.x + coll_xr, b.y + coll_y, b.z + b.speed_z - 5))
                            && b.speed_z != 0
                        {
                            b.speed_z += 1;
                        }
                        b.current_image = 15 + self.anim.next_frame(4, 5);
                        b.speed_x = 0;
                    }
                    DOWN_DIR => {
                        if b.speed_z < MAX_Z_SPEED {
                            b.speed_z += Z_ACCEL;
                        }
                        while (g.check_wall(b.x + b.coll_x, b.y + coll_y, b.z + b.speed_z + 5)
                            || g.check_wall(b.x + coll_xr, b.y + coll_y, b.z + b.speed_z + 5))
                            && b.speed_z != 0
                        {
                            b.speed_z -= 1;
                        }
                        b.current_image = 10 + self.anim.next_frame(4, 5);
                        b.speed_x = 0;
                    }
                    _ => b.direction = LEFT_DIR,
                },
                STATE_STOP => {
                    b.speed_x = 0;
                    b.speed_z = 0;
                }
                STATE_TURN => {
                    if b.direction == RIGHT_DIR || b.direction == LEFT_DIR {
                        b.direction = if b.direction == RIGHT_DIR { LEFT_DIR } else { RIGHT_DIR };
                        b.speed_x = 0;
                        b.counter = random() % MAX_NUM + 15;
                        b.state = STATE_WALK;
                    }
                }
                STATE_NONE => {
                    b.current_image = if b.direction == RIGHT_DIR || b.direction == LEFT_DIR {
                        if b.direction == RIGHT_DIR { 4 } else { 9 }
                    } else if b.direction == UP_DIR {
                        14
                    } else {
                        19
                    };
                    b.counter = 0;
                    self.anim.reset();
                }
                STATE_ALARM => {
                    let alarm_r = if kind == Kind::Arnold { shot_r } else { 22 };
                    if b.current_image != alarm_r && b.current_image != 20 {
                        b.counter = ALARM_DELAY;
                        b.current_image = if b.direction == RIGHT_DIR { alarm_r } else { 20 };
                        b.speed_x = 0;
                        b.speed_z = 0;
                    }
                }
                STATE_FIRE | STATE_FIRING => {
                    if st == STATE_FIRE {
                        b.speed_x = 0;
                        b.speed_z = 0;
                        b.direction = if px < b.x { LEFT_DIR } else { RIGHT_DIR };
                        if kind == Kind::Dolph {
                            b.counter3 = if b.direction == LEFT_DIR {
                                (px > b.x - FLAME_RANGE) as i32
                            } else {
                                (px < b.x + b.width + FLAME_RANGE) as i32
                            };
                        }
                        b.counter = c.fire_delay;
                        b.state = STATE_FIRING;

                        self.anim.reset();
                        b.counter2 = 0;
                    }
                    let base_img = if b.direction == RIGHT_DIR { shot_r } else { shot_l };
                    let last = c.fire_frames;
                    let r = self.anim.next_frame(last, c.firing_delay);
                    b.current_image = base_img + r;
                    let right = b.direction == RIGHT_DIR;
                    let (x, y, z) = (b.x, b.y, b.z);
                    let evil = !(FRIEND_PLAYER | FRIEND_SCIENTIST) as i32;
                    match kind {
                        Kind::Seagal => {
                            if r == 1 && b.counter2 == 0 {
                                if right {
                                    g.create_object(Missile::new(x + 70, y + 22, z, 10));
                                } else {
                                    g.create_object(Missile::new(x, y + 22, z, -10));
                                }
                                b.counter2 = 1;
                            } else if r == last {
                                b.state = STATE_STOP;
                            }
                        }
                        Kind::Dolph => {
                            if r == 1 && b.counter2 == 0 {
                                if b.counter3 == 1 {
                                    g.create_object(FireBall::new(
                                        x + if right { 47 } else { 20 },
                                        y + 17,
                                        z,
                                        b.friends as i32,
                                        if right { 1 } else { -1 },
                                        0,
                                        0,
                                        FLAME_STRENGTH,
                                    ));
                                } else {
                                    g.create_object(HighSpeedBullet::new(x, y + 22, z, b.direction, evil, c.fire_strength));
                                    shell(g, right, x, y, z);
                                }
                                b.counter2 = 1;
                            } else if r == 0 {
                                b.counter2 = 0;
                            }
                            if r == 1 && b.counter3 == 1 {
                                b.current_image += 1;
                            }
                        }
                        Kind::Rambo | Kind::Arnold => {
                            if r == 1 && b.counter2 == 0 {
                                if kind == Kind::Arnold {
                                    crate::sound::play_sfx("samples/shotgn2.wav");
                                }
                                g.create_object(HighSpeedBullet::new(x, y + 22, z, b.direction, evil, c.fire_strength));
                                b.counter2 = 1;
                                shell(g, right, x, y, z);
                            } else if r == 0 {
                                b.counter2 = 0;
                            }
                        }
                    }
                }
                STATE_HIT | STATE_HIT_COUNTDOWN => {
                    if st == STATE_HIT {
                        if b.direction == UP_DIR || b.direction == DOWN_DIR {
                            b.direction = if px < b.x { LEFT_DIR } else { RIGHT_DIR };
                        }
                        b.state = STATE_HIT_COUNTDOWN;
                        b.counter = HIT_DELAY;
                    }
                    b.current_image = if b.direction == RIGHT_DIR { hit_r } else { hit_r + 1 };
                }
                _ => {}
            }
        }

        if b.speed_y == 0 {
            match b.direction {
                LEFT_DIR => {
                    if hole(g, b, FOOT_LEFT + b.speed_x, 0) {
                        b.speed_x = 0;
                        b.state = STATE_TURN;
                    }
                }
                RIGHT_DIR => {
                    if hole(g, b, FOOT_RIGHT + b.speed_x, 0) {
                        b.speed_x = 0;
                        b.state = STATE_TURN;
                    }
                }
                UP_DIR | DOWN_DIR => {
                    if hole(g, b, FOOT_RIGHT, b.speed_z) {
                        b.speed_z = 0;
                        b.state = STATE_TURN;
                    }
                }
                _ => {}
            }
        }

        // Fall or stop
        if g.check_floor(b.x + FOOT_LEFT, b.y + b.height, b.z) || g.check_floor(b.x + FOOT_RIGHT, b.y + b.height, b.z) {
            b.speed_y = 0;
            if c.jumps && b.state == STATE_JUMPING {
                b.state = STATE_LAND;
            }
        } else if b.speed_y < MAX_Y_SPEED {
            b.speed_y += Y_ACCEL;
            b.speed_z = 0;
        }

        // Airborne: the jump's horizontal speed must not carry through walls.
        if b.speed_y != 0 {
            if b.speed_x > 0 {
                while g.check_wall(b.x + coll_xr + b.speed_x, b.y + coll_y, b.z) && b.speed_x != 0 {
                    b.speed_x -= 1;
                }
            } else if b.speed_x < 0 {
                while g.check_wall(b.x + b.coll_x + b.speed_x, b.y + coll_y, b.z) && b.speed_x != 0 {
                    b.speed_x += 1;
                }
            }
        }

        b.x += b.speed_x;
        b.y += b.speed_y;
        b.z += b.speed_z;

        if b.x < 0 {
            b.x = 0;
        }

        if b.z > MIN_Z {
            b.z = MIN_Z;
            b.speed_z = 0;
            b.state = STATE_STOP;
            b.counter = 0;
        }
        if b.z < MAX_Z {
            b.z = MAX_Z;
            b.speed_z = 0;
            b.state = STATE_STOP;
            b.counter = 0;
        }

        b.layer = b.z / TILE_TOP_HEIGHT;

        let jumping = c.jumps && (b.state == STATE_JUMP || b.state == STATE_JUMPING);
        if b.speed_x == 0
            && b.speed_z == 0
            && !jumping
            && b.state != STATE_HIT_COUNTDOWN
            && b.state != STATE_FIRING
            && b.state != STATE_ALARM
        {
            if b.direction == LEFT_DIR || b.direction == RIGHT_DIR {
                b.current_image = if b.direction == RIGHT_DIR { 4 } else { 9 };
            } else if b.direction == UP_DIR || b.direction == DOWN_DIR {
                b.current_image = if b.direction == UP_DIR { 14 } else { 19 };
            }
        }
        Upd::Keep
    }

    pub fn collision(&mut self, b: &mut Base, other: &mut dyn Obj, g: &mut Game) {
        if b.energy == 0 {
            return;
        }
        default_collision(b, other.base());

        let o = other.base();
        if o.strength > 0 && o.friends & b.me == 0 {
            let (x, y, z) = (b.x, b.y, b.z);
            let side = if b.direction == RIGHT_DIR { "spech" } else { "specv" };
            if b.energy <= 0 {
                if o.me & ENEMY_EXPLOSION != 0 {
                    death_by_explosion(b, g);
                } else if o.me & (ENEMY_FIREBALL | ENEMY_BEAM) != 0 {
                    g.create_effect(BurningBody::new(x, y + b.height, z, 1));
                } else if o.me & ENEMY_ICEBALL != 0 {
                    g.create_object(FrosenBody::new(x, y + b.height, z, b.direction));
                } else {
                    let name = format!("{}/{}/dead", cfg(self.kind).dir, side);
                    g.create_effect(Animation::new(x, y + b.height, z, &name, 4, 3, 0, 0, 0));
                }
            }
            let no_blood = if self.kind == Kind::Dolph {
                ENEMY_FIREBALL | ENEMY_ICEBALL
            } else {
                ENEMY_FIREBALL | ENEMY_ICEBALL | ENEMY_BEAM
            };
            if o.me & no_blood == 0 {
                for _ in 0..3 {
                    let by = y + random() % b.height;
                    let sx = -2 + random() % 4;
                    g.create_effect(Blood::new(x + b.width / 2, by, z, sx, 0, 0));
                }
            }
            if b.energy % 4 == 0 {
                b.state = STATE_HIT;
            }
        }
    }

    /// Boss_*_o::update: drops a card when it dies.
    pub fn boss_update(&mut self, b: &mut Base, g: &mut Game) -> Upd {
        let ret = self.update(b, g);
        if ret == Upd::Delete {
            g.create_object(Card::new(b.x, b.y, b.z, CARD_GREEN));
        }
        ret
    }
}

fn hole(g: &Game, b: &Base, dx: i32, dz: i32) -> bool {
    !g.check_floor(b.x + dx, b.y + b.height, b.z + dz) && !g.check_floor(b.x + dx, b.y + b.height + TILE_SIDE_HEIGHT, b.z + dz)
}

fn shell(g: &mut Game, right: bool, x: i32, y: i32, z: i32) {
    if right {
        let sx = -(random() % 5);
        let sy = -(random() % 8);
        g.create_effect(Bodyparts::new(x + 56, y + 22, z, "soldier/hylsa", 1, sx, sy, 0));
    } else {
        let sx = random() % 5;
        let sy = -(random() % 8);
        g.create_effect(Bodyparts::new(x + 44, y + 22, z, "soldier/hylsa", 1, sx, sy, 0));
    }
}

/// Generates `$name` and its boss variant as thin wrappers over `Hero`.
#[macro_export]
macro_rules! hero_objects {
    ($name:ident, $boss:ident, $kind:expr, $boss_score:expr, $boss_energy:expr) => {
        #[derive(serde::Serialize, serde::Deserialize)]
        pub struct $name {
            pub base: $crate::engine::Base,
            ai: $crate::objects::ai_hero::Hero,
        }

        impl $name {
            pub fn new(x: i32, y: i32, z: i32) -> $name {
                let (ai, base) = $crate::objects::ai_hero::Hero::new($kind, x, y, z);
                $name { base, ai }
            }
        }

        #[typetag::serde]

        impl $crate::engine::Obj for $name {
            $crate::obj_boilerplate!();
            fn update(&mut self, g: &mut $crate::engine::Game) -> $crate::engine::Upd {
                self.ai.update(&mut self.base, g)
            }
            fn collision(&mut self, other: &mut dyn $crate::engine::Obj, g: &mut $crate::engine::Game) {
                self.ai.collision(&mut self.base, other, g)
            }
        }

        #[derive(serde::Serialize, serde::Deserialize)]

        pub struct $boss {
            pub base: $crate::engine::Base,
            ai: $crate::objects::ai_hero::Hero,
        }

        impl $boss {
            pub fn new(x: i32, y: i32, z: i32) -> $boss {
                let (ai, mut base) = $crate::objects::ai_hero::Hero::new($kind, x, y, z);
                base.score = $boss_score;
                base.energy = $boss_energy;
                $boss { base, ai }
            }
        }

        #[typetag::serde]

        impl $crate::engine::Obj for $boss {
            $crate::obj_boilerplate!();
            fn update(&mut self, g: &mut $crate::engine::Game) -> $crate::engine::Upd {
                self.ai.boss_update(&mut self.base, g)
            }
            fn collision(&mut self, other: &mut dyn $crate::engine::Obj, g: &mut $crate::engine::Game) {
                self.ai.collision(&mut self.base, other, g)
            }
        }
    };
}
