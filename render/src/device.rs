//! Active backend information. No installed-device enumeration or subprocesses.
//! Some native Makepad 1.0 backends leave GpuInfo empty. Linux then queries the
//! already-current GL context during startup only, and caches the result.
use makepad_widgets::Cx;
#[cfg(target_os = "linux")]
mod native;

#[derive(Debug, Default)]
pub struct DeviceInfo {
    reported_vendor: String,
    reported_renderer: String,
    vendor: String,
    renderer: String,
    #[cfg(target_os = "linux")]
    native_resolved: bool,
    #[cfg(target_os = "linux")]
    native_attempts: u8,
}

impl DeviceInfo {
    pub fn refresh(&mut self, cx: &Cx) -> bool {
        let info = cx.gpu_info();
        if known(&info.renderer).is_some() {
            return self.update(&info.vendor, &info.renderer);
        }
        #[cfg(target_os = "linux")]
        {
            if self.native_resolved { return false; }
            // Early UI draws may precede context creation. Never request redraws
            // for this probe and stop retrying after a few naturally occurring draws.
            if self.native_attempts < 8 {
                self.native_attempts += 1;
                if let Some((vendor, renderer)) = native::active_opengl() {
                    self.native_resolved = true;
                    return self.update(&vendor, &renderer);
                }
            }
        }
        self.update(&info.vendor, &info.renderer)
    }

    pub fn update(&mut self, vendor: &str, renderer: &str) -> bool {
        if self.reported_vendor == vendor && self.reported_renderer == renderer {
            return false;
        }
        self.reported_vendor = vendor.to_owned();
        self.reported_renderer = renderer.to_owned();
        self.vendor = single_line(vendor);
        self.renderer = single_line(renderer);
        if std::env::var_os("SCOPE_TRACE").is_some() {
            eprintln!("scope graphics: renderer={} vendor={}", self.renderer(), self.vendor());
        }
        true
    }

    pub fn renderer(&self) -> &str { known(&self.renderer).unwrap_or("not reported") }
    pub fn vendor(&self) -> &str { known(&self.vendor).unwrap_or("not reported") }

    /// Not a hardware-acceleration verdict for other renderer names.
    pub fn software_hint(&self) -> bool {
        let name = self.renderer.to_ascii_lowercase();
        ["llvmpipe", "softpipe", "swiftshader", "software rasterizer", "basic render driver"]
            .iter().any(|pattern| name.contains(pattern))
    }
}
fn single_line(value: &str) -> String { value.split_whitespace().collect::<Vec<_>>().join(" ") }
fn known(value: &str) -> Option<&str> {
    (!value.is_empty() && !value.eq_ignore_ascii_case("unknown")).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unchanged_backend_information_is_not_reallocated() {
        let mut info = DeviceInfo::default();
        assert!(info.update("Vendor", "Device / PCIe"));
        let pointer = info.renderer().as_ptr();
        assert!(!info.update("Vendor", "Device / PCIe"));
        assert_eq!(info.renderer().as_ptr(), pointer);
    }
    #[test]
    fn delayed_backend_information_replaces_the_unknown_value() {
        let mut info = DeviceInfo::default();
        info.update("unknown", "unknown");
        assert_eq!(info.renderer(), "not reported");
        assert!(info.update("Vendor", "A GPU"));
        assert_eq!(info.renderer(), "A GPU");
    }
    #[test]
    fn renderer_text_cannot_introduce_extra_footer_rows() {
        let mut info = DeviceInfo::default();
        info.update("  vendor\t", "  Device\n (driver version)\r\n");
        assert_eq!(info.renderer(), "Device (driver version)");
        assert_eq!(info.vendor(), "vendor");
    }
    #[test]
    fn software_renderer_is_not_presented_as_a_physical_gpu() {
        let mut info = DeviceInfo::default();
        info.update("Mesa", "llvmpipe (LLVM 20, 256 bits)");
        assert!(info.software_hint());
        info.update("vendor", "unrecognised device");
        assert!(!info.software_hint());
    }
}
