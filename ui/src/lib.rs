//! Desktop composition: shell, input, dashboard and inspector.
mod app;
mod theme;
pub mod workspace;
use analysis::ScanOptions;
use std::{path::PathBuf,sync::OnceLock};
struct Startup { root: PathBuf, options: ScanOptions }
static STARTUP: OnceLock<Startup> = OnceLock::new();
pub fn run(root: PathBuf, options: ScanOptions) {
    let _ = STARTUP.set(Startup { root, options });
    app::app_main();
}
