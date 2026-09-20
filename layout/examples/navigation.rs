//! Matched hot-path microbenchmark, not a whole-application FPS claim.
use layout::{Box2, hit_test, navigation::{NodeIndex,QueryKind}};
use model::{Node,Tree};
use std::{hint::black_box,time::Instant};
fn stats(values:&mut [f64])->String {
    values.sort_by(f64::total_cmp);let n=values.len();
    format!("{{\"samples\":{n},\"median_us\":{:.3},\"p95_us\":{:.3},\"max_us\":{:.3}}}",values[n/2],values[(n*95).div_ceil(100)-1],values[n-1])
}
fn main() {
    let side=500usize;let total=side*side;
    let mut tree=Tree::new("synthetic-wide-directory".into());let world=Box2::new(0.0,0.0,5000.0,5020.0);let mut rects=vec![world];
    for i in 0..total {
        let id=tree.nodes.len();tree.nodes.push(Node::directory(format!("f{i}"),format!("f{i}").into(),Some(0)));tree.nodes[0].children.push(id);
        rects.push(Box2::new((i%side) as f64*10.0+1.0,(i/side) as f64*10.0+11.0,8.0,8.0));
    }
    let begin=Instant::now();let index=NodeIndex::new(&tree,&rects);let build_ms=begin.elapsed().as_secs_f64()*1000.0;
    let mut old=Vec::new();let mut new=Vec::new();let mut old_region=Vec::new();let mut new_region=Vec::new();let mut max_visits=0;let mut max_region_visits=0;
    for sample in 0..528usize {
        let id=(sample*104729)%total+1;let b=rects[id];let(x,y)=(b.x+4.0,b.y+4.0);
        let start=Instant::now();let a=black_box(hit_test(black_box(&tree),black_box(&rects),x,y));let old_us=start.elapsed().as_secs_f64()*1e6;
        let start=Instant::now();let(c,visits)=black_box(index.hit_with_visits(x,y));let new_us=start.elapsed().as_secs_f64()*1e6;
        assert_eq!(a,c);assert_eq!(c,Some(id));max_visits=max_visits.max(visits);
        let area=Box2::new(b.x,b.y,30.0,30.0);
        let start=Instant::now();let mut reference:Vec<_>=rects.iter().enumerate().filter_map(|(id,&r)|r.intersects(area).then_some(id)).collect();let old_r=start.elapsed().as_secs_f64()*1e6;
        let start=Instant::now();let mut cursor=index.cursor(area,QueryKind::Nodes);let mut budget=usize::MAX;let mut result=Vec::new();
        while let Some(id)=cursor.next(&index,&mut budget){result.push(id);}
        let new_r=start.elapsed().as_secs_f64()*1e6;max_region_visits=max_region_visits.max(usize::MAX-budget);
        reference.sort_unstable();result.sort_unstable();assert_eq!(result,reference);
        if sample>=16{old.push(old_us);new.push(new_us);old_region.push(old_r);new_region.push(new_r);}
    }
    assert!(max_visits<512);assert!(max_region_visits<1024);
    println!("{{\"fixture\":{{\"nodes\":{},\"sibling_nodes\":{total}}},\"scope\":\"same-process release BVH versus previous linear point/region traversal; not GPU time, file indexing, or FPS\",\"index_build_ms\":{build_ms:.3},\"index_accounted_bytes\":{},\"point_before\":{},\"point_after\":{},\"region_before\":{},\"region_after\":{},\"max_point_visits\":{max_visits},\"max_region_visits\":{max_region_visits}}}",rects.len(),index.storage_bytes(),stats(&mut old),stats(&mut new),stats(&mut old_region),stats(&mut new_region));
}
