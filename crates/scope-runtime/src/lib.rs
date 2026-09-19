//! Application services with no Makepad dependency. The host provides a wake
//! callback; jobs publish immutable snapshots, not mutable UI state.
pub mod cache;
pub mod session;
pub use cache::SourceCache;
pub use session::{Session,Snapshot};
pub type Wake = fn();
pub(crate) fn noop() {}
