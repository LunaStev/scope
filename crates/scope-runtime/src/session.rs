use crate::{SourceCache,Wake,noop};
use scope_analysis::{scan,ScanOptions};
use scope_core::{AreaMetric,Tree};
use scope_layout::{self,Box2,Camera};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc,mpsc,atomic::{AtomicBool,Ordering}};

pub struct Snapshot{pub tree:Arc<Tree>,pub rectangles:Vec<Box2>}
type JobResult=Result<(Snapshot,AreaMetric),String>;
pub struct Session{
    pub root:PathBuf,
    pub options:ScanOptions,
    pub snapshot:Option<Snapshot>,
    pub camera:Camera,
    pub selected:Option<usize>,
    pub hovered:Option<usize>,
    pub focus_pending:Option<usize>,
    pub query:String,
    pub matches:HashSet<usize>,
    pub matched_files:usize,
    pub sources:bool,
    pub status:String,
    pub metric:AreaMetric,
    pub requested_metric:AreaMetric,
    pub cache:SourceCache,
    rx:Option<mpsc::Receiver<JobResult>>,
    cancel:Arc<AtomicBool>,
    wake:Wake,
}
impl Default for Session{
    fn default()->Self{
        Self{root:PathBuf::new(),options:ScanOptions::default(),snapshot:None,camera:Camera::default(),selected:None,hovered:None,focus_pending:Some(0),query:String::new(),matches:HashSet::new(),matched_files:0,sources:true,status:"Open a source directory".into(),metric:AreaMetric::default(),requested_metric:AreaMetric::default(),cache:SourceCache::default(),rx:None,cancel:Arc::new(AtomicBool::new(false)),wake:noop}
    }
}
impl Drop for Session{fn drop(&mut self){self.cancel.store(true,Ordering::Relaxed);}}
impl Session{
    pub fn busy(&self)->bool{self.rx.is_some()}
    pub fn open(&mut self,root:PathBuf,options:ScanOptions,wake:Wake){
        self.cancel.store(true,Ordering::Relaxed);
        self.cancel=Arc::new(AtomicBool::new(false));
        self.root=root.clone();self.options=options.clone();self.wake=wake;
        self.snapshot=None;self.selected=None;self.hovered=None;self.matches.clear();self.matched_files=0;
        self.cache=SourceCache::default();self.cache.wake=wake;
        self.focus_pending=Some(0);self.status=format!("Indexing {}",root.display());
        let(tx,rx)=mpsc::channel();self.rx=Some(rx);let cancel=self.cancel.clone();let metric=self.requested_metric;
        std::thread::spawn(move||{
            let result=scan(&root,&options,&cancel).map(|tree|{
                let rectangles=scope_layout::tree_layout(&tree,metric);
                (Snapshot{tree:Arc::new(tree),rectangles},metric)
            });
            let _=tx.send(result);wake();
        });
    }
    pub fn refresh(&mut self){self.open(self.root.clone(),self.options.clone(),self.wake);}
    pub fn cycle_metric(&mut self){
        self.requested_metric=self.requested_metric.next();
        let Some(snapshot)=&self.snapshot else{return;};
        let tree=snapshot.tree.clone();let metric=self.requested_metric;let wake=self.wake;
        let(tx,rx)=mpsc::channel();self.rx=Some(rx);self.status="Rebuilding spatial layout".into();
        std::thread::spawn(move||{
            let rectangles=scope_layout::tree_layout(&tree,metric);
            let _=tx.send(Ok((Snapshot{tree,rectangles},metric)));wake();
        });
    }
    pub fn poll(&mut self)->bool{
        let mut changed=self.cache.poll();
        let result=self.rx.as_ref().and_then(|rx|rx.try_recv().ok());
        if let Some(result)=result{
            self.rx=None;changed=true;
            match result{
                Ok((snapshot,metric))=>{
                    self.root=snapshot.tree.root.clone();self.snapshot=Some(snapshot);self.metric=metric;
                    self.selected=self.selected.or(Some(0));self.focus_pending=self.selected;
                    self.status="Ready · local, read-only snapshot".into();self.update_matches();
                },
                Err(error)=>self.status=error,
            }
        }
        changed
    }
    pub fn filter(&mut self,query:String){self.query=query.trim().to_lowercase();self.update_matches();}
    fn update_matches(&mut self){
        self.matches.clear();self.matched_files=0;if self.query.is_empty(){return;}
        if let Some(snapshot)=&self.snapshot{
            for(id,node)in snapshot.tree.nodes.iter().enumerate(){
                if node.relative.to_string_lossy().to_lowercase().contains(&self.query){
                    self.matches.insert(id);if node.file.is_some(){self.matched_files+=1;}
                }
            }
        }
    }
    pub fn focus(&mut self,id:usize){
        if self.snapshot.as_ref().is_some_and(|s|id<s.tree.nodes.len()){self.selected=Some(id);self.focus_pending=Some(id);}
    }
    pub fn parent(&mut self){
        let id=self.snapshot.as_ref().and_then(|s|s.tree.nodes.get(self.selected.unwrap_or(0))).and_then(|n|n.parent).unwrap_or(0);
        self.focus(id);
    }
    pub fn hit(&self,x:f64,y:f64)->Option<usize>{
        let s=self.snapshot.as_ref()?;let(x,y)=self.camera.unproject(x,y);
        scope_layout::hit_test(&s.tree,&s.rectangles,x,y)
    }
    pub fn request_source(&mut self,id:usize){
        let Some(snapshot)=&self.snapshot else{return;};
        let Some(node)=snapshot.tree.nodes.get(id)else{return;};
        if node.file.is_some(){self.cache.request(id,snapshot.tree.root.join(&node.relative),self.options.max_file_bytes);}
    }
}
