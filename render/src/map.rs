use crate::{MapPainter, RenderStats, retained::CachedTile, tiles, palette as p};
use makepad_widgets::*;
use ::model::format::shorten;
use ::layout::Box2;
use ::runtime::Session;
use std::time::Instant;

impl MapPainter {
    pub fn paint_map(&mut self,cx:&mut Cx2d,session:&Session,view:Box2)->RenderStats {
        let start=Instant::now(); let mut stats=RenderStats::default();
        let Some(snapshot)=session.snapshot.as_ref() else {
            self.retained.clear();
            self.text(cx,view.x+32.0,view.y+38.0,17.0,p::text(),if session.busy(){"Indexing source…"}else{"Open a codebase"});
            self.text(cx,view.x+32.0,view.y+70.0,11.0,p::secondary(),&shorten(&session.status,((view.w-64.0)/7.0).max(1.0) as usize));
            return stats;
        };
        self.retained.prepare(&snapshot.tree,session.metric,cx.current_dpi_factor());
        let tree=&snapshot.tree;
        if tree.totals().files==0 {
            self.text(cx,view.x+32.0,view.y+38.0,12.0,p::secondary(),"No indexed text files. Check the path or ignore rules.");return stats;
        }
        let (wx,wy)=session.camera.unproject(view.x,view.y);
        let world_view=Box2::new(wx,wy,view.w/session.camera.scale,view.h/session.camera.scale);
        let mut stack=vec![0];let mut visible=Vec::new();let mut work=Vec::new();
        while let Some(id)=stack.pop() {
            let world=snapshot.rectangles[id];
            if world.w<=0.0||world.h<=0.0||!world.intersects(world_view){continue;}
            visible.push((id,session.camera.project(world)));
            stack.extend(tree.nodes[id].children.iter().rev().copied());
        }
        stats.visible_nodes=visible.len();
        self.quad.begin_many_instances(cx);
        for &(id,b) in &visible {
            let node=&tree.nodes[id];
            let base=node.file.as_ref().map(|f|p::language(&f.language)).unwrap_or(p::secondary());
            let dim=if !session.query.is_empty()&&!session.matches.contains(&id)&&node.file.is_some(){0.28}else{1.0};
            let selected=session.selected==Some(id)&&id!=0;
            let hovered=session.hovered==Some(id)&&id!=0;
            self.fill(cx,b,if selected{p::accent()}else if hovered{p::secondary()}else{p::shade(base,0.32*dim)});
            let edge=(b.w.min(b.h)*0.02).min(if selected{1.5}else{0.65});
            self.fill(cx,b.inset(edge),if node.file.is_some(){p::shade(base,0.095*dim)}else{p::background()});
            if session.sources {
                if let (Some(file),Some(page))=(&node.file,snapshot.pages[id]) {
                    tiles::visible_tiles(id,page,file.document.lines.len(),file.document.max_columns,world_view,&mut work);
                }
            }
        }
        self.quad.end_many_instances(cx);
        // Warm cached geometry first. Limit *preparation work*, not displayed
        // source. All visible tiles eventually load, at any zoom level.
        let mut cold=Vec::new();
        for tile in &work {
            let dim=if !session.query.is_empty()&&!session.matches.contains(&tile.key.file){0.28}else{1.0};
            if let Some(cached)=self.retained.tiles.get_mut(&tile.key) {
                cached.replay(cx,session.camera,view,dim,false);
                stats.reused_tiles+=1;stats.glyph_runs+=cached.runs;
            }else{cold.push(tile);}
        }
        // Start close to the cursor/selection so immediate reading stays useful.
        cold.sort_by_key(|t|usize::from(session.selected!=Some(t.key.file)));
        let build_start=Instant::now();
        for tile in cold {
            if stats.built_tiles>0&&(build_start.elapsed().as_millis()>=5||stats.built_tiles>=8) {
                stats.pending_tiles+=1;continue;
            }
            let id=tile.key.file;let file=tree.nodes[id].file.as_ref().unwrap();let page=snapshot.pages[id].unwrap();
            let mut cached=CachedTile::build(cx,&mut self.code,tile,page,&file.document);
            let dim=if !session.query.is_empty()&&!session.matches.contains(&id){0.28}else{1.0};
            cached.replay(cx,session.camera,view,dim,true);
            stats.submitted_runs+=cached.runs;stats.glyph_runs+=cached.runs;stats.built_tiles+=1;
            self.retained.bytes+=cached.bytes;self.retained.tiles.insert(tile.key,cached);
        }
        // File/directory labels are screen-space annotations, never substitutes
        // for the source glyphs retained underneath them.
        for &(id,b) in &visible {
            let node=&tree.nodes[id];
            let header=snapshot.pages[id].map(|page|session.camera.project(page.content).y-b.y).unwrap_or((b.h*0.05).min(20.0));
            if b.w>64.0&&header>=13.0 {
                self.text(cx,b.x+5.0,b.y+3.0,10.0,p::secondary(),&shorten(&node.name,((b.w-10.0)/6.6) as usize));
            }
            if session.sources {
                if let (Some(file),Some(page))=(&node.file,snapshot.pages[id]) {
                    // Trace actual availability, not a made-up FPS counter.
                    if stats.pending_tiles==0 {
                        stats.source_lines+=file.document.lines.len();
                        if std::env::var_os("SCOPE_TRACE").is_some() {
                            eprintln!("scope source: lines={} runs={} font={:.4} representation=source",file.document.lines.len(),stats.glyph_runs,page.font_size*session.camera.scale);
                        }
                    }
                }
            }
        }
        stats.retained_bytes=self.retained.bytes;
        stats.cpu_submit_ms=start.elapsed().as_secs_f64()*1000.0;
        if std::env::var_os("SCOPE_TRACE").is_some() {
            eprintln!("scope frame: nodes={} lines={} runs={} scale={:.6} representation=source cpu_ms={:.4} built={} reused={} pending={} submitted_runs={} geometry_bytes={}",stats.visible_nodes,stats.source_lines,stats.glyph_runs,session.camera.scale,stats.cpu_submit_ms,stats.built_tiles,stats.reused_tiles,stats.pending_tiles,stats.submitted_runs,stats.retained_bytes);
        }
        stats
    }
}
