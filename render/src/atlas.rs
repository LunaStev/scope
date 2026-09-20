//! One font upload, instanced source glyphs, no per-token shaping in ASCII tiles.
use makepad_widgets::*;
use raster::atlas::{Atlas, Glyph, ATLAS_SIZE};
use std::sync::{Arc, mpsc};

live_design! {
    use link::shaders::*;
    pub DrawAtlasGlyph = {{DrawAtlasGlyph}} {
        texture atlas_texture: texture2d
        color: #bcc5cf
        uv: vec4(0.0, 0.0, 1.0, 1.0)
        draw_depth: 1.0
        varying t: vec2
        fn vertex(self) -> vec4 {
            let ax = self.view_transform * vec4(1.0, 0.0, 0.0, 0.0);
            let ay = self.view_transform * vec4(0.0, 1.0, 0.0, 0.0);
            let off = self.view_transform * vec4(0.0, 0.0, 0.0, 1.0);
            let origin = self.rect_pos * vec2(ax.x, ay.y) + off.xy;
            let size = max(self.rect_size * vec2(ax.x, ay.y), vec2(0.0000001));
            let clipped = clamp(origin + size * self.geom_pos, ax.zw, ay.zw);
            self.pos = (clipped - origin) / size;
            self.t = self.uv.xy + self.pos * self.uv.zw;
            self.world = vec4(clipped, self.draw_depth + self.draw_zbias, 1.0);
            return self.camera_projection * (self.camera_view * self.world);
        }
        fn pixel(self) -> vec4 {
            // A distance-to-coverage conversion in physical screen pixels;
            // zoom never selects a magnified low-resolution glyph bitmap.
            let dx = length(dFdx(self.t) * vec2(1024.0));
            let dy = length(dFdy(self.t) * vec2(1024.0));
            let footprint = max(max(dx, dy), 0.0001);
            let distance = (sample2d(self.atlas_texture, self.t).x - 0.5) * 12.0;
            let coverage = clamp(distance / footprint + 0.5, 0.0, 1.0);
            let metadata = self.view_transform * vec4(0.0, 0.0, 1.0, 0.0);
            let alpha = coverage * self.color.a * metadata.w;
            return vec4(self.color.rgb * metadata.z * alpha, alpha);
        }
    }
}

#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
pub struct DrawAtlasGlyph {
    #[deref] pub draw_super: DrawQuad,
    #[live] pub color: Vec4,
    #[live] pub uv: Vec4,
}

pub struct ResidentAtlas {
    pub texture: Texture,
    pub glyphs: [Glyph; 128],
}

#[derive(Default)]
pub struct AtlasState {
    started: bool,
    rx: Option<mpsc::Receiver<Result<Atlas, String>>>,
    pending: Option<Atlas>,
    pub resident: Option<ResidentAtlas>,
}

impl AtlasState {
    pub fn start(&mut self, font: Arc<Vec<u8>>) {
        if self.started { return; }
        self.started = true;
        // Diagnostic A/B path; never silently selected as an optimization.
        if std::env::var("SCOPE_GLYPH_PATH").as_deref() == Ok("legacy") { return; }
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(|| Atlas::build(&font))
                .unwrap_or_else(|_| Err("Source atlas worker failed".into()));
            let _ = tx.send(result);
            SignalToUI::set_ui_signal();
        });
    }

    pub fn poll(&mut self) -> bool {
        let Some(rx) = &self.rx else { return false; };
        let result = match rx.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return false,
            Err(mpsc::TryRecvError::Disconnected) => Err("Source atlas worker disconnected".into()),
        };
        self.rx = None;
        match result {
            Ok(atlas) => self.pending = Some(atlas),
            Err(error) => eprintln!("scope atlas: {error}; retaining native text fallback"),
        }
        true
    }

    pub fn waiting_upload(&self) -> bool { self.pending.is_some() }

    pub fn upload(&mut self, cx: &mut Cx2d) -> bool {
        let Some(atlas) = self.pending.take() else { return false; };
        let texture = Texture::new_with_format(cx, TextureFormat::VecBGRAu8_32 {
            width: ATLAS_SIZE, height: ATLAS_SIZE, data: Some(atlas.pixels), updated: TextureUpdated::Full,
        });
        self.resident = Some(ResidentAtlas { texture, glyphs: atlas.glyphs });
        true
    }
}

impl DrawAtlasGlyph {
    pub fn glyph(&mut self, cx: &mut Cx2d, glyph: Glyph, x: f64, y: f64) {
        if glyph.size[0] <= 0.0 || glyph.size[1] <= 0.0 { return; }
        self.uv = vec4(glyph.uv[0], glyph.uv[1], glyph.uv[2], glyph.uv[3]);
        self.draw_abs(cx, Rect { pos: dvec2(x + glyph.offset[0] as f64, y + glyph.offset[1] as f64),
            size: dvec2(glyph.size[0] as f64, glyph.size[1] as f64) });
    }
}
