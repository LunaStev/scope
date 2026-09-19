//! UI-independent contracts shared by indexing, layout, CLI and rendering.
pub mod format;
pub mod metrics;
pub mod tree;
pub use metrics::{AreaMetric, Stats};
pub use tree::{FileInfo, LanguageStats, LinePreview, Node, NodeId, ScanReport, Tree};
