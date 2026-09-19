use crate::Box2;
use model::Document;

pub const GLYPH_ADVANCE: f64 = 0.81;
pub const LINE_HEIGHT: f64 = 1.65;
pub const READING_SIZE: f64 = 12.0;
/// Separation is measured in character cells, never leftover tile width.
pub const COLUMN_GAP: f64 = 3.0;
pub const CHUNK_LINES: usize = 128;

#[derive(Clone, Debug)]
pub struct SourceColumn {
    pub first_line: usize,
    pub end_line: usize,
    pub x: f64,
    pub width: f64,
}

/// A fixed, tightly packed page. Camera movement never reflows source.
#[derive(Clone, Debug)]
pub struct SourceLayout {
    pub content: Box2,
    pub columns: usize,
    pub rows: usize,
    pub column_width: f64,
    pub line_height: f64,
    pub font_size: f64,
    pub packed: Vec<SourceColumn>,
}
impl SourceLayout {
    pub fn new(bounds: Box2, lines: usize, max_columns: usize) -> Self {
        Self::from_widths(bounds, &vec![max_columns; lines])
    }
    pub fn for_document(bounds: Box2, document: &Document) -> Self {
        let widths: Vec<usize> = document.runs.iter().map(|runs| {
            runs.iter().map(|r| r.column + r.text.trim_end().chars().count()).max().unwrap_or(0)
        }).collect();
        Self::from_widths(bounds, &widths)
    }
    fn from_widths(bounds: Box2, widths: &[usize]) -> Self {
        let padding = (bounds.w.min(bounds.h) * 0.009).min(3.0);
        let mut content = bounds.inset(padding);
        let title = (content.h * 0.035).min(18.0);
        content.y += title;
        content.h = (content.h - title).max(1e-12);
        content.w = content.w.max(1e-12);
        let n = widths.len().max(1);
        // An isolated long line enlarges only its own column.
        let mean = if widths.is_empty() { 24.0 } else {
            widths.iter().map(|&w| w as f64).sum::<f64>() / n as f64
        }.max(12.0);
        let estimate = (content.w * n as f64 * LINE_HEIGHT /
            (content.h * (mean + COLUMN_GAP) * GLYPH_ADVANCE)).sqrt().round() as usize;
        let estimate = estimate.clamp(1, n);
        let mut candidates = vec![1, estimate];
        for delta in 1..=3 {
            candidates.push(estimate.saturating_sub(delta).max(1));
            candidates.push(estimate.saturating_add(delta).min(n));
        }
        candidates.sort_unstable(); candidates.dedup();
        let mut best_font = 0.0;
        let mut best_rows = n;
        let mut best_widths = Vec::new();
        for count in candidates {
            let rows = n.div_ceil(count);
            let cells: Vec<f64> = if widths.is_empty() { vec![12.0] } else {
                widths.chunks(rows).map(|c| c.iter().copied().max().unwrap_or(0).max(1) as f64).collect()
            };
            let span = cells.iter().sum::<f64>() + COLUMN_GAP * cells.len().saturating_sub(1) as f64;
            let font = (content.w / (span.max(1.0) * GLYPH_ADVANCE))
                .min(content.h / (rows as f64 * LINE_HEIGHT)).max(1e-15);
            if font > best_font { best_font = font; best_rows = rows; best_widths = cells; }
        }
        let cell = best_font * GLYPH_ADVANCE;
        let mut x = content.x;
        let mut packed = Vec::with_capacity(best_widths.len());
        for (column, width) in best_widths.into_iter().enumerate() {
            let first_line = column * best_rows;
            let width = width * cell;
            packed.push(SourceColumn { first_line, end_line: (first_line + best_rows).min(widths.len()), x, width });
            x += width + COLUMN_GAP * cell;
        }
        let column_width = packed.iter().map(|c|c.width).fold(0.0,f64::max) + COLUMN_GAP * cell;
        Self { content, columns: packed.len(), rows: best_rows, column_width,
            line_height: best_font * LINE_HEIGHT, font_size: best_font, packed }
    }
    pub fn line_origin(&self, index: usize) -> (f64,f64) {
        let col = (index / self.rows).min(self.packed.len().saturating_sub(1));
        (self.packed[col].x, self.content.y + (index % self.rows) as f64 * self.line_height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Camera;
    #[test] fn zoom_only_scales_the_same_text_geometry() {
        let page = SourceLayout::new(Box2::new(10.0,20.0,600.0,400.0),300,90);
        let (x,y)=page.line_origin(173);
        let mut camera=Camera::default(); camera.zoom_at(3.0,0.0,0.0);
        let b=camera.project(Box2::new(x,y,page.font_size,page.line_height));
        assert!((b.x-x*3.0).abs()<1e-10); assert_eq!(page.line_origin(173),(x,y));
    }
    #[test] fn every_line_fits() {
        for n in [0,1,50,150_000] {
            let p=SourceLayout::new(Box2::new(0.0,0.0,1000.0,500.0),n,100);
            assert!(p.font_size.is_finite() && p.font_size>0.0);
            for i in 0..n {let(x,y)=p.line_origin(i);assert!(p.content.contains(x,y));}
            for c in &p.packed {assert!(c.x+c.width<=p.content.x+p.content.w+1e-7);}
        }
    }
    #[test] fn gap_is_exactly_three_cells_not_spare_tile_width() {
        let p=SourceLayout::new(Box2::new(0.0,0.0,1900.0,220.0),1200,42);
        assert!(p.columns>1);
        for c in p.packed.windows(2) {
            assert!(((c[1].x-c[0].x-c[0].width)/(p.font_size*GLYPH_ADVANCE)-COLUMN_GAP).abs()<1e-8);
        }
    }
    #[test] fn long_line_does_not_widen_unrelated_columns() {
        let mut widths=vec![32;1200]; widths[0]=280;
        let p=SourceLayout::from_widths(Box2::new(0.0,0.0,2000.0,250.0),&widths);
        assert!(p.columns>1);
        assert!(p.packed[0].width>p.packed[1].width*5.0);
        assert_eq!(p.packed.iter().map(|c|c.end_line-c.first_line).sum::<usize>(),1200);
    }
}
