# Architecture

Root packages are `app`, `model`, `language`, `analysis`, `layout`, `raster`, `runtime`, `render` and `ui`. Only the executable is named `scope`. Exact direct dependency edges are enforced by `scripts/check_architecture.py`. Headless modules do not depend on Makepad; the executable composes services instead of owning scanning or draw loops.

## Ownership

Analysis produces an immutable tree, exact compact line widths and revisions using bounded parallel workers. Language owns detection and metric rules. Layout creates fixed-world source columns and hierarchy rectangles, plus an immutable balanced index for querying nearby readable pages. Session publishes only matching viewport/metric generations.

Raster owns glyph filtering, actual-source images, resolution tiles, private image persistence and revision-checked raw-source reuse. Detail sources store line offsets and lexical checkpoints. Raster depends on no windowing API; existing host font bytes are supplied rather than vendored or discovered arbitrarily.

Runtime maps own cancellation, scene lifetime, one persistent image worker, priority queues and bounded decoded-image transfer. Rasterization, source reads, PNG encode/decode and cache pruning run there. Runtime documents independently own bounded lexical preparation for native text.

Render owns GPU textures and retained native SDF source tiles. ImageMap pins the complete backing even when detail is absent. `live` selects foreground pages from the spatial index; `schedule` admits bounded missing native tiles; `document` builds and replays their GPU resources. Cold preparation and warm replay are separate. Native eviction cannot remove the underlying map. Folder annotations and selection outlines remain separate.

UI retains compact controls and status/inspector behavior. CLI inventory constructs no maps and needs no display. All statistics come from shared model measurements, not rendered glyph or pixel counts.

## Extension boundaries

A faster raster worker or GPU preprocessing can replace preparation without changing layout or complete-backing semantics. Incremental persistent per-file updates can extend scene caching. AST adapters belong in language/analysis and publish spans through model. The UI is not the owner of language heuristics, disk-cache policy, spatial queries or per-character iteration.

See RENDERING.md for budgets, cold/warm behavior, privacy and measurement scope.
