//! Index/layout stress measurement, not a GPU or full-application benchmark.
use analysis::{scan_with_progress, ScanOptions};
use model::AreaMetric;
use runtime::session::Snapshot;
use std::{fs, path::PathBuf, sync::{Arc, atomic::AtomicBool}, time::Instant};

fn main() {
    let mut args = std::env::args_os().skip(1);
    let root = PathBuf::from(args.next().expect("fixture directory"));
    let expected_files: Option<u64> = args.next().map(|n| n.to_str().unwrap().parse().unwrap());
    let expected_lines: Option<u64> = args.next().map(|n| n.to_str().unwrap().parse().unwrap());
    let options = ScanOptions::default();
    let start = Instant::now();
    let mut updates = 0;
    let mut first_update = None;
    let tree = Arc::new(scan_with_progress(&root, &options, &AtomicBool::new(false), None, |_| {
        updates += 1;
        first_update.get_or_insert(start.elapsed().as_millis());
    }).expect("initial index"));
    let scan_ms = start.elapsed().as_millis();
    if let Some(expected) = expected_files { assert_eq!(tree.totals().files, expected); }
    if let Some(expected) = expected_lines { assert_eq!(tree.totals().lines, expected); }
    assert!(tree.totals().is_consistent());
    assert_eq!(tree.report.warning_count, 0);
    assert_eq!(tree.report.unclassified_files, 0, "all fixture languages should classify");

    let start = Instant::now();
    let scene = Snapshot::new(tree.clone(), AreaMetric::Lines);
    let layout_ms = start.elapsed().as_millis();
    assert_eq!(scene.rectangles.len(), tree.nodes.len());
    for rectangle in scene.rectangles.iter() { assert!(rectangle.area().is_finite()); }

    let start = Instant::now();
    let reused = scan_with_progress(&root, &options, &AtomicBool::new(false), Some(&tree), |_| {})
        .expect("same-process warm index");
    let warm_ms = start.elapsed().as_millis();
    assert_eq!(reused.totals(), tree.totals());
    assert_eq!(reused.report.reused_files, tree.totals().files);
    let status = fs::read_to_string("/proc/self/status").unwrap_or_default();
    let peak_kib = status.lines().find(|line| line.starts_with("VmHWM:"))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u64>().ok());
    let peak = peak_kib.map_or("null".into(), |n| n.to_string());
    println!(concat!("{{\"files\":{},\"lines\":{},\"text_bytes\":{},\"line_index_bytes\":{},",
        "\"scan_ms\":{},\"layout_ms\":{},\"warm_scan_ms\":{},\"reused_files\":{},",
        "\"nodes\":{},\"workers\":{},\"progress_updates\":{},\"first_progress_ms\":{},",
        "\"peak_rss_kib\":{},\"os_page_cache_dropped\":false,",
        "\"scope\":\"index + layout + same-process warm reuse; excludes GPU/source residency\"}}"),
        tree.totals().files, tree.totals().lines, tree.totals().bytes, tree.source_memory_bytes,
        scan_ms, layout_ms, warm_ms, reused.report.reused_files, scene.rectangles.len(),
        tree.report.workers, updates, first_update.unwrap_or(0), peak);
}
