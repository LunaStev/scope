# Root module architecture

Each major module sits directly at the repository root. Its package and import names match its directory. The executable in `app/` is still called `scope`.

| Module | Owns |
| --- | --- |
| `model` | `Tree`, `Stats`, `Document`, source text runs and formatting |
| `analysis` | File policy, bounded reads, metric classification and text preparation |
| `layout` | Treemap, camera, hit-testing and fixed `SourceLayout` pages |
| `runtime` | Background jobs and immutable `Snapshot` publication |
| `render` | Visible-source drawing, shared palette and GPU resources |
| `ui` | Shell, controls, dashboard, inspector and input routing |
| `app` | CLI parsing and report output |

Dependency direction is enforced by `scripts/check_architecture.py`. Model, analysis, layout and runtime must not import Makepad. The renderer does not walk the filesystem or lex strings during a frame.

## Index-to-frame flow

1. `analysis` reads each accepted file once, measures it, prepares its complete source text, and stores an `Arc<Document>` in the file node.
2. `runtime` publishes a snapshot with the tree, treemap rectangles, and fixed per-file source pages computed by `layout`.
3. `render` traverses intersecting rectangles and emits actual source glyph runs using the page geometry projected through the camera.
4. `ui` adds repository statistics and selected-node annotations, using the same model as the CLI.

A new repository replaces the active result receiver and cancels the earlier scan. Rebuilding an area layout shares the same source documents. Camera navigation never rebuilds source columns, starts file reads or changes the text representation.

## Lifetime and memory

Source and lexical display strings are retained for the lifetime of the snapshot; this deliberately replaces the former tiny preview cache. `Tree::source_memory_bytes` accounts for string and run/index allocations, not the entire process or GPU atlas. The scanner's per-file input limit still applies. Source shown after zoom belongs to the indexed revision even if the underlying file changes meanwhile.

## Extension ownership

Add symbol analysis under analysis and expose spans through model. Add spatial algorithms under layout and GPU passes under render. Background indexing services belong under runtime. Keep shell orchestration in UI and process startup in app; no single file becomes the owner of unrelated functionality.
