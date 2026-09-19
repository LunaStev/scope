//! Fixed, tightly packed source columns. Camera changes never reflow text.
use crate::Box2;
use model::Document;

pub const GLYPH_ADVANCE: f64 = 0.81;
pub const LINE_HEIGHT: f64 = 1.85;
pub const READING_SIZE: f64 = 12.0;
/// Gap between adjacent columns, in source-character cells, not screen pixels.
pub const COLUMN_GAP: f64 = 3.0;

#[derive(Clone, Debug)]
pub struct SourceColumn {
    pub offset: f64,
    pub characters: usize,
}
#[derive(Clone, Debug)]
pub struct SourceLayout {
    pub content: Box2,
    pub columns: Vec<SourceColumn>,
    pub rows: usize,
    pub line_height: f64,
    pub font_size: f64,
}
impl SourceLayout {
    /// Convenience constructor for uniform-width documents and geometry tests.
    pub fn new(bounds: Box2, lines: usize, max_columns: usize) -> Self {
        Self::from_widths(bounds, &vec![max_columns; lines])
    }
    pub fn from_document(bounds: Box2, doc: &Document) -> Self {
        let widths: Vec<usize> = doc.runs.iter().map(|runs| {
            runs.last().map_or(0, |run| run.column + run.text.chars().count())
        }).collect();
        Self::from_widths(bounds, &widths)
    }
    pub fn from_widths(bounds: Box2, widths: &[usize]) -> Self {
        let padding = (bounds.w.min(bounds.h).max(0.0) * 0.018).min(5.0);
        let mut content = bounds.inset(padding);
        let title = (content.h * 0.05).min(24.0);
        content.y += title;
        content.h = (content.h - title).max(1e-12);
        content.w = content.w.max(1e-12);
        let count = widths.len().max(1);
        let average = widths.iter().map(|&v| v.max(1) as f64).sum::<f64>() / count as f64;
        let estimate = (content.w * count as f64 * LINE_HEIGHT
            / (content.h * (average.max(12.0) + COLUMN_GAP) * GLYPH_ADVANCE)).sqrt().max(1.0);
        // A bounded number of linear passes, rather than trying all N layouts.
        let mut candidates = vec![1usize];
        for ratio in [0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0] {
            candidates.push(((estimate * ratio).round() as usize).clamp(1, count));
        }
        let center = (estimate.round() as usize).clamp(1, count);
        for delta in 1..=3 {
            candidates.push(center.saturating_sub(delta).max(1));
            candidates.push(center.saturating_add(delta).min(count));
        }
        candidates.sort_unstable(); candidates.dedup();
        let mut best = (0.0f64, count, vec![1usize]);
        for columns in candidates {
            let rows = count.div_ceil(columns);
            let mut maxima: Vec<usize> = widths.chunks(rows).map(|chunk| chunk.iter().copied().max().unwrap_or(0).max(1)).collect();
            if maxima.is_empty() { maxima.push(1); }
            let cells = maxima.iter().map(|&n| n as f64).sum::<f64>() + COLUMN_GAP * maxima.len().saturating_sub(1) as f64;
            let font = (content.w / (cells * GLYPH_ADVANCE)).min(content.h / (rows as f64 * LINE_HEIGHT)).max(1e-15);
            if font > best.0 * (1.0 + 1e-12) { best = (font, rows, maxima); }
        }
        let (font_size, rows, maxima) = best;
        let mut offset = 0.0;
        let columns = maxima.into_iter().map(|characters| {
            let column = SourceColumn { offset, characters };
            // No equal-width cells or justification: only this column's code
            // width followed by the fixed three-character gutter.
            offset += (characters as f64 + COLUMN_GAP) * font_size * GLYPH_ADVANCE;
            column
        }).collect();
        Self { content, columns, rows, line_height: font_size * LINE_HEIGHT, font_size }
    }
    pub fn line_origin(&self, index: usize) -> (f64, f64) {
        let column = index / self.rows;
        (self.content.x + self.columns[column.min(self.columns.len()-1)].offset,
         self.content.y + (index % self.rows) as f64 * self.line_height)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Camera;
    #[test] fn zoom_only_scales_the_same_text_geometry() {
        let page = SourceLayout::new(Box2::new(10.0,20.0,600.0,400.0),300,90);
        let origin = page.line_origin(173);
        let mut camera = Camera::default(); camera.zoom_at(3.0,0.0,0.0);
        let projected = camera.project(Box2::new(origin.0,origin.1,page.font_size,page.line_height));
        assert!((projected.x-origin.0*3.0).abs()<1e-10);
        assert_eq!(page.line_origin(173),origin);
    }
    #[test] fn every_line_fits_and_columns_never_overlap() {
        for n in [0,1,50,150_000] {
            let page=SourceLayout::new(Box2::new(0.0,0.0,1000.0,500.0),n,100);
            assert!(page.font_size.is_finite()&&page.font_size>0.0);
            for i in 0..n { let(x,y)=page.line_origin(i); assert!(page.content.contains(x,y)); }
            for pair in page.columns.windows(2) {
                let gap=pair[1].offset-pair[0].offset-pair[0].characters as f64*page.font_size*GLYPH_ADVANCE;
                assert!((gap-COLUMN_GAP*page.font_size*GLYPH_ADVANCE).abs()<1e-8);
            }
            let last=page.columns.last().unwrap();
            assert!(last.offset+last.characters as f64*page.font_size*GLYPH_ADVANCE<=page.content.w+1e-8);
        }
    }
    #[test] fn column_width_uses_its_own_lines_not_the_global_longest_line() {
        let mut widths=vec![24;1200]; widths[0]=180;
        let p=SourceLayout::from_widths(Box2::new(0.0,0.0,1200.0,600.0),&widths);
        assert!(p.columns.len()>1);
        assert_eq!(p.columns[0].characters,180);
        assert!(p.columns.iter().skip(1).all(|c| c.characters==24));
        for pair in p.columns.windows(2) {
            assert!((pair[1].offset-pair[0].offset-(pair[0].characters as f64+COLUMN_GAP)*p.font_size*GLYPH_ADVANCE).abs()<1e-8);
        }
    }
}
