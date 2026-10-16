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
        eprintln!("{}", e);
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
