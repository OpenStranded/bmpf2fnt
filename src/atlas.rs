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

use crate::bmpf::BmpfFont;
use crate::error::BmpfError;

const MAGENTA_BGR: (u8, u8, u8) = (255, 0, 255);
const ALPHA_TOLERANCE: u8 = 10;

#[derive(Debug, Clone)]
pub struct GlyphRegion {
    pub code: u8,
    pub x_advance: f32,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub x_offset: i32,
    pub y_offset: i32,
}

#[derive(Debug, Clone)]
pub struct FontAtlas {
    pub texture_w: u32,
    pub texture_h: u32,
    pub glyphs: Vec<GlyphRegion>,
    pub line_height: u32,
    pub base: u32,
}

#[derive(Debug, Clone)]
struct GlyphBounds {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

pub fn build_font_atlas(
    bmp_rgba: &[u8],
    img_w: u32,
    img_h: u32,
    bmpf: &BmpfFont,
) -> Result<FontAtlas, BmpfError> {
    let tile_w = bmpf.tile_w as u32;
    let tile_h = bmpf.tile_h as u32;
    let glyphs = extract_tile_regions(bmp_rgba, img_w, img_h, tile_w, tile_h, bmpf.chars.len())?;

    let mut result = Vec::with_capacity(bmpf.chars.len());

    for (i, bc) in bmpf.chars.iter().enumerate() {
        if let Some(gr) = &glyphs[i] {
            result.push(GlyphRegion {
                code: bc.code,
                x_advance: bc.advance as f32,
                x: gr.x,
                y: gr.y,
                w: gr.w,
                h: gr.h,
                x_offset: 0,
                y_offset: 0,
            });
        } else {
            result.push(GlyphRegion {
                code: bc.code,
                x_advance: bc.advance as f32,
                x: 0,
                y: 0,
                w: 0,
                h: 0,
                x_offset: 0,
                y_offset: 0,
            });
        }
    }

    let line_h = tile_h.max(1);
    let base = result
        .iter()
        .filter(|g| g.h > 0)
        .map(|g| g.y + g.h)
        .max()
        .unwrap_or(line_h)
        .min(line_h);

    Ok(FontAtlas {
        texture_w: img_w,
        texture_h: img_h,
        glyphs: result,
        line_height: line_h,
        base,
    })
}

/// For each tile in the grid, compute the bounding box of all non-empty
/// pixels within that tile's area. Returns a Vec with one entry per tile
/// (None if the tile is entirely empty).
fn extract_tile_regions(
    rgba: &[u8],
    img_w: u32,
    img_h: u32,
    tile_w: u32,
    tile_h: u32,
    tile_count: usize,
) -> Result<Vec<Option<GlyphBounds>>, BmpfError> {
    let w = img_w as usize;
    let h = img_h as usize;
    if rgba.len() < w * h * 4 {
        return Err(BmpfError::Io("image buffer too small".into()));
    }

    let tw = tile_w as usize;
    let th = tile_h as usize;
    let mut result = Vec::with_capacity(tile_count);

    for ti in 0..tile_count {
        let tx = ti * tw;
        if tx >= w {
            result.push(None);
            continue;
        }

        let mut min_x = w;
        let mut max_x = 0usize;
        let mut min_y = h;
        let mut max_y = 0usize;
        let mut found = false;

        for dy in 0..th.min(h) {
            for dx in 0..tw.min(w - tx) {
                let px = tx + dx;
                let py = dy;
                let idx = (py * w + px) * 4;
                if !is_pixel_empty(rgba[idx], rgba[idx + 1], rgba[idx + 2], rgba[idx + 3]) {
                    found = true;
                    min_x = min_x.min(px);
                    max_x = max_x.max(px);
                    min_y = min_y.min(py);
                    max_y = max_y.max(py);
                }
            }
        }

        if found {
            result.push(Some(GlyphBounds {
                x: min_x as u32,
                y: min_y as u32,
                w: (max_x - min_x + 1) as u32,
                h: (max_y - min_y + 1) as u32,
            }));
        } else {
            result.push(None);
        }
    }

    Ok(result)
}

fn is_pixel_empty(r: u8, g: u8, b: u8, a: u8) -> bool {
    if a == 0 {
        return true;
    }
    let dr = r.abs_diff(MAGENTA_BGR.2);
    let dg = g.abs_diff(MAGENTA_BGR.1);
    let db = b.abs_diff(MAGENTA_BGR.0);
    dr <= ALPHA_TOLERANCE && dg <= ALPHA_TOLERANCE && db <= ALPHA_TOLERANCE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_atlas_simple() {
        let bmpf = BmpfFont {
            chars: vec![
                crate::bmpf::BmpfChar {
                    code: 65,
                    advance: 10,
                },
            ],
            tile_w: 16,
            tile_h: 16,
            frames: 1,
        };

        let w = 20u32;
        let h = 20u32;
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        // Draw a non-magenta, non-transparent pixel for 'A'
        let px = (5 * w as usize + 5) * 4;
        rgba[px] = 0;
        rgba[px + 1] = 0;
        rgba[px + 2] = 0;
        rgba[px + 3] = 255;

        let atlas = build_font_atlas(&rgba, w, h, &bmpf).unwrap();
        assert_eq!(atlas.glyphs.len(), 1);
        assert_eq!(atlas.glyphs[0].code, 65);
        assert_eq!(atlas.glyphs[0].x, 5);
        assert_eq!(atlas.glyphs[0].y, 5);
    }
}
