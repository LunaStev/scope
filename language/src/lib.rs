//! Pure language services: no filesystem walking, process execution or GUI.
//! Detection, lexical metrics and display preparation are separate contracts.
pub mod definition;
pub mod detection;
pub mod registry;
pub mod metrics;
pub mod highlight;
pub mod stream;
mod backend;
mod shebang;
pub mod wave;
pub use definition::Definition;
pub use detection::{Detection, Evidence};
pub use registry::Registry;
pub use metrics::{measure, Measurement};
