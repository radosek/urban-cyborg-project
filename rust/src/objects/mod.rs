//! Game objects (port of ref/objects/*.cc).

pub mod anim;
pub mod ai_hero;
pub mod ai_sci;
pub mod ai_soldier;
pub mod airexp;
pub mod airgren;
pub mod airstrke;
pub mod animate;
pub mod arnold;
pub mod barrel;
pub mod bdyparts;
pub mod beam;
pub mod blood;
pub mod bubble;
pub mod bubblmak;
pub mod burning;
pub mod card;
pub mod checkp;
pub mod clugren;
pub mod cluster;
pub mod compscrn;
pub mod crycheck;
pub mod dekor;
pub mod dolph;
pub mod door;
pub mod drgreen;
pub mod einstein;
pub mod elevator;
pub mod elevstat;
pub mod elevwire;
pub mod explode;
pub mod exptile;
pub mod fire;
pub mod fireball;
pub mod floorgun;
pub mod frosen;
pub mod gnistor;
pub mod grenade;
pub mod hs_bulle;
pub mod iceball;
pub mod mine;
pub mod missile;
pub mod plasma;
pub mod player;
pub mod points;
pub mod powerup;
pub mod rambo;
pub mod rocket;
pub mod science;
pub mod seagal;
pub mod smallexp;
pub mod soldier;
pub mod soldier2;
pub mod soldier3;
pub mod soldier4;
pub mod spy;
pub mod tank;
pub mod tankwhel;
pub mod vaktm;
pub mod water;

use crate::engine::*;
use crate::otypes::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Spawn {
    Player(Box<player::Player>),
    Object(Box<dyn Obj>),
    Inner(Box<dyn Obj>),
    Always(Box<dyn Obj>),
}

fn o<T: Obj>(x: T) -> Option<Spawn> {
    Some(Spawn::Object(Box::new(x)))
}

fn inner<T: Obj>(x: T) -> Option<Spawn> {
    Some(Spawn::Inner(Box::new(x)))
}

/// Engine::create_objects: builds the object for map cell type `t` at tile-space origin (x, y)
/// on layer `k`. Most objects stand mid-tile (`z = k*TOP + TOP/2`); doors, barrels, decorations
/// and the tank sit on the tile edge (`z = k*TOP`).
pub fn spawn(t: i32, x: i32, y: i32, k: i32, controls: i32) -> Option<Spawn> {
    let z = k * TILE_TOP_HEIGHT + (TILE_TOP_HEIGHT >> 1);
    let ze = k * TILE_TOP_HEIGHT;
    let pu = |kind| o(powerup::Powerup::new(x, y, z, kind));
    let dk = |f: &str| o(dekor::Dekor::new(x, y, ze, f));
    match t {
        TYPE_PLAYER => Some(Spawn::Player(Box::new(player::Player::new(x, y, z, controls)))),
        TYPE_SCIENTIST => o(science::Scientist::new(x, y, z)),
        TYPE_SOLDIER => o(soldier::Soldier::new(x, y, z)),
        TYPE_SOLDIER2 => o(soldier2::Soldier2::new(x, y, z)),
        TYPE_VAKTM => o(vaktm::Vaktm::new(x, y, z)),
        TYPE_POWERUP_ENERGY => pu(POWERUP_ENERGY),
        TYPE_EXTRA_LIFE => pu(POWERUP_EXTRA_LIFE),
        TYPE_CHECKPOINT => o(checkp::Checkpoint::new(x, y, z)),
        TYPE_WEAPON_FLAME_THROWER => pu(POWERUP_WEAPON_FLAME_THROWER),
        TYPE_WEAPON_ICEMAKER => pu(POWERUP_WEAPON_ICEMAKER),
        TYPE_WEAPON_GRENADE_LAUNCHER => pu(POWERUP_WEAPON_GRENADE_LAUNCHER),
        TYPE_WEAPON_PLASMA => pu(POWERUP_WEAPON_PLASMA),
        TYPE_AMMO_PLASMA => pu(POWERUP_AMMO_PLASMA),
        TYPE_WEAPON_ELECTRIC => pu(POWERUP_WEAPON_ELECTRIC),
        TYPE_AMMO_ELECTRIC => pu(POWERUP_AMMO_ELECTRIC),
        TYPE_AMMO_FLAME_THROWER => pu(POWERUP_AMMO_FLAME_THROWER),
        TYPE_AMMO_ICEMAKER => pu(POWERUP_AMMO_ICEMAKER),
        TYPE_AMMO_GRENADE_LAUNCHER => pu(POWERUP_AMMO_GRENADE_LAUNCHER),
        TYPE_AMMO_GRENADE_LAUNCHER6 => pu(POWERUP_AMMO_GRENADE_LAUNCHER6),
        TYPE_WEAPON_MINIGUN => pu(POWERUP_WEAPON_MINIGUN),
        TYPE_AMMO_MINIGUN => pu(POWERUP_AMMO_MINIGUN),
        TYPE_BOSS_SCIENTIST => o(science::BossScientist::new(x, y, z)),
        TYPE_BOSS_SOLDIER => o(soldier::BossSoldier::new(x, y, z)),
        TYPE_MINE => o(mine::Mine::new(x, y, z)),
        TYPE_FLOORGUN => o(floorgun::FloorGun::new(x, y, z)),
        TYPE_RAMBO => o(rambo::Rambo::new(x, y, z)),
        TYPE_BOSS_RAMBO => o(rambo::BossRambo::new(x, y, z)),
        TYPE_ARNOLD => o(arnold::Arnold::new(x, y, z)),
        TYPE_BOSS_ARNOLD => o(arnold::BossArnold::new(x, y, z)),
        TYPE_DOOR | TYPE_GREEN_DOOR => inner(door::Door::new(x, y, ze, CARD_GREEN)),
        TYPE_RED_DOOR => inner(door::Door::new(x, y, ze, CARD_RED)),
        TYPE_BLUE_DOOR => inner(door::Door::new(x, y, ze, CARD_BLUE)),
        TYPE_DOOR_EXTRA_LEVEL => inner(door::Door::new(x, y, ze, CARD_EXTRA_LEVEL)),
        TYPE_BARREL => o(barrel::Barrel::new(x, y, ze, 0)),
        TYPE_DOLPH => o(dolph::Dolph::new(x, y, z)),
        TYPE_BOSS_DOLPH => o(dolph::BossDolph::new(x, y, z)),
        TYPE_EINSTEIN => o(einstein::Einstein::new(x, y, z)),
        TYPE_BOSS_EINSTEIN => o(einstein::BossEinstein::new(x, y, z)),
        TYPE_CRYCHECK => o(crycheck::Crycheck::new(x, y, z)),
        TYPE_BOSS_CRYCHECK => o(crycheck::BossCrycheck::new(x, y, z)),
        TYPE_LEFT_COMPUTER_SCREEN => o(compscrn::ComputerScreen::new(x, y, z, 0)),
        TYPE_RIGHT_COMPUTER_SCREEN => o(compscrn::ComputerScreen::new(x, y, z, 1)),
        TYPE_DR_GREEN => o(drgreen::DrGreen::new(x, y, z)),
        TYPE_BOSS_DR_GREEN => o(drgreen::BossDrGreen::new(x, y, z)),
        TYPE_DEKOR_ROBOT1 => dk("dekor/robot1.pcx"),
        TYPE_DEKOR_ROBOT2 => dk("dekor/robot2.pcx"),
        TYPE_DEKOR_ROBOT3 => dk("dekor/robot3.pcx"),
        TYPE_DEKOR_STOL1 => dk("dekor/stol1.pcx"),
        TYPE_DEKOR_STOL2 => dk("dekor/stol2.pcx"),
        TYPE_DEKOR_RITNING1 => dk("dekor/ritning1.pcx"),
        TYPE_DEKOR_RITNING2 => dk("dekor/ritning2.pcx"),
        TYPE_DEKOR_WEAPON1 => dk("dekor/vapen.pcx"),
        TYPE_DEKOR_HELMET1 => dk("dekor/hjalm.pcx"),
        TYPE_DEKOR_ROCKETPART1 => dk("dekor/rockpart.pcx"),
        TYPE_DEKOR_ROCKETPART2 => dk("dekor/warhead.pcx"),
        TYPE_DEKOR_ROCKETPART3 => dk("dekor/rockp2.pcx"),
        TYPE_DEKOR_FLAG => dk("dekor/flagga.pcx"),
        TYPE_SEAGAL => o(seagal::Seagal::new(x, y, z)),
        TYPE_BOSS_SEAGAL => o(seagal::BossSeagal::new(x, y, z)),
        TYPE_ELEVATOR => inner(elevator::Elevator::new(x, y, z)),
        TYPE_ELEVUP => inner(elevstat::ElevatorStation::new(x, y, z, UP_DIR, CARD_NONE)),
        TYPE_ELEVDOWN => inner(elevstat::ElevatorStation::new(x, y, z, DOWN_DIR, CARD_NONE)),
        TYPE_ELEVUP_GREEN => inner(elevstat::ElevatorStation::new(x, y, z, UP_DIR, CARD_GREEN)),
        TYPE_ELEVDOWN_GREEN => inner(elevstat::ElevatorStation::new(x, y, z, DOWN_DIR, CARD_GREEN)),
        TYPE_ELEVWIRE => inner(elevwire::ElevatorWire::new(x, y, z)),
        TYPE_SOLDIER4 => o(soldier4::Soldier4::new(x, y, z)),
        TYPE_SOLDIER3 => o(soldier3::Soldier3::new(x, y, z)),
        TYPE_WATER => o(water::Water::new(x, y - 2 + k, z)),
        TYPE_BUBBLEMAKER => o(bubblmak::BubbleMaker::new(x, y, z)),
        TYPE_AIRSTRIKER => Some(Spawn::Always(Box::new(airstrke::Airstrike::new(x, y, z)))),
        TYPE_RED_CARD => o(card::Card::new(x, y, z, CARD_RED)),
        TYPE_BLUE_CARD => o(card::Card::new(x, y, z, CARD_BLUE)),
        TYPE_GREEN_CARD => o(card::Card::new(x, y, z, CARD_GREEN)),
        TYPE_TANK => o(tank::Tank::new(x, y, ze)),
        _ => None,
    }
}
