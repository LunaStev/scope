//! Private, bounded, atomic map-image cache. A corrupt or incompatible entry
//! is a miss. Only this cache's own validated filenames are eligible for pruning.
use crate::{Image,TileKey};
use std::{fs,path::{Path,PathBuf},io::{Read,Write,BufWriter}};
const DISK_BUDGET:u64=512*1024*1024;
const MAX_FILE:u64=32*1024*1024;
#[derive(Clone)]pub struct Cache {directory:Option<PathBuf>}
impl Default for Cache {
    fn default()->Self{
        if std::env::var_os("SCOPE_MAP_CACHE_OFF").is_some(){return Self{directory:None};}
        let root=std::env::var_os("SCOPE_MAP_CACHE_DIR").map(PathBuf::from).or_else(||std::env::var_os("XDG_CACHE_HOME").map(|p|PathBuf::from(p).join("scope/maps-v1"))).or_else(||std::env::var_os("HOME").map(|p|PathBuf::from(p).join(".cache/scope/maps-v1")));
        root.map(Self::at).unwrap_or(Self{directory:None})
    }
}
impl Cache {
    pub fn at(directory:PathBuf)->Self{
        if fs::create_dir_all(&directory).is_err(){return Self{directory:None};}
        #[cfg(unix)]{use std::os::unix::fs::PermissionsExt;if fs::set_permissions(&directory,fs::Permissions::from_mode(0o700)).is_err(){return Self{directory:None};}}
        Self{directory:Some(directory)}
    }
    fn path(&self,fingerprint:&str,key:TileKey)->Option<PathBuf>{
        if fingerprint.len()!=64||!fingerprint.bytes().all(|b|b.is_ascii_hexdigit())||!key.valid(){return None;}
        Some(self.directory.as_ref()?.join(format!("{fingerprint}-{}-{}-{}.map.png",key.level,key.x,key.y)))
    }
    pub fn load(&self,fingerprint:&str,key:TileKey)->Option<Image>{
        let path=self.path(fingerprint,key)?;let meta=path.symlink_metadata().ok()?;
        if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>MAX_FILE{return None;}
        let mut bytes=Vec::new();fs::File::open(path).ok()?.take(MAX_FILE+1).read_to_end(&mut bytes).ok()?;if bytes.len() as u64>MAX_FILE{return None;}
        let decoder=png::Decoder::new_with_limits(bytes.as_slice(),png::Limits{bytes:32*1024*1024});let mut reader=decoder.read_info().ok()?;
        let size=key.pixels();let info=reader.info();
        if info.width as usize!=size||info.height as usize!=size||info.color_type!=png::ColorType::Rgba||info.bit_depth!=png::BitDepth::Eight{return None;}
        if reader.output_buffer_size()>size*size*4{return None;}
        let mut rgba=vec![0;size*size*4];reader.next_frame(&mut rgba).ok()?;
        let pixels=rgba.chunks_exact(4).map(|c|(c[3] as u32)<<24|(c[0] as u32)<<16|(c[1] as u32)<<8|c[2] as u32).collect();
        Some(Image{width:size,height:size,pixels})
    }
    pub fn store(&self,fingerprint:&str,key:TileKey,image:&Image)->std::io::Result<()> {
        let Some(path)=self.path(fingerprint,key)else{return Ok(());};
        if image.width!=key.pixels()||image.height!=key.pixels()||image.pixels.len()!=image.width*image.height{return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,"Invalid map dimensions"));}
        let directory=path.parent().unwrap();let mut file=tempfile::NamedTempFile::new_in(directory)?;
        {
            let mut encoder=png::Encoder::new(BufWriter::new(file.as_file_mut()),image.width as u32,image.height as u32);
            encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);encoder.set_compression(png::Compression::Fast);
            let mut writer=encoder.write_header()?;let mut rgba=Vec::with_capacity(image.bytes());
            for &p in &image.pixels{rgba.extend_from_slice(&[(p>>16)as u8,(p>>8)as u8,p as u8,(p>>24)as u8]);}
            writer.write_image_data(&rgba)?;writer.finish()?;
        }
        file.as_file_mut().flush()?;file.persist(&path).map_err(|e|e.error)?;self.prune(Some(&path));Ok(())
    }
    fn prune(&self,keep:Option<&Path>){
        let Some(dir)=&self.directory else{return;};let Ok(entries)=fs::read_dir(dir)else{return;};
        let mut files=Vec::new();let mut total=0u64;
        for entry in entries.flatten(){let name=entry.file_name();let name=name.to_string_lossy();if !name.ends_with(".map.png")||name.len()<70||!name.as_bytes()[..64].iter().all(|b|b.is_ascii_hexdigit()){continue;}
            if let Ok(meta)=entry.path().symlink_metadata(){if !meta.is_file()||meta.file_type().is_symlink(){continue;}total=total.saturating_add(meta.len());files.push((meta.modified().ok(),entry.path(),meta.len()));}}
        if total<=DISK_BUDGET{return;}files.sort_by_key(|f|f.0);
        for(_,path,size)in files{if total<=DISK_BUDGET{break;}if keep==Some(path.as_path()){continue;}if fs::remove_file(&path).is_ok(){total=total.saturating_sub(size);}}
    }
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn corrupt_cache_is_a_miss(){let dir=tempfile::tempdir().unwrap();let c=Cache::at(dir.path().into());let hash="a".repeat(64);let path=c.path(&hash,TileKey::ROOT).unwrap();fs::write(path,b"broken").unwrap();assert!(c.load(&hash,TileKey::ROOT).is_none());}
    #[test]fn invalid_paths_are_not_accepted(){let dir=tempfile::tempdir().unwrap();let c=Cache::at(dir.path().into());assert!(c.path("../../unsafe",TileKey::ROOT).is_none());}
    #[test]fn cache_round_trip_preserves_all_pixels(){let dir=tempfile::tempdir().unwrap();let c=Cache::at(dir.path().into());let hash="b".repeat(64);let key=TileKey{level:3,x:1,y:4};let image=Image::new(512,512,[100,140,180]);c.store(&hash,key,&image).unwrap();assert_eq!(c.load(&hash,key).unwrap().pixels,image.pixels);}
}
