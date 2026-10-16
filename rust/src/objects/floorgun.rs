//! Port of ref/objects/floorgun.cc: a floor turret that fires left when its spy, placed 150px to
//! the left, is touched.

use crate::engine::*;
use crate::icache;
use crate::objects::anim::Anim;
use crate::objects::hs_bulle::HighSpeedBullet;
use crate::objects::spy::Spy;

const STATE_NONE: i32 = 0x00;
const STATE_FIRE: i32 = 0x01;
const STATE_FIRING: i32 = 0x02;
const FIRE_STRENGTH: i32 = 3;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FloorGun {
    pub base: Base,
    anim: Anim,
    spy: Option<ObjId>,
}

impl FloorGun {
    pub fn new(x: i32, y: i32, z: i32) -> FloorGun {
        let mut b = Base::new(x, y, z);
        b.images.push(icache::get("auto-f/shot1.pcx"));
        b.images.push(icache::get("auto-f/shot2.pcx"));
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
        b.me = FRIEND_MINE;
        b.state = STATE_NONE;
        FloorGun { base: b, anim: Anim::default(), spy: None }
    }
}

#[typetag::serde]
impl Obj for FloorGun {
    crate::obj_boilerplate!();

    fn update(&mut self, g: &mut Game) -> Upd {
        let b = &mut self.base;
        if b.counter != 0 {
            b.counter -= 1;
        }
        if b.counter == 0 && b.counter3 == 0 && b.state == STATE_FIRING {
            b.state = STATE_NONE;
        }
        if b.counter3 != 0 {
            b.counter3 -= 1;
        }

        let mut state = b.state;
        if state == STATE_NONE {
            if b.counter3 == 0 {
                if let Some(spy) = self.spy.and_then(|id| g.find::<Spy>(id)) {
                    if spy.get_spy() != 0 {
                        spy.reset_spy();
                        crate::sound::play_sfx("samples/hydraul1.wav");
                        b.state = STATE_FIRE;
                        b.counter3 = 30;
                        b.counter = b.counter3 + 1;
                    }
                }
            }
            b.current_image = 0;
        } else if state == STATE_FIRE {
            if b.counter3 == 0 {
                b.state = STATE_FIRING;
                b.counter = 100;
                self.anim.reset();
                b.current_image = 0;
                state = STATE_FIRING;
            }
        }
        if state == STATE_FIRING {
            if b.current_image == 0 {
                b.current_image = self.anim.next_frame(1, 3);
                if b.current_image != 0 {
                    g.create_object(HighSpeedBullet::new(b.x, b.y + 3, b.z, LEFT_DIR, 0, FIRE_STRENGTH));
                }
            } else {
                b.current_image = self.anim.next_frame(1, 3);
            }
        }

        // Fall or stop
        if g.check_floor(b.x, b.y + b.height, b.z) || g.check_floor(b.x + b.width, b.y + b.height, b.z) {
            b.speed_y = 0;
            if b.counter2 == 0 {
                let spy = Spy::new(b.x - 150, b.y, b.z);
                self.spy = Some(spy.base.id);
                g.create_object(spy);
                b.counter2 = 1;
            }
        } else {
            b.y += b.speed_y;
        }
        Upd::Keep
    }

    fn collision(&mut self, _other: &mut dyn Obj, _g: &mut Game) {}
}
