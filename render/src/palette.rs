//! Semantic tokens shared by map and shell.
use makepad_widgets::{Vec4,vec4};
pub fn rgb(r:u8,g:u8,b:u8)->Vec4{vec4(r as f32/255.0,g as f32/255.0,b as f32/255.0,1.0)}
pub fn shade(c:Vec4,v:f32)->Vec4{vec4(c.x*v,c.y*v,c.z*v,1.0)}
pub fn background()->Vec4{rgb(17,21,24)}
pub fn panel()->Vec4{rgb(23,27,31)}
pub fn card()->Vec4{rgb(27,32,37)}
pub fn border()->Vec4{rgb(42,48,54)}
pub fn text()->Vec4{rgb(219,225,231)}
pub fn secondary()->Vec4{rgb(163,175,188)}
pub fn muted()->Vec4{rgb(120,133,146)}
pub fn accent()->Vec4{rgb(147,190,180)}
pub fn comment()->Vec4{rgb(162,143,210)}
pub fn blank()->Vec4{rgb(88,113,144)}
pub fn unknown()->Vec4{rgb(196,169,126)}
pub fn language(name:&str)->Vec4{match name{
    "Wave"=>rgb(112,191,182),"Rust"=>rgb(196,153,124),"C"|"C Header"|"C++"|"C++ Header"=>rgb(129,155,193),
    "Python"=>rgb(188,177,133),"JavaScript"|"JSX"=>rgb(190,178,120),"TypeScript"|"TSX"=>rgb(125,161,194),"Go"=>rgb(124,178,185),
    "Java"|"Kotlin"=>rgb(169,150,187),"JSON"|"TOML"|"YAML"=>rgb(136,174,158),"Markdown"|"HTML"|"CSS"=>rgb(150,160,185),
    s if s.starts_with("Unknown")||s=="Text / unknown"||s=="Text"=>unknown(),
    _=>{let h=name.bytes().fold(0u32,|h,b|h.wrapping_mul(31).wrapping_add(b as u32));match h%4{0=>rgb(140,177,173),1=>rgb(165,151,190),2=>rgb(167,174,138),_=>rgb(138,160,188)}}
}}
