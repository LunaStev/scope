//! Bounded background source preparation at any scale, without a zoom gate.
use crate::{Wake,noop,residency::Residency};
use model::{Document,FileStamp};
use std::{path::PathBuf,sync::{Arc,mpsc},collections::{HashSet,HashMap}};
const MAX_IN_FLIGHT:usize=2;
pub const DOCUMENT_BYTES:usize=128*1024*1024;
type Message=(usize,Result<Document,String>);
pub struct Documents{cache:Residency<usize,Arc<Document>>,loading:HashSet<usize>,errors:HashMap<usize,String>,tx:mpsc::SyncSender<Message>,rx:mpsc::Receiver<Message>,pub wake:Wake,frame:u64}
impl Default for Documents{fn default()->Self{let(tx,rx)=mpsc::sync_channel(MAX_IN_FLIGHT);Self{cache:Residency::new(DOCUMENT_BYTES,256),loading:HashSet::new(),errors:HashMap::new(),tx,rx,wake:noop,frame:1}}}
impl Documents{
    pub fn begin_frame(&mut self){self.frame=self.frame.wrapping_add(1).max(1);}
    pub fn bytes(&self)->usize{self.cache.bytes()}
    pub fn pending(&self)->usize{self.loading.len()}
    pub fn error(&self,id:usize)->Option<&str>{self.errors.get(&id).map(String::as_str)}
    pub fn get(&mut self,id:usize)->Option<Arc<Document>>{self.cache.get(id,self.frame).cloned()}
    pub fn poll(&mut self)->bool{
        let mut changed=false;
        while let Ok((id,result))=self.rx.try_recv(){changed=true;self.loading.remove(&id);match result{
            Ok(doc)=>{let bytes=doc.storage_bytes();if !self.cache.insert(id,Arc::new(doc),bytes,self.frame+1){self.errors.insert(id,"Source preparation exceeds the resident-source budget".into());}}
            Err(e)=>{self.errors.insert(id,e);}
        }}changed
    }
    pub fn request(&mut self,id:usize,path:PathBuf,stamp:FileStamp,limit:u64)->bool{
        if self.cache.contains(id)||self.errors.contains_key(&id){return false;}
        if self.loading.contains(&id){return true;}if self.loading.len()>=MAX_IN_FLIGHT{return true;}
        self.loading.insert(id);let tx=self.tx.clone();let wake=self.wake;
        std::thread::spawn(move||{let result=std::panic::catch_unwind(||analysis::source::load_document(&path,stamp,limit)).unwrap_or_else(|_|Err("Source preparation worker failed".into()));let _=tx.send((id,result));wake();});true
    }
}
#[cfg(test)]mod tests{use super::*;#[test]fn changed_revisions_are_rejected(){let t=tempfile::tempdir().unwrap();let p=t.path().join("x.wave");std::fs::write(&p,"fun main() {}\n").unwrap();let stamp=FileStamp::from_metadata(&p.metadata().unwrap());std::fs::write(&p,"changed source after scan, another length").unwrap();assert!(analysis::source::load_document(&p,stamp,4096).is_err());}}
