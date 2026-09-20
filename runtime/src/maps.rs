//! Latest-view image work. Identical requests are coalesced and an obsolete
//! running refinement yields cooperatively; the complete overview is never
//! cancelled just because the camera moves.
use crate::{Snapshot, Wake};
use raster::{cache::Cache, Image, Rasterizer, Report, Scene, TileKey};
use std::{collections::{HashSet, VecDeque}, sync::{atomic::{AtomicBool, AtomicU64, Ordering}, mpsc, Arc, Condvar, Mutex}, time::{Duration, Instant}};

pub struct ImageUpdate { pub key: TileKey, pub image: Image }
struct ResultMessage { key: TileKey, result: Result<(Image, Report), String>, cancelled: bool }
#[derive(Default)]
struct Work { queue: VecDeque<TileKey>, active: Option<(TileKey, Arc<AtomicBool>)> }
impl Work {
    fn cancel_obsolete(&self, wanted: &HashSet<TileKey>) {
        if let Some((key, cancel)) = &self.active {
            if !wanted.contains(key) { cancel.store(true, Ordering::Relaxed); }
        }
    }
    fn begin(&mut self) -> Option<(TileKey, Arc<AtomicBool>)> {
        let task = (self.queue.pop_front()?, Arc::new(AtomicBool::new(false)));
        self.active = Some(task.clone()); Some(task)
    }
}
struct Control { work: Mutex<Work>, changed: Condvar, cancel: AtomicBool, completed: AtomicU64, total: AtomicU64 }
impl Default for Control {
    fn default() -> Self { Self { work: Mutex::new(Work::default()), changed: Condvar::new(), cancel: AtomicBool::new(false), completed: AtomicU64::new(0), total: AtomicU64::new(0) } }
}
#[derive(Default)]
struct RequestMemo { keys: Vec<TileKey>, resident: HashSet<TileKey>, revision: u64, valid: bool }
impl RequestMemo {
    fn changed(&mut self, keys: &[TileKey], resident: &HashSet<TileKey>, revision: u64) -> bool {
        if self.valid && self.keys == keys && self.resident == *resident && self.revision == revision { return false; }
        self.keys.clear(); self.keys.extend_from_slice(keys); self.resident.clone_from(resident);
        self.revision = revision; self.valid = true; true
    }
}
#[derive(Default)]
pub struct Maps {
    pub generation: u64, pub ready: bool, pub cache_hit: bool, pub errors: u64, pub error: Option<String>,
    pub prepared_lines: u64, pub prepare_ms: u64, pub replans: u64, pub cancelled_details: u64,
    control: Option<Arc<Control>>, rx: Option<mpsc::Receiver<ResultMessage>>,
    images: VecDeque<ImageUpdate>, in_flight: HashSet<TileKey>, failed: HashSet<TileKey>, started: Option<Instant>,
    wanted: HashSet<TileKey>, memo: RequestMemo, result_revision: u64,
}
impl Drop for Maps { fn drop(&mut self) { self.cancel(); } }
impl Maps {
    fn cancel(&self) {
        if let Some(c) = &self.control {
            c.cancel.store(true, Ordering::Relaxed);
            if let Ok(work) = c.work.lock() { if let Some((_, token)) = &work.active { token.store(true, Ordering::Relaxed); } }
            c.changed.notify_all();
        }
    }
    pub fn reset(&mut self) { self.cancel(); *self = Self::default(); }
    pub fn progress(&self) -> (u64,u64) { self.control.as_ref().map(|c|(c.completed.load(Ordering::Relaxed),c.total.load(Ordering::Relaxed))).unwrap_or_default() }
    pub fn pending(&self) -> usize { self.in_flight.len() }
    pub fn has_images(&self) -> bool { !self.images.is_empty() }
    pub fn take_image(&mut self) -> Option<ImageUpdate> {
        let image = self.images.pop_front(); if image.is_some() { self.result_revision = self.result_revision.wrapping_add(1); } image
    }
    pub fn start(&mut self, snapshot: Arc<Snapshot>, font: Arc<Vec<u8>>, limit: u64, wake: Wake) {
        self.reset(); self.generation = snapshot.generation; self.started = Some(Instant::now());
        let control = Arc::new(Control::default()); control.total.store(snapshot.tree.totals().files, Ordering::Relaxed);
        self.control = Some(control.clone()); let (tx,rx) = mpsc::sync_channel(2); self.rx = Some(rx); self.in_flight.insert(TileKey::ROOT);
        std::thread::spawn(move || {
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // Arc slices avoid copying every page and packed column on map start.
                let scene = Scene::with_index(snapshot.tree.clone(), snapshot.rectangles.clone(), snapshot.pages.clone(), &font, limit, snapshot.node_index.clone());
                let cache = Cache::default(); let mut renderer = Rasterizer::new(&font)?;
                let mut last_progress = Instant::now();
                let mut build = |key: TileKey, token: &AtomicBool| -> Result<(Image,Report),String> {
                    if token.load(Ordering::Relaxed) { return Err("Map preparation cancelled".into()); }
                    if let Some(image) = cache.load(&scene.fingerprint,key) {
                        let mut report = Report { cache_hit:true, ..Report::default() };
                        if key == TileKey::ROOT { report.files=scene.tree.totals().files; report.lines=scene.tree.totals().lines; }
                        return Ok((image,report));
                    }
                    let result = renderer.image(&scene,key,token,|done,total| {
                        if key == TileKey::ROOT {
                            control.completed.store(done,Ordering::Relaxed); control.total.store(total,Ordering::Relaxed);
                            if last_progress.elapsed() >= Duration::from_millis(150) { last_progress=Instant::now(); wake(); }
                        }
                    })?;
                    if result.1.errors == 0 && !token.load(Ordering::Relaxed) && !control.cancel.load(Ordering::Relaxed) { let _=cache.store(&scene.fingerprint,key,&result.0); }
                    Ok(result)
                };
                let root=build(TileKey::ROOT,&control.cancel); let success=root.is_ok();
                if !send(&tx,ResultMessage { key:TileKey::ROOT, result:root, cancelled:false },&control,wake) || !success { return Ok(()); }
                loop {
                    let (key,token) = {
                        let mut work=control.work.lock().unwrap();
                        while work.queue.is_empty() && !control.cancel.load(Ordering::Relaxed) { work=control.changed.wait(work).unwrap(); }
                        if control.cancel.load(Ordering::Relaxed) { break; }
                        work.begin().unwrap()
                    };
                    let result=build(key,&token);
                    // The key remains in the UI's in-flight set until poll.
                    control.work.lock().unwrap().active=None;
                    let cancelled=token.load(Ordering::Relaxed);
                    if !send(&tx,ResultMessage { key,result,cancelled },&control,wake) { break; }
                }
                Ok::<(),String>(())
            }));
            let error=match outcome { Ok(Err(e))=>Some(e), Err(_)=>Some("Map worker failed".into()), _=>None };
            if let Some(error)=error { let _=send(&tx,ResultMessage { key:TileKey::ROOT,result:Err(error),cancelled:false },&control,wake); }
        });
    }
    pub fn poll(&mut self) -> bool {
        let mut changed=false;
        while self.images.len()<2 {
            let result=match self.rx.as_ref().map(|r|r.try_recv()) { Some(Ok(m))=>m, _=>break };
            changed=true; self.result_revision=self.result_revision.wrapping_add(1); self.in_flight.remove(&result.key);
            if result.cancelled { self.cancelled_details+=1; continue; }
            if result.key!=TileKey::ROOT && !self.wanted.contains(&result.key) { continue; }
            match result.result {
                Ok((image,report)) => {
                    self.errors+=report.errors; if self.error.is_none() { self.error=report.error_sample; }
                    if result.key==TileKey::ROOT { self.ready=true; self.cache_hit=report.cache_hit; self.prepared_lines=report.lines; self.prepare_ms=self.started.map_or(0,|s|s.elapsed().as_millis() as u64); }
                    self.images.push_back(ImageUpdate { key:result.key,image });
                }
                Err(error) => { self.errors+=1; self.failed.insert(result.key); if self.error.is_none() { self.error=Some(error); } }
            }
        }
        changed
    }
    pub fn request(&mut self, keys: &[TileKey], resident: &HashSet<TileKey>) {
        if !self.ready || !self.memo.changed(keys,resident,self.result_revision) { return; }
        self.replans+=1; self.wanted.clear(); self.wanted.extend(keys.iter().copied());
        // Do not spend a GPU upload slot on a stale decoded refinement.
        self.images.retain(|i|i.key==TileKey::ROOT || self.wanted.contains(&i.key));
        let Some(control)=&self.control else{return;}; let mut work=control.work.lock().unwrap();
        work.cancel_obsolete(&self.wanted);
        for key in work.queue.drain(..) { self.in_flight.remove(&key); }
        for &key in keys {
            if key==TileKey::ROOT || resident.contains(&key) || self.failed.contains(&key) || self.images.iter().any(|i|i.key==key) { continue; }
            if self.in_flight.insert(key) { work.queue.push_back(key); }
        }
        if !work.queue.is_empty() { control.changed.notify_one(); }
    }
}
fn send(tx:&mpsc::SyncSender<ResultMessage>,mut message:ResultMessage,control:&Control,wake:Wake)->bool {
    loop { if control.cancel.load(Ordering::Relaxed) { return false; }
        match tx.try_send(message) {
            Ok(())=>{wake();return true;}, Err(mpsc::TrySendError::Disconnected(_))=>return false,
            Err(mpsc::TrySendError::Full(m))=>{message=m;std::thread::sleep(Duration::from_millis(3));}
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn key(x:u32)->TileKey {TileKey{level:3,x,y:0}}
    #[test]fn identical_requests_do_not_rebuild_or_wake_the_queue() {
        let mut memo=RequestMemo::default();let resident=HashSet::new();
        assert!(memo.changed(&[key(1)],&resident,0));
        for _ in 0..1000 { assert!(!memo.changed(&[key(1)],&resident,0)); }
        assert!(memo.changed(&[key(1)],&resident,1));
        assert!(memo.changed(&[key(1)],&HashSet::from([key(1)]),1));
    }
    #[test]fn camera_change_cancels_only_obsolete_active_detail() {
        let mut work=Work::default();work.queue.push_back(key(1));let(_,token)=work.begin().unwrap();
        work.cancel_obsolete(&HashSet::from([key(1),key(2)]));assert!(!token.load(Ordering::Relaxed));
        work.cancel_obsolete(&HashSet::from([key(2)]));assert!(token.load(Ordering::Relaxed));
    }
    #[test]fn cancelled_detail_is_retryable_and_not_a_source_error() {
        let mut maps=Maps::default();let(tx,rx)=mpsc::sync_channel(2);maps.rx=Some(rx);maps.in_flight.insert(key(1));
        tx.send(ResultMessage{key:key(1),result:Err("cancelled".into()),cancelled:true}).unwrap();
        assert!(maps.poll());assert_eq!(maps.errors,0);assert!(maps.failed.is_empty());assert_eq!(maps.pending(),0);assert_eq!(maps.cancelled_details,1);
    }
    #[test]fn stale_decoded_images_are_discarded_but_root_is_pinned() {
        let mut maps=Maps::default();maps.ready=true;maps.control=Some(Arc::new(Control::default()));
        for k in [TileKey::ROOT,key(1)] { maps.images.push_back(ImageUpdate{key:k,image:Image::new(1,1,[1,2,3])}); }
        maps.request(&[key(2)],&HashSet::new());assert_eq!(maps.images.len(),1);assert_eq!(maps.images[0].key,TileKey::ROOT);
    }
}
