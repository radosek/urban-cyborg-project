//! Global command-line switches (game.h _flags).

use std::sync::atomic::{AtomicBool, Ordering};

static SOUND: AtomicBool = AtomicBool::new(true);
static SOUNDFX: AtomicBool = AtomicBool::new(true);
static BACKGROUND: AtomicBool = AtomicBool::new(true);
static INTRO: AtomicBool = AtomicBool::new(true);
static SPEC_LEVEL: AtomicBool = AtomicBool::new(false);

pub fn sound_on() -> bool { SOUND.load(Ordering::Relaxed) }
pub fn set_sound(v: bool) { SOUND.store(v, Ordering::Relaxed) }
pub fn soundfx_on() -> bool { SOUNDFX.load(Ordering::Relaxed) }
pub fn set_soundfx(v: bool) { SOUNDFX.store(v, Ordering::Relaxed) }
/// `-b` disables backgrounds (the original wired it to the sound-effects flag by mistake).
pub fn background_on() -> bool { BACKGROUND.load(Ordering::Relaxed) }
pub fn set_background(v: bool) { BACKGROUND.store(v, Ordering::Relaxed) }
pub fn intro_on() -> bool { INTRO.load(Ordering::Relaxed) }
pub fn set_intro(v: bool) { INTRO.store(v, Ordering::Relaxed) }
pub fn spec_level() -> bool { SPEC_LEVEL.load(Ordering::Relaxed) }
pub fn set_spec_level(v: bool) { SPEC_LEVEL.store(v, Ordering::Relaxed) }
