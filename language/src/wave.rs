//! Independent Wave viewer lexer: //, nested /* */, escaped strings/chars.
use model::{Document,Ink,Stats,TextRun};
#[derive(Default)]pub struct Lexer{depth:usize}
impl Lexer{
    pub fn line(&mut self,value:&str)->Vec<TextRun>{
        let chars:Vec<char>=crate::highlight::expand_tabs(value).chars().collect();let mut out:Vec<TextRun>=Vec::new();let mut i=0;
        while i<chars.len(){
            let start=i;let ink;
            if self.depth>0||(chars[i]=='/'&&chars.get(i+1)==Some(&'*')){
                if self.depth==0{self.depth=1;i+=2;}
                while i<chars.len(){if chars[i]=='/'&&chars.get(i+1)==Some(&'*'){self.depth+=1;i+=2;}else if chars[i]=='*'&&chars.get(i+1)==Some(&'/'){self.depth-=1;i+=2;if self.depth==0{break;}}else{i+=1;}}
                ink=Ink::Comment;
            }else if chars[i]=='/'&&chars.get(i+1)==Some(&'/'){i=chars.len();ink=Ink::Comment;}
            else if matches!(chars[i],'"'|'\''){
                let quote=chars[i];i+=1;while i<chars.len(){if chars[i]=='\\'{i=(i+2).min(chars.len());}else if chars[i]==quote{i+=1;break;}else{i+=1;}}ink=Ink::String;
            }else if chars[i].is_ascii_digit(){i+=1;while i<chars.len()&&(chars[i].is_ascii_alphanumeric()||matches!(chars[i],'_'|'.')){i+=1;}ink=Ink::Number;}
            else if chars[i].is_alphabetic()||chars[i]=='_'{
                i+=1;while i<chars.len()&&(chars[i].is_alphanumeric()||chars[i]=='_'){i+=1;}
                let word:String=chars[start..i].iter().collect();ink=if keyword(&word){Ink::Keyword}else{Ink::Text};
            }else{i+=1;ink=Ink::Text;}
            let text:String=chars[start..i].iter().collect();if let Some(previous)=out.last_mut().filter(|r|r.ink==ink){previous.text.push_str(&text);}else{out.push(TextRun{column:start,text,ink});}
        }out
    }
}
fn keyword(word:&str)->bool{
    matches!(word,"fun"|"let"|"mut"|"const"|"if"|"else"|"while"|"for"|"return"|"break"|"continue"|"struct"|"enum"|"extern"|"import"|"export"|"true"|"false"|"null"|"asm"|"as"|"sizeof"|"alignof"|"array"|"ptr"|"bool"|"str"|"char"|"i8"|"i16"|"i32"|"i64"|"u8"|"u16"|"u32"|"u64"|"f32"|"f64")
}
/// Allocation-free metrics pass; no display token strings are constructed.
pub fn measure(text:&str)->Stats{
    let mut stats=Stats{files:1,bytes:text.len() as u64,..Stats::default()};let mut depth=0usize;
    for line in text.lines(){
        stats.lines+=1;let line=line.trim_start_matches('\u{feff}');let bytes=line.as_bytes();let mut i=0;let mut code=false;
        while i<bytes.len(){
            if depth>0{if bytes[i..].starts_with(b"/*"){depth+=1;i+=2;}else if bytes[i..].starts_with(b"*/"){depth-=1;i+=2;}else{i+=1;}}
            else if bytes[i..].starts_with(b"//"){break;}
            else if bytes[i..].starts_with(b"/*"){depth=1;i+=2;}
            else if matches!(bytes[i],b'"'|b'\''){
                code=true;let quote=bytes[i];i+=1;while i<bytes.len(){if bytes[i]==b'\\'{i=(i+2).min(bytes.len());}else if bytes[i]==quote{i+=1;break;}else{i+=1;}}
            }else{if !bytes[i].is_ascii_whitespace(){code=true;}i+=1;}
        }
        if line.trim().is_empty(){stats.blanks+=1;}else if code{stats.code+=1;}else{stats.comments+=1;}
    }stats
}
pub fn prepare(text:String)->Document{
    let mut doc=Document::indexed(text);let mut lexer=Lexer::default();
    for i in 0..doc.lines.len(){doc.runs[i]=lexer.line(doc.line(i));}doc
}
