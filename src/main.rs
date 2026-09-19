#[cfg(feature = "gui")]
mod app;
#[cfg(feature = "gui")]
mod view;

use scope::scanner::{scan, ScanOptions};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

#[derive(Clone)]
pub struct Arguments { pub root: PathBuf, pub options: ScanOptions, pub scan_only: bool }

pub fn arguments() -> Result<Arguments, String> {
    let mut out = Arguments { root: std::env::current_dir().map_err(|e| e.to_string())?, options: ScanOptions::default(), scan_only: false };
    let mut paths = 0;
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--scan") => out.scan_only = true,
            Some("--no-ignore") => out.options.respect_ignore = false,
            Some("--include-build") => out.options.default_excludes = false,
            Some("--exclude") => out.options.excludes.push(args.next().ok_or("--exclude needs a directory/file name")?.to_string_lossy().into_owned()),
            Some("--max-file-mib") => {
                let value = args.next().ok_or("--max-file-mib needs a positive integer")?;
                let value: u64 = value.to_str().ok_or("invalid size")?.parse().map_err(|_| "invalid size")?;
                if !(1..=1024).contains(&value) { return Err("--max-file-mib must be between 1 and 1024".into()); }
                out.options.max_file_bytes = value*1024*1024;
            }
            Some("--help" | "-h") => return Err(HELP.into()),
            Some("--") => {
                out.root = args.next().ok_or("expected a path after --")?.into(); paths += 1;
                if args.next().is_some() { return Err("only one root path is accepted".into()); }
            }
            Some(s) if s.starts_with('-') => return Err(format!("Unknown option: {s}\n{HELP}")),
            _ => { out.root = arg.into(); paths += 1; }
        }
    }
    if paths > 1 { return Err("only one root path is accepted".into()); }
    Ok(out)
}

const HELP: &str = "Scope - spatial codebase explorer\n\nUsage: scope [OPTIONS] [PATH]\n\n  --scan               Print inventory without opening a window\n  --no-ignore          Ignore .gitignore/.ignore/.scopeignore rules\n  --include-build      Include normally excluded build/dependency folders\n  --exclude NAME       Exclude a file/directory basename (repeatable)\n  --max-file-mib N     Per-file read limit, default 8 MiB\n  --help               Show this help\n\n.git/.hg/.svn and symlinks are always excluded. Files must be UTF-8 text.\nArea means non-blank lines (comments included), not semantic SLOC.";

fn main() {
    let args = match arguments() {
        Ok(args) => args,
        Err(message) => {
            let help = std::env::args_os().any(|s| s == "--help" || s == "-h");
            eprintln!("{message}"); std::process::exit(if help { 0 } else { 2 });
        }
    };
    if args.scan_only {
        match scan(&args.root, &args.options, &AtomicBool::new(false)) {
            Ok(tree) => {
                let s = tree.nodes[0].stats;
                println!("Root: {}\nFiles: {}\nLines: {}\nNon-blank lines: {}\nText bytes: {}\nSkipped binary/non-UTF8: {}\nSkipped oversized: {}\nWarnings: {}", tree.root.display(),s.files,s.lines,s.non_blank,s.bytes,tree.skipped_binary,tree.skipped_large,tree.warning_count);
                for warning in &tree.warnings { eprintln!("{warning}"); }
            }
            Err(error) => { eprintln!("Scope: {error}"); std::process::exit(1); }
        }
        return;
    }
    #[cfg(feature="gui")]
    app::app_main();
    #[cfg(not(feature="gui"))]
    { eprintln!("This build has no GUI. Use --scan or rebuild with default features."); std::process::exit(2); }
}
