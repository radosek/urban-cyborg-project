//! Health, ammo, weapon and extra-life pickups (objects/powerup.cc).

use crate::engine::*;
use crate::icache;

const POWERUP_STRENGTH: i32 = 100;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Powerup {
    pub base: Base,
    type_: i32,
    msg: String,
}

impl Powerup {
    pub fn new(x: i32, y: i32, z: i32, type_: i32) -> Powerup {
        let mut b = Base::new(x, y, z);
        let mut strength = 0;
        let (file, msg) = match type_ {
            POWERUP_ENERGY => {
                strength = -POWERUP_STRENGTH;
                ("items/medipak.pcx", format!("{}% health", POWERUP_STRENGTH / 3))
            }
            POWERUP_EXTRA_LIFE => ("items/xlife.pcx", "Extra life".to_string()),
            POWERUP_WEAPON_FLAME_THROWER => ("items/flame.pcx", "Flamethrower\nLet's Burn!".to_string()),
            POWERUP_WEAPON_ICEMAKER => ("items/ice.pcx", "IceMaker (tm)\nOrder your own at 555-URBAN".to_string()),
            POWERUP_WEAPON_PLASMA => ("items/plasmal.pcx", "Plasma Gun\nHightech stuff".to_string()),
            POWERUP_WEAPON_MINIGUN => ("items/minigun.pcx", "BIG MiniGun".to_string()),
            POWERUP_WEAPON_ELECTRIC => ("items/electric.pcx", "Electrifier".to_string()),
            POWERUP_WEAPON_GRENADE_LAUNCHER => ("items/granadel.pcx", "Rocket Launcher".to_string()),
            POWERUP_AMMO_FLAME_THROWER => ("items/flamea.pcx", "More Ammo".to_string()),
            POWERUP_AMMO_ICEMAKER => ("items/icea.pcx", "More Ammo".to_string()),
            POWERUP_AMMO_GRENADE_LAUNCHER => ("items/granade.pcx", "More Ammo".to_string()),
            POWERUP_AMMO_GRENADE_LAUNCHER6 => ("items/granadea.pcx", "More Ammo".to_string()),
            POWERUP_AMMO_PLASMA => ("items/plasmaa.pcx", "More Ammo".to_string()),
            POWERUP_AMMO_MINIGUN => ("items/miniguna.pcx", "More Ammo".to_string()),
            POWERUP_AMMO_ELECTRIC => ("items/ela.pcx", "More Ammo".to_string()),
            _ => ("oopsUnkonwnpowerup.pcx", String::new()),
        };
        b.strength = strength;
        b.images = vec![icache::get(file)];
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
        b.friends = !FRIEND_PLAYER;
        b.enemies = FRIEND_PLAYER;
        b.me = FRIEND_POWERUP;
        Powerup { base: b, type_, msg }
    }
}

#[typetag::serde]
impl Obj for Powerup {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
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
        if self.base.energy == 0 || self.base.friends & other.base().me != 0 {
            return;
        }
        g.push_message(&self.msg);

        if other.base().me != FRIEND_PLAYER {
            return;
        }
        self.base.energy = 0;
        let Some(p) = player_of(other, g) else { return };

        let weapon = match self.type_ {
            POWERUP_WEAPON_FLAME_THROWER => Some(FLAME_THROWER),
            POWERUP_WEAPON_ICEMAKER => Some(ICEMAKER),
            POWERUP_WEAPON_GRENADE_LAUNCHER => Some(GRENADE_LAUNCHER),
            POWERUP_WEAPON_PLASMA => Some(PLASMA_GUN),
            POWERUP_WEAPON_ELECTRIC => Some(ELECTRIC),
            POWERUP_WEAPON_MINIGUN => Some(MINIGUN),
            _ => None,
        };
        if let Some(t) = weapon {
            if p.have_weapon(t) {
                self.base.energy = 1;
            } else {
                p.enable_weapon(t);
            }
            return;
        }

        match self.type_ {
            POWERUP_EXTRA_LIFE => p.add_life(),
            POWERUP_AMMO_FLAME_THROWER => p.add_ammo(FLAME_THROWER, 50),
            POWERUP_AMMO_ICEMAKER => p.add_ammo(ICEMAKER, 50),
            POWERUP_AMMO_GRENADE_LAUNCHER => p.add_ammo(GRENADE_LAUNCHER, 1),
            POWERUP_AMMO_GRENADE_LAUNCHER6 => p.add_ammo(GRENADE_LAUNCHER, 6),
            POWERUP_AMMO_PLASMA => p.add_ammo(PLASMA_GUN, 20),
            POWERUP_AMMO_MINIGUN => p.add_ammo(MINIGUN, 100),
            POWERUP_AMMO_ELECTRIC => p.add_ammo(ELECTRIC, 20),
            _ => {}
        }
    }
}
