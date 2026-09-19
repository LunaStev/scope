//! Window composition. Control behavior is kept in app_actions.rs.
use makepad_widgets::*;
live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    use crate::theme::*;
    use crate::workspace::Workspace;
    App = {{App}} {
        ui: <Root> { main_window = <Window> {
            window: {inner_size: vec2(1440,900),title: "Scope"}
            body = <View> {
                flow: Down, show_bg: true, draw_bg: {color: #111317}
                <View> {
                    width: Fill,height: 54,padding: {left: 18,right: 16},spacing: 12,align: {y: 0.5}
                    <Label> {width: 80,text: "Scope",draw_text: {color: #e0e4ed,text_style: {font_size: 21}}}
                    root_path = <ScopeInput> {width: Fill,empty_text: "Repository or source folder"}
                    open = <ScopePrimary> {text: "Open"}
                    refresh = <ScopeButton> {text: "Refresh"}
                }
                <View> {width: Fill,height: 1,show_bg: true,draw_bg: {color: #2a2e37}}
                <View> {
                    width: Fill,height: 43,padding: {left: 16,right: 16},spacing: 6,align: {y: 0.5}
                    home = <ScopeButton> {text: "Overview"}
                    parent = <ScopeButton> {text: "Up"}
                    fit = <ScopeButton> {text: "Read"}
                    <View> {width: 8,height: 1}
                    metric = <ScopeButton> {width: 126,text: "Non-blank lines"}
                    sources = <ScopeButton> {text: "Code: on"}
                    <View> {width: 8,height: 1}
                    filter = <ScopeInput> {width: Fill,empty_text: "Search paths"}
                    details = <ScopeButton> {text: "Details"}
                }
                workspace = <Workspace> {width: Fill,height: Fill}
            }
        }}
    }
}
app_main!(App);
#[derive(Live,LiveHook)]
pub struct App { #[live] pub ui: WidgetRef }
impl LiveRegister for App {
    fn live_register(cx:&mut Cx) {
        makepad_widgets::live_design(cx); ::render::live_design(cx);
        crate::theme::live_design(cx); crate::workspace::live_design(cx);
    }
}
impl AppMain for App {
    fn handle_event(&mut self,cx:&mut Cx,event:&Event) {
        self.match_event(cx,event);self.ui.handle_event(cx,event,&mut Scope::empty());
    }
}
