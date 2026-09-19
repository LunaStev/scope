//! Cold work has resumable cursors and separate ready/read queues. A worker
//! completion goes straight to the ready queue, not behind thousands of files.
use crate::{MapPainter,RenderStats,budget::FrameBudget,cache::ChunkKey};
use makepad_widgets::*;
use ::layout::{Box2,Camera,tiles::{SourceTile,TileCursor},source::{CHUNK_LINES,CHUNK_COLUMNS}};
use ::runtime::{Session,Snapshot,documents::{Documents,Request}};
use std::{collections::VecDeque,time::Instant};

struct FileWork { node:usize,cursor:TileCursor,next:Option<SourceTile> }
#[derive(Default)]
pub struct SourceQueue {
    pending:VecDeque<FileWork>,ready:VecDeque<FileWork>,waiting:VecDeque<FileWork>,
    pub limited:bool,pub errors:usize,
}
impl SourceQueue {
    pub fn add(&mut self,node:usize,snapshot:&Snapshot,camera:Camera,view:Box2) {
        if let Some(page)=&snapshot.pages[node] {
            self.pending.push_back(FileWork{node,cursor:TileCursor::new(page,camera,view),next:None});
        }
    }
    fn poll(&mut self,docs:&Documents) {
        for _ in 0..self.waiting.len() {
            let work=self.waiting.pop_front().unwrap();
            if docs.contains(work.node) { self.ready.push_back(work); }
            else if docs.error(work.node).is_some() { self.errors+=1; }
            else { self.waiting.push_back(work); }
        }
    }
    pub fn pending(&self)->usize { self.pending.len()+self.ready.len() }
    pub fn waiting(&self)->usize { self.waiting.len() }
    pub fn runnable(&self,docs:&Documents)->bool {
        !self.limited&&(!self.ready.is_empty()||(!self.pending.is_empty()&&docs.pending()<2))
    }
}
impl MapPainter {
    pub(crate) fn prepare_source(&mut self,cx:&mut Cx2d,queue:&mut SourceQueue,session:&mut Session,snapshot:&Snapshot,view:Box2,started:Instant,stats:&mut RenderStats) {
        queue.poll(&session.documents);
        if queue.limited { return; }
        let mut budget=FrameBudget::new(started);
        while budget.can_check() {
            let was_ready=!queue.ready.is_empty();
            let work=if was_ready {queue.ready.pop_front()} else {queue.pending.pop_front()};
            let Some(mut work)=work else {break;};
            budget.checked+=1;
            let Some(page)=&snapshot.pages[work.node] else {continue;};
            let Some(file)=&snapshot.tree.nodes[work.node].file else {continue;};
            if work.next.is_none() { work.next=work.cursor.next(page); }
            let Some(tile)=work.next else {continue;};
            let key=ChunkKey{node:work.node,column:tile.column,block:tile.block,tile:tile.tile};
            if self.cache.chunks.contains(key) {
                work.next=None;
                if was_ready {queue.ready.push_front(work);} else {queue.pending.push_front(work);}
                continue;
            }
            let document=session.documents.get(work.node);
            if let Some(document)=document {
                let reserve=CHUNK_LINES*CHUNK_COLUMNS;
                if !budget.can_build(reserve) {queue.ready.push_front(work);break;}
                // Reserve only after source is ready. Previously every missing
                // tile could evict useful geometry while its file was loading.
                if !self.cache.chunks.make_room(reserve,self.cache.frame) {
                    queue.ready.push_front(work);queue.limited=true;break;
                }
                let dim=if !session.query.is_empty()&&!session.matches.contains(&work.node){0.2}else{1.0};
                let glyphs=self.build_tile(cx,key,&tile,page,&document,session.camera,view,dim);
                budget.built+=1;budget.glyphs+=glyphs;work.next=None;
                queue.ready.push_back(work);
            } else {
                let path=snapshot.tree.root.join(&snapshot.tree.nodes[work.node].relative);
                match session.documents.request(work.node,path,file.stamp,session.options.max_file_bytes) {
                    Request::Ready=>queue.ready.push_back(work),
                    Request::Queued|Request::Loading=>queue.waiting.push_back(work),
                    Request::Full=>{queue.pending.push_front(work);break;}
                    Request::Failed=>queue.errors+=1,
                }
            }
        }
        stats.tile_checks=budget.checked;stats.built_chunks=budget.built;stats.built_glyphs=budget.glyphs;
    }
}
