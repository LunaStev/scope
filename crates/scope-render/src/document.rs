use crate::{MapPainter,geometry::code_geometry,palette as p};
use makepad_widgets::*;
use scope_analysis::source::{self,Document,Ink};
use scope_core::FileInfo;
use scope_layout::Box2;

impl MapPainter{
    pub(crate) fn draw_preview(&mut self,cx:&mut Cx2d,b:Box2,file:&FileInfo,lines:usize,ink:Vec4){
        let(cols,rows,content,_)=code_geometry(b,lines);let rows=rows.max(1);
        let cell_w=content.w/cols as f64;let line_h=content.h/rows as f64;
        for sample in &file.preview{
            if sample.width==0{continue;}
            let col=sample.line/rows;let row=sample.line%rows;if col>=cols{continue;}
            let x=content.x+col as f64*cell_w+(sample.indent.min(140) as f64/180.0)*cell_w;
            let width=(sample.width.min(140) as f64/180.0*cell_w).max(0.5).min((content.x+(col+1) as f64*cell_w-x).max(0.0));
            self.fill(cx,Box2::new(x,content.y+row as f64*line_h,width,(line_h*0.45).clamp(0.45,2.0)),if sample.comment{p::shade(ink,0.50)}else{ink});
        }
    }
    pub(crate) fn draw_document(&mut self,cx:&mut Cx2d,b:Box2,view:Box2,doc:&Document,budget:&mut usize){
        let(cols,rows,content,font)=code_geometry(b,doc.lines.len());if font<7.0||*budget==0{return;}
        let before=*budget;let col_w=content.w/cols as f64;let row_h=content.h/rows.max(1) as f64;
        let raster_size=font.min(28.0) as f32;let char_w=font*0.81;
        let first_row=(((view.y-content.y)/row_h).floor() as isize).max(0) as usize;
        let last_row=(((view.y+view.h-content.y)/row_h).ceil() as isize).max(0) as usize;
        self.code.text_style.font_size=raster_size;self.code.font_scale=(font/raster_size as f64) as f32;
        for col in 0..cols{
            let x=content.x+col as f64*col_w;if x+col_w<view.x||x>view.x+view.w{continue;}
            for row in first_row..last_row.min(rows){
                let line=col*rows+row;if line>=doc.lines.len()||*budget==0{break;}
                let y=content.y+row as f64*row_h;
                for(column,text,ink)in source::runs(doc.line(line)){
                    if *budget==0{break;}
                    let x=x+column as f64*char_w;if x>=content.x+(col+1) as f64*col_w{break;}
                    if x+text.chars().count() as f64*char_w<view.x{continue;}
                    let max_chars=((content.x+(col+1) as f64*col_w-x)/char_w).floor().max(0.0) as usize;
                    let text:String=text.chars().take(max_chars).collect();if text.is_empty(){continue;}
                    self.code.color=match ink{Ink::Text=>p::rgb(176,195,213),Ink::Keyword=>p::rgb(190,160,219),Ink::Number=>p::unknown(),Ink::String=>p::rgb(150,202,162),Ink::Comment=>p::rgb(104,151,149)};
                    self.code.draw_abs(cx,dvec2(x,y),&text);*budget-=1;
                }
            }
        }
        if std::env::var_os("SCOPE_TRACE").is_some()&&before>*budget{
            eprintln!("scope source: lines={} runs={} font={font:.1}",doc.lines.len(),before-*budget);
        }
    }
}
