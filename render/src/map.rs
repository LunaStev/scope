use crate::{MapPainter, RenderStats, palette as p};
use makepad_widgets::*;
use ::model::format::shorten;
use ::layout::Box2;
use ::runtime::Session;
use std::time::Instant;

impl MapPainter {
    pub fn paint_map(&mut self, cx: &mut Cx2d, session: &Session, view: Box2) -> RenderStats {
        let start = Instant::now();
        let mut stats = RenderStats::default();
        let Some(snapshot) = session.snapshot.as_ref() else {
            self.text(cx,view.x+28.0,view.y+40.0,18.0,p::text(),if session.busy(){"Indexing your codebase"}else{"Your codebase, in one view"});
            self.text(cx,view.x+28.0,view.y+78.0,11.0,p::secondary(),&shorten(&session.status,((view.w-56.0)/7.0).max(1.0) as usize));
            return stats;
        };
        let tree = &snapshot.tree;
        if tree.totals().files == 0 {
            self.text(cx,view.x+28.0,view.y+40.0,14.0,p::secondary(),"No readable text files. Check the path, ignores and file-size limit.");
            return stats;
        }
        let mut stack = vec![0];
        let mut visible = Vec::new();
        while let Some(id) = stack.pop() {
            let b = session.camera.project(snapshot.rectangles[id]);
            // Only geometric clipping. No font/zoom threshold, sampling or LOD cap.
            if b.w <= 0.0 || b.h <= 0.0 || !b.intersects(view) { continue; }
            visible.push((id,b));
            stack.extend(tree.nodes[id].children.iter().rev().copied());
        }
        stats.visible_nodes = visible.len();
        self.quad.begin_many_instances(cx);
        for &(id,b) in &visible {
            let node = &tree.nodes[id];
            let base = node.file.as_ref().map(|f|p::language(&f.language)).unwrap_or(p::secondary());
            let dim = if !session.query.is_empty() && !session.matches.contains(&id) && node.file.is_some() {0.20}else{1.0};
            let selected = session.selected == Some(id);
            let hovered = session.hovered == Some(id);
            self.fill(cx,b,if selected{p::accent()}else if hovered{p::text()}else{p::shade(base,0.50*dim)});
            let edge = (b.w.min(b.h)*0.02).min(if selected||hovered{1.5}else{0.75});
            self.fill(cx,b.inset(edge),p::shade(base,if node.file.is_some(){0.14*dim}else{0.045}));
        }
        self.quad.end_many_instances(cx);
        // Actual source is already in the snapshot, including at overview scale.
        for &(id,b) in &visible {
            let node = &tree.nodes[id];
            if session.sources {
                if let (Some(file),Some(page)) = (&node.file,snapshot.pages[id]) {
                    let dim = if !session.query.is_empty()&&!session.matches.contains(&id){0.2}else{1.0};
                    let (lines,runs) = self.draw_document(cx,page,view,session.camera,&file.document,dim);
                    stats.source_lines += lines; stats.glyph_runs += runs;
                }
            }
            // Labels are annotations, not a substitute for the source layer.
            let space = snapshot.pages[id].map(|page|session.camera.project(page.content).y-b.y).unwrap_or((b.h*0.05).min(20.0));
            if b.w>54.0 && space>=12.0 {
                self.text(cx,b.x+5.0,b.y+2.0,10.0,p::secondary(),&shorten(&node.name,((b.w-10.0)/6.7) as usize));
            }
        }
        stats.cpu_submit_ms = start.elapsed().as_secs_f64()*1000.0;
        if std::env::var_os("SCOPE_TRACE").is_some() {
            eprintln!("scope frame: nodes={} lines={} runs={} scale={:.6} representation=source",stats.visible_nodes,stats.source_lines,stats.glyph_runs,session.camera.scale);
        }
        stats
    }
}
