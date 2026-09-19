use super::Workspace;
use makepad_widgets::*;
impl Workspace{
    pub(super) fn input(&mut self,cx:&mut Cx,event:&Event){
        if self.state.session.poll(){self.redraw_view(cx);}
        match event.hits(cx,self.draw_bg.area()){
            Hit::FingerDown(e)=>{
                if self.state.viewport.contains(e.abs.x,e.abs.y){
                    cx.set_key_focus(self.draw_bg.area());
                    self.state.session.selected=self.state.session.hit(e.abs.x,e.abs.y);
                    self.state.drag_at=Some(e.abs);self.state.panel_scroll=0.0;
                    if e.tap_count>=2{self.state.session.focus_pending=self.state.session.selected;}
                    self.redraw_view(cx);
                }
            }
            Hit::FingerMove(e)=>{
                if let Some(last)=self.state.drag_at{
                    self.state.session.camera.pan(e.abs.x-last.x,e.abs.y-last.y);self.state.drag_at=Some(e.abs);self.redraw_view(cx);
                }
            }
            Hit::FingerUp(_)=>{self.state.drag_at=None;}
            Hit::FingerScroll(e)=>{
                if self.state.details&&self.state.panel.contains(e.abs.x,e.abs.y){
                    self.state.panel_scroll=(self.state.panel_scroll+e.scroll.y).clamp(0.0,(self.state.panel_height-self.state.panel.h).max(0.0));
                    self.redraw_view(cx);
                }else if self.state.viewport.contains(e.abs.x,e.abs.y){
                    self.state.session.camera.zoom_at((-e.scroll.y*0.006).clamp(-1.0,1.0).exp(),e.abs.x,e.abs.y);self.redraw_view(cx);
                }
            }
            Hit::FingerHoverIn(e)|Hit::FingerHoverOver(e)=>{
                let hit=if self.state.viewport.contains(e.abs.x,e.abs.y){self.state.session.hit(e.abs.x,e.abs.y)}else{None};
                if hit!=self.state.session.hovered{self.state.session.hovered=hit;self.redraw_view(cx);}
            }
            Hit::FingerHoverOut(_)=>{self.state.session.hovered=None;self.redraw_view(cx);}
            Hit::KeyDown(e)=>{
                match e.key_code{
                    KeyCode::Home=>self.state.session.focus(0),
                    KeyCode::KeyF=>{let id=self.state.session.selected.unwrap_or(0);self.state.session.focus(id);}
                    KeyCode::Backspace=>self.state.session.parent(),
                    _=>{}
                }
                self.redraw_view(cx);
            }
            _=>{}
        }
    }
}
