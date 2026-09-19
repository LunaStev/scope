use model::Stats;
use std::path::Path;
use tokei::{Config, LanguageType};

pub struct Measurement { pub stats: Stats, pub language: String, pub classified: bool }

/// Count physical lines first. A classifier must account for exactly those lines.
/// Unknown languages and any inconsistent embedded-language result are explicit
/// unclassified non-blank lines, never guessed SLOC.
pub fn measure(path: &Path, text: &str) -> Measurement {
    let lines = text.lines().count() as u64;
    let blanks = text.lines().filter(|line| line.trim().is_empty()).count() as u64;
    let mut stats = Stats { files: 1, lines, blanks, bytes: text.len() as u64, unclassified: lines - blanks, ..Stats::default() };
    let config = Config::default();
    // Detect from the path/known filenames; the bounded reader has already run.
    let lang = LanguageType::from_path(path, &config);
    let language = lang.map(|l| l.name().to_owned()).unwrap_or_else(|| {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext.is_empty() { "Text / unknown".into() } else { format!("Unknown (.{ext})") }
    });
    let mut classified = false;
    if let Some(lang) = lang {
        if lang != LanguageType::Text {
            let result = lang.parse_from_str(text, &config).summarise();
            if result.lines() as u64 == lines {
                stats.code = result.code as u64;
                stats.comments = result.comments as u64;
                stats.blanks = result.blanks as u64;
                stats.unclassified = 0;
                classified = true;
            }
        }
    }
    debug_assert!(stats.is_consistent());
    Measurement { stats, language, classified }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn rust_counts_comments_and_code_separately() {
        let s = measure(Path::new("main.rs"), "// note\n\nfn main() {} // inline\n").stats;
        assert_eq!((s.lines,s.code,s.comments,s.blanks,s.unclassified), (3,1,1,1,0));
        assert!(s.is_consistent());
    }
    #[test] fn quoted_delimiters_are_not_comments() {
        let s = measure(Path::new("x.rs"), "let url = \"https://example.test\";\n/* one\n two */\n").stats;
        assert_eq!((s.code,s.comments), (1,2));
    }
    #[test] fn unknown_text_is_not_fabricated_code() {
        let m = measure(Path::new("x.unrecognised_scope_extension"), "a\n\n b\n");
        assert!(!m.classified);
        assert_eq!((m.stats.code,m.stats.unclassified,m.stats.blanks), (0,2,1));
    }
    #[test] fn empty_and_crlf_are_consistent() {
        for text in ["", "\r\n", "x\r\n\r\n", "last line"] {
            let s = measure(Path::new("x.rs"), text).stats;
            assert!(s.is_consistent());
            assert_eq!(s.lines, text.lines().count() as u64);
        }
    }
    #[test] fn embedded_language_accounting_is_conservative() {
        let s = measure(Path::new("page.html"), "<script>\n// js\nlet a=1;\n</script>\n").stats;
        assert!(s.is_consistent());
        assert_eq!(s.lines, 4);
    }
}
