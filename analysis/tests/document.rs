use analysis::source::{Document,read_text};
#[test]fn document_preserves_crlf_and_utf8_line_boundaries(){
    let doc=Document::new("안녕\r\nworld\n".into());assert_eq!(doc.lines.len(),2);assert_eq!(doc.line(0),"안녕");assert_eq!(doc.line(1),"world");assert_eq!(doc.line(2),"");
}
#[test]fn empty_document_has_no_phantom_line(){assert!(Document::new(String::new()).lines.is_empty());}
#[test]fn bounded_reader_rejects_oversized_input(){
    let root=tempfile::tempdir().unwrap();let path=root.path().join("x");std::fs::write(&path,b"12345").unwrap();
    assert!(read_text(&path,4).is_err());assert_eq!(read_text(&path,5).unwrap().unwrap(),"12345");
}
