//! Background indexing and immutable scene snapshots. No GUI dependency.
pub mod session;
pub use session::{Session, Snapshot};
pub type Wake = fn();
pub(crate) fn noop() {}
