//! Controlled local stress measurement; does not assume a downloaded repo.
use analysis::{scan,scan_with_progress,ScanOptions};
use runtime::session::Snapshot;
use model::AreaMetric;
use std::{fs,sync::{Arc,atomic::AtomicBool},time::Instant,path::PathBuf};
fn main(){
    let root=PathBuf::from(std::env::args_os().nth(1).expect("fixture directory"));
    let start=Instant::now();let tree=Arc::new(scan(&root,&ScanOptions::default(),&AtomicBool::new(false)).unwrap());let index_ms=start.elapsed().as_millis();
    let start=Instant::now();let scene=Snapshot::new(tree.clone(),AreaMetric::Lines);let layout_ms=start.elapsed().as_millis();
    let start=Instant::now();let reused=scan_with_progress(&root,&ScanOptions::default(),&AtomicBool::new(false),Some(&tree),|_|{}).unwrap();let warm_ms=start.elapsed().as_millis();
    let status=fs::read_to_string("/proc/self/status").unwrap_or_default();let rss=status.lines().find(|s|s.starts_with("VmHWM:")).unwrap_or("not available").replace('\t'," ");
    println!("{{\"files\":{},\"lines\":{},\"line_index_bytes\":{},\"scan_ms\":{},\"layout_ms\":{},\"warm_scan_ms\":{},\"reused_files\":{},\"nodes\":{},\"peak_rss\":\"{}\"}}",tree.totals().files,tree.totals().lines,tree.source_memory_bytes,index_ms,layout_ms,warm_ms,reused.report.reused_files,scene.rectangles.len(),rss.trim());
}
