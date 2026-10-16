//! Outtro sequence (outtro/outtro.cc); unlike the intro it can't be skipped.

use crate::intro::{display_image, play_mod, Player};
use crate::sys;

pub fn run_outtro() {
    let mut p = Player::new(false);
    sys::clear_keybuf();

    play_mod("snd/modules/rock.xm");
    display_image("intro/atom.pcx");
    sys::rest(6000);
    sys::fade_out(5);

    let scenes: [(&str, &str); 11] = [
        ("gfx/intro/12.flc", "A zhame we had to nuke\nzee complex to destroy him...   "),
        ("gfx/intro/9.flc", "Well, at least we \nmanaged to stop him.   "),
        ("gfx/intro/12.flc", "Unfortunately, all zee\nother prototypes were\nlost in zee explozion.   "),
        ("gfx/intro/9.flc", "Yeah. He managed to make\nquite a mess out there.    "),
        ("gfx/intro/8.flc", "Yes. Except for zee\nsmall... malfunction, he\nwas operating quite nizely.    "),
        ("gfx/intro/9.flc", "Lots of money and\nequipment was wasted though.      "),
        ("gfx/intro/8.flc", "Not to worry. Nobody\nwill ever know what\nhappened out zhere.     "),
        ("gfx/intro/9.flc", "That's right. I'll\ntake care of the\ncoverup personally.     "),
        ("gfx/intro/12.flc", "Won't you require\nmore money for zhat?     "),
        ("gfx/intro/9.flc", "No problem. We can\nalways start a new\nwar somewhere...     "),
        ("gfx/intro/8.flc", "Aah, Thiz iz why I love\nthiz country so much.\nOn to zee next project!     "),
    ];
    for (f, t) in scenes {
        if !p.play_flc(f, t) {
            return;
        }
    }

    sys::fade_out(4);
    sys::rest(2000);
    sys::clear_keybuf();
}
