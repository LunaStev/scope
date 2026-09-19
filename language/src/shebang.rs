/// Inspect a bounded first line. Never execute an interpreter.
pub(crate) fn interpreter(text: &str) -> Option<&str> {
    let line = text.trim_start_matches('\u{feff}').lines().next()?;
    if line.len() > 4096 { return None; }
    let mut words = line.strip_prefix("#!")?.split_whitespace();
    let first = words.next()?.rsplit('/').next()?;
    if first != "env" { return Some(first); }
    let mut skip = false;
    for word in words {
        if skip { skip = false; continue; }
        if word == "-u" || word == "--unset" { skip = true; continue; }
        if word == "-S" || word == "--split-string" || word == "--" { continue; }
        if word.starts_with('-') || word.contains('=') { continue; }
        return word.rsplit('/').next();
    }
    None
}
