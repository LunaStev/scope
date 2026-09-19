//! One clipped instance per node, instead of a fill plus four border quads.
use makepad_widgets::*;
use ::layout::Box2;

live_design! {
    use link::shaders::*;
    pub DrawNode = {{DrawNode}} {
        color: #171b20
        border: #353e45
        edges: vec4(0.65)
        fn pixel(self) -> vec4 {
            let p = self.pos * self.rect_size;
            let inside = step(self.edges.x,p.x) * step(self.edges.y,p.y)
                * step(self.edges.z,self.rect_size.x-p.x)
                * step(self.edges.w,self.rect_size.y-p.y);
            return mix(self.border,self.color,inside);
        }
    }
}
#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
pub struct DrawNode {
    #[deref] pub draw_super: DrawColor,
    #[live] pub border: Vec4,
    #[live] pub edges: Vec4,
}
impl DrawNode {
    pub fn clipped(&mut self,cx:&mut Cx2d,b:Box2,view:Box2,fill:Vec4,border:Vec4,edge:f64) {
        let Some(r)=b.intersection(view) else { return; };
        self.color=fill; self.border=border;
        // A viewport cut through the middle of a box is NOT a new box edge.
        self.edges=vec4(
            if b.x>=view.x {edge as f32}else{0.0},
            if b.y>=view.y {edge as f32}else{0.0},
            if b.x+b.w<=view.x+view.w {edge as f32}else{0.0},
            if b.y+b.h<=view.y+view.h {edge as f32}else{0.0},
        );
        self.draw_abs(cx,Rect{pos:dvec2(r.x,r.y),size:dvec2(r.w,r.h)});
    }
}
