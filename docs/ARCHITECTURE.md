# Root module architecture

Each major module sits directly at the repository root; directory, package and import names agree. Only the executable package in `app/` is named `scope`.

| Module | Ownership |
| --- | --- |
| model | Tree, counts, file revision stamps, compact source shapes and resident documents |
| language | Catalogue, JSON definition validation, detection, line classifiers, display lexers |
| analysis | Ignore policy, parallel traversal, bounded reads, progress and unchanged-record reuse |
| layout | Treemap, shared node headers, viewport fit, packed source pages, camera and hit testing |
| runtime | Background jobs, scene generations and bounded document residency |
| render | Retained glyph tiles, clipping, labels and native GPU resources |
| ui | Toolbar, summary, scrollable inspector and input routing |
| app | CLI parsing and headless reports |

`check_architecture.py` enforces dependency direction. Model, language, analysis, layout and runtime do not import Makepad. Scanning and lexical preparation are background work, not draw-pass work.

## Pipeline

1. Parallel scanner workers read accepted text, classify through language services and build a compact `SourceShape`. A bounded queue of 32 results feeds a single tree owner; this avoids keeping an unbounded pile of full-file results.
2. Initial indexing discards source/token strings after measurement. The tree retains line widths, block maxima, file stamps and statistics. Throttled progress events let the UI report actual indexed file/line counts without pretending to know the total in advance.
3. Layout derives treemap rectangles and fixed source pages from those shapes. The viewport aspect is part of scene layout. The same header/content allocator drives subdivision and labels.
4. Runtime publishes an immutable versioned snapshot. Visible source is prepared by at most two background jobs, verified against the indexed revision and admitted to bounded resident storage.
5. Rendering prepares small retained glyph tiles from resident documents. Warm camera movement reuses native draw lists; cold work is scheduled progressively at any zoom level. UI and CLI read the same measurement model.

## Reuse and cancellation

Opening another repository replaces the result receiver and cancels the old scan. Results from the previous document cache cannot populate the new cache. Re-index shares unchanged source-shape records when path, file size and modification time match. The filesystem and ignore policy are still traversed, so deletions and new files are considered. Reuse is within the same process; no persistent cache has been implemented.

Resizing or changing the area basis creates a new scene generation and invalidates geometry. Ordinary pan/zoom never recomputes source column layout. Source documents can be evicted independently of retained geometry because draw lists own their prepared glyph instances.

## Memory ownership

The line index grows with the input, but repository-wide expanded token strings are not stored. `runtime/residency.rs` provides LRU storage with frame pinning; document and geometry services have separate accounting policies. Visible geometry is pinned before cold admission so repeated misses do not evict the same current-frame working set. See [rendering](RENDERING.md) for exact accounting limits and what they exclude.

## Extension boundaries

New language definitions/classifiers belong under language; AST adapters under analysis with spans in model. Layout strategies and coordinate rules belong under layout. Persistent index services belong under runtime/analysis. Render passes belong under render and window composition under UI. Do not put these into a monolithic app entry or workspace widget.
