//! URBAN `.dat` archives ("URBN" magic) and the PCX images stored in them.

use crate::gfx::{Bitmap, Palette, Rgb};
use std::collections::HashMap;
use std::path::Path;

const MAGIC: i32 = 0x5552424E;
const ENTRY_SIZE: usize = 128;

pub struct Datfile {
    data: Vec<u8>,
    entries: HashMap<String, (usize, usize)>,
    order: Vec<String>,
}

fn rd_i32(b: &[u8], off: usize) -> i32 {
    i32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

impl Datfile {
    pub fn open(path: &Path) -> Result<Datfile, String> {
        let data = std::fs::read(path).map_err(|e| format!("{}: {}", path.display(), e))?;
        if data.len() < 12 || rd_i32(&data, 0) != MAGIC {
            return Err(format!("{}: not a URBAN datfile", path.display()));
        }
        let start = rd_i32(&data, 4) as usize;
        let n = rd_i32(&data, 8) as usize;
        let mut entries = HashMap::new();
        let mut order = Vec::new();
        for i in 0..n {
            let e = start + i * ENTRY_SIZE;
            if e + ENTRY_SIZE > data.len() {
                return Err(format!("{}: truncated directory", path.display()));
            }
            let name_bytes = &data[e..e + 120];
            let len = name_bytes.iter().position(|&c| c == 0).unwrap_or(120);
            let name = String::from_utf8_lossy(&name_bytes[..len]).into_owned();
            let size = rd_i32(&data, e + 120) as usize;
            let offset = rd_i32(&data, e + 124) as usize;
            if offset + size > data.len() {
                return Err(format!("{}: entry {} out of range", path.display(), name));
            }
            order.push(name.clone());
            entries.insert(name, (offset, size));
        }
        Ok(Datfile { data, entries, order })
    }

    pub fn get(&self, name: &str) -> Option<&[u8]> {
        self.entries.get(name).map(|&(o, s)| &self.data[o..o + s])
    }

    /// Bytes from the entry's start to the end of the archive (the original read past entry ends).
    pub fn get_open(&self, name: &str) -> Option<&[u8]> {
        self.entries.get(name).map(|&(o, _)| &self.data[o..])
    }

    pub fn names(&self) -> &[String] {
        &self.order
    }
}

/// Decodes an 8-bit single-plane PCX with optional trailing 256-colour palette.
pub fn decode_pcx(b: &[u8]) -> Option<(Bitmap, Palette)> {
    if b.len() < 128 {
        return None;
    }
    let g16 = |o: usize| u16::from_le_bytes([b[o], b[o + 1]]) as i32;
    let width = g16(8) - g16(4) + 1;
    let height = g16(10) - g16(6) + 1;
    let bpl = g16(66).max(width) as usize;
    if width <= 0 || height <= 0 {
        return None;
    }
    let mut bmp = Bitmap::new(width, height);
    let mut p = 128;
    let mut line = vec![0u8; bpl];
    for y in 0..height as usize {
        let mut x = 0;
        while x < bpl {
            let c = *b.get(p)?;
            p += 1;
            if c & 0xC0 == 0xC0 {
                let n = (c & 0x3F) as usize;
                let v = *b.get(p)?;
                p += 1;
                for _ in 0..n {
                    if x < bpl {
                        line[x] = v;
                    }
                    x += 1;
                }
            } else {
                line[x] = c;
                x += 1;
            }
        }
        let row = y * width as usize;
        bmp.data[row..row + width as usize].copy_from_slice(&line[..width as usize]);
    }
    let mut pal = [Rgb::default(); 256];
    if b.len() >= 769 && b[b.len() - 769] == 12 {
        let pb = &b[b.len() - 768..];
        for i in 0..256 {
            pal[i] = Rgb { r: pb[i * 3] >> 2, g: pb[i * 3 + 1] >> 2, b: pb[i * 3 + 2] >> 2 };
        }
    } else if b.get(p) == Some(&12) && b.len() >= p + 769 {
        let pb = &b[p + 1..p + 769];
        for i in 0..256 {
            pal[i] = Rgb { r: pb[i * 3] >> 2, g: pb[i * 3 + 1] >> 2, b: pb[i * 3 + 2] >> 2 };
        }
    }
    Some((bmp, pal))
}
