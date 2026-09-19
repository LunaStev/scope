use crate::{MapPainter,RenderStats,palette as p};
use makepad_widgets::*;
use ::model::format::shorten;
use ::layout::Box2;
use ::runtime::Session;
use std::time::Instant;
impl MapPainter{
    pub fn paint_map(&mut self,cx:&mut Cx2d,session:&mut Session,view:Box2)->RenderStats{
        let started=Instant::now();let mut stats=RenderStats::default();
        let Some(snapshot)=session.snapshot.clone()else{self.cache.clear();self.text(cx,view.x+24.0,view.y+32.0,16.0,p::text(),if session.busy(){"Indexing codebase"}else{"Open a codebase"});self.text(cx,view.x+24.0,view.y+66.0,11.0,p::secondary(),&shorten(&session.status,((view.w-48.0)/7.0).max(1.0) as usize));return stats;};
        self.cache.prepare(snapshot.generation,cx.current_dpi_factor());session.documents.begin_frame();let camera=session.camera;
        self.cache.chunks.pin_where(self.cache.frame,|_,c|camera.project(c.world).intersects(view));
        let tree=&snapshot.tree;let mut stack=vec![0];let mut visible=Vec::new();let pixel=0.65/cx.current_dpi_factor().max(0.1);
        while let Some(id)=stack.pop(){let b=session.camera.project(snapshot.rectangles[id]);if !b.intersects(view)||b.w<pixel||b.h<pixel{continue;}visible.push((id,b));if visible.len()>=50000{stats.limited_nodes+=stack.len();break;}stack.extend(tree.nodes[id].children.iter().rev().copied());}
        stats.visible_nodes=visible.len();self.quad.begin_many_instances(cx);
        for &(id,b)in &visible{
            let node=&tree.nodes[id];let base=node.file.as_ref().map(|f|p::language(&f.language)).unwrap_or(p::secondary());
            let dim=if !session.query.is_empty()&&!session.matches.contains(&id)&&node.file.is_some(){0.2}else{1.0};
            let selected=session.selected==Some(id)&&id!=0;let hovered=session.hovered==Some(id);let edge=(b.w.min(b.h)*0.02).min(if selected||hovered{1.2}else{0.65});
            self.box_clipped(cx,b,view,p::shade(base,if node.file.is_some(){0.12*dim}else{0.07}),if selected{p::accent()}else if hovered{p::secondary()}else{p::shade(base,0.4*dim)},edge);
        }
        self.quad.end_many_instances(cx);
        if session.sources{
            // Fair cold work instead of starving code below the first big file.
            let mut files:Vec<usize>=visible.iter().filter_map(|(id,_)|tree.nodes[*id].file.as_ref().map(|_|*id)).collect();
            if !files.is_empty(){let offset=self.cache.cursor%files.len();files.rotate_left(offset);self.cache.cursor+=1;}
            if let Some(selected)=session.selected{if let Some(at)=files.iter().position(|&id|id==selected){files.swap(0,at);}}
            for id in files{if let(Some(file),Some(page))=(&tree.nodes[id].file,snapshot.pages[id].as_ref()){
                let dim=if !session.query.is_empty()&&!session.matches.contains(&id){0.2}else{1.0};
                self.draw_document(cx,id,page,view,session.camera,file,&tree.root.join(&tree.nodes[id].relative),session.options.max_file_bytes,&mut session.documents,dim,started,&mut stats);
            }}
        }
        for &(id,_)in &visible{self.node_label(cx,&tree.nodes[id],snapshot.rectangles[id],session.camera,view);}
        if stats.limited_chunks>0&&stats.built_chunks==0{stats.pending_chunks=0;}
        stats.cached_chunks=self.cache.chunks.len();stats.resident_glyphs=self.cache.chunks.bytes();stats.cpu_submit_ms=started.elapsed().as_secs_f64()*1000.0;
        if std::env::var_os("SCOPE_TRACE").is_some(){eprintln!("scope frame: nodes={} lines={} runs={} scale={:.6} font_min={:.4} font_max={:.4} reused={} built={} pending={} waiting={} limited={} glyphs={} cpu_ms={:.3} representation=source",stats.visible_nodes,stats.source_lines,stats.glyph_runs,session.camera.scale,stats.min_font,stats.max_font,stats.reused_chunks,stats.built_chunks,stats.pending_chunks,stats.waiting_sources,stats.limited_chunks,stats.resident_glyphs,stats.cpu_submit_ms);}stats
    }
}
