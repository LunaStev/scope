//! GPU image-map composition with retained native annotations.
pub mod cache;
pub mod device;
pub mod palette;
pub mod painter;
pub mod image_map;
mod budget;
mod node;
mod presentation;
mod schedule;
mod map;
mod document;
mod labels;
pub use painter::{MapPainter,RenderStats};
pub fn live_design(cx:&mut makepad_widgets::Cx){node::live_design(cx);image_map::live_design(cx);painter::live_design(cx);}
