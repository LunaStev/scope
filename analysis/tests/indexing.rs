use analysis::{scan,ScanOptions};
use std::{fs,sync::atomic::AtomicBool};
#[test]
fn scan_is_read_only_excludes_metadata_and_accounts_for_all_lines(){
    let root=tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join(".git")).unwrap();fs::write(root.path().join(".git/huge"),"secret").unwrap();
    fs::create_dir(root.path().join("target")).unwrap();fs::write(root.path().join("target/build.rs"),"x").unwrap();
    fs::write(root.path().join("main.rs"),"// note\nfn main() {}\n\n").unwrap();
    fs::write(root.path().join("blob.bin"),b"a\0b").unwrap();
    let mut tree=scan(root.path(),&ScanOptions::default(),&AtomicBool::new(false)).unwrap();
    let s=tree.totals();assert_eq!(s.files,1);assert_eq!(s.lines,3);assert!(s.is_consistent());
    assert_eq!(tree.report.skipped_binary,1);assert_eq!(tree.report.pruned_entries,2);
    tree.finish();assert_eq!(tree.totals(),s);
    assert!(root.path().join(".git/huge").exists());
}
#[test]
fn ignore_rules_apply_outside_a_git_repository(){
    let root=tempfile::tempdir().unwrap();fs::write(root.path().join(".scopeignore"),"secret.rs\n").unwrap();
    fs::write(root.path().join("secret.rs"),"hidden").unwrap();fs::write(root.path().join("ok.rs"),"visible").unwrap();
    let t=scan(root.path(),&ScanOptions::default(),&AtomicBool::new(false)).unwrap();
    assert!(!t.nodes.iter().any(|n|n.name=="secret.rs"));assert!(t.nodes.iter().any(|n|n.name=="ok.rs"));
}
#[test]
fn oversized_file_is_reported(){
    let root=tempfile::tempdir().unwrap();fs::write(root.path().join("large.rs"),"12345").unwrap();
    let opts=ScanOptions{max_file_bytes:4,..ScanOptions::default()};
    let t=scan(root.path(),&opts,&AtomicBool::new(false)).unwrap();assert_eq!(t.report.skipped_large,1);assert_eq!(t.totals().files,0);
}
#[test]
fn cancelled_scan_does_not_publish_partial_success(){
    let root=tempfile::tempdir().unwrap();
    assert!(scan(root.path(),&ScanOptions::default(),&AtomicBool::new(true)).is_err());
}
#[cfg(unix)]
#[test]
fn does_not_follow_symlinks(){
    let root=tempfile::tempdir().unwrap();let other=tempfile::tempdir().unwrap();fs::write(other.path().join("x.rs"),"private").unwrap();
    std::os::unix::fs::symlink(other.path().join("x.rs"),root.path().join("link.rs")).unwrap();
    let t=scan(root.path(),&ScanOptions::default(),&AtomicBool::new(false)).unwrap();assert_eq!(t.totals().files,0);assert_eq!(t.report.skipped_links,1);
}
