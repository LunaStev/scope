//! Admission limits cover discovery AND preparation, including the first tile.
//! Time is a soft budget: a single native text operation is not preemptible.
use std::time::{Duration, Instant};

pub const NODE_VISITS: usize = 2048;
pub const LABEL_VISITS: usize = 256;
pub const TILE_CHECKS: usize = 128;
pub const NEW_TILES: usize = 2;
pub const NEW_GLYPHS: usize = 8192;

pub struct FrameBudget {
    started: Instant,
    pub checked: usize,
    pub built: usize,
    pub glyphs: usize,
}

impl FrameBudget {
    pub fn new(started: Instant) -> Self {
        Self { started, checked: 0, built: 0, glyphs: 0 }
    }
    pub fn can_check(&self) -> bool {
        self.checked < TILE_CHECKS && self.started.elapsed() < Duration::from_millis(4)
    }
    pub fn can_build(&self, upper_glyphs: usize) -> bool {
        self.can_check() && self.built < NEW_TILES
            && self.glyphs.saturating_add(upper_glyphs) <= NEW_GLYPHS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_tile_cannot_bypass_an_expired_frame() {
        let budget = FrameBudget::new(Instant::now() - Duration::from_millis(10));
        assert!(!budget.can_build(4096));
        assert!(!budget.can_check());
    }
    #[test]
    fn missing_tile_checks_are_bounded_without_any_builds() {
        let mut budget = FrameBudget::new(Instant::now());
        budget.checked = TILE_CHECKS;
        assert!(!budget.can_check());
    }
    #[test]
    fn uploads_have_a_hard_glyph_and_tile_limit() {
        let mut budget = FrameBudget::new(Instant::now());
        budget.glyphs = NEW_GLYPHS;
        assert!(!budget.can_build(1));
        budget.glyphs = 0;
        budget.built = NEW_TILES;
        assert!(!budget.can_build(1));
    }
}
