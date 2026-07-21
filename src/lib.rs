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

//! Parse `.bmpf` bitmap font files (Stranded II / Blitz3D format) and
//! generate `.fnt` (BMFont / AngelCode text format) output.
//!
//! # `.bmpf` format (reverse-engineered from Blitz3D source)
//!
//! ```text
//! [46-byte ASCII header]  "Unreal Software Bitmap Font Wizard bmpf File\r\n"
//! u16 LE frames           Number of tiles/frames in the spritesheet
//! u16 LE tile_w           Width of each tile in pixels
//! u16 LE tile_h           Height of each tile in pixels (font line height)
//! [3-byte records …]
//!   u8:    character code
//!   u16 LE: advance width (in pixels)
//! [terminator]             00 00 00
//! ```

pub mod atlas;
pub mod bmpf;
pub mod cli;
pub mod error;
pub mod fnt;

pub use atlas::{build_font_atlas, FontAtlas, GlyphRegion};
pub use bmpf::{BmpfChar, BmpfFont};
pub use error::BmpfError;
pub use fnt::generate_bmfont;
