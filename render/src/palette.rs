//! Restrained neutral chrome; color belongs primarily to source and selection.
use makepad_widgets::{Vec4,vec4};
pub fn rgb(r:u8,g:u8,b:u8)->Vec4{vec4(r as f32/255.0,g as f32/255.0,b as f32/255.0,1.0)}
pub fn shade(c:Vec4,v:f32)->Vec4{vec4(c.x*v,c.y*v,c.z*v,1.0)}
pub fn background()->Vec4{rgb(17,19,23)}
pub fn panel()->Vec4{rgb(22,24,29)}
pub fn card()->Vec4{rgb(26,29,35)}
pub fn border()->Vec4{rgb(42,46,55)}
pub fn text()->Vec4{rgb(221,224,231)}
pub fn secondary()->Vec4{rgb(161,169,183)}
pub fn muted()->Vec4{rgb(114,125,142)}
pub fn accent()->Vec4{rgb(142,172,240)}
pub fn comment()->Vec4{rgb(170,157,200)}
pub fn blank()->Vec4{rgb(90,102,120)}
pub fn unknown()->Vec4{rgb(204,177,133)}
pub fn language(name:&str)->Vec4{
    match name {
        "Rust"=>rgb(196,157,132),"C"|"C Header"|"C++"|"C++ Header"=>rgb(131,164,206),
        "Python"=>rgb(191,183,130),"JavaScript"|"JSX"=>rgb(196,185,122),
        "TypeScript"|"TSX"=>rgb(129,167,205),"Go"=>rgb(119,177,187),
        "Java"|"Kotlin"=>rgb(174,151,193),"JSON"|"TOML"|"YAML"=>rgb(141,179,165),
        "Markdown"|"HTML"|"CSS"=>rgb(158,169,195),
        s if s.starts_with("Unknown")||s=="Text / unknown"||s=="Text"=>unknown(),
        _=>{let h=name.bytes().fold(0u32,|h,b|h.wrapping_mul(31).wrapping_add(b as u32));
            match h%4{0=>rgb(142,181,176),1=>rgb(167,154,194),2=>rgb(171,179,139),_=>rgb(142,167,194)}}
    }
}
