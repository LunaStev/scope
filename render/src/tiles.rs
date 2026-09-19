//! Fixed source tiles are cache units, never alternate visual representations.
use ::layout::{Box2,SourceLayout,source::GLYPH_ADVANCE};
use std::ops::Range;
pub const TILE_ROWS:usize=48;
pub const TILE_COLUMNS:usize=192;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub struct TileKey{pub file:usize,pub column:usize,pub row:usize,pub horizontal:usize}
#[derive(Clone,Debug)]
pub struct SourceTile{pub key:TileKey,pub lines:Range<usize>,pub chars:Range<usize>,pub bounds:Box2}
pub fn visible_tiles(file:usize,page:&SourceLayout,lines:usize,world_view:Box2,out:&mut Vec<SourceTile>){
    if lines==0||!page.content.intersects(world_view){return;}
    let advance=page.font_size*GLYPH_ADVANCE;
    for(col,column)in page.columns.iter().enumerate(){
        let x=page.content.x+column.offset;
        let width=column.characters as f64*advance;
        if x+width<world_view.x||x>world_view.x+world_view.w{continue;}
        let count=lines.saturating_sub(col*page.rows).min(page.rows);
        let first_row=(((world_view.y-page.content.y)/page.line_height).floor().max(0.0)as usize).saturating_sub(2).min(count)/TILE_ROWS;
        let last_row=(((world_view.y+world_view.h-page.content.y)/page.line_height).ceil().max(0.0)as usize).saturating_add(2).min(count).div_ceil(TILE_ROWS);
        let first_x=(((world_view.x-x)/advance).floor().max(0.0)as usize).saturating_sub(2).min(column.characters)/TILE_COLUMNS;
        let last_x=(((world_view.x+world_view.w-x)/advance).ceil().max(0.0)as usize).saturating_add(2).min(column.characters).div_ceil(TILE_COLUMNS);
        for row in first_row..last_row{
            let first=row*TILE_ROWS;let last=(first+TILE_ROWS).min(count);
            for horizontal in first_x..last_x{
                let left=horizontal*TILE_COLUMNS;let right=(left+TILE_COLUMNS).min(column.characters);
                out.push(SourceTile{key:TileKey{file,column:col,row,horizontal},lines:col*page.rows+first..col*page.rows+last,chars:left..right,
                    bounds:Box2::new(x+left as f64*advance,page.content.y+first as f64*page.line_height,(right-left)as f64*advance,(last-first)as f64*page.line_height)});
            }
        }
    }
}
pub fn slice(text:&str,skip:usize,take:usize)->&str{
    if text.is_ascii(){let a=skip.min(text.len());return &text[a..a.saturating_add(take).min(text.len())];}
    let a=text.char_indices().nth(skip).map_or(text.len(),|(i,_)|i);let tail=&text[a..];
    let b=tail.char_indices().nth(take).map_or(tail.len(),|(i,_)|i);&tail[..b]
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn tiles_cover_every_character_once_without_zoom(){
        let n=1200;let width=450;let page=SourceLayout::new(Box2::new(0.0,0.0,1000.0,600.0),n,width);
        let mut tiles=Vec::new();visible_tiles(1,&page,n,page.content,&mut tiles);
        assert_eq!(tiles.iter().map(|t|t.lines.len()*t.chars.len()).sum::<usize>(),n*width);
        let unique:std::collections::HashSet<_>=tiles.iter().map(|t|t.key).collect();assert_eq!(unique.len(),tiles.len());
    }
    #[test]fn utf8_and_long_line_tiles_do_not_truncate(){
        assert_eq!(slice("a안녕b",1,2),"안녕");assert_eq!(slice("a안녕b",50,2),"");
        let text="x".repeat(1500);let joined=(0..1500).step_by(TILE_COLUMNS).map(|i|slice(&text,i,TILE_COLUMNS)).collect::<String>();assert_eq!(joined,text);
    }
    #[test]fn small_view_visits_a_subset(){
        let p=SourceLayout::new(Box2::new(0.0,0.0,1000.0,600.0),1200,400);let mut all=Vec::new();let mut part=Vec::new();
        visible_tiles(1,&p,1200,p.content,&mut all);visible_tiles(1,&p,1200,Box2::new(p.content.x,p.content.y,1.0,1.0),&mut part);
        assert!(!part.is_empty()&&part.len()<all.len());
    }
}
