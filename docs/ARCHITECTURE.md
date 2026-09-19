# Root module architecture

Each major module sits directly at the repository root. Its package and import names match its directory. The executable in `app/` is still called `scope`.

| Module | Owns |
| --- | --- |
| `model` | `Tree`, `Stats`, `Document`, source runs and formatting |
| `analysis` | File policy, bounded reads, metric classification and text preparation |
| `layout` | Treemap, camera, hit-testing and fixed packed source pages |
| `runtime` | Background jobs, unique scene generations and immutable snapshot publication |
| `render` | Retained source geometry, viewport clipping, palette and GPU resources |
| `ui` | Compact shell, one-line summary, inspector and input routing |
| `app` | CLI parsing and report output |

Dependency direction is enforced by `scripts/check_architecture.py`. Model, analysis, layout and runtime do not import Makepad. Filesystem traversal and lexical analysis never run in drawing methods.

## Index-to-frame flow

1. Analysis reads each accepted file once, measures it, prepares its complete source, and stores an `Arc<Document>` in the file node.
2. Runtime publishes a snapshot containing the tree, treemap rectangles and packed per-file source pages. Column widths use actual content rather than equal divisions of available space.
3. The renderer identifies intersecting files, columns and source blocks. A new block prepares retained glyph geometry once. Subsequent navigation reuses its draw list and changes camera/clipping uniforms.
4. UI composes the map with a compact summary and selected-node details. UI and CLI consume the same measured statistics.

Opening a repository replaces the result receiver and cancels the earlier scan. An area-metric change shares source documents but allocates a new scene generation and source layout. Camera movement does not rebuild columns, read files or switch source representation.

## Lifetime and memory

The snapshot retains source and prepared lexical display strings. `Tree::source_memory_bytes` accounts for their string/run/index allocations, not the whole process or GPU atlas.

The renderer additionally owns retained 128-line source blocks for the active scene. These are invalidated on scene generation, DPI or live renderer-style changes. This cache deliberately trades instance memory for reduced CPU work and buffer uploads; it has no fixed total memory ceiling. The inspector does not conflate its source allocation number with GPU memory. See RENDERING.md for the cache and shader contracts.

## UI ownership

`ui/src/app.rs` composes controls and routes actions. `workspace/frame.rs` places surfaces, `summary.rs` formats the single-line workspace totals, `inspector.rs` renders details, and `input.rs` handles camera/selection and independent panel scrolling. `render/src/cache.rs` owns cache keys/lifetime, not the application widget. Geometry preparation requests another frame only while visible blocks are pending.

## Extension ownership

Symbol analysis belongs under analysis with span types in model. Spatial strategies belong under layout, GPU passes under render, background index services under runtime, and shell orchestration under UI. Keep these dependencies explicit rather than growing a monolithic process entry or widget file.
