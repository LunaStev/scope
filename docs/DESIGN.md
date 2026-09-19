# Compact explorer design

The source map is the primary surface. Avoid large dashboard cards, duplicate branding and controls for actions that do not exist.

## Window hierarchy

- 44 px repository row: Scope, path, Open, Re-index and Details.
- 38 px navigation row: Overview, Up, Read, path search, Area and Source.
- 30 px one-line workspace summary.
- Map filling the remaining body; optional independently scrolling 248–280 px inspector.
- 24 px footer for pending work, detail limits, cached reuse, CPU submission time and the active renderer name.

At widths below 720 px the inspector yields to the map. Numeric values use actual measured text width rather than character-count estimates. Tests capture full-size and 960 x 640 windows.

## Quiet renderer information

The active graphics renderer appears at the far right of the existing footer as 9 pt muted text. It has no badge background, new row, icon, floating overlay or unsolicited popup. Its allocated width never exceeds 240 logical pixels or 28% of the available footer width; long names are ellipsized using actual font measurements. Progress has a separate clipped region. Secondary performance telemetry yields first when the window becomes narrow.

The complete renderer and vendor strings wrap in the inspector's Graphics renderer section, below repository metrics. The app first uses Makepad's cached active-backend information. When native Linux backends leave it unset, a bounded startup probe reads the already-current GLX/EGL context without creating or switching contexts. A successful result is cached; no installed-GPU enumeration, external commands or recurring driver polling is used. Unknown names stay not reported. Recognised software-renderer names remain visible without claiming hardware acceleration.

## Map labels and density

Graphite backgrounds, restrained borders, flat controls and a sage selection accent keep source readable. Palette tokens live in render; native widget styling lives in UI.

`layout::node_frame` allocates the shared world-space header and child/source area. Directory/file annotations use that same header, rather than independently estimating screen height. Counts disappear before overlapping the name; long names are ellipsized within the visible header. Text and edges are clipped before crossing panel boundaries. Zoom does not alter the underlying file/line count.

Source columns take their own actual maximum line width and a three-character gap. Leftover horizontal width is not justified between columns. The column search considers both small counts and a wider set for large files, reducing avoidable vertical underfill. The viewport aspect includes available height, so overview fitting does not retain an obsolete fixed map aspect and leave a large blank strip at the bottom.

## Camera and source

Actual source stays the only code representation. Zoom enlarges the same layout; it does not switch a preview into a reading mode. Double-click targets 12 px under the cursor; Read/F starts at the file beginning; Home returns to overview. Window resizing can recompute scene layout, unlike ordinary zoom.

The uploaded video reference was inspected locally at overview, intermediate zoom and readable-source positions. Its useful design cues here are compact controls, thin hierarchy boundaries, tightly packed source and continuous navigation. Symbol subdivision shown in that reference is not implied to be implemented.

Progressive preparation remains independent of zoom and is labeled in the footer. Bounded detail can stop at the current working-set limit; the footer must distinguish that from pending work rather than spinning forever. See [rendering](RENDERING.md).
