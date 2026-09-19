use crate::{options::ScanOptions,metrics,source::read_text};
use model::{FileInfo,FileStamp,SourceShape,Stats};
use std::{path::PathBuf,sync::Arc};
pub(crate) enum Entry{File{path:PathBuf,stats:Stats,info:FileInfo,reused:bool},Binary,Large,Link,Warning(String)}
pub(crate) fn file(path:PathBuf,options:&ScanOptions,old:Option<&(Stats,FileInfo)>)->Entry{
    let work=||->Result<Entry,std::io::Error>{
        let meta=path.symlink_metadata()?;let stamp=FileStamp::from_metadata(&meta);
        if !meta.is_file()||meta.file_type().is_symlink(){return Ok(Entry::Link);}
        if stamp.bytes>options.max_file_bytes{return Ok(Entry::Large);}
        if let Some((stats,info))=old.filter(|(_,f)|f.stamp==stamp&&stamp.modified.is_some()){
            return Ok(Entry::File{path:path.clone(),stats:*stats,info:info.clone(),reused:true});
        }
        let text=match read_text(&path,options.max_file_bytes){Ok(Some(t))=>t,Ok(None)=>return Ok(Entry::Binary),Err(e)if e.kind()==std::io::ErrorKind::FileTooLarge=>return Ok(Entry::Large),Err(e)=>return Err(e)};
        let after=FileStamp::from_metadata(&path.symlink_metadata()?);
        if stamp!=after{return Ok(Entry::Warning(format!("{} changed during indexing",path.display())));}
        let m=metrics::measure(&path,&text);let shape=Arc::new(SourceShape::from_text(&text));
        Ok(Entry::File{path:path.clone(),stats:m.stats,info:FileInfo{language:m.language,classified:m.classified,shape,stamp},reused:false})
    };
    work().unwrap_or_else(|e|Entry::Warning(format!("{}: {e}",path.display())))
}
