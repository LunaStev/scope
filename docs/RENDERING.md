# Bounded continuous-source rendering

## Ownership

A snapshot owns measured tree data, compact source widths and fixed world-space pages. Source text and lexical runs are prepared on background workers with a 128 MiB resident-document accounting budget. The renderer retains actual glyph geometry; it does not swap text for a line-bar or thumbnail representation at a zoom threshold.

## First paint is separate from indexing

Finishing the index must not trigger a full synchronous walk of every source tile. Presentation now uses a resumable DFS (at most 2,048 edge/node visits per pass), progressively retained node batches and separately scheduled labels (at most 256 candidates per pass). A directory with huge fan-out never pushes all its children onto a stack at once.

Each box is a single fill/border shader instance, rather than five quads. Background and measured-label draw lists are keyed by scene, camera, viewport, DPI and search query. Source-ready events reuse those lists. Selection and hover are lightweight border overlays and do not invalidate the static layers.

The first layout is published only after its aspect and area metric match the latest requested viewport. An obsolete initial layout is not briefly displayed, populated with source and immediately discarded.

## Cold source admission

`layout::tiles::TileCursor` produces visible 32-line by 128-character tiles lazily. Ready, pending and worker-waiting file queues are distinct. Completed reads are promoted directly to the ready queue. Missing tails are not scanned to compute an exact remaining-block count. The footer reports queued regions instead.

A preparation pass admits at most 128 candidate checks, two new glyph tiles and 8,192 source character instances across the entire map, not per file. A roughly four-millisecond elapsed CPU guard applies before the first tile too. Time is a soft guard: native font layout/rasterization and a single admitted tile are not preemptible. This is not a GPU frame deadline.

Only resident tiles are replayed and pinned. Missing-tile discovery does not happen again just to render cached tiles. Geometry eviction is attempted only when the corresponding source document is ready. Full I/O capacity is reported as `Full`, not as a newly queued read. When reads are the only possible progress, their completion signal wakes the UI rather than a busy redraw loop.

Document memory accounting runs on preparation workers. Heavy evicted token allocations are released off the event thread. Camera changes retain glyph tiles while restarting the bounded visibility/cold-work cursors for the new region.

## Continuous geometry

Columns use their own longest line plus three character cells of separation. Ordinary zoom changes only the camera, not line/column positions. Shared node-frame geometry controls both folder allocation and clipped names/counts. Screen clipping is done in f64 before GPU conversion and does not invent edges at viewport cuts.

Resize may recompute the world aspect. The overview includes its 12-pixel margin. Source aspect and line lengths can still leave some space within an individual file; text is not distorted to force complete area occupancy.

## Accounting and limits

The geometry cache retains at most one million glyphs and 4,096 tile entries. Node discovery retains the existing 50,000 visible-node safety limit. Detail-budget exhaustion stops futile automatic rebuilding and is shown in the footer; focusing a smaller region changes the eligible working set. Limits are accounting bounds, not total process RAM or GPU-memory caps.

Snapshots, in-flight source preparation, font atlases, native allocations, retired data and GPU copies require additional memory. Same-process index reuse is based on size/mtime, not a persistent content-hashed index.

## Verification

Unit tests cover cursor suspension, huge line widths, admission limits, explicit I/O capacity and initial-aspect publication. GUI regressions preserve real source before zoom, zoom round trips, clipping, blank files and compact windows. The first-paint benchmark separately measures the post-index map submission path and input-handler latency during detail population. These are not FPS or GPU-completion measurements, and generated fixtures are not Chromium/Fuchsia checkouts.
