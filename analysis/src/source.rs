use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
pub use model::{Document, Ink, TextRun};

/// Read only; bounded even if a file grows after stat().
pub fn read_text(path: &Path, max_bytes: u64) -> io::Result<Option<String>> {
    let meta = path.symlink_metadata()?;
    if !meta.is_file() || meta.file_type().is_symlink() { return Ok(None); }
    if meta.len() > max_bytes { return Err(io::Error::new(io::ErrorKind::FileTooLarge, "file size limit exceeded")); }
    let mut bytes = Vec::with_capacity(meta.len() as usize);
    File::open(path)?.take(max_bytes.saturating_add(1)).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes { return Err(io::Error::new(io::ErrorKind::FileTooLarge, "file grew beyond size limit")); }
    if bytes.contains(&0) { return Ok(None); }
    Ok(String::from_utf8(bytes).ok())
}

/// Prepare actual source once during indexing, not when crossing a zoom level.
/// Lexical color hints are independent of the metric classifier.
pub fn prepare(text: String) -> Document {
    let mut doc = Document::new(text);
    for index in 0..doc.lines.len() { doc.runs[index] = runs(doc.line(index)); }
    doc
}

pub fn runs(line: &str) -> Vec<TextRun> {
    let mut expanded = String::new();
    let mut col = 0;
    for ch in line.chars() {
        if ch == '\t' {
            let n = 4 - col % 4;
            expanded.extend(std::iter::repeat_n(' ', n)); col += n;
        } else { expanded.push(ch); col += 1; }
    }
    let chars: Vec<char> = expanded.chars().collect();
    let mut result: Vec<TextRun> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let start = i;
        let ink;
        if chars[i] == '/' && chars.get(i + 1) == Some(&'/') {
            i = chars.len(); ink = Ink::Comment;
        } else if matches!(chars[i], '"' | '\'') {
            let quote = chars[i]; i += 1;
            while i < chars.len() {
                if chars[i] == '\\' { i = (i + 2).min(chars.len()); }
                else if chars[i] == quote { i += 1; break; }
                else { i += 1; }
            }
            ink = Ink::String;
        } else if chars[i].is_ascii_digit() {
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || matches!(chars[i], '.' | '_')) { i += 1; }
            ink = Ink::Number;
        } else if chars[i].is_alphabetic() || chars[i] == '_' {
            i += 1;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') { i += 1; }
            let word: String = chars[start..i].iter().collect();
            ink = if matches!(word.as_str(), "fn" | "fun" | "func" | "def" | "function" | "class" | "struct" | "enum" | "impl" | "pub" | "let" | "mut" | "const" | "static" | "return" | "if" | "else" | "for" | "while" | "match" | "switch" | "case" | "import" | "from" | "use" | "mod" | "async" | "await" | "trait" | "interface" | "type" | "void" | "int" | "float" | "bool" | "true" | "false" | "null" | "None" | "self" | "this" | "new" | "export" | "public" | "private") { Ink::Keyword } else { Ink::Text };
        } else { i += 1; ink = Ink::Text; }
        let text: String = chars[start..i].iter().collect();
        if let Some(previous) = result.last_mut().filter(|run| run.ink == ink) {
            previous.text.push_str(&text);
        } else { result.push(TextRun { column: start, text, ink }); }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn retains_all_text_including_long_lines() {
        let line = format!("fn main() {{}} // {}", "x".repeat(1024));
        let result: String = runs(&line).iter().map(|r| r.text.as_str()).collect();
        assert_eq!(result, line);
    }
    #[test] fn prepares_every_line_without_zoom() {
        let doc = prepare("\tfn main() {}\n\n// done\n".into());
        assert_eq!(doc.runs.len(), 3);
        assert!(doc.runs[1].is_empty());
        assert_eq!(doc.runs[0].iter().map(|r| r.text.as_str()).collect::<String>(), "    fn main() {}");
    }
}
