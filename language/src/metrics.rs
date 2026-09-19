use crate::{Registry, Detection, backend::Engine, wave};
use model::Stats;
use std::path::Path;
use tokei::{Config, LanguageType};
pub struct Measurement {
    pub stats: Stats,
    pub language: String,
    pub classified: bool,
    pub detection: Detection,
}
pub fn measure(path: &Path, text: &str) -> Measurement {
    classify(Registry::builtin().detect(path, text), text)
}
pub fn classify(detection: Detection, text: &str) -> Measurement {
    let lines = text.lines().count() as u64;
    let blanks = text.lines().filter(|line| line.trim().is_empty()).count() as u64;
    let mut stats = Stats { files: 1, lines, blanks, bytes: text.len() as u64, unclassified: lines-blanks, ..Stats::default() };
    let mut classified = false;
    match detection.engine {
        Engine::Wave => { stats = wave::measure(text); classified = true; }
        Engine::Tokei(lang) if lang != LanguageType::Text => {
            let result = lang.parse_from_str(text, &Config::default()).summarise();
            if result.lines() as u64 == lines {
                stats.code = result.code as u64;
                stats.comments = result.comments as u64;
                stats.blanks = result.blanks as u64;
                stats.unclassified = 0;
                classified = true;
            }
        }
        _ => {}
    }
    debug_assert!(stats.is_consistent());
    Measurement { stats, language: detection.name.clone(), classified, detection }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn code_and_comments_are_separate() {
        let s=measure(Path::new("main.rs"),"// note\n\nfn main() {} // inline\n").stats;
        assert_eq!((s.lines,s.code,s.comments,s.blanks,s.unclassified),(3,1,1,1,0));
    }
    #[test] fn quoted_delimiters_are_not_comments() {
        let s=measure(Path::new("x.rs"),"let url = \"https://example.test\";\n/* one\n two */\n").stats;
        assert_eq!((s.code,s.comments),(1,2));
    }
    #[test] fn unknown_text_is_not_guessed_code() {
        let m=measure(Path::new("x.unrecognised_scope_extension"),"a\n\n b\n");
        assert!(!m.classified);assert_eq!((m.stats.code,m.stats.unclassified,m.stats.blanks),(0,2,1));
    }
    #[test] fn empty_crlf_and_embedded_languages_keep_the_invariant() {
        for text in ["","\r\n","x\r\n\r\n","last line"] {let s=measure(Path::new("x.rs"),text).stats;assert!(s.is_consistent());assert_eq!(s.lines,text.lines().count() as u64);}
        let s=measure(Path::new("page.html"),"<script>\n// js\nlet a=1;\n</script>\n").stats;assert!(s.is_consistent());assert_eq!(s.lines,4);
    }
}
