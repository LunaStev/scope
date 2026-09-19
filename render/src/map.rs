use crate::{MapPainter,RenderStats,palette as p,schedule::SourceQueue};
use makepad_widgets::*;
use ::model::format::shorten;
use ::layout::Box2;
use ::runtime::Session;
use std::time::Instant;

impl MapPainter {
    pub fn paint_map(&mut self,cx:&mut Cx2d,session:&mut Session,view:Box2)->RenderStats {
        let started=Instant::now();let mut stats=RenderStats::default();
        let Some(snapshot)=session.snapshot.clone() else {
            self.cache.clear();
            self.text(cx,view.x+24.0,view.y+32.0,16.0,p::text(),if session.busy(){"Indexing codebase"}else{"Open a codebase"});
            self.text(cx,view.x+24.0,view.y+66.0,11.0,p::secondary(),&shorten(&session.status,((view.w-48.0)/7.0).max(1.0) as usize));
            return stats;
        };
        let camera=session.camera;
        self.cache.prepare(snapshot.generation,cx.current_dpi_factor());
        session.documents.begin_frame();
        let mut scene=std::mem::take(&mut self.cache.scene);
        let mut queue=std::mem::take(&mut self.cache.queue);
        if scene.prepare(&snapshot,camera,view,cx.current_dpi_factor(),&session.query) {queue=SourceQueue::default();}
        scene.replay_backgrounds(cx,&mut stats);
        let files=self.advance_nodes(cx,&mut scene,&snapshot,camera,view,&session.query,&session.matches,&mut stats);
        for id in files {queue.add(id,&snapshot,camera,view);}
        if session.sources {
            // Pin only resident geometry, before any eviction. No missing-tile
            // enumeration is needed for this bounded residency pass.
            self.cache.chunks.pin_where(self.cache.frame,|_,c|camera.project(c.world).intersects(view));
            self.prepare_source(cx,&mut queue,session,&snapshot,view,started,&mut stats);
            self.replay_source(cx,&snapshot,camera,view,&session.query,&session.matches,&mut stats);
            stats.pending_chunks=if queue.limited{0}else{queue.pending()};
            stats.waiting_sources=if queue.limited{0}else{queue.waiting()};
            stats.limited_chunks=if queue.limited{queue.pending().max(1)}else{0};
            stats.source_errors=queue.errors;
            stats.needs_redraw=queue.runnable(&session.documents);
        }
        scene.replay_labels(cx,&mut stats);
        self.advance_labels(cx,&mut scene,&snapshot,camera,view,&mut stats);
        self.quad.begin_many_instances(cx);
        for (id,color) in [(session.hovered,p::secondary()),(session.selected.filter(|&id|id!=0),p::accent())] {
            if let Some(id)=id {if let Some(&b)=snapshot.rectangles.get(id){self.outline_clipped(cx,camera.project(b),view,color);}}
        }
        self.quad.end_many_instances(cx);
        stats.visible_nodes=scene.visible.len();stats.limited_nodes=usize::from(scene.limited);
        stats.needs_redraw|=scene.unfinished();
        self.cache.scene=scene;self.cache.queue=queue;
        stats.cached_chunks=self.cache.chunks.len();stats.resident_glyphs=self.cache.chunks.bytes();
        stats.cpu_submit_ms=started.elapsed().as_secs_f64()*1000.0;
        if std::env::var_os("SCOPE_TRACE").is_some() {
            eprintln!("scope frame: nodes={} lines={} runs={} scale={:.6} font_min={:.4} font_max={:.4} reused={} built={} pending={} waiting={} limited={} glyphs={} cpu_ms={:.3} node_visits={} label_visits={} checks={} new_glyphs={} layers_built={} layers_reused={} redraw={} representation=source",stats.visible_nodes,stats.source_lines,stats.glyph_runs,camera.scale,stats.min_font,stats.max_font,stats.reused_chunks,stats.built_chunks,stats.pending_chunks,stats.waiting_sources,stats.limited_chunks,stats.resident_glyphs,stats.cpu_submit_ms,stats.node_visits,stats.label_visits,stats.tile_checks,stats.built_glyphs,stats.built_layers,stats.reused_layers,usize::from(stats.needs_redraw));
        }
        stats
    }
}
