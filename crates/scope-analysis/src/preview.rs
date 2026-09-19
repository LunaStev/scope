use scope_core::LinePreview;

/// Bounded visual samples, not a second source-of-truth for line statistics.
pub fn sample(text: &str, lines: usize) -> Vec<LinePreview> {
    text.lines().enumerate().step_by(lines.div_ceil(128).max(1)).map(|(line,value)| {
        let mut indent = 0;
        for ch in value.chars().take_while(|ch| ch.is_whitespace()) {
            indent += if ch == '\t' { 4 - indent % 4 } else { 1 };
        }
        LinePreview { line, indent: indent.min(160), width: value.trim().chars().count().min(160), comment: value.trim_start().starts_with("//") }
    }).collect()
}
