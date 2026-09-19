use super::Workspace;
use makepad_widgets::*;
use ::model::format as f;
use ::layout::Box2;
use ::render::palette as p;
impl Workspace{
    pub(super) fn inspector(&mut self,cx:&mut Cx2d,b:Box2){
        self.chrome.fill(cx,b,p::panel());self.chrome.fill(cx,Box2::new(b.x,b.y,1.0,b.h),p::border());
        let x=b.x+16.0;let right=b.x+b.w-16.0;let width=b.w-32.0;
        self.state.panel_scroll=self.state.panel_scroll.clamp(0.0,(self.state.panel_height-b.h).max(0.0));
        let top=b.y-self.state.panel_scroll;let mut y=top+18.0;
        let Some(tree)=self.state.session.snapshot.as_ref().map(|s|s.tree.clone())else{self.chrome.text(cx,x,y,11.0,p::secondary(),"Details");return;};
        let id=self.state.session.selected.unwrap_or(0).min(tree.nodes.len()-1);let node=&tree.nodes[id];let stats=node.stats;
        self.chrome.text(cx,x,y,13.0,p::text(),&f::shorten(&node.name,(width/9.0) as usize));y+=26.0;
        let path=if id==0{tree.root.display().to_string()}else{node.relative.display().to_string()};
        for line in f::wrap(&path,(width/7.0) as usize).into_iter().take(4){self.chrome.text(cx,x,y,10.0,p::muted(),&line);y+=17.0;}y+=12.0;
        let kind=node.file.as_ref().map(|f|f.language.as_str()).unwrap_or("Directory");self.chrome.text(cx,x,y,10.0,p::language(kind),kind);y+=30.0;
        for(label,value,color)in[("Files",f::count(stats.files),p::text()),("Lines",f::count(stats.lines),p::text()),("Code",f::count(stats.code),p::accent()),("Comments",f::count(stats.comments),p::secondary()),("Blank",f::count(stats.blanks),p::secondary()),("Unclassified",f::count(stats.unclassified),p::unknown()),("Text",f::bytes(stats.bytes),p::text()),("Share of lines",f::percent(stats.lines,tree.totals().lines),p::secondary())]{self.chrome.text(cx,x,y,10.0,p::muted(),label);self.chrome.text_right(cx,right,y,10.0,color,&value);y+=23.0;}
        y+=10.0;self.chrome.fill(cx,Box2::new(x,y,width,1.0),p::border());y+=22.0;self.chrome.text(cx,x,y,10.0,p::text(),"Languages");y+=20.0;self.chrome.text(cx,x,y,9.0,p::muted(),"Workspace · physical lines");y+=24.0;
        for lang in tree.languages.iter().take(7){self.chrome.fill(cx,Box2::new(x,y+3.0,5.0,8.0),p::language(&lang.language));self.chrome.text(cx,x+13.0,y,10.0,p::secondary(),&f::shorten(&lang.language,21));self.chrome.text_right(cx,right,y,10.0,p::muted(),&f::percent(lang.stats.lines,tree.totals().lines));y+=23.0;}
        if tree.languages.len()>7{self.chrome.text(cx,x,y,9.0,p::muted(),&format!("{} more languages",tree.languages.len()-7));y+=23.0;}
        y+=10.0;self.chrome.fill(cx,Box2::new(x,y,width,1.0),p::border());y+=22.0;self.chrome.text(cx,x,y,10.0,p::text(),"Index");y+=25.0;
        let r=&tree.report;
        for(label,value)in[("Scan time",f::duration(r.elapsed_ms)),("Line index",f::bytes(tree.source_memory_bytes as u64)),("Resident source",f::bytes(self.state.session.documents.bytes() as u64)),("Reused files",f::count(r.reused_files)),("Warnings",f::count(r.warning_count))]{self.chrome.text(cx,x,y,10.0,p::muted(),label);self.chrome.text_right(cx,right,y,10.0,p::secondary(),&value);y+=23.0;}
        for line in [format!("Skipped: {} binary, {} oversized",r.skipped_binary,r.skipped_large),format!("{} links · {} explicit exclusions",r.skipped_links,r.pruned_entries)]{self.chrome.text(cx,x,y,9.0,p::muted(),&f::shorten(&line,(width/6.0) as usize));y+=19.0;}
        for warning in r.warnings.iter().take(2){for line in f::wrap(warning,(width/6.5) as usize).into_iter().take(3){self.chrome.text(cx,x,y,9.0,p::unknown(),&line);y+=17.0;}}
        if let Some(error)=self.state.session.documents.error(id).map(str::to_owned){for line in f::wrap(&error,(width/6.5) as usize){self.chrome.text(cx,x,y,9.0,p::unknown(),&line);y+=17.0;}}
        y+=20.0;self.chrome.text(cx,x,y,9.0,p::muted(),"Code + comments + blank + unclassified");y+=17.0;self.chrome.text(cx,x,y,9.0,p::muted(),"= physical lines. Counts are not estimates.");y+=32.0;
        self.state.panel_height=y-top;
        if self.state.panel_height>b.h{let h=(b.h*b.h/self.state.panel_height).max(24.0);let offset=self.state.panel_scroll/(self.state.panel_height-b.h).max(1.0)*(b.h-h);self.chrome.fill(cx,Box2::new(b.x+b.w-4.0,b.y+offset,2.0,h),p::muted());}
    }
}
