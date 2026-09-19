//! Existing 24 px row: progress first, optional telemetry, renderer last.
//! The renderer is plain muted text, not a badge, overlay or popup.
use super::{frame::clip, Workspace};
use ::layout::Box2;
use ::model::format as f;
use ::render::{device::DeviceInfo, palette as p, MapPainter};
use makepad_widgets::*;

const FONT_SIZE: f32 = 9.0;

#[derive(Default)]
pub(super) struct GraphicsLabel {
    info: DeviceInfo,
    caption: String,
    fitted: String,
    fitted_width: Option<f64>,
}

impl GraphicsLabel {
    pub fn refresh(&mut self, cx: &Cx) {
        if self.info.refresh(cx) || self.caption.is_empty() {
            self.caption = format!("Renderer · {}", self.info.renderer());
            self.fitted_width = None;
        }
    }

    fn fitted(&mut self, cx: &mut Cx2d, paint: &mut MapPainter, width: f64) -> &str {
        if self.fitted_width != Some(width) {
            self.fitted = fit_text(&self.caption, width, |s| measure(cx, paint, s));
            self.fitted_width = Some(width);
        }
        &self.fitted
    }
}

#[derive(Debug)]
struct FooterRegions {
    status: Box2,
    telemetry: Option<Box2>,
    renderer: Box2,
}

impl FooterRegions {
    fn new(foot: Box2) -> Self {
        let padding = 14.0_f64.min(foot.w.max(0.0) / 2.0);
        let width = (foot.w - padding * 2.0).max(0.0);
        let renderer_width = (width * 0.28).min(240.0);
        let gap = 16.0_f64.min(width * 0.04);
        let right = foot.x + foot.w - padding;
        let renderer = Box2::new(right - renderer_width, foot.y, renderer_width, foot.h);
        // Keep progress and the device name; hide performance telemetry first.
        let telemetry = (width >= 900.0).then(|| {
            Box2::new(renderer.x - gap - 260.0, foot.y, 260.0, foot.h)
        });
        let status_right = telemetry.map_or(renderer.x, |b| b.x) - gap;
        let status = Box2::new(
            foot.x + padding, foot.y,
            (status_right - foot.x - padding).max(0.0), foot.h,
        );
        Self { status, telemetry, renderer }
    }
}

impl Workspace {
    pub(super) fn status_bar(&mut self, cx: &mut Cx2d, foot: Box2) {
        self.chrome.fill(cx, foot, p::panel());
        self.chrome.fill(cx, Box2::new(foot.x, foot.y, foot.w, 1.0), p::border());
        let slots = FooterRegions::new(foot);
        let frame = self.state.frame;
        let session = &self.state.session;
        let status = if frame.pending_chunks > 0 {
            format!("Preparing source · {} queued regions", f::count(frame.pending_chunks as u64))
        } else if session.documents.pending() > 0 {
            format!("Loading source · {} active reads", session.documents.pending())
        } else if frame.limited_chunks > 0 || frame.limited_nodes > 0 {
            "Visible detail budget reached · focus a region".into()
        } else if frame.needs_redraw {
            "Preparing map labels".into()
        } else if session.busy() {
            session.status.clone()
        } else if !session.query.is_empty() {
            format!("{} matching files", f::count(session.matched_files as u64))
        } else {
            "Wheel zoom · Drag pan · Double-click read · Home overview".into()
        };
        draw_slot(cx, &mut self.chrome, slots.status, &status, false);
        if let Some(bounds) = slots.telemetry {
            let telemetry = format!("{} batches reused · {:.2} ms CPU", frame.reused_chunks, frame.cpu_submit_ms);
            draw_slot(cx, &mut self.chrome, bounds, &telemetry, true);
        }
        let caption = self.state.graphics.fitted(cx, &mut self.chrome, slots.renderer.w);
        if !caption.is_empty() {
            clip(cx, slots.renderer);
            self.chrome.text_right(cx, slots.renderer.x + slots.renderer.w, foot.y + 7.0, FONT_SIZE, p::muted(), caption);
            cx.end_turtle();
        }
    }

    /// The complete backend name lives away from the canvas and primary metrics.
    pub(super) fn graphics_details(&mut self, cx: &mut Cx2d, x: f64, mut y: f64, width: f64) -> f64 {
        self.chrome.fill(cx, Box2::new(x, y, width, 1.0), p::border());
        y += 22.0;
        self.chrome.text(cx, x, y, 10.0, p::text(), "Graphics renderer");
        y += 25.0;
        let info = &self.state.graphics.info;
        for (label, value) in [("Renderer", info.renderer()), ("Vendor", info.vendor())] {
            self.chrome.text(cx, x, y, FONT_SIZE, p::muted(), label);
            y += 17.0;
            for line in wrap_text(value, width, |s| measure(cx, &mut self.chrome, s)) {
                self.chrome.text(cx, x, y, FONT_SIZE, p::secondary(), &line);
                y += 17.0;
            }
            y += 8.0;
        }
        if info.software_hint() {
            self.chrome.text(cx, x, y, FONT_SIZE, p::muted(), "Software renderer");
            y += 19.0;
        }
        y + 12.0
    }
}

fn measure(cx: &mut Cx2d, paint: &mut MapPainter, text: &str) -> f64 {
    paint.label.text_style.font_size = FONT_SIZE;
    paint.label.font_scale = 1.0;
    paint.label.layout(cx, 0.0, 0.0, None, false, Align::default(), text).size_in_lpxs.width as f64
}

fn draw_slot(cx: &mut Cx2d, paint: &mut MapPainter, bounds: Box2, text: &str, right: bool) {
    let text = fit_text(text, bounds.w, |s| measure(cx, paint, s));
    if text.is_empty() { return; }
    clip(cx, bounds);
    if right {
        paint.text_right(cx, bounds.x + bounds.w, bounds.y + 7.0, FONT_SIZE, p::muted(), &text);
    } else {
        paint.text(cx, bounds.x, bounds.y + 7.0, FONT_SIZE, p::muted(), &text);
    }
    cx.end_turtle();
}

/// Measure with the active font; never slice through a UTF-8 code point.
fn fit_text(text: &str, width: f64, mut measure: impl FnMut(&str) -> f64) -> String {
    if width <= 0.0 || !width.is_finite() { return String::new(); }
    if measure(text) <= width { return text.to_owned(); }
    if measure("…") > width { return String::new(); }
    let ends: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    let (mut lo, mut hi) = (0, ends.len());
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        let end = ends.get(mid).copied().unwrap_or(text.len());
        if measure(&format!("{}…", &text[..end])) <= width { lo = mid; } else { hi = mid - 1; }
    }
    let end = ends.get(lo).copied().unwrap_or(text.len());
    format!("{}…", &text[..end])
}

/// Details wrap the full name, including driver suffixes.
fn wrap_text(text: &str, width: f64, mut measure: impl FnMut(&str) -> f64) -> Vec<String> {
    if width <= 0.0 || !width.is_finite() { return Vec::new(); }
    let mut lines = Vec::new();
    let mut line = String::new();
    for ch in text.chars() {
        let mut candidate = line.clone();
        candidate.push(ch);
        if !line.is_empty() && measure(&candidate) > width {
            lines.push(std::mem::take(&mut line));
        }
        line.push(ch);
    }
    if !line.is_empty() { lines.push(line); }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cells(text: &str) -> f64 { text.chars().count() as f64 }

    #[test]
    fn device_and_progress_never_overlap_at_any_window_width() {
        for width in [0.0, 12.0, 28.0, 80.0, 320.0, 720.0, 1040.0, 1480.0, 3840.0] {
            let r = FooterRegions::new(Box2::new(47.0, 300.0, width, 24.0));
            assert!(r.renderer.w <= 240.0);
            assert!(r.status.w >= 0.0);
            assert!(r.status.x + r.status.w <= r.renderer.x + 0.001);
            if let Some(t) = r.telemetry {
                assert!(r.status.x + r.status.w <= t.x);
                assert!(t.x + t.w <= r.renderer.x);
            }
            assert!(r.renderer.x + r.renderer.w <= 47.0 + width + 0.001);
            assert_eq!(r.renderer.h, 24.0);
        }
    }

    #[test]
    fn compact_window_removes_telemetry_before_device_or_progress() {
        let r = FooterRegions::new(Box2::new(0.0, 0.0, 720.0, 24.0));
        assert!(r.telemetry.is_none());
        assert!(r.status.w > r.renderer.w);
    }

    #[test]
    fn long_names_are_width_fitted_and_unicode_safe() {
        assert_eq!(fit_text("llvmpipe (LLVM 20, 256 bits)", 10.0, cells), "llvmpipe …");
        assert_eq!(fit_text("그래픽장치", 3.0, cells), "그래…");
        assert_eq!(fit_text("GPU", 3.0, cells), "GPU");
        assert_eq!(fit_text("GPU", 0.0, cells), "");
        assert_eq!(fit_text("GPU", 0.5, cells), "");
    }

    #[test]
    fn full_renderer_name_is_preserved_in_details() {
        let name = "Device (driver / version / PCIe)";
        let lines = wrap_text(name, 9.0, cells);
        assert_eq!(lines.concat(), name);
        assert!(lines.iter().all(|s| cells(s) <= 9.0));
    }
}
