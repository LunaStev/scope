use std::ops::Range;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Ink{Text,Keyword,Number,String,Comment}
#[derive(Debug)]
pub struct TextRun{pub column:usize,pub text:String,pub ink:Ink}
#[derive(Debug)]
pub struct Document{
    pub text:String,
    pub lines:Vec<Range<usize>>,
    pub runs:Vec<Vec<TextRun>>,
    pub max_columns:usize,
}
impl Document{
    /// Build offsets without first allocating disposable copies of every line.
    pub fn indexed(text:String)->Self{
        let mut offset=0;let mut max_columns=0;
        let lines:Vec<Range<usize>>=text.split_inclusive('\n').map(|line|{
            let start=offset;offset+=line.len();let mut end=offset-usize::from(line.ends_with('\n'));
            if end>start&&text.as_bytes()[end-1]==b'\r'{end-=1;}
            let mut columns=0;for ch in text[start..end].chars(){columns+=if ch=='\t'{4-columns%4}else{1};}
            max_columns=max_columns.max(columns);start..end
        }).collect();
        let runs=std::iter::repeat_with(Vec::new).take(lines.len()).collect();Self{text,lines,runs,max_columns}
    }
    pub fn new(text:String)->Self{
        let mut doc=Self::indexed(text);
        for index in 0..doc.lines.len(){
            let mut display=String::new();let mut column=0;
            for ch in doc.line(index).chars(){if ch=='\t'{let n=4-column%4;display.extend(std::iter::repeat_n(' ',n));column+=n;}else{display.push(ch);column+=1;}}
            if !display.is_empty(){doc.runs[index].push(TextRun{column:0,text:display,ink:Ink::Text});}
        }doc
    }
    pub fn line(&self,index:usize)->&str{self.lines.get(index).map_or("",|range|&self.text[range.clone()])}
    pub fn storage_bytes(&self)->usize{
        self.text.capacity()+self.lines.capacity()*std::mem::size_of::<Range<usize>>()
        +self.runs.capacity()*std::mem::size_of::<Vec<TextRun>>()
        +self.runs.iter().map(|line|line.capacity()*std::mem::size_of::<TextRun>()+line.iter().map(|run|run.text.capacity()).sum::<usize>()).sum::<usize>()
    }
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn long_lines_are_not_truncated(){let s="x".repeat(1200);let d=Document::new(s.clone());assert_eq!(d.line(0),s);assert_eq!(d.runs[0][0].text.len(),1200);assert_eq!(d.max_columns,1200);}
    #[test]fn original_tabs_and_display_positions_agree(){let d=Document::new("a\tb\r\n".into());assert_eq!(d.line(0),"a\tb");assert_eq!(d.runs[0][0].text,"a   b");assert_eq!(d.max_columns,5);}
}
