use super::Workspace;
use makepad_widgets::*;
use ::layout::wheel_zoom_factor;
impl Workspace {
    pub(super) fn input(&mut self,cx:&mut Cx,event:&Event) {
        if self.state.session.poll(){self.redraw_view(cx);}
        if matches!(event,Event::NextFrame(_))&&self.state.frame.needs_redraw{self.redraw_view(cx);}
        match event.hits(cx,self.draw_bg.area()) {
            Hit::FingerDown(e)=>{
                if self.state.viewport.contains(e.abs.x,e.abs.y){
                    cx.set_key_focus(self.draw_bg.area());self.state.session.selected=self.state.session.hit(e.abs.x,e.abs.y);
                    self.state.drag_at=Some(e.abs);self.state.panel_scroll=0.0;
                    if e.tap_count>=2{if let Some(id)=self.state.session.selected{self.state.session.read_at(id,e.abs.x,e.abs.y);}}
                    self.redraw_view(cx);
                }
            }
            Hit::FingerMove(e)=>{
                if let Some(last)=self.state.drag_at{self.state.session.camera.pan(e.abs.x-last.x,e.abs.y-last.y);self.state.drag_at=Some(e.abs);self.redraw_view(cx);}
            }
            Hit::FingerUp(_)=>{self.state.drag_at=None;}
            Hit::FingerScroll(e)=>{
                if self.state.details&&self.state.panel.contains(e.abs.x,e.abs.y){
                    self.state.panel_scroll=(self.state.panel_scroll+e.scroll.y).clamp(0.0,(self.state.panel_height-self.state.panel.h).max(0.0));self.redraw_view(cx);
                }else if self.state.viewport.contains(e.abs.x,e.abs.y){
                    self.state.session.camera.zoom_at(wheel_zoom_factor(e.scroll.y),e.abs.x,e.abs.y);self.redraw_view(cx);
                }
                if std::env::var_os("SCOPE_INPUT_TRACE").is_some(){eprintln!("scope input: kind=scroll timestamp_ns={}",std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());}
            }
            Hit::FingerHoverIn(e)|Hit::FingerHoverOver(e)=>{
                let hit=if self.state.viewport.contains(e.abs.x,e.abs.y){self.state.session.hit(e.abs.x,e.abs.y)}else{None};
                if hit!=self.state.session.hovered{self.state.session.hovered=hit;self.redraw_view(cx);}
            }
            Hit::FingerHoverOut(_)=>{if self.state.session.hovered.take().is_some(){self.redraw_view(cx);}}
            Hit::KeyDown(e)=>{
                match e.key_code{KeyCode::Home=>self.state.session.focus(0),KeyCode::KeyF=>{let id=self.state.session.selected.unwrap_or(0);self.state.session.focus(id);},KeyCode::Backspace=>self.state.session.parent(),_=>return,}
                self.redraw_view(cx);
            }
            _=>{}
        }
    }
}
