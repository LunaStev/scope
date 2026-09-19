use super::Workspace;
use makepad_widgets::*;
use ::model::format as f;
use ::layout::Box2;
use ::render::palette as p;
impl Workspace{
    pub(super) fn dashboard(&mut self,cx:&mut Cx2d,b:Box2){
        self.chrome.fill(cx,b,p::background());
        let tree=self.state.session.snapshot.as_ref().map(|s|s.tree.clone());
        let data=if let Some(tree)=tree{
            let s=tree.totals();
            vec![
                ("TEXT FILES",f::count(s.files),format!("{} indexed folders",f::count(tree.nodes.len().saturating_sub(s.files as usize) as u64)),p::accent()),
                ("PHYSICAL LINES",f::count(s.lines),"All indexed text, including blanks".into(),p::text()),
                ("CODE LINES",f::count(s.code),if s.unclassified>0{format!("{} non-blank unclassified",f::count(s.unclassified))}else{"Language-aware classification".into()},if s.unclassified>0{p::unknown()}else{p::accent()}),
                ("TEXT SIZE",f::bytes(s.bytes),format!("Indexed in {}",f::duration(tree.report.elapsed_ms)),p::text()),
            ]
        }else{
            vec![("TEXT FILES","—".into(),"Awaiting index".into(),p::muted()),("PHYSICAL LINES","—".into(),"No snapshot loaded".into(),p::muted()),("CODE LINES","—".into(),"No fabricated totals".into(),p::muted()),("TEXT SIZE","—".into(),"Local files only".into(),p::muted())]
        };
        let gap=12.0;let width=((b.w-40.0-3.0*gap)/4.0).max(1.0);
        for(i,(title,value,caption,accent))in data.iter().enumerate(){
            let tile=Box2::new(b.x+20.0+i as f64*(width+gap),b.y+12.0,width,86.0);
            self.chrome.fill(cx,tile,p::border());self.chrome.fill(cx,tile.inset(1.0),p::card());
            self.chrome.fill(cx,Box2::new(tile.x+1.0,tile.y+1.0,2.0,tile.h-2.0),*accent);
            self.chrome.text(cx,tile.x+15.0,tile.y+12.0,9.0,p::muted(),title);
            let font=((width-30.0)/(value.chars().count().max(1) as f64*0.82)).min(26.0).max(10.0) as f32;
            self.chrome.text(cx,tile.x+15.0,tile.y+32.0,font,p::text(),value);
            self.chrome.text(cx,tile.x+15.0,tile.y+66.0,9.0,*accent,&f::shorten(caption,((width-30.0)/6.0).max(1.0) as usize));
        }
    }
}
