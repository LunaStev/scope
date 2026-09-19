use crate::{squarify,Box2,WORLD};
use scope_core::{AreaMetric,Tree};

pub fn tree_layout(tree:&Tree,metric:AreaMetric)->Vec<Box2> {
    if tree.nodes.is_empty(){return Vec::new();}
    // Minimum visibility is applied per file, THEN aggregated. A directory's
    // area therefore agrees with the effective weights of all its descendants.
    let mut weights:Vec<f64>=tree.nodes.iter().map(|n|if n.file.is_some(){metric.value(n.stats).max(1) as f64}else{0.0}).collect();
    for id in (1..tree.nodes.len()).rev(){if let Some(parent)=tree.nodes[id].parent{weights[parent]+=weights[id];}}
    let mut rects=vec![Box2::default();tree.nodes.len()]; rects[0]=WORLD;
    let mut stack=vec![0];
    while let Some(id)=stack.pop(){
        let bounds=rects[id];
        let mut children=tree.nodes[id].children.clone();
        children.sort_by(|a,b|weights[*b].total_cmp(&weights[*a]).then_with(||tree.nodes[*a].relative.cmp(&tree.nodes[*b].relative)));
        if children.is_empty(){continue;}
        let p=(bounds.w.min(bounds.h)*0.012).min(9.0);
        let mut inner=bounds.inset(p); let header=(inner.h*0.07).min(27.0); inner.y+=header; inner.h-=header;
        let values:Vec<f64>=children.iter().map(|id|weights[*id]).collect();
        for (child,r) in children.into_iter().zip(squarify(&values,inner)){
            rects[child]=r.inset((r.w.min(r.h)*0.012).min(2.0)); stack.push(child);
        }
    }
    rects
}
pub fn hit_test(tree:&Tree,rectangles:&[Box2],x:f64,y:f64)->Option<usize>{
    if !rectangles.first()?.contains(x,y){return None;}
    let mut id=0;
    loop {
        match tree.nodes.get(id)?.children.iter().copied().find(|child|rectangles.get(*child).is_some_and(|r|r.contains(x,y))){
            Some(child)=>id=child,None=>return Some(id),
        }
    }
}
