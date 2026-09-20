use crate::{MapPainter,RenderStats,palette as p,presentation::Presentation};
use makepad_widgets::*;
use ::model::format::{shorten,count};
use ::layout::Box2;
use ::runtime::Session;
use std::{time::Instant,sync::Arc,collections::HashSet};
impl MapPainter{
    pub fn paint_map(&mut self,cx:&mut Cx2d,session:&mut Session,view:Box2)->RenderStats{
        let started=Instant::now();let mut stats=RenderStats::default();
        let Some(snapshot)=session.snapshot.clone()else{
            if self.maps.generation!=0{self.maps.reset();self.images.clear();self.cache.clear();}
            self.text(cx,view.x+24.0,view.y+32.0,16.0,p::text(),if session.busy(){"Indexing codebase"}else{"Open a codebase"});
            self.text(cx,view.x+24.0,view.y+66.0,11.0,p::secondary(),&shorten(&session.status,((view.w-48.0)/7.0).max(1.0) as usize));return stats;
        };
        if self.maps.generation!=snapshot.generation{
            match cx.get_dependency(self.map_font.as_str()){
                Ok(font)=>{self.maps.start(snapshot.clone(),Arc::new(font.as_ref().clone()),session.options.max_file_bytes,SignalToUI::set_ui_signal);self.images.clear();self.images.generation=snapshot.generation;self.cache.clear();}
                Err(error)=>{self.text(cx,view.x+24.0,view.y+32.0,12.0,p::unknown(),&shorten(&error,100));return stats;}
            }
        }
        self.maps.poll();let camera=session.camera;let world=snapshot.rectangles[0];let dpi=cx.current_dpi_factor();
        let desired=if session.sources{raster::tile::requests(world,camera,view,dpi)}else{Vec::new()};let protected:HashSet<_>=desired.iter().copied().collect();
        self.maps.request(&desired,&self.images.resident());
        if let Some(update)=self.maps.take_image(){self.images.upload(cx,update,world,&protected);stats.texture_uploads=1;}
        let(done,total)=self.maps.progress();stats.map_completed=done;stats.map_total=total;stats.map_ready=self.images.ready();stats.map_cache_hit=self.maps.cache_hit;stats.map_errors=self.maps.errors;
        if !self.images.ready(){
            self.text(cx,view.x+24.0,view.y+32.0,16.0,p::text(),"Preparing the complete code map");
            let message=self.maps.error.clone().unwrap_or_else(||format!("{} / {} files · navigation opens with a complete overview",count(done),count(total)));
            self.text(cx,view.x+24.0,view.y+66.0,11.0,p::secondary(),&shorten(&message,((view.w-48.0)/7.0).max(1.0) as usize));stats.needs_redraw=self.maps.has_images();return stats;
        }
        if self.images.source_mode!=session.sources{self.images.source_mode=session.sources;self.cache.scene=Presentation::default();self.live=crate::live::LiveText::default();}
        if session.sources{
            let(count,fading)=self.images.draw(cx,&mut self.image,camera,view,1.0);
            stats.image_tiles=count;stats.reused_chunks=count;stats.source_lines=self.maps.prepared_lines as usize;stats.needs_redraw=fading||self.maps.has_images();
        }
        self.cache.prepare(snapshot.generation,dpi);
        if session.sources{self.paint_live(cx,session,&snapshot,view,&mut stats);stats.needs_redraw|=stats.live_fading;}
        stats.source_lines=self.maps.prepared_lines as usize;
        let mut scene=std::mem::take(&mut self.cache.scene);scene.prepare(&snapshot,camera,view,dpi,&session.query);
        if session.sources{scene.discover_labels(&snapshot,camera,view,dpi,&mut stats);}else{scene.replay_backgrounds(cx,&mut stats);let _=self.advance_nodes(cx,&mut scene,&snapshot,camera,view,&session.query,&session.matches,&mut stats);}
        scene.replay_labels(cx,&mut stats);self.advance_labels(cx,&mut scene,&snapshot,camera,view,&mut stats);
        self.quad.begin_many_instances(cx);
        for(id,color)in[(session.hovered,p::secondary()),(session.selected.filter(|&id|id!=0),p::accent())]{if let Some(id)=id{if let Some(&b)=snapshot.rectangles.get(id){self.outline_clipped(cx,camera.project(b),view,color);}}}
        if !session.query.is_empty(){for &(id,b)in scene.visible.iter().filter(|(id,_)|session.matches.contains(id)).take(128){let _=id;self.outline_clipped(cx,b,view,p::accent());}}
        self.quad.end_many_instances(cx);stats.visible_nodes=scene.visible.len();stats.needs_redraw|=scene.unfinished();self.cache.scene=scene;
        stats.image_pending=self.maps.pending();stats.cached_chunks=self.images.len();
        let sample=session.selected.and_then(|id|snapshot.pages.get(id)).and_then(Option::as_ref).or_else(||snapshot.pages.iter().flatten().next());
        if stats.max_font==0.0{if let Some(page)=sample{stats.min_font=page.font_size*camera.scale;stats.max_font=stats.min_font;}}
        stats.cpu_submit_ms=started.elapsed().as_secs_f64()*1000.0;
        if std::env::var_os("SCOPE_TRACE").is_some(){eprintln!("scope frame: nodes={} lines={} runs={} scale={:.6} font_min={:.4} font_max={:.4} reused={} built={} pending={} waiting=0 limited=0 glyphs={} cpu_ms={:.3} map_ready=1 image_tiles={} uploads={} map_pending={} map_errors={} map_cache_hit={} map_prepare_ms={} redraw={} representation=hybrid-sdf native_glyphs={} native_resident={} native_reused={} native_pending={} map_replans={} cancelled_details={} native_queries={}",stats.visible_nodes,stats.source_lines,stats.glyph_runs,camera.scale,stats.min_font,stats.max_font,stats.reused_chunks,stats.built_chunks,stats.image_pending,stats.built_glyphs,stats.cpu_submit_ms,stats.image_tiles,stats.texture_uploads,stats.image_pending,stats.map_errors,usize::from(stats.map_cache_hit),self.maps.prepare_ms,usize::from(stats.needs_redraw),stats.live_drawn,stats.live_glyphs,stats.live_reused,stats.live_pending,self.maps.replans,self.maps.cancelled_details,self.live.queries);}
        stats
    }
}
