use std::fs::File;
use std::io::{self, Read};
use std::ops::Range;
use std::path::Path;

/// Read only; bounded even if a file grows after stat().
pub fn read_text(path: &Path, max_bytes: u64) -> io::Result<Option<String>> {
    let meta = path.symlink_metadata()?;
    if !meta.is_file() || meta.file_type().is_symlink() { return Ok(None); }
    if meta.len() > max_bytes {
        return Err(io::Error::new(io::ErrorKind::FileTooLarge, "file size limit exceeded"));
    }
    let mut bytes = Vec::with_capacity(meta.len() as usize);
    File::open(path)?.take(max_bytes.saturating_add(1)).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        return Err(io::Error::new(io::ErrorKind::FileTooLarge, "file grew beyond size limit"));
    }
    if bytes.contains(&0) { return Ok(None); }
    Ok(String::from_utf8(bytes).ok())
}

/// Display label only; unknown UTF-8 files work without a language adapter.
pub fn language(path: &Path) -> String {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase().as_str() {
        "rs" => "Rust".into(),
        "c" | "h" => "C".into(),
        "cc" | "cpp" | "cxx" | "hpp" | "hxx" => "C++".into(),
        "py" | "pyi" => "Python".into(),
        "js" | "jsx" | "mjs" | "cjs" => "JavaScript".into(),
        "ts" | "tsx" => "TypeScript".into(),
        "go" => "Go".into(),
        "java" => "Java".into(),
        "kt" | "kts" => "Kotlin".into(),
        "cs" => "C#".into(),
        "rb" => "Ruby".into(),
        "swift" => "Swift".into(),
        "zig" => "Zig".into(),
        "sh" | "bash" | "zsh" => "Shell".into(),
        "md" | "mdx" => "Markdown".into(),
        "json" => "JSON".into(),
        "toml" => "TOML".into(),
        "yml" | "yaml" => "YAML".into(),
        "html" | "htm" => "HTML".into(),
        "css" | "scss" => "CSS".into(),
        "" => "Text".into(),
        extension => extension.to_owned(),
    }
}

#[derive(Debug)]
pub struct Document { pub text: String, pub lines: Vec<Range<usize>> }
impl Document {
    pub fn new(text: String) -> Self {
        let mut offset = 0;
        let lines = text.split_inclusive('\n').map(|line| {
            let start = offset;
            offset += line.len();
            let end = offset - usize::from(line.ends_with('\n'));
            let end = end - usize::from(text.as_bytes().get(end.wrapping_sub(1)) == Some(&b'\r'));
            start..end.max(start)
        }).collect();
        Self { text, lines }
    }
    pub fn line(&self, index: usize) -> &str {
        self.lines.get(index).map_or("", |r| &self.text[r.clone()])
    }
    pub fn storage_bytes(&self) -> usize {
        self.text.capacity() + self.lines.capacity() * std::mem::size_of::<Range<usize>>()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ink { Text, Keyword, Number, String, Comment }

/// Language-neutral, single-line lexical hints; deliberately not an AST lexer.
pub fn runs(line: &str) -> Vec<(usize, String, Ink)> {
    let chars: Vec<char> = line.chars().take(240).collect();
    let mut result = Vec::new();
    let mut i = 0;
    let mut column = 0;
    while i < chars.len() {
        if chars[i].is_whitespace() {
            column += if chars[i] == '\t' { 4 - column % 4 } else { 1 };
            i += 1; continue;
        }
        let start = i;
        let ink;
        if (chars[i] == '/' && chars.get(i + 1) == Some(&'/')) || chars[i] == '#' {
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
        result.push((column, chars[start..i].iter().collect(), ink));
        column += i - start;
    }
    result
}
