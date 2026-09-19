use super::Workspace;
use makepad_widgets::*;
use ::model::format as f;
use ::layout::{Box2,Camera,WORLD};
use ::render::palette as p;
fn clip(cx:&mut Cx2d,b:Box2){
    cx.begin_turtle(Walk{abs_pos:Some(dvec2(b.x,b.y)),width:Size::Fixed(b.w),height:Size::Fixed(b.h),..Walk::default()},Layout{clip_x:true,clip_y:true,..Layout::default()});
}
impl Workspace {
    pub(super) fn frame(&mut self,cx:&mut Cx2d,walk:Walk)->DrawStep {
        self.draw_bg.begin(cx,walk,self.layout);let outer=cx.turtle().rect();
        let b=Box2::new(outer.pos.x,outer.pos.y,outer.size.x,outer.size.y);
        self.overview(cx,Box2::new(b.x,b.y,b.w,38.0));
        let side=if self.state.details&&b.w>=850.0{290.0}else{0.0};
        let view=Box2::new(b.x,b.y+38.0,(b.w-side).max(1.0),(b.h-64.0).max(1.0));
        if self.state.viewport!=view {
            let old=self.state.viewport;
            if old.w>0.0{self.state.session.camera.pan((view.w-old.w)*0.5+view.x-old.x,(view.h-old.h)*0.5+view.y-old.y);}
            self.state.viewport=view;self.state.panel_dirty=true;
        }
        self.state.session.apply_focus(view);
        clip(cx,view);self.state.frame=self.map.paint_map(cx,&self.state.session,view);cx.end_turtle();
        self.state.frame.trace_perf();
        let panel=Box2::new(view.x+view.w,view.y,side,view.h);
        if self.state.panel!=panel{self.state.panel_dirty=true;self.state.panel=panel;}
        if side>0.0 {
            let mut list=self.panel_list.take().unwrap_or_else(||DrawList2d::new(cx));
            if list.begin_maybe(cx,self.state.panel_dirty).is_redrawing() {
                clip(cx,panel);self.inspector(cx,panel);cx.end_turtle();list.end(cx);self.state.panel_dirty=false;
            }
            self.panel_list=Some(list);
        }
        let foot=Box2::new(b.x,view.y+view.h,b.w,26.0);
        self.chrome.fill(cx,foot,p::panel());self.chrome.fill(cx,Box2::new(foot.x,foot.y,foot.w,1.0),p::border());
        let frame=self.state.frame;
        let status=if self.state.session.busy()||self.state.session.snapshot.is_none(){self.state.session.status.clone()}
            else if frame.pending_tiles>0{format!("Preparing text · {} visible tiles remaining",f::count(frame.pending_tiles as u64))}
            else{"Ready".into()};
        self.chrome.fill(cx,Box2::new(foot.x+16.0,foot.y+10.0,4.0,4.0),if frame.pending_tiles>0{p::unknown()}else{p::accent()});
        self.chrome.text(cx,foot.x+28.0,foot.y+7.0,10.0,p::muted(),&f::shorten(&status,((foot.w*0.48)/6.6) as usize));
        if foot.w>1000.0{self.chrome.text(cx,foot.x+foot.w*0.42,foot.y+7.0,10.0,p::muted(),"Scroll to zoom   ·   Double-click to read   ·   Home to fit");}
        let mut fit=Camera::default();fit.fit(WORLD,view);
        self.chrome.text_right(cx,foot.x+foot.w-16.0,foot.y+7.0,10.0,p::secondary(),&format!("{:.0}%   ·   Read-only",self.state.session.camera.scale/fit.scale*100.0));
        if frame.pending_tiles>0{cx.new_next_frame();}
        self.draw_bg.end(cx);DrawStep::done()
    }
}
