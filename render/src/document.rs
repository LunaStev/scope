//! Bounded cold tile builds and retained native source replay.
use crate::{MapPainter,RenderStats,palette as p,cache::{ChunkKey,CachedChunk,source_uniform_alpha}};
use makepad_widgets::*;
use ::model::{Document,Ink};
use ::layout::{Box2,Camera,SourceLayout,tiles::SourceTile,source::{GLYPH_ADVANCE,LINE_HEIGHT,CHUNK_COLUMNS}};
use ::runtime::Snapshot;
impl MapPainter{
    pub(crate)fn build_tile(&mut self,cx:&mut Cx2d,key:ChunkKey,tile:&SourceTile,page:&SourceLayout,doc:&Document,camera:Camera,view:Box2,dim:f32,background:Vec4)->usize{
        let left=tile.tile*CHUNK_COLUMNS;let right=left+CHUNK_COLUMNS;
        let mut list=DrawList2d::new(cx);list.begin_always(cx);
        self.source_base.color=background;
        self.source_base.draw_abs(cx,Rect{pos:dvec2(0.0,0.0),size:dvec2(tile.world.w/page.font_size*16.0,tile.world.h/page.font_size*16.0)});
        self.code.text_style.font_size=16.0;self.code.font_scale=1.0;let mut runs=0;let mut glyphs=0;
        for index in tile.first_line..tile.end_line{
            let Some(line)=doc.runs.get(index)else{continue;};
            for run in line{
                if run.column>=right{break;}
                let skip=left.saturating_sub(run.column);let start_col=run.column.max(left);let text=window(&run.text,skip,right-start_col);
                if text.is_empty(){continue;}
                self.code.color=match run.ink{Ink::Text=>p::rgb(174,184,196),Ink::Keyword=>p::rgb(175,165,197),Ink::Number=>p::rgb(191,167,136),Ink::String=>p::rgb(153,184,154),Ink::Comment=>p::rgb(105,133,125)};
                self.code.draw_abs(cx,dvec2((start_col-left) as f64*16.0*GLYPH_ADVANCE,(index-tile.first_line) as f64*16.0*LINE_HEIGHT),text);
                runs+=1;glyphs+=text.chars().count();
            }
        }
        list.end(cx);
        let clip=camera.project(page.content).intersection(view).and_then(|clip|clip.intersection(camera.project(tile.world))).unwrap_or_default();let origin=camera.project(tile.world);
        list.set_view_transform(cx,&source_uniform_alpha(page.font_size*camera.scale/16.0,origin.x,origin.y,clip,dim,0.0));
        let chunk=CachedChunk{list,lines:tile.end_line-tile.first_line,runs,glyphs,world:tile.world,born_frame:self.cache.frame,born:std::time::Instant::now()};
        let inserted=self.cache.chunks.insert(key,chunk,glyphs.max(1),self.cache.frame);debug_assert!(inserted);glyphs
    }
    pub(crate)fn replay_source(&mut self,cx:&mut Cx2d,snapshot:&Snapshot,camera:Camera,view:Box2,query:&str,matches:&std::collections::HashSet<usize>,stats:&mut RenderStats){
        let dpi=cx.current_dpi_factor();let frame=self.cache.frame;let line_blocks=&mut self.cache.line_blocks;
        self.cache.chunks.visit_mut(frame,|key,chunk|{
            let Some(page)=snapshot.pages[key.node].as_ref()else{return false;};
            let Some(clip)=camera.project(page.content).intersection(view)else{return false;};
            let bounds=camera.project(chunk.world);let Some(clip)=clip.intersection(bounds)else{return false;};
            let font=page.font_size*camera.scale;let opacity=crate::live::ink_opacity(font,dpi);if opacity<=0.0{return false;}
            let age=(chunk.born.elapsed().as_secs_f32()/0.09).min(1.0);stats.live_fading|=age<1.0;
            if stats.min_font==0.0||font<stats.min_font{stats.min_font=font;}stats.max_font=stats.max_font.max(font);
            if chunk.born_frame!=frame{let _=chunk.list.begin_maybe(cx,false);stats.live_reused+=1;}
            let _=(query,matches);let dim=1.0;
            chunk.list.set_view_transform(cx,&source_uniform_alpha(font/16.0,bounds.x,bounds.y,clip,dim,opacity*age));
            if line_blocks.insert((key.node,key.column,key.block)){stats.source_lines+=chunk.lines;}
            stats.glyph_runs+=chunk.runs;stats.live_drawn+=chunk.glyphs;true
        });
    }
}
fn window(text:&str,skip:usize,length:usize)->&str{
    if text.is_ascii(){let start=skip.min(text.len());return &text[start..start.saturating_add(length).min(text.len())];}
    let start=text.char_indices().nth(skip).map_or(text.len(),|(i,_)|i);
    let end=text[start..].char_indices().nth(length).map_or(text.len(),|(i,_)|start+i);&text[start..end]
}
#[cfg(test)]mod tests{
    use super::window;
    #[test]fn source_windows_preserve_utf8(){assert_eq!(window("abc",1,2),"bc");assert_eq!(window("ab한국어z",2,2),"한국");assert_eq!(window("😀x",99,10),"");assert_eq!(window("abcd",1,0),"");}
}
