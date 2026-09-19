use makepad_widgets::*;
use ::layout::Box2;
use ::runtime::Session;
use ::render::{MapPainter,RenderStats};
use analysis::ScanOptions;
use std::path::PathBuf;
mod overview;
mod inspector;
mod frame;
mod input;
live_design! {
    use link::theme::*;
    use link::shaders::*;
    use render::painter::MapPainter;
    pub Workspace = {{Workspace}} {
        width: Fill,height: Fill,
        draw_bg: {color: #111317}
        map: <MapPainter> {}
        chrome: <MapPainter> {}
    }
}
#[derive(Live,LiveHook,Widget)]
pub struct Workspace {
    #[redraw] #[live] draw_bg: DrawColor,
    #[live] map: MapPainter,
    #[live] chrome: MapPainter,
    #[walk] walk: Walk,
    #[layout] layout: Layout,
    #[rust] pub state: ViewState,
    #[rust] panel_list: Option<DrawList2d>,
}
pub struct ViewState {
    pub session: Session,
    viewport: Box2,
    panel: Box2,
    drag_at: Option<DVec2>,
    pub details: bool,
    pub panel_dirty: bool,
    panel_scroll: f64,
    panel_height: f64,
    frame: RenderStats,
}
impl Default for ViewState {
    fn default()->Self {
        Self {session:Session::default(),viewport:Box2::default(),panel:Box2::default(),drag_at:None,details:true,panel_dirty:true,panel_scroll:0.0,panel_height:0.0,frame:RenderStats::default()}
    }
}
impl Workspace {
    pub fn open(&mut self,cx:&mut Cx,root:PathBuf,options:ScanOptions) {
        self.state.panel_scroll=0.0;self.state.panel_dirty=true;
        self.state.session.open(root,options,SignalToUI::set_ui_signal);self.redraw_view(cx);
    }
    pub fn redraw_view(&mut self,cx:&mut Cx){self.draw_bg.redraw(cx);}
}
impl Widget for Workspace {
    fn handle_event(&mut self,cx:&mut Cx,event:&Event,_scope:&mut Scope){self.input(cx,event);}
    fn draw_walk(&mut self,cx:&mut Cx2d,_scope:&mut Scope,walk:Walk)->DrawStep{self.frame(cx,walk)}
}
