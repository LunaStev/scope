# Persistent source-image rendering

## The visual contract

The complete accepted codebase contributes to the initial map, rather than stopping when a glyph cache fills. After that map is available, every navigation frame has the existing image as its backing. An unavailable higher-resolution image must not erase it. Source line/column positions remain fixed during pan and zoom.

The backing image is rasterized from actual characters, not line-length bars. Resolution changes only sampling density. Source is not switched from an abstract representation into text. The current CPU rasterizer uses the existing Liberation Mono font dependency and generic display spans; line classification remains the language service's separate authority. Font coverage/fallback and complex shaping are not claimed to match every script's native editor.

## Preparation and ownership

The initial index retains exact counts, source revisions and compact per-line widths. When a matching scene is published, `runtime::maps` starts a cancellable worker that constructs a scene fingerprint and tries the private disk cache. On a miss `raster::Rasterizer` traverses every intersecting file, validates its revision, streams source lines, and accumulates filtered glyph coverage into the overview. There is no source-file/glyph count cutoff.

The 2048-square overview is published only after this pass finishes. Progress is throttled. File-read failures are shown as warnings and are not persisted as a supposedly complete cached map. Initial preparation can therefore take longer than indexing; a warm cache avoids re-reading/rasterizing unchanged source for the overview.

One persistent worker owns its rasterizer. Camera requests replace obsolete queued regions; an already executing request may finish and become useful cache data. Results cross a channel of capacity two, and the UI also retains at most two decoded images awaiting upload. Closing or replacing a scene cancels its worker and isolates stale results.

## Resolution hierarchy and GPU path

`raster::TileKey` addresses world-space subdivisions. A detail tile is 512 square pixels. Desired resolution is selected from the projected world density, with a screen-sized region, a surrounding halo and a small deeper central prefetch. Work enumeration is capped rather than walking the entire resolution level.

`render::ImageMap` always retains the root texture. It draws cached parents before children and clips UVs in f64 before passing coordinates to the GPU. New children fade in over 120 ms without removing their parents. A cold sudden jump can be temporarily soft; a cached move uses existing textures immediately. This is not a promise of precomputed infinite zoom or instantaneous detail at every possible location.

The UI admits at most one image upload per paint. Root upload is 16 MiB; regional uploads are one MiB. A completed idle map does not redraw continuously. Live annotations remain bounded native text, and hovering/selection use small outlines. No per-source-character shaping or glyph buffer construction occurs in a navigation frame.

## Persistence

PNG images use scene fingerprint + tile coordinate filenames. The fingerprint includes file size/mtime, root, source paths, geometry, font and implementation format revision. It is not content-hash validation of every file. A scene geometry change invalidates that scene's images. Cache writes use a private directory and atomic temporary-file replacement; corrupt, oversized, incorrectly sized or invalid-format images are misses. Only the cache's own image filenames are pruned to its approximate 512 MiB budget.

These images can reveal source content. They never leave the machine as part of normal application operation. `SCOPE_MAP_CACHE_OFF` disables persistence; `SCOPE_MAP_CACHE_DIR` selects a dedicated private directory.

## Memory and limits

One root plus at most 96 detail images represents about 112 MiB of uncompressed image data. Native CPU copies and GPU copies may both exist. The raster worker's float coverage buffer, temporary text, font data, scene geometry, queued results, driver resources and cache encoding are additional allocations. Neither the image limit nor the disk limit describes total RAM/VRAM.

The previous native glyph tile implementation is retained internally but is not the active source rendering path. Its old one-million-glyph limit does not apply to overview image coverage.

## Verification and telemetry

`source_lines` in source-image traces is the number of physical source lines processed into the root image, not the number of separately submitted visible glyphs. `image_tiles` counts composed image rectangles. `texture_uploads` counts newly supplied image textures in that paint. CPU submission excludes worker rasterization and GPU completion; `map_prepare_ms` includes fingerprint/cache lookup or raster preparation up to receipt.

GUI tests cover a complete 1,200-line map before zoom, preserved backing through uncached enlargement, a readable-scale detail image, zero navigation-time source glyph construction, persistent restart reuse and edited-file invalidation. The four-million-line native benchmark records cold and warm preparation separately from navigation/input. Headless large-index tests measure a different stage. No result is described as monitor FPS or a real Chromium/Fuchsia hardware benchmark.
