//! A resumable visibility walk and retained screen-space chrome. Source-ready
//! events do not rebuild the node boxes or reshape all their labels.
use makepad_widgets::*;
use ::layout::{Box2,Camera};
use ::runtime::Snapshot;
use crate::{MapPainter,RenderStats,palette as p,budget};
use std::time::{Duration,Instant};

#[derive(PartialEq)]
struct ViewKey { generation:u64, coordinates:[u64;7], dpi:u64, query:String }
#[derive(Default)]
pub struct Presentation {
    key:Option<ViewKey>,
    root_pending:bool,
    stack:Vec<(usize,usize)>,
    pub visible:Vec<(usize,Box2)>,
    backgrounds:Vec<DrawList2d>,
    labels:Vec<DrawList2d>,
    label_at:usize,
    pub limited:bool,
}
impl Presentation {
    pub fn prepare(&mut self,snapshot:&Snapshot,camera:Camera,view:Box2,dpi:f64,query:&str)->bool {
        let key=ViewKey{generation:snapshot.generation,coordinates:[camera.x.to_bits(),camera.y.to_bits(),camera.scale.to_bits(),view.x.to_bits(),view.y.to_bits(),view.w.to_bits(),view.h.to_bits()],dpi:dpi.to_bits(),query:query.into()};
        if self.key.as_ref()==Some(&key) { return false; }
        *self=Self{key:Some(key),root_pending:true,..Self::default()}; true
    }
    pub fn discovering(&self)->bool { self.root_pending || !self.stack.is_empty() }
    pub fn unfinished(&self)->bool { self.discovering() || self.label_at<self.visible.len() }
    pub fn replay_backgrounds(&mut self,cx:&mut Cx2d,stats:&mut RenderStats) {
        for list in &mut self.backgrounds { let _=list.begin_maybe(cx,false); stats.reused_layers+=1; }
    }
    pub fn replay_labels(&mut self,cx:&mut Cx2d,stats:&mut RenderStats) {
        for list in &mut self.labels { let _=list.begin_maybe(cx,false); stats.reused_layers+=1; }
    }
    /// At most one tree edge is examined per call. Even a directory containing
    /// a million children cannot grow the DFS stack in one unbounded extend().
    fn step(&mut self,snapshot:&Snapshot,camera:Camera,view:Box2,pixel:f64)->Option<(usize,Box2)> {
        let id=if self.root_pending { self.root_pending=false; 0 } else {
            let &(parent,index)=self.stack.last()?;
            let children=&snapshot.tree.nodes[parent].children;
            if index>=children.len() { self.stack.pop(); return None; }
            self.stack.last_mut().unwrap().1+=1; children[index]
        };
        let bounds=camera.project(snapshot.rectangles[id]);
        if !bounds.intersects(view)||bounds.w<pixel||bounds.h<pixel { return None; }
        if !snapshot.tree.nodes[id].children.is_empty() { self.stack.push((id,0)); }
        Some((id,bounds))
    }
}
impl MapPainter {
    pub(crate) fn advance_nodes(&mut self,cx:&mut Cx2d,scene:&mut Presentation,snapshot:&Snapshot,camera:Camera,view:Box2,query:&str,matches:&std::collections::HashSet<usize>,stats:&mut RenderStats)->Vec<usize> {
        let start=Instant::now(); let from=scene.visible.len(); let mut files=Vec::new();
        let pixel=0.65/cx.current_dpi_factor().max(0.1);
        while scene.discovering()&&stats.node_visits<budget::NODE_VISITS&&start.elapsed()<Duration::from_millis(2) {
            stats.node_visits+=1;
            if let Some((id,b))=scene.step(snapshot,camera,view,pixel) {
                scene.visible.push((id,b));
                if snapshot.tree.nodes[id].file.is_some() { files.push(id); }
                if scene.visible.len()>=50_000 { scene.stack.clear(); scene.root_pending=false; scene.limited=true; break; }
            }
        }
        if scene.visible.len()>from {
            let mut list=DrawList2d::new(cx); list.begin_always(cx);
            self.node.begin_many_instances(cx);
            for &(id,b) in &scene.visible[from..] {
                let node=&snapshot.tree.nodes[id];
                let base=node.file.as_ref().map(|f|p::language(&f.language)).unwrap_or(p::secondary());
                let dim=if !query.is_empty()&&!matches.contains(&id)&&node.file.is_some(){0.2}else{1.0};
                self.node.clipped(cx,b,view,p::shade(base,if node.file.is_some(){0.12*dim}else{0.07}),p::shade(base,0.4*dim),(b.w.min(b.h)*0.02).min(0.65));
            }
            self.node.end_many_instances(cx); list.end(cx); scene.backgrounds.push(list);
            stats.built_layers+=1;
        }
        files
    }
    pub(crate) fn advance_labels(&mut self,cx:&mut Cx2d,scene:&mut Presentation,snapshot:&Snapshot,camera:Camera,view:Box2,stats:&mut RenderStats) {
        if scene.label_at>=scene.visible.len() { return; }
        let start=Instant::now(); let mut list=DrawList2d::new(cx); list.begin_always(cx);
        // Empty label lists must not be replayed with an unmatched begin/end.
        let mut painted=false; let mut visited=0;
        while scene.label_at<scene.visible.len()&&visited<budget::LABEL_VISITS&&start.elapsed()<Duration::from_millis(1) {
            let id=scene.visible[scene.label_at].0; scene.label_at+=1; visited+=1;
            painted|=self.node_label(cx,&snapshot.tree.nodes[id],snapshot.rectangles[id],camera,view);
        }
        list.end(cx); stats.label_visits+=visited;
        if painted { scene.labels.push(list); stats.built_layers+=1; }
    }
}
