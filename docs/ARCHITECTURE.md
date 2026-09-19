# Architecture

## Dependency direction

Root workspace modules are `app`, `model`, `language`, `analysis`, `layout`, `raster`, `runtime`, `render` and `ui`. Only the executable is named `scope`.

```text
model
  <- language
  <- analysis (also uses language)
  <- layout
       <- raster (also uses analysis and language)
            <- runtime
                 <- render (Makepad boundary)
                      <- ui
                           <- app
```

The exact allowed direct edges are checked by `scripts/check_architecture.py`. No Makepad dependency is allowed in model/language/analysis/layout/raster/runtime. The executable composes services instead of owning scanning or draw loops.

## Ownership

Analysis produces an immutable tree, compact exact line widths and file revisions using bounded parallel workers. Language owns detection and metric rules. Layout creates source columns and hierarchy rectangles in fixed world coordinates. Session publishes only matching metric/viewport generations.

Raster owns glyph filtering, actual-source image construction, tile addressing and private persistent caching. It depends on no windowing API. The host supplies existing font bytes; raster does not vendor or discover arbitrary installed fonts.

Runtime maps own cancellation, one worker, scene lifetime, priority queues and bounded decoded-image transfer. Font parsing, source reads, rasterization, PNG encode/decode and cache pruning happen here rather than on the UI draw thread.

Render owns native textures and GPU composition. Its ImageMap retains the root backing image even while detailed children are absent. It clips regional images and fades in ready detail. Native folder annotations and selection outlines are separate from the cached source layer.

UI displays index/map preparation separately, preserves the quiet renderer label, and routes input. CLI inventory does not construct maps or require a display.

## Extension boundaries

A faster raster worker or GPU preprocessing implementation can replace rasterization without changing the source layout or navigation contract. Persistent per-file incremental updates can evolve the scene cache. AST adapters belong in language/analysis and publish source spans through model. The UI is not the owner of language heuristics, disk cache policy or per-character loops.

See RENDERING.md for budgets, cache privacy, failure behavior and measurement scope.
