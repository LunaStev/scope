use crate::{Box2, SourceLayout};
use crate::source::READING_SIZE;
#[derive(Clone, Copy, Debug)]
pub struct Camera { pub scale: f64, pub x: f64, pub y: f64 }
impl Default for Camera { fn default() -> Self { Self { scale: 1.0, x: 0.0, y: 0.0 } } }
/// Approximately 3.7x the previous logarithmic scroll sensitivity, anchored to
/// the cursor. Exponential scaling keeps zoom-in and zoom-out symmetric.
pub fn wheel_zoom_factor(delta: f64) -> f64 { (-delta * 0.022).clamp(-0.7,0.7).exp() }
impl Camera {
    pub fn fit(&mut self, world: Box2, viewport: Box2) {
        self.scale = ((viewport.w-40.0).max(1.0)/world.w.max(0.000001)).min((viewport.h-40.0).max(1.0)/world.h.max(0.000001)).clamp(0.00001,1_000_000.0);
        self.x = viewport.x+(viewport.w-world.w*self.scale)*0.5-world.x*self.scale;
        self.y = viewport.y+(viewport.h-world.h*self.scale)*0.5-world.y*self.scale;
    }
    pub fn project(self, b: Box2) -> Box2 { Box2::new(self.x+b.x*self.scale,self.y+b.y*self.scale,b.w*self.scale,b.h*self.scale) }
    pub fn unproject(self, x: f64, y: f64) -> (f64,f64) { ((x-self.x)/self.scale,(y-self.y)/self.scale) }
    pub fn zoom_at(&mut self, factor: f64, x: f64, y: f64) {
        if !factor.is_finite() || factor<=0.0 { return; }
        let (wx,wy) = self.unproject(x,y);
        self.scale = (self.scale*factor).clamp(0.00001,1_000_000.0);
        self.x=x-wx*self.scale; self.y=y-wy*self.scale;
    }
    pub fn read_at(&mut self, page: SourceLayout, x: f64, y: f64) {
        self.zoom_at((READING_SIZE / (page.font_size*self.scale)).max(1.0),x,y);
    }
    pub fn read_page(&mut self, page: SourceLayout, viewport: Box2) {
        self.fit(page.content,viewport);
        self.scale=self.scale.max(READING_SIZE/page.font_size).clamp(0.00001,1_000_000.0);
        self.x=viewport.x+24.0-page.content.x*self.scale;
        self.y=viewport.y+24.0-page.content.y*self.scale;
    }
    pub fn pan(&mut self, dx: f64, dy: f64) { self.x+=dx; self.y+=dy; }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn cursor_anchor_survives_zoom() {
        let mut c=Camera::default(); let before=c.unproject(120.0,50.0);
        c.zoom_at(4.0,120.0,50.0); assert_eq!(before,c.unproject(120.0,50.0));
    }
    #[test] fn fit_is_finite_for_empty_view() {
        let mut c=Camera::default(); c.fit(Box2::default(),Box2::default()); assert!(c.scale.is_finite());
        c.zoom_at(f64::NAN,0.0,0.0); assert!(c.scale.is_finite());
    }
    #[test] fn wheel_is_faster_and_reversible() {
        assert!(wheel_zoom_factor(-10.0) > 1.2);
        assert!((wheel_zoom_factor(10.0)*wheel_zoom_factor(-10.0)-1.0).abs()<1e-12);
    }
    #[test] fn double_click_reaches_readable_text_without_reflow() {
        let page=SourceLayout::new(Box2::new(0.0,0.0,400.0,200.0),1000,80);
        let mut c=Camera::default();let anchor=c.unproject(20.0,30.0);
        c.read_at(page,20.0,30.0);
        assert!((c.scale*page.font_size-READING_SIZE).abs()<1e-10);
        let after=c.unproject(20.0,30.0);
        assert!((anchor.0-after.0).abs()<1e-10);
    }
}
