use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ink { Text, Keyword, Number, String, Comment }

/// A real source-text run. Columns account for tab stops; text is never sampled.
#[derive(Debug)]
pub struct TextRun {
    pub column: usize,
    pub text: String,
    pub ink: Ink,
}

#[derive(Debug)]
pub struct Document {
    pub text: String,
    pub lines: Vec<Range<usize>>,
    pub runs: Vec<Vec<TextRun>>,
    pub max_columns: usize,
}

impl Document {
    pub fn new(text: String) -> Self {
        let mut offset = 0;
        let mut max_columns = 0;
        let mut runs = Vec::new();
        let lines = text.split_inclusive('\n').map(|line| {
            let start = offset;
            offset += line.len();
            let end = offset - usize::from(line.ends_with('\n'));
            let end = end - usize::from(end > start && text.as_bytes()[end - 1] == b'\r');
            let mut display = String::new();
            let mut column = 0;
            for ch in text[start..end].chars() {
                if ch == '\t' {
                    let n = 4 - column % 4;
                    display.extend(std::iter::repeat_n(' ', n));
                    column += n;
                } else {
                    display.push(ch);
                    column += 1;
                }
            }
            max_columns = max_columns.max(column);
            runs.push(if display.is_empty() { Vec::new() } else {
                vec![TextRun { column: 0, text: display, ink: Ink::Text }]
            });
            start..end
        }).collect();
        Self { text, lines, runs, max_columns }
    }

    pub fn line(&self, index: usize) -> &str {
        self.lines.get(index).map_or("", |range| &self.text[range.clone()])
    }

    /// Source and prepared text allocation, excluding the tree and GPU atlas.
    pub fn storage_bytes(&self) -> usize {
        self.text.capacity() + self.lines.capacity() * std::mem::size_of::<Range<usize>>()
            + self.runs.capacity() * std::mem::size_of::<Vec<TextRun>>()
            + self.runs.iter().map(|line| line.capacity() * std::mem::size_of::<TextRun>()
                + line.iter().map(|run| run.text.capacity()).sum::<usize>()).sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn no_truncation_at_240_characters() {
        let text = "x".repeat(1200);
        let doc = Document::new(text.clone());
        assert_eq!(doc.line(0), text);
        assert_eq!(doc.runs[0][0].text.len(), 1200);
        assert_eq!(doc.max_columns, 1200);
    }
    #[test] fn tabs_preserve_original_and_expand_for_display() {
        let doc = Document::new("a\tb\r\n".into());
        assert_eq!(doc.line(0), "a\tb");
        assert_eq!(doc.runs[0][0].text, "a   b");
        assert_eq!(doc.max_columns, 5);
    }
}
