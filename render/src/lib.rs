//! A single real-source rendering path with retained GPU geometry.
pub mod palette;
pub mod painter;
mod cache;
mod map;
mod document;
pub use painter::{MapPainter, RenderStats};
pub fn live_design(cx: &mut makepad_widgets::Cx) { painter::live_design(cx); }
