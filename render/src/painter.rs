use makepad_widgets::*;
use ::layout::Box2;
use crate::retained::RetainedSources;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    pub MapPainter = {{MapPainter}} {
        quad: {color: #1a1c22}
        label: {text_style: <THEME_FONT_REGULAR> {font_size: 11}, color: #d9dce3}
        code: {
            text_style: <THEME_FONT_CODE> {font_size: 16}, color: #b8c3d2
            uniform scope_camera: vec4 = vec4(0.0, 0.0, 1.0, 1.0);
            uniform scope_clip: vec4 = vec4(0.0, 0.0, 1000000.0, 1000000.0);
            fn vertex(self) -> vec4 {
                let origin = self.rect_pos * self.scope_camera.z + self.scope_camera.xy;
                let extent = max(self.rect_size * self.scope_camera.z, vec2(0.0000001));
                let screen = origin + self.geom_pos * extent;
                let clipped = clamp(screen, self.scope_clip.xy, self.scope_clip.zw);
                let uv = clamp((clipped - origin) / extent, vec2(0.0), vec2(1.0));
                self.pos = uv;
                self.t = self.t_min + (self.t_max - self.t_min) * uv;
                self.world = vec4(clipped, self.glyph_depth + self.draw_zbias, 1.0);
                return self.camera_projection * self.camera_view * self.world;
            }
            fn get_color(self) -> vec4 {
                return vec4(self.color.rgb * self.scope_camera.w, self.color.a);
            }
        }
    }
}
#[derive(Live, LiveHook)]
pub struct MapPainter {
    #[live] pub quad: DrawColor,
    #[live] pub label: DrawText,
    #[live] pub code: DrawText,
    #[rust] pub retained: RetainedSources,
}
impl LiveRegister for MapPainter { fn live_register(_cx: &mut Cx) {} }
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderStats {
    pub visible_nodes: usize,
    pub glyph_runs: usize,
    pub source_lines: usize,
    pub pending_tiles: usize,
    pub built_tiles: usize,
    pub reused_tiles: usize,
    pub retained_bytes: usize,
    pub submitted_runs: usize,
    /// CPU submission wall time, not GPU completion time or FPS.
    pub cpu_submit_ms: f64,
}
impl MapPainter {
    pub fn fill(&mut self, cx: &mut Cx2d, bounds: Box2, color: Vec4) {
        if bounds.w <= 0.0 || bounds.h <= 0.0 { return; }
        self.quad.color = color;
        self.quad.draw_abs(cx, Rect {pos: dvec2(bounds.x,bounds.y), size: dvec2(bounds.w,bounds.h)});
    }
    pub fn text(&mut self, cx: &mut Cx2d, x: f64, y: f64, size: f32, color: Vec4, text: &str) {
        self.label.color = color; self.label.text_style.font_size = size;
        self.label.draw_abs(cx, dvec2(x,y), text);
    }
    pub fn text_right(&mut self, cx: &mut Cx2d, right: f64, y: f64, size: f32, color: Vec4, text: &str) {
        self.label.text_style.font_size = size;
        let width = self.label.layout(cx,0.0,0.0,None,false,Align::default(),text).size_in_lpxs.width;
        self.text(cx,right-width as f64,y,size,color,text);
    }
}
