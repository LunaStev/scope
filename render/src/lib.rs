//! Retained source tiles and independently scheduled first-paint work.
pub mod cache;
pub mod device;
pub mod palette;
pub mod painter;
mod budget;
mod node;
mod presentation;
mod schedule;
mod map;
mod document;
mod labels;
pub use painter::{MapPainter,RenderStats};
pub fn live_design(cx:&mut makepad_widgets::Cx){node::live_design(cx);painter::live_design(cx);}
