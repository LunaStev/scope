use makepad_widgets::*;
use crate::view::CodeMap;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    use crate::view::CodeMap;

    App = {{App}} {
        ui: <Root> {
            main_window = <Window> {
                window: { inner_size: vec2(1440, 900), title: "Scope" }
                body = <View> {
                    flow: Down,
                    show_bg: true,
                    draw_bg: {color: #101318}
                    <View> {
                        width: Fill, height: 54,
                        padding: 10, spacing: 8, align: {y: 0.5}
                        <Label> {text: "SCOPE", draw_text: {color: #e3ebf4, text_style: {font_size: 17}}}
                        root_path = <TextInput> {width: Fill, empty_text: "Repository or source directory"}
                        open = <Button> {text: "Open"}
                        refresh = <Button> {text: "Refresh"}
                        home = <Button> {text: "Home"}
                        parent = <Button> {text: "Parent"}
                        fit = <Button> {text: "Fit selection"}
                    }
                    <View> {
                        width: Fill, height: 40,
                        padding: {left: 10, right: 10}, spacing: 10, align: {y: 0.5}
                        filter = <TextInput> {width: 260, empty_text: "Find a file or directory..."}
                        sources = <Button> {text: "Source: on"}
                        <Label> {text: "Area: non-blank lines  |  Wheel: zoom  |  Drag: pan  |  Double-click: focus", draw_text: {color: #8795a8}}
                    }
                    code_map = <CodeMap> {width: Fill, height: Fill}
                }
            }
        }
    }
}

app_main!(App);
#[derive(Live, LiveHook)]
pub struct App { #[live] ui: WidgetRef }
impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
        crate::view::live_design(cx);
    }
}
impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        if let Ok(args) = crate::arguments() {
            self.ui.text_input(id!(root_path)).set_text(cx, &args.root.to_string_lossy());
            let widget = self.ui.widget(id!(code_map));
            if let Some(mut map) = widget.borrow_mut::<CodeMap>() { map.open(cx, args.root, args.options); };
        }
    }
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        let widget = self.ui.widget(id!(code_map));
        let Some(mut map) = widget.borrow_mut::<CodeMap>() else { return; };
        if self.ui.button(id!(open)).clicked(actions) {
            let path = self.ui.text_input(id!(root_path)).text();
            let options = map.options(); map.open(cx, path.into(), options);
        }
        if self.ui.button(id!(refresh)).clicked(actions) { map.refresh(cx); }
        if self.ui.button(id!(home)).clicked(actions) { map.focus(cx, Some(0)); }
        if self.ui.button(id!(fit)).clicked(actions) { map.focus_selection(cx); }
        if self.ui.button(id!(parent)).clicked(actions) { map.parent(cx); }
        if self.ui.button(id!(sources)).clicked(actions) {
            let enabled = map.toggle_sources(cx);
            self.ui.button(id!(sources)).set_text(cx, if enabled { "Source: on" } else { "Source: off" });
        }
        if let Some(query) = self.ui.text_input(id!(filter)).changed(actions) { map.filter(cx, query); }
    }
}
impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
