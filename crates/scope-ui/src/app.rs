use makepad_widgets::*;
use crate::workspace::Workspace;
live_design!{
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    use crate::theme::{ScopeButton,ScopeInput};
    use crate::workspace::Workspace;
    App={{App}}{
        ui:<Root>{main_window=<Window>{
            window:{inner_size:vec2(1480,940),title:"Scope · Codebase explorer"}
            body=<View>{
                flow:Down,show_bg:true,draw_bg:{color:#0b1018}
                <View>{
                    width:Fill,height:68,padding:{left:22,right:20},spacing:16,align:{y:0.5}
                    <View>{width:122,height:Fit,flow:Down,spacing:3
                        <Label>{text:"SCOPE",draw_text:{color:#e7eef6,text_style:{font_size:22}}}
                        <Label>{text:"CODEBASE EXPLORER",draw_text:{color:#76adad,text_style:{font_size:8}}}
                    }
                    root_path=<ScopeInput>{width:Fill,empty_text:"Open a repository or source directory"}
                    open=<ScopeButton>{text:"Open folder"}
                    refresh=<ScopeButton>{text:"Re-index"}
                    <Label>{text:"LOCAL / READ-ONLY",draw_text:{color:#66dac4,text_style:{font_size:9}}}
                }
                <View>{width:Fill,height:1,show_bg:true,draw_bg:{color:#273546}}
                <View>{
                    width:Fill,height:52,padding:{left:20,right:20},spacing:8,align:{y:0.5}
                    filter=<ScopeInput>{width:Fill,empty_text:"Find a file or directory by path..."}
                    home=<ScopeButton>{text:"Overview"}
                    parent=<ScopeButton>{text:"Parent"}
                    fit=<ScopeButton>{text:"Focus"}
                    metric=<ScopeButton>{text:"Area: non-blank"}
                    sources=<ScopeButton>{text:"Source: on"}
                    details=<ScopeButton>{text:"Inspector: on"}
                }
                workspace=<Workspace>{width:Fill,height:Fill}
            }
        }}
    }
}
app_main!(App);
#[derive(Live,LiveHook)]
pub struct App{#[live]ui:WidgetRef}
impl LiveRegister for App{
    fn live_register(cx:&mut Cx){
        makepad_widgets::live_design(cx);scope_render::live_design(cx);
        crate::theme::live_design(cx);crate::workspace::live_design(cx);
    }
}
impl MatchEvent for App{
    fn handle_startup(&mut self,cx:&mut Cx){
        if let Some(args)=crate::STARTUP.get(){
            self.ui.text_input(id!(root_path)).set_text(cx,&args.root.to_string_lossy());
            let widget=self.ui.widget(id!(workspace));
            if let Some(mut view)=widget.borrow_mut::<Workspace>(){view.open(cx,args.root.clone(),args.options.clone());};
        }
    }
    fn handle_actions(&mut self,cx:&mut Cx,actions:&Actions){
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
        if !(open||refresh||home||parent||fit||metric||sources||details||query.is_some()){return;}
        // Resolve controls before borrowing the workspace widget, then release
        // the borrow before writing labels back through the widget tree.
        let widget=self.ui.widget(id!(workspace));
        let (metric_label,source_label,details_label)={
            let Some(mut view)=widget.borrow_mut::<Workspace>()else{return;};
            if let Some(path)=path{let options=view.state.session.options.clone();view.open(cx,path.trim().into(),options);}
            let session=&mut view.state.session;
            if refresh{session.refresh();}
            if home{session.focus(0);}
            if parent{session.parent();}
            if fit{let id=session.selected.unwrap_or(0);session.focus(id);}
            if let Some(query)=query{session.filter(query);}
            if metric{session.cycle_metric();}
            if sources{session.sources=!session.sources;}
            let metric_label=format!("Area: {}",session.requested_metric.label());
            let source_label=if session.sources{"Source: on"}else{"Source: off"};
            if details{view.state.details=!view.state.details;}
            let details_label=if view.state.details{"Inspector: on"}else{"Inspector: off"};
            view.redraw_view(cx);(metric_label,source_label,details_label)
        };
        self.ui.button(id!(metric)).set_text(cx,&metric_label);
        self.ui.button(id!(sources)).set_text(cx,source_label);
        self.ui.button(id!(details)).set_text(cx,details_label);
    }
}
impl AppMain for App{
    fn handle_event(&mut self,cx:&mut Cx,event:&Event){
        self.match_event(cx,event);self.ui.handle_event(cx,event,&mut Scope::empty());
    }
}
