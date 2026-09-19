//! Retained source geometry and GPU presentation; independent of the app shell.
pub mod palette;
pub mod painter;
pub mod retained;
pub mod tiles;
mod map;
mod performance;
pub use painter::{MapPainter,RenderStats};
pub fn live_design(cx:&mut makepad_widgets::Cx){painter::live_design(cx);}
