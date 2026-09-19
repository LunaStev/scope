use crate::layout::Box2;

#[derive(Clone, Copy, Debug)]
pub struct Camera { pub scale: f64, pub x: f64, pub y: f64 }
impl Default for Camera { fn default() -> Self { Self { scale: 1.0, x: 0.0, y: 0.0 } } }
impl Camera {
    pub fn fit(&mut self, world: Box2, viewport: Box2) {
        self.scale = ((viewport.w - 36.0).max(1.0) / world.w.max(0.000001))
            .min((viewport.h - 36.0).max(1.0) / world.h.max(0.000001)).clamp(0.00001, 1_000_000.0);
        self.x = viewport.x + (viewport.w - world.w*self.scale)*0.5 - world.x*self.scale;
        self.y = viewport.y + (viewport.h - world.h*self.scale)*0.5 - world.y*self.scale;
    }
    pub fn project(self, world: Box2) -> Box2 { Box2::new(self.x + world.x*self.scale, self.y + world.y*self.scale, world.w*self.scale, world.h*self.scale) }
    pub fn unproject(self, x:f64, y:f64) -> (f64,f64) { ((x-self.x)/self.scale, (y-self.y)/self.scale) }
    pub fn zoom_at(&mut self, factor:f64, x:f64, y:f64) {
        let (wx,wy) = self.unproject(x,y);
        self.scale = (self.scale*factor).clamp(0.00001,1_000_000.0);
        self.x = x-wx*self.scale; self.y = y-wy*self.scale;
    }
    pub fn pan(&mut self, dx:f64,dy:f64) { self.x += dx; self.y += dy; }
}
