//! Shared semantic color tokens. Panels and language legends use the same map.
use makepad_widgets::{Vec4,vec4};
pub fn rgb(r:u8,g:u8,b:u8)->Vec4{vec4(r as f32/255.0,g as f32/255.0,b as f32/255.0,1.0)}
pub fn shade(c:Vec4,v:f32)->Vec4{vec4(c.x*v,c.y*v,c.z*v,1.0)}
pub fn background()->Vec4{rgb(11,16,24)}
pub fn panel()->Vec4{rgb(16,23,34)}
pub fn card()->Vec4{rgb(21,30,43)}
pub fn border()->Vec4{rgb(39,53,70)}
pub fn text()->Vec4{rgb(231,238,246)}
pub fn secondary()->Vec4{rgb(158,178,200)}
pub fn muted()->Vec4{rgb(112,137,161)}
pub fn accent()->Vec4{rgb(102,218,196)}
pub fn comment()->Vec4{rgb(162,143,210)}
pub fn blank()->Vec4{rgb(88,113,144)}
pub fn unknown()->Vec4{rgb(221,175,105)}
pub fn language(name:&str)->Vec4{
    match name {
        "Rust"=>rgb(221,166,126),"C"|"C Header"|"C++"|"C++ Header"=>rgb(118,164,225),
        "Python"=>rgb(221,200,120),"JavaScript"|"JSX"=>rgb(221,195,96),
        "TypeScript"|"TSX"=>rgb(106,171,226),"Go"=>rgb(104,204,219),
        "Java"|"Kotlin"=>rgb(189,146,210),"JSON"|"TOML"|"YAML"=>rgb(132,194,174),
        "Markdown"|"HTML"|"CSS"=>rgb(155,167,215),
        s if s.starts_with("Unknown")||s=="Text / unknown"||s=="Text"=>unknown(),
        _=>{let h=name.bytes().fold(0u32,|h,b|h.wrapping_mul(31).wrapping_add(b as u32));
            match h%4{0=>rgb(130,196,190),1=>rgb(177,155,218),2=>rgb(185,192,133),_=>rgb(128,174,220)}}
    }
}
