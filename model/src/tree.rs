use crate::{Document, Stats};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

pub type NodeId = usize;
#[derive(Debug)]
pub struct FileInfo {
    pub language: String,
    pub classified: bool,
    /// The exact indexed source. Zooming never reads a different file revision.
    pub document: Arc<Document>,
}
#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub relative: PathBuf,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub stats: Stats,
    pub file: Option<FileInfo>,
}
impl Node {
    pub fn directory(name: String, relative: PathBuf, parent: Option<NodeId>) -> Self {
        Self { name, relative, parent, children: Vec::new(), stats: Stats::default(), file: None }
    }
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct ScanReport {
    pub elapsed_ms: u64,
    pub visited_files: u64,
    pub skipped_binary: u64,
    pub skipped_large: u64,
    pub skipped_links: u64,
    pub pruned_entries: u64,
    pub unclassified_files: u64,
    pub warning_count: u64,
    pub warnings: Vec<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct LanguageStats { pub language: String, pub stats: Stats }
#[derive(Debug)]
pub struct Tree {
    pub root: PathBuf,
    pub nodes: Vec<Node>,
    pub report: ScanReport,
    pub languages: Vec<LanguageStats>,
    pub source_memory_bytes: usize,
}
impl Tree {
    pub fn new(root: PathBuf) -> Self {
        let name = root.file_name().unwrap_or(root.as_os_str()).to_string_lossy().into_owned();
        Self { root, nodes: vec![Node::directory(name, PathBuf::new(), None)], report: ScanReport::default(), languages: Vec::new(), source_memory_bytes: 0 }
    }
    pub fn totals(&self) -> Stats { self.nodes.first().map_or(Stats::default(), |n| n.stats) }
    pub fn warning(&mut self, message: String) {
        self.report.warning_count += 1;
        if self.report.warnings.len() < 50 { self.report.warnings.push(message); }
    }
    pub fn finish(&mut self) {
        let mut languages = BTreeMap::<String, Stats>::new();
        self.source_memory_bytes = 0;
        for node in &mut self.nodes {
            if let Some(file) = &node.file {
                languages.entry(file.language.clone()).or_default().add(node.stats);
                self.source_memory_bytes += file.document.storage_bytes();
            } else { node.stats = Stats::default(); }
        }
        for id in (1..self.nodes.len()).rev() {
            if let Some(parent) = self.nodes[id].parent {
                let stats = self.nodes[id].stats;
                self.nodes[parent].stats.add(stats);
            }
        }
        self.languages = languages.into_iter().map(|(language, stats)| LanguageStats { language, stats }).collect();
        self.languages.sort_by(|a,b| b.stats.lines.cmp(&a.stats.lines).then(a.language.cmp(&b.language)));
    }
}
