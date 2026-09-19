use crate::options::{ScanOptions,DEFAULT_EXCLUDES};
use ignore::WalkBuilder;
use std::{path::Path,sync::{Arc,atomic::{AtomicU64,Ordering}}};
pub(crate) fn builder(root:&Path,options:&ScanOptions,pruned:Arc<AtomicU64>)->WalkBuilder{
    let mut b=WalkBuilder::new(root);let rules=options.clone();
    b.hidden(false).follow_links(false).parents(false).git_ignore(options.respect_ignore).git_exclude(options.respect_ignore)
        .git_global(false).ignore(options.respect_ignore).require_git(false).threads(options.worker_count());
    if options.respect_ignore{b.add_custom_ignore_filename(".scopeignore");}
    b.filter_entry(move|e|{
        if e.depth()==0{return true;}let name=e.file_name().to_string_lossy();
        let skip=matches!(name.as_ref(),".git"|".hg"|".svn")||rules.excludes.iter().any(|n|n==name.as_ref())
            ||(rules.default_excludes&&e.file_type().is_some_and(|t|t.is_dir())&&DEFAULT_EXCLUDES.contains(&name.as_ref()));
        if skip{pruned.fetch_add(1,Ordering::Relaxed);}!skip
    });b
}
