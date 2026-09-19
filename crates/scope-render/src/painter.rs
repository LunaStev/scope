use makepad_widgets::*;
use scope_layout::Box2;
use crate::geometry::rect;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    pub MapPainter = {{MapPainter}} {
        quad: {color: #172130}
        label: {text_style: <THEME_FONT_REGULAR> {font_size: 11}, color: #e7eef6}
        code: {text_style: <THEME_FONT_CODE> {font_size: 11}, color: #b8c9db}
    }
}

#[derive(Live, LiveHook)]
pub struct MapPainter {
    #[live] pub quad: DrawColor,
    #[live] pub label: DrawText,
    #[live] pub code: DrawText,
}

// This is a reusable drawing resource, not a Widget. Register it explicitly
// instead of relying on the Widget derive to provide LiveRegister.
impl LiveRegister for MapPainter {
    fn live_register(_cx: &mut Cx) {}
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderStats {
    pub visible_nodes: usize,
    pub glyph_runs: usize,
    pub capped: bool,
    /// CPU submission time, not GPU completion time or a frame-rate estimate.
    pub cpu_submit_ms: f64,
}

impl MapPainter {
    pub fn fill(&mut self, cx: &mut Cx2d, bounds: Box2, color: Vec4) {
        if bounds.w <= 0.0 || bounds.h <= 0.0 { return; }
        self.quad.color = color;
        self.quad.draw_abs(cx, rect(bounds));
    }

    pub fn text(&mut self, cx: &mut Cx2d, x: f64, y: f64, size: f32, color: Vec4, text: &str) {
        self.label.color = color;
        self.label.text_style.font_size = size;
        self.label.draw_abs(cx, dvec2(x, y), text);
    }
}
