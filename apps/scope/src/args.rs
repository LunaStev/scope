use scope_analysis::ScanOptions;
use std::{ffi::OsString,path::PathBuf};
pub struct Arguments{pub root:PathBuf,pub options:ScanOptions,pub scan_only:bool,pub json:bool}
pub const HELP:&str="Scope - local codebase explorer\n\nUsage: scope [OPTIONS] [PATH]\n\n  --scan               Print inventory without a window\n  --json               Print JSON inventory (implies --scan)\n  --no-ignore          Disable .gitignore/.ignore/.scopeignore rules\n  --include-build      Include excluded build/dependency directories\n  --exclude NAME       Exclude an exact basename (repeatable)\n  --max-file-mib N     File read limit, 1-1024 MiB; default 8\n  --help, -h           Show help\n\n.git/.hg/.svn and symlinks are always excluded. Only UTF-8 text is read.\nUnknown non-blank lines are unclassified, not guessed code.";
pub fn parse(args:impl IntoIterator<Item=OsString>)->Result<Option<Arguments>,String>{
    let mut out=Arguments{root:std::env::current_dir().map_err(|e|e.to_string())?,options:ScanOptions::default(),scan_only:false,json:false};
    let mut args=args.into_iter();let mut paths=0;
    while let Some(arg)=args.next(){
        match arg.to_str(){
            Some("--scan")=>out.scan_only=true,
            Some("--json")=>{out.json=true;out.scan_only=true;}
            Some("--no-ignore")=>out.options.respect_ignore=false,
            Some("--include-build")=>out.options.default_excludes=false,
            Some("--exclude")=>out.options.excludes.push(args.next().ok_or("--exclude needs a basename")?.to_string_lossy().into_owned()),
            Some("--max-file-mib")=>{
                let value=args.next().ok_or("--max-file-mib needs a positive integer")?;
                let value:u64=value.to_str().ok_or("invalid size")?.parse().map_err(|_|"invalid size")?;
                if !(1..=1024).contains(&value){return Err("--max-file-mib must be between 1 and 1024".into());}
                out.options.max_file_bytes=value*1024*1024;
            }
            Some("--help"|"-h")=>return Ok(None),
            Some("--")=>{
                out.root=args.next().ok_or("expected a path after --")?.into();paths+=1;
                if args.next().is_some(){return Err("only one root path is accepted".into());}
            }
            Some(s)if s.starts_with('-')=>return Err(format!("Unknown option: {s}")),
            _=>{out.root=arg.into();paths+=1;}
        }
    }
    if paths>1{return Err("only one root path is accepted".into());}Ok(Some(out))
}
#[cfg(test)]
mod tests{
    use super::*;
    fn argv(s:&[&str])->Vec<OsString>{s.iter().map(OsString::from).collect()}
    #[test] fn json_is_headless(){assert!(parse(argv(&["--json"])).unwrap().unwrap().scan_only);}
    #[test] fn rejects_unknown_option(){assert!(parse(argv(&["--typo"])).is_err());}
    #[test] fn rejects_two_roots(){assert!(parse(argv(&["a","b"])).is_err());}
    #[test] fn rejects_zero_size(){assert!(parse(argv(&["--max-file-mib","0"])).is_err());}
    #[test] fn help_succeeds(){assert!(parse(argv(&["-h"])).unwrap().is_none());}
}
