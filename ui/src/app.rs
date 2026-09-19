use makepad_widgets::*;
use crate::workspace::Workspace;
live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    use crate::theme::*;
    use crate::workspace::Workspace;
    App = {{App}} {
        ui: <Root> {main_window = <Window> {
            window: {inner_size: vec2(1440,900), title: "Scope"}
            body = <View> {
                flow: Down, show_bg: true, draw_bg: {color: #111518}
                <View> {
                    width: Fill, height: 44, padding: {left: 14,right: 14}, spacing: 8, align: {y: 0.5}
                    <Label> {width: 74, text: "Scope",draw_text: {color: #e0e5e9,text_style: {font_size: 17}}}
                    root_path = <ScopeInput> {width: Fill,empty_text: "Repository path"}
                    open = <ScopeButton> {text: "Open"}
                    refresh = <ScopeButton> {text: "Re-index"}
                    details = <ScopeButton> {text: "Details: on"}
                }
                <View> {width: Fill,height: 1,show_bg: true,draw_bg: {color: #292f35}}
                <View> {
                    width: Fill,height: 38,padding: {left: 14,right: 14},spacing: 6,align: {y: 0.5}
                    home = <ScopeButton> {text: "Overview"}
                    parent = <ScopeButton> {text: "Up"}
                    fit = <ScopeButton> {text: "Read"}
                    filter = <ScopeInput> {width: Fill,empty_text: "Find path..."}
                    metric = <ScopeButton> {width: 176,text: "Area: Non-blank lines"}
                    sources = <ScopeButton> {text: "Source: on"}
                }
                workspace = <Workspace> {width: Fill,height: Fill}
            }
        }}
    }
}
app_main!(App);
#[derive(Live,LiveHook)]
pub struct App { #[live] ui: WidgetRef }
impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx); ::render::live_design(cx);
        crate::theme::live_design(cx); crate::workspace::live_design(cx);
    }
}
impl MatchEvent for App {
    fn handle_startup(&mut self,cx:&mut Cx) {
        if let Some(args)=crate::STARTUP.get() {
            self.ui.text_input(id!(root_path)).set_text(cx,&args.root.to_string_lossy());
            let widget=self.ui.widget(id!(workspace));
            if let Some(mut view)=widget.borrow_mut::<Workspace>() {view.open(cx,args.root.clone(),args.options.clone());};
        }
    }
    fn handle_actions(&mut self,cx:&mut Cx,actions:&Actions) {
        let open=self.ui.button(id!(open)).clicked(actions);
        let refresh=self.ui.button(id!(refresh)).clicked(actions);
        let home=self.ui.button(id!(home)).clicked(actions);
        let parent=self.ui.button(id!(parent)).clicked(actions);
        let fit=self.ui.button(id!(fit)).clicked(actions);
        let metric=self.ui.button(id!(metric)).clicked(actions);
        let sources=self.ui.button(id!(sources)).clicked(actions);
        let details=self.ui.button(id!(details)).clicked(actions);
        let query=self.ui.text_input(id!(filter)).changed(actions);
        let path=open.then(||self.ui.text_input(id!(root_path)).text());
        if !(open||refresh||home||parent||fit||metric||sources||details||query.is_some()) {return;}
        let widget=self.ui.widget(id!(workspace));
        let (metric_label,source_label,details_label)={
            let Some(mut view)=widget.borrow_mut::<Workspace>()else{return;};
            if let Some(path)=path {let options=view.state.session.options.clone();view.open(cx,path.trim().into(),options);}
            let session=&mut view.state.session;
            if refresh {session.refresh();}
            if home {session.focus(0);}
            if parent {session.parent();}
            if fit {let id=session.selected.unwrap_or(0);session.focus(id);}
            if let Some(query)=query {session.filter(query);}
            if metric {session.cycle_metric();}
            if sources {session.sources=!session.sources;}
            let metric_label=format!("Area: {}",session.requested_metric.label());
            let source_label=if session.sources {"Source: on"} else {"Source: off"};
            if details {view.state.details=!view.state.details;}
            let details_label=if view.state.details {"Details: on"} else {"Details: off"};
            view.redraw_view(cx);(metric_label,source_label,details_label)
        };
        self.ui.button(id!(metric)).set_text(cx,&metric_label);
        self.ui.button(id!(sources)).set_text(cx,source_label);
        self.ui.button(id!(details)).set_text(cx,details_label);
    }
}
impl AppMain for App {
    fn handle_event(&mut self,cx:&mut Cx,event:&Event) {
        self.match_event(cx,event);self.ui.handle_event(cx,event,&mut Scope::empty());
    }
}
