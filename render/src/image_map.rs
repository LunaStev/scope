//! GPU composition of prebuilt source images. The root texture is never evicted.
//! Missing detail means a lower-resolution parent, never an empty source region.
use makepad_widgets::*;
use layout::{Box2,Camera};
use raster::TileKey;
use std::{collections::{HashMap,HashSet},time::Instant};

live_design! {
    use link::shaders::*;
    pub DrawMapImage = {{DrawMapImage}} {
        texture image: texture2d
        uv_origin: vec2(0.0,0.0)
        uv_extent: vec2(1.0,1.0)
        opacity: 1.0
        dim: 1.0
        fn pixel(self) -> vec4 {
            let color = sample2d(self.image,self.uv_origin+self.pos*self.uv_extent);
            return vec4(color.rgb*self.dim,color.a)*self.opacity;
        }
    }
}
#[derive(Live,LiveHook,LiveRegister)]
#[repr(C)]
pub struct DrawMapImage {
    #[deref] pub draw_super:DrawQuad,
    #[live] pub uv_origin:Vec2,
    #[live] pub uv_extent:Vec2,
    #[live] pub opacity:f32,
    #[live] pub dim:f32,
}
struct Resident {texture:Texture,world:Box2,last_used:u64,born:Instant}
#[derive(Default)]
pub struct ImageMap {pub generation:u64,tiles:HashMap<TileKey,Resident>,frame:u64,pub source_mode:bool}
impl ImageMap {
    pub fn clear(&mut self){self.tiles.clear();self.generation=0;}
    pub fn ready(&self)->bool{self.tiles.contains_key(&TileKey::ROOT)}
    pub fn resident(&self)->HashSet<TileKey>{self.tiles.keys().copied().collect()}
    pub fn len(&self)->usize{self.tiles.len()}
    pub fn upload(&mut self,cx:&mut Cx2d,update:runtime::maps::ImageUpdate,world:Box2,protected:&HashSet<TileKey>){
        const MAX_IMAGES:usize=97; // pinned 16 MiB root + at most 96 MiB details
        if self.tiles.len()>=MAX_IMAGES&&!self.tiles.contains_key(&update.key){
            let victim=self.tiles.iter().filter(|(k,_)|**k!=TileKey::ROOT&&!protected.contains(k)).min_by_key(|(_,v)|v.last_used).map(|(k,_)|*k);
            if let Some(victim)=victim{self.tiles.remove(&victim);}else{return;}
        }
        let texture=Texture::new_with_format(cx,TextureFormat::VecBGRAu8_32{
            width:update.image.width,height:update.image.height,data:Some(update.image.pixels),updated:TextureUpdated::Full,
        });
        self.tiles.insert(update.key,Resident{texture,world:update.key.bounds(world),last_used:self.frame,born:Instant::now()});
    }
    pub fn draw(&mut self,cx:&mut Cx2d,paint:&mut DrawMapImage,camera:Camera,view:Box2,dim:f32)->(usize,bool){
        self.frame=self.frame.wrapping_add(1);let mut keys:Vec<TileKey>=self.tiles.keys().copied().collect();keys.sort();
        let mut count=0;let mut fading=false;
        for key in keys {let tile=self.tiles.get_mut(&key).unwrap();let projected=camera.project(tile.world);let Some(b)=projected.intersection(view)else{continue;};
            tile.last_used=self.frame;
            let opacity=if key==TileKey::ROOT{1.0}else{(tile.born.elapsed().as_secs_f32()/0.12).min(1.0)};fading|=opacity<1.0;
            paint.draw_vars.texture_slots[0]=Some(tile.texture.clone());
            paint.uv_origin=vec2(((b.x-projected.x)/projected.w) as f32,((b.y-projected.y)/projected.h) as f32);
            paint.uv_extent=vec2((b.w/projected.w) as f32,(b.h/projected.h) as f32);
            paint.opacity=opacity;paint.dim=dim;
            paint.draw_abs(cx,Rect{pos:dvec2(b.x,b.y),size:dvec2(b.w,b.h)});count+=1;
        }(count,fading)
    }
}
