//! FLI/FLC player for the intro and outtro, following Allegro 3.11's `play_memory_fli`.
//!
//! Chunk decoding is bounds-checked: reads past the end of a chunk yield zeros (Allegro's
//! FLI_KLUDGE), and writes that would leave the frame bitmap stop the chunk.

use crate::gfx::{self, Bitmap, Palette, BLACK_PALETTE};
use crate::sys;
use std::time::{Duration, Instant};

const HEADER_SIZE: usize = 128;
const FRAME_SIZE: usize = 16;
const CHUNK_SIZE: usize = 6;
const FLI_MAGIC1: u16 = 0xAF11;
const FLI_MAGIC2: u16 = 0xAF12;
const FRAME_MAGIC: u16 = 0xF1FA;
const FRAME_PREFIX: u16 = 0xF100;
const FRAME_USELESS: u16 = 0x00A1;

pub const FLI_OK: i32 = 0;
pub const FLI_ERROR: i32 = -2;

struct Rd<'a> {
    b: &'a [u8],
    p: usize,
}

impl Rd<'_> {
    fn u8(&mut self) -> u8 {
        let v = self.b.get(self.p).copied().unwrap_or(0);
        self.p += 1;
        v
    }
    fn i8(&mut self) -> i32 {
        self.u8() as i8 as i32
    }
    fn u16(&mut self) -> u16 {
        self.u8() as u16 | (self.u8() as u16) << 8
    }
    fn i16(&mut self) -> i32 {
        self.u16() as i16 as i32
    }
    fn u32(&mut self) -> u32 {
        self.u16() as u32 | (self.u16() as u32) << 16
    }
}

pub struct Fli<'a> {
    data: &'a [u8],
    pos: usize,
    pub frame_count: i32,
    pub speed_ms: u64,
    frame: i32,
    pub bitmap: Bitmap,
    pub pal: Palette,
    bmp_dirty: (i32, i32),
    pal_dirty: (i32, i32),
    status: i32,
}

impl<'a> Fli<'a> {
    pub fn open(data: &'a [u8]) -> Option<Fli<'a>> {
        if data.len() < HEADER_SIZE {
            return None;
        }
        let mut r = Rd { b: data, p: 0 };
        let size = r.u32();
        let magic = r.u16();
        let frame_count = r.u16() as i32;
        let width = r.u16() as i32;
        let height = r.u16() as i32;
        let _bits = r.u16();
        let _flags = r.u16();
        let speed = r.u16() as u64;
        if (size as usize) < HEADER_SIZE || (magic != FLI_MAGIC1 && magic != FLI_MAGIC2) || width == 0 || height == 0 {
            return None;
        }
        let speed_ms = if magic == FLI_MAGIC1 { speed * 1000 / 70 } else { speed };
        Some(Fli {
            data,
            pos: HEADER_SIZE,
            frame_count,
            speed_ms: if speed_ms == 0 { 1000 / 70 } else { speed_ms },
            frame: 0,
            bitmap: Bitmap::new(width, height),
            pal: BLACK_PALETTE,
            bmp_dirty: (i32::MAX, i32::MIN),
            pal_dirty: (i32::MAX, i32::MIN),
            status: FLI_OK,
        })
    }

    fn reset_dirty(&mut self) {
        self.bmp_dirty = (i32::MAX, i32::MIN);
        self.pal_dirty = (i32::MAX, i32::MIN);
    }

    fn dirty_rows(&mut self, from: i32, to: i32) {
        self.bmp_dirty = (self.bmp_dirty.0.min(from), self.bmp_dirty.1.max(to));
    }

    /// Decodes the next frame; with `looping` it rewinds to the first frame at the end.
    /// Returns false at the end of a non-looping animation or on a broken stream.
    pub fn next_frame(&mut self, looping: bool) -> bool {
        if self.status != FLI_OK {
            return false;
        }
        if self.frame >= self.frame_count {
            if !looping {
                return false;
            }
            self.pos = HEADER_SIZE;
            self.frame = 0;
        }
        self.read_frame();
        self.status == FLI_OK
    }

    fn read_frame(&mut self) {
        if self.frame == 0 {
            self.bitmap.clear();
        }
        loop {
            if self.pos + FRAME_SIZE > self.data.len() {
                self.status = FLI_ERROR;
                return;
            }
            let mut r = Rd { b: self.data, p: self.pos };
            let size = r.u32() as usize;
            let ftype = r.u16();
            let chunks = r.u16();
            if size < FRAME_SIZE {
                self.status = FLI_ERROR;
                return;
            }
            if ftype == FRAME_PREFIX || ftype == FRAME_USELESS {
                self.pos += size;
                self.frame += 1;
                if self.frame >= self.frame_count {
                    return;
                }
                continue;
            }
            if ftype != FRAME_MAGIC {
                self.status = FLI_ERROR;
                return;
            }
            let start = self.pos + FRAME_SIZE;
            let mut frame_size = size - FRAME_SIZE;
            self.pos += size;
            if frame_size == 0 {
                self.frame += 1;
                return;
            }
            if start + frame_size > self.data.len() {
                self.status = FLI_ERROR;
                return;
            }
            let body = &self.data[start..start + frame_size];
            let mut p = 0usize;
            for _ in 0..chunks {
                if frame_size < CHUNK_SIZE {
                    break;
                }
                let mut r = Rd { b: body, p };
                let csize = r.u32() as usize;
                let ctype = r.u16();
                if csize < CHUNK_SIZE || csize > frame_size {
                    break;
                }
                let c = &body[p + CHUNK_SIZE..(p + csize).min(body.len())];
                frame_size -= csize;
                match ctype {
                    4 => self.color(c, 4),
                    7 => self.delta_flc(c),
                    11 => self.color(c, 1),
                    12 => self.delta_fli(c),
                    13 => {
                        self.bitmap.clear();
                        let h = self.bitmap.h;
                        self.dirty_rows(0, h - 1);
                    }
                    15 => self.brun(c),
                    16 => self.copy(c),
                    _ => {}
                }
                p += csize;
            }
            self.frame += 1;
            return;
        }
    }

    fn color(&mut self, c: &[u8], div: u8) {
        let mut r = Rd { b: c, p: 0 };
        let mut packets = r.i16();
        let mut offset = 0usize;
        while packets > 0 {
            packets -= 1;
            offset += r.u8() as usize;
            let mut length = r.u8() as usize;
            if length == 0 {
                length = 256;
            }
            let end = offset + length;
            if end > 256 {
                return;
            }
            self.pal_dirty = (self.pal_dirty.0.min(offset as i32), self.pal_dirty.1.max(end as i32 - 1));
            while offset < end {
                self.pal[offset].r = r.u8() / div;
                self.pal[offset].g = r.u8() / div;
                self.pal[offset].b = r.u8() / div;
                offset += 1;
            }
        }
    }

    fn delta_flc(&mut self, c: &[u8]) {
        let (w, h) = (self.bitmap.w, self.bitmap.h);
        let end = (w * h) as usize;
        let mut r = Rd { b: c, p: 0 };
        let mut lines = r.i16();
        let mut y = 0i32;
        while lines > 0 {
            lines -= 1;
            if r.p >= c.len() {
                return;
            }
            let mut packets = r.i16();
            while packets < 0 {
                if packets & 0x4000 != 0 {
                    y -= packets;
                } else if y < h {
                    self.bitmap.data[(y * w + w - 1) as usize] = (packets & 0xff) as u8;
                }
                if r.p >= c.len() {
                    return;
                }
                packets = r.i16();
            }
            if y >= h {
                return;
            }
            self.dirty_rows(y, y);
            let mut curr = (y * w) as usize;
            while packets > 0 {
                packets -= 1;
                curr += r.u8() as usize;
                let size = r.i8();
                if size > 0 {
                    let n = size as usize * 2;
                    if curr + n > end {
                        return;
                    }
                    for i in 0..n {
                        self.bitmap.data[curr + i] = r.u8();
                    }
                    curr += n;
                } else if size < 0 {
                    let n = (-size) as usize;
                    if curr + n * 2 > end {
                        return;
                    }
                    let (v1, v2) = (r.u8(), r.u8());
                    for i in 0..n {
                        self.bitmap.data[curr + i * 2] = v1;
                        self.bitmap.data[curr + i * 2 + 1] = v2;
                    }
                    curr += n * 2;
                }
            }
            y += 1;
        }
    }

    fn delta_fli(&mut self, c: &[u8]) {
        let (w, h) = (self.bitmap.w, self.bitmap.h);
        let end = (w * h) as usize;
        let mut r = Rd { b: c, p: 0 };
        let mut y = r.u16() as i32;
        let mut lines = r.i16();
        if y >= h {
            return;
        }
        if y + lines > h {
            lines = h - y;
        }
        self.dirty_rows(y, y + lines - 1);
        while lines > 0 {
            lines -= 1;
            let mut packets = r.u8();
            let mut curr = (y * w) as usize;
            while packets > 0 {
                packets -= 1;
                curr += r.u8() as usize;
                let size = r.i8();
                if size > 0 {
                    let n = size as usize;
                    if curr + n > end {
                        return;
                    }
                    for i in 0..n {
                        self.bitmap.data[curr + i] = r.u8();
                    }
                    curr += n;
                } else if size < 0 {
                    let n = (-size) as usize;
                    if curr + n > end {
                        return;
                    }
                    let v = r.u8();
                    self.bitmap.data[curr..curr + n].fill(v);
                    curr += n;
                }
            }
            y += 1;
        }
    }

    fn brun(&mut self, c: &[u8]) {
        let (w, h) = (self.bitmap.w, self.bitmap.h);
        let end = (w * h) as usize;
        let mut r = Rd { b: c, p: 0 };
        self.dirty_rows(0, h - 1);
        for y in 0..h {
            let mut packets = r.u8() as i32;
            let mut curr = (y * w) as usize;
            let line_end = curr + w as usize;
            // FLC files store 0 here and fill the whole line; FLI files give a packet count.
            let flc = packets == 0;
            while if flc { curr < line_end } else { packets > 0 } {
                packets -= 1;
                if r.p >= c.len() {
                    return;
                }
                let size = r.i8();
                if size < 0 {
                    let n = (-size) as usize;
                    if curr + n > end {
                        return;
                    }
                    for i in 0..n {
                        self.bitmap.data[curr + i] = r.u8();
                    }
                    curr += n;
                } else if size > 0 {
                    let n = size as usize;
                    if curr + n > end {
                        return;
                    }
                    let v = r.u8();
                    self.bitmap.data[curr..curr + n].fill(v);
                    curr += n;
                }
            }
        }
    }

    fn copy(&mut self, c: &[u8]) {
        let n = self.bitmap.data.len();
        if c.len() < n {
            return;
        }
        self.bitmap.data.copy_from_slice(&c[..n]);
        let h = self.bitmap.h;
        self.dirty_rows(0, h - 1);
    }

    /// Pushes the changed palette range and bitmap rows to the screen.
    fn show(&mut self) {
        if self.pal_dirty.0 <= self.pal_dirty.1 {
            for i in self.pal_dirty.0..=self.pal_dirty.1 {
                sys::set_color(i as usize, self.pal[i as usize]);
            }
        }
        if self.bmp_dirty.0 <= self.bmp_dirty.1 {
            let (from, to) = self.bmp_dirty;
            let b = &self.bitmap;
            sys::with_screen(|s| gfx::blit(b, s, 0, from, 0, from, b.w, 1 + to - from));
        }
        sys::present(true);
        self.reset_dirty();
    }
}

/// `play_memory_fli(data, screen, loop, callback)`: the callback runs after every displayed frame
/// and stops playback by returning true. Frames are paced by the file's speed, as Allegro's
/// timer interrupt did (it starts two ticks ahead).
pub fn play_memory_fli(data: &[u8], looping: bool, mut callback: impl FnMut() -> bool) -> i32 {
    let Some(mut fli) = Fli::open(data) else { return FLI_ERROR };
    let start = Instant::now();
    let period = Duration::from_millis(fli.speed_ms);
    let mut decoded: i64 = 0;
    let timer = |decoded: i64| 2 + (start.elapsed().as_nanos() / period.as_nanos()) as i64 - decoded;
    decoded += 1;
    let mut ok = fli.next_frame(looping);
    while ok {
        fli.show();
        if callback() {
            break;
        }
        decoded += 1;
        ok = fli.next_frame(looping);
        while timer(decoded) <= 0 && std::env::var_os("URBAN_FLI_TEST").is_none() {
            sys::pump();
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    if fli.status == FLI_OK { FLI_OK } else { fli.status }
}

#[cfg(test)]
mod tests {
    #[test]
    fn decodes_intro_animations() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../ref/data/intro.dat");
        let Ok(dat) = crate::dat::Datfile::open(&path) else { return };
        let names: Vec<_> = dat.names().iter().filter(|n| n.ends_with(".flc") || n.ends_with(".fli")).collect();
        assert_eq!(names.len(), 11);
        for name in names {
            let mut f = super::Fli::open(dat.get(name).unwrap()).expect(name);
            for _ in 0..f.frame_count {
                assert!(f.next_frame(false), "{}", name);
            }
            assert!(f.bitmap.data.iter().any(|&p| p != 0), "{}", name);
            assert!(f.pal.iter().any(|c| c.r != 0 || c.g != 0 || c.b != 0), "{}", name);
        }
    }
}
