//! Revision-checked detail sources with lexical checkpoints for visible rows.
use model::FileStamp;
use language::stream::LineLexer;
use std::{collections::HashMap,ops::Range,path::{Path,PathBuf},sync::Arc};
const STEP:usize=128;const BYTES:usize=64*1024*1024;const ENTRIES:usize=64;
pub struct IndexedSource{pub text:String,lines:Vec<Range<usize>>,checkpoints:Vec<LineLexer>}
impl IndexedSource{
    pub fn new(text:String,checkpoint:bool)->Self{
        let mut lines=Vec::new();let mut checkpoints=Vec::new();let mut offset=0;let mut lexer=LineLexer::default();
        for line in text.split_inclusive('\n'){
            let start=offset;offset+=line.len();let mut end=offset-usize::from(line.ends_with('\n'));
            if end>start&&text.as_bytes()[end-1]==b'\r'{end-=1;}
            if checkpoint{if lines.len()%STEP==0{checkpoints.push(lexer);}lexer.spans(&text[start..end],|_,_,_|{});}
            lines.push(start..end);
        }Self{text,lines,checkpoints}
    }
    pub fn line(&self,index:usize)->&str{self.lines.get(index).map_or("",|r|&self.text[r.clone()])}
    pub fn lines(&self)->usize{self.lines.len()}
    pub fn lexer_at(&self,index:usize)->LineLexer{
        let block=index/STEP;let mut lexer=self.checkpoints.get(block).copied().unwrap_or_default();
        let first=if self.checkpoints.is_empty(){0}else{block*STEP};
        for line in first..index.min(self.lines.len()){lexer.spans(self.line(line),|_,_,_|{});}lexer
    }
    fn bytes(&self)->usize{self.text.capacity()+self.lines.capacity()*std::mem::size_of::<Range<usize>>()+self.checkpoints.capacity()*std::mem::size_of::<LineLexer>()}
}
struct Entry{stamp:FileStamp,data:Arc<IndexedSource>,bytes:usize,age:u64}
#[derive(Default)]pub struct Sources{entries:HashMap<PathBuf,Entry>,bytes:usize,clock:u64,pub reads:u64,pub hits:u64}
impl Sources{
    pub fn get(&mut self,path:&Path,stamp:FileStamp,limit:u64,detail:bool)->Result<Arc<IndexedSource>,String>{
        let meta=path.symlink_metadata().map_err(|e|e.to_string())?;
        if !meta.is_file()||meta.file_type().is_symlink()||FileStamp::from_metadata(&meta)!=stamp{return Err("Source changed since indexing; use Re-index".into());}
        self.clock=self.clock.wrapping_add(1);
        if detail{if let Some(entry)=self.entries.get_mut(path).filter(|e|e.stamp==stamp){entry.age=self.clock;self.hits+=1;return Ok(entry.data.clone());}}
        let text=analysis::source::read_text(path,limit).map_err(|e|e.to_string())?.ok_or("Source is no longer UTF-8 text")?;
        if FileStamp::from_metadata(&path.symlink_metadata().map_err(|e|e.to_string())?)!=stamp{return Err("Source changed during reading".into());}
        self.reads+=1;let data=Arc::new(IndexedSource::new(text,detail));let size=data.bytes();
        if detail&&size<=BYTES{
            if let Some(old)=self.entries.remove(path){self.bytes-=old.bytes;}
            while self.entries.len()>=ENTRIES||self.bytes+size>BYTES{
                let Some(old)=self.entries.iter().min_by_key(|(_,e)|e.age).map(|(p,_)|p.clone())else{break;};
                if let Some(old)=self.entries.remove(&old){self.bytes-=old.bytes;}
            }
            self.entries.insert(path.into(),Entry{stamp,data:data.clone(),bytes:size,age:self.clock});self.bytes+=size;
        }Ok(data)
    }
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn checkpoints_preserve_cross_line_comments(){
        let text=format!("/* start\n{}end */ fun x() {{}}\n","inside\n".repeat(300));
        let source=IndexedSource::new(text.clone(),true);let mut sequential=LineLexer::default();
        for(i,line)in text.lines().enumerate(){let mut expected=Vec::new();sequential.spans(line,|a,b,c|expected.push((a,b,c)));let mut actual=Vec::new();source.lexer_at(i).spans(line,|a,b,c|actual.push((a,b,c)));assert_eq!(actual,expected);}
    }
    #[test]fn cached_reads_do_not_accept_changed_revisions(){
        let root=tempfile::tempdir().unwrap();let path=root.path().join("main.wave");std::fs::write(&path,"fun main() {}\n").unwrap();
        let stamp=FileStamp::from_metadata(&path.metadata().unwrap());let mut cache=Sources::default();
        let a=cache.get(&path,stamp,4096,true).unwrap();let b=cache.get(&path,stamp,4096,true).unwrap();assert!(Arc::ptr_eq(&a,&b));assert_eq!((cache.reads,cache.hits),(1,1));
        std::fs::write(&path,"changed size\n").unwrap();assert!(cache.get(&path,stamp,4096,true).is_err());
    }
    #[test]fn crlf_and_empty_input_match_physical_rows(){assert_eq!(IndexedSource::new("a\r\n\r\nend".into(),true).line(1),"");assert_eq!(IndexedSource::new("".into(),true).lines(),0);}
}
