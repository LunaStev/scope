//! Linux-only fallback for native Makepad backends without populated GpuInfo.
//! Inspect the caller thread's existing GLX/EGL context; never create or switch one.
use std::ffi::{c_void, CStr};

#[link(name = "GL")]
extern "C" {
    fn glXGetCurrentContext() -> *mut c_void;
    fn glGetString(name: u32) -> *const u8;
}
#[link(name = "EGL")]
extern "C" {
    fn eglGetCurrentContext() -> *mut c_void;
}

pub(super) fn active_opengl() -> Option<(String, String)> {
    // SAFETY: These functions only inspect thread-local current-context handles.
    // No GL string query is made unless the renderer has already made a context current.
    let current = unsafe {
        !glXGetCurrentContext().is_null() || !eglGetCurrentContext().is_null()
    };
    if !current { return None; }
    // SAFETY: With a current context, GL_VENDOR/GL_RENDERER return driver-owned,
    // NUL-terminated byte strings (or null on failure). Copy before returning;
    // no pointers survive this call and no graphics state is modified.
    unsafe {
        let renderer = glGetString(0x1F01); // GL_RENDERER
        if renderer.is_null() { return None; }
        let renderer = CStr::from_ptr(renderer.cast()).to_string_lossy().into_owned();
        if renderer.trim().is_empty() || renderer.eq_ignore_ascii_case("unknown") { return None; }
        let vendor = glGetString(0x1F00); // GL_VENDOR
        let vendor = if vendor.is_null() {
            String::new()
        } else {
            CStr::from_ptr(vendor.cast()).to_string_lossy().into_owned()
        };
        Some((vendor, renderer))
    }
}
