use super::Workspace;
use makepad_widgets::*;
use ::model::format as f;
use ::layout::Box2;
use ::render::palette as p;

fn clip(cx:&mut Cx2d,b:Box2) {
    cx.begin_turtle(Walk {abs_pos:Some(dvec2(b.x,b.y)),width:Size::Fixed(b.w),height:Size::Fixed(b.h),..Walk::default()},Layout {clip_x:true,clip_y:true,..Layout::default()});
}
impl Workspace {
    pub(super) fn frame(&mut self,cx:&mut Cx2d,walk:Walk)->DrawStep {
        self.draw_bg.begin(cx,walk,self.layout); let outer=cx.turtle().rect();
        let bounds=Box2::new(outer.pos.x,outer.pos.y,outer.size.x,outer.size.y);
        self.dashboard(cx,Box2::new(bounds.x,bounds.y,bounds.w,110.0));
        let head=Box2::new(bounds.x,bounds.y+110.0,bounds.w,38.0);
        self.chrome.fill(cx,head,p::panel());
        self.chrome.fill(cx,Box2::new(head.x,head.y+head.h-1.0,head.w,1.0),p::border());
        self.chrome.text(cx,head.x+20.0,head.y+12.0,9.0,p::accent(),"SPATIAL MAP");
        let selected_path=self.state.session.snapshot.as_ref().map(|s| {
            let node=&s.tree.nodes[self.state.session.selected.unwrap_or(0).min(s.tree.nodes.len()-1)];
            if node.relative.as_os_str().is_empty() {node.name.clone()} else {format!("{} / {}",s.tree.nodes[0].name,node.relative.display())}
        }).unwrap_or_else(||self.state.session.root.display().to_string());
        self.chrome.text(cx,head.x+125.0,head.y+11.0,10.0,p::secondary(),&f::shorten(&selected_path,((head.w-450.0)/7.0).max(10.0) as usize));
        let detail=if !self.state.session.query.is_empty() {format!("{} matching files",f::count(self.state.session.matched_files as u64))} else {format!("Area: {}",self.state.session.metric.label())};
        self.chrome.text(cx,head.x+head.w-290.0,head.y+11.0,10.0,p::muted(),&detail);
        let side=if self.state.details&&bounds.w>=760.0 {(bounds.w*0.245).clamp(280.0,360.0)} else {0.0};
        let view=Box2::new(bounds.x,head.y+head.h,(bounds.w-side).max(1.0),(bounds.h-180.0).max(1.0));
        if self.state.viewport!=view {
            let previous=self.state.viewport;
            if previous.w>0.0 {self.state.session.camera.pan((view.w-previous.w)*0.5+view.x-previous.x,(view.h-previous.h)*0.5+view.y-previous.y);}
            self.state.viewport=view;
        }
        self.state.session.apply_focus(view);
        clip(cx,view); self.state.frame=self.map.paint_map(cx,&self.state.session,view); cx.end_turtle();
        self.state.panel=Box2::new(view.x+view.w,view.y,side,view.h);
        if side>0.0 {let panel=self.state.panel; clip(cx,panel); self.inspector(cx,panel); cx.end_turtle();}
        let foot=Box2::new(bounds.x,view.y+view.h,bounds.w,32.0);
        self.chrome.fill(cx,foot,p::panel()); self.chrome.fill(cx,Box2::new(foot.x,foot.y,foot.w,1.0),p::border());
        let status=if self.state.session.busy()||self.state.session.snapshot.is_none() {self.state.session.status.clone()} else {"Wheel: zoom  ·  Double-click / F: read code  ·  Home: overview".into()};
        self.chrome.text(cx,foot.x+18.0,foot.y+10.0,9.0,p::secondary(),&f::shorten(&status,((foot.w*0.55-25.0)/6.0).max(1.0) as usize));
        let frame=self.state.frame;
        let source_bytes=self.state.session.snapshot.as_ref().map_or(0,|s|s.tree.source_memory_bytes as u64);
        let telemetry=format!("{} text lines  ·  CPU draw {:.2} ms  ·  Source {}",f::count(frame.source_lines as u64),frame.cpu_submit_ms,f::bytes(source_bytes));
        self.chrome.text(cx,foot.x+foot.w*0.56,foot.y+10.0,9.0,p::muted(),&f::shorten(&telemetry,((foot.w*0.44-16.0)/6.0).max(1.0) as usize));
        self.draw_bg.end(cx); DrawStep::done()
    }
}
