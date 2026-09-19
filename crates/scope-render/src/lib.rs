//! GPU presentation of spatial snapshots. No repository walking, job scheduling
//! policy or application shell lives here.
pub mod geometry;
pub mod palette;
pub mod painter;
mod map;
mod document;
pub use painter::{MapPainter,RenderStats};
pub fn live_design(cx:&mut makepad_widgets::Cx){painter::live_design(cx);}
