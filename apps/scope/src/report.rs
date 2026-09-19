use scope_core::{Tree,format as f};
pub fn print(tree:&Tree,json:bool){
    let s=tree.totals();
    if json{
        let value=serde_json::json!({"schema_version":1,"root":tree.root,"metrics":s,"languages":tree.languages,"scan":tree.report,"classification_engine":"tokei 12.1.2","line_invariant":"physical = code + comment-only + blank + unclassified"});
        println!("{}",serde_json::to_string_pretty(&value).expect("JSON report contains only serializable data"));return;
    }
    println!("Scope - {}\n",tree.root.display());
    for(label,value)in[
        ("Text files",f::count(s.files)),("Physical lines",f::count(s.lines)),
        ("Code lines",f::count(s.code)),("Comment-only lines",f::count(s.comments)),
        ("Blank lines",f::count(s.blanks)),("Unclassified lines",f::count(s.unclassified)),
        ("Text size",format!("{} ({} bytes)",f::bytes(s.bytes),s.bytes)),("Index time",f::duration(tree.report.elapsed_ms)),
    ]{println!("{label:<22}{value:>24}");}
    println!("\nLanguages (physical lines / files)");
    for lang in &tree.languages{println!("  {:<24} {:>12} / {:>8}",lang.language,f::count(lang.stats.lines),f::count(lang.stats.files));}
    println!("\nSkipped: {} binary/non-UTF8, {} oversized, {} symlinks",tree.report.skipped_binary,tree.report.skipped_large,tree.report.skipped_links);
    println!("Explicitly pruned entries: {} (not all ignored descendants)",tree.report.pruned_entries);
    println!("Warnings: {}",tree.report.warning_count);
    for warning in &tree.report.warnings{eprintln!("{warning}");}
}
