use scope::camera::Camera;
use scope::layout::{squarify, tree_layout, hit_test, Box2, WORLD};
use scope::scanner::{scan, ScanOptions};
use scope::source::{Document, runs};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path=std::env::temp_dir().join(format!("scope-test-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
        std::fs::create_dir_all(&path).unwrap(); Self(path)
    }
    fn file(&self,path:&str,bytes:&[u8]) {
        let p=self.0.join(path); std::fs::create_dir_all(p.parent().unwrap()).unwrap(); std::fs::write(p,bytes).unwrap();
    }
}
impl Drop for Fixture {fn drop(&mut self) {let _=std::fs::remove_dir_all(&self.0);}}

#[test]
fn scans_any_text_language_and_excludes_vcs_and_builds() {
    let f=Fixture::new();
    f.file("src/a.rs",b"fn main() {\n\n}\n");
    f.file("custom/program.unknown",b"hello\nworld\n");
    f.file(".git/objects/pack/large.pack",b"never scan this");
    f.file("target/generated.rs",b"ignored");
    f.file("image.bin",b"\x00\xff\x00");
    f.file("empty.txt",b"");
    let tree=scan(&f.0,&ScanOptions::default(),&AtomicBool::new(false)).unwrap();
    let s=tree.nodes[0].stats;
    assert_eq!(s.files,3); assert_eq!(s.lines,5); assert_eq!(s.non_blank,4);
    assert_eq!(tree.skipped_binary,1);
    assert!(tree.nodes.iter().all(|n| !n.relative.starts_with(".git")));
    assert!(tree.nodes.iter().all(|n| !n.relative.starts_with("target")));
    assert!(tree.nodes.iter().any(|n| n.file.as_ref().is_some_and(|f| f.language=="unknown")));
}

#[test]
fn honors_scopeignore_and_gitignore_without_git_directory() {
    let f=Fixture::new(); f.file(".scopeignore",b"hidden.txt\n"); f.file(".gitignore",b"*.skip\n");
    f.file("hidden.txt",b"ignore me"); f.file("other.skip",b"ignore me too"); f.file("keep.c",b"int main() {}\n");
    let tree=scan(&f.0,&ScanOptions::default(),&AtomicBool::new(false)).unwrap();
    assert!(!tree.nodes.iter().any(|n| n.name=="hidden.txt" || n.name=="other.skip"));
    let options=ScanOptions {respect_ignore:false,..ScanOptions::default()};
    let tree=scan(&f.0,&options,&AtomicBool::new(false)).unwrap();
    assert!(tree.nodes.iter().any(|n| n.name=="hidden.txt"));
}

#[test]
fn supports_build_opt_in_and_explicit_exclusion() {
    let f=Fixture::new(); f.file("build/a.txt",b"a"); f.file("custom/b.txt",b"b");
    let options=ScanOptions {default_excludes:false,excludes:vec!["custom".into()],..ScanOptions::default()};
    let tree=scan(&f.0,&options,&AtomicBool::new(false)).unwrap();
    assert_eq!(tree.nodes[0].stats.files,1);
    assert!(tree.nodes.iter().any(|n| n.name=="a.txt"));
}

#[test]
fn oversized_and_cancelled_scans_are_explicit() {
    let f=Fixture::new(); f.file("large.txt",b"123456789");
    let options=ScanOptions {max_file_bytes:4,..ScanOptions::default()};
    let tree=scan(&f.0,&options,&AtomicBool::new(false)).unwrap(); assert_eq!(tree.skipped_large,1);
    assert!(scan(&f.0,&options,&AtomicBool::new(true)).is_err());
    assert!(scan(&f.0.join("missing"),&options,&AtomicBool::new(false)).is_err());
}

#[cfg(unix)]
#[test]
fn skips_symlinks_even_when_they_point_outside_the_root() {
    let f=Fixture::new(); let outside=Fixture::new(); outside.file("private.txt",b"outside");
    f.file("a.txt",b"inside");
    std::os::unix::fs::symlink(&outside.0,f.0.join("escape")).unwrap();
    std::os::unix::fs::symlink(outside.0.join("private.txt"),f.0.join("external.txt")).unwrap();
    let tree=scan(&f.0,&ScanOptions::default(),&AtomicBool::new(false)).unwrap(); assert_eq!(tree.nodes[0].stats.files,1);
}

#[test]
fn rectangles_conserve_area_and_do_not_overlap() {
    for count in [1,2,7,100,1000] {
        let weights:Vec<f64>=(1..=count).rev().map(|i| (i*i) as f64).collect();
        let bounds=Box2::new(7.0,11.0,1200.0,700.0); let r=squarify(&weights,bounds);
        let total:f64=weights.iter().sum();
        assert!((r.iter().map(|b| b.area()).sum::<f64>()-bounds.area()).abs()<0.001);
        for (i,b) in r.iter().enumerate() {
            assert!(b.x>=bounds.x-0.00001 && b.y>=bounds.y-0.00001);
            assert!(b.x+b.w<=bounds.x+bounds.w+0.00001 && b.y+b.h<=bounds.y+bounds.h+0.00001);
            assert!((b.area()/bounds.area()-weights[i]/total).abs()<0.00001);
            for c in &r[i+1..] { assert!(!b.inset(0.00001).intersects(c.inset(0.00001))); }
        }
    }
}

#[test]
fn treemap_handles_extreme_ratios_and_empty_input() {
    assert!(squarify(&[],WORLD).is_empty());
    for b in squarify(&[1e12,1.0,1.0],WORLD) { assert!(b.w.is_finite() && b.h.is_finite()); }
    for b in squarify(&[0.0,-2.0,f64::NAN],WORLD) { assert!(b.w.is_finite() && b.h.is_finite()); }
}

#[test]
fn hierarchy_bounds_and_hit_testing() {
    let f=Fixture::new(); f.file("src/a.rs",b"a\nb\n"); f.file("src/b.rs",b"b"); f.file("docs/a.md",b"notes");
    let tree=scan(&f.0,&ScanOptions::default(),&AtomicBool::new(false)).unwrap(); let r=tree_layout(&tree);
    for (id,n) in tree.nodes.iter().enumerate() {
        if let Some(p)=n.parent { assert!(r[p].contains(r[id].x,r[id].y)); }
        if n.file.is_some() { assert_eq!(hit_test(&tree,&r,r[id].x+r[id].w*0.5,r[id].y+r[id].h*0.5),Some(id)); }
    }
    assert_eq!(hit_test(&tree,&r,-1.0,-1.0),None);
}

#[test]
fn zoom_preserves_the_world_point_under_the_cursor() {
    let mut camera=Camera::default(); camera.fit(WORLD,Box2::new(10.0,20.0,1200.0,800.0));
    let before=camera.unproject(440.0,310.0); camera.zoom_at(2.6,440.0,310.0); let after=camera.unproject(440.0,310.0);
    assert!((before.0-after.0).abs()<1e-9 && (before.1-after.1).abs()<1e-9);
}

#[test]
fn document_handles_crlf_unicode_and_no_trailing_newline() {
    let document=Document::new("fn 시작() {\r\n\r\n  hello(\"🌊\");\n}".into());
    assert_eq!(document.lines.len(),4); assert_eq!(document.line(0),"fn 시작() {"); assert_eq!(document.line(1),""); assert_eq!(document.line(3),"}");
    assert_eq!(Document::new(String::new()).lines.len(),0);
    assert!(!runs("let 이름 = \"🌊\"; // hello").is_empty());
}
