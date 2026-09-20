# Complete backing and live source rendering

## One fixed code layout

The complete accepted codebase contributes to a pinned overview image. Actual source and line/column positions are shared by the image and the native foreground; zoom never reflows or invents source. The 2048-square overview is not discarded while detail is being prepared. Areas without prepared foreground retain their existing image, not a hole.

Near the camera, retained native Makepad signed-distance-field glyphs are composited over the image. This avoids enlarging a bitmap as the only readable-code renderer. Foreground preparation starts at a nominal 1.5 physical-pixel font density in a 12% viewport halo. Its opacity smoothly increases from 3 to 6 physical pixels; newly prepared chunks fade in over 90 ms. These are presentation densities, not a separate reading screen or a code-reflow threshold. DPI participates in the decision.

Each native tile carries an interior background and its glyphs. The mask is clipped to the tile's actual column width, so it neither doubles bitmap ink nor erases an adjacent tightly packed column. The complete image still exists underneath it. Folder labels, selection, search outlines and counts remain independent native annotations.

## Discover and retain, rather than rescan

A balanced immutable `layout::spatial::SourceIndex` is built with each snapshot on a background worker. Branches retain geometric bounds and maximum nominal font size. Foreground queries skip off-screen and too-small-text subtrees instead of scanning hundreds of thousands of files each paint. The optional native query admits at most 256 pages and 8,192 branch visits; the image still covers the entire repository.

A cursor enumerates visible and nearby 16-line by 96-character source tiles. Native source documents are prepared by at most two background jobs. Cold GPU glyph preparation retains the existing whole-map admission limits: at most two tiles, 128 checks and an approximately four-millisecond soft CPU guard per paint. A single font operation is not preemptible; this is not a hard frame-time guarantee.

Prepared tiles keep native draw lists and glyph instances. Warm movement replays resident geometry with updated transforms/clips rather than slicing source or reshaping every character. ASCII source windows use borrowed byte slices; non-ASCII windows preserve UTF-8 boundaries. Discovery is repeated for a changed view, but unchanged views do not restart their queues.

A very large visible foreground can exhaust its native detail accounting budget. This is a quality limit for the optional foreground, not a reason to remove the complete image. Source errors likewise leave the existing backing available. Not every possible view is precomputed.

## Faster regional image preparation

The overview still processes every accepted physical source line, without the old glyph-count cutoff. Regional images now visit only intersecting columns and rows. A worker-owned 64 MiB / 64-entry source cache stores revision-checked text, line offsets and lexical checkpoints every 128 lines. A region can resume comment state near its first visible row rather than reparsing from file start, and neighbouring regions reuse the same source read.

The cache validates size/modification time before reuse; it does not silently mix revisions. Unknown languages still use conservative metric classification. Display coloring is a separate lexical aid. Checkpointing preserves cross-line block comments. Image and native font paths are not claimed to be pixel-identical for all scripts, complex shaping or color-emoji fallback.

## Image hierarchy and persistence

`raster::TileKey` addresses world subdivisions. Detail images are 512 square pixels; requests cover the viewport, a surrounding halo and a small deeper central region. Existing parents remain while children arrive. Image admission stays limited to one texture upload per paint: 16 MiB for the root or one MiB per detail tile.

One persistent raster worker owns its font/cache state. Obsolete queued camera requests are replaced; an executing request may finish and become reusable. Result transfer and decoded-image queues each have capacity two. Cancellation isolates scene generations. Successful map images persist in the existing private cache; failures are not written as complete maps.

Cache identity includes root, source paths, size/mtime, geometry, font bytes and map format, not hashes of every source's contents. Re-index after edits. Images contain source content and stay local: `SCOPE_MAP_CACHE_OFF=1` disables persistence, and `SCOPE_MAP_CACHE_DIR` selects a dedicated private directory. The approximate 512 MiB disk budget applies to owned map-image files.

## Memory accounting

The complete overview and at most 96 image details represent about 112 MiB of uncompressed image data. The native foreground has a one-million-glyph / 4,096-entry geometry accounting limit and a 128 MiB / 256-document source budget. The raster worker has a separate 64 MiB source-index cache.

These are not total process or VRAM caps. Active reads, CPU/GPU image copies, font atlases, native draw-list allocations, spatial indices, raster buffers and retired objects are additional. Cold native preparation allocates more memory than an image-only scene in return for live scalable text. A settled idle view does not request continuous redraws.

## Verification and telemetry

`source_lines` remains the root map's processed physical-line count. `native_glyphs` counts retained foreground characters submitted in a paint (not separately visible pixels); `native_resident` is its glyph-accounting size. `built` counts cold native tiles; `native_reused` counts reused native draw lists. `image_tiles` and `uploads` remain separate image-composition measurements.

GUI regression tests require the complete backing before zoom, actual native glyphs at reading scale, preserved backing through enlargement, cached inward zoom without new glyph builds, cached restart and revision invalidation. A release benchmark exercises readable code and repeated zoom separately from overview composition. Generated 50-million-line GUI and 100,000-file headless tests continue to cover different stages.

CPU submission excludes background work and GPU completion. Input measurements end at application input handling, not display scan-out. None is FPS or a real Fuchsia/Chromium hardware result. Realtime here describes camera rendering; automatic filesystem watching is not implemented.
