//! Retained GPU source tiles, geometry clipping and measured labels.
pub mod cache;
pub mod palette;
pub mod painter;
mod map;
mod document;
mod labels;
pub use painter::{MapPainter,RenderStats};
pub fn live_design(cx:&mut makepad_widgets::Cx){painter::live_design(cx);}
