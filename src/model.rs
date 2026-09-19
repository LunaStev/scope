use std::path::PathBuf;

pub type NodeId = usize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub files: u64,
    pub lines: u64,
    pub non_blank: u64,
    pub bytes: u64,
}
impl Stats {
    pub fn add(&mut self, other: Self) {
        self.files += other.files;
        self.lines += other.lines;
        self.non_blank += other.non_blank;
        self.bytes += other.bytes;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LinePreview {
    pub line: usize,
    pub indent: usize,
    pub width: usize,
    pub comment: bool,
}
#[derive(Debug)]
pub struct FileInfo {
    pub language: String,
    pub preview: Vec<LinePreview>,
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
    pub fn weight(&self) -> f64 {
        self.stats.non_blank.max(self.stats.files).max(1) as f64
    }
}

#[derive(Debug)]
pub struct Tree {
    pub root: PathBuf,
    pub nodes: Vec<Node>,
    pub skipped_binary: u64,
    pub skipped_large: u64,
    pub warnings: Vec<String>,
    pub warning_count: u64,
}
impl Tree {
    pub fn warning(&mut self, message: String) {
        self.warning_count += 1;
        if self.warnings.len() < 50 { self.warnings.push(message); }
    }
    pub fn finish(&mut self) {
        // Parents precede children: reverse traversal is postorder.
        for id in (1..self.nodes.len()).rev() {
            if let Some(parent) = self.nodes[id].parent {
                let stats = self.nodes[id].stats;
                self.nodes[parent].stats.add(stats);
            }
        }
    }
}
