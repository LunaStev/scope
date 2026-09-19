//! Read-only parallel indexing; language policy is in the language module.
pub mod metrics;
pub mod options;
pub mod scanner;
pub mod source;
pub mod progress;
mod walk;
mod worker;
pub use options::ScanOptions;
pub use scanner::{scan,scan_with_progress};
