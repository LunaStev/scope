//! Visible foreground is prepared before speculative halo tiles.
use crate::{MapPainter,RenderStats,budget::FrameBudget,cache::ChunkKey,document::ascii_tile};
use makepad_widgets::*;
use ::layout::{Box2,Camera,tiles::{SourceTile,TileCursor},source::{CHUNK_LINES,CHUNK_COLUMNS}};
use ::runtime::{Session,Snapshot,documents::{Documents,Request}};
use std::{collections::VecDeque,time::Instant};
struct FileWork{node:usize,cursor:TileCursor,prefetch:Option<TileCursor>,next:Option<SourceTile>}
#[derive(Default)]
pub struct SourceQueue {
    pending:VecDeque<FileWork>,ready:VecDeque<FileWork>,waiting:VecDeque<FileWork>,deferred:VecDeque<FileWork>,
    pub limited:bool,pub errors:usize,
}
impl SourceQueue {
    pub fn add(&mut self,node:usize,snapshot:&Snapshot,camera:Camera,view:Box2,halo:Box2) {
        if let Some(page)=&snapshot.pages[node] {
            self.pending.push_back(FileWork{node,cursor:TileCursor::new(page,camera,view),
                prefetch:Some(TileCursor::new(page,camera,halo)),next:None});
        }
    }
    fn poll(&mut self,docs:&Documents) {
        for _ in 0..self.waiting.len() {
            let work=self.waiting.pop_front().unwrap();
            if docs.contains(work.node){self.ready.push_back(work);}
            else if docs.error(work.node).is_some(){self.errors+=1;}
            else{self.waiting.push_back(work);}
        }
    }
    pub fn pending(&self)->usize{self.pending.len()+self.ready.len()+self.deferred.len()}
    pub fn waiting(&self)->usize{self.waiting.len()}
    pub fn runnable(&self,docs:&Documents)->bool {
        !self.limited&&(!self.ready.is_empty()||(!self.pending.is_empty()&&docs.pending()<2)
            ||(self.pending.is_empty()&&self.waiting.is_empty()&&!self.deferred.is_empty()&&docs.pending()<2))
    }
}
impl MapPainter {
    pub(crate) fn prepare_source(&mut self,cx:&mut Cx2d,queue:&mut SourceQueue,session:&mut Session,snapshot:&Snapshot,view:Box2,started:Instant,stats:&mut RenderStats) {
        queue.poll(&session.documents);if queue.limited{return;}
        let mut budget=FrameBudget::new(started);
        while budget.can_check() {
            let work=queue.ready.pop_front().or_else(||queue.pending.pop_front()).or_else(||{
                if queue.waiting.is_empty(){queue.deferred.pop_front()}else{None}
            });
            let Some(mut work)=work else{break;};budget.checked+=1;
            let Some(page)=&snapshot.pages[work.node]else{continue;};
            let Some(file)=&snapshot.tree.nodes[work.node].file else{continue;};
            if work.next.is_none(){work.next=work.cursor.next(page);}
            let Some(tile)=work.next else {
                if let Some(prefetch)=work.prefetch.take(){work.cursor=prefetch;queue.deferred.push_back(work);}
                continue;
            };
            let key=ChunkKey{node:work.node,column:tile.column,block:tile.block,tile:tile.tile};
            if self.cache.chunks.contains(key){work.next=None;queue.ready.push_front(work);continue;}
            if let Some(document)=session.documents.get(work.node) {
                let fast=self.atlas.resident.is_some()&&ascii_tile(&document,&tile);
                let reserve=CHUNK_LINES*CHUNK_COLUMNS;
                if !budget.can_build_kind(reserve,fast){queue.ready.push_front(work);break;}
                if !self.cache.chunks.make_room(reserve,self.cache.frame){queue.ready.push_front(work);queue.limited=true;break;}
                let background=background(snapshot,work.node);let build_start=Instant::now();
                let glyphs=self.build_tile(cx,key,&tile,page,&document,session.camera,view,1.0,background,fast);
                stats.build_cpu_ms+=build_start.elapsed().as_secs_f64()*1000.0;
                if fast{stats.atlas_builds+=1;}else{stats.fallback_builds+=1;}
                budget.built+=1;budget.glyphs+=glyphs;work.next=None;queue.ready.push_back(work);
            }else {
                let path=snapshot.tree.root.join(&snapshot.tree.nodes[work.node].relative);
                match session.documents.request(work.node,path,file.stamp,session.options.max_file_bytes) {
                    Request::Ready=>queue.ready.push_back(work),
                    Request::Queued|Request::Loading=>queue.waiting.push_back(work),
                    Request::Full=>{queue.pending.push_front(work);break;},
                    Request::Failed=>queue.errors+=1,
                }
            }
        }
        stats.tile_checks=budget.checked;stats.built_chunks=budget.built;stats.built_glyphs=budget.glyphs;
    }
}
fn background(snapshot:&Snapshot,mut node:usize)->Vec4 {
    let mut chain=Vec::new();loop{chain.push(node);match snapshot.tree.nodes[node].parent{Some(parent)=>node=parent,None=>break}}
    let mut rgb=[16u8,20u8,24u8];
    for id in chain.into_iter().rev(){let node=&snapshot.tree.nodes[id];let color=node.file.as_ref().map(|f|raster::scene::language_color(&f.language)).unwrap_or([163,175,188]);let alpha=if node.file.is_some(){0.06_f32}else{0.025_f32};for i in 0..3{rgb[i]=(rgb[i] as f32*(1.0-alpha)+color[i] as f32*alpha).round() as u8;}}
    crate::palette::rgb(rgb[0],rgb[1],rgb[2])
}
