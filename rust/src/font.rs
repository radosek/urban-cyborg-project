//! UrbanFont: bitmap fonts cut from dafont2/dafont3/rsifont.pcx. Text is CP437 bytes.

use crate::gfx::{self, Bitmap, Palette};
use crate::icache::{self, Img};

pub const SMALL_FONT: i32 = 0;
pub const LARGE_FONT: i32 = 1;
pub const SMALL_FONT2: i32 = 2;

struct Info {
    width: i32,
    height: i32,
    space_x: i32,
    space_y: i32,
}

const INFO: [Info; 2] = [
    Info { width: 9, height: 15, space_x: 8, space_y: 15 },
    Info { width: 32, height: 32, space_x: 34, space_y: 41 },
];

const SMALL_CHARS: &[(u8, i32, i32)] = &[
    (b'A', 0, 0), (b'B', 1, 0), (b'C', 2, 0), (b'D', 3, 0), (b'E', 4, 0), (b'F', 5, 0), (b'G', 6, 0),
    (b'H', 7, 0), (b'I', 8, 0), (b'J', 9, 0), (b'K', 10, 0), (b'L', 11, 0), (b'M', 12, 0), (b'N', 13, 0),
    (b'O', 14, 0), (b'P', 15, 0), (b'Q', 16, 0), (b'R', 17, 0), (b'S', 0, 1), (b'T', 1, 1), (b'U', 2, 1),
    (b'V', 3, 1), (b'W', 4, 1), (b'X', 5, 1), (b'Y', 6, 1), (b'Z', 7, 1), (0x8F, 8, 1), (0x8E, 9, 1),
    (0x99, 10, 1), (b'a', 0, 2), (b'b', 1, 2), (b'c', 2, 2), (b'd', 3, 2), (b'e', 4, 2), (b'f', 5, 2),
    (b'g', 6, 2), (b'h', 7, 2), (b'i', 8, 2), (b'j', 9, 2), (b'k', 10, 2), (b'l', 11, 2), (b'm', 12, 2),
    (b'n', 13, 2), (b'o', 14, 2), (b'p', 15, 2), (b'q', 16, 2), (b'r', 17, 2), (b's', 0, 3), (b't', 1, 3),
    (b'u', 2, 3), (b'v', 3, 3), (b'w', 4, 3), (b'x', 5, 3), (b'y', 6, 3), (b'z', 7, 3), (0x86, 8, 3),
    (0x84, 9, 3), (0x94, 10, 3), (b'1', 0, 4), (b'2', 1, 4), (b'3', 2, 4), (b'4', 3, 4), (b'5', 4, 4),
    (b'6', 5, 4), (b'7', 6, 4), (b'8', 7, 4), (b'9', 8, 4), (b'0', 9, 4), (b',', 10, 4), (b'.', 11, 4),
    (b':', 12, 4), (b'%', 13, 4), (b'!', 14, 4), (b'?', 15, 4), (b'(', 16, 4), (b')', 17, 4), (b'\'', 0, 5),
];

const LARGE_CHARS: &[(u8, i32, i32)] = &[
    (b'A', 0, 0), (b'B', 1, 0), (b'C', 2, 0), (b'D', 3, 0), (b'E', 4, 0), (b'F', 5, 0), (b'G', 6, 0),
    (b'H', 7, 0), (b'I', 8, 0), (b'J', 9, 0), (b'K', 0, 1), (b'L', 1, 1), (b'M', 2, 1), (b'N', 3, 1),
    (b'O', 4, 1), (b'P', 5, 1), (b'Q', 6, 1), (b'R', 7, 1), (b'S', 8, 1), (b'T', 9, 1), (b'U', 0, 2),
    (b'V', 1, 2), (b'W', 2, 2), (b'X', 3, 2), (b'Y', 4, 2), (b'Z', 5, 2), (b':', 6, 2), (b'-', 7, 2),
];

pub struct UrbanFont {
    image: Img,
    pal: Palette,
    nr: usize,
    scale: i32,
    space_x: i32,
    space_y: i32,
    width: i32,
    height: i32,
}

/// Maps Rust text to the CP437 bytes the font tables use (Swedish letters included).
pub fn cp437(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| match c {
            'Å' => 0x8F,
            'Ä' => 0x8E,
            'Ö' => 0x99,
            'å' => 0x86,
            'ä' => 0x84,
            'ö' => 0x94,
            c if (c as u32) < 128 => c as u8,
            _ => b'?',
        })
        .collect()
}

impl UrbanFont {
    pub fn new(font: i32) -> UrbanFont {
        let (file, nr) = match font {
            LARGE_FONT => ("rsifont.pcx", 1),
            SMALL_FONT2 => ("dafont3.pcx", 0),
            _ => ("dafont2.pcx", 0),
        };
        let (image, pal) = icache::get_pal(file);
        let i = &INFO[nr];
        UrbanFont { image, pal, nr, scale: 100, space_x: i.space_x, space_y: i.space_y, width: i.width, height: i.height }
    }

    pub fn pal(&self) -> &Palette {
        &self.pal
    }

    pub fn set_scale(&mut self, proc_: i32) {
        let i = &INFO[self.nr];
        self.scale = proc_;
        self.space_x = proc_ * i.space_x / 100;
        self.space_y = proc_ * i.space_y / 100;
        self.width = proc_ * i.width / 100;
        self.height = proc_ * i.height / 100;
    }

    fn text_size(&self, text: &[u8]) -> (i32, i32) {
        let (mut w, mut h, mut max_w) = (0, self.space_y, 0);
        for &c in text {
            if c == b'\n' {
                h += self.space_y;
                w = 0;
            } else {
                w += self.space_x;
                max_w = max_w.max(w);
            }
        }
        (max_w, h)
    }

    /// Renders text into a new bitmap (colour 0 is transparent).
    pub fn render(&self, text: &str) -> Bitmap {
        let text = cp437(text);
        let (w, h) = self.text_size(&text);
        let mut bmp = Bitmap::new(w, h);
        let table = if self.nr == 0 { SMALL_CHARS } else { LARGE_CHARS };
        let info = &INFO[self.nr];
        let (mut dx, mut dy) = (0, 0);
        for &c in &text {
            match c {
                b'\n' => {
                    dy += self.space_y;
                    dx = 0;
                }
                b' ' => dx += self.space_x,
                _ => {
                    let Some(&(_, cx, cy)) = table.iter().find(|e| e.0 == c) else { continue };
                    if self.scale == 100 {
                        gfx::blit(&self.image, &mut bmp, cx * info.width, cy * info.height, dx, dy, self.width, self.height);
                    } else {
                        gfx::stretch_blit(&self.image, &mut bmp, cx * info.width, cy * info.height, info.width, info.height, dx, dy, self.width, self.height);
                    }
                    dx += self.space_x;
                }
            }
        }
        bmp
    }

    pub fn print(&self, text: &str, x: i32, y: i32, dest: &mut Bitmap) {
        let b = self.render(text);
        gfx::masked_blit(&b, dest, 0, 0, x, y, b.w, b.h);
    }

    pub fn print_centre(&self, text: &str, x: i32, y: i32, dest: &mut Bitmap) {
        let b = self.render(text);
        gfx::masked_blit(&b, dest, 0, 0, x - b.w / 2, y - b.h / 2, b.w, b.h);
    }

    pub fn print_screen(&self, text: &str, x: i32, y: i32) {
        crate::sys::with_screen(|s| self.print(text, x, y, s));
        crate::sys::present(false);
    }

    pub fn print_centre_screen(&self, text: &str, x: i32, y: i32) {
        crate::sys::with_screen(|s| self.print_centre(text, x, y, s));
        crate::sys::present(false);
    }
}
