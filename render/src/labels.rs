//! Header placement is shared with allocation. Cached label batches are only
//! rebuilt for a new camera/viewport/scene, not for each arriving source file.
use crate::{MapPainter,palette as p};
use makepad_widgets::*;
use ::layout::{Box2,Camera,node_frame};
use ::model::{Node,format};
impl MapPainter {
    pub(crate) fn node_label(&mut self,cx:&mut Cx2d,node:&Node,bounds:Box2,camera:Camera,view:Box2)->bool {
        let header=camera.project(node_frame(bounds).header);let Some(visible)=header.intersection(view)else{return false;};
        let size=(header.h/1.6).min(10.0);if size<6.0||visible.w<20.0||visible.h<size*1.25{return false;}
        let x=header.x.max(view.x)+3.0;let y=header.y+((header.h-size*1.4)*0.5).max(0.0);
        if y<visible.y-0.5||y+size*1.3>visible.y+visible.h{return false;}
        cx.begin_turtle(Walk{abs_pos:Some(dvec2(visible.x,visible.y)),width:Size::Fixed(visible.w),height:Size::Fixed(visible.h),..Walk::default()},Layout{clip_x:true,clip_y:true,..Layout::default()});
        let right=visible.x+visible.w-3.0;
        let metric=if node.file.is_none(){format!("{} files · {} lines",format::count(node.stats.files),format::count(node.stats.lines))}else{format!("{} lines",format::count(node.stats.lines))};
        self.label.text_style.font_size=size as f32;self.label.font_scale=1.0;
        let label=self.label.layout(cx,0.0,0.0,None,false,Align::default(),&node.name);
        let number=self.label.layout(cx,0.0,0.0,None,false,Align::default(),&metric);
        let show_count=right-x>label.size_in_lpxs.width as f64+number.size_in_lpxs.width as f64+22.0;
        let available=right-x-if show_count{number.size_in_lpxs.width as f64+16.0}else{0.0};let mut title=node.name.clone();
        if label.size_in_lpxs.width as f64>available {
            let ends:Vec<usize>=node.name.char_indices().map(|(i,_)|i).chain(std::iter::once(node.name.len())).collect();let(mut lo,mut hi)=(0,ends.len()-1);
            while lo<hi{let mid=(lo+hi).div_ceil(2);let candidate=format!("{}…",&node.name[..ends[mid]]);let width=self.label.layout(cx,0.0,0.0,None,false,Align::default(),&candidate).size_in_lpxs.width as f64;if width<=available{lo=mid;}else{hi=mid-1;}}
            title=if lo==0{String::new()}else{format!("{}…",&node.name[..ends[lo]])};
        }
        let alpha=((size-6.0)/2.0).clamp(0.0,1.0) as f32;let mut ink=p::text();ink.w=alpha;
        self.text(cx,x,y,size as f32,ink,&title);
        if show_count{let mut ink=p::muted();ink.w=alpha;self.text_right(cx,right,y,size as f32,ink,&metric);}
        cx.end_turtle(); !title.is_empty()||show_count
    }
    pub(crate) fn outline_clipped(&mut self,cx:&mut Cx2d,b:Box2,view:Box2,color:Vec4) {
        let edge=(b.w.min(b.h)*0.02).min(1.2);
        for r in [Box2::new(b.x,b.y,b.w,edge),Box2::new(b.x,b.y+b.h-edge,b.w,edge),Box2::new(b.x,b.y,edge,b.h),Box2::new(b.x+b.w-edge,b.y,edge,b.h)] {
            if let Some(r)=r.intersection(view){self.fill(cx,r,color);}
        }
    }
}
