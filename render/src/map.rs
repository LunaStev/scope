use crate::{MapPainter,RenderStats,palette as p};
use makepad_widgets::*;
use ::model::format::shorten;
use ::layout::Box2;
use ::runtime::Session;
use std::time::Instant;

impl MapPainter {
    pub fn paint_map(&mut self,cx:&mut Cx2d,session:&Session,view:Box2)->RenderStats {
        let start=Instant::now();let mut stats=RenderStats::default();
        let Some(snapshot)=session.snapshot.as_ref()else{
            self.cache.clear();
            self.text(cx,view.x+28.0,view.y+40.0,18.0,p::text(),if session.busy(){"Indexing your codebase"}else{"Open a codebase"});
            self.text(cx,view.x+28.0,view.y+78.0,11.0,p::secondary(),&shorten(&session.status,((view.w-56.0)/7.0).max(1.0) as usize));return stats;
        };
        self.cache.prepare(snapshot.generation,cx.current_dpi_factor());
        let tree=&snapshot.tree;
        if tree.totals().files==0 {
            self.text(cx,view.x+28.0,view.y+40.0,14.0,p::secondary(),"No readable text files. Check the path, ignores and file-size limit.");return stats;
        }
        let mut stack=vec![0];let mut visible=Vec::new();
        while let Some(id)=stack.pop(){
            let b=session.camera.project(snapshot.rectangles[id]);
            if b.w<=0.0||b.h<=0.0||!b.intersects(view){continue;}
            visible.push((id,b));stack.extend(tree.nodes[id].children.iter().rev().copied());
        }
        stats.visible_nodes=visible.len();
        self.quad.begin_many_instances(cx);
        for &(id,b)in &visible {
            let node=&tree.nodes[id];
            let base=node.file.as_ref().map(|f|p::language(&f.language)).unwrap_or(p::secondary());
            let dim=if !session.query.is_empty()&&!session.matches.contains(&id)&&node.file.is_some(){0.2}else{1.0};
            let selected=session.selected==Some(id)&&id!=0;let hovered=session.hovered==Some(id);
            self.fill(cx,b,if selected{p::accent()}else if hovered{p::secondary()}else{p::shade(base,0.40*dim)});
            let edge=(b.w.min(b.h)*0.02).min(if selected||hovered{1.2}else{0.65});
            self.fill(cx,b.inset(edge),p::shade(base,if node.file.is_some(){0.12*dim}else{0.07}));
        }
        self.quad.end_many_instances(cx);
        for &(id,b)in &visible {
            let node=&tree.nodes[id];
            if session.sources {
                if let(Some(file),Some(page))=(&node.file,snapshot.pages[id].as_ref()){
                    let dim=if !session.query.is_empty()&&!session.matches.contains(&id){0.2}else{1.0};
                    self.draw_document(cx,id,page,view,session.camera,&file.document,dim,start,&mut stats);
                }
            }
            let space=snapshot.pages[id].as_ref().map(|page|session.camera.project(page.content).y-b.y).unwrap_or((b.h*0.04).min(19.0));
            if b.w>54.0&&space>=12.0 {
                self.text(cx,b.x+4.0,b.y+2.0,10.0,p::secondary(),&shorten(&node.name,((b.w-8.0)/6.7) as usize));
            }
        }
        stats.cached_chunks=self.cache.chunks.len();
        stats.cpu_submit_ms=start.elapsed().as_secs_f64()*1000.0;
        if std::env::var_os("SCOPE_TRACE").is_some(){
            eprintln!("scope frame: nodes={} lines={} runs={} scale={:.6} font_min={:.4} font_max={:.4} reused={} built={} pending={} cpu_ms={:.3} representation=source",stats.visible_nodes,stats.source_lines,stats.glyph_runs,session.camera.scale,stats.min_font,stats.max_font,stats.reused_chunks,stats.built_chunks,stats.pending_chunks,stats.cpu_submit_ms);
        }
        stats
    }
}
