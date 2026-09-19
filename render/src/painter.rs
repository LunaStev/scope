use makepad_widgets::*;
use ::layout::Box2;
use crate::cache::GeometryCache;
live_design! {
    use link::theme::*;
    use link::shaders::*;
    pub MapPainter = {{MapPainter}} {
        quad: {color: #171b20}
        label: {text_style: <THEME_FONT_REGULAR> {font_size: 11}, color: #d9dfe4}
        code: {
            text_style: <THEME_FONT_CODE> {font_size: 16}
            color: #bcc5cf
            fn vertex(self) -> vec4 {
                let axis_x = self.view_transform * vec4(1.0,0.0,0.0,0.0);
                let axis_y = self.view_transform * vec4(0.0,1.0,0.0,0.0);
                let offset = self.view_transform * vec4(0.0,0.0,0.0,1.0);
                let scale = vec2(axis_x.x,axis_y.y);
                let origin = self.rect_pos * scale + offset.xy;
                let size = max(self.rect_size * scale,vec2(0.0000001));
                let pos = origin + size * self.geom_pos;
                let clipped = clamp(pos,axis_x.zw,axis_y.zw);
                self.pos = (clipped-origin)/size;
                self.t = mix(self.t_min,self.t_max,self.pos);
                self.world = vec4(clipped,self.glyph_depth+self.draw_zbias,1.0);
                return self.camera_projection * (self.camera_view * self.world);
            }
            fn get_color(self) -> vec4 {
                let metadata = self.view_transform * vec4(0.0,0.0,1.0,0.0);
                return vec4(self.color.rgb*metadata.z,self.color.a);
            }
        }
    }
}
#[derive(Live)]
pub struct MapPainter {
    #[live] pub quad: DrawColor,
    #[live] pub label: DrawText,
    #[live] pub code: DrawText,
    #[rust] pub cache: GeometryCache,
}
impl LiveHook for MapPainter {fn after_apply(&mut self,_cx:&mut Cx,_apply:&mut Apply,_index:usize,_nodes:&[LiveNode]){self.cache.clear();}}
impl LiveRegister for MapPainter {fn live_register(_cx:&mut Cx){}}
#[derive(Clone,Copy,Debug,Default)]
pub struct RenderStats{
    pub visible_nodes:usize,pub glyph_runs:usize,pub source_lines:usize,pub reused_chunks:usize,
    pub built_chunks:usize,pub pending_chunks:usize,pub cached_chunks:usize,pub waiting_sources:usize,
    pub limited_chunks:usize,pub limited_nodes:usize,pub source_errors:usize,pub resident_glyphs:usize,
    pub min_font:f64,pub max_font:f64,pub cpu_submit_ms:f64,
}
impl MapPainter{
    pub fn fill(&mut self,cx:&mut Cx2d,b:Box2,color:Vec4){if b.w<=0.0||b.h<=0.0{return;}self.quad.color=color;self.quad.draw_abs(cx,Rect{pos:dvec2(b.x,b.y),size:dvec2(b.w,b.h)});}
    pub fn text(&mut self,cx:&mut Cx2d,x:f64,y:f64,size:f32,color:Vec4,text:&str){self.label.color=color;self.label.text_style.font_size=size;self.label.font_scale=1.0;self.label.draw_abs(cx,dvec2(x,y),text);}
    pub fn text_right(&mut self,cx:&mut Cx2d,right:f64,y:f64,size:f32,color:Vec4,text:&str){self.label.text_style.font_size=size;let measured=self.label.layout(cx,0.0,0.0,None,false,Align::default(),text);self.text(cx,right-measured.size_in_lpxs.width as f64,y,size,color,text);}
}
