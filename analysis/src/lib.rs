//! Read-only source indexing and lexical analysis, without a GUI dependency.
pub mod metrics;
pub mod options;
pub mod scanner;
pub mod source;
pub use options::ScanOptions;
pub use scanner::scan;
