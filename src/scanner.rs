use crate::model::{FileInfo, LinePreview, Node, Stats, Tree};
use crate::source::{language, read_text};
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

pub const DEFAULT_EXCLUDES: &[&str] = &["target", "node_modules", "build", "dist", ".venv", "venv", "__pycache__", ".cache"];
#[derive(Clone, Debug)]
pub struct ScanOptions {
    pub respect_ignore: bool,
    pub default_excludes: bool,
    pub excludes: Vec<String>,
    pub max_file_bytes: u64,
}
impl Default for ScanOptions {
    fn default() -> Self {
        Self { respect_ignore: true, default_excludes: true, excludes: Vec::new(), max_file_bytes: 8 * 1024 * 1024 }
    }
}

pub fn scan(root: &Path, options: &ScanOptions, cancel: &AtomicBool) -> Result<Tree, String> {
    let root = root.canonicalize().map_err(|e| format!("{}: {e}", root.display()))?;
    if !root.is_dir() { return Err(format!("Not a directory: {}", root.display())); }
    let mut tree = Tree {
        nodes: vec![Node {
            name: root.file_name().unwrap_or(root.as_os_str()).to_string_lossy().into_owned(),
            relative: PathBuf::new(), parent: None, children: Vec::new(), stats: Stats::default(), file: None,
        }],
        root: root.clone(), skipped_binary: 0, skipped_large: 0, warnings: Vec::new(), warning_count: 0,
    };
    let mut directories = HashMap::from([(PathBuf::new(), 0)]);
    let rules = options.clone();
    let mut builder = WalkBuilder::new(&root);
    builder.hidden(false).follow_links(false).parents(false)
        .git_ignore(options.respect_ignore).git_exclude(options.respect_ignore)
        .git_global(false).ignore(options.respect_ignore).require_git(false);
    if options.respect_ignore { builder.add_custom_ignore_filename(".scopeignore"); }
    builder.filter_entry(move |entry| {
        if entry.depth() == 0 { return true; }
        let name = entry.file_name().to_string_lossy();
        if matches!(name.as_ref(), ".git" | ".hg" | ".svn") { return false; }
        if rules.excludes.iter().any(|n| n == name.as_ref()) { return false; }
        !(rules.default_excludes && entry.file_type().is_some_and(|t| t.is_dir()) && DEFAULT_EXCLUDES.contains(&name.as_ref()))
    });
    for entry in builder.build() {
        if cancel.load(Ordering::Relaxed) { return Err("Scan cancelled".into()); }
        let entry = match entry { Ok(e) => e, Err(e) => { tree.warning(e.to_string()); continue; } };
        if !entry.file_type().is_some_and(|t| t.is_file()) { continue; }
        let path = entry.path();
        let text = match read_text(path, options.max_file_bytes) {
            Ok(Some(text)) => text,
            Ok(None) => { tree.skipped_binary += 1; continue; }
            Err(e) if e.kind() == std::io::ErrorKind::FileTooLarge => { tree.skipped_large += 1; continue; }
            Err(e) => { tree.warning(format!("{}: {e}", path.display())); continue; }
        };
        let relative = path.strip_prefix(&root).map_err(|e| e.to_string())?.to_path_buf();
        let parent_path = relative.parent().unwrap_or(Path::new(""));
        let parent = ensure_directory(parent_path, &mut tree, &mut directories);
        let lines = text.lines().count();
        let non_blank = text.lines().filter(|line| !line.trim().is_empty()).count();
        let stride = lines.div_ceil(128).max(1);
        let preview = text.lines().enumerate().step_by(stride).map(|(line, value)| {
            let mut indent = 0;
            for c in value.chars().take_while(|c| c.is_whitespace()) {
                indent += if c == '\t' { 4 - indent % 4 } else { 1 };
            }
            LinePreview {
                line, indent: indent.min(160), width: value.trim().chars().count().min(160),
                comment: value.trim_start().starts_with('#') || value.trim_start().starts_with("//"),
            }
        }).collect();
        let id = tree.nodes.len();
        tree.nodes.push(Node {
            name: entry.file_name().to_string_lossy().into_owned(), relative,
            parent: Some(parent), children: Vec::new(),
            stats: Stats { files: 1, lines: lines as u64, non_blank: non_blank as u64, bytes: text.len() as u64 },
            file: Some(FileInfo { language: language(path), preview }),
        });
        tree.nodes[parent].children.push(id);
    }
    tree.finish();
    Ok(tree)
}

fn ensure_directory(path: &Path, tree: &mut Tree, directories: &mut HashMap<PathBuf, usize>) -> usize {
    if let Some(&id) = directories.get(path) { return id; }
    let parent = ensure_directory(path.parent().unwrap_or(Path::new("")), tree, directories);
    let id = tree.nodes.len();
    tree.nodes.push(Node {
        name: path.file_name().unwrap_or(path.as_os_str()).to_string_lossy().into_owned(),
        relative: path.to_path_buf(), parent: Some(parent), children: Vec::new(), stats: Stats::default(), file: None,
    });
    tree.nodes[parent].children.push(id);
    directories.insert(path.to_path_buf(), id);
    id
}
