//! Fixed-size, reusable SDF glyph templates. No source text or UI resources.
//! ASCII source takes the fast path; unsupported characters retain the host's
//! native shaping/fallback path. The font bytes come from the host dependency.
use fontdue::{Font, FontSettings};
use sdfer::{Image2d, Unorm8, esdt::{self, Params}};

pub const ATLAS_SIZE: usize = 1024;
pub const FONT_SIZE: f64 = 48.0;
pub const DISTANCE_RANGE: f32 = 12.0;
const PAD: usize = 6;

#[derive(Clone, Copy, Debug, Default)]
pub struct Glyph {
    /// Position and size relative to a nominal 16-point source cell.
    pub offset: [f32; 2],
    pub size: [f32; 2],
    pub uv: [f32; 4],
}

pub struct Atlas {
    pub pixels: Vec<u32>,
    pub glyphs: [Glyph; 128],
}

impl Atlas {
    pub fn build(bytes: &[u8]) -> Result<Self, String> {
        let font = Font::from_bytes(bytes, FontSettings::default()).map_err(|e| e.to_string())?;
        let mut atlas = Self {
            pixels: vec![0xff000000; ATLAS_SIZE * ATLAS_SIZE],
            glyphs: [Glyph::default(); 128],
        };
        let (mut x, mut y, mut row_height) = (1usize, 1usize, 0usize);
        let mut buffers = None;
        for ch in 33u8..=126 {
            let (metrics, bitmap) = font.rasterize(ch as char, (FONT_SIZE * 4.0 / 3.0) as f32);
            if metrics.width == 0 || metrics.height == 0 { continue; }
            let mut input = Image2d::from_storage(metrics.width, metrics.height,
                bitmap.into_iter().map(Unorm8::from_bits).collect::<Vec<_>>());
            let params = Params { pad: PAD, radius: DISTANCE_RANGE, cutoff: 0.5,
                solidify: false, preprocess: true };
            let (sdf, reused) = esdt::glyph_to_sdf(&mut input, params, buffers.take());
            buffers = Some(reused);
            let (w, h) = (sdf.width(), sdf.height());
            if w + 2 >= ATLAS_SIZE || h + 2 >= ATLAS_SIZE {
                return Err("Glyph exceeds fixed source atlas dimensions".into());
            }
            if x + w + 1 > ATLAS_SIZE { x = 1; y += row_height + 1; row_height = 0; }
            if y + h + 1 > ATLAS_SIZE { return Err("Source atlas is full".into()); }
            for py in 0..h {
                for px in 0..w {
                    let value = sdf[(px, py)].to_bits() as u32;
                    atlas.pixels[(y + py) * ATLAS_SIZE + x + px] = 0xff000000 | value * 0x010101;
                }
            }
            let scale = (16.0 / FONT_SIZE) as f32;
            // Match the complete map's baseline and the fixed source geometry.
            atlas.glyphs[ch as usize] = Glyph {
                offset: [(metrics.xmin as f32 - PAD as f32) * scale,
                    (FONT_SIZE as f32 * 1.05 - metrics.ymin as f32 - metrics.height as f32 - PAD as f32) * scale],
                size: [w as f32 * scale, h as f32 * scale],
                uv: [x as f32 / ATLAS_SIZE as f32, y as f32 / ATLAS_SIZE as f32,
                    w as f32 / ATLAS_SIZE as f32, h as f32 / ATLAS_SIZE as f32],
            };
            x += w + 1;
            row_height = row_height.max(h);
        }
        Ok(atlas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn invalid_font_is_a_recoverable_error() { assert!(Atlas::build(b"not a font").is_err()); }
    #[test] fn sdf_has_the_expected_polarity_and_padding() {
        let mut input: Image2d<Unorm8> = Image2d::from_fn(12, 12, |x, y| {
            Unorm8::from_bits(if (3..9).contains(&x) && (3..9).contains(&y) { 255 } else { 0 })
        });
        let (sdf, _) = esdt::glyph_to_sdf(&mut input, Params {
            pad: PAD, radius: DISTANCE_RANGE, cutoff: 0.5, solidify: false, preprocess: true,
        }, None);
        assert_eq!((sdf.width(), sdf.height()), (24, 24));
        assert!(sdf[(12, 12)].decode() > 0.5);
        assert!(sdf[(0, 0)].decode() < 0.5);
    }
}
