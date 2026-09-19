use std::{fs::File,io::{self,Read},path::Path};
pub use model::{Document,Ink,TextRun};
pub use language::highlight::{prepare,prepare_for,runs};
/// Bounded even if the file grows after the first metadata read.
pub fn read_text(path:&Path,max_bytes:u64)->io::Result<Option<String>>{
    let meta=path.symlink_metadata()?;
    if !meta.is_file()||meta.file_type().is_symlink(){return Ok(None);}
    if meta.len()>max_bytes{return Err(io::Error::new(io::ErrorKind::FileTooLarge,"file size limit exceeded"));}
    let mut bytes=Vec::with_capacity(meta.len() as usize);
    File::open(path)?.take(max_bytes.saturating_add(1)).read_to_end(&mut bytes)?;
    if bytes.len() as u64>max_bytes{return Err(io::Error::new(io::ErrorKind::FileTooLarge,"file grew beyond size limit"));}
    if bytes.contains(&0){return Ok(None);}Ok(String::from_utf8(bytes).ok())
}
/// Avoid mixing an old measurement snapshot with a newly edited source file.
pub fn load_document(path:&Path,stamp:model::FileStamp,limit:u64)->Result<Document,String>{
    let before=path.symlink_metadata().map_err(|e|e.to_string())?;
    if !before.is_file()||before.file_type().is_symlink()||model::FileStamp::from_metadata(&before)!=stamp{return Err("Source changed since indexing; use Re-index".into());}
    let text=read_text(path,limit).map_err(|e|e.to_string())?.ok_or("Source is no longer UTF-8 text")?;
    let after=path.symlink_metadata().map_err(|e|e.to_string())?;
    if model::FileStamp::from_metadata(&after)!=stamp||text.len() as u64!=stamp.bytes{return Err("Source changed during reading; use Re-index".into());}
    let detection=language::Registry::builtin().detect(path,&text);Ok(prepare_for(text,&detection))
}
