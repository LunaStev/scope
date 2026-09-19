//! Selection details. Retained independently from the moving source map.
use super::Workspace;
use makepad_widgets::*;
use ::model::format as f;
use ::layout::Box2;
use ::render::palette as p;
impl Workspace {
    fn detail_row(&mut self,cx:&mut Cx2d,b:Box2,y:f64,label:&str,value:&str,color:Vec4) {
        self.chrome.text(cx,b.x+20.0,y,11.0,p::secondary(),label);
        self.chrome.text_right(cx,b.x+b.w-20.0,y,11.0,color,value);
    }
    fn section(&mut self,cx:&mut Cx2d,b:Box2,y:&mut f64,title:&str) {
        *y+=12.0;self.chrome.fill(cx,Box2::new(b.x+20.0,*y,b.w-40.0,1.0),p::border());
        *y+=22.0;self.chrome.text(cx,b.x+20.0,*y,11.0,p::text(),title);*y+=29.0;
    }
    pub(super) fn inspector(&mut self,cx:&mut Cx2d,b:Box2) {
        self.chrome.fill(cx,b,p::panel());self.chrome.fill(cx,Box2::new(b.x,b.y,1.0,b.h),p::border());
        let x=b.x+20.0;let width=(b.w-40.0).max(1.0);let top=b.y-self.state.panel_scroll;let mut y=top+22.0;
        self.chrome.text(cx,x,y,11.0,p::muted(),"Selection");y+=30.0;
        let Some(tree)=self.state.session.snapshot.as_ref().map(|s|s.tree.clone())else{
            self.chrome.text(cx,x,y,12.0,p::secondary(),"Open a source folder to begin.");self.state.panel_height=100.0;return;
        };
        let id=self.state.session.selected.unwrap_or(0).min(tree.nodes.len()-1);let node=&tree.nodes[id];let s=node.stats;
        self.chrome.text(cx,x,y,16.0,p::text(),&f::shorten(&node.name,(width/10.7) as usize));y+=29.0;
        let kind=node.file.as_ref().map(|f|f.language.as_str()).unwrap_or("Folder");
        self.chrome.fill(cx,Box2::new(x,y+3.0,5.0,9.0),p::language(kind));
        self.chrome.text(cx,x+13.0,y,10.0,p::secondary(),&f::shorten(kind,28));y+=29.0;
        let path=if id==0{tree.root.to_string_lossy().into_owned()}else{node.relative.to_string_lossy().into_owned()};
        for line in f::wrap(&path,(width/6.6) as usize).into_iter().take(3){self.chrome.text(cx,x,y,10.0,p::muted(),&line);y+=17.0;}
        self.section(cx,b,&mut y,"Metrics");
        for (label,value,color) in [
            ("Files",f::count(s.files),p::text()),("Lines",f::count(s.lines),p::text()),
            ("Code",f::count(s.code),p::text()),("Comments",f::count(s.comments),p::secondary()),
            ("Blank",f::count(s.blanks),p::secondary()),("Unclassified",f::count(s.unclassified),if s.unclassified>0{p::unknown()}else{p::muted()}),
            ("Text size",f::bytes(s.bytes),p::text()),
        ] {self.detail_row(cx,b,y,label,&value,color);y+=26.0;}
        y+=4.0;
        let mut offset=0.0;self.chrome.fill(cx,Box2::new(x,y,width,3.0),p::border());
        for (value,color) in [(s.code,p::accent()),(s.comments,p::comment()),(s.blanks,p::blank()),(s.unclassified,p::unknown())] {
            let w=if s.lines>0{value as f64/s.lines as f64*width}else{0.0};
            self.chrome.fill(cx,Box2::new(x+offset,y,w,3.0),color);offset+=w;
        }
        y+=17.0;
        self.chrome.text(cx,x,y,9.0,p::muted(),"Physical lines; inline comments count as code.");y+=17.0;
        self.section(cx,b,&mut y,"Workspace languages");
        for lang in tree.languages.iter().take(5) {
            self.chrome.fill(cx,Box2::new(x,y+3.0,5.0,9.0),p::language(&lang.language));
            self.chrome.text(cx,x+13.0,y,11.0,p::secondary(),&f::shorten(&lang.language,20));
            self.chrome.text_right(cx,b.x+b.w-20.0,y,11.0,p::muted(),&f::percent(lang.stats.lines,tree.totals().lines));y+=26.0;
        }
        if tree.languages.len()>5 {
            let other:u64=tree.languages.iter().skip(5).map(|l|l.stats.lines).sum();
            self.detail_row(cx,b,y,"Other",&f::percent(other,tree.totals().lines),p::muted());y+=26.0;
        }
        self.section(cx,b,&mut y,"Index");
        let r=&tree.report;
        for(label,value)in[
            ("Scan time",f::duration(r.elapsed_ms)),("Binary skipped",f::count(r.skipped_binary)),
            ("Oversized skipped",f::count(r.skipped_large)),("Read warnings",f::count(r.warning_count)),
            ("Source allocation",f::bytes(tree.source_memory_bytes as u64)),
        ]{self.detail_row(cx,b,y,label,&value,p::muted());y+=26.0;}
        for warning in r.warnings.iter().take(2) {
            for line in f::wrap(warning,(width/6.5) as usize).into_iter().take(3){self.chrome.text(cx,x,y,9.0,p::unknown(),&line);y+=17.0;}
        }
        y+=13.0;self.chrome.text(cx,x,y,9.0,p::muted(),"Local snapshot · source files unchanged");y+=32.0;
        self.state.panel_height=y-top;
        if self.state.panel_height>b.h {
            let height=(b.h*b.h/self.state.panel_height).max(24.0);
            let offset=self.state.panel_scroll/(self.state.panel_height-b.h).max(1.0)*(b.h-height);
            self.chrome.fill(cx,Box2::new(b.x+b.w-3.0,b.y+offset,2.0,height),p::border());
        }
    }
}
