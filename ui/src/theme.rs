use makepad_widgets::*;
live_design!{
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    pub ScopeButton=<Button>{
        padding:{left:13,right:13,top:9,bottom:9}
        draw_text:{color:#c4d3e4,text_style:{font_size:11}}
        draw_bg:{
            fn pixel(self)->vec4{
                let sdf=Sdf2d::viewport(self.pos*self.rect_size);
                sdf.box(0.5,0.5,self.rect_size.x-1.0,self.rect_size.y-1.0,4.0);
                sdf.fill_keep(#192638);sdf.stroke(#304154,1.0);
                return sdf.result;
            }
        }
    }
    pub ScopeInput=<TextInput>{
        padding:{left:12,right:12,top:9,bottom:9}
        draw_text:{color:#cfdeed,text_style:{font_size:11}}
    }
}
