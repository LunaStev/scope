use makepad_widgets::*;
live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    pub ScopeButton = <Button> {
        padding: {left: 11,right: 11,top: 8,bottom: 8}
        draw_text: {color: #b7bfcd, text_style: {font_size: 11}}
        draw_bg: {
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0.5,0.5,self.rect_size.x-1.0,self.rect_size.y-1.0,4.0);
                sdf.fill_keep(mix(#181b21,#292f3a,self.hover));
                sdf.stroke(mix(#292d36,#404a5b,self.hover),1.0);
                return sdf.result;
            }
        }
    }
    pub ScopePrimary = <ScopeButton> {
        draw_text: {color: #c8d8f6}
    }
    pub ScopeInput = <TextInput> {
        padding: {left: 11,right: 11,top: 8,bottom: 8}
        draw_text: {color: #d0d5df,text_style: {font_size: 11}}
        draw_bg: {
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0.5,0.5,self.rect_size.x-1.0,self.rect_size.y-1.0,4.0);
                sdf.fill_keep(#111318);
                sdf.stroke(#2b303b,1.0);
                return sdf.result;
            }
        }
    }
}
