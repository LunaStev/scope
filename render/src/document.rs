use crate::{MapPainter,RenderStats,palette as p,cache::{ChunkKey,CachedChunk,source_uniform}};
use makepad_widgets::*;
use ::model::{Document,Ink};
use ::layout::{Box2,Camera,SourceLayout,source::{GLYPH_ADVANCE,LINE_HEIGHT,CHUNK_LINES}};
use std::time::Instant;

impl MapPainter {
    pub(crate) fn draw_document(&mut self,cx:&mut Cx2d,node:usize,page:&SourceLayout,view:Box2,camera:Camera,doc:&Document,dim:f32,started:Instant,stats:&mut RenderStats) {
        let content=camera.project(page.content);
        if !content.intersects(view)||doc.lines.is_empty(){return;}
        let left=view.x.max(content.x);let top=view.y.max(content.y);
        let clip=Box2::new(left,top,(view.x+view.w).min(content.x+content.w)-left,(view.y+view.h).min(content.y+content.h)-top);
        let font=page.font_size*camera.scale;
        if stats.min_font==0.0||font<stats.min_font{stats.min_font=font;}stats.max_font=stats.max_font.max(font);
        let row_h=page.line_height*camera.scale;
        for (column,c) in page.packed.iter().enumerate() {
            let sx=camera.x+c.x*camera.scale;
            if sx>clip.x+clip.w||sx+c.width*camera.scale<clip.x {continue;}
            let n=c.end_line-c.first_line;
            if n==0{continue;}
            let first_row=(((clip.y-content.y)/row_h).floor().max(0.0) as usize).saturating_sub(1).min(n);
            let last_row=(((clip.y+clip.h-content.y)/row_h).ceil().max(0.0) as usize).saturating_add(1).min(n);
            if first_row>=last_row{continue;}
            for block in first_row/CHUNK_LINES..last_row.div_ceil(CHUNK_LINES) {
                let key=ChunkKey {node,column,block};
                if !self.cache.chunks.contains_key(&key) {
                    // Yield cold geometry preparation between chunks. This
                    // never substitutes previews or depends on the zoom level.
                    if stats.built_chunks>0&&started.elapsed().as_millis()>=8 {
                        stats.pending_chunks+=1;continue;
                    }
                    let first=c.first_line+block*CHUNK_LINES;
                    let end=(first+CHUNK_LINES).min(c.end_line);
                    let mut list=DrawList2d::new(cx);list.begin_always(cx);
                    self.code.text_style.font_size=16.0;self.code.font_scale=1.0;
                    let mut runs=0;
                    for index in first..end {
                        for run in &doc.runs[index] {
                            self.code.color=match run.ink {
                                Ink::Text=>p::rgb(174,184,196),Ink::Keyword=>p::rgb(175,165,197),
                                Ink::Number=>p::rgb(191,167,136),Ink::String=>p::rgb(153,184,154),Ink::Comment=>p::rgb(105,133,125),
                            };
                            self.code.draw_abs(cx,dvec2(run.column as f64*16.0*GLYPH_ADVANCE,(index-first) as f64*16.0*LINE_HEIGHT),&run.text);
                            runs+=1;
                        }
                    }
                    list.end(cx);
                    self.cache.chunks.insert(key,CachedChunk {list,lines:end-first,runs});
                    stats.built_chunks+=1;
                } else {
                    let chunk=self.cache.chunks.get_mut(&key).unwrap();
                    // Reuse GPU instances; no reshaping, UTF-8 slicing or upload.
                    if chunk.runs>0 { let _=chunk.list.begin_maybe(cx,false); }
                    stats.reused_chunks+=1;
                }
                let chunk=self.cache.chunks.get(&key).unwrap();
                let sy=content.y+block as f64*CHUNK_LINES as f64*row_h;
                chunk.list.set_view_transform(cx,&source_uniform(font/16.0,sx,sy,clip,dim));
                stats.source_lines+=chunk.lines;stats.glyph_runs+=chunk.runs;
            }
        }
    }
}
