//! Retained glyph geometry. Camera movement updates uniforms, not text or VBOs.
use makepad_widgets::*;
use ::layout::{Box2,Camera,SourceLayout,source::GLYPH_ADVANCE};
use ::model::{AreaMetric,Document,Ink,Tree};
use crate::{palette as p,tiles::{SourceTile,TileKey,slice}};
use std::{collections::HashMap,sync::{Arc,Weak}};

pub struct CachedTile{
    pub list:DrawList2d,pub areas:Vec<Area>,pub runs:usize,pub bytes:usize,
    last_camera:[f32;4],last_clip:[f32;4],
}
#[derive(Default)]
pub struct RetainedSources{
    pub tiles:HashMap<TileKey,CachedTile>,tree:Weak<Tree>,metric:Option<AreaMetric>,dpi:f64,pub bytes:usize,
}
impl RetainedSources{
    pub fn prepare(&mut self,tree:&Arc<Tree>,metric:AreaMetric,dpi:f64){
        if self.tree.upgrade().is_none_or(|old|!Arc::ptr_eq(&old,tree))||self.metric!=Some(metric)||self.dpi!=dpi{
            self.tiles.clear();self.bytes=0;self.tree=Arc::downgrade(tree);self.metric=Some(metric);self.dpi=dpi;
        }
    }
    pub fn clear(&mut self){self.tiles.clear();self.bytes=0;self.tree=Weak::new();self.metric=None;}
}
impl CachedTile{
    pub fn build(cx:&mut Cx2d,code:&mut DrawText,tile:&SourceTile,page:&SourceLayout,doc:&Document)->Self{
        let mut list=DrawList2d::new(cx);list.begin_always(cx);
        code.text_style.font_size=16.0;code.font_scale=(page.font_size/16.0)as f32;let mut runs=0;
        for index in tile.lines.clone(){
            let(x,y)=page.line_origin(index);
            for run in &doc.runs[index]{
                if run.column>=tile.chars.end{break;}
                let skip=tile.chars.start.saturating_sub(run.column);let start=run.column+skip;
                let text=slice(&run.text,skip,tile.chars.end.saturating_sub(start));if text.is_empty(){continue;}
                code.color=match run.ink{
                    Ink::Text=>p::rgb(183,193,208),Ink::Keyword=>p::rgb(176,157,211),Ink::Number=>p::rgb(200,174,132),Ink::String=>p::rgb(150,184,157),Ink::Comment=>p::rgb(105,131,132),
                };
                code.draw_abs(cx,dvec2(x+start as f64*page.font_size*GLYPH_ADVANCE,y),text);runs+=1;
            }
        }
        list.end(cx);
        let stored=&cx.draw_lists[list.id()];let mut areas=Vec::new();let mut bytes=0;
        for i in 0..stored.draw_items.len(){
            let item=&stored.draw_items[i];
            if let(Some(call),Some(data))=(item.draw_call(),item.instances.as_ref()){
                bytes+=data.len()*std::mem::size_of::<f32>();
                if !data.is_empty(){areas.push(Area::Instance(InstanceArea{draw_list_id:list.id(),draw_item_id:i,instance_offset:0,instance_count:data.len()/call.total_instance_slots,redraw_id:stored.redraw_id}));}
            }
        }
        Self{list,areas,runs,bytes,last_camera:[f32::NAN;4],last_clip:[f32::NAN;4]}
    }
    pub fn replay(&mut self,cx:&mut Cx2d,camera:Camera,view:Box2,dim:f32,already_attached:bool){
        if !already_attached{
            if self.areas.is_empty(){return;}
            if self.list.begin_maybe(cx,false).is_redrawing(){self.list.end(cx);}
        }
        let camera=[camera.x as f32,camera.y as f32,camera.scale as f32,dim];
        let clip=[view.x as f32,view.y as f32,(view.x+view.w)as f32,(view.y+view.h)as f32];
        for area in &self.areas{
            if camera!=self.last_camera{if let Some(w)=area.get_write_ref(cx,live_id!(scope_camera),ShaderTy::Vec4,"scope_camera"){w.buffer[..4].copy_from_slice(&camera);}}
            if clip!=self.last_clip{if let Some(w)=area.get_write_ref(cx,live_id!(scope_clip),ShaderTy::Vec4,"scope_clip"){w.buffer[..4].copy_from_slice(&clip);}}
        }
        self.last_camera=camera;self.last_clip=clip;
    }
}
