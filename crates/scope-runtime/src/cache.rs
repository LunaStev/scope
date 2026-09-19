use crate::{Wake,noop};
use scope_analysis::source::{self,Document};
use std::collections::{HashMap,HashSet,VecDeque};
use std::path::PathBuf;
use std::sync::{Arc,mpsc};

type ResultMessage=(usize,Result<Document,String>);
pub const CACHE_LIMIT:usize=32*1024*1024;
pub const CACHE_FILES:usize=8;
pub struct SourceCache {
    documents:HashMap<usize,Arc<Document>>,
    order:VecDeque<usize>,
    errors:HashMap<usize,String>,
    loading:HashSet<usize>,
    bytes:usize,
    tx:mpsc::Sender<ResultMessage>,
    rx:mpsc::Receiver<ResultMessage>,
    pub wake:Wake,
}
impl Default for SourceCache {
    fn default()->Self {
        let(tx,rx)=mpsc::channel();
        Self{documents:HashMap::new(),order:VecDeque::new(),errors:HashMap::new(),loading:HashSet::new(),bytes:0,tx,rx,wake:noop}
    }
}
impl SourceCache {
    pub fn bytes(&self)->usize{self.bytes}
    pub fn len(&self)->usize{self.documents.len()}
    pub fn is_empty(&self)->bool{self.documents.is_empty()}
    pub fn contains(&self,id:usize)->bool{self.documents.contains_key(&id)}
    pub fn loading(&self,id:usize)->bool{self.loading.contains(&id)}
    pub fn error(&self,id:usize)->Option<&str>{self.errors.get(&id).map(String::as_str)}
    pub fn get(&mut self,id:usize)->Option<Arc<Document>>{
        let doc=self.documents.get(&id)?.clone();
        self.order.retain(|&old|old!=id);self.order.push_back(id);Some(doc)
    }
    pub fn insert(&mut self,id:usize,doc:Document)->Result<(),String>{
        let size=doc.storage_bytes();
        if size>CACHE_LIMIT{return Err("Source index exceeds the 32 MiB preview cache budget".into());}
        if let Some(old)=self.documents.remove(&id){self.bytes-=old.storage_bytes();}
        self.order.retain(|&old|old!=id);
        while self.documents.len()>=CACHE_FILES || self.bytes+size>CACHE_LIMIT{
            let Some(old)=self.order.pop_front()else{break;};
            if let Some(old)=self.documents.remove(&old){self.bytes-=old.storage_bytes();}
        }
        self.bytes+=size;self.documents.insert(id,Arc::new(doc));self.order.push_back(id);Ok(())
    }
    pub fn request(&mut self,id:usize,path:PathBuf,limit:u64){
        if self.loading.len()>=3 || self.loading.contains(&id) || self.documents.contains_key(&id) || self.errors.contains_key(&id){return;}
        self.loading.insert(id);
        let tx=self.tx.clone();let wake=self.wake;
        std::thread::spawn(move||{
            let result=source::read_text(&path,limit.min(16*1024*1024)).map_err(|e|e.to_string())
                .and_then(|t|t.ok_or_else(||"File is no longer readable UTF-8 text".into())).map(Document::new);
            let _=tx.send((id,result));wake();
        });
    }
    pub fn poll(&mut self)->bool{
        let mut changed=false;
        while let Ok((id,result))=self.rx.try_recv(){
            changed=true;self.loading.remove(&id);
            if let Err(error)=result.and_then(|doc|self.insert(id,doc)){self.errors.insert(id,error);}
        }
        changed
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test] fn replacement_does_not_double_count(){
        let mut c=SourceCache::default();c.insert(1,Document::new("x".into())).unwrap();
        let bytes=c.bytes();c.insert(1,Document::new("x".into())).unwrap();assert_eq!(bytes,c.bytes());assert_eq!(c.len(),1);
    }
    #[test] fn recently_read_document_survives_eviction(){
        let mut c=SourceCache::default();for id in 0..8{c.insert(id,Document::new("x".into())).unwrap();}
        c.get(0);c.insert(8,Document::new("x".into())).unwrap();assert!(c.contains(0));assert!(!c.contains(1));assert_eq!(c.len(),8);
    }
}
