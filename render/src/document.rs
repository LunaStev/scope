use crate::{MapPainter,RenderStats,palette as p,cache::{ChunkKey,CachedChunk,source_uniform}};
use makepad_widgets::*;
use ::model::{FileInfo,Ink};
use ::layout::{Box2,Camera,SourceLayout,source::{GLYPH_ADVANCE,LINE_HEIGHT,CHUNK_LINES,CHUNK_COLUMNS}};
use ::runtime::documents::Documents;
use std::{path::Path,time::Instant};
impl MapPainter{
    pub(crate) fn draw_document(&mut self,cx:&mut Cx2d,node:usize,page:&SourceLayout,view:Box2,camera:Camera,file:&FileInfo,path:&Path,limit:u64,documents:&mut Documents,dim:f32,started:Instant,stats:&mut RenderStats){
        let content=camera.project(page.content);let Some(clip)=content.intersection(view)else{return;};if file.shape.lines()==0{return;}
        let font=page.font_size*camera.scale;let row_h=page.line_height*camera.scale;
        if !font.is_finite()||font<=0.0||row_h<=0.0{return;}
        if stats.min_font==0.0||font<stats.min_font{stats.min_font=font;}stats.max_font=stats.max_font.max(font);
        let mut prepared=None;let mut file_builds=0;
        for(column,c)in page.packed.iter().enumerate(){
            let sx=camera.x+c.x*camera.scale;if sx>clip.x+clip.w||sx+c.width*camera.scale<clip.x{continue;}
            let n=c.end_line-c.first_line;if n==0{continue;}
            let first_row=(((clip.y-content.y)/row_h).floor().max(0.0) as usize).saturating_sub(1).min(n);
            let last_row=(((clip.y+clip.h-content.y)/row_h).ceil().max(0.0) as usize).saturating_add(1).min(n);
            if first_row>=last_row{continue;}
            let tile_width=font*GLYPH_ADVANCE*CHUNK_COLUMNS as f64;
            let tiles=((c.width*camera.scale)/tile_width).ceil().max(1.0) as usize;
            let first_tile=(((clip.x-sx)/tile_width).floor().max(0.0) as usize).min(tiles);
            let last_tile=(((clip.x+clip.w-sx)/tile_width).ceil().max(0.0) as usize).min(tiles);
            for block in first_row/CHUNK_LINES..last_row.div_ceil(CHUNK_LINES){
                let first=c.first_line+block*CHUNK_LINES;let end=(first+CHUNK_LINES).min(c.end_line);
                for tile in first_tile..last_tile{
                    let key=ChunkKey{node,column,block,tile};let frame=self.cache.frame;
                    if !self.cache.chunks.contains(key){
                        if (stats.built_chunks>0&&started.elapsed().as_millis()>=5)||file_builds>=2{stats.pending_chunks+=1;continue;}
                        if !self.cache.chunks.make_room(CHUNK_LINES*CHUNK_COLUMNS,frame){stats.limited_chunks+=1;continue;}
                        if prepared.is_none(){prepared=documents.get(node);}
                        let Some(doc)=prepared.as_ref()else{if documents.request(node,path.to_owned(),file.stamp,limit){stats.waiting_sources+=1;}else{stats.source_errors+=1;}continue;};
                        let left=tile*CHUNK_COLUMNS;let right=left+CHUNK_COLUMNS;let mut list=DrawList2d::new(cx);list.begin_always(cx);
                        self.code.text_style.font_size=16.0;self.code.font_scale=1.0;let mut runs=0;let mut glyphs=0;
                        for index in first..end{
                            let Some(line)=doc.runs.get(index)else{continue;};
                            for run in line{
                                if run.column>=right{break;}let skip=left.saturating_sub(run.column);let start_col=run.column.max(left);
                                let text:String=run.text.chars().skip(skip).take(right-start_col).collect();if text.is_empty(){continue;}
                                self.code.color=match run.ink{Ink::Text=>p::rgb(174,184,196),Ink::Keyword=>p::rgb(175,165,197),Ink::Number=>p::rgb(191,167,136),Ink::String=>p::rgb(153,184,154),Ink::Comment=>p::rgb(105,133,125)};
                                self.code.draw_abs(cx,dvec2((start_col-left) as f64*16.0*GLYPH_ADVANCE,(index-first) as f64*16.0*LINE_HEIGHT),&text);runs+=1;glyphs+=text.chars().count();
                            }
                        }
                        list.end(cx);
                        let world=Box2::new(c.x+tile as f64*CHUNK_COLUMNS as f64*page.font_size*GLYPH_ADVANCE,page.content.y+block as f64*CHUNK_LINES as f64*page.line_height,CHUNK_COLUMNS as f64*page.font_size*GLYPH_ADVANCE,(end-first) as f64*page.line_height);
                        let inserted=self.cache.chunks.insert(key,CachedChunk{list,lines:end-first,runs,glyphs,world},glyphs.max(1),frame);
                        debug_assert!(inserted);stats.built_chunks+=1;file_builds+=1;
                    }else{let chunk=self.cache.chunks.get(key,frame).unwrap();if chunk.runs>0{let _=chunk.list.begin_maybe(cx,false);}stats.reused_chunks+=1;}
                    if let Some(chunk)=self.cache.chunks.get(key,frame){let sy=content.y+block as f64*CHUNK_LINES as f64*row_h;chunk.list.set_view_transform(cx,&source_uniform(font/16.0,sx+tile as f64*tile_width,sy,clip,dim));if tile==first_tile{stats.source_lines+=chunk.lines;}stats.glyph_runs+=chunk.runs;}
                }
            }
        }
    }
}
