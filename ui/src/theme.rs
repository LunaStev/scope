use makepad_widgets::*;
live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    pub ScopeButton = <Button> {
        padding: {left: 10, right: 10, top: 7, bottom: 7}
        draw_text: {color: #b9c1ca, text_style: {font_size: 10}}
        draw_bg: {
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos*self.rect_size);
                sdf.box(0.0,0.0,self.rect_size.x,self.rect_size.y,3.0);
                sdf.fill(mix(#1b2025,#303840,self.hover));
                return sdf.result;
            }
        }
    }
    pub ScopeInput = <TextInput> {
        padding: {left: 10, right: 10, top: 7, bottom: 7}
        draw_text: {color: #c6ced7, text_style: {font_size: 10}}
        draw_bg: {
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos*self.rect_size);
                sdf.box(0.5,0.5,self.rect_size.x-1.0,self.rect_size.y-1.0,3.0);
                sdf.fill_keep(#13171b); sdf.stroke(#2b3239,1.0);
                return sdf.result;
            }
        }
    }
}
