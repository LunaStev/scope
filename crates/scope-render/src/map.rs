use crate::{MapPainter,RenderStats,geometry::code_geometry,palette as p};
use makepad_widgets::*;
use scope_core::format::shorten;
use scope_layout::Box2;
use scope_runtime::Session;
use std::time::Instant;

impl MapPainter{
    pub fn paint_map(&mut self,cx:&mut Cx2d,session:&mut Session,view:Box2)->RenderStats{
        let start=Instant::now();let mut stats=RenderStats::default();
        let Some(snapshot)=session.snapshot.as_ref()else{
            self.text(cx,view.x+28.0,view.y+40.0,18.0,p::text(),if session.busy(){"Indexing your codebase"}else{"Your codebase, in one view"});
            self.text(cx,view.x+28.0,view.y+78.0,11.0,p::secondary(),&shorten(&session.status,((view.w-56.0)/7.0).max(1.0) as usize));
            return stats;
        };
        let tree=snapshot.tree.clone();
        if tree.totals().files==0{
            self.text(cx,view.x+28.0,view.y+40.0,14.0,p::secondary(),"No readable text files. Check the path, ignores and file-size limit.");return stats;
        }
        let mut stack=vec![0];let mut visible=Vec::new();
        while let Some(id)=stack.pop(){
            let b=session.camera.project(snapshot.rectangles[id]);
            if b.w<0.6||b.h<0.6||!b.intersects(view){continue;}
            visible.push((id,b));
            if visible.len()>=30000{stats.capped=true;break;}
            stack.extend(tree.nodes[id].children.iter().rev().copied());
        }
        stats.visible_nodes=visible.len();
        // One background batch, followed by glyphs. This prevents later quad
        // instances from covering text when Makepad combines draw calls.
        self.quad.begin_many_instances(cx);
        for &(id,b) in &visible{
            let node=&tree.nodes[id];
            let base=node.file.as_ref().map(|f|p::language(&f.language)).unwrap_or(p::secondary());
            let dim=if !session.query.is_empty()&&!session.matches.contains(&id)&&node.file.is_some(){0.20}else{1.0};
            let selected=session.selected==Some(id);let hovered=session.hovered==Some(id);
            self.fill(cx,b,if selected{p::accent()}else if hovered{p::text()}else{p::shade(base,0.50*dim)});
            self.fill(cx,b.inset(if selected||hovered{1.5}else{0.75}),p::shade(base,if node.file.is_some(){0.14*dim}else{0.045}));
            if node.file.is_some()&&b.w>35.0&&b.h>30.0{
                self.fill(cx,Box2::new(b.x+1.5,b.y+1.5,(b.w-3.0).max(0.0),17.0),p::shade(base,0.21*dim));
            }
            if session.sources{
                if let Some(file)=&node.file{
                    let(_,_,content,font)=code_geometry(b,node.stats.lines as usize);
                    if b.w>=120.0&&b.h>=70.0&&font>=7.0&&session.cache.contains(id){self.fill(cx,content,p::background());}
                    else if b.w>10.0&&b.h>14.0{self.draw_preview(cx,b,file,node.stats.lines as usize,p::shade(base,0.85*dim));}
                }
            }
        }
        self.quad.end_many_instances(cx);
        let mut budget=6500usize;let mut requested=0;
        for &(id,b) in &visible{
            let node=&tree.nodes[id];
            let header=if node.file.is_some(){18.0}else{(b.h*0.07).min(27.0*session.camera.scale)};
            if b.w>54.0&&b.h>22.0&&header>=12.0{
                self.text(cx,b.x+5.0,b.y+3.0,10.0,if node.file.is_some(){p::secondary()}else{p::text()},&shorten(&node.name,((b.w-10.0)/6.7) as usize));
            }
            if !session.sources||node.file.is_none()||b.w<120.0||b.h<70.0{continue;}
            let(_,_,_,font)=code_geometry(b,node.stats.lines as usize);if font<7.0{continue;}
            if let Some(doc)=session.cache.get(id){self.draw_document(cx,b,view,&doc,&mut budget);}
            else if requested<3{session.request_source(id);requested+=1;}
        }
        stats.glyph_runs=6500-budget;stats.capped|=budget==0;stats.cpu_submit_ms=start.elapsed().as_secs_f64()*1000.0;
        stats
    }
}
