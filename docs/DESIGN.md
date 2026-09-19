# Compact explorer design

The source map is the primary surface. Avoid turning a spatial explorer into a dashboard of large statistic cards, repeating the brand in several places or adding controls for unimplemented actions.

## Window hierarchy

- 44 px repository row: Scope, editable path, Open, Re-index and Details.
- 38 px navigation row: Overview, Up, Read, path search, Area and Source.
- 30 px summary line: exact workspace file/line/code totals and text size.
- Map occupying the remaining body, with an optional 248–280 px detail panel.
- 24 px operational footer: input hints, pending preparation, cached batch reuse and CPU submission time.

The detail panel scrolls independently. At widths below 720 px it yields to the map. The 960 x 640 compact window is included in GUI regression captures. Numeric values use measured glyph width for right alignment, not an estimate from string length.

## Visual language

Use quiet graphite backgrounds, restrained separators, small flat controls, subdued language colors and a single sage selection accent. Color tokens live in `render/src/palette.rs`; native controls in `ui/src/theme.rs`. Comments, unknown text and classification categories remain distinguishable without making every section a bright card.

Directory padding and inter-file gutters are compact. Source columns use their own actual width plus a three-character-cell gap. No extra horizontal space is distributed among columns. A very long line affects only its column rather than every column of the file.

## Source and camera

There is one actual-source representation, including at microscopic scale. Cached glyph geometry is enlarged by camera scale/translation; reading does not switch modes or reflow lines. Double-click reaches the readable 12 px target under the cursor. F/Read starts at the file's beginning, directories fit normally, and Home restores the overview.

Progressive cold geometry preparation is independent of zoom and is labeled in the footer. It must continue without input until ready. It is not a hidden source-visibility threshold. See RENDERING.md for ownership, memory and validation.
