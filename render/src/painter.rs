use makepad_widgets::*;
use ::layout::Box2;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    pub MapPainter = {{MapPainter}} {
        quad: {color: #172130}
        label: {text_style: <THEME_FONT_REGULAR> {font_size: 11}, color: #e7eef6}
        code: {text_style: <THEME_FONT_CODE> {font_size: 16}, color: #b8c9db}
    }
}
#[derive(Live, LiveHook)]
pub struct MapPainter {
    #[live] pub quad: DrawColor,
    #[live] pub label: DrawText,
    #[live] pub code: DrawText,
}
impl LiveRegister for MapPainter { fn live_register(_cx: &mut Cx) {} }
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderStats {
    pub visible_nodes: usize,
    pub glyph_runs: usize,
    pub source_lines: usize,
    /// CPU submission time; not GPU completion time or FPS.
    pub cpu_submit_ms: f64,
}
impl MapPainter {
    pub fn fill(&mut self, cx: &mut Cx2d, bounds: Box2, color: Vec4) {
        if bounds.w <= 0.0 || bounds.h <= 0.0 { return; }
        self.quad.color = color;
        self.quad.draw_abs(cx, Rect { pos: dvec2(bounds.x,bounds.y), size: dvec2(bounds.w,bounds.h) });
    }
    pub fn text(&mut self, cx: &mut Cx2d, x: f64, y: f64, size: f32, color: Vec4, text: &str) {
        self.label.color = color; self.label.text_style.font_size = size;
        self.label.draw_abs(cx, dvec2(x,y), text);
    }
}
