use crate::{Wake,noop};
use analysis::{scan,ScanOptions};
use model::{AreaMetric,Tree};
use layout::{self,Box2,Camera,SourceLayout};
use std::{collections::HashSet,path::PathBuf,sync::{Arc,mpsc,atomic::{AtomicBool,Ordering}}};

pub struct Snapshot {pub tree:Arc<Tree>,pub rectangles:Vec<Box2>,pub pages:Vec<Option<SourceLayout>>}
impl Snapshot {
    pub fn new(tree:Arc<Tree>,metric:AreaMetric)->Self{
        let rectangles=layout::tree_layout(&tree,metric);
        let pages=tree.nodes.iter().zip(&rectangles).map(|(node,&bounds)|node.file.as_ref().map(|file|SourceLayout::from_document(bounds,&file.document))).collect();
        Self{tree,rectangles,pages}
    }
}
type JobResult=Result<(Snapshot,AreaMetric),String>;
pub struct Session{
    pub root:PathBuf,pub options:ScanOptions,pub snapshot:Option<Snapshot>,pub camera:Camera,
    pub selected:Option<usize>,pub hovered:Option<usize>,pub focus_pending:Option<usize>,
    pub query:String,pub matches:HashSet<usize>,pub matched_files:usize,pub sources:bool,
    pub status:String,pub metric:AreaMetric,pub requested_metric:AreaMetric,
    rx:Option<mpsc::Receiver<JobResult>>,cancel:Arc<AtomicBool>,wake:Wake,
}
impl Default for Session{
    fn default()->Self{Self{root:PathBuf::new(),options:ScanOptions::default(),snapshot:None,camera:Camera::default(),selected:None,hovered:None,focus_pending:Some(0),query:String::new(),matches:HashSet::new(),matched_files:0,sources:true,status:"Open a source directory".into(),metric:AreaMetric::default(),requested_metric:AreaMetric::default(),rx:None,cancel:Arc::new(AtomicBool::new(false)),wake:noop}}
}
impl Drop for Session{fn drop(&mut self){self.cancel.store(true,Ordering::Relaxed);}}
impl Session{
    pub fn busy(&self)->bool{self.rx.is_some()}
    pub fn open(&mut self,root:PathBuf,options:ScanOptions,wake:Wake){
        self.cancel.store(true,Ordering::Relaxed);self.cancel=Arc::new(AtomicBool::new(false));
        self.root=root.clone();self.options=options.clone();self.wake=wake;
        self.snapshot=None;self.selected=None;self.hovered=None;self.matches.clear();self.matched_files=0;
        self.focus_pending=Some(0);self.status=format!("Indexing {}",root.display());
        let(tx,rx)=mpsc::channel();self.rx=Some(rx);let cancel=self.cancel.clone();let metric=self.requested_metric;
        std::thread::spawn(move||{let result=scan(&root,&options,&cancel).map(|tree|(Snapshot::new(Arc::new(tree),metric),metric));let _=tx.send(result);wake();});
    }
    pub fn refresh(&mut self){self.open(self.root.clone(),self.options.clone(),self.wake);}
    pub fn cycle_metric(&mut self){self.requested_metric=self.requested_metric.next();self.relayout();}
    fn relayout(&mut self){
        let Some(snapshot)=&self.snapshot else{return;};let tree=snapshot.tree.clone();let metric=self.requested_metric;let wake=self.wake;
        let(tx,rx)=mpsc::channel();self.rx=Some(rx);self.status="Rebuilding spatial layout".into();
        std::thread::spawn(move||{let _=tx.send(Ok((Snapshot::new(tree,metric),metric)));wake();});
    }
    pub fn poll(&mut self)->bool{
        let result=match self.rx.as_ref().map(|rx|rx.try_recv()){
            Some(Ok(result))=>result,Some(Err(mpsc::TryRecvError::Disconnected))=>Err("Index worker ended without a snapshot".into()),_=>return false,
        };
        self.rx=None;
        match result{
            Ok((snapshot,metric))=>{
                self.root=snapshot.tree.root.clone();self.snapshot=Some(snapshot);self.metric=metric;
                self.selected=self.selected.or(Some(0));self.focus_pending=self.selected;self.status="Ready".into();self.update_matches();
                if metric!=self.requested_metric{self.relayout();}
            }
            Err(error)=>self.status=error,
        }
        true
    }
    pub fn filter(&mut self,query:String){self.query=query.trim().to_lowercase();self.update_matches();}
    fn update_matches(&mut self){
        self.matches.clear();self.matched_files=0;if self.query.is_empty(){return;}
        if let Some(snapshot)=&self.snapshot{for(id,node)in snapshot.tree.nodes.iter().enumerate(){
            if node.relative.to_string_lossy().to_lowercase().contains(&self.query){self.matches.insert(id);if node.file.is_some(){self.matched_files+=1;}}
        }}
    }
    pub fn focus(&mut self,id:usize){if self.snapshot.as_ref().is_some_and(|s|id<s.tree.nodes.len()){self.selected=Some(id);self.focus_pending=Some(id);}}
    pub fn read_at(&mut self,id:usize,x:f64,y:f64){
        if let Some(page)=self.snapshot.as_ref().and_then(|s|s.pages.get(id)).and_then(Option::as_ref){
            self.selected=Some(id);self.focus_pending=None;self.camera.read_at(page,x,y);
        }else{self.focus(id);}
    }
    pub fn apply_focus(&mut self,viewport:Box2){
        let Some(id)=self.focus_pending.take()else{return;};let Some(snapshot)=&self.snapshot else{return;};
        if let Some(Some(page))=snapshot.pages.get(id){self.camera.read_page(page,viewport);}
        else if let Some(&bounds)=snapshot.rectangles.get(id){self.camera.fit(bounds,viewport);}
    }
    pub fn parent(&mut self){let id=self.snapshot.as_ref().and_then(|s|s.tree.nodes.get(self.selected.unwrap_or(0))).and_then(|n|n.parent).unwrap_or(0);self.focus(id);}
    pub fn hit(&self,x:f64,y:f64)->Option<usize>{let s=self.snapshot.as_ref()?;let(x,y)=self.camera.unproject(x,y);layout::hit_test(&s.tree,&s.rectangles,x,y)}
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn source_is_available_in_the_initial_snapshot(){
        let root=tempfile::tempdir().unwrap();let path=root.path().join("main.rs");std::fs::write(&path,"fn main() {}\n").unwrap();
        let tree=scan(root.path(),&ScanOptions::default(),&AtomicBool::new(false)).unwrap();let snapshot=Snapshot::new(Arc::new(tree),AreaMetric::NonBlank);
        std::fs::write(&path,"changed after scan").unwrap();let id=snapshot.tree.nodes.iter().position(|n|n.name=="main.rs").unwrap();
        assert_eq!(snapshot.tree.nodes[id].file.as_ref().unwrap().document.line(0),"fn main() {}");assert!(snapshot.pages[id].is_some());assert!(snapshot.tree.source_memory_bytes>0);
    }
    #[test]fn changing_area_reuses_the_same_source_documents(){
        let root=tempfile::tempdir().unwrap();std::fs::write(root.path().join("x.rs"),"let x = 1;\n").unwrap();
        let tree=Arc::new(scan(root.path(),&ScanOptions::default(),&AtomicBool::new(false)).unwrap());
        let a=Snapshot::new(tree.clone(),AreaMetric::Lines);let b=Snapshot::new(tree,AreaMetric::Bytes);
        assert!(Arc::ptr_eq(&a.tree,&b.tree));assert_eq!(a.pages.len(),b.pages.len());
    }
}
