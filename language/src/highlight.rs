//! Display hints are independent from measurement classifiers.
use model::{Document, Ink, TextRun};
use crate::{Detection, backend::Engine};
pub fn expand_tabs(line: &str) -> String {
    let mut out=String::new();let mut col=0;
    for ch in line.chars(){if ch=='\t'{let n=4-col%4;out.extend(std::iter::repeat_n(' ',n));col+=n;}else{out.push(ch);col+=1;}}
    out
}
pub fn prepare_for(text:String,detection:&Detection)->Document{
    if matches!(detection.engine,Engine::Wave){return crate::wave::prepare(text);}prepare(text)
}
pub fn prepare(text:String)->Document{
    let mut doc=Document::indexed(text);
    for i in 0..doc.lines.len(){doc.runs[i]=runs(doc.line(i));}doc
}
pub fn runs(line:&str)->Vec<TextRun>{
    let chars:Vec<char>=expand_tabs(line).chars().collect();let mut result:Vec<TextRun>=Vec::new();let mut i=0;
    while i<chars.len(){
        let start=i;let ink;
        if chars[i]=='/'&&chars.get(i+1)==Some(&'/'){i=chars.len();ink=Ink::Comment;}
        else if matches!(chars[i],'"'|'\''){
            let quote=chars[i];i+=1;while i<chars.len(){if chars[i]=='\\'{i=(i+2).min(chars.len());}else if chars[i]==quote{i+=1;break;}else{i+=1;}}ink=Ink::String;
        }else if chars[i].is_ascii_digit(){i+=1;while i<chars.len()&&(chars[i].is_ascii_alphanumeric()||matches!(chars[i],'.'|'_')){i+=1;}ink=Ink::Number;}
        else if chars[i].is_alphabetic()||chars[i]=='_'{
            i+=1;while i<chars.len()&&(chars[i].is_alphanumeric()||chars[i]=='_'){i+=1;}
            let word:String=chars[start..i].iter().collect();
            ink=if matches!(word.as_str(),"fn"|"fun"|"func"|"def"|"function"|"class"|"struct"|"enum"|"impl"|"pub"|"let"|"mut"|"const"|"static"|"return"|"if"|"else"|"for"|"while"|"match"|"switch"|"case"|"import"|"from"|"use"|"mod"|"async"|"await"|"trait"|"interface"|"type"|"void"|"int"|"float"|"bool"|"true"|"false"|"null"|"None"|"self"|"this"|"new"|"export"|"public"|"private"){Ink::Keyword}else{Ink::Text};
        }else{i+=1;ink=Ink::Text;}
        let text:String=chars[start..i].iter().collect();
        if let Some(previous)=result.last_mut().filter(|run|run.ink==ink){previous.text.push_str(&text);}else{result.push(TextRun{column:start,text,ink});}
    }result
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn long_source_is_not_truncated(){let s=format!("fn main() {{}} // {}","x".repeat(1024));assert_eq!(runs(&s).iter().map(|r|r.text.as_str()).collect::<String>(),s);}
    #[test]fn blank_lines_and_tabs_survive_preparation(){let d=prepare("\tfn main() {}\n\n// done\n".into());assert_eq!(d.runs.len(),3);assert!(d.runs[1].is_empty());assert_eq!(d.runs[0].iter().map(|r|r.text.as_str()).collect::<String>(),"    fn main() {}");}
}
