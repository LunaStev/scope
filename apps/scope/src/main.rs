mod args;
mod report;
use scope_analysis::scan;
use std::sync::atomic::AtomicBool;
fn main(){
    let args=match args::parse(std::env::args_os().skip(1)){
        Ok(Some(args))=>args,Ok(None)=>{println!("{}",args::HELP);return;}
        Err(error)=>{eprintln!("Scope: {error}");std::process::exit(2);}
    };
    if args.scan_only{
        match scan(&args.root,&args.options,&AtomicBool::new(false)){
            Ok(tree)=>report::print(&tree,args.json),
            Err(error)=>{eprintln!("Scope: {error}");std::process::exit(1);}
        }
        return;
    }
    #[cfg(feature="gui")]
    scope_ui::run(args.root,args.options);
    #[cfg(not(feature="gui"))]
    {eprintln!("This build has no GUI. Use --scan or rebuild with default features.");std::process::exit(2);}
}
