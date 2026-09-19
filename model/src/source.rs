//! Lightweight exact source geometry; no repository-wide token strings.
use std::{fs::Metadata,time::SystemTime};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct FileStamp{pub bytes:u64,pub modified:Option<SystemTime>}
impl FileStamp{pub fn from_metadata(meta:&Metadata)->Self{Self{bytes:meta.len(),modified:meta.modified().ok()}}}
const BLOCK:usize=64;
#[derive(Debug,Default)]
pub struct SourceShape{widths:Vec<u32>,maxima:Vec<u32>}
impl SourceShape{
    pub fn from_text(text:&str)->Self{Self::new(text.lines().map(|s|{let mut column=0u32;for ch in s.trim_end().chars(){column=column.saturating_add(if ch=='\t'{4-column%4}else{1});}column}).collect())}
    pub fn new(mut widths:Vec<u32>)->Self{widths.shrink_to_fit();let maxima=widths.chunks(BLOCK).map(|s|s.iter().copied().max().unwrap_or(0)).collect();Self{widths,maxima}}
    pub fn lines(&self)->usize{self.widths.len()}
    pub fn max_width(&self)->u32{self.maxima.iter().copied().max().unwrap_or(0)}
    pub fn mean_width(&self)->f64{if self.widths.is_empty(){12.0}else{self.widths.iter().map(|&w|w as f64).sum::<f64>()/self.widths.len() as f64}}
    pub fn width(&self,start:usize,end:usize)->u32{let mut at=start.min(self.widths.len());let end=end.min(self.widths.len());let mut max=0;while at<end&&at%BLOCK!=0{max=max.max(self.widths[at]);at+=1;}while at+BLOCK<=end{max=max.max(self.maxima[at/BLOCK]);at+=BLOCK;}while at<end{max=max.max(self.widths[at]);at+=1;}max}
    pub fn storage_bytes(&self)->usize{(self.widths.capacity()+self.maxima.capacity())*4}
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn range_maxima_are_exact(){let v:Vec<u32>=(0..1234).map(|n|((n*17)%93) as u32).collect();let s=SourceShape::new(v.clone());for start in (0..v.len()).step_by(11){for end in (start..v.len()).step_by(31){assert_eq!(s.width(start,end),v[start..end].iter().copied().max().unwrap_or(0));}}}
    #[test]fn tabs_crlf_and_blank_lines(){let s=SourceShape::from_text("a\tb\r\n\nlast\n");assert_eq!(s.lines(),3);assert_eq!(s.width(0,1),5);assert_eq!(s.width(1,2),0);}
}
