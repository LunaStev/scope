//! Immutable broad-phase index for readable-source discovery. Built with the
//! snapshot, never during a paint. A font-size bound rejects whole subtrees.
use crate::{Box2, SourceLayout};

#[derive(Clone, Copy)]
struct Item { id: usize, bounds: Box2, font: f64 }
struct Branch { bounds: Box2, font: f64, start: usize, end: usize, children: Option<(usize, usize)> }
#[derive(Default)]
pub struct SourceIndex { items: Vec<Item>, branches: Vec<Branch> }
impl SourceIndex {
    pub fn new(pages: &[Option<SourceLayout>]) -> Self {
        let items = pages.iter().enumerate().filter_map(|(id, page)| {
            let page = page.as_ref()?;
            (!page.packed.is_empty()).then_some(Item { id, bounds: page.content, font: page.font_size })
        }).collect();
        let mut out = Self { items, branches: Vec::new() };
        if !out.items.is_empty() { out.build(0, out.items.len()); }
        out
    }
    fn build(&mut self, start: usize, end: usize) -> usize {
        let mut b = self.items[start].bounds;
        let mut font = 0.0_f64;
        for item in &self.items[start..end] {
            let x = b.x.min(item.bounds.x); let y = b.y.min(item.bounds.y);
            b = Box2::new(x, y, (b.x+b.w).max(item.bounds.x+item.bounds.w)-x, (b.y+b.h).max(item.bounds.y+item.bounds.h)-y);
            font = font.max(item.font);
        }
        let id = self.branches.len();
        self.branches.push(Branch { bounds: b, font, start, end, children: None });
        if end-start > 8 {
            let middle = start+(end-start)/2;
            let coordinate = |item: &Item| if b.w >= b.h { item.bounds.x+item.bounds.w*0.5 } else { item.bounds.y+item.bounds.h*0.5 };
            self.items[start..end].select_nth_unstable_by(middle-start, |a,b| coordinate(a).total_cmp(&coordinate(b)).then(a.id.cmp(&b.id)));
            let left = self.build(start, middle); let right = self.build(middle, end);
            self.branches[id].children = Some((left, right));
        }
        id
    }
    /// Output and traversal are bounded. The complete image remains the backing
    /// for any detail omitted by this optional native foreground admission.
    pub fn query(&self, area: Box2, min_font: f64, limit: usize) -> Vec<usize> {
        let mut out = Vec::new();
        if self.branches.is_empty() || limit == 0 { return out; }
        let mut stack = vec![0]; let mut visited = 0;
        while let Some(id) = stack.pop() {
            visited += 1; if visited > 8192 { break; }
            let branch = &self.branches[id];
            if branch.font < min_font || !branch.bounds.intersects(area) { continue; }
            if let Some((left, right)) = branch.children {
                stack.push(right); stack.push(left);
            } else {
                for item in &self.items[branch.start..branch.end] {
                    if item.font >= min_font && item.bounds.intersects(area) {
                        out.push(item.id); if out.len() == limit { return out; }
                    }
                }
            }
        }
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn selects_only_intersecting_readable_pages() {
        let pages: Vec<_> = (0..100).map(|i| Some(SourceLayout::new(Box2::new(i as f64*100.0,0.0,90.0,80.0),10,20))).collect();
        let index = SourceIndex::new(&pages);
        let got = index.query(Box2::new(200.0,0.0,90.0,80.0), 0.0, 256);
        assert_eq!(got, vec![2]);
        assert!(index.query(Box2::new(0.0,0.0,10000.0,100.0), 1e9, 256).is_empty());
    }
    #[test] fn empty_and_bounded_queries() {
        assert!(SourceIndex::default().query(Box2::default(),0.0,10).is_empty());
        let pages = vec![Some(SourceLayout::new(Box2::new(0.0,0.0,100.0,80.0),1,20)); 1000];
        let index=SourceIndex::new(&pages);
        assert_eq!(index.query(Box2::new(0.0,0.0,100.0,80.0),0.0,7).len(),7);
    }
}
