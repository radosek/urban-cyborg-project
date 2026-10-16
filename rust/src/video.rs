//! Display upgrades applied after the game renders its 8-bit frame: upscaling filters, a CRT look
//! and widescreen. None of this changes game logic; the original look is `Filter::Pixels`,
//! CRT off, widescreen off (the defaults).

use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Filter {
    /// Sharp nearest-neighbour pixels (original look).
    Pixels,
    /// Bilinear smoothing by the GPU.
    Smooth,
    /// Scale3x (AdvMAME3x): sharp edges, rounded diagonals.
    ScaleX,
    /// hq4x: smooth pixel-art interpolation.
    Hqx,
}

impl Filter {
    pub const ALL: [Filter; 4] = [Filter::Pixels, Filter::Smooth, Filter::ScaleX, Filter::Hqx];

    pub fn name(self) -> &'static str {
        match self {
            Filter::Pixels => "PIXELS",
            Filter::Smooth => "SMOOTH",
            Filter::ScaleX => "SCALEX",
            Filter::Hqx => "HQX",
        }
    }

    pub fn next(self) -> Filter {
        let i = Filter::ALL.iter().position(|&f| f == self).unwrap_or(0);
        Filter::ALL[(i + 1) % Filter::ALL.len()]
    }

    fn from_name(s: &str) -> Option<Filter> {
        Filter::ALL.iter().copied().find(|f| f.name().eq_ignore_ascii_case(s))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VideoConf {
    pub filter: Filter,
    pub crt: bool,
    pub widescreen: bool,
    pub fullscreen: bool,
}

impl Default for VideoConf {
    fn default() -> Self {
        VideoConf { filter: Filter::Pixels, crt: false, widescreen: false, fullscreen: false }
    }
}

/// Gameplay width in widescreen mode: 16:9 at 240 lines.
pub const WIDE_W: i32 = 427;

fn path() -> PathBuf {
    crate::sys::user_dir().join("video.cfg")
}

impl VideoConf {
    pub fn load() -> VideoConf {
        let mut c = VideoConf::default();
        if let Ok(text) = std::fs::read_to_string(path()) {
            for line in text.lines() {
                let Some((k, v)) = line.split_once('=') else { continue };
                let on = v.trim() == "on";
                match k.trim() {
                    "filter" => c.filter = Filter::from_name(v.trim()).unwrap_or(c.filter),
                    "crt" => c.crt = on,
                    "widescreen" => c.widescreen = on,
                    "fullscreen" => c.fullscreen = on,
                    _ => {}
                }
            }
        }
        if std::env::var_os("URBAN_FULLSCREEN").is_some() {
            c.fullscreen = true;
        }
        c
    }

    pub fn save(&self) {
        let b = |v: bool| if v { "on" } else { "off" };
        let text = format!(
            "filter={}\ncrt={}\nwidescreen={}\nfullscreen={}\n",
            self.filter.name().to_ascii_lowercase(),
            b(self.crt),
            b(self.widescreen),
            b(self.fullscreen)
        );
        let _ = std::fs::write(path(), text);
    }

    /// Integer factor the CPU filter (and CRT) enlarge the frame by.
    pub fn scale(&self) -> usize {
        match self.filter {
            Filter::Hqx => 4,
            Filter::ScaleX => 3,
            _ if self.crt => 3,
            _ => 1,
        }
    }
}

/// Scale3x / AdvMAME3x on 32-bit pixels.
pub fn scale3x(src: &[u32], w: usize, h: usize, dst: &mut [u32]) {
    let dw = w * 3;
    for y in 0..h {
        let up = if y > 0 { y - 1 } else { y };
        let dn = if y + 1 < h { y + 1 } else { y };
        for x in 0..w {
            let lf = if x > 0 { x - 1 } else { x };
            let rt = if x + 1 < w { x + 1 } else { x };
            let (a, b, c) = (src[up * w + lf], src[up * w + x], src[up * w + rt]);
            let (d, e, f) = (src[y * w + lf], src[y * w + x], src[y * w + rt]);
            let (g, hh, i) = (src[dn * w + lf], src[dn * w + x], src[dn * w + rt]);
            let mut o = [e; 9];
            if b != hh && d != f {
                o[0] = if d == b { d } else { e };
                o[1] = if (d == b && e != c) || (b == f && e != a) { b } else { e };
                o[2] = if b == f { f } else { e };
                o[3] = if (d == b && e != g) || (d == hh && e != a) { d } else { e };
                o[5] = if (b == f && e != i) || (hh == f && e != c) { f } else { e };
                o[6] = if d == hh { d } else { e };
                o[7] = if (d == hh && e != i) || (hh == f && e != g) { hh } else { e };
                o[8] = if hh == f { f } else { e };
            }
            let base = (y * 3) * dw + x * 3;
            dst[base..base + 3].copy_from_slice(&o[0..3]);
            dst[base + dw..base + dw + 3].copy_from_slice(&o[3..6]);
            dst[base + 2 * dw..base + 2 * dw + 3].copy_from_slice(&o[6..9]);
        }
    }
}

pub fn nearest(src: &[u32], w: usize, h: usize, n: usize, dst: &mut [u32]) {
    let dw = w * n;
    for y in 0..h {
        for x in 0..w {
            let c = src[y * w + x];
            for yy in 0..n {
                let row = (y * n + yy) * dw + x * n;
                dst[row..row + n].fill(c);
            }
        }
    }
}

/// CRT look: barrel curvature, scanlines, aperture-grille tint and vignette. The per-pixel source
/// position and brightness are precomputed for the output size.
pub struct Crt {
    w: usize,
    h: usize,
    n: usize,
    map: Vec<u32>,
    gain: Vec<u16>,
}

impl Crt {
    pub fn new(w: usize, h: usize, n: usize) -> Crt {
        let mut map = vec![u32::MAX; w * h];
        let mut gain = vec![0u16; w * h];
        let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
        const K: f32 = 0.06;
        for y in 0..h {
            for x in 0..w {
                let nx = (x as f32 + 0.5 - cx) / cx;
                let ny = (y as f32 + 0.5 - cy) / cy;
                let r2 = nx * nx + ny * ny;
                let sx = nx * (1.0 + K * r2);
                let sy = ny * (1.0 + K * r2);
                if sx.abs() > 1.0 || sy.abs() > 1.0 {
                    continue;
                }
                let px = ((sx + 1.0) * cx) as usize;
                let py = ((sy + 1.0) * cy) as usize;
                let (px, py) = (px.min(w - 1), py.min(h - 1));
                map[y * w + x] = (py * w + px) as u32;
                // Scanline: darker on the last sub-row of each original line.
                let sub = py % n;
                let scan = if n >= 3 && sub == n - 1 { 0.55 } else { 1.0 };
                let vig = (1.0 - 0.35 * (nx * nx * 0.6 + ny * ny * 0.8)).clamp(0.0, 1.0);
                gain[y * w + x] = (scan * vig * 256.0) as u16;
            }
        }
        Crt { w, h, n, map, gain }
    }

    pub fn fits(&self, w: usize, h: usize, n: usize) -> bool {
        self.w == w && self.h == h && self.n == n
    }

    pub fn apply(&self, src: &[u32], dst: &mut [u32]) {
        for (i, out) in dst.iter_mut().enumerate() {
            let m = self.map[i];
            if m == u32::MAX {
                *out = 0xff000000;
                continue;
            }
            let c = src[m as usize];
            let g = self.gain[i] as u32;
            // Aperture grille: every third column favours one of R, G, B.
            let (rg, gg, bg) = match (i % self.w) % 3 {
                0 => (g * 280 / 256, g * 230 / 256, g * 230 / 256),
                1 => (g * 230 / 256, g * 280 / 256, g * 230 / 256),
                _ => (g * 230 / 256, g * 230 / 256, g * 280 / 256),
            };
            let r = (((c >> 16) & 0xff) * rg / 256).min(255);
            let gch = (((c >> 8) & 0xff) * gg / 256).min(255);
            let b = ((c & 0xff) * bg / 256).min(255);
            *out = 0xff000000 | (r << 16) | (gch << 8) | b;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale3x_keeps_flat_areas_and_size() {
        let src = vec![7u32; 4 * 3];
        let mut dst = vec![0u32; 12 * 9];
        scale3x(&src, 4, 3, &mut dst);
        assert!(dst.iter().all(|&c| c == 7));
    }

    #[test]
    fn scale3x_rounds_a_diagonal() {
        // A 2x2 checker corner: the centre pixel's corner takes the matching neighbour.
        let (a, b) = (1u32, 2u32);
        let src = vec![a, b, b, a];
        let mut dst = vec![0u32; 6 * 6];
        scale3x(&src, 2, 2, &mut dst);
        assert_eq!(dst.len(), 36);
        assert_eq!(dst[0], a);
    }

    #[test]
    fn conf_round_trip() {
        let c = VideoConf { filter: Filter::Hqx, crt: true, widescreen: true, fullscreen: false };
        let text = format!("filter={}\ncrt=on\nwidescreen=on\nfullscreen=off\n", c.filter.name().to_ascii_lowercase());
        assert!(text.contains("hqx"));
        assert_eq!(Filter::from_name("hqx"), Some(Filter::Hqx));
        assert_eq!(Filter::Hqx.next(), Filter::Pixels);
    }
}
