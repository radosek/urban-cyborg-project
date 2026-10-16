//! 8-bit indexed bitmaps and 6-bit VGA palettes, mirroring the Allegro 3 primitives the game uses.

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub type Palette = [Rgb; 256];

pub const BLACK_PALETTE: Palette = [Rgb { r: 0, g: 0, b: 0 }; 256];

#[derive(Clone, Debug)]
pub struct Bitmap {
    pub w: i32,
    pub h: i32,
    pub data: Vec<u8>,
}

impl Bitmap {
    pub fn new(w: i32, h: i32) -> Bitmap {
        let w = w.max(0);
        let h = h.max(0);
        Bitmap { w, h, data: vec![0; (w * h) as usize] }
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32) -> u8 {
        self.data[(y * self.w + x) as usize]
    }

    pub fn getpixel(&self, x: i32, y: i32) -> i32 {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            -1
        } else {
            self.get(x, y) as i32
        }
    }

    pub fn putpixel(&mut self, x: i32, y: i32, c: i32) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.data[(y * self.w + x) as usize] = c as u8;
        }
    }

    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    pub fn clear_to_color(&mut self, c: i32) {
        self.data.fill(c as u8);
    }

    pub fn rectfill(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: i32) {
        let (mut x1, mut x2) = if x2 < x1 { (x2, x1) } else { (x1, x2) };
        let (mut y1, mut y2) = if y2 < y1 { (y2, y1) } else { (y1, y2) };
        x1 = x1.max(0);
        y1 = y1.max(0);
        x2 = x2.min(self.w - 1);
        y2 = y2.min(self.h - 1);
        if x1 > x2 || y1 > y2 {
            return;
        }
        for y in y1..=y2 {
            let row = (y * self.w) as usize;
            self.data[row + x1 as usize..=row + x2 as usize].fill(c as u8);
        }
    }

    pub fn hline(&mut self, x1: i32, y: i32, x2: i32, c: i32) {
        self.rectfill(x1, y, x2, y, c);
    }

    pub fn vline(&mut self, x: i32, y1: i32, y2: i32, c: i32) {
        self.rectfill(x, y1, x, y2, c);
    }

    pub fn rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: i32) {
        self.hline(x1, y1, x2, c);
        self.hline(x1, y2, x2, c);
        self.vline(x1, y1, y2, c);
        self.vline(x2, y1, y2, c);
    }

    pub fn line(&mut self, mut x1: i32, mut y1: i32, x2: i32, y2: i32, c: i32) {
        let dx = (x2 - x1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let dy = -(y2 - y1).abs();
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.putpixel(x1, y1, c);
            if x1 == x2 && y1 == y2 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x1 += sx;
            }
            if e2 <= dx {
                err += dx;
                y1 += sy;
            }
        }
    }

    pub fn sub(&self, x: i32, y: i32, w: i32, h: i32) -> Bitmap {
        let mut b = Bitmap::new(w, h);
        blit(self, &mut b, x, y, 0, 0, w, h);
        b
    }
}

/// Clips a source/destination rectangle pair; returns false when nothing remains.
fn clip(src: &Bitmap, dst: &Bitmap, sx: &mut i32, sy: &mut i32, dx: &mut i32, dy: &mut i32, w: &mut i32, h: &mut i32) -> bool {
    if *sx < 0 {
        *w += *sx;
        *dx -= *sx;
        *sx = 0;
    }
    if *sy < 0 {
        *h += *sy;
        *dy -= *sy;
        *sy = 0;
    }
    *w = (*w).min(src.w - *sx);
    *h = (*h).min(src.h - *sy);
    if *dx < 0 {
        *w += *dx;
        *sx -= *dx;
        *dx = 0;
    }
    if *dy < 0 {
        *h += *dy;
        *sy -= *dy;
        *dy = 0;
    }
    *w = (*w).min(dst.w - *dx);
    *h = (*h).min(dst.h - *dy);
    *w > 0 && *h > 0
}

pub fn blit(src: &Bitmap, dst: &mut Bitmap, mut sx: i32, mut sy: i32, mut dx: i32, mut dy: i32, mut w: i32, mut h: i32) {
    if !clip(src, dst, &mut sx, &mut sy, &mut dx, &mut dy, &mut w, &mut h) {
        return;
    }
    for y in 0..h {
        let s = ((sy + y) * src.w + sx) as usize;
        let d = ((dy + y) * dst.w + dx) as usize;
        dst.data[d..d + w as usize].copy_from_slice(&src.data[s..s + w as usize]);
    }
}

pub fn masked_blit(src: &Bitmap, dst: &mut Bitmap, mut sx: i32, mut sy: i32, mut dx: i32, mut dy: i32, mut w: i32, mut h: i32) {
    if !clip(src, dst, &mut sx, &mut sy, &mut dx, &mut dy, &mut w, &mut h) {
        return;
    }
    for y in 0..h {
        let s = ((sy + y) * src.w + sx) as usize;
        let d = ((dy + y) * dst.w + dx) as usize;
        for x in 0..w as usize {
            let c = src.data[s + x];
            if c != 0 {
                dst.data[d + x] = c;
            }
        }
    }
}

/// Draws a whole sprite with colour 0 transparent.
pub fn draw(dst: &mut Bitmap, sprite: &Bitmap, x: i32, y: i32) {
    masked_blit(sprite, dst, 0, 0, x, y, sprite.w, sprite.h);
}

fn stretch(src: &Bitmap, dst: &mut Bitmap, sx: i32, sy: i32, sw: i32, sh: i32, dx: i32, dy: i32, dw: i32, dh: i32, masked: bool) {
    if sw <= 0 || sh <= 0 || dw <= 0 || dh <= 0 {
        return;
    }
    for y in 0..dh {
        let ty = dy + y;
        if ty < 0 || ty >= dst.h {
            continue;
        }
        let syy = sy + (y as i64 * sh as i64 / dh as i64) as i32;
        if syy < 0 || syy >= src.h {
            continue;
        }
        for x in 0..dw {
            let tx = dx + x;
            if tx < 0 || tx >= dst.w {
                continue;
            }
            let sxx = sx + (x as i64 * sw as i64 / dw as i64) as i32;
            if sxx < 0 || sxx >= src.w {
                continue;
            }
            let c = src.get(sxx, syy);
            if !masked || c != 0 {
                dst.data[(ty * dst.w + tx) as usize] = c;
            }
        }
    }
}

pub fn stretch_blit(src: &Bitmap, dst: &mut Bitmap, sx: i32, sy: i32, sw: i32, sh: i32, dx: i32, dy: i32, dw: i32, dh: i32) {
    stretch(src, dst, sx, sy, sw, sh, dx, dy, dw, dh, false);
}

pub fn stretch_sprite(dst: &mut Bitmap, sprite: &Bitmap, x: i32, y: i32, w: i32, h: i32) {
    stretch(sprite, dst, 0, 0, sprite.w, sprite.h, x, y, w, h, true);
}

/// Allegro 3.11 fade_interpolate (gfx.c), including its (63 - pos) / 64 weighting.
pub fn fade_interpolate(src: &Palette, dst: &Palette, out: &mut Palette, pos: i32, from: usize, to: usize) {
    let pos = pos.clamp(0, 64);
    for i in from..=to.min(255) {
        let f = |a: u8, b: u8| ((a as i32 * (63 - pos) + b as i32 * pos) / 64) as u8;
        out[i] = Rgb { r: f(src[i].r, dst[i].r), g: f(src[i].g, dst[i].g), b: f(src[i].b, dst[i].b) };
    }
}
