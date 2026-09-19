//! Glyph residency and retained presentation have different invalidation keys.
use makepad_widgets::*;
use ::runtime::residency::Residency;
use crate::{presentation::Presentation,schedule::SourceQueue};
use std::collections::HashSet;
#[derive(Clone,Copy,Hash,PartialEq,Eq,PartialOrd,Ord)]
pub struct ChunkKey { pub node:usize,pub column:usize,pub block:usize,pub tile:usize }
pub struct CachedChunk {
    pub list:DrawList2d,pub lines:usize,pub runs:usize,pub glyphs:usize,
    pub world: ::layout::Box2,pub born_frame:u64,
}
pub struct GeometryCache {
    generation:u64,dpi:u64,
    pub chunks:Residency<ChunkKey,CachedChunk>,pub frame:u64,
    pub scene:Presentation,pub queue:SourceQueue,
    pub line_blocks:HashSet<(usize,usize,usize)>,
}
impl Default for GeometryCache {
    fn default()->Self { Self{generation:0,dpi:0,chunks:Residency::new(1_000_000,4096),frame:0,scene:Presentation::default(),queue:SourceQueue::default(),line_blocks:HashSet::new()} }
}
impl GeometryCache {
    pub fn prepare(&mut self,generation:u64,dpi:f64) {
        if self.generation!=generation||self.dpi!=dpi.to_bits() { self.clear();self.generation=generation;self.dpi=dpi.to_bits(); }
        self.frame=self.frame.wrapping_add(1).max(1);self.line_blocks.clear();
    }
    pub fn clear(&mut self) { self.chunks.clear();self.scene=Presentation::default();self.queue=SourceQueue::default();self.line_blocks.clear();self.generation=0; }
}
/// Source-only packed uniforms. Translate in f64 before converting GPU floats.
pub fn source_uniform(scale:f64,x:f64,y:f64,clip: ::layout::Box2,dim:f32)->Mat4 {
    Mat4{v:[scale as f32,0.0,clip.x as f32,clip.y as f32,0.0,scale as f32,(clip.x+clip.w) as f32,(clip.y+clip.h) as f32,0.0,0.0,dim,0.0,x as f32,y as f32,0.0,1.0]}
}
