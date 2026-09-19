use super::Workspace;
use makepad_widgets::*;
use scope_core::format as f;
use scope_layout::Box2;
use scope_render::palette as p;

impl Workspace{
    pub(super) fn inspector(&mut self,cx:&mut Cx2d,b:Box2){
        self.chrome.fill(cx,b,p::panel());self.chrome.fill(cx,Box2::new(b.x,b.y,1.0,b.h),p::border());
        let x=b.x+20.0;let width=(b.w-40.0).max(1.0);let top=b.y-self.state.panel_scroll;
        let mut y=top+18.0;
        self.chrome.text(cx,x,y,10.0,p::accent(),"INSPECTOR");y+=30.0;
        let Some(tree)=self.state.session.snapshot.as_ref().map(|s|s.tree.clone())else{
            self.chrome.text(cx,x,y,11.0,p::secondary(),"Select a file or directory");return;
        };
        let id=self.state.session.selected.unwrap_or(0).min(tree.nodes.len()-1);let node=&tree.nodes[id];let stats=node.stats;
        self.chrome.text(cx,x,y,17.0,p::text(),&f::shorten(&node.name,(width/11.5) as usize));y+=30.0;
        let path=if id==0{tree.root.to_string_lossy().into_owned()}else{node.relative.to_string_lossy().into_owned()};
        for line in f::wrap(&path,(width/7.0) as usize).into_iter().take(4){self.chrome.text(cx,x,y,10.0,p::muted(),&line);y+=17.0;}
        y+=12.0;
        let language=node.file.as_ref().map(|f|f.language.as_str()).unwrap_or("Directory");
        self.chrome.fill(cx,Box2::new(x,y,5.0,12.0),p::language(language));
        self.chrome.text(cx,x+13.0,y,10.0,p::secondary(),&f::shorten(language,(width/7.0) as usize));y+=32.0;
        for(label,value,color)in[
            ("Files",f::count(stats.files),p::text()),
            ("Physical lines",f::count(stats.lines),p::text()),
            ("Code lines",f::count(stats.code),p::accent()),
            ("Comment-only lines",f::count(stats.comments),p::comment()),
            ("Blank lines",f::count(stats.blanks),p::secondary()),
            ("Unclassified lines",f::count(stats.unclassified),p::unknown()),
            ("Text size",f::bytes(stats.bytes),p::text()),
            ("Share of indexed lines",f::percent(stats.lines,tree.totals().lines),p::secondary()),
        ]{
            self.chrome.text(cx,x,y,10.0,p::secondary(),label);
            self.chrome.text(cx,b.x+b.w-20.0-value.chars().count() as f64*7.3,y,10.0,color,&value);y+=24.0;
        }
        y+=8.0;
        self.chrome.fill(cx,Box2::new(x,y,width,6.0),p::border());
        let mut offset=0.0;
        for(value,color)in[(stats.code,p::accent()),(stats.comments,p::comment()),(stats.blanks,p::blank()),(stats.unclassified,p::unknown())]{
            let w=if stats.lines>0{value as f64/stats.lines as f64*width}else{0.0};
            self.chrome.fill(cx,Box2::new(x+offset,y,w,6.0),color);offset+=w;
        }
        y+=18.0;
        self.chrome.text(cx,x,y,9.0,p::muted(),"Code + comments + blank + unclassified");y+=18.0;
        self.chrome.text(cx,x,y,9.0,p::muted(),"Mixed code/comment lines count as code.");y+=33.0;
        self.chrome.fill(cx,Box2::new(x,y,width,1.0),p::border());y+=22.0;
        self.chrome.text(cx,x,y,10.0,p::accent(),"WORKSPACE LANGUAGES");y+=20.0;
        self.chrome.text(cx,x,y,9.0,p::muted(),"Share of all indexed physical lines");y+=25.0;
        for lang in tree.languages.iter().take(6){
            self.chrome.fill(cx,Box2::new(x,y+3.0,7.0,7.0),p::language(&lang.language));
            self.chrome.text(cx,x+16.0,y,10.0,p::secondary(),&f::shorten(&lang.language,22));
            let percent=f::percent(lang.stats.lines,tree.totals().lines);
            self.chrome.text(cx,b.x+b.w-20.0-percent.len() as f64*7.3,y,10.0,p::text(),&percent);y+=24.0;
        }
        if tree.languages.len()>6{
            let rest:u64=tree.languages.iter().skip(6).map(|l|l.stats.lines).sum();
            self.chrome.text(cx,x,y,9.0,p::muted(),&format!("+ {} others · {}",tree.languages.len()-6,f::percent(rest,tree.totals().lines)));y+=23.0;
        }
        y+=10.0;self.chrome.fill(cx,Box2::new(x,y,width,1.0),p::border());y+=23.0;
        self.chrome.text(cx,x,y,10.0,p::accent(),"INDEX HEALTH");y+=25.0;
        let report=&tree.report;
        for line in[
            format!("{} binary / non-UTF-8 skipped",f::count(report.skipped_binary)),
            format!("{} oversized · {} symlinks skipped",f::count(report.skipped_large),f::count(report.skipped_links)),
            format!("{} explicit entries pruned",f::count(report.pruned_entries)),
            format!("{} read warnings",f::count(report.warning_count)),
            format!("File read limit: {}",f::bytes(self.state.session.options.max_file_bytes)),
        ]{self.chrome.text(cx,x,y,10.0,p::secondary(),&line);y+=21.0;}
        if let Some(error)=self.state.session.cache.error(id){
            for line in f::wrap(error,(width/7.0) as usize).into_iter().take(4){self.chrome.text(cx,x,y,10.0,p::unknown(),&line);y+=19.0;}
        }
        for warning in report.warnings.iter().take(3){
            for line in f::wrap(warning,(width/7.0) as usize).into_iter().take(3){self.chrome.text(cx,x,y,9.0,p::unknown(),&line);y+=17.0;}
        }
        y+=22.0;self.chrome.text(cx,x,y,9.0,p::muted(),"Snapshot only · original files never changed");y+=30.0;
        self.state.panel_height=y-top;
        if self.state.panel_height>b.h{
            let h=(b.h*b.h/self.state.panel_height).max(24.0);
            let offset=self.state.panel_scroll/(self.state.panel_height-b.h).max(1.0)*(b.h-h);
            self.chrome.fill(cx,Box2::new(b.x+b.w-4.0,b.y+offset,2.0,h),p::muted());
        }
    }
}
