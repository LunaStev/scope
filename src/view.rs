use makepad_widgets::*;
use scope::camera::Camera;
use scope::layout::{self, Box2, WORLD};
use scope::model::{FileInfo, Tree};
use scope::scanner::{self, ScanOptions};
use scope::source::{self, Document, Ink};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, mpsc, atomic::{AtomicBool, Ordering}};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    pub CodeMap = {{CodeMap}} {
        width: Fill, height: Fill,
        draw_bg: {color: #0b1016}
        quad: {color: #1c2935}
        label: {text_style: <THEME_FONT_REGULAR> {font_size: 11}, color: #c5d1df}
        code: {text_style: <THEME_FONT_CODE> {font_size: 11}, color: #bac9d7}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct CodeMap {
    #[redraw] #[live] draw_bg: DrawColor,
    #[live] quad: DrawColor,
    #[live] label: DrawText,
    #[live] code: DrawText,
    #[walk] walk: Walk,
    #[layout] layout: Layout,
    #[rust] state: State,
}
struct Loaded { tree: Arc<Tree>, rectangles: Vec<Box2> }
type SourceResult = (usize, Result<Document, String>);
struct State {
    root: PathBuf,
    options: ScanOptions,
    tree: Option<Arc<Tree>>,
    rectangles: Vec<Box2>,
    camera: Camera,
    viewport: Box2,
    selected: Option<usize>,
    hovered: Option<usize>,
    focus_pending: Option<usize>,
    drag_at: Option<DVec2>,
    query: String,
    matches: HashSet<usize>,
    sources: bool,
    status: String,
    scan_rx: Option<mpsc::Receiver<Result<Loaded,String>>>,
    cancel: Arc<AtomicBool>,
    source_tx: mpsc::Sender<SourceResult>,
    source_rx: mpsc::Receiver<SourceResult>,
    loading: HashSet<usize>,
    cache: HashMap<usize,Arc<Document>>,
    order: VecDeque<usize>,
    cache_bytes: usize,
    source_errors: HashMap<usize,String>,
}
impl Default for State {
    fn default() -> Self {
        let (source_tx,source_rx) = mpsc::channel();
        Self {
            root: PathBuf::new(), options: ScanOptions::default(), tree: None, rectangles: Vec::new(),
            camera: Camera::default(), viewport: Box2::default(), selected: None, hovered: None,
            focus_pending: Some(0), drag_at: None, query: String::new(), matches: HashSet::new(), sources: true,
            status: "Open a source directory to begin".into(), scan_rx: None, cancel: Arc::new(AtomicBool::new(false)),
            source_tx,source_rx,loading:HashSet::new(),cache:HashMap::new(),order:VecDeque::new(),cache_bytes:0,source_errors:HashMap::new(),
        }
    }
}

impl CodeMap {
    pub fn options(&self) -> ScanOptions { self.state.options.clone() }
    pub fn open(&mut self, cx:&mut Cx, root:PathBuf, options:ScanOptions) {
        self.state.cancel.store(true,Ordering::Relaxed);
        let sources = self.state.sources;
        self.state = State { root:root.clone(), options:options.clone(), sources, status:format!("Scanning {} ...",root.display()), ..State::default() };
        let (tx,rx) = mpsc::channel(); self.state.scan_rx = Some(rx);
        let cancel = self.state.cancel.clone();
        std::thread::spawn(move || {
            let result = scanner::scan(&root,&options,&cancel).map(|tree| {
                let rectangles = layout::tree_layout(&tree);
                Loaded { tree:Arc::new(tree),rectangles }
            });
            let _ = tx.send(result); SignalToUI::set_ui_signal();
        });
        self.draw_bg.redraw(cx);
    }
    pub fn refresh(&mut self,cx:&mut Cx) { self.open(cx,self.state.root.clone(),self.state.options.clone()); }
    pub fn focus(&mut self,cx:&mut Cx,id:Option<usize>) {
        if let Some(id) = id { self.state.selected=Some(id); self.state.focus_pending=Some(id); self.draw_bg.redraw(cx); }
    }
    pub fn focus_selection(&mut self,cx:&mut Cx) { self.focus(cx,self.state.selected.or(Some(0))); }
    pub fn parent(&mut self,cx:&mut Cx) {
        let parent=self.state.tree.as_ref().and_then(|t| t.nodes.get(self.state.selected.unwrap_or(0))).and_then(|n| n.parent).unwrap_or(0);
        self.focus(cx,Some(parent));
    }
    pub fn toggle_sources(&mut self,cx:&mut Cx)->bool { self.state.sources=!self.state.sources; self.draw_bg.redraw(cx); self.state.sources }
    pub fn filter(&mut self,cx:&mut Cx,query:String) {
        self.state.query=query.to_lowercase(); self.update_matches(); self.draw_bg.redraw(cx);
    }
    fn update_matches(&mut self) {
        self.state.matches.clear();
        if self.state.query.is_empty() { return; }
        if let Some(tree)=&self.state.tree {
            for (id,node) in tree.nodes.iter().enumerate() {
                if node.relative.to_string_lossy().to_lowercase().contains(&self.state.query) { self.state.matches.insert(id); }
            }
        }
    }
    fn receive(&mut self,cx:&mut Cx) {
        let received=self.state.scan_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(result)=received {
            self.state.scan_rx=None;
            match result {
                Ok(loaded)=>{
                    let stats=loaded.tree.nodes[0].stats;
                    self.state.status=format!("{} files  /  {} lines  /  {} non-blank lines",stats.files,stats.lines,stats.non_blank);
                    self.state.root=loaded.tree.root.clone(); self.state.tree=Some(loaded.tree); self.state.rectangles=loaded.rectangles;
                    self.state.selected=Some(0); self.state.focus_pending=Some(0); self.update_matches();
                }
                Err(error)=>self.state.status=error,
            }
            self.draw_bg.redraw(cx);
        }
        while let Ok((id,result))=self.state.source_rx.try_recv() {
            self.state.loading.remove(&id);
            match result {
                Ok(document)=>{
                    let size=document.storage_bytes();
                    if size>32*1024*1024 {
                        self.state.source_errors.insert(id,"Source line index exceeds the 32 MiB preview cache limit".into());
                        self.draw_bg.redraw(cx); continue;
                    }
                    while self.state.cache.len()>=8 || self.state.cache_bytes+size>32*1024*1024 {
                        let Some(old)=self.state.order.pop_front() else { break; };
                        if let Some(old)=self.state.cache.remove(&old) { self.state.cache_bytes-=old.storage_bytes(); }
                    }
                    self.state.cache_bytes+=size; self.state.cache.insert(id,Arc::new(document)); self.state.order.push_back(id);
                }
                Err(error)=>{ self.state.source_errors.insert(id,error); }
            }
            self.draw_bg.redraw(cx);
        }
    }
    fn request_source(&mut self,id:usize) {
        if self.state.loading.len()>=3 || self.state.loading.contains(&id) || self.state.cache.contains_key(&id) || self.state.source_errors.contains_key(&id) { return; }
        let Some(tree)=&self.state.tree else { return; };
        let Some(node)=tree.nodes.get(id) else { return; };
        if node.file.is_none() { return; }
        let path=tree.root.join(&node.relative);
        let tx=self.state.source_tx.clone();
        let limit=self.state.options.max_file_bytes.min(16*1024*1024);
        self.state.loading.insert(id);
        std::thread::spawn(move || {
            let result=source::read_text(&path,limit).map_err(|e| e.to_string()).and_then(|text| text.ok_or_else(|| "File is no longer UTF-8 text".into())).map(Document::new);
            let _=tx.send((id,result)); SignalToUI::set_ui_signal();
        });
    }
    fn box_fill(&mut self,cx:&mut Cx2d,b:Box2,c:Vec4) {
        if b.w<=0.0 || b.h<=0.0 { return; }
        self.quad.color=c; self.quad.draw_abs(cx,rect(b));
    }
    fn text(&mut self,cx:&mut Cx2d,x:f64,y:f64,size:f32,color:Vec4,text:&str) {
        self.label.color=color; self.label.text_style.font_size=size;
        self.label.draw_abs(cx,dvec2(x,y),text);
    }
    fn draw_map(&mut self,cx:&mut Cx2d,view:Box2) {
        let Some(tree)=self.state.tree.clone() else {
            let status=self.state.status.clone(); self.text(cx,view.x+24.0,view.y+36.0,14.0,rgb(160,182,200),&status); return;
        };
        if tree.nodes[0].stats.files==0 {
            self.text(cx,view.x+24.0,view.y+40.0,14.0,rgb(160,182,200),"No readable text files. Check the path and exclusion options."); return;
        }
        let mut stack=vec![0];
        let mut visible=Vec::new();
        while let Some(id)=stack.pop() {
            let b=self.state.camera.project(self.state.rectangles[id]);
            if b.w<0.6 || b.h<0.6 || !b.intersects(view) { continue; }
            visible.push((id,b));
            if visible.len()>=30000 { break; }
            stack.extend(tree.nodes[id].children.iter().rev().copied());
        }
        self.quad.begin_many_instances(cx);
        for &(id,b) in &visible {
            let node=&tree.nodes[id];
            let base=node.file.as_ref().map(|f| tint(&f.language)).unwrap_or(rgb(90,111,128));
            let dim=if !self.state.query.is_empty() && !self.state.matches.contains(&id) && node.file.is_some() { 0.25 } else { 1.0 };
            let selected=self.state.selected==Some(id);
            let hovered=self.state.hovered==Some(id);
            self.box_fill(cx,b,if selected { rgb(201,222,237) } else if hovered { rgb(141,190,215) } else { shade(base,0.65*dim) });
            self.box_fill(cx,b.inset(if selected || hovered { 1.6 } else { 0.8 }),shade(base,if node.file.is_some() {0.13*dim} else {0.065}));
            if self.state.sources {
                if let Some(file)=&node.file {
                    if b.w>10.0 && b.h>14.0 { self.draw_preview(cx,b,file,node.stats.lines as usize,shade(base,0.70*dim)); }
                }
            }
        }
        self.quad.end_many_instances(cx);
        let mut text_budget=6500usize;
        let mut requested=0;
        for &(id,b) in &visible {
            let node=&tree.nodes[id];
            if b.w>45.0 && b.h>18.0 {
                let name=shorten(&node.name,((b.w-8.0)/6.7) as usize);
                self.text(cx,b.x+4.0,b.y+3.0,10.0,if node.file.is_some(){rgb(161,190,211)}else{rgb(228,231,237)},&name);
            }
            if !self.state.sources || node.file.is_none() || b.w<120.0 || b.h<70.0 { continue; }
            let (_,_,_,font)=code_geometry(b,node.stats.lines as usize);
            if font<7.0 { continue; }
            if let Some(doc)=self.state.cache.get(&id).cloned() {
                self.draw_document(cx,b,view,&doc,&mut text_budget);
            } else if requested<3 { self.request_source(id); requested+=1; }
        }
    }
    fn draw_preview(&mut self,cx:&mut Cx2d,b:Box2,file:&FileInfo,lines:usize,ink:Vec4) {
        let (cols,rows,content,_)=code_geometry(b,lines);
        let cell_w=content.w/cols as f64;
        let line_h=content.h/rows.max(1) as f64;
        for sample in &file.preview {
            if sample.width==0 { continue; }
            let col=sample.line/rows.max(1); let row=sample.line%rows.max(1);
            if col>=cols { continue; }
            let x=content.x+col as f64*cell_w+(sample.indent.min(140) as f64/180.0)*cell_w;
            let width=(sample.width.min(140) as f64/180.0*cell_w).max(0.5).min((content.x+(col+1) as f64*cell_w-x).max(0.0));
            self.box_fill(cx,Box2::new(x,content.y+row as f64*line_h,width,(line_h*0.45).clamp(0.45,2.0)),if sample.comment { shade(ink,0.5) } else { ink });
        }
    }
    fn draw_document(&mut self,cx:&mut Cx2d,b:Box2,view:Box2,doc:&Document,budget:&mut usize) {
        let (cols,rows,content,font)=code_geometry(b,doc.lines.len());
        if font<7.0 || *budget==0 { return; }
        self.box_fill(cx,content,rgb(12,19,26));
        let col_w=content.w/cols as f64;
        let row_h=content.h/rows.max(1) as f64;
        let font=font.min(28.0) as f32;
        let char_w=font as f64*0.81;
        let first_row=(((view.y-content.y)/row_h).floor() as isize).max(0) as usize;
        let last_row=(((view.y+view.h-content.y)/row_h).ceil() as isize).max(0) as usize;
        self.code.text_style.font_size=font;
        for col in 0..cols {
            let x=content.x+col as f64*col_w;
            if x+col_w<view.x || x>view.x+view.w { continue; }
            for row in first_row..last_row.min(rows) {
                let line=col*rows+row;
                if line>=doc.lines.len() || *budget==0 { break; }
                let y=content.y+row as f64*row_h;
                for (column,text,ink) in source::runs(doc.line(line)) {
                    if *budget==0 { break; }
                    let x=x+column as f64*char_w;
                    if x>=content.x+(col+1) as f64*col_w { break; }
                    let max_chars=((content.x+(col+1) as f64*col_w-x)/char_w).floor().max(0.0) as usize;
                    let text:String=text.chars().take(max_chars).collect();
                    if text.is_empty() { continue; }
                    self.code.color=match ink { Ink::Text=>rgb(159,180,198),Ink::Keyword=>rgb(197,154,210),Ink::Number=>rgb(219,180,125),Ink::String=>rgb(155,187,148),Ink::Comment=>rgb(89,130,125) };
                    self.code.draw_abs(cx,dvec2(x,y),&text); *budget-=1;
                }
            }
        }
    }
    fn inspector(&mut self,cx:&mut Cx2d,b:Box2) {
        self.box_fill(cx,b,rgb(17,22,29));
        self.box_fill(cx,Box2::new(b.x,b.y,1.0,b.h),rgb(48,62,76));
        let x=b.x+16.0;
        self.text(cx,x,b.y+18.0,16.0,rgb(225,233,241),"Inspector");
        let Some(tree)=self.state.tree.clone() else { return; };
        let id=self.state.selected.or(self.state.hovered).unwrap_or(0);
        let node=&tree.nodes[id];
        let mut y=b.y+56.0;
        let width=((b.w-32.0)/7.2).max(12.0) as usize;
        let path=if id==0 {tree.root.to_string_lossy().into_owned()}else{node.relative.to_string_lossy().into_owned()};
        for line in wrap(&path,width).into_iter().take(8) { self.text(cx,x,y,11.0,rgb(190,204,217),&line); y+=18.0; }
        y+=15.0;
        let s=node.stats;
        let kind=node.file.as_ref().map(|f|f.language.as_str()).unwrap_or("Directory");
        for line in [format!("Type         {kind}"),format!("Files        {}",s.files),format!("Lines        {}",s.lines),format!("Non-blank    {}",s.non_blank),format!("Text         {:.1} KiB",s.bytes as f64/1024.0)] {
            self.text(cx,x,y,11.0,rgb(148,170,189),&line); y+=24.0;
        }
        y+=20.0;
        for line in ["Area = non-blank lines.","Comments are included.","UTF-8 text; no AST required.","Source files are read-only."] {
            self.text(cx,x,y,10.0,rgb(102,123,140),line); y+=18.0;
        }
        y+=20.0;
        for line in [format!("Skipped binary: {}",tree.skipped_binary),format!("Skipped large: {}",tree.skipped_large),format!("Read warnings: {}",tree.warning_count)] {
            self.text(cx,x,y,10.0,rgb(119,144,164),&line); y+=19.0;
        }
        if !self.state.query.is_empty() {
            y+=15.0; self.text(cx,x,y,11.0,rgb(184,200,143),&format!("{} path matches",self.state.matches.len())); y+=25.0;
        }
        if let Some(error)=self.state.source_errors.get(&id).cloned() {
            for line in wrap(&error,width).into_iter().take(4) { self.text(cx,x,y,10.0,rgb(219,156,135),&line); y+=17.0; }
        }
        if self.state.loading.contains(&id) { self.text(cx,x,y+5.0,10.0,rgb(146,190,211),"Loading source..."); }
        if y+90.0<b.y+b.h {
            y+=35.0;
            for line in ["Wheel       Zoom at cursor","Drag        Pan","Click       Select","Double-click Focus","Home / F    Fit view / selection"] {
                self.text(cx,x,y,10.0,rgb(109,130,146),line); y+=19.0;
            }
        }
    }
}

impl Widget for CodeMap {
    fn handle_event(&mut self,cx:&mut Cx,event:&Event,_scope:&mut Scope) {
        self.receive(cx);
        match event.hits(cx,self.draw_bg.area()) {
            Hit::FingerDown(e)=>{
                if self.state.viewport.contains(e.abs.x,e.abs.y) {
                    cx.set_key_focus(self.draw_bg.area());
                    let (x,y)=self.state.camera.unproject(e.abs.x,e.abs.y);
                    self.state.selected=self.state.tree.as_ref().and_then(|t|layout::hit_test(t,&self.state.rectangles,x,y));
                    self.state.drag_at=Some(e.abs);
                    if e.tap_count>=2 { self.state.focus_pending=self.state.selected; }
                    self.draw_bg.redraw(cx);
                }
            }
            Hit::FingerMove(e)=>{
                if let Some(last)=self.state.drag_at { self.state.camera.pan(e.abs.x-last.x,e.abs.y-last.y); self.state.drag_at=Some(e.abs); self.draw_bg.redraw(cx); }
            }
            Hit::FingerUp(_)=>{self.state.drag_at=None;}
            Hit::FingerScroll(e)=>{
                if self.state.viewport.contains(e.abs.x,e.abs.y) {
                    self.state.camera.zoom_at((-e.scroll.y*0.006).clamp(-1.0,1.0).exp(),e.abs.x,e.abs.y);
                    self.draw_bg.redraw(cx);
                }
            }
            Hit::FingerHoverIn(e) | Hit::FingerHoverOver(e)=>{
                let (x,y)=self.state.camera.unproject(e.abs.x,e.abs.y);
                let hit=if self.state.viewport.contains(e.abs.x,e.abs.y) { self.state.tree.as_ref().and_then(|t|layout::hit_test(t,&self.state.rectangles,x,y)) } else { None };
                if hit!=self.state.hovered { self.state.hovered=hit; self.draw_bg.redraw(cx); }
            }
            Hit::FingerHoverOut(_)=>{self.state.hovered=None;self.draw_bg.redraw(cx);}
            Hit::KeyDown(e)=>match e.key_code {
                KeyCode::Home=>self.focus(cx,Some(0)),
                KeyCode::KeyF=>self.focus_selection(cx),
                KeyCode::Backspace=>self.parent(cx),
                _=>{}
            },
            _=>{}
        }
    }
    fn draw_walk(&mut self,cx:&mut Cx2d,_scope:&mut Scope,walk:Walk)->DrawStep {
        self.draw_bg.begin(cx,walk,self.layout);
        let outer=cx.turtle().rect();
        let side=(outer.size.x*0.23).clamp(180.0,300.0);
        let view=Box2::new(outer.pos.x,outer.pos.y,(outer.size.x-side).max(1.0),(outer.size.y-27.0).max(1.0));
        if self.state.viewport!=view {
            let previous=self.state.viewport;
            if previous.w>0.0 { self.state.camera.pan((view.w-previous.w)*0.5+view.x-previous.x,(view.h-previous.h)*0.5+view.y-previous.y); }
            self.state.viewport=view;
        }
        if let Some(id)=self.state.focus_pending.take() {
            self.state.camera.fit(self.state.rectangles.get(id).copied().unwrap_or(WORLD),view);
        }
        cx.begin_turtle(Walk { abs_pos:Some(dvec2(view.x,view.y)),width:Size::Fixed(view.w),height:Size::Fixed(view.h),..Walk::default() },Layout {clip_x:true,clip_y:true,..Layout::default()});
        self.draw_map(cx,view);
        cx.end_turtle();
        let panel=Box2::new(view.x+view.w,view.y,side,outer.size.y);
        cx.begin_turtle(Walk {abs_pos:Some(dvec2(panel.x,panel.y)),width:Size::Fixed(panel.w),height:Size::Fixed(panel.h),..Walk::default()},Layout {clip_x:true,clip_y:true,..Layout::default()});
        self.inspector(cx,panel);
        cx.end_turtle();
        let footer=Box2::new(view.x,view.y+view.h,view.w,27.0);
        self.box_fill(cx,footer,rgb(16,22,29));
        let status=shorten(&self.state.status,((view.w-20.0)/7.0).max(1.0) as usize);
        self.text(cx,view.x+10.0,footer.y+6.0,10.0,rgb(126,150,172),&status);
        self.draw_bg.end(cx);
        DrawStep::done()
    }
}

fn rect(b:Box2)->Rect { Rect {pos:dvec2(b.x,b.y),size:dvec2(b.w,b.h)} }
fn rgb(r:u8,g:u8,b:u8)->Vec4 { vec4(r as f32/255.0,g as f32/255.0,b as f32/255.0,1.0) }
fn shade(c:Vec4,v:f32)->Vec4 { vec4(c.x*v,c.y*v,c.z*v,1.0) }
fn tint(language:&str)->Vec4 {
    let hash=language.bytes().fold(0u32,|h,b|h.wrapping_mul(31).wrapping_add(b as u32));
    match hash%8 {0=>rgb(107,168,188),1=>rgb(168,145,192),2=>rgb(171,166,120),3=>rgb(113,171,155),4=>rgb(185,132,138),5=>rgb(123,151,194),6=>rgb(175,152,121),_=>rgb(132,179,181)}
}
fn shorten(value:&str,max:usize)->String {
    if value.chars().count()<=max {value.into()}else if max<2 {String::new()}else{format!("{}…",value.chars().take(max-1).collect::<String>())}
}
fn wrap(value:&str,width:usize)->Vec<String> { value.chars().collect::<Vec<_>>().chunks(width.max(1)).map(|c|c.iter().collect()).collect() }
fn code_geometry(b:Box2,lines:usize)->(usize,usize,Box2,f64) {
    let title=if b.h>35.0 {17.0}else{b.h*0.15};
    let inner=b.inset(3.0);
    let content=Box2::new(inner.x,inner.y+title,inner.w.max(0.1),(inner.h-title).max(0.1));
    let columns=((content.w/content.h*lines.max(1) as f64/72.0).sqrt().round() as usize).clamp(1,128).min(lines.max(1));
    let rows=lines.max(1).div_ceil(columns);
    let font=(content.h/rows as f64*0.54).min(content.w/columns as f64/90.0/0.81);
    (columns,rows,content,font)
}
