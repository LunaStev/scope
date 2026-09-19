//! Bounded background preparation. Admission is explicit; a full queue is not
//! reported as a successful read. Allocation accounting runs on the worker.
use crate::{Wake, noop, residency::Residency};
use model::{Document, FileStamp};
use std::{path::PathBuf, sync::{Arc, mpsc}, collections::{HashSet, HashMap}};
const MAX_IN_FLIGHT: usize = 2;
pub const DOCUMENT_BYTES: usize = 128 * 1024 * 1024;
type Message = (usize, Result<(Arc<Document>, usize), String>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request { Ready, Loading, Queued, Full, Failed }

pub struct Documents {
    cache: Residency<usize, Arc<Document>>,
    loading: HashSet<usize>, errors: HashMap<usize, String>,
    tx: mpsc::SyncSender<Message>, rx: mpsc::Receiver<Message>,
    pub wake: Wake, frame: u64,
}
impl Default for Documents {
    fn default() -> Self {
        let (tx, rx) = mpsc::sync_channel(MAX_IN_FLIGHT);
        Self { cache: Residency::new(DOCUMENT_BYTES,256), loading: HashSet::new(),
            errors: HashMap::new(), tx, rx, wake: noop, frame: 1 }
    }
}
impl Documents {
    pub fn begin_frame(&mut self) { self.frame = self.frame.wrapping_add(1).max(1); }
    pub fn bytes(&self) -> usize { self.cache.bytes() }
    pub fn pending(&self) -> usize { self.loading.len() }
    pub fn contains(&self, id: usize) -> bool { self.cache.contains(id) }
    pub fn error(&self, id: usize) -> Option<&str> { self.errors.get(&id).map(String::as_str) }
    pub fn get(&mut self, id: usize) -> Option<Arc<Document>> { self.cache.get(id,self.frame).cloned() }
    pub fn poll(&mut self) -> bool {
        let mut changed = false;
        let mut retired = Vec::new();
        while let Ok((id,result)) = self.rx.try_recv() {
            changed = true; self.loading.remove(&id);
            match result {
                Ok((doc,bytes)) => {
                    if self.cache.make_room_with(bytes,self.frame+1,|old|retired.push(old)) {
                        let inserted = self.cache.insert(id,doc,bytes,self.frame+1);
                        debug_assert!(inserted);
                    } else {
                        retired.push(doc);
                        self.errors.insert(id,"Source preparation exceeds the resident-source budget".into());
                    }
                }
                Err(error) => { self.errors.insert(id,error); }
            }
        }
        // Dropping thousands of token strings must not block input dispatch.
        if !retired.is_empty() { std::thread::spawn(move || drop(retired)); }
        changed
    }
    pub fn request(&mut self, id: usize, path: PathBuf, stamp: FileStamp, limit: u64) -> Request {
        if self.cache.contains(id) { return Request::Ready; }
        if self.errors.contains_key(&id) { return Request::Failed; }
        if self.loading.contains(&id) { return Request::Loading; }
        if self.loading.len() >= MAX_IN_FLIGHT { return Request::Full; }
        self.loading.insert(id);
        let tx = self.tx.clone(); let wake = self.wake;
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(|| {
                analysis::source::load_document(&path,stamp,limit).map(|doc| {
                    let bytes = doc.storage_bytes();
                    (Arc::new(doc),bytes)
                })
            }).unwrap_or_else(|_|Err("Source preparation worker failed".into()));
            let _ = tx.send((id,result)); wake();
        });
        Request::Queued
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn changed_revisions_are_rejected() { let t=tempfile::tempdir().unwrap(); let p=t.path().join("x.wave"); std::fs::write(&p,"fun main() {}\n").unwrap(); let stamp=FileStamp::from_metadata(&p.metadata().unwrap()); std::fs::write(&p,"changed source after scan, another length").unwrap(); assert!(analysis::source::load_document(&p,stamp,4096).is_err()); }
    #[test] fn full_queue_is_not_a_queued_request() {
        let mut docs=Documents::default(); docs.loading.extend([1,2]);
        let stamp=FileStamp{bytes:0,modified:None};
        assert_eq!(docs.request(3,PathBuf::new(),stamp,10),Request::Full);
        assert_eq!(docs.pending(),2);
        assert_eq!(docs.request(1,PathBuf::new(),stamp,10),Request::Loading);
    }
}
