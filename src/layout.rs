use crate::model::Tree;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Box2 { pub x: f64, pub y: f64, pub w: f64, pub h: f64 }
impl Box2 {
    pub const fn new(x: f64, y: f64, w: f64, h: f64) -> Self { Self { x, y, w, h } }
    pub fn area(self) -> f64 { self.w.max(0.0) * self.h.max(0.0) }
    pub fn contains(self, x: f64, y: f64) -> bool { x >= self.x && y >= self.y && x <= self.x + self.w && y <= self.y + self.h }
    pub fn intersects(self, b: Self) -> bool { self.x < b.x + b.w && b.x < self.x + self.w && self.y < b.y + b.h && b.y < self.y + self.h }
    pub fn inset(self, p: f64) -> Self { Self::new(self.x + p, self.y + p, (self.w - 2.0*p).max(0.0), (self.h - 2.0*p).max(0.0)) }
}
pub const WORLD: Box2 = Box2::new(0.0, 0.0, 4096.0, 2560.0);

/// Squarified treemap, returned in input order. Callers supply sorted weights.
pub fn squarify(weights: &[f64], bounds: Box2) -> Vec<Box2> {
    if weights.is_empty() { return Vec::new(); }
    let weights: Vec<f64> = weights.iter().map(|w| if w.is_finite() && *w > 0.0 { *w } else { 1.0 }).collect();
    let sum: f64 = weights.iter().sum();
    let areas: Vec<f64> = weights.iter().map(|w| w / sum * bounds.area()).collect();
    let mut out = vec![Box2::default(); areas.len()];
    let mut remaining = bounds;
    let mut start = 0;
    while start < areas.len() {
        let side = remaining.w.min(remaining.h).max(f64::MIN_POSITIVE);
        let mut end = start + 1;
        let mut row_sum = areas[start];
        let mut lo = areas[start];
        let mut hi = areas[start];
        let mut score = worst(row_sum, lo, hi, side);
        while end < areas.len() {
            let next_sum = row_sum + areas[end];
            let next_lo = lo.min(areas[end]);
            let next_hi = hi.max(areas[end]);
            let next_score = worst(next_sum, next_lo, next_hi, side);
            if next_score > score { break; }
            row_sum = next_sum; lo = next_lo; hi = next_hi; score = next_score; end += 1;
        }
        if remaining.w >= remaining.h {
            let width = if end == areas.len() { remaining.w } else { (row_sum / remaining.h.max(f64::MIN_POSITIVE)).min(remaining.w) };
            let mut y = remaining.y;
            for i in start..end {
                let height = if i + 1 == end { (remaining.y + remaining.h - y).max(0.0) } else { areas[i] / width.max(f64::MIN_POSITIVE) };
                out[i] = Box2::new(remaining.x, y, width, height); y += height;
            }
            remaining.x += width; remaining.w = (remaining.w - width).max(0.0);
        } else {
            let height = if end == areas.len() { remaining.h } else { (row_sum / remaining.w.max(f64::MIN_POSITIVE)).min(remaining.h) };
            let mut x = remaining.x;
            for i in start..end {
                let width = if i + 1 == end { (remaining.x + remaining.w - x).max(0.0) } else { areas[i] / height.max(f64::MIN_POSITIVE) };
                out[i] = Box2::new(x, remaining.y, width, height); x += width;
            }
            remaining.y += height; remaining.h = (remaining.h - height).max(0.0);
        }
        start = end;
    }
    out
}
fn worst(sum: f64, lo: f64, hi: f64, side: f64) -> f64 {
    if lo <= 0.0 || sum <= 0.0 { return f64::INFINITY; }
    let square = side * side;
    ((square * hi) / (sum * sum)).max((sum * sum) / (square * lo))
}

pub fn tree_layout(tree: &Tree) -> Vec<Box2> {
    let mut rectangles = vec![Box2::default(); tree.nodes.len()]; rectangles[0] = WORLD;
    let mut stack = vec![0];
    while let Some(id) = stack.pop() {
        let bounds = rectangles[id];
        let mut children = tree.nodes[id].children.clone();
        children.sort_by(|a,b| tree.nodes[*b].weight().total_cmp(&tree.nodes[*a].weight()).then_with(|| tree.nodes[*a].relative.cmp(&tree.nodes[*b].relative)));
        if children.is_empty() { continue; }
        let p = (bounds.w.min(bounds.h) * 0.012).min(9.0);
        let mut inner = bounds.inset(p);
        let header = (inner.h * 0.07).min(27.0); inner.y += header; inner.h -= header;
        let weights: Vec<f64> = children.iter().map(|id| tree.nodes[*id].weight()).collect();
        for (child, rect) in children.into_iter().zip(squarify(&weights, inner)) {
            rectangles[child] = rect.inset((rect.w.min(rect.h) * 0.012).min(2.0)); stack.push(child);
        }
    }
    rectangles
}

pub fn hit_test(tree: &Tree, rectangles: &[Box2], x: f64, y: f64) -> Option<usize> {
    if !rectangles.first()?.contains(x, y) { return None; }
    let mut id = 0;
    loop {
        match tree.nodes[id].children.iter().copied().find(|child| rectangles[*child].contains(x, y)) {
            Some(child) => id = child,
            None => return Some(id),
        }
    }
}
