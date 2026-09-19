use makepad_widgets::{Rect,dvec2};
use scope_layout::Box2;
pub fn rect(b:Box2)->Rect{Rect{pos:dvec2(b.x,b.y),size:dvec2(b.w,b.h)}}
pub fn code_geometry(b:Box2,lines:usize)->(usize,usize,Box2,f64){
    let title=if b.h>35.0{19.0}else{b.h*0.15};
    let inner=b.inset(4.0);
    let content=Box2::new(inner.x,inner.y+title,inner.w.max(0.1),(inner.h-title).max(0.1));
    let columns=((content.w/content.h*lines.max(1) as f64/72.0).sqrt().round() as usize).clamp(1,128).min(lines.max(1));
    let rows=lines.max(1).div_ceil(columns);
    let font=(content.h/rows as f64*0.54).min(content.w/columns as f64/90.0/0.81);
    (columns,rows,content,font)
}
