use crate::{Wake,noop};
use analysis::{scan_with_progress,ScanOptions,progress::Progress};
use model::{AreaMetric,Tree};
use layout::{self,Box2,Camera,SourceLayout};
use std::{collections::HashSet,path::PathBuf,sync::{Arc,mpsc,atomic::{AtomicBool,AtomicU64,Ordering}}};
static NEXT_SCENE:AtomicU64=AtomicU64::new(1);
pub struct Snapshot{pub generation:u64,pub tree:Arc<Tree>,pub rectangles:Vec<Box2>,pub pages:Vec<Option<SourceLayout>>}
impl Snapshot{
    pub fn new(tree:Arc<Tree>,metric:AreaMetric)->Self{Self::with_bounds(tree,metric,layout::WORLD)}
    pub fn with_bounds(tree:Arc<Tree>,metric:AreaMetric,bounds:Box2)->Self{
        let rectangles=layout::tree_layout_in(&tree,metric,bounds);
        let pages=tree.nodes.iter().zip(&rectangles).map(|(node,&bounds)|node.file.as_ref().map(|file|SourceLayout::for_shape(bounds,&file.shape))).collect();
        Self{generation:NEXT_SCENE.fetch_add(1,Ordering::Relaxed),tree,rectangles,pages}
    }
}
type JobResult=Result<(Snapshot,AreaMetric),String>;
enum JobEvent{Progress(Progress),Complete(JobResult)}
pub struct Session{
    pub root:PathBuf,pub options:ScanOptions,pub snapshot:Option<Arc<Snapshot>>,
    pub documents:crate::documents::Documents,pub progress:Progress,pub viewport:Box2,bounds:Box2,
    pub camera:Camera,pub selected:Option<usize>,pub hovered:Option<usize>,pub focus_pending:Option<usize>,
    pub query:String,pub matches:HashSet<usize>,pub matched_files:usize,pub sources:bool,pub status:String,
    pub metric:AreaMetric,pub requested_metric:AreaMetric,
    rx:Option<mpsc::Receiver<JobEvent>>,cancel:Arc<AtomicBool>,wake:Wake,
}
impl Default for Session{
    fn default()->Self{Self{root:PathBuf::new(),options:ScanOptions::default(),snapshot:None,documents:crate::documents::Documents::default(),progress:Progress::default(),viewport:Box2::default(),bounds:layout::WORLD,camera:Camera::default(),selected:None,hovered:None,focus_pending:Some(0),query:String::new(),matches:HashSet::new(),matched_files:0,sources:true,status:"Open a source directory".into(),metric:AreaMetric::default(),requested_metric:AreaMetric::default(),rx:None,cancel:Arc::new(AtomicBool::new(false)),wake:noop}}
}
impl Drop for Session{fn drop(&mut self){self.cancel.store(true,Ordering::Relaxed);}}
impl Session{
    pub fn busy(&self)->bool{self.rx.is_some()}
    pub fn open(&mut self,root:PathBuf,options:ScanOptions,wake:Wake){
        self.cancel.store(true,Ordering::Relaxed);self.cancel=Arc::new(AtomicBool::new(false));
        self.root=root.clone();self.options=options.clone();self.wake=wake;
        let previous=self.snapshot.take();self.selected=None;self.hovered=None;
        self.documents=crate::documents::Documents::default();self.documents.wake=wake;self.progress=Progress::default();
        self.matches.clear();self.matched_files=0;self.focus_pending=Some(0);self.status=format!("Indexing {}",root.display());
        let(tx,rx)=mpsc::channel();self.rx=Some(rx);let cancel=self.cancel.clone();let metric=self.requested_metric;let bounds=self.bounds;
        std::thread::spawn(move||{
            let prior=previous.as_ref().map(|s|s.tree.as_ref());
            let result=scan_with_progress(&root,&options,&cancel,prior,|p|{let _=tx.send(JobEvent::Progress(p));wake();}).map(|tree|(Snapshot::with_bounds(Arc::new(tree),metric,bounds),metric));
            let _=tx.send(JobEvent::Complete(result));wake();
        });
    }
    pub fn refresh(&mut self){self.open(self.root.clone(),self.options.clone(),self.wake);}
    pub fn cycle_metric(&mut self){self.requested_metric=self.requested_metric.next();self.relayout();}
    fn relayout(&mut self){
        let Some(snapshot)=&self.snapshot else{return;};let tree=snapshot.tree.clone();let metric=self.requested_metric;let wake=self.wake;let bounds=self.bounds;
        let(tx,rx)=mpsc::channel();self.rx=Some(rx);self.status="Rebuilding spatial layout".into();
        std::thread::spawn(move||{let _=tx.send(JobEvent::Complete(Ok((Snapshot::with_bounds(tree,metric,bounds),metric))));wake();});
    }
    pub fn poll(&mut self)->bool{
        let mut changed=self.documents.poll();
        loop{
            let event=match self.rx.as_ref().map(|rx|rx.try_recv()){
                Some(Ok(event))=>event,
                Some(Err(mpsc::TryRecvError::Disconnected))=>JobEvent::Complete(Err("Index worker ended without a snapshot".into())),
                _=>break,
            };changed=true;
            match event{
                JobEvent::Progress(p)=>{self.progress=p;self.status=format!("Indexing · {} files · {} lines",model::format::count(p.indexed),model::format::count(p.lines));}
                JobEvent::Complete(result)=>{
                    self.rx=None;
                    match result{
                        Ok((snapshot,metric))=>{
                            let same_shape=snapshot.rectangles.first().is_some_and(|r|*r==self.bounds);
                            self.root=snapshot.tree.root.clone();self.snapshot=Some(Arc::new(snapshot));self.metric=metric;
                            self.selected=self.selected.or(Some(0));self.focus_pending=self.selected;self.status="Ready".into();self.update_matches();
                            if metric!=self.requested_metric||!same_shape{self.relayout();}
                        }
                        Err(error)=>self.status=error,
                    }break;
                }
            }
        }changed
    }
    /// Resize changes the scene aspect, not the behavior of ordinary zoom.
    pub fn resize(&mut self,view:Box2){
        if view.w<1.0||view.h<1.0{return;}let next=layout::world_for_viewport(view);let change=(next.w-self.bounds.w).abs()>0.5;
        self.viewport=view;self.bounds=next;if change&&self.snapshot.is_some()&&!self.busy(){self.relayout();}
    }
    pub fn filter(&mut self,query:String){self.query=query.trim().to_lowercase();self.update_matches();}
    fn update_matches(&mut self){self.matches.clear();self.matched_files=0;if self.query.is_empty(){return;}if let Some(snapshot)=&self.snapshot{for(id,node)in snapshot.tree.nodes.iter().enumerate(){if node.relative.to_string_lossy().to_lowercase().contains(&self.query){self.matches.insert(id);if node.file.is_some(){self.matched_files+=1;}}}}}
    pub fn focus(&mut self,id:usize){if self.snapshot.as_ref().is_some_and(|s|id<s.tree.nodes.len()){self.selected=Some(id);self.focus_pending=Some(id);}}
    pub fn read_at(&mut self,id:usize,x:f64,y:f64){let page=self.snapshot.as_ref().and_then(|s|s.pages.get(id)).and_then(Option::as_ref);if let Some(page)=page{self.selected=Some(id);self.focus_pending=None;self.camera.read_at(page,x,y);}else{self.focus(id);}}
    pub fn apply_focus(&mut self,viewport:Box2){let Some(id)=self.focus_pending.take()else{return;};let Some(s)=&self.snapshot else{return;};if let Some(Some(page))=s.pages.get(id){self.camera.read_page(page,viewport);}else if let Some(&b)=s.rectangles.get(id){self.camera.fit(b,viewport);}}
    pub fn parent(&mut self){let id=self.snapshot.as_ref().and_then(|s|s.tree.nodes.get(self.selected.unwrap_or(0))).and_then(|n|n.parent).unwrap_or(0);self.focus(id);}
    pub fn hit(&self,x:f64,y:f64)->Option<usize>{let s=self.snapshot.as_ref()?;let(x,y)=self.camera.unproject(x,y);layout::hit_test(&s.tree,&s.rectangles,x,y)}
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn initial_snapshot_is_lightweight(){let t=tempfile::tempdir().unwrap();std::fs::write(t.path().join("x.wave"),"fun main() {}\n".repeat(1000)).unwrap();let tree=analysis::scan(t.path(),&ScanOptions::default(),&AtomicBool::new(false)).unwrap();assert!(tree.source_memory_bytes<6000);let s=Snapshot::new(Arc::new(tree),AreaMetric::Lines);assert!(s.pages.iter().any(Option::is_some));}
    #[test]fn layout_reuses_shapes_but_changes_generation(){let t=tempfile::tempdir().unwrap();std::fs::write(t.path().join("x.rs"),"let x=1;\n").unwrap();let tree=Arc::new(analysis::scan(t.path(),&ScanOptions::default(),&AtomicBool::new(false)).unwrap());let a=Snapshot::new(tree.clone(),AreaMetric::Lines);let b=Snapshot::new(tree.clone(),AreaMetric::Bytes);assert!(Arc::ptr_eq(&a.tree,&b.tree));assert_ne!(a.generation,b.generation);}
}
