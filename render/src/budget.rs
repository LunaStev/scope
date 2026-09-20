//! Bounded glyph-instance admission; native fallback keeps the smaller budget.
use std::time::{Duration, Instant};
pub const NODE_VISITS: usize = 2048;
pub const LABEL_VISITS: usize = 256;
pub const TILE_CHECKS: usize = 128;
pub const NEW_TILES: usize = 2;
pub const NEW_GLYPHS: usize = 8192;
pub const ATLAS_TILES: usize = 12;
pub const ATLAS_GLYPHS: usize = 18_432;
pub struct FrameBudget {started:Instant,pub checked:usize,pub built:usize,pub glyphs:usize}
impl FrameBudget {
    pub fn new(started:Instant)->Self{Self{started,checked:0,built:0,glyphs:0}}
    pub fn can_check(&self)->bool{self.checked<TILE_CHECKS&&self.started.elapsed()<Duration::from_millis(4)}
    pub fn can_build(&self,upper_glyphs:usize)->bool{self.can_build_kind(upper_glyphs,false)}
    pub fn can_build_kind(&self,upper_glyphs:usize,atlas:bool)->bool {
        let (tiles,glyphs)=if atlas{(ATLAS_TILES,ATLAS_GLYPHS)}else{(NEW_TILES,NEW_GLYPHS)};
        self.can_check()&&self.built<tiles&&self.glyphs.saturating_add(upper_glyphs)<=glyphs
    }
}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn first_tile_cannot_bypass_an_expired_frame(){let b=FrameBudget::new(Instant::now()-Duration::from_millis(10));assert!(!b.can_build(4096));assert!(!b.can_build_kind(1536,true));assert!(!b.can_check());}
    #[test]fn missing_tile_checks_are_bounded_without_any_builds(){let mut b=FrameBudget::new(Instant::now());b.checked=TILE_CHECKS;assert!(!b.can_check());}
    #[test]fn uploads_have_a_hard_glyph_and_tile_limit(){let mut b=FrameBudget::new(Instant::now());b.glyphs=NEW_GLYPHS;assert!(!b.can_build(1));b.glyphs=0;b.built=NEW_TILES;assert!(!b.can_build(1));}
    #[test]fn fast_instances_are_not_an_unbounded_upload(){let mut b=FrameBudget::new(Instant::now());b.built=ATLAS_TILES;assert!(!b.can_build_kind(1536,true));b.built=0;b.glyphs=ATLAS_GLYPHS;assert!(!b.can_build_kind(1,true));}
}
