# Bounded continuous-source rendering

## Ownership

A `Snapshot` owns the tree, file revision stamps, compact per-line width indices, metric-derived rectangles and fixed source pages. It does not retain repository-wide source/token strings. `Documents` prepares actual source on background workers at any zoom level and retains a bounded working set. `GeometryCache` owns native draw lists keyed by `(node, column, block, tile)`, scene generation and display DPI.

Indexing, source preparation and GPU geometry residency are separate. The first map does not wait for the whole repository's glyphs. A missing resident source is pending work, not an instruction to switch from bars into source at a zoom threshold. No alternative representation is substituted.

## Geometry and clipping

`layout::node_frame` is the single header/content allocator used by both treemap subdivision and annotations. File and directory names/counts are measured, ellipsized and clipped within that region. Counts are immutable model values. Rectangle edges are clipped in f64 before GPU float conversion, avoiding synthetic borders on the viewport edge at large zoom.

`world_for_viewport` accounts for the usable map width and height and the overview's 12 px margin. Resize may rebuild the layout; pan/zoom only transforms an existing scene. Source columns keep local maximum line widths plus three character cells of gap. Candidate column counts span small counts and sampled larger counts so a narrow search around average line width does not unnecessarily leave a large blank bottom.

Source aspect and line lengths still affect unused space inside an individual file; text is not stretched or clipped to pretend every rectangle is completely full.

## Cold / warm paths

Each cold glyph tile covers at most 32 lines and 128 character columns. A very long source line is split horizontally rather than creating one unbounded draw job. Instances are built at canonical size 16 and retain chunk-local coordinates. Warm camera movement appends the existing list and changes only camera/clipping/dimming uniforms.

Cold work has an approximately 5 ms soft CPU preparation budget, with at most two new tiles per file in one pass. A tile can exceed this budget on slow machines; this is not a hard frame deadline. Fair file ordering prevents the first large file from consuming all cold preparation. Completed idle views stop requesting redraws.

Off-screen/subpixel file rectangles, off-screen columns and off-screen tiles are culled. The visible working set is pinned before eviction. When it exceeds the geometry budget, the footer reports a limit and stops unproductive automatic rebuild attempts. Focusing another/smaller region makes that region's detail eligible. The renderer does not promise to draw every character of a fifty-million-line repository simultaneously.

## Residency accounting

- Documents: 128 MiB accounted source/run capacity, at most 256 entries and two active preparation workers.
- Glyph geometry: one million glyphs and 4,096 retained draw-list entries; glyph count is not a byte measurement.
- File read limit: eight MiB by default, configurable independently.
- Index: approximately four bytes per physical-line width plus block maxima, file/tree records and layout data. It scales with indexed input.

In-flight preparation, renderer/font atlases, native allocations, GPU copies and other process state are additional memory. UI counters must distinguish line index, resident source and glyph count; none is advertised as a total-memory cap. Size/mtime checks reject source revisions that differ from the indexed record; same-size edits with deliberately preserved timestamps are not detected by a content hash.

## Validation

GUI tests exercise overview source before zoom, readable double-click, retained reuse, zoom round trips, blank files and window resizing. Separate generated stress fixtures measure only indexing, layout and same-process metadata reuse. Peak Linux RSS in those headless tests excludes a window and resident glyph/source preparation; it cannot establish full-app GPU performance on Chromium or Fuchsia.
