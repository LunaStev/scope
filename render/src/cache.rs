//! Retained glyph geometry. Pan/zoom updates per-draw-list uniforms only.
use makepad_widgets::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ChunkKey { pub node: usize, pub column: usize, pub block: usize }
pub struct CachedChunk {
    pub list: DrawList2d,
    pub lines: usize,
    pub runs: usize,
}
#[derive(Default)]
pub struct GeometryCache {
    generation: u64,
    dpi: u64,
    pub chunks: HashMap<ChunkKey, CachedChunk>,
}
impl GeometryCache {
    pub fn prepare(&mut self, generation: u64, dpi: f64) {
        if generation != self.generation || dpi.to_bits() != self.dpi {
            self.chunks.clear(); self.generation = generation; self.dpi = dpi.to_bits();
        }
    }
    pub fn clear(&mut self) { self.chunks.clear(); self.generation=0; }
}

/// Source-only packed uniforms: xy is the affine camera; unused z/w lanes
/// carry screen clipping and dimming. UI lists never use this encoding.
/// Chunk-local coordinates prevent cancellation at large magnifications.
pub fn source_uniform(scale: f64, x: f64, y: f64, clip: ::layout::Box2, dim: f32) -> Mat4 {
    Mat4 { v: [
        scale as f32, 0.0, clip.x as f32, clip.y as f32,
        0.0, scale as f32, (clip.x+clip.w) as f32, (clip.y+clip.h) as f32,
        0.0, 0.0, dim, 0.0,
        x as f32, y as f32, 0.0, 1.0,
    ] }
}
