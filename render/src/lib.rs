//! A single real-source rendering path at every camera scale.
pub mod palette;
pub mod painter;
mod map;
mod document;
pub use painter::{MapPainter, RenderStats};
pub fn live_design(cx: &mut makepad_widgets::Cx) { painter::live_design(cx); }
