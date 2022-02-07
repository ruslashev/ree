// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::lazy::OnceCell;

use crate::multiboot::BootloaderInfo;
use crate::panic::panic_early;
use crate::spinlock::SpinlockMutex;

static CONSOLE: SpinlockMutex<OnceCell<Console>> = SpinlockMutex::new(OnceCell::new());

const FONT: &[u8] = include_bytes!("../Lat7-Fixed14.psf");

#[derive(Debug)]
struct Console {
    addr: usize,
    width: u32,
    height: u32,
    pitch: u32,
    bytes_per_pixel: u8,
    font: Font,
}

impl Console {
    fn from_info(info: &BootloaderInfo) -> Self {
        let fb = &info.framebuffer;

        Console {
            addr: usize::try_from(fb.addr).unwrap(),
            width: fb.width,
            height: fb.height,
            pitch: fb.pitch,
            bytes_per_pixel: fb.bpp / 8,
            font: Font::from_bytes(FONT),
        }
    }

    fn putpixel(&self, x: u32, y: u32, color: u32) {
        let pos = y * self.pitch + x * self.bytes_per_pixel as u32;
        let ptr = (self.addr + pos as usize) as *mut u32;

        unsafe {
            ptr.write(color);
        }
    }

    fn write_byte(&self, b: u8) {
        let offset = self.font.height * b as usize;
        let glyph = &self.font.glyphs[offset..offset + self.font.height];

        let sx: u32 = 0;
        let sy: u32 = 0;

        let mut y = sy;
        let mut x;

        for byte in glyph {
            let mut copy = *byte;
            x = sx + self.font.width;

            for _ in 0..8 {
                let bit = copy & 1;

                if bit == 1 {
                    self.putpixel(x, y, 0xffffff);
                }

                x -= 1;
                copy >>= 1;
            }

            y += 1;
        }
    }
}

#[derive(Debug)]
struct Font {
    width: u32,
    height: usize,
    glyphs: &'static [u8],
}

impl Font {
    fn from_bytes(bytes: &'static [u8]) -> Self {
        if bytes[0..=1] != [0x36, 0x04] {
            panic_early("Font magic mismatch");
        }

        Font {
            width: 8,
            height: bytes[3] as usize,
            glyphs: &bytes[4..],
        }
    }
}

pub fn init(info: &BootloaderInfo) {
    let cell = CONSOLE.guard();

    cell.set(Console::from_info(info)).unwrap();

    let cons = cell.get().unwrap();

    cons.write_byte(b'A');
}
