use super::Workspace;
use makepad_widgets::*;
use ::layout::Box2;
use ::model::format as f;
use ::render::palette as p;
impl Workspace {
    pub(super) fn summary(&mut self,cx:&mut Cx2d,b:Box2) {
        self.chrome.fill(cx,b,p::panel());
        self.chrome.fill(cx,Box2::new(b.x,b.y+b.h-1.0,b.w,1.0),p::border());
        let text=if let Some(s)=self.state.session.snapshot.as_ref(){
            let t=s.tree.totals();format!("{} files    {} lines    {} code    {}",f::count(t.files),f::count(t.lines),f::count(t.code),f::bytes(t.bytes))
        }else{self.state.session.status.clone()};
        self.chrome.text(cx,b.x+14.0,b.y+9.0,10.0,p::secondary(),&f::shorten(&text,((b.w-150.0)/7.0).max(1.0) as usize));
        self.chrome.text_right(cx,b.x+b.w-14.0,b.y+9.0,9.0,p::muted(),if self.state.session.busy(){"Indexing"}else{"Local / read-only"});
    }
}
