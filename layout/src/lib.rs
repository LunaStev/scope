//! Deterministic spatial geometry, independent of the windowing toolkit.
pub mod camera;
pub mod geometry;
pub mod hierarchy;
pub mod source;
pub mod squarify;
pub use camera::{Camera, wheel_zoom_factor};
pub use geometry::{Box2, WORLD};
pub use hierarchy::{hit_test, tree_layout};
pub use source::SourceLayout;
pub use squarify::squarify;
