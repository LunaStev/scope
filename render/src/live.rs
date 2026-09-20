//! Retained SDF source over a complete backing image, in fixed world positions.
use makepad_widgets::*;
use crate::{MapPainter,RenderStats,schedule::SourceQueue};
use ::layout::{Box2,Camera};
use ::runtime::{Session,Snapshot};
use std::time::Instant;
live_design! {
    use link::shaders::*;
    pub DrawSourceBase = {{DrawSourceBase}} {
        fn vertex(self) -> vec4 {
            let ax=self.view_transform*vec4(1.0,0.0,0.0,0.0);
            let ay=self.view_transform*vec4(0.0,1.0,0.0,0.0);
            let off=self.view_transform*vec4(0.0,0.0,0.0,1.0);
            let scale=vec2(ax.x,ay.y);
            let origin=self.rect_pos*scale+off.xy;
            let size=max(self.rect_size*scale,vec2(0.0000001));
            let clipped=clamp(origin+size*self.geom_pos,ax.zw,ay.zw);
            self.pos=(clipped-origin)/size;
            self.world=vec4(clipped,self.draw_depth+self.draw_zbias,1.0);
            return self.camera_projection*(self.camera_view*self.world);
        }
        fn pixel(self) -> vec4 {
            let metadata=self.view_transform*vec4(0.0,0.0,1.0,0.0);
            return vec4(self.color.rgb*metadata.z*metadata.w,metadata.w);
        }
    }
}
#[derive(Live,LiveHook,LiveRegister)]
#[repr(C)]
pub struct DrawSourceBase {#[deref] pub draw_super:DrawColor}
#[derive(Default)]
pub struct LiveText {key:Option<[u64;8]>,pub candidates:usize}
/// Prepare at 1.5 physical pixels and blend between 3 and 6, without reflow.
pub fn ink_opacity(font:f64,dpi:f64)->f32{let t=((font*dpi-3.0)/3.0).clamp(0.0,1.0);(t*t*(3.0-2.0*t)) as f32}
impl LiveText {
    fn prepare(&mut self,snapshot:&Snapshot,camera:Camera,view:Box2,dpi:f64,queue:&mut SourceQueue){
        let key=[snapshot.generation,camera.x.to_bits(),camera.y.to_bits(),camera.scale.to_bits(),view.x.to_bits(),view.y.to_bits(),view.w.to_bits(),view.h.to_bits()^dpi.to_bits()];
        if self.key==Some(key){return;}self.key=Some(key);*queue=SourceQueue::default();
        let halo=Box2::new(view.x-view.w*0.12,view.y-view.h*0.12,view.w*1.24,view.h*1.24);
        let(x,y)=camera.unproject(halo.x,halo.y);let world=Box2::new(x,y,halo.w/camera.scale,halo.h/camera.scale);
        let mut ids=snapshot.source_index.query(world,1.5/(camera.scale*dpi.max(0.5)),256);
        let centre=camera.unproject(view.x+view.w*0.5,view.y+view.h*0.5);
        ids.sort_by(|&a,&b|{
            let score=|id:usize|{let p=snapshot.pages[id].as_ref().unwrap();
                let dx=(centre.0-p.content.x).clamp(0.0,p.content.w)+p.content.x-centre.0;
                let dy=(centre.1-p.content.y).clamp(0.0,p.content.h)+p.content.y-centre.1;dx*dx+dy*dy};
            score(a).total_cmp(&score(b)).then(a.cmp(&b))
        });
        self.candidates=ids.len();for id in ids{queue.add(id,snapshot,camera,halo);}
    }
}
impl MapPainter{
    pub(crate)fn paint_live(&mut self,cx:&mut Cx2d,session:&mut Session,snapshot:&Snapshot,view:Box2,stats:&mut RenderStats){
        session.documents.begin_frame();let dpi=cx.current_dpi_factor();let camera=session.camera;
        let mut queue=std::mem::take(&mut self.cache.queue);self.live.prepare(snapshot,camera,view,dpi,&mut queue);
        let frame=self.cache.frame;
        self.cache.chunks.pin_where(frame,|key,chunk|snapshot.pages[key.node].as_ref().is_some_and(|p|ink_opacity(p.font_size*camera.scale,dpi)>0.0)&&camera.project(chunk.world).intersects(view));
        self.prepare_source(cx,&mut queue,session,snapshot,view,Instant::now(),stats);
        self.replay_source(cx,snapshot,camera,view,&session.query,&session.matches,stats);
        stats.live_pending=queue.pending()+queue.waiting();stats.live_glyphs=self.cache.chunks.bytes();
        stats.needs_redraw|=queue.runnable(&session.documents);self.cache.queue=queue;
    }
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn opacity_is_continuous_and_dpi_aware(){assert_eq!(ink_opacity(2.0,1.0),0.0);assert_eq!(ink_opacity(6.0,1.0),1.0);assert_eq!(ink_opacity(3.0,2.0),1.0);let mut previous=0.0;for i in 0..1000{let value=ink_opacity(i as f64/100.0,1.0);assert!(value>=previous);previous=value;}}
}
