//! Immutable node BVH shared by picking, annotations and regional rasterization.
//! Construction runs with the snapshot; queries never walk every sibling in a
//! large directory. Query cursors can yield without losing their position.
use crate::{node_frame, Box2};
use model::Tree;

#[derive(Clone, Copy)]
struct Item { id: usize, bounds: Box2, header: Box2, depth: usize }
struct Branch {
    bounds: Box2, header_height: f64, start: usize, end: usize,
    children: Option<(usize, usize)>,
}
#[derive(Default)]
pub struct NodeIndex { items: Vec<Item>, branches: Vec<Branch>, order: Vec<usize> }
#[derive(Clone, Copy, Debug)]
pub enum QueryKind { Nodes, Headers { min_height: f64 } }
#[derive(Default)]
pub struct NodeCursor {
    stack: Vec<usize>, leaf: Option<(usize, usize)>, area: Box2,
    min_header: Option<f64>,
}
fn union(a: Box2, b: Box2) -> Box2 {
    let x = a.x.min(b.x); let y = a.y.min(b.y);
    Box2::new(x,y,(a.x+a.w).max(b.x+b.w)-x,(a.y+a.h).max(b.y+b.h)-y)
}
impl NodeIndex {
    pub fn new(tree: &Tree, rectangles: &[Box2]) -> Self {
        let mut depths = vec![0; tree.nodes.len()];
        let mut order = vec![0; tree.nodes.len()];
        let mut rank = 0;
        let mut stack = if tree.nodes.is_empty() { Vec::new() } else { vec![(0,0)] };
        // Preserve the original parent-before-child and sibling paint order.
        while let Some((id,depth)) = stack.pop() {
            depths[id] = depth; order[id] = rank; rank += 1;
            stack.extend(tree.nodes[id].children.iter().rev().map(|&c|(c,depth+1)));
        }
        let items = rectangles.iter().copied().enumerate().filter_map(|(id,b)| {
            (id<depths.len() && b.w>0.0 && b.h>0.0 && [b.x,b.y,b.w,b.h].iter().all(|x|x.is_finite()))
                .then(||Item { id, bounds:b, header:node_frame(b).header, depth:depths[id] })
        }).collect();
        let mut out = Self { items, branches:Vec::new(), order };
        if !out.items.is_empty() { out.build(0,out.items.len()); }
        out
    }
    fn build(&mut self, start:usize, end:usize) -> usize {
        let mut bounds = self.items[start].bounds; let mut height = 0.0_f64;
        for i in &self.items[start..end] { bounds=union(bounds,i.bounds); height=height.max(i.header.h); }
        let id = self.branches.len();
        self.branches.push(Branch { bounds,header_height:height,start,end,children:None });
        if end-start>8 {
            let mid=start+(end-start)/2;
            let axis=|i:&Item| if bounds.w>=bounds.h { i.bounds.x+i.bounds.w*0.5 } else { i.bounds.y+i.bounds.h*0.5 };
            self.items[start..end].select_nth_unstable_by(mid-start,|a,b|axis(a).total_cmp(&axis(b)).then(a.id.cmp(&b.id)));
            let a=self.build(start,mid);let b=self.build(mid,end);self.branches[id].children=Some((a,b));
        }
        id
    }
    pub fn cursor(&self, area:Box2, kind:QueryKind) -> NodeCursor {
        let min_header=match kind { QueryKind::Nodes=>None, QueryKind::Headers{min_height}=>Some(min_height) };
        NodeCursor { stack:if self.branches.is_empty()||area.w<=0.0||area.h<=0.0 {Vec::new()} else {vec![0]}, leaf:None,area,min_header }
    }
    /// All intersecting nodes, with exact original painter order. No detail cap:
    /// this query is used by the background rasterizer, including overview.
    pub fn intersections(&self, area:Box2) -> Vec<usize> {
        let mut cursor=self.cursor(area,QueryKind::Nodes);let mut remaining=usize::MAX;let mut ids=Vec::new();
        while let Some(id)=cursor.next(self,&mut remaining) { ids.push(id); }
        ids.sort_unstable_by_key(|&id|self.order[id]);ids
    }
    pub fn hit(&self,x:f64,y:f64)->Option<usize> { self.hit_with_visits(x,y).0 }
    pub fn hit_with_visits(&self,x:f64,y:f64)->(Option<usize>,usize) {
        if self.branches.is_empty()||!x.is_finite()||!y.is_finite() { return (None,0); }
        // A balanced binary tree over a usize-indexed vector has <=usize::BITS
        // levels. DFS therefore needs at most this many pending siblings.
        let mut stack=[0usize;128];let mut len=1;let mut visited=0;let mut best:Option<Item>=None;
        while len>0 {
            len-=1;let branch=&self.branches[stack[len]];visited+=1;
            if !branch.bounds.contains(x,y) {continue;}
            if let Some((a,b))=branch.children {stack[len]=b;stack[len+1]=a;len+=2;}
            else {for &item in &self.items[branch.start..branch.end] {
                visited+=1;
                if item.bounds.contains(x,y) && best.is_none_or(|old|item.depth>old.depth || (item.depth==old.depth&&self.order[item.id]<self.order[old.id])) {best=Some(item);}
            }}
        }
        (best.map(|i|i.id),visited)
    }
    pub fn storage_bytes(&self)->usize {
        self.items.capacity()*std::mem::size_of::<Item>()+self.branches.capacity()*std::mem::size_of::<Branch>()+self.order.capacity()*std::mem::size_of::<usize>()
    }
}
impl NodeCursor {
    pub fn is_done(&self)->bool { self.stack.is_empty()&&self.leaf.is_none() }
    /// A None return with !is_done() means the caller's work allowance expired.
    /// Both branch and leaf visits consume allowance, including rejected items.
    pub fn next(&mut self,index:&NodeIndex,remaining:&mut usize)->Option<usize> {
        while *remaining>0 {
            if let Some((at,end))=self.leaf {
                if at==end {self.leaf=None;continue;}
                *remaining-=1;self.leaf=Some((at+1,end));let item=index.items[at];
                let b=if let Some(height)=self.min_header {if item.header.h<height {continue;}item.header} else {item.bounds};
                if b.intersects(self.area) {return Some(item.id);}
            } else {
                let id=self.stack.pop()?;*remaining-=1;let b=&index.branches[id];
                if !b.bounds.intersects(self.area)||self.min_header.is_some_and(|h|b.header_height<h) {continue;}
                if let Some((a,b))=b.children {self.stack.push(b);self.stack.push(a);}else{self.leaf=Some((b.start,b.end));}
            }
        }
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(n:usize)->(Tree,Vec<Box2>) {
        let mut t=Tree::new("fixture".into());let mut r=vec![Box2::new(0.0,0.0,n as f64*10.0,100.0)];
        for i in 0..n {let id=t.nodes.len();t.nodes.push(model::Node::directory(i.to_string(),i.to_string().into(),Some(0)));t.nodes[0].children.push(id);r.push(Box2::new(i as f64*10.0+1.0,10.0,8.0,80.0));}
        (t,r)
    }
    #[test]fn indexed_picking_matches_reference_including_gaps() {
        let(t,r)=fixture(2000);let index=NodeIndex::new(&t,&r);
        for i in 0..4000 {let x=i as f64*5.0;assert_eq!(index.hit(x,25.0),crate::hit_test(&t,&r,x,25.0));}
        assert_eq!(index.hit(-1.0,0.0),None);assert_eq!(index.hit(f64::NAN,0.0),None);
    }
    #[test]fn point_lookup_does_not_walk_a_large_sibling_list() {
        let(t,r)=fixture(100_000);let index=NodeIndex::new(&t,&r);let(id,visits)=index.hit_with_visits(999_995.0,50.0);
        assert_eq!(id,Some(100_000));assert!(visits<256,"{visits}");
    }
    #[test]fn query_resumes_with_one_visit_without_omission() {
        let(t,r)=fixture(200);let index=NodeIndex::new(&t,&r);let mut c=index.cursor(r[0],QueryKind::Nodes);let mut result=Vec::new();
        while !c.is_done() {let mut budget=1;if let Some(id)=c.next(&index,&mut budget){result.push(id);}}
        result.sort_unstable();assert_eq!(result,(0..201).collect::<Vec<_>>());
    }
    #[test]fn headers_reject_text_content_and_small_names() {
        let(t,r)=fixture(100);let index=NodeIndex::new(&t,&r);
        let mut c=index.cursor(Box2::new(1.0,30.0,8.0,10.0),QueryKind::Headers{min_height:0.0});let mut budget=1000;assert_eq!(c.next(&index,&mut budget),None);
        let mut c=index.cursor(r[0],QueryKind::Headers{min_height:1000.0});let mut budget=1000;assert_eq!(c.next(&index,&mut budget),None);assert!(c.is_done());
    }
    #[test]fn region_coverage_keeps_parent_then_child_order() {
        let(mut t,mut r)=fixture(3);let id=t.nodes.len();t.nodes.push(model::Node::directory("nested".into(),"0/nested".into(),Some(1)));t.nodes[1].children.push(id);r.push(Box2::new(2.0,12.0,3.0,3.0));
        let index=NodeIndex::new(&t,&r);assert_eq!(index.intersections(r[0]),vec![0,1,4,2,3]);assert_eq!(index.hit(3.0,13.0),Some(4));
    }
}
