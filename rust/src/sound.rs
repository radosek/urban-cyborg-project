//! Music (XM via xmrsplayer) and sound effects (WAV) on top of the sys mixer.

use crate::sys::{self, Sample};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
struct Cache {
    samples: HashMap<String, Option<Arc<Sample>>>,
    modules: HashMap<String, Option<&'static xmrs::prelude::Module>>,
}

thread_local! {
    static CACHE: RefCell<Cache> = RefCell::new(Cache::default());
}

fn load_wav(path: &std::path::Path) -> Option<Sample> {
    let mut r = hound::WavReader::open(path).ok()?;
    let spec = r.spec();
    let ch = spec.channels.max(1) as usize;
    let raw: Vec<i32> = match (spec.sample_format, spec.bits_per_sample) {
        (hound::SampleFormat::Int, 8) => r.samples::<i8>().filter_map(Result::ok).map(|s| (s as i32) << 8).collect(),
        (hound::SampleFormat::Int, 16) => r.samples::<i16>().filter_map(Result::ok).map(|s| s as i32).collect(),
        (hound::SampleFormat::Int, b) => r.samples::<i32>().filter_map(Result::ok).map(|s| s >> (b - 16)).collect(),
        (hound::SampleFormat::Float, _) => r.samples::<f32>().filter_map(Result::ok).map(|s| (s * 32767.0) as i32).collect(),
    };
    let data = raw.chunks(ch).map(|c| (c.iter().sum::<i32>() / ch as i32) as i16).collect();
    Some(Sample { data, rate: spec.sample_rate })
}

fn sample(name: &str) -> Option<Arc<Sample>> {
    CACHE.with(|c| {
        c.borrow_mut()
            .samples
            .entry(name.to_string())
            .or_insert_with(|| load_wav(&sys::data_dir().join("snd").join(name)).map(Arc::new))
            .clone()
    })
}

fn module(path: &str) -> Option<&'static xmrs::prelude::Module> {
    CACHE.with(|c| {
        *c.borrow_mut().modules.entry(path.to_string()).or_insert_with(|| {
            let bytes = std::fs::read(sys::data_dir().join(path)).ok()?;
            match xmrs::prelude::Module::load(&bytes) {
                Ok(m) => Some(&*Box::leak(Box::new(m))),
                Err(e) => {
                    eprintln!("{}: {:?}", path, e);
                    None
                }
            }
        })
    })
}

/// Plays a module given relative to the data dir, e.g. "snd/modules/nole.xm".
pub fn play_music(path: &str) -> bool {
    if !sys::audio_enabled() {
        return false;
    }
    match module(path) {
        Some(m) => {
            sys::play_music(m);
            true
        }
        None => false,
    }
}

pub fn stop_music() {
    sys::stop_music();
}

pub fn set_music_volume(v: i32) {
    sys::set_music_volume(v);
}

pub fn music_volume() -> i32 {
    sys::music_volume()
}

fn sfx_vol() -> i32 {
    crate::config::get().sfx_vol
}

/// Plays `snd/<name>` unless sound effects are disabled.
pub fn play_sfx(name: &str) {
    if !crate::flags::soundfx_on() {
        return;
    }
    if let Some(s) = sample(name) {
        sys::play_sample(&s, sfx_vol(), false);
    }
}

pub fn play_sfx_critical(name: &str) {
    if !crate::flags::soundfx_on() {
        return;
    }
    if let Some(s) = sample(name) {
        sys::play_sample(&s, sfx_vol(), true);
    }
}

pub fn load_sfx(name: &str) {
    let _ = sample(name);
}
