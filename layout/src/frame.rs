//! One world-space header definition shared by allocation and label rendering.
use crate::Box2;
#[derive(Clone,Copy,Debug)]pub struct NodeFrame{pub header:Box2,pub content:Box2}
pub fn node_frame(bounds:Box2)->NodeFrame{
    let short=bounds.w.min(bounds.h).max(0.0);let inset=(short*0.005).min(2.0);let inner=bounds.inset(inset);
    let height=(short*0.12).min(48.0).min(inner.h*0.2);
    NodeFrame{header:Box2::new(inner.x,inner.y,inner.w,height),content:Box2::new(inner.x,inner.y+height,inner.w,(inner.h-height).max(0.0))}
}
pub fn world_for_viewport(view:Box2)->Box2{let ratio=((view.w-24.0).max(1.0)/(view.h-24.0).max(1.0)).clamp(0.15,8.0);Box2::new(0.0,0.0,2560.0*ratio,2560.0)}
#[cfg(test)]mod tests{
    use super::*;use crate::Camera;
    #[test]fn headers_never_overlap_children_at_any_scale(){let f=node_frame(Box2::new(4.0,7.0,312.0,987.0));for zoom in [0.001,0.1,1.0,33.0,100000.0]{let c=Camera{scale:zoom,x:-200.0,y:8.0};let h=c.project(f.header);let b=c.project(f.content);assert!((h.y+h.h-b.y).abs()<1e-5);}}
    #[test]fn overview_fits_bottom_and_sides(){for(w,h)in[(1200.0,600.0),(800.0,1100.0),(1600.0,900.0)]{let v=Box2::new(0.0,113.0,w,h);let root=world_for_viewport(v);let mut c=Camera::default();c.fit(root,v);let p=c.project(root);assert!((p.x-v.x-12.0).abs()<1e-5);assert!((p.y+p.h-(v.y+v.h-12.0)).abs()<1e-5);}}
}
