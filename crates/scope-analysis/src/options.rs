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
