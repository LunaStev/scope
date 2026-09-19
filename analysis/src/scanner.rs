use crate::{metrics, options::{ScanOptions, DEFAULT_EXCLUDES}, source::{read_text, prepare}};
use model::{FileInfo, Node, Tree};
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::time::Instant;

pub fn scan(root: &Path, options: &ScanOptions, cancel: &AtomicBool) -> Result<Tree, String> {
    let started = Instant::now();
    let root = root.canonicalize().map_err(|e| format!("{}: {e}", root.display()))?;
    if !root.is_dir() { return Err(format!("Not a directory: {}", root.display())); }
    let mut tree = Tree::new(root.clone());
    let mut directories = HashMap::from([(PathBuf::new(), 0)]);
    let rules = options.clone();
    let pruned = Arc::new(AtomicU64::new(0));
    let pruned_counter = pruned.clone();
    let mut builder = WalkBuilder::new(&root);
    builder.hidden(false).follow_links(false).parents(false)
        .git_ignore(options.respect_ignore).git_exclude(options.respect_ignore)
        .git_global(false).ignore(options.respect_ignore).require_git(false);
    if options.respect_ignore { builder.add_custom_ignore_filename(".scopeignore"); }
    builder.filter_entry(move |entry| {
        if entry.depth() == 0 { return true; }
        let name = entry.file_name().to_string_lossy();
        let excluded = matches!(name.as_ref(), ".git" | ".hg" | ".svn")
            || rules.excludes.iter().any(|n| n == name.as_ref())
            || (rules.default_excludes && entry.file_type().is_some_and(|t| t.is_dir()) && DEFAULT_EXCLUDES.contains(&name.as_ref()));
        if excluded { pruned_counter.fetch_add(1, Ordering::Relaxed); }
        !excluded
    });
    for entry in builder.build() {
        if cancel.load(Ordering::Relaxed) { return Err("Scan cancelled".into()); }
        let entry = match entry { Ok(e) => e, Err(e) => { tree.warning(e.to_string()); continue; } };
        if entry.file_type().is_some_and(|t| t.is_symlink()) { tree.report.skipped_links += 1; continue; }
        if !entry.file_type().is_some_and(|t| t.is_file()) { continue; }
        tree.report.visited_files += 1;
        let path = entry.path();
        let text = match read_text(path, options.max_file_bytes) {
            Ok(Some(text)) => text,
            Ok(None) => { tree.report.skipped_binary += 1; continue; }
            Err(e) if e.kind() == std::io::ErrorKind::FileTooLarge => { tree.report.skipped_large += 1; continue; }
            Err(e) => { tree.warning(format!("{}: {e}", path.display())); continue; }
        };
        if cancel.load(Ordering::Relaxed) { return Err("Scan cancelled".into()); }
        let measured = metrics::measure(path, &text);
        if !measured.classified { tree.report.unclassified_files += 1; }
        let relative = path.strip_prefix(&root).map_err(|e| e.to_string())?.to_path_buf();
        let parent = ensure_directory(relative.parent().unwrap_or(Path::new("")), &mut tree, &mut directories);
        let id = tree.nodes.len();
        tree.nodes.push(Node {
            name: entry.file_name().to_string_lossy().into_owned(), relative,
            parent: Some(parent), children: Vec::new(), stats: measured.stats,
            file: Some(FileInfo { language: measured.language, classified: measured.classified, document: Arc::new(prepare(text)) }),
        });
        tree.nodes[parent].children.push(id);
    }
    tree.report.pruned_entries = pruned.load(Ordering::Relaxed);
    tree.finish();
    tree.report.elapsed_ms = started.elapsed().as_millis() as u64;
    Ok(tree)
}
fn ensure_directory(path: &Path, tree: &mut Tree, dirs: &mut HashMap<PathBuf, usize>) -> usize {
    if let Some(&id) = dirs.get(path) { return id; }
    let parent = ensure_directory(path.parent().unwrap_or(Path::new("")), tree, dirs);
    let id = tree.nodes.len();
    tree.nodes.push(Node::directory(path.file_name().unwrap_or(path.as_os_str()).to_string_lossy().into_owned(), path.to_owned(), Some(parent)));
    tree.nodes[parent].children.push(id);
    dirs.insert(path.to_owned(), id);
    id
}
