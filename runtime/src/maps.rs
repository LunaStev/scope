//! Map preparation is independent of camera drawing. One persistent worker
//! prioritises the newest view. Results and GPU upload admission are bounded.
use crate::{Snapshot,Wake};
use raster::{Image,Report,Scene,Rasterizer,TileKey,cache::Cache};
use std::{collections::{HashSet,VecDeque},sync::{Arc,Mutex,Condvar,mpsc,atomic::{AtomicBool,AtomicU64,Ordering}},time::{Duration,Instant}};

pub struct ImageUpdate { pub key:TileKey, pub image:Image }
struct ResultMessage {key:TileKey,result:Result<(Image,Report),String>}
#[derive(Default)]struct Work {queue:VecDeque<TileKey>}
struct Control {work:Mutex<Work>,changed:Condvar,cancel:AtomicBool,completed:AtomicU64,total:AtomicU64}
impl Default for Control {fn default()->Self{Self{work:Mutex::new(Work::default()),changed:Condvar::new(),cancel:AtomicBool::new(false),completed:AtomicU64::new(0),total:AtomicU64::new(0)}}}
#[derive(Default)]
pub struct Maps {
    pub generation:u64,pub ready:bool,pub cache_hit:bool,pub errors:u64,pub error:Option<String>,
    pub prepared_lines:u64,pub prepare_ms:u64,
    control:Option<Arc<Control>>,rx:Option<mpsc::Receiver<ResultMessage>>,
    images:VecDeque<ImageUpdate>,in_flight:HashSet<TileKey>,failed:HashSet<TileKey>,started:Option<Instant>,
}
impl Drop for Maps {fn drop(&mut self){self.cancel();}}
impl Maps {
    fn cancel(&self){if let Some(c)=&self.control{c.cancel.store(true,Ordering::Relaxed);c.changed.notify_all();}}
    pub fn reset(&mut self){self.cancel();*self=Self::default();}
    pub fn progress(&self)->(u64,u64){self.control.as_ref().map(|c|(c.completed.load(Ordering::Relaxed),c.total.load(Ordering::Relaxed))).unwrap_or_default()}
    pub fn pending(&self)->usize{self.in_flight.len()}
    pub fn has_images(&self)->bool{!self.images.is_empty()}
    pub fn take_image(&mut self)->Option<ImageUpdate>{self.images.pop_front()}
    pub fn start(&mut self,snapshot:Arc<Snapshot>,font:Arc<Vec<u8>>,limit:u64,wake:Wake){
        self.reset();self.generation=snapshot.generation;self.started=Some(Instant::now());
        let control=Arc::new(Control::default());control.total.store(snapshot.tree.totals().files,Ordering::Relaxed);
        self.control=Some(control.clone());let(tx,rx)=mpsc::sync_channel(2);self.rx=Some(rx);self.in_flight.insert(TileKey::ROOT);
        std::thread::spawn(move||{
            let outcome=std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let scene=Scene::new(snapshot.tree.clone(),snapshot.rectangles.clone().into(),snapshot.pages.clone().into(),&font,limit);
                let cache=Cache::default();let mut renderer=Rasterizer::new(&font)?;
                let mut last_progress=Instant::now();
                let mut build=|key:TileKey|->Result<(Image,Report),String>{
                    if let Some(image)=cache.load(&scene.fingerprint,key){
                        let mut report=Report{cache_hit:true,..Report::default()};
                        if key==TileKey::ROOT{report.files=scene.tree.totals().files;report.lines=scene.tree.totals().lines;}
                        return Ok((image,report));
                    }
                    let result=renderer.image(&scene,key,&control.cancel,|done,total|{
                        if key==TileKey::ROOT{control.completed.store(done,Ordering::Relaxed);control.total.store(total,Ordering::Relaxed);if last_progress.elapsed()>=Duration::from_millis(150){last_progress=Instant::now();wake();}}
                    })?;
                    if result.1.errors==0&&!control.cancel.load(Ordering::Relaxed){let _=cache.store(&scene.fingerprint,key,&result.0);}
                    Ok(result)
                };
                let root=build(TileKey::ROOT);let success=root.is_ok();
                if !send(&tx,ResultMessage{key:TileKey::ROOT,result:root},&control,wake)||!success{return Ok(());}
                loop {
                    if control.cancel.load(Ordering::Relaxed){break;}
                    let key={let mut work=control.work.lock().unwrap();
                        while work.queue.is_empty()&&!control.cancel.load(Ordering::Relaxed){work=control.changed.wait(work).unwrap();}
                        if control.cancel.load(Ordering::Relaxed){break;}work.queue.pop_front().unwrap()};
                    let result=build(key);
                    if !send(&tx,ResultMessage{key,result},&control,wake){break;}
                }
                Ok::<(),String>(())
            }));
            let error=match outcome{Ok(Err(e))=>Some(e),Err(_)=>Some("Map worker failed".into()),_=>None};
            if let Some(error)=error{let _=send(&tx,ResultMessage{key:TileKey::ROOT,result:Err(error)},&control,wake);}
        });
    }
    pub fn poll(&mut self)->bool{
        let mut changed=false;
        // Never let fast cache reads accumulate unbounded decoded image buffers.
        while self.images.len()<2 {
            let result=match self.rx.as_ref().map(|r|r.try_recv()){Some(Ok(m))=>m,_=>break};
            changed=true;self.in_flight.remove(&result.key);
            match result.result {
                Ok((image,report))=>{
                    self.errors+=report.errors;if self.error.is_none(){self.error=report.error_sample;}
                    if result.key==TileKey::ROOT{self.ready=true;self.cache_hit=report.cache_hit;self.prepared_lines=report.lines;self.prepare_ms=self.started.map_or(0,|s|s.elapsed().as_millis() as u64);}
                    self.images.push_back(ImageUpdate{key:result.key,image});
                }
                Err(error)=>{self.errors+=1;self.failed.insert(result.key);if self.error.is_none(){self.error=Some(error);}}
            }
        }changed
    }
    /// Replace queued off-screen work, not the task already executing. The
    /// executing result is harmlessly cached even after a rapid camera jump.
    pub fn request(&mut self,keys:&[TileKey],resident:&HashSet<TileKey>){
        if !self.ready{return;}let Some(control)=&self.control else{return;};let mut work=control.work.lock().unwrap();
        for key in work.queue.drain(..){self.in_flight.remove(&key);}
        for &key in keys {
            if key==TileKey::ROOT||resident.contains(&key)||self.failed.contains(&key)||self.images.iter().any(|i|i.key==key){continue;}
            if self.in_flight.insert(key){work.queue.push_back(key);}
        }
        control.changed.notify_one();
    }
}
fn send(tx:&mpsc::SyncSender<ResultMessage>,mut message:ResultMessage,control:&Control,wake:Wake)->bool{
    loop{if control.cancel.load(Ordering::Relaxed){return false;}match tx.try_send(message){Ok(())=>{wake();return true;},Err(mpsc::TrySendError::Disconnected(_))=>return false,Err(mpsc::TrySendError::Full(m))=>{message=m;std::thread::sleep(Duration::from_millis(3));}}}
}
