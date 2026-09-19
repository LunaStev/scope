//! Bounded, read-only indexing. This crate has no windowing/GPU dependency.
pub mod metrics;
pub mod options;
pub mod preview;
pub mod scanner;
pub mod source;
pub use options::ScanOptions;
pub use scanner::scan;
