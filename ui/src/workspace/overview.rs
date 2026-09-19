//! A compact context strip instead of a second dashboard above the canvas.
use super::Workspace;
use makepad_widgets::*;
use ::model::format as f;
use ::layout::Box2;
use ::render::palette as p;
impl Workspace {
    pub(super) fn overview(&mut self,cx:&mut Cx2d,b:Box2) {
        self.chrome.fill(cx,b,p::panel());
        self.chrome.fill(cx,Box2::new(b.x,b.y+b.h-1.0,b.w,1.0),p::border());
        let (path,totals)=self.state.session.snapshot.as_ref().map(|s| {
            let n=&s.tree.nodes[self.state.session.selected.unwrap_or(0).min(s.tree.nodes.len()-1)];
            let path=if n.relative.as_os_str().is_empty(){n.name.clone()}else{format!("{} / {}",s.tree.nodes[0].name,n.relative.display())};
            (path,Some(s.tree.totals()))
        }).unwrap_or_else(||(self.state.session.root.display().to_string(),None));
        let reserved=if b.w>1050.0{450.0}else{270.0};
        self.chrome.text(cx,b.x+18.0,b.y+12.0,11.0,p::secondary(),&f::shorten(&path,((b.w-reserved-36.0)/7.0).max(1.0) as usize));
        if let Some(s)=totals {
            let summary=if !self.state.session.query.is_empty(){format!("{} matching files",f::count(self.state.session.matched_files as u64))}
                else if b.w>1050.0{format!("{} files   ·   {} lines   ·   {}",f::count(s.files),f::count(s.lines),f::bytes(s.bytes))}
                else{format!("{} files   ·   {} lines",f::count(s.files),f::count(s.lines))};
            self.chrome.text_right(cx,b.x+b.w-18.0,b.y+12.0,11.0,p::muted(),&summary);
        }
    }
}
