# Navigation working-set optimization

This change preserves the complete source-image backing and native SDF foreground. It does not lower source resolution, remove indexed files, or add a new display mode.

## Shared spatial geometry

`layout::navigation::NodeIndex` is an immutable balanced bounding-volume hierarchy built with each snapshot. It serves point picking, screen-space header discovery and background regional image preparation. A directory with a very large child list no longer requires checking all siblings on every pointer movement. Header queries carry maximum-header-height bounds, skip unreadable subtrees and resume within the existing work allowance.

Raster queries preserve the original depth-first paint order after spatial filtering. They have no node-count cap: the complete overview still includes every accepted source file. The index itself consumes additional CPU memory and construction time. Snapshot page and rectangle arrays are now Arc slices; the raster worker shares them and the node index instead of deep-cloning every packed source column on startup.

## Stable native preparation

Native source discovery uses a 20% halo on each side and prepares one font-density octave ahead. Moving within that world-space coverage continues the existing source cursors rather than starting over each pixel. An increased detail requirement, a different scene/DPI, a coverage miss, a truncated candidate list or exhausted residency causes a new query. Existing foreground opacity and source positions are unchanged.

## Latest-view image scheduling

Image requests are compared by desired tiles, resident tiles and result revision. Repainting unchanged work does not drain/rebuild the worker queue. A camera change sets a cancellation token on a running detail that is no longer requested; the rasterizer checks that token during node/row/character processing. Such cancellation is not a source error and the tile remains retryable. A still-useful active tile is allowed to finish.

The pinned overview is never cancelled by a camera-only request. Scene replacement still cancels the whole worker. The backing remains visible while detail arrives. Obsolete decoded refinements are discarded before consuming a GPU upload slot. Cancellation is cooperative, not a hard latency bound: an OS read, font operation or PNG encode/decode already executing is not preempted.

## Validation

`cargo run --release -p layout --example navigation` compares picking and regional traversal against linear reference algorithms on 250,000 sibling nodes, with exact result agreement and operation-count checks. Timings are subsystem microbenchmarks, not full application frame rates or real Fuchsia measurements. GUI tests still verify complete overview coverage, zoom, native glyph reuse, persistent caching and 50-million-line input.

Optional trace fields `native_queries`, `map_replans` and `cancelled_details` expose discovery/coalescing/cancellation without adding visible UI controls.
