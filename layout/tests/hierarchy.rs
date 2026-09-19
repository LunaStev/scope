use model::{AreaMetric,FileStamp,SourceShape,FileInfo,Node,Stats,Tree};
use layout::{tree_layout,hit_test};
use std::sync::Arc;
fn tree()->Tree{let mut t=Tree::new("fixture".into());for(name,lines)in[("a.rs",100),("b.rs",10),("empty.rs",0)]{let id=t.nodes.len();t.nodes.push(Node{name:name.into(),relative:name.into(),parent:Some(0),children:vec![],stats:Stats{files:1,lines,code:lines,bytes:lines*10,..Stats::default()},file:Some(FileInfo{language:"Rust".into(),classified:true,shape:Arc::new(SourceShape::from_text(&"x\n".repeat(lines as usize))),stamp:FileStamp{bytes:lines*2,modified:None}})});t.nodes[0].children.push(id);}t.finish();t}
#[test]fn all_metrics_produce_finite_visible_geometry(){let t=tree();for metric in [AreaMetric::NonBlank,AreaMetric::Lines,AreaMetric::Code,AreaMetric::Bytes]{let r=tree_layout(&t,metric);assert_eq!(r.len(),t.nodes.len());for b in r{assert!(b.area().is_finite());assert!(b.area()>0.0);}}}
#[test]fn deterministic_layout_and_deepest_hit(){let t=tree();let a=tree_layout(&t,AreaMetric::Code);assert_eq!(a,tree_layout(&t,AreaMetric::Code));let r=a[1];assert_eq!(hit_test(&t,&a,r.x+r.w/2.0,r.y+r.h/2.0),Some(1));}
