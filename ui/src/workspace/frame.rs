use super::Workspace;
use makepad_widgets::*;
use ::layout::Box2;
pub(super) fn clip(cx:&mut Cx2d,b:Box2){cx.begin_turtle(Walk{abs_pos:Some(dvec2(b.x,b.y)),width:Size::Fixed(b.w),height:Size::Fixed(b.h),..Walk::default()},Layout{clip_x:true,clip_y:true,..Layout::default()});}
impl Workspace {
    pub(super) fn frame(&mut self,cx:&mut Cx2d,walk:Walk)->DrawStep {
        self.state.graphics.refresh(cx);
        self.draw_bg.begin(cx,walk,self.layout);let outer=cx.turtle().rect();let b=Box2::new(outer.pos.x,outer.pos.y,outer.size.x,outer.size.y);
        self.summary(cx,Box2::new(b.x,b.y,b.w,30.0));let side=if self.state.details&&b.w>=720.0{(b.w*0.2).clamp(248.0,280.0)}else{0.0};
        let view=Box2::new(b.x,b.y+30.0,(b.w-side).max(1.0),(b.h-54.0).max(1.0));
        if self.state.viewport!=view{let old=self.state.viewport;if old.w>0.0{self.state.session.camera.pan((view.w-old.w)*0.5+view.x-old.x,(view.h-old.h)*0.5+view.y-old.y);}self.state.viewport=view;self.state.session.resize(view);}
        self.state.session.apply_focus(view);clip(cx,view);self.state.frame=self.map.paint_map(cx,&mut self.state.session,view);cx.end_turtle();
        // An active source read wakes us itself; don't spin waiting for workers.
        if self.state.frame.needs_redraw{let _=cx.new_next_frame();}
        self.state.panel=Box2::new(view.x+view.w,view.y,side,view.h);if side>0.0{let panel=self.state.panel;clip(cx,panel);self.inspector(cx,panel);cx.end_turtle();}
        self.status_bar(cx,Box2::new(b.x,view.y+view.h,b.w,24.0));
        self.draw_bg.end(cx);DrawStep::done()
    }
}
