//! Deterministic geometry and camera math; independent of the GPU and filesystem.
pub mod camera;
pub mod geometry;
pub mod hierarchy;
pub mod squarify;
pub use camera::Camera;
pub use geometry::{Box2, WORLD};
pub use hierarchy::{hit_test, tree_layout};
pub use squarify::squarify;
