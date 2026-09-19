use crate::{MapPainter, palette as p};
use makepad_widgets::*;
use ::model::{Document, Ink};
use ::layout::{Box2, Camera, SourceLayout, source::GLYPH_ADVANCE};

/// Return a Unicode-safe portion of a run intersecting the viewport.
fn visible_slice(text: &str, skip: usize, take: usize) -> &str {
    let start = text.char_indices().nth(skip).map_or(text.len(), |(i,_)|i);
    let tail = &text[start..];
    let end = tail.char_indices().nth(take).map_or(tail.len(), |(i,_)|i);
    &tail[..end]
}

impl MapPainter {
    pub(crate) fn draw_document(&mut self, cx: &mut Cx2d, page: SourceLayout, view: Box2, camera: Camera, doc: &Document, dim: f32) -> (usize,usize) {
        let content = camera.project(page.content);
        if !content.intersects(view) || doc.lines.is_empty() { return (0,0); }
        let font = page.font_size * camera.scale;
        let row_h = page.line_height * camera.scale;
        let col_w = page.column_width * camera.scale;
        let char_w = font * GLYPH_ADVANCE;
        // One atlas size and one rendering path; even subpixel source stays text.
        self.code.text_style.font_size = 16.0;
        self.code.font_scale = (font / 16.0) as f32;
        let first_row = (((view.y-content.y)/row_h).floor().max(0.0) as usize).saturating_sub(1);
        let last_row = (((view.y+view.h-content.y)/row_h).ceil().max(0.0) as usize).min(page.rows);
        let first_col = (((view.x-content.x)/col_w).floor().max(0.0) as usize).min(page.columns);
        let last_col = (((view.x+view.w-content.x)/col_w).ceil().max(0.0) as usize).min(page.columns);
        let mut line_count = 0;
        let mut run_count = 0;
        for col in first_col..last_col {
            let x = content.x + col as f64 * col_w;
            for row in first_row..last_row {
                let index = col * page.rows + row;
                let Some(runs) = doc.runs.get(index) else { break; };
                let y = content.y + row as f64 * row_h;
                line_count += 1;
                for run in runs {
                    let left = x + run.column as f64 * char_w;
                    if left >= view.x+view.w { break; }
                    let skip = (((view.x-left)/char_w).floor().max(0.0) as usize).saturating_sub(1);
                    let draw_x = left + skip as f64 * char_w;
                    let take = (((view.x+view.w-draw_x)/char_w).ceil().max(0.0) as usize).saturating_add(1);
                    let text = visible_slice(&run.text, skip, take);
                    if text.is_empty() { continue; }
                    self.code.color = p::shade(match run.ink {
                        Ink::Text=>p::rgb(176,195,213),Ink::Keyword=>p::rgb(190,160,219),
                        Ink::Number=>p::unknown(),Ink::String=>p::rgb(150,202,162),Ink::Comment=>p::rgb(104,151,149)
                    },dim);
                    self.code.draw_abs(cx,dvec2(draw_x,y),text);
                    run_count += 1;
                }
            }
        }
        if std::env::var_os("SCOPE_TRACE").is_some() && run_count>0 {
            eprintln!("scope source: lines={line_count} runs={run_count} font={font:.4} representation=source");
        }
        (line_count,run_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn viewport_slice_does_not_split_utf8() {
        assert_eq!(visible_slice("a안녕b",1,2),"안녕");
        assert_eq!(visible_slice("a안녕b",50,2),"");
    }
}
