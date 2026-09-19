//! Background jobs, immutable snapshots and bounded residency. No GUI dependency.
pub mod documents;
pub mod residency;
pub mod session;
pub mod maps;
pub use session::{Session,Snapshot};
pub type Wake = fn();
pub(crate) fn noop() {}
