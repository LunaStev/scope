#[derive(Clone,Copy,Debug,Default,PartialEq)]
pub struct Box2{pub x:f64,pub y:f64,pub w:f64,pub h:f64}
impl Box2{
    pub const fn new(x:f64,y:f64,w:f64,h:f64)->Self{Self{x,y,w,h}}
    pub fn area(self)->f64{self.w.max(0.0)*self.h.max(0.0)}
    pub fn contains(self,x:f64,y:f64)->bool{x>=self.x&&y>=self.y&&x<=self.x+self.w&&y<=self.y+self.h}
    pub fn intersects(self,b:Self)->bool{self.x<b.x+b.w&&b.x<self.x+self.w&&self.y<b.y+b.h&&b.y<self.y+self.h}
    pub fn intersection(self,b:Self)->Option<Self>{let x=self.x.max(b.x);let y=self.y.max(b.y);let w=(self.x+self.w).min(b.x+b.w)-x;let h=(self.y+self.h).min(b.y+b.h)-y;if w>0.0&&h>0.0&&w.is_finite()&&h.is_finite(){Some(Self::new(x,y,w,h))}else{None}}
    pub fn inset(self,p:f64)->Self{Self::new(self.x+p,self.y+p,(self.w-2.0*p).max(0.0),(self.h-2.0*p).max(0.0))}
}
pub const WORLD:Box2=Box2::new(0.0,0.0,4096.0,2560.0);
