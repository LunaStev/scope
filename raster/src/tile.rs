use layout::{Box2, Camera};

pub const ROOT_PIXELS: usize = 2048;
pub const TILE_PIXELS: usize = 512;
pub const MAX_LEVEL: u8 = 24;
pub const MAX_REQUESTS: usize = 80;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TileKey { pub level: u8, pub x: u32, pub y: u32 }
impl TileKey {
    pub const ROOT: Self = Self { level: 0, x: 0, y: 0 };
    pub fn valid(self) -> bool {
        self.level <= MAX_LEVEL && self.x < (1u32 << self.level) && self.y < (1u32 << self.level)
    }
    pub fn bounds(self, world: Box2) -> Box2 {
        let n = (1u32 << self.level.min(MAX_LEVEL)) as f64;
        Box2::new(world.x + world.w * self.x as f64 / n, world.y + world.h * self.y as f64 / n, world.w / n, world.h / n)
    }
    pub fn pixels(self) -> usize { if self.level == 0 { ROOT_PIXELS } else { TILE_PIXELS } }
    pub fn parent(self) -> Self {
        if self.level <= 3 { Self::ROOT } else { Self { level: self.level - 1, x: self.x / 2, y: self.y / 2 } }
    }
    pub fn contains(self, child: Self) -> bool {
        if self.level > child.level { return false; }
        let shift = child.level - self.level;
        (child.x >> shift) == self.x && (child.y >> shift) == self.y
    }
}

/// Only enumerate a screen-sized set, never the full level's 4^z tiles.
pub fn requests(world: Box2, camera: Camera, view: Box2, dpi: f64) -> Vec<TileKey> {
    if world.w <= 0.0 || world.h <= 0.0 || !camera.scale.is_finite() || camera.scale <= 0.0 { return Vec::new(); }
    let density = (world.w.max(world.h) * camera.scale * dpi.clamp(0.5, 4.0)).max(1.0);
    if density <= ROOT_PIXELS as f64 { return Vec::new(); }
    let level = ((density / TILE_PIXELS as f64).log2().ceil() as u8).clamp(3, MAX_LEVEL);
    let (x, y) = camera.unproject(view.x, view.y);
    let visible = Box2::new(x, y, view.w / camera.scale, view.h / camera.scale);
    let halo = Box2::new(visible.x - visible.w * 0.15, visible.y - visible.h * 0.15, visible.w * 1.3, visible.h * 1.3);
    let mut fine = level_keys(world, halo, level);
    let cx = visible.x + visible.w * 0.5;
    let cy = visible.y + visible.h * 0.5;
    fine.sort_by(|a,b| {
        let score = |key: &TileKey| { let b = key.bounds(world); ((b.x+b.w*0.5-cx)/world.w).powi(2) + ((b.y+b.h*0.5-cy)/world.h).powi(2) };
        score(a).total_cmp(&score(b))
    });
    let mut result = Vec::new();
    // A nearby parent sharpens a jump before the individual fine tiles arrive.
    for key in fine.iter().take(48) {
        let parent = key.parent();
        if parent != TileKey::ROOT && !result.contains(&parent) { result.push(parent); }
    }
    for key in fine.into_iter().take(48) { if !result.contains(&key) { result.push(key); } }
    // Pre-fetch one deeper level around the centre; old images remain visible.
    let point = Box2::new(cx-visible.w*0.08, cy-visible.h*0.08, visible.w*0.16, visible.h*0.16);
    if level < MAX_LEVEL {
        for key in level_keys(world, point, level+1).into_iter().take(8) {
            if !result.contains(&key) { result.push(key); }
        }
    }
    result.truncate(MAX_REQUESTS); result
}
fn level_keys(world: Box2, area: Box2, level: u8) -> Vec<TileKey> {
    let Some(area) = world.intersection(area) else { return Vec::new(); };
    let n = 1u32 << level;
    let at = |v: f64, origin: f64, length: f64| (((v-origin)/length*n as f64).floor().max(0.0) as u32).min(n-1);
    let (x0,y0) = (at(area.x,world.x,world.w),at(area.y,world.y,world.h));
    let (x1,y1) = (at(area.x+area.w,world.x,world.w),at(area.y+area.h,world.y,world.h));
    let mut result = Vec::new();
    for y in y0..=y1 { for x in x0..=x1 { result.push(TileKey{level,x,y}); if result.len() >= MAX_REQUESTS { return result; } } }
    result
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn parent_coverage_never_disappears() { let c=TileKey{level:20,x:45678,y:98765};assert!(TileKey::ROOT.contains(c));assert!(c.parent().contains(c)); }
    #[test] fn huge_zoom_has_a_bounded_working_set() { for scale in [0.001,1.0,1000.0,1e6] { let w=Box2::new(0.0,0.0,4000.0,2560.0);let camera=Camera{scale,x:-w.w*scale/2.0+600.0,y:-w.h*scale/2.0+400.0};let r=requests(w,camera,Box2::new(0.0,0.0,1200.0,800.0),2.0);assert!(r.len()<=MAX_REQUESTS);assert!(r.iter().all(|k|k.valid())); } }
    #[test] fn overview_needs_only_the_pinned_image() { assert!(requests(Box2::new(0.0,0.0,4000.0,2560.0),Camera{scale:0.2,x:0.0,y:0.0},Box2::new(0.0,0.0,800.0,600.0),1.0).is_empty()); }
    #[test] fn child_bounds_are_inside_parent() { let w=Box2::new(0.0,0.0,4000.0,2560.0);let c=TileKey{level:8,x:120,y:200};let a=c.bounds(w);let b=c.parent().bounds(w);assert!(b.contains(a.x,a.y)&&b.contains(a.x+a.w,a.y+a.h)); }
}
