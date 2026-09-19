use layout::Box2;

/// Opaque BGRA words, directly transferable to the native GPU texture.
#[derive(Clone, Debug)]
pub struct Image { pub width: usize, pub height: usize, pub pixels: Vec<u32> }
impl Image {
    pub fn new(width: usize, height: usize, color: [u8;3]) -> Self {
        let word = 0xff000000 | (color[0] as u32)<<16 | (color[1] as u32)<<8 | color[2] as u32;
        Self { width, height, pixels: vec![word; width*height] }
    }
    pub fn bytes(&self) -> usize { self.pixels.len()*4 }
}
/// Area coverage accumulates before compositing. Subpixel source is not lost
/// to repeated integer rounding or replaced with synthetic line bars.
pub struct Surface { pub image: Image, ink: Vec<[f32;4]> }
impl Surface {
    pub fn new(size: usize) -> Self { Self { image:Image::new(size,size,[16,20,24]),ink:vec![[0.0;4];size*size] } }
    pub fn rect(&mut self, b: Box2, color: [u8;3], opacity: f32) {
        let x0=b.x.floor().max(0.0) as usize; let y0=b.y.floor().max(0.0) as usize;
        let x1=(b.x+b.w).ceil().max(0.0).min(self.image.width as f64) as usize;
        let y1=(b.y+b.h).ceil().max(0.0).min(self.image.height as f64) as usize;
        for y in y0..y1 { for x in x0..x1 {
            let area=((x as f64+1.0).min(b.x+b.w)-(x as f64).max(b.x)).max(0.0)*((y as f64+1.0).min(b.y+b.h)-(y as f64).max(b.y)).max(0.0);
            let a=(area as f32*opacity).clamp(0.0,1.0);let p=&mut self.image.pixels[y*self.image.width+x];
            let c=|shift:u32,value:u8| (((*p>>shift)&255) as f32*(1.0-a)+value as f32*a).round() as u32;
            *p=0xff000000 | c(16,color[0])<<16 | c(8,color[1])<<8 | c(0,color[2]);
        } }
    }
    pub fn deposit(&mut self,x:usize,y:usize,coverage:f32,color:[u8;3]) {
        if x>=self.image.width||y>=self.image.height||coverage<=0.0 { return; }
        let p=&mut self.ink[y*self.image.width+x];
        for c in 0..3 {p[c]+=color[c] as f32*coverage;}p[3]+=coverage;
    }
    pub fn finish(mut self) -> Image {
        for (pixel, ink) in self.image.pixels.iter_mut().zip(self.ink) {
            if ink[3]<=0.0 {continue;}let a=ink[3].min(1.0);
            let value=|shift:u32,c:usize| (((( *pixel>>shift)&255) as f32*(1.0-a) + ink[c]/ink[3]*a).round().clamp(0.0,255.0)) as u32;
            *pixel=0xff000000 | value(16,0)<<16 | value(8,1)<<8 | value(0,2);
        } self.image
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn tiny_glyphs_accumulate_instead_of_rounding_to_zero(){let mut s=Surface::new(2);for _ in 0..100{s.deposit(0,0,0.005,[200,200,200]);}let i=s.finish();assert!(((i.pixels[0]>>16)&255)>90);}
    #[test] fn painting_is_clipped(){let mut s=Surface::new(3);s.rect(Box2::new(-100.0,-20.0,101.0,21.0),[255,0,0],1.0);assert_eq!(s.finish().pixels[0],0xffff0000);}
}
