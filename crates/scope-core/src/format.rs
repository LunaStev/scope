//! Exact counts and explicit units; never turn a small repository into `0.0 GiB`.
pub fn count(value: u64) -> String {
    let value = value.to_string();
    let mut out = String::with_capacity(value.len() + value.len() / 3);
    for (index, ch) in value.chars().enumerate() {
        if index > 0 && (value.len() - index) % 3 == 0 { out.push(','); }
        out.push(ch);
    }
    out
}
pub fn bytes(value: u64) -> String {
    if value < 1024 { return format!("{} B", count(value)); }
    let units = ["KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    let mut n = value as f64 / 1024.0;
    let mut i = 0;
    while n >= 1024.0 && i + 1 < units.len() { n /= 1024.0; i += 1; }
    format!("{n:.2} {}", units[i])
}
pub fn percent(part: u64, total: u64) -> String {
    if total == 0 { return "—".into(); }
    format!("{:.1}%", part as f64 / total as f64 * 100.0)
}
pub fn duration(ms: u64) -> String {
    if ms < 1000 { format!("{ms} ms") } else { format!("{:.2} s", ms as f64 / 1000.0) }
}
pub fn shorten(text: &str, max: usize) -> String {
    if text.chars().count() <= max { text.into() }
    else if max < 2 { String::new() }
    else { format!("{}…", text.chars().take(max - 1).collect::<String>()) }
}
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    text.chars().collect::<Vec<_>>().chunks(width.max(1)).map(|s| s.iter().collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn exact_counts() { assert_eq!(count(0), "0"); assert_eq!(count(1234567), "1,234,567"); }
    #[test] fn adaptive_units() { assert_eq!(bytes(0), "0 B"); assert_eq!(bytes(512), "512 B"); assert_eq!(bytes(1536), "1.50 KiB"); assert_eq!(bytes(1048576), "1.00 MiB"); }
    #[test] fn empty_denominator_is_not_zero_percent() { assert_eq!(percent(0,0), "—"); assert_eq!(percent(1,4), "25.0%"); }
    #[test] fn unicode_truncation() { assert_eq!(shorten("안녕하세요", 3), "안녕…"); }
}
