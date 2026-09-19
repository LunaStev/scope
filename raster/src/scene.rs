use crate::{font::Glyphs,image::{Surface,Image},tile::TileKey};
use layout::{Box2,SourceLayout};
use model::{Tree,FileStamp,Ink};
use std::{sync::{Arc,atomic::{AtomicBool,Ordering}},time::UNIX_EPOCH};

pub struct Scene {
    pub tree:Arc<Tree>, pub rectangles:Arc<[Box2]>, pub pages:Arc<[Option<SourceLayout>]>,
    pub fingerprint:String, pub max_file_bytes:u64,
}
impl Scene {
    pub fn new(tree:Arc<Tree>,rectangles:Arc<[Box2]>,pages:Arc<[Option<SourceLayout>]>,font:&[u8],max_file_bytes:u64)->Self {
        let mut hash=blake3::Hasher::new();hash.update(b"scope-source-map-v1-box-filter\0");hash.update(font);
        hash.update(tree.root.to_string_lossy().as_bytes());
        let mut ids:Vec<usize>=(0..tree.nodes.len()).collect();ids.sort_by_key(|&id|&tree.nodes[id].relative);
        for id in ids {let n=&tree.nodes[id];hash.update(n.relative.to_string_lossy().as_bytes());hash.update(&[0]);
            for x in [rectangles[id].x,rectangles[id].y,rectangles[id].w,rectangles[id].h] {hash.update(&x.to_bits().to_le_bytes());}
            if let Some(f)=&n.file {hash.update(&f.stamp.bytes.to_le_bytes());let t=f.stamp.modified.and_then(|t|t.duration_since(UNIX_EPOCH).ok()).map_or(0,|t|t.as_nanos());hash.update(&t.to_le_bytes());hash.update(f.language.as_bytes());}
        }
        Self{tree,rectangles,pages,fingerprint:hash.finalize().to_hex().to_string(),max_file_bytes}
    }
    pub fn world(&self)->Box2{self.rectangles.first().copied().unwrap_or(layout::WORLD)}
}
#[derive(Clone,Debug,Default)]
pub struct Report {pub files:u64,pub lines:u64,pub errors:u64,pub error_sample:Option<String>,pub cache_hit:bool}

pub struct Rasterizer {glyphs:Glyphs}
impl Rasterizer {
    pub fn new(font:&[u8])->Result<Self,String>{Ok(Self{glyphs:Glyphs::new(font)?})}
    /// There is no file/glyph count cutoff. All indexed files intersecting this
    /// image contribute their actual source coverage, even below a screen pixel.
    pub fn image(&mut self,scene:&Scene,key:TileKey,cancel:&AtomicBool,mut progress:impl FnMut(u64,u64))->Result<(Image,Report),String> {
        if !key.valid(){return Err("Invalid map tile".into());}
        let bounds=key.bounds(scene.world());let size=key.pixels();let sx=size as f64/bounds.w;let sy=size as f64/bounds.h;
        let project=|b:Box2|Box2::new((b.x-bounds.x)*sx,(b.y-bounds.y)*sy,b.w*sx,b.h*sy);
        let mut image=Surface::new(size);let mut report=Report::default();let mut stack=vec![0usize];let mut leaves=Vec::new();
        while let Some(id)=stack.pop() {
            if cancel.load(Ordering::Relaxed){return Err("Map preparation cancelled".into());}
            let n=&scene.tree.nodes[id];let world=scene.rectangles[id];if !world.intersects(bounds){continue;}
            let b=project(world);let color=n.file.as_ref().map(|f|language_color(&f.language)).unwrap_or([163,175,188]);
            image.rect(b,color,if n.file.is_some(){0.06}else{0.025});
            let edge=(world.w.min(world.h)*0.005*sx.min(sy)).min(0.7);
            for r in [Box2::new(b.x,b.y,b.w,edge),Box2::new(b.x,b.y+b.h-edge,b.w,edge),Box2::new(b.x,b.y,edge,b.h),Box2::new(b.x+b.w-edge,b.y,edge,b.h)] {image.rect(r,color,0.35);}
            if n.file.is_some(){leaves.push(id);}else{stack.extend(n.children.iter().rev().copied());}
        }
        let total=leaves.len() as u64;
        for id in leaves {
            if cancel.load(Ordering::Relaxed){return Err("Map preparation cancelled".into());}
            let node=&scene.tree.nodes[id];let file=node.file.as_ref().unwrap();let Some(page)=&scene.pages[id]else{continue;};
            let path=scene.tree.root.join(&node.relative);
            let source=(|| {
                let meta=path.symlink_metadata().map_err(|e|e.to_string())?;
                if !meta.is_file()||meta.file_type().is_symlink()||FileStamp::from_metadata(&meta)!=file.stamp{return Err("Source changed since indexing; use Re-index".to_owned());}
                let text=analysis::source::read_text(&path,scene.max_file_bytes).map_err(|e|e.to_string())?.ok_or("Source is no longer UTF-8 text")?;
                let meta=path.symlink_metadata().map_err(|e|e.to_string())?;
                if FileStamp::from_metadata(&meta)!=file.stamp{return Err("Source changed during map preparation".to_owned());}Ok(text)
            })();
            let text=match source {Ok(s)=>s,Err(error)=>{report.errors+=1;if report.error_sample.is_none(){report.error_sample=Some(format!("{}: {error}",node.relative.display()));}report.files+=1;progress(report.files,total);continue;}};
            let mut lexer=language::stream::LineLexer::default();
            for (line,value) in text.lines().enumerate() {
                if line%128==0&&cancel.load(Ordering::Relaxed){return Err("Map preparation cancelled".into());}
                if line>=file.shape.lines(){break;}
                let (wx,wy)=page.line_origin(line);let y=(wy-bounds.y)*sy;let font_y=page.font_size*sy;
                let visible=y+page.line_height*sy>=0.0&&y<size as f64;
                let mut column=0usize;
                lexer.spans(value,|a,b,ink| {
                    let color=ink_color(ink);
                    for ch in value[a..b].chars() {
                        if ch=='\t'{column+=4-column%4;continue;}
                        let x=(wx+column as f64*page.font_size*layout::source::GLYPH_ADVANCE-bounds.x)*sx;
                        if visible&&x+page.font_size*sx>=0.0&&x<size as f64 {
                            self.glyphs.paint(&mut image,ch,x,y,page.font_size*sx,font_y,color);
                        }
                        column+=1;
                    }
                });
                report.lines+=1;
            }
            report.files+=1;progress(report.files,total);
        }
        Ok((image.finish(),report))
    }
}
pub fn ink_color(ink:Ink)->[u8;3]{match ink {Ink::Text=>[174,184,196],Ink::Keyword=>[175,165,197],Ink::Number=>[191,167,136],Ink::String=>[153,184,154],Ink::Comment=>[105,133,125]}}
pub fn language_color(name:&str)->[u8;3]{match name{
    "Wave"=>[112,191,182],"Rust"=>[196,153,124],"C"|"C Header"|"C++"|"C++ Header"=>[129,155,193],"Python"=>[188,177,133],"JavaScript"|"JSX"=>[190,178,120],"TypeScript"|"TSX"=>[125,161,194],"Go"=>[124,178,185],"Java"|"Kotlin"=>[169,150,187],"JSON"|"TOML"|"YAML"=>[136,174,158],"Markdown"|"HTML"|"CSS"=>[150,160,185],s if s.starts_with("Unknown")=>[196,169,126],_=>[140,167,178]
}}
