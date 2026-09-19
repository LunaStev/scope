//! Allocation-free display spans for map construction. Offsets are UTF-8 byte
//! boundaries. Tabs are kept intact; the consumer applies the shared 4-cell rule.
use model::Ink;
#[derive(Default)]
pub struct LineLexer { depth: usize }
impl LineLexer {
    pub fn spans(&mut self, text:&str, mut emit:impl FnMut(usize,usize,Ink)) {
        let b=text.as_bytes();let mut i=0;
        while i<b.len() {
            let start=i;let ink;
            if self.depth>0 || b[i..].starts_with(b"/*") {
                if self.depth==0 {self.depth=1;i+=2;}
                while i<b.len() {
                    if b[i..].starts_with(b"/*"){self.depth+=1;i+=2;}
                    else if b[i..].starts_with(b"*/"){self.depth-=1;i+=2;if self.depth==0{break;}}
                    else {i+=text[i..].chars().next().unwrap().len_utf8();}
                } ink=Ink::Comment;
            } else if b[i..].starts_with(b"//") {i=b.len();ink=Ink::Comment;}
            else if matches!(b[i],b'"'|b'\'') {
                let quote=b[i];i+=1;
                while i<b.len() {if b[i]==b'\\'{i+=1;if i<b.len(){i+=text[i..].chars().next().unwrap().len_utf8();}}
                    else if b[i]==quote{i+=1;break;}else{i+=text[i..].chars().next().unwrap().len_utf8();}}
                ink=Ink::String;
            } else if b[i].is_ascii_digit() {
                i+=1;while i<b.len()&&(b[i].is_ascii_alphanumeric()||matches!(b[i],b'.'|b'_')){i+=1;}ink=Ink::Number;
            } else if b[i].is_ascii_alphabetic()||b[i]==b'_' {
                i+=1;while i<b.len()&&(b[i].is_ascii_alphanumeric()||b[i]==b'_'){i+=1;}
                ink=if keyword(&text[start..i]){Ink::Keyword}else{Ink::Text};
            } else {
                i+=text[i..].chars().next().unwrap().len_utf8();
                while i<b.len()&&b[i].is_ascii_whitespace(){i+=1;}ink=Ink::Text;
            }
            emit(start,i,ink);
        }
    }
}
fn keyword(s:&str)->bool{matches!(s,"fn"|"fun"|"func"|"def"|"function"|"class"|"struct"|"enum"|"impl"|"pub"|"let"|"mut"|"const"|"static"|"return"|"if"|"else"|"for"|"while"|"match"|"switch"|"case"|"import"|"from"|"use"|"mod"|"async"|"await"|"trait"|"interface"|"type"|"void"|"int"|"float"|"bool"|"true"|"false"|"null"|"None"|"self"|"this"|"new"|"export"|"public"|"private"|"break"|"continue"|"extern"|"asm"|"as"|"sizeof"|"alignof"|"array"|"ptr"|"str"|"char"|"i8"|"i16"|"i32"|"i64"|"u8"|"u16"|"u32"|"u64"|"f32"|"f64")}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn all_source_bytes_are_preserved(){let text="fun hi() {\t\"한국어\"; /* :) */ }";let mut lexer=LineLexer::default();let mut out=String::new();lexer.spans(text,|a,b,_|out.push_str(&text[a..b]));assert_eq!(out,text);}
    #[test] fn block_comments_cross_lines(){let mut l=LineLexer::default();l.spans("/* hello",|_,_,ink|assert_eq!(ink,Ink::Comment));let mut parts=Vec::new();l.spans("world */ fun hi() {}",|_,_,ink|parts.push(ink));assert_eq!(parts[0],Ink::Comment);assert!(parts.contains(&Ink::Keyword));}
    #[test] fn quotes_do_not_change_comment_state(){let mut l=LineLexer::default();l.spans("\"/*\"",|_,_,ink|assert_eq!(ink,Ink::String));l.spans("fun",|_,_,ink|assert_eq!(ink,Ink::Keyword));}
}
