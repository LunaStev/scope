//! Complete map backing with retained native SDF text at readable densities.
pub mod cache;
pub mod device;
pub mod palette;
pub mod painter;
pub mod image_map;
pub mod live;
mod budget;
mod node;
mod presentation;
mod schedule;
mod map;
mod document;
mod labels;
pub use painter::{MapPainter,RenderStats};
pub fn live_design(cx:&mut makepad_widgets::Cx){node::live_design(cx);image_map::live_design(cx);live::live_design(cx);painter::live_design(cx);}
