use crate::view::CodeMap;
use makepad_widgets::*;

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
pub struct App {
    #[live]
    ui: WidgetRef,
}

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
            if let Some(mut map) = widget.borrow_mut::<CodeMap>() {
                map.open(cx, args.root, args.options);
            };
        }
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // Widget lookup traverses the tree. Resolve all controls before taking
        // a mutable borrow of CodeMap, and release it before updating controls.
        let open = self.ui.button(id!(open)).clicked(actions);
        let refresh = self.ui.button(id!(refresh)).clicked(actions);
        let home = self.ui.button(id!(home)).clicked(actions);
        let fit = self.ui.button(id!(fit)).clicked(actions);
        let parent = self.ui.button(id!(parent)).clicked(actions);
        let sources = self.ui.button(id!(sources)).clicked(actions);
        let query = self.ui.text_input(id!(filter)).changed(actions);
        let path = open.then(|| self.ui.text_input(id!(root_path)).text());

        if !(open || refresh || home || fit || parent || sources || query.is_some()) {
            return;
        }

        let widget = self.ui.widget(id!(code_map));
        let enabled = {
            let Some(mut map) = widget.borrow_mut::<CodeMap>() else { return; };
            if let Some(path) = path {
                let options = map.options();
                map.open(cx, path.into(), options);
            }
            if refresh { map.refresh(cx); }
            if home { map.focus(cx, Some(0)); }
            if fit { map.focus_selection(cx); }
            if parent { map.parent(cx); }
            if let Some(query) = query { map.filter(cx, query); }
            sources.then(|| map.toggle_sources(cx))
        };

        if let Some(enabled) = enabled {
            self.ui.button(id!(sources)).set_text(cx, if enabled { "Source: on" } else { "Source: off" });
        }
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
