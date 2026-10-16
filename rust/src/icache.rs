//! Image cache over `urban.dat` (gfx/...). Images are shared and never freed, as in the original.

use crate::dat::{decode_pcx, Datfile};
use crate::gfx::{Bitmap, Palette};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type Img = Rc<Bitmap>;

struct Cache {
    dat: Datfile,
    images: HashMap<String, (Img, Rc<Palette>)>,
}

thread_local! {
    static CACHE: RefCell<Option<Cache>> = const { RefCell::new(None) };
}

pub fn init() {
    let path = crate::sys::data_dir().join("urban.dat");
    let dat = Datfile::open(&path).unwrap_or_else(|e| {
        let msg = crate::sys::missing_data_message();
        eprintln!("{}\n\n({})", msg, e);
        let _ = std::fs::create_dir_all(crate::sys::user_dir().join("data"));
        let _ = sdl2::messagebox::show_simple_message_box(sdl2::messagebox::MessageBoxFlag::ERROR, "URBAN", &msg, None);
        std::process::exit(6)
    });
    CACHE.with(|c| *c.borrow_mut() = Some(Cache { dat, images: HashMap::new() }));
}

/// Loads `gfx/<name>` with its palette. Missing images are fatal, as in the original.
pub fn get_pal(name: &str) -> (Img, Palette) {
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        let c = c.as_mut().expect("icache::init not called");
        let key = format!("gfx/{}", name);
        if let Some((img, pal)) = c.images.get(&key) {
            return (img.clone(), **pal);
        }
        let Some((bmp, pal)) = c.dat.get(&key).and_then(decode_pcx) else {
            eprintln!("Can't load \"{}\"", key);
            std::process::exit(1);
        };
        let img = Rc::new(bmp);
        c.images.insert(key, (img.clone(), Rc::new(pal)));
        (img, pal)
    })
}

pub fn get(name: &str) -> Img {
    get_pal(name).0
}

/// Loads `<prefix><n><suffix>` for n in `first..first+count` (the common sprintf loop).
pub fn seq(prefix: &str, first: i32, count: i32, suffix: &str) -> Vec<Img> {
    (first..first + count).map(|n| get(&format!("{}{}{}", prefix, n, suffix))).collect()
}

pub fn exists(name: &str) -> bool {
    CACHE.with(|c| c.borrow().as_ref().map(|c| c.dat.get(&format!("gfx/{}", name)).is_some()).unwrap_or(false))
}

/// Name of a cached image, found by pointer (used when saving games).
pub fn name_of(img: &Img) -> Option<String> {
    CACHE.with(|c| {
        let c = c.borrow();
        let c = c.as_ref()?;
        c.images.iter().find(|(_, (i, _))| Rc::ptr_eq(i, img)).map(|(k, _)| k.trim_start_matches("gfx/").to_string())
    })
}

/// Serde adapter for `Vec<Img>`: cached images are stored by file name and reloaded from the
/// game data; images made at runtime (rendered text) are stored as raw pixels.
pub mod imgs_serde {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    enum ImgRef {
        Cached(String),
        Raw { w: i32, h: i32, data: Vec<u8> },
    }

    pub fn serialize<S: Serializer>(imgs: &Vec<Img>, s: S) -> Result<S::Ok, S::Error> {
        let refs: Vec<ImgRef> = imgs
            .iter()
            .map(|i| match name_of(i) {
                Some(n) => ImgRef::Cached(n),
                None => ImgRef::Raw { w: i.w, h: i.h, data: i.data.clone() },
            })
            .collect();
        refs.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Img>, D::Error> {
        let refs = Vec::<ImgRef>::deserialize(d)?;
        Ok(refs
            .into_iter()
            .map(|r| match r {
                ImgRef::Cached(n) => get(&n),
                ImgRef::Raw { w, h, data } => Rc::new(Bitmap { w, h, data }),
            })
            .collect())
    }
}
