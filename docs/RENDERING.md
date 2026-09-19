# Retained source rendering

## Ownership and lifetime

`runtime::Snapshot` owns the immutable tree, metric-derived rectangles and fixed packed source layouts. Every snapshot has a monotonically assigned generation. `render::GeometryCache` owns native draw lists keyed by `(node, column, block)` for that generation and display DPI. A new scene/DPI or live renderer style invalidates these lists. No filesystem read or lexical analysis occurs in a draw pass.

`SourceLayout::for_document` measures each line from prepared source runs. It evaluates a small set of column counts, uses the local maximum width within each column, and places the next column after that width plus three character cells. Remaining width stays at the outer edge; columns are not justified across the page. Camera movement never changes this arrangement.

## Cold and warm paths

A cold source block contains up to 128 consecutive lines from one column. Its glyph instances are built at canonical font size 16 in chunk-local coordinates and retained in a Makepad `DrawList2d`. The source shader uses a per-list affine transform, clipping rectangle and dim factor. Local coordinates keep zoom from magnifying large global-coordinate rounding errors.

The warm path appends the existing list and updates its uniforms. It does not call `DrawText::draw_abs` again for that block, does not slice UTF-8 strings, and does not upload a fresh instance buffer because the camera moved. Map backgrounds, annotations and controls still have their own CPU work. This optimization is not a claim that every frame is constant-time.

Files, individual packed columns and 128-line blocks are viewport-culled. A partly visible block may submit extra lines clipped by the GPU. Trace `lines` is therefore the number of submitted source lines, not the exact count of readable on-screen rows.

Cold work yields between blocks once approximately 8 ms have elapsed and at least one block has been built. One large block can exceed that budget. A next-frame request continues pending blocks until they are ready, even with no input. There is no zoom-dependent source threshold, sampling, preview pass, bitmap replacement or permanently omitted tail of the repository. Pending geometry is explicitly shown in the footer.

## Shader contract

Only retained source lists use the packed `view_transform` encoding in `render/src/cache.rs`. The source vertex shader reconstructs screen origin and scale, clips in screen space and adjusts UV coordinates after the transform. Source dimming uses a spare matrix lane. The UI's ordinary text and shape draw lists do not use this encoding.

Retained instances depend on the current font atlas and Makepad renderer lifetime. Snapshot/DPI/style invalidation is covered; device-loss/recreation behavior still depends on the underlying Makepad backend and should be tested on each supported platform.

## Memory

Source text and prepared runs remain in the snapshot. Glyph instances are retained for blocks visited in that snapshot, potentially the entire repository when viewing its overview. This trades instance memory for much less repeated CPU work and upload traffic. It is not a fixed 32 MiB cache and the UI must not imply otherwise. The Source allocation counter excludes the draw-list cache, GPU allocation and other process memory.

## Validation and benchmark

The GUI test requires all 1,200 real source lines at sub-7 px font size before zoom input and then readable double-click without reflow. It verifies cached reuse with zero new source blocks, exercises blank files, and captures full/compact windows.

`scripts/performance.py` compares release builds on the same runner, using 24 files with 500 physical lines each and 24 small pan events after warm-up. Every measured sample must submit the same 12,000 source lines; optimized samples must rebuild zero blocks. JSON output reports the median and 95th percentile CPU map-submission times. These are not GPU completion times or FPS, do not include cold preparation, and do not establish performance on the user's hardware.
