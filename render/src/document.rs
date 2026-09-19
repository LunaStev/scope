//! A build visits ONE bounded tile. Replay visits resident tiles only; it never
//! walks the missing tail of every visible source document.
use crate::{MapPainter,RenderStats,palette as p,cache::{ChunkKey,CachedChunk,source_uniform}};
use makepad_widgets::*;
use ::model::{Document,Ink};
use ::layout::{Box2,Camera,SourceLayout,tiles::SourceTile,source::{GLYPH_ADVANCE,LINE_HEIGHT,CHUNK_COLUMNS}};
use ::runtime::Snapshot;

impl MapPainter {
    pub(crate) fn build_tile(&mut self,cx:&mut Cx2d,key:ChunkKey,tile:&SourceTile,page:&SourceLayout,doc:&Document,camera:Camera,view:Box2,dim:f32)->usize {
        let left=tile.tile*CHUNK_COLUMNS;let right=left+CHUNK_COLUMNS;
        let mut list=DrawList2d::new(cx);list.begin_always(cx);
        self.code.text_style.font_size=16.0;self.code.font_scale=1.0;
        let mut runs=0;let mut glyphs=0;
        for index in tile.first_line..tile.end_line {
            let Some(line)=doc.runs.get(index) else {continue;};
            for run in line {
                if run.column>=right {break;}
                let skip=left.saturating_sub(run.column);let start_col=run.column.max(left);
                let text:String=run.text.chars().skip(skip).take(right-start_col).collect();
                if text.is_empty(){continue;}
                self.code.color=match run.ink {
                    Ink::Text=>p::rgb(174,184,196),Ink::Keyword=>p::rgb(175,165,197),
                    Ink::Number=>p::rgb(191,167,136),Ink::String=>p::rgb(153,184,154),
                    Ink::Comment=>p::rgb(105,133,125),
                };
                self.code.draw_abs(cx,dvec2((start_col-left) as f64*16.0*GLYPH_ADVANCE,(index-tile.first_line) as f64*16.0*LINE_HEIGHT),&text);
                runs+=1;glyphs+=text.chars().count();
            }
        }
        list.end(cx);
        let clip=camera.project(page.content).intersection(view).unwrap_or_default();
        let origin=camera.project(tile.world);
        list.set_view_transform(cx,&source_uniform(page.font_size*camera.scale/16.0,origin.x,origin.y,clip,dim));
        let chunk=CachedChunk{list,lines:tile.end_line-tile.first_line,runs,glyphs,world:tile.world,born_frame:self.cache.frame};
        let inserted=self.cache.chunks.insert(key,chunk,glyphs.max(1),self.cache.frame);
        debug_assert!(inserted);glyphs
    }

    pub(crate) fn replay_source(&mut self,cx:&mut Cx2d,snapshot:&Snapshot,camera:Camera,view:Box2,query:&str,matches:&std::collections::HashSet<usize>,stats:&mut RenderStats) {
        let frame=self.cache.frame;let line_blocks=&mut self.cache.line_blocks;
        self.cache.chunks.visit_mut(frame,|key,chunk| {
            let Some(page)=snapshot.pages[key.node].as_ref() else {return false;};
            let Some(clip)=camera.project(page.content).intersection(view) else {return false;};
            let bounds=camera.project(chunk.world);
            if !bounds.intersects(clip){return false;}
            let font=page.font_size*camera.scale;
            if stats.min_font==0.0||font<stats.min_font{stats.min_font=font;}
            stats.max_font=stats.max_font.max(font);
            if chunk.born_frame!=frame {
                if chunk.runs>0{let _=chunk.list.begin_maybe(cx,false);}
                stats.reused_chunks+=1;
            }
            let dim=if !query.is_empty()&&!matches.contains(&key.node){0.2}else{1.0};
            chunk.list.set_view_transform(cx,&source_uniform(font/16.0,bounds.x,bounds.y,clip,dim));
            if line_blocks.insert((key.node,key.column,key.block)){stats.source_lines+=chunk.lines;}
            stats.glyph_runs+=chunk.runs;true
        });
    }
}
