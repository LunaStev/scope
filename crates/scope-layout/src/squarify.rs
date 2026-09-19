use crate::Box2;

/// Squarified treemap in input order. Positive finite weights are normalized
/// before summing, avoiding overflow even with extreme input magnitudes.
pub fn squarify(weights: &[f64], bounds: Box2) -> Vec<Box2> {
    if weights.is_empty() { return Vec::new(); }
    if bounds.area()<=0.0 || !bounds.area().is_finite() { return vec![Box2::default();weights.len()]; }
    let mut weights: Vec<f64> = weights.iter().map(|&w| if w.is_finite() && w>0.0 {w}else{1.0}).collect();
    let max=weights.iter().copied().fold(1.0,f64::max);
    for w in &mut weights { *w=(*w/max).max(f64::EPSILON); }
    let sum: f64=weights.iter().sum();
    let areas: Vec<f64>=weights.iter().map(|w|w/sum*bounds.area()).collect();
    let mut out=vec![Box2::default();areas.len()];
    let mut remaining=bounds;
    let mut start=0;
    while start<areas.len() {
        let side=remaining.w.min(remaining.h).max(f64::MIN_POSITIVE);
        let mut end=start+1;
        let mut row_sum=areas[start]; let mut lo=areas[start]; let mut hi=areas[start];
        let mut score=worst(row_sum,lo,hi,side);
        while end<areas.len() {
            let next_sum=row_sum+areas[end]; let next_lo=lo.min(areas[end]); let next_hi=hi.max(areas[end]);
            let next_score=worst(next_sum,next_lo,next_hi,side);
            if next_score>score { break; }
            row_sum=next_sum; lo=next_lo; hi=next_hi; score=next_score; end+=1;
        }
        if remaining.w>=remaining.h {
            let width=if end==areas.len(){remaining.w}else{(row_sum/remaining.h.max(f64::MIN_POSITIVE)).min(remaining.w)};
            let mut y=remaining.y;
            for i in start..end {
                let height=if i+1==end{(remaining.y+remaining.h-y).max(0.0)}else{areas[i]/width.max(f64::MIN_POSITIVE)};
                out[i]=Box2::new(remaining.x,y,width,height); y+=height;
            }
            remaining.x+=width; remaining.w=(remaining.w-width).max(0.0);
        } else {
            let height=if end==areas.len(){remaining.h}else{(row_sum/remaining.w.max(f64::MIN_POSITIVE)).min(remaining.h)};
            let mut x=remaining.x;
            for i in start..end {
                let width=if i+1==end{(remaining.x+remaining.w-x).max(0.0)}else{areas[i]/height.max(f64::MIN_POSITIVE)};
                out[i]=Box2::new(x,remaining.y,width,height); x+=width;
            }
            remaining.y+=height; remaining.h=(remaining.h-height).max(0.0);
        }
        start=end;
    }
    out
}
fn worst(sum:f64,lo:f64,hi:f64,side:f64)->f64 {
    if lo<=0.0 || sum<=0.0{return f64::INFINITY;}
    let square=side*side;
    ((square*hi)/(sum*sum)).max((sum*sum)/(square*lo))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn conserves_area() {
        let b=Box2::new(0.0,0.0,100.0,60.0);
        let r=squarify(&[6.0,3.0,1.0],b);
        assert!((r.iter().map(|b|b.area()).sum::<f64>()-6000.0).abs()<1e-6);
        assert!((r[0].area()/r[2].area()-6.0).abs()<1e-6);
    }
    #[test] fn invalid_weights_remain_finite() {
        for r in squarify(&[0.0,-1.0,f64::NAN,f64::MAX],Box2::new(0.0,0.0,100.0,100.0)) { assert!(r.area().is_finite()); }
    }
}
