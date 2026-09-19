//! Lazy visible source tiles. A huge file must not allocate or walk a list of
//! every missing tile on each frame. The cursor resumes at the next tile.
use crate::{Box2, Camera, SourceLayout};
use crate::source::{CHUNK_COLUMNS, CHUNK_LINES, GLYPH_ADVANCE};

#[derive(Clone, Copy, Debug)]
pub struct SourceTile {
    pub column: usize,
    pub block: usize,
    pub tile: usize,
    pub first_line: usize,
    pub end_line: usize,
    pub world: Box2,
}

#[derive(Debug, Default)]
pub struct TileCursor {
    clip: Box2,
    column: usize,
    end_column: usize,
    block: usize,
    end_block: usize,
    tile: usize,
    first_tile: usize,
    end_tile: usize,
    initialized: bool,
}

impl TileCursor {
    pub fn new(page: &SourceLayout, camera: Camera, view: Box2) -> Self {
        if !camera.scale.is_finite() || camera.scale <= 0.0 {
            return Self::default();
        }
        let (x, y) = camera.unproject(view.x, view.y);
        let world = Box2::new(x, y, view.w / camera.scale, view.h / camera.scale);
        let Some(clip) = page.content.intersection(world) else {
            return Self::default();
        };
        let column = page.packed.partition_point(|c| c.x + c.width < clip.x);
        let end_column = page.packed.partition_point(|c| c.x <= clip.x + clip.w);
        Self { clip, column, end_column, ..Self::default() }
    }

    pub fn next(&mut self, page: &SourceLayout) -> Option<SourceTile> {
        while self.column < self.end_column {
            let column = &page.packed[self.column];
            let count = column.end_line - column.first_line;
            let tile_width = page.font_size * GLYPH_ADVANCE * CHUNK_COLUMNS as f64;
            if !self.initialized {
                let first = (((self.clip.y - page.content.y) / page.line_height)
                    .floor().max(0.0) as usize).saturating_sub(1).min(count);
                let end = (((self.clip.y + self.clip.h - page.content.y) / page.line_height)
                    .ceil().max(0.0) as usize).saturating_add(1).min(count);
                self.block = first / CHUNK_LINES;
                self.end_block = end.div_ceil(CHUNK_LINES);
                let tiles = (column.width / tile_width).ceil().max(1.0) as usize;
                self.first_tile = (((self.clip.x - column.x) / tile_width)
                    .floor().max(0.0) as usize).min(tiles);
                self.end_tile = (((self.clip.x + self.clip.w - column.x) / tile_width)
                    .ceil().max(0.0) as usize).min(tiles);
                self.tile = self.first_tile;
                self.initialized = true;
            }
            if self.block >= self.end_block || self.tile >= self.end_tile || count == 0 {
                self.column += 1;
                self.initialized = false;
                continue;
            }
            let first_line = column.first_line + self.block * CHUNK_LINES;
            let end_line = (first_line + CHUNK_LINES).min(column.end_line);
            let tile = SourceTile {
                column: self.column, block: self.block, tile: self.tile,
                first_line, end_line,
                world: Box2::new(
                    column.x + self.tile as f64 * tile_width,
                    page.content.y + self.block as f64 * CHUNK_LINES as f64 * page.line_height,
                    tile_width, (end_line - first_line) as f64 * page.line_height,
                ),
            };
            self.tile += 1;
            if self.tile == self.end_tile {
                self.tile = self.first_tile;
                self.block += 1;
            }
            return Some(tile);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn suspended_cursor_visits_each_visible_tile_once() {
        let page = SourceLayout::new(Box2::new(0.0, 0.0, 800.0, 600.0), 1200, 400);
        let mut cursor = TileCursor::new(&page, Camera::default(), page.content);
        let mut keys = HashSet::new();
        let mut blocks = HashSet::new();
        let mut lines = 0;
        // Taking one item at a time models yielding to the event loop.
        while let Some(tile) = cursor.next(&page) {
            assert!(keys.insert((tile.column, tile.block, tile.tile)));
            if blocks.insert((tile.column, tile.block)) {
                lines += tile.end_line - tile.first_line;
            }
        }
        assert_eq!(lines, 1200);
        assert!(cursor.next(&page).is_none());
    }

    #[test]
    fn offscreen_page_has_no_work() {
        let page = SourceLayout::new(Box2::new(0.0, 0.0, 800.0, 600.0), 1000, 80);
        let mut cursor = TileCursor::new(&page, Camera::default(), Box2::new(-100.0, -100.0, 10.0, 10.0));
        assert!(cursor.next(&page).is_none());
    }

    #[test]
    fn long_line_does_not_allocate_a_tile_vector() {
        let page = SourceLayout::new(Box2::new(0.0, 0.0, 1000.0, 500.0), 1, 1_000_000);
        let mut cursor = TileCursor::new(&page, Camera::default(), page.content);
        assert!(std::mem::size_of_val(&cursor) < 256);
        for _ in 0..16 { assert!(cursor.next(&page).is_some()); }
    }
}
