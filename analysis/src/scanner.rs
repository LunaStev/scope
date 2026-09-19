use crate::{options::ScanOptions,progress::Progress,worker::{self,Entry},walk};
use model::{FileInfo,Node,Tree,Stats};
use ignore::WalkState;
use std::{collections::HashMap,path::{Path,PathBuf},sync::{Arc,mpsc,atomic::{AtomicBool,AtomicU64,Ordering}},time::{Instant,Duration}};

pub fn scan(root:&Path,options:&ScanOptions,cancel:&AtomicBool)->Result<Tree,String>{scan_with_progress(root,options,cancel,None,|_|{})}
/// Bounded parallel producers, a single tree owner, and throttled progress.
/// Whole-file source strings and display tokens are not retained by the index.
pub fn scan_with_progress(root:&Path,options:&ScanOptions,cancel:&AtomicBool,previous:Option<&Tree>,mut progress:impl FnMut(Progress))->Result<Tree,String>{
    let started=Instant::now();let root=root.canonicalize().map_err(|e|format!("{}: {e}",root.display()))?;
    if !root.is_dir(){return Err(format!("Not a directory: {}",root.display()));}
    if cancel.load(Ordering::Relaxed){return Err("Scan cancelled".into());}
    let prior:HashMap<PathBuf,(Stats,FileInfo)>=previous.filter(|t|t.root==root).map(|t|t.nodes.iter().filter_map(|n|n.file.as_ref().map(|f|(root.join(&n.relative),(n.stats,f.clone())))).collect()).unwrap_or_default();
    let pruned=Arc::new(AtomicU64::new(0));let walker=walk::builder(&root,options,pruned.clone()).build_parallel();
    let mut tree=Tree::new(root.clone());let mut dirs=HashMap::from([(PathBuf::new(),0)]);
    let mut state=Progress::default();let mut last=Instant::now();let(tx,rx)=mpsc::sync_channel(32);
    std::thread::scope(|scope|{
        let prior=&prior;
        scope.spawn(move||{
            walker.run(||{
                let tx=tx.clone();
                Box::new(move|entry|{
                    if cancel.load(Ordering::Relaxed){return WalkState::Quit;}
                    let item=match entry{
                        Err(e)=>Entry::Warning(e.to_string()),
                        Ok(e)if e.file_type().is_some_and(|t|t.is_symlink())=>Entry::Link,
                        Ok(e)if e.file_type().is_some_and(|t|t.is_file())=>worker::file(e.path().to_owned(),options,prior.get(e.path())),
                        _=>return WalkState::Continue,
                    };
                    if tx.send(item).is_err(){WalkState::Quit}else{WalkState::Continue}
                })
            });
        });
        for item in rx{
            if cancel.load(Ordering::Relaxed){continue;}
            match item{
                Entry::File{path,stats,info,reused}=>{
                    state.visited+=1;state.indexed+=1;state.lines+=stats.lines;state.bytes+=stats.bytes;
                    if !info.classified{tree.report.unclassified_files+=1;}if reused{tree.report.reused_files+=1;}
                    let relative=path.strip_prefix(&root).unwrap_or(&path).to_path_buf();
                    let parent=directory(relative.parent().unwrap_or(Path::new("")),&mut tree,&mut dirs);let id=tree.nodes.len();
                    tree.nodes.push(Node{name:relative.file_name().unwrap_or_default().to_string_lossy().into_owned(),relative,parent:Some(parent),children:Vec::new(),stats,file:Some(info)});
                    tree.nodes[parent].children.push(id);
                }
                Entry::Binary=>{state.visited+=1;tree.report.skipped_binary+=1;}
                Entry::Large=>{state.visited+=1;tree.report.skipped_large+=1;}
                Entry::Link=>tree.report.skipped_links+=1,
                Entry::Warning(e)=>tree.warning(e),
            }
            if last.elapsed()>=Duration::from_millis(150){progress(state);last=Instant::now();}
        }
    });
    if cancel.load(Ordering::Relaxed){return Err("Scan cancelled".into());}
    tree.report.visited_files=state.visited;tree.report.workers=options.worker_count();tree.report.pruned_entries=pruned.load(Ordering::Relaxed);
    tree.finish();tree.report.elapsed_ms=started.elapsed().as_millis() as u64;progress(state);Ok(tree)
}
fn directory(path:&Path,tree:&mut Tree,dirs:&mut HashMap<PathBuf,usize>)->usize{
    if let Some(&id)=dirs.get(path){return id;}
    let mut missing=Vec::new();let mut at=path;
    while !dirs.contains_key(at){missing.push(at.to_owned());at=at.parent().unwrap_or(Path::new(""));}
    let mut parent=dirs[at];
    for part in missing.into_iter().rev(){let id=tree.nodes.len();tree.nodes.push(Node::directory(part.file_name().unwrap_or_default().to_string_lossy().into_owned(),part.clone(),Some(parent)));tree.nodes[parent].children.push(id);dirs.insert(part,id);parent=id;}parent
}
