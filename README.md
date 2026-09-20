# Scope

A local, language-independent spatial codebase explorer built with Rust and Makepad. Independent of Makepad Scope and not specific to Wave.

## Run

```sh
git clone https://github.com/LunaStev/scope.git
cd scope
cargo run --release -- /path/to/repository
```

Update with `git pull --ff-only origin master`. The executable is `target/release/scope`. Use a release build for interactive exploration. Headless inventory remains `--scan` or `--json`; `cargo run --no-default-features -- --json .` needs no display or map preparation. `--threads N` controls indexing workers, not map rasterization.

## Complete overview, live code up close

Scope prepares a complete actual-source overview before revealing the map. All accepted files contribute their actual characters to a 2048-square image. Tiny characters contribute filtered coverage rather than disappearing at a glyph-count limit. No sampled bars or invented blocks stand in for code.

That overview stays resident. Distant navigation scales and translates prepared images on the GPU. Near the camera, retained native GPU SDF text is layered over the same layout, so readable code is no longer only an enlarged image. Native geometry is prepared before reading density and blended gradually at the same positions. Zoom does not reflow lines or spread columns apart.

The view and nearby source are prepared ahead of time. Warm native movement reuses draw lists rather than reconstructing glyphs. Regional images now reuse a revision-checked source cache and lexical checkpoints; only intersecting columns and rows are rasterized. An immutable spatial index selects native foreground pages without walking the entire codebase each frame.

This is CPU background preparation plus GPU image composition and retained SDF text, not GPU parsing or a GPU raster-preparation claim. Cold native glyph setup remains bounded work; an abrupt uncached jump can need preparation while the complete backing stays interactive. Realtime refers to rendering, not automatic on-disk edit tracking.

Map images persist between launches. Initial preparation can take longer than indexing; restarting an unchanged codebase reuses its cached overview instead of rasterizing every character again.

## Controls

- Wheel: cursor-anchored zoom; drag: pan.
- Double-click a file: target a readable 12 px scale under the pointer.
- `F` / Read: focus a file from its beginning; fit directories.
- `Home` / Overview: fit the whole repository. Backspace / Up: parent.
- Source: manually show/hide source. Details: show/hide the inspector.
- Area: non-blank lines, physical lines, code lines or text bytes.

Source columns keep three character cells of separation. Ordinary pan/zoom does not reflow code. Resize and Area changes can create another layout and map. Search highlights matching visible paths without rerasterizing the entire map. The compact toolbar, accurate statistics and quiet renderer name stay unchanged.

## Root modules

```text
app/        Process entry, CLI and reports
model/      Tree, line shapes, source and metric contracts
language/   Language catalogue, detection, metrics and display lexing
analysis/   Read-only parallel indexing and unchanged-record reuse
layout/     Treemap, packed code, spatial queries and camera
raster/     Actual-glyph images, checkpointed source reuse and persistent cache
runtime/    Background jobs, revisions, source documents and map queues
render/     GPU image composition, native SDF foreground and annotations
ui/         Compact controls, summary, inspector and input
```

These are root-level workspace packages, without `crates/` or `scope-*` prefixes. Only the executable package is named `scope`. Model, language, analysis, layout, raster and runtime have no Makepad dependency.

## Cache and memory

Map images contain visual copies of code and stay local. Linux defaults to `$XDG_CACHE_HOME/scope/maps-v1` or `~/.cache/scope/maps-v1`. The private cache directory is 0700 on Unix. Image writes are atomic; invalid images become misses. Owned images are pruned to an approximate 512 MiB disk budget.

`SCOPE_MAP_CACHE_OFF=1` disables persistent image caching. `SCOPE_MAP_CACHE_DIR=/path/to/private/cache` selects a dedicated directory.

Keys include root, paths, size/modification time, geometry, font bytes and format revision. This is not content-hash validation of every source file. Re-index after edits. A changed scene layout invalidates its map images as a whole.

The GPU image layer retains the 16 MiB overview and at most 96 one-MiB detail images. Native detail has a one-million-glyph / 4,096-entry geometry accounting budget and a 128 MiB source-document budget. The raster worker's separate source cache is 64 MiB. These are independent budgets, not total RAM or VRAM caps; CPU/GPU copies, fonts, spatial indices, temporary buffers and in-flight data are additional. Native detail trades extra memory and bounded initial glyph work for sharper realtime code.

## Language and metrics

`language/` owns filename/compound-extension/shebang detection and lexical classification. Wave is explicitly recognised. Every indexed file obeys:

```text
physical lines = code + comment-only + blank + unclassified
```

Summary counts are repository-wide; inspector counts are selected-node data. Unknown text remains unclassified. Byte units are B/KiB/MiB/GiB. Image resolution or native detail does not change measurements.

All access is local and read-only. Metadata directories and symlinks are excluded; ignore rules are respected. Options include `--include-build`, `--no-ignore`, repeatable `--exclude NAME`, and `--max-file-mib N` (default eight). Only accepted UTF-8 text contributes. Detected revision changes are reported instead of mixing old measurements with new source.

## Validation

```sh
cargo test --workspace
cargo test -p scope --no-default-features
cargo build --release -p scope
python3 scripts/check_architecture.py
```

Tests cover complete source before zoom, native glyphs at reading density, background preservation, warm native reuse without glyph rebuilding, cached restart, changed-file invalidation and compact windows. A separate release benchmark measures readable-source CPU submission and input receipt. Generated four-million-line and fifty-million-line native-window tests and 100,000-file headless tests cover other stages.

CPU/input measurements are not GPU time, display latency or FPS. Synthetic CI fixtures are not actual Fuchsia/Chromium hardware tests. GUI CI uses Ubuntu software OpenGL.

See [rendering](docs/RENDERING.md), [architecture](docs/ARCHITECTURE.md), [design](docs/DESIGN.md), [metrics](docs/METRICS.md) and [language](docs/LANGUAGE.md). AST subdivision, 3D navigation and filesystem watching are not implemented.
