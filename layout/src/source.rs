use crate::Box2;

pub const GLYPH_ADVANCE: f64 = 0.81;
pub const LINE_HEIGHT: f64 = 1.85;
pub const READING_SIZE: f64 = 12.0;

/// One world-space typesetting solution. This has NO camera or zoom parameter.
/// Every glyph remains at this position throughout navigation.
#[derive(Clone, Copy, Debug)]
pub struct SourceLayout {
    pub content: Box2,
    pub columns: usize,
    pub rows: usize,
    pub column_width: f64,
    pub line_height: f64,
    pub font_size: f64,
}
impl SourceLayout {
    pub fn new(bounds: Box2, lines: usize, max_columns: usize) -> Self {
        let padding = (bounds.w.min(bounds.h) * 0.018).min(5.0);
        let mut content = bounds.inset(padding);
        let title = (content.h * 0.05).min(24.0);
        content.y += title;
        content.h = (content.h - title).max(1e-12);
        content.w = content.w.max(1e-12);
        let lines = lines.max(1);
        let glyph_columns = max_columns.max(36) as f64 + 4.0;
        let columns = ((content.w * lines as f64 * LINE_HEIGHT / (content.h * glyph_columns * GLYPH_ADVANCE)).sqrt().round() as usize).clamp(1, lines);
        let rows = lines.div_ceil(columns);
        let column_width = content.w / columns as f64;
        let font_size = (column_width / (glyph_columns * GLYPH_ADVANCE)).min(content.h / (rows as f64 * LINE_HEIGHT)).max(1e-15);
        Self { content, columns, rows, column_width, line_height: font_size * LINE_HEIGHT, font_size }
    }
    pub fn line_origin(self, index: usize) -> (f64, f64) {
        (self.content.x + (index / self.rows) as f64 * self.column_width,
         self.content.y + (index % self.rows) as f64 * self.line_height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Camera;
    #[test] fn zoom_only_scales_the_same_text_geometry() {
        let page = SourceLayout::new(Box2::new(10.0,20.0,600.0,400.0),300,90);
        let (x,y) = page.line_origin(173);
        let mut camera = Camera::default();
        camera.zoom_at(3.0,0.0,0.0);
        let projected = camera.project(Box2::new(x,y,page.font_size,page.line_height));
        assert!((projected.x-x*3.0).abs()<1e-10);
        assert!((projected.h-page.line_height*3.0).abs()<1e-10);
        assert_eq!(page.line_origin(173), (x,y));
    }
    #[test] fn every_line_fits_the_world_page() {
        for n in [0,1,50,150_000] {
            let page=SourceLayout::new(Box2::new(0.0,0.0,1000.0,500.0),n,100);
            assert!(page.font_size.is_finite() && page.font_size>0.0);
            for i in 0..n {
                let(x,y)=page.line_origin(i);
                assert!(page.content.contains(x,y));
            }
        }
    }
}
