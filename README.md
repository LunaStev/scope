# Scope

A local, language-independent spatial codebase explorer built with Rust and Makepad. Scope is independent of Makepad Scope and is not specific to Wave.

## Run

```sh
git clone https://github.com/LunaStev/scope.git
cd scope
cargo run --release -- /path/to/repository
```

Update an existing checkout with `git pull --ff-only origin master`. The executable is `target/release/scope`. Use a release build for interactive exploration.

Headless reports: `--scan` or `--json`. `cargo run --no-default-features -- --json .` needs no display. `--threads N` controls indexing workers (1–32; automatic selection is capped at eight).

## Root modules

```text
scope/
  app/          Process entry, CLI and reports
  model/        Tree, source shape, documents and measurement contracts
  language/     Language catalogue, detection, classifiers and highlighting
  analysis/     Bounded parallel scanning and index reuse
  layout/       Treemap, shared node frames, packed code and camera
  runtime/      Background jobs, snapshots and resident documents
  render/       Retained glyph tiles, clipping and annotations
  ui/           Toolbar, summary, inspector and input
  docs/
  scripts/
  Cargo.toml
```

No `crates/` container or `scope-*` module prefixes. All modules are independent workspace packages. Only the executable package is named `scope`. Headless modules do not depend on Makepad.

## Compact map and continuous source

A small toolbar and one-line summary leave the main window for the code map. **Details** toggles the independently scrollable inspector. Counts use the measured tree, grouped digits and measured text alignment.

Source columns use their own longest line plus three character cells of separation; spare tile width is not spread between columns. Allocation and folder/file annotations share the same world-space header geometry. Labels are clipped to that header and omit the count before colliding with the name. Their statistics never change because of zoom.

The map's initial layout adapts to the available viewport, including its height. Resizing the window or changing the area metric can recompute the scene. Ordinary pan/zoom does not reflow the code.

- Wheel zooms at the cursor; drag pans.
- Double-click a file enlarges the same source under the pointer to a 12 px reading target.
- `F` / **Read** opens the beginning of a file at readable scale or fits a directory.
- `Home` / **Overview** fits the repository; `Backspace` / **Up** focuses its parent.
- **Area** cycles non-blank lines, physical lines, code lines and text bytes.
- **Source** manually toggles source; no automatic representation switch occurs with zoom.

## Large-repository pipeline

The initial index retains counts, file revision metadata and compact per-line widths, **not every source string and token**. Parallel workers feed a bounded result queue. Progress is throttled to approximately 150 ms. Re-index reuses unchanged records in the same process based on file size and modification time; there is no persistent disk index or content-hash guarantee.

Real source is prepared on background workers at any zoom level when needed for the visible region. It is never replaced with sampled bars or thumbnails. This changes when data is resident, not how code is represented. The resident-document cache has a 128 MiB accounting budget and at most two active preparation jobs.

Glyph geometry is split into 32-line by 128-character tiles. Off-screen files, columns and tiles are culled. Warm pan/zoom reuses native draw lists, updating only the camera/clipping transform. Cold preparation yields after an approximately 5 ms soft budget and at most two new tiles per file per pass. The geometry cache is limited to one million glyphs and 4,096 entries. These are accounting limits, not total-process or GPU-memory caps.

When the visible working set exceeds a detail budget, the footer says so instead of continually evicting and rebuilding that same view. Focusing a smaller region makes its detail eligible. Pending reads and preparation are also visible in the footer. A completed idle view does not request continuous redraws.

## Language services

`language/` is the shared service for the scanner and source preparation, with JSON definitions, validated registration and a pinned Tokei catalogue adapter. Detection checks exact filenames, compound extensions and shebangs without reopening or executing files. Wave is explicitly recognized and has its own line classifier and stateful display lexer. Unknown text remains unclassified rather than being counted as guessed code. See [language contracts](docs/LANGUAGE.md).

```text
physical lines = code + comment-only + blank + unclassified
```

The summary is repository-wide. The selected-node inspector separates line categories, index memory, resident source, reused files and warnings. Language shares use workspace physical lines. Byte units adapt between B/KiB/MiB/GiB.

## Index policy

All repository access is local and read-only. `.git`, `.hg`, `.svn` and symlinks are excluded. `.gitignore`, `.ignore` and `.scopeignore` are respected by default. Build/dependency folder exclusions can be changed with `--include-build`; other options include `--no-ignore`, repeatable `--exclude NAME`, and `--max-file-mib N` (default eight). Only accepted UTF-8 text is indexed. Re-index after editing a file; a mismatched revision is not silently mixed with old measurements.

## Validation

```sh
cargo test --workspace
cargo test -p scope --no-default-features
cargo build --release -p scope
python3 scripts/check_architecture.py
```

CI verifies line accounting, language detection, parallel/serial agreement, revision reuse, cache eviction, viewport/header geometry, source rendering before zoom, zoom round trips and compact windows. It additionally measures generated five-language fixtures: 20,000 files / 50 million physical lines and 100,000 files / 10 million physical lines. Reports separate indexing, layout, same-process warm reuse and peak process RSS.

Those generated fixtures are not Chromium or Fuchsia checkouts, and headless index timings are not GPU rendering or FPS measurements. Linux GUI CI uses Ubuntu with software OpenGL; it is not Fedora hardware validation.

See [architecture](docs/ARCHITECTURE.md), [rendering](docs/RENDERING.md), [metrics](docs/METRICS.md) and [design](docs/DESIGN.md). AST subdivision, a 3D camera, persistent indexing and filesystem watching are not implemented yet.
