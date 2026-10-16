//! Level maps ("URBAN MAP 1.0" text format inside levels.dat).

use crate::dat::Datfile;
use crate::gfx::{self, Palette, Rgb, BLACK_PALETTE};
use crate::icache::{self, Img};
use crate::tiles::{BG_TILES, TILES};

pub const MAX_TILES: usize = 300;
pub const MAX_OPTIONS: usize = 50;
pub const BG_TILE_WIDTH: i32 = 120;
pub const BG_TILE_HEIGHT: i32 = 120;
pub const PAL_AIR: i32 = 0;
pub const PAL_WATER: i32 = 1;
const BLUE_LEVEL: i32 = 30;

pub const TRANSPARENT_TILE: i32 = 4096;
pub const FAKE_TILE: i32 = 8192;

struct Lines<'a> {
    lines: Vec<&'a str>,
    pos: usize,
    name: &'a str,
}

impl<'a> Lines<'a> {
    fn next(&mut self) -> Result<&'a str, String> {
        let l = self.lines.get(self.pos).copied().ok_or_else(|| format!("{}: unexpected end of map", self.name))?;
        self.pos += 1;
        Ok(l)
    }
}

pub struct Map {
    dat: Datfile,
    sections: [Vec<i32>; 3],
    objects: [Vec<i32>; 3],
    background: Vec<i32>,
    tiles: Vec<Img>,
    tile_info: Vec<i32>,
    bg_tiles: Vec<Img>,
    options: Vec<String>,
    pub x_size: i32,
    pub y_size: i32,
    bg_x_size: i32,
    bg_y_size: i32,
    pal_count: i32,
    pal_type: i32,
    pal: Palette,
    pal_water: Palette,
    pal_air: Palette,
}

impl Map {
    pub fn new() -> Map {
        let dat = Datfile::open(&crate::sys::data_dir().join("levels.dat")).unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(6)
        });
        let (_, pal_air) = icache::get_pal("tiles/desk10.pcx");
        let blue = [Rgb { r: 0, g: 0, b: 63 }; 256];
        let mut pal_water = BLACK_PALETTE;
        gfx::fade_interpolate(&pal_air, &blue, &mut pal_water, BLUE_LEVEL, 0, 255);
        Map {
            dat,
            sections: Default::default(),
            objects: Default::default(),
            background: Vec::new(),
            tiles: Vec::new(),
            tile_info: Vec::new(),
            bg_tiles: Vec::new(),
            options: Vec::new(),
            x_size: 0,
            y_size: 0,
            bg_x_size: 0,
            bg_y_size: 0,
            pal_count: 0,
            pal_type: PAL_AIR,
            pal: pal_air,
            pal_water,
            pal_air,
        }
    }

    /// Loads a map from levels.dat, falling back to the filesystem (relative to the data dir).
    pub fn load(&mut self, filename: &str) -> Result<(), String> {
        let text = match self.dat.get(filename) {
            Some(b) => b.to_vec(),
            None => std::fs::read(crate::sys::data_dir().join(filename))
                .or_else(|_| std::fs::read(filename))
                .map_err(|e| format!("{}: {}", filename, e))?,
        };
        let text = String::from_utf8_lossy(&text).into_owned();
        // Blank lines and '#' comments are skipped; CRLF is tolerated (the original kept a stray '\r').
        let lines: Vec<&str> = text.lines().map(|l| l.trim_end_matches('\r')).filter(|l| !l.is_empty() && !l.starts_with('#')).collect();
        let mut cur = Lines { lines, pos: 0, name: filename };
        let next = &mut cur;

        if !next.next()?.starts_with("URBAN MAP 1.0") {
            return Err(format!("{}: not an URBAN map", filename));
        }
        let size = |l: &str| l.get(7..).unwrap_or("").trim().parse::<i32>().unwrap_or(0);
        let x_size = size(next.next()?);
        let y_size = size(next.next()?);
        if x_size <= 0 || y_size <= 0 || x_size > 4096 || y_size > 4096 {
            return Err(format!("{}: bad map size {}x{}", filename, x_size, y_size));
        }
        self.x_size = x_size;
        self.y_size = y_size;
        self.bg_x_size = x_size * crate::engine::TILE_WIDTH / BG_TILE_WIDTH + 1;
        self.bg_y_size = (y_size * crate::engine::TILE_SIDE_HEIGHT / BG_TILE_HEIGHT + 1).max(4);
        let n = (x_size * y_size) as usize;
        self.sections = [vec![0; n], vec![0; n], vec![0; n]];
        self.objects = [vec![0; n], vec![0; n], vec![0; n]];
        self.background = vec![0; (self.bg_x_size * self.bg_y_size) as usize];
        self.options.clear();

        self.tiles = TILES.iter().map(|(_, f)| icache::get(f)).collect();
        self.tile_info = TILES.iter().map(|(i, _)| *i).collect();
        self.bg_tiles = BG_TILES.iter().map(|f| icache::get(f)).collect();

        fn parse_grid(next: &mut Lines, w: i32, h: i32, dst: &mut [i32], end: &str) -> Result<(), String> {
            for y in 0..h {
                let line = next.next()?;
                for (x, tok) in line.split([' ', ',']).filter(|t| !t.is_empty()).enumerate() {
                    if (x as i32) < w {
                        dst[(y * w) as usize + x] = i32::from_str_radix(tok.trim(), 16).unwrap_or(0);
                    }
                }
            }
            let close = next.next()?;
            if close.split_whitespace().next() != Some(end) {
                return Err(format!("expected {}", end));
            }
            Ok(())
        }

        loop {
            let Ok(line) = next.next() else { break };
            if line == "END" {
                break;
            }
            match line {
                "<TILES>" => loop {
                    let l = next.next()?;
                    if l == "</TILES>" {
                        break;
                    }
                    if self.tiles.len() >= MAX_TILES {
                        return Err(format!("{}: too many tiles", filename));
                    }
                    let name = l.get(5..).unwrap_or("").trim();
                    self.tiles.push(icache::get(name));
                    self.tile_info.push(i32::from_str_radix(l.get(..4).unwrap_or("0").trim(), 16).unwrap_or(0));
                },
                "<BGTILES>" => loop {
                    let l = next.next()?;
                    if l == "</BGTILES>" {
                        break;
                    }
                    self.bg_tiles.push(icache::get(l.get(5..).unwrap_or("").trim()));
                },
                "<SECTION_0>" | "<SECTION_1>" | "<SECTION_2>" => {
                    let z = (line.as_bytes()[9] - b'0') as usize;
                    let end = format!("</SECTION_{}>", z);
                    parse_grid(next, x_size, y_size, &mut self.sections[z], &end)?;
                }
                "<OBJECT_0>" | "<OBJECT_1>" | "<OBJECT_2>" => {
                    let z = (line.as_bytes()[8] - b'0') as usize;
                    let end = format!("</OBJECT_{}>", z);
                    parse_grid(next, x_size, y_size, &mut self.objects[z], &end)?;
                }
                "<BACKGROUND>" => {
                    let (w, h) = (self.bg_x_size, self.bg_y_size);
                    parse_grid(next, w, h, &mut self.background, "</BACKGROUND>")?;
                }
                "<OPTIONS>" => loop {
                    let l = next.next()?;
                    if l == "</OPTIONS>" {
                        break;
                    }
                    if self.options.len() < MAX_OPTIONS {
                        self.options.push(l.to_string());
                    }
                },
                _ => {}
            }
        }
        Ok(())
    }

    pub fn width(&self) -> i32 {
        self.x_size
    }

    pub fn height(&self) -> i32 {
        self.y_size
    }

    fn idx(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        if x < 0 || y < 0 || z < 0 || x >= self.x_size || y >= self.y_size || z > 2 {
            None
        } else {
            Some((x + y * self.x_size) as usize)
        }
    }

    pub fn get_bg(&self, x: i32, y: i32, z: i32) -> i32 {
        self.idx(x, y, z).map(|i| self.sections[z as usize][i]).unwrap_or(0)
    }

    pub fn set_bg(&mut self, x: i32, y: i32, z: i32, v: i32) {
        if let Some(i) = self.idx(x, y, z) {
            self.sections[z as usize][i] = v;
        }
    }

    pub fn get_obj(&self, x: i32, y: i32, z: i32) -> i32 {
        self.idx(x, y, z).map(|i| self.objects[z as usize][i]).unwrap_or(0)
    }

    pub fn get_background(&self, x: i32, y: i32) -> i32 {
        if x < 0 || y < 0 || x >= self.bg_x_size || y >= self.bg_y_size {
            return 0;
        }
        self.background[(x + y * self.bg_x_size) as usize]
    }

    /// Tile image for a 0-based tile number; out-of-range numbers fall back to tile 0 (original hack).
    pub fn tile(&self, num: i32) -> &Img {
        self.tiles.get(num as usize).unwrap_or(&self.tiles[0])
    }

    pub fn bg_tile(&self, num: i32) -> &Img {
        self.bg_tiles.get(num as usize).unwrap_or(&self.bg_tiles[0])
    }

    /// Flags for a 1-based map cell value; 0 (empty) and out-of-range have no flags.
    pub fn cell_info(&self, cell: i32) -> i32 {
        if cell <= 0 {
            return 0;
        }
        self.tile_info.get((cell - 1) as usize).copied().unwrap_or(0)
    }

    pub fn is_transparent(&self, cell: i32) -> bool {
        self.cell_info(cell) & TRANSPARENT_TILE != 0
    }

    pub fn is_fake(&self, cell: i32) -> bool {
        self.cell_info(cell) & FAKE_TILE != 0
    }

    pub fn option(&self, name: &str) -> Option<&str> {
        self.options.iter().find_map(|o| {
            let mut it = o.splitn(2, '=');
            (it.next()? == name).then(|| it.next().unwrap_or(""))
        })
    }

    pub fn pal(&self) -> &Palette {
        &self.pal
    }

    pub fn pal_mut(&mut self) -> &mut Palette {
        &mut self.pal
    }

    pub fn set_pal(&mut self, t: i32) {
        self.pal_type = t;
    }

    pub fn pal_type(&self) -> i32 {
        self.pal_type
    }

    /// Steps the air/water palette crossfade by one frame.
    pub fn update_pal(&mut self) {
        if self.pal_type == PAL_AIR && self.pal_count > 0 {
            gfx::fade_interpolate(&self.pal_air, &self.pal_water, &mut self.pal, self.pal_count, 0, 255);
            crate::sys::set_palette(&self.pal);
            self.pal_count -= 1;
        } else if self.pal_type == PAL_WATER && self.pal_count < 64 {
            gfx::fade_interpolate(&self.pal_air, &self.pal_water, &mut self.pal, self.pal_count, 0, 255);
            crate::sys::set_palette(&self.pal);
            self.pal_count += 1;
        }
    }
}
