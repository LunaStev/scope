use crate::image::Surface;
use fontdue::{Font,FontSettings};
use std::{collections::HashMap,sync::Arc};

const CANONICAL:f64=32.0;
struct Glyph { width:usize,height:usize, x:f64,y:f64, integral:Vec<f32>, mass:f32 }
impl Glyph {
    fn integral_at(&self,x:f64,y:f64)->f32 {
        let x=x.clamp(0.0,self.width as f64);let y=y.clamp(0.0,self.height as f64);
        let ix=x.floor() as usize;let iy=y.floor() as usize;
        let nx=(ix+1).min(self.width);let ny=(iy+1).min(self.height);
        let fx=(x-ix as f64) as f32;let fy=(y-iy as f64) as f32;
        let s=self.width+1;let a=self.integral[iy*s+ix];let b=self.integral[iy*s+nx];let c=self.integral[ny*s+ix];let d=self.integral[ny*s+nx];
        (a*(1.0-fx)+b*fx)*(1.0-fy)+(c*(1.0-fx)+d*fx)*fy
    }
    fn area(&self,x0:f64,y0:f64,x1:f64,y1:f64)->f32{(self.integral_at(x1,y1)-self.integral_at(x0,y1)-self.integral_at(x1,y0)+self.integral_at(x0,y0)).max(0.0)}
}
/// A fixed glyph raster with exact box integration on minification. Font files
/// are supplied by the host's existing dependency; none is vendored here.
pub struct Glyphs { font:Arc<Font>, ascii:Vec<Glyph>, extra:HashMap<char,Glyph> }
impl Glyphs {
    pub fn new(bytes:&[u8])->Result<Self,String>{
        let font=Arc::new(Font::from_bytes(bytes,FontSettings::default()).map_err(|e|e.to_string())?);
        let ascii=(0..128).map(|ch|Self::make(&font,char::from_u32(ch).unwrap())).collect();
        Ok(Self{font,ascii,extra:HashMap::new()})
    }
    fn make(font:&Font,ch:char)->Glyph {
        let (m,bitmap)=font.rasterize(ch,(CANONICAL*4.0/3.0) as f32);
        let mut integral=vec![0.0;(m.width+1)*(m.height+1)];
        for y in 0..m.height {let mut row=0.0;for x in 0..m.width {row+=bitmap[y*m.width+x] as f32/255.0;integral[(y+1)*(m.width+1)+x+1]=row+integral[y*(m.width+1)+x+1];}}
        let mass=*integral.last().unwrap_or(&0.0);
        let baseline=CANONICAL*1.05;
        Glyph{width:m.width,height:m.height,x:m.xmin as f64,y:baseline-m.ymin as f64-m.height as f64,integral,mass}
    }
    pub fn paint(&mut self,surface:&mut Surface,ch:char,x:f64,y:f64,font_x:f64,font_y:f64,color:[u8;3]) {
        if ch.is_whitespace()||ch.is_control() {return;}
        let glyph=if (ch as u32)<128 {&self.ascii[ch as usize]} else {
            // Keep character-cache memory finite even for adversarial source.
            if !self.extra.contains_key(&ch)&&self.extra.len()>=4096{self.extra.clear();}
            self.extra.entry(ch).or_insert_with(||Self::make(&self.font,ch))
        };
        if glyph.width==0||glyph.height==0||glyph.mass<=0.0{return;}
        let sx=font_x/CANONICAL;let sy=font_y/CANONICAL;
        if sx<=0.0||sy<=0.0||!sx.is_finite()||!sy.is_finite(){return;}
        let gx=x+glyph.x*sx;let gy=y+glyph.y*sy;let w=glyph.width as f64*sx;let h=glyph.height as f64*sy;
        if gx+w<=0.0||gy+h<=0.0||gx>=surface.image.width as f64||gy>=surface.image.height as f64{return;}
        let x0=gx.floor().max(0.0) as usize;let y0=gy.floor().max(0.0) as usize;
        let x1=(gx+w).ceil().min(surface.image.width as f64).max(0.0) as usize;
        let y1=(gy+h).ceil().min(surface.image.height as f64).max(0.0) as usize;
        if x1==x0+1 && y1==y0+1 && gx>=0.0&&gy>=0.0&&gx+w<=surface.image.width as f64&&gy+h<=surface.image.height as f64 {
            surface.deposit(x0,y0,glyph.mass*(sx*sy) as f32,color);return;
        }
        for py in y0..y1 {for px in x0..x1 {
            let area=glyph.area((px as f64-gx)/sx,(py as f64-gy)/sy,((px+1) as f64-gx)/sx,((py+1) as f64-gy)/sy)*(sx*sy) as f32;
            surface.deposit(px,py,area,color);
        }}
    }
}
