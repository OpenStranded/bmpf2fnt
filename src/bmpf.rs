// bmpf2fnt – convert Stranded II .bmpf bitmap fonts to BMFont .fnt
// Copyright (C) 2024  bmpf2fnt contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Parses `.bmpf` bitmap font files in the Stranded II / Blitz3D format.
//!
//! # Format (reverse-engineered from Blitz3D source `load_bmpf.bb`)
//!
//! ```text
//! [46-byte magic]    "Unreal Software Bitmap Font Wizard bmpf File\r\n"
//! u16 LE frames      Number of tiles/frames in the spritesheet
//! u16 LE tile_w      Width of each tile in pixels
//! u16 LE tile_h      Height of each tile in pixels (font line height)
//! [repeating…]
//!   u8   code        ASCII/ANSI character code
//!   u16 LE advance   Character advance width in pixels
//! [terminator]       00 00 00 (code=0, advance=0)
//! ```

use crate::error::BmpfError;

const MAGIC: &[u8] = b"Unreal Software Bitmap Font Wizard bmpf File\r\n";
const MAGIC_LEN: usize = 46;

#[derive(Debug, Clone)]
pub struct BmpfChar {
    pub code: u8,
    /// Advance width in pixels (horizontal distance to next character origin).
    pub advance: u16,
}

#[derive(Debug, Clone)]
pub struct BmpfFont {
    pub chars: Vec<BmpfChar>,
    /// Tile width from the BMPF header (frame width).
    pub tile_w: u16,
    /// Tile height from the BMPF header (frame height, used as line height).
    pub tile_h: u16,
    /// Number of glyph frames in the spritesheet.
    pub frames: u16,
}

impl BmpfFont {
    /// Parse a `.bmpf` binary file.
    ///
    /// # Errors
    ///
    /// - [`BmpfError::BadMagic`] if the file doesn't start with the expected magic.
    /// - [`BmpfError::BadHeader`] if the file is too short.
    pub fn parse(data: &[u8]) -> Result<Self, BmpfError> {
        if data.len() < MAGIC_LEN + 6 {
            return Err(BmpfError::BadHeader);
        }
        if &data[..MAGIC_LEN] != MAGIC {
            return Err(BmpfError::BadMagic);
        }

        let rest = &data[MAGIC_LEN..];

        // ── Header: 3 shorts ──────────────────────────────────────────
        let frames = u16::from_le_bytes([rest[0], rest[1]]);
        let tile_w = u16::from_le_bytes([rest[2], rest[3]]);
        let tile_h = u16::from_le_bytes([rest[4], rest[5]]);

        // ── Glyph records ─────────────────────────────────────────────
        let records_data = &rest[6..];
        let mut chars = Vec::with_capacity(frames as usize);
        let mut i = 0;

        while i + 3 <= records_data.len() {
            // Terminator: 00 00 00 (code=0, advance=0)
            if records_data[i] == 0
                && records_data[i + 1] == 0
                && records_data[i + 2] == 0
            {
                break;
            }
            let code = records_data[i];
            let advance =
                u16::from_le_bytes([records_data[i + 1], records_data[i + 2]]);
            chars.push(BmpfChar { code, advance });
            i += 3;
        }

        Ok(BmpfFont {
            chars,
            tile_w,
            tile_h,
            frames,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_font_norm() {
        let mut data = MAGIC.to_vec();
        // Header: frames=3, tile_w=16, tile_h=19
        data.extend_from_slice(&[0x03, 0x00, 0x10, 0x00, 0x13, 0x00]);
        // Records
        data.extend_from_slice(&[0x65, 0x0a, 0x00]); // 'e', advance=10
        data.extend_from_slice(&[0x21, 0x06, 0x00]); // '!', advance=6
        data.extend_from_slice(&[0x22, 0x09, 0x00]); // '"', advance=9
        data.extend_from_slice(&[0x00, 0x00, 0x00]); // terminator

        let font = BmpfFont::parse(&data).unwrap();
        assert_eq!(font.frames, 3);
        assert_eq!(font.tile_w, 16);
        assert_eq!(font.tile_h, 19);
        assert_eq!(font.chars.len(), 3);
        assert_eq!(font.chars[0].code, 0x65);
        assert_eq!(font.chars[0].advance, 10);
        assert_eq!(font.chars[1].code, 0x21);
        assert_eq!(font.chars[1].advance, 6);
        assert_eq!(font.chars[2].code, 0x22);
        assert_eq!(font.chars[2].advance, 9);
    }

    #[test]
    fn test_parse_real_font_norm() {
        let raw = include_bytes!("../tests/in/font_norm.bmpf");
        let font = BmpfFont::parse(raw).unwrap();
        assert_eq!(font.frames, 101);
        assert_eq!(font.tile_w, 16);
        assert_eq!(font.tile_h, 19);
        assert!(!font.chars.is_empty());
        // First real glyph should be '!' (0x21) not 'e'
        // because the first chars in the file are in ASCII order
        assert!(font.chars.iter().any(|c| c.code == b'e'));
        // 'e' advance should be 10 pixels (not 4096 as before the fix)
        let e_char = font.chars.iter().find(|c| c.code == b'e').unwrap();
        assert_eq!(e_char.advance, 10);
    }

    #[test]
    fn test_parse_real_font_tiny() {
        let raw = include_bytes!("../tests/in/font_tiny.bmpf");
        let font = BmpfFont::parse(raw).unwrap();
        assert_eq!(font.frames, 256);
        assert_eq!(font.tile_w, 13);
        assert_eq!(font.tile_h, 16);
        assert!(font.chars.len() > 200);
    }

    #[test]
    fn test_reject_bad_magic() {
        let data = b"not a bmpf file";
        let result = BmpfFont::parse(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_too_short() {
        // Magic only, no header
        let data = MAGIC.to_vec();
        let result = BmpfFont::parse(&data);
        assert!(result.is_err());
    }
}
