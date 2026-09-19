//! Background index/source services without a GUI dependency.
pub mod session;
pub mod residency;
pub mod documents;
pub use session::{Session,Snapshot};
pub type Wake=fn();
pub(crate) fn noop(){}
