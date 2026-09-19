# Measurement contracts

## Population and accounting

Every statistic is computed from the accepted UTF-8 files at the indexed root, after ignore/exclusion and size-limit rules. Text bytes are source bytes, not disk allocation or Git history. Indexed files are not inferred from rendered tile counts.

`physical lines = code + comment-only + blank + unclassified`

Recognized language classification uses the language service, with a dedicated Wave classifier and pinned Tokei catalogue backend. Unknown non-blank text remains unclassified. Mixed code/comment lines count as code. Empty files have no phantom line, and a trailing newline does not create an additional physical line. Rendering display spans do not override metric classification.

The summary is workspace-wide. The main inspector section describes the selection and its descendants. Language percentages use workspace physical lines as their denominator. Empty denominators show an em dash. Counts use exact integers with grouping separators, and bytes use B/KiB/MiB/GiB.

## Geometry

Area uses non-blank lines, physical lines, classified code lines or text bytes. Empty or zero-weight files receive a minimum visual weight for selection, but reported counts do not include that weight. Zoom and image resolution never change file/line totals. Code columns are fixed world-space layouts; camera movement only transforms them.

## Preparation and drawing

Scan time measures indexing. Map preparation time is separate and includes scene fingerprinting plus cached image decoding or actual-source rasterization through receipt of the prepared root. Warm map reuse does not skip inventory validation of file metadata and layout.

The image-map trace's `source_lines` is the number of physical lines processed into the complete root map (or the matching snapshot's count on a valid cache hit). It is not the number of readable screen lines, sampled glyphs, or separately submitted visible text instances. A failed source read is recorded in map errors, not silently counted as rendered source.

`image_tiles` counts image rectangles composed in the current paint, including the backing overview and available child resolutions. `texture_uploads` is the number of new image textures admitted in that paint, at most one. `cpu_submit_ms` measures CPU drawing/submission, excluding background map preparation and actual GPU completion. None of these fields is FPS.

## Memory

Line-index memory refers to compact source widths. The image compositor keeps a pinned 16 MiB root and at most 96 one-MiB regional textures, by uncompressed image size. This is not total process RAM or GPU allocation: CPU copies, native driver resources, fonts, source-read buffers, floating-point raster coverage and in-flight results are additional. The persistent PNG cache has its own approximate 512 MiB pruning budget.

## Benchmarks

The generated four-million-line fixture runs in a real native window. Reports separate launch-to-first-map, cold/warm cache preparation, per-paint CPU submission and native input-handler latency. The old renderer's first view is partial; the new one waits for its complete overview. They do not have equivalent readiness times, and preparation time must not be hidden in an FPS claim.

Generated 50-million-line and 100,000-file headless fixtures still test indexing separately. Neither these fixtures nor software-OpenGL CI stand in for a real Fuchsia/Chromium checkout on Fedora hardware.
