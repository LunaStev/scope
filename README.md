# Scope

A local, language-independent spatial codebase explorer built with Rust and Makepad. Scope is independent of Makepad Scope and is not specific to Wave.

## Run

```sh
git clone https://github.com/LunaStev/scope.git
cd scope
cargo run --release -- /path/to/repository
```

Update an existing checkout with `git pull --ff-only origin master`. The executable is `target/release/scope`. Use a release build for interactive exploration.

Headless reports: `--scan` or `--json`. `cargo run --no-default-features -- --json .` needs no display. `--threads N` controls indexing workers (1–32, automatically capped at eight).

## Root modules

```text
scope/
  app/          Process entry, CLI and reports
  model/        Tree, source shape, documents and measurement contracts
  language/     Catalogue, detection, classifiers and highlighting
  analysis/     Bounded parallel scanning and index reuse
  layout/       Treemap, packed code, camera and resumable tile cursors
  runtime/      Background jobs, versioned snapshots and resident documents
  render/       Retained layers, glyph tiles and first-paint scheduling
  ui/           Toolbar, summary, inspector and input
  docs/
  scripts/
  Cargo.toml
```

There is no `crates/` container or `scope-*` module prefix. All modules are independent workspace packages; only the executable package is named `scope`. Headless modules do not depend on Makepad.

## Compact, continuous source

A small toolbar and one-line summary leave the window for the code map. **Details** toggles the independently scrollable inspector. Measured names/counts share the allocator's header geometry and are clipped rather than overlapping. Numeric data does not change with zoom.

Source columns use their own longest line plus three character cells of separation. Ordinary pan/zoom scales the same actual text at the same positions, without bars, thumbnails or a representation-switch threshold. Resize and area changes may recompute the scene; ordinary zoom does not reflow it.

- Wheel zooms at the cursor; drag pans.
- Double-click a file enlarges the same source under the pointer to a 12 px reading target.
- `F` / **Read** focuses a file from its beginning or fits a directory.
- `Home` / **Overview** fits the repository; `Backspace` / **Up** focuses the parent.
- **Area** cycles non-blank lines, physical lines, code lines and text bytes.
- **Source** manually toggles the source layer.

## First-paint performance

Index completion no longer starts an unbounded scan of all missing glyph tiles. The renderer discovers nodes and labels incrementally, retains their draw lists, and uses lazy source cursors that resume where the previous pass stopped. A missing source read is requested once per admitted file, not once per tile. Ready worker results go directly to a ready queue.

Node backgrounds use one fill/border instance rather than five quads. While source arrives, existing backgrounds and names are reused. Selection and hover are separate outlines. Warm source rendering visits only resident tiles, instead of also walking every missing tile of every visible file.

Per-pass admission limits are 2,048 node/edge visits, 256 label candidates, 128 source-tile checks, **two new source tiles across the whole map**, and 8,192 new source character instances. Cold source work has a roughly four-millisecond elapsed CPU guard, including the first tile. This is a soft time guard, not a hard GPU frame deadline: an individual native font operation can exceed it.

The first layout is published only after matching the latest viewport/metric. An obsolete initial layout is not displayed and immediately discarded. Source-allocation accounting and heavyweight document retirement run off the UI event thread. When waiting for reads, completion signals wake the UI instead of continuous polling redraws.

## Large repositories and residency

Initial indexing retains statistics, revision metadata and compact per-line widths, not every source/token string. Parallel workers feed a bounded result queue; progress is throttled to roughly 150 ms. Same-process re-indexing reuses records whose size and modification time are unchanged. There is no persistent or content-hashed index.

Visible actual source is prepared on background workers at any zoom level. The document cache has a 128 MiB accounting budget, 256 entries and at most two active workers. Glyph geometry uses 32-line by 128-character tiles and retains up to one million glyphs / 4,096 tile entries. The existing 50,000 visible-node safety limit is retained. Exhausted detail budgets are reported instead of causing endless eviction/rebuild loops; focusing a smaller region changes the eligible working set.

These limits do not cap total process RAM or GPU memory. Font atlases, in-flight/retired data, native allocations and GPU copies are additional. See [rendering details](docs/RENDERING.md).

## Languages and measurements

`language/` owns JSON definitions, validated registration, filename/compound-extension/shebang detection, a pinned Tokei catalogue adapter, classifiers and display lexers. Wave is recognized explicitly and has its own comment/string-aware classifier. The scanner and UI do not contain language-name special cases. See [language contracts](docs/LANGUAGE.md).

```text
physical lines = code + comment-only + blank + unclassified
```

The summary is repository-wide; the inspector's main section is selected-node data. Language shares use workspace physical lines. Unknown text remains unclassified. Byte units adapt between B/KiB/MiB/GiB; counts are not inflated to match minimum visual tile weights.

All access is local and read-only. `.git`, `.hg`, `.svn` and symlinks are excluded. Ignore files are respected by default. Options include `--include-build`, `--no-ignore`, repeatable `--exclude NAME`, and `--max-file-mib N` (default eight). Only accepted UTF-8 text is indexed. Re-index after edits; revision-mismatched source is not silently mixed with earlier measurements.

## Validation

```sh
cargo test --workspace
cargo test -p scope --no-default-features
cargo build --release -p scope
python3 scripts/check_architecture.py
```

CI checks module boundaries, metric accounting, language detection, parallel/serial agreement, residency, suspended tile cursors, initial-aspect publication, source before zoom, zoom round trips and compact windows. Generated large-index fixtures remain 20,000 files / 50 million lines and 100,000 files / 10 million lines.

A separate matched release benchmark opens **4,000 files / four million lines in a real window** and sends scroll input during post-index detail population. It preserves first-map/early-frame CPU submission measurements and native input-handler latencies, with per-pass admission assertions. These results are not GPU completion time or FPS. GUI CI uses Ubuntu and software OpenGL, not Fedora hardware; the generated fixtures are not actual Chromium or Fuchsia checkouts.

See [architecture](docs/ARCHITECTURE.md), [design](docs/DESIGN.md), [metrics](docs/METRICS.md) and [contributing](CONTRIBUTING.md). AST subdivision, a 3D camera, persistent indexing and filesystem watching are not implemented yet.
