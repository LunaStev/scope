//! Route toolbar commands without embedding behavior in window composition.
use makepad_widgets::*;
use crate::{app::App,workspace::Workspace};
impl MatchEvent for App {
    fn handle_startup(&mut self,cx:&mut Cx) {
        if let Some(args)=crate::STARTUP.get() {
            self.ui.text_input(id!(root_path)).set_text(cx,&args.root.to_string_lossy());
            let widget=self.ui.widget(id!(workspace));
            if let Some(mut view)=widget.borrow_mut::<Workspace>(){view.open(cx,args.root.clone(),args.options.clone());};
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
        if !(open||refresh||home||parent||fit||metric||sources||details||query.is_some()){return;}
        let widget=self.ui.widget(id!(workspace));
        let (metric_label,source_label,detail_label)={
            let Some(mut view)=widget.borrow_mut::<Workspace>()else{return;};
            if let Some(path)=path{let options=view.state.session.options.clone();view.open(cx,path.into(),options);}
            let session=&mut view.state.session;
            if refresh{session.refresh();}
            if home{session.focus(0);}
            if parent{session.parent();}
            if fit{let id=session.selected.unwrap_or(0);session.focus(id);}
            if let Some(query)=query{session.filter(query);}
            if metric{session.cycle_metric();}
            if sources{session.sources=!session.sources;}
            let metric_label=session.requested_metric.label();
            let source_label=if session.sources{"Code: on"}else{"Code: off"};
            if details{view.state.details=!view.state.details;}
            let detail_label=if view.state.details{"Details"}else{"Show details"};
            view.state.panel_dirty=true;view.redraw_view(cx);
            (metric_label,source_label,detail_label)
        };
        self.ui.button(id!(metric)).set_text(cx,metric_label);
        self.ui.button(id!(sources)).set_text(cx,source_label);
        self.ui.button(id!(details)).set_text(cx,detail_label);
    }
}
