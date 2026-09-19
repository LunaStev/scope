use super::Workspace;
use makepad_widgets::*;
use ::model::format as f;
use ::layout::Box2;
use ::render::palette as p;
fn clip(cx:&mut Cx2d,b:Box2){cx.begin_turtle(Walk{abs_pos:Some(dvec2(b.x,b.y)),width:Size::Fixed(b.w),height:Size::Fixed(b.h),..Walk::default()},Layout{clip_x:true,clip_y:true,..Layout::default()});}
impl Workspace {
    pub(super) fn frame(&mut self,cx:&mut Cx2d,walk:Walk)->DrawStep {
        self.draw_bg.begin(cx,walk,self.layout);let outer=cx.turtle().rect();let b=Box2::new(outer.pos.x,outer.pos.y,outer.size.x,outer.size.y);
        self.summary(cx,Box2::new(b.x,b.y,b.w,30.0));let side=if self.state.details&&b.w>=720.0{(b.w*0.2).clamp(248.0,280.0)}else{0.0};
        let view=Box2::new(b.x,b.y+30.0,(b.w-side).max(1.0),(b.h-54.0).max(1.0));
        if self.state.viewport!=view{let old=self.state.viewport;if old.w>0.0{self.state.session.camera.pan((view.w-old.w)*0.5+view.x-old.x,(view.h-old.h)*0.5+view.y-old.y);}self.state.viewport=view;self.state.session.resize(view);}
        self.state.session.apply_focus(view);clip(cx,view);self.state.frame=self.map.paint_map(cx,&mut self.state.session,view);cx.end_turtle();
        // An active source read wakes us itself. Do not spin at display speed
        // while a background worker is the only thing that can make progress.
        if self.state.frame.needs_redraw{let _=cx.new_next_frame();}
        self.state.panel=Box2::new(view.x+view.w,view.y,side,view.h);if side>0.0{let panel=self.state.panel;clip(cx,panel);self.inspector(cx,panel);cx.end_turtle();}
        let foot=Box2::new(b.x,view.y+view.h,b.w,24.0);self.chrome.fill(cx,foot,p::panel());self.chrome.fill(cx,Box2::new(foot.x,foot.y,foot.w,1.0),p::border());
        let status=if self.state.frame.pending_chunks>0{format!("Preparing source · {} queued regions",f::count(self.state.frame.pending_chunks as u64))}
            else if self.state.session.documents.pending()>0{format!("Loading source · {} active reads",self.state.session.documents.pending())}
            else if self.state.frame.limited_chunks>0||self.state.frame.limited_nodes>0{"Visible detail budget reached · focus a region".into()}
            else if self.state.frame.needs_redraw{"Preparing map labels".into()}
            else if self.state.session.busy(){self.state.session.status.clone()}
            else if !self.state.session.query.is_empty(){format!("{} matching files",f::count(self.state.session.matched_files as u64))}
            else{"Wheel zoom · Drag pan · Double-click read · Home overview".into()};
        self.chrome.text(cx,foot.x+14.0,foot.y+7.0,9.0,p::muted(),&f::shorten(&status,((b.w*0.52-25.0)/6.0).max(1.0) as usize));
        let frame=self.state.frame;let telemetry=format!("{} batches reused · {:.2} ms CPU",frame.reused_chunks,frame.cpu_submit_ms);
        self.chrome.text_right(cx,foot.x+foot.w-14.0,foot.y+7.0,9.0,p::muted(),&telemetry);self.draw_bg.end(cx);DrawStep::done()
    }
}
