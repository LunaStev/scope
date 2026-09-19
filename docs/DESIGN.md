# Design and navigation

The visual hierarchy is repository controls, navigation, summary cards, map context, the spatial map with inspector, and operational footer. Palette tokens live in `render/src/palette.rs`; native control styling lives in `ui/src/theme.rs`. The inspector scrolls independently and can be hidden.

## One source representation

Actual source text is present at overview scale. Do not add a miniature-bars pass, sampled-line substitute, zoom threshold, or a second layout for reading mode. A file's `SourceLayout` is world-space geometry computed once when the scene is indexed or its area metric changes. The camera applies scale and translation only. A microscopic glyph naturally covers less than a pixel; it is still the same glyph, not a preview primitive.

Text columns depend on file shape, line count and source width, never camera zoom. Source loading and lexical preparation happen before snapshot publication, not inside drawing. Only off-screen geometry is culled. Text layer visibility is a user toggle, not an automatic level-of-detail switch.

## Camera controls

Wheel zoom is cursor-anchored with logarithmic sensitivity 0.022. Double-click targets a readable 12 px source size at the pointer, preserving that world-space anchor. `F`/Read focuses a file's first column at readable scale. Directories fit normally; Home restores the repository overview. These operations do not change glyph positions or column layout.

## Validation

GUI regression tests must show real source glyph submissions before the first zoom event, including fonts below 7 px. They must also test a double-click directly reaching readable scale. Screenshots come from the actual native app. Runtime tests verify source snapshot stability and layout tests verify camera scaling without reflow.
