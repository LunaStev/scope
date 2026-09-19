# Architecture

## Dependency direction

```text
apps/scope (CLI / process entry)
  +-- scope-analysis ------> scope-core
  +-- scope-ui
        +-- scope-runtime --> scope-analysis + scope-layout + scope-core
        +-- scope-render ---> scope-runtime + scope-layout + scope-core
        +-- scope-layout ---> scope-core
```

`scope-core`, `scope-analysis`, `scope-layout` and `scope-runtime` must remain free of GUI dependencies. No library depends on `apps/scope`. `scope-render` does not import the application shell. The dependency audit in CI records the allowed graph explicitly.

## Data flow

1. The CLI or UI supplies a path and `ScanOptions`.
2. `scope-analysis` traverses the local tree, applies filters, bounded-reads text and produces an immutable `Tree` plus `ScanReport`.
3. `scope-layout` derives weighted world rectangles from that tree. It uses a per-file visibility floor and deterministic path tie-breakers.
4. `scope-runtime` publishes a `Snapshot` through a result channel. The UI polls only completed results; opening a new root replaces the receiver and cancels the old scan.
5. `scope-render` projects visible rectangles and submits the background batch before labels. At readable zoom it requests source through runtime services, not direct filesystem calls.
6. `scope-ui` composes the renderer with dashboard, inspector and footer. Numeric values come from `scope-core::Stats`; UI components do not re-count source.

## Internal modules

- Analysis: `scanner` owns traversal, `options` owns policy, `metrics` owns the Tokei adapter, `preview` owns line samples, `source` owns bounded reads/document indexing/lexical display hints.
- Layout: `geometry`, `camera`, `squarify`, `hierarchy`.
- Runtime: `session` owns navigation, result channels and snapshots; `cache` owns document loading, limits and eviction.
- Render: `painter` owns GPU resources, `map` owns map passes, `document` owns source LOD, `geometry` owns display geometry, `palette` owns semantic colors.
- UI: `app` composes widgets and routes control actions; `theme` styles native controls; `workspace/{frame,dashboard,inspector,input}` keeps unrelated responsibilities apart.
- Application: `main` selects CLI/GUI, `args` validates input, `report` formats headless output.

## Concurrency and lifetime

Workers receive cloned options and immutable paths/trees. The desktop injects a function-pointer wake callback; runtime never imports a windowing toolkit. Source cache results use a receiver unique to the active cache, preventing old repository results from being inserted into a new snapshot.

Per-cache limits: eight documents, 32 MiB including line indices, at most three active source reads. Access refreshes LRU order. Full source text is not stored in every tree node. Oversized cache insertions are surfaced as preview errors.

## Extension seams

An AST adapter belongs in analysis and emits symbol spans through core types. A different treemap strategy belongs in layout. A 3D renderer belongs alongside the map renderer, consuming the same immutable snapshots. Persistent indexing or filesystem events belong in runtime/index services. None requires converting `main.rs` or the top-level widget into an all-purpose implementation file.

This is a modular foundation, not a claim that million-file scaling or every target platform has already been benchmarked.
