# Scope

A local, language-independent spatial codebase explorer built with Rust and Makepad. Independent of Makepad Scope and not specific to Wave.

## Run

```sh
git clone https://github.com/LunaStev/scope.git
cd scope
cargo run --release -- /path/to/repository
```

Update with `git pull --ff-only origin master`. The executable is `target/release/scope`. Use a release build for interactive exploration.

Headless inventory is unchanged: `--scan` or `--json`. `cargo run --no-default-features -- --json .` needs no display or map preparation. `--threads N` controls indexing workers, not map rasterization.

## Prepare once, navigate the map

Scope now finishes a **complete actual-source overview** before revealing the map. It reads the accepted source files and draws their actual characters into a 2048 × 2048 image. Tiny characters contribute filtered glyph coverage rather than disappearing at an arbitrary glyph-count limit. No sampled bars or invented code blocks stand in for source.

That overview stays resident. Navigation scales and translates prepared images on the GPU. It does not rebuild millions of text glyphs on the UI thread. When more resolution is needed, a background worker produces 512 × 512 regional images. The existing parent image stays visible while detail arrives; detail fades in at the same coordinates. An abrupt uncached jump can briefly be soft, but is not replaced with an empty region.

The view and nearby regions are requested ahead of time. Cached resolutions are reused on return visits. Map images persist between application launches. First preparation can take longer than indexing; restarting an unchanged codebase reuses its cached overview instead of rasterizing every character again.

This is **CPU background preparation plus GPU image composition**, not a claim that source parsing or map-image construction runs on the GPU. The code layout, three-character column gaps, selection, search, measurement definitions and quiet renderer footer remain.

## Controls

- Wheel: cursor-anchored zoom; drag: pan.
- Double-click a file: target a readable 12 px scale under the pointer.
- `F` / Read: focus a file from its beginning; fit directories.
- `Home` / Overview: fit the whole repository. Backspace / Up: parent.
- Source: manually show/hide source images. Details: show/hide the inspector.
- Area: non-blank lines, physical lines, code lines or text bytes.

The same source is used at all resolutions. Ordinary zoom does not reflow its lines or columns. Resizing the window or changing Area can produce a different layout and therefore a new map. Search highlights matching visible paths; it does not rerasterize the whole map.

## Root modules

```text
app/        Process entry, CLI and reports
model/      Tree, line shapes, source and metric contracts
language/   Language catalogue, detection, metrics and display lexing
analysis/   Read-only parallel indexing and unchanged-record reuse
layout/     Treemap, packed code pages and camera
raster/     Actual-glyph rasterization, resolution tiles and persistent image cache
runtime/    Background jobs, scene revisions and map preparation queues
render/     GPU texture composition and native annotations
ui/         Compact controls, summary, inspector and input
```

All are root-level workspace packages; no `crates/` container or `scope-*` module prefixes. Only the executable package is named `scope`. Model, language, analysis, layout, raster and runtime have no Makepad dependency.

## Cache and memory

Map images contain visual copies of code. They stay local. On Linux the default cache is `$XDG_CACHE_HOME/scope/maps-v1`, or `~/.cache/scope/maps-v1`. The directory is private (0700 on Unix), image writes are atomic, invalid images become cache misses, and old cache images are pruned to an approximate 512 MiB disk budget. No remote upload or interpreter execution is involved.

- `SCOPE_MAP_CACHE_OFF=1`: disable persistent image caching.
- `SCOPE_MAP_CACHE_DIR=/path/to/private/cache`: select a dedicated cache directory.

The cache key covers the root, file paths, size/modification-time revisions, world geometry, font bytes and map-format revision. This is not a content-hash check of every source file. Re-index after edits. A layout change currently invalidates the scene's map cache as a whole, not just the changed file.

GPU composition retains the pinned 16 MiB overview and at most 96 one-MiB detail images. Those image-byte counts are not total process or GPU-memory caps: CPU copies, driver allocations, raster working buffers, fonts and in-flight work are additional. Decoded result queues and per-paint texture uploads are bounded.

## Language and metrics

`language/` owns filename/compound-extension/shebang detection and lexical classification. Wave is explicitly recognised. All indexed text obeys:

```text
physical lines = code + comment-only + blank + unclassified
```

Summary counts are repository-wide; inspector counts are for the selection. Unknown text remains unclassified. Byte units are B/KiB/MiB/GiB. Image resolution and visible detail never change measured file or line counts.

`.git`, `.hg`, `.svn` and symlinks are excluded. Ignore files are respected. Options include `--include-build`, `--no-ignore`, repeatable `--exclude NAME`, and `--max-file-mib N` (default eight). Only accepted UTF-8 text contributes to the index and map. Source changes detected during map preparation are reported instead of silently mixing old measurements with new text.

## Validation

```sh
cargo test --workspace
cargo test -p scope --no-default-features
cargo build --release -p scope
python3 scripts/check_architecture.py
```

GUI tests verify the full source fixture before any zoom, a persistent backing image during uncached zoom, detailed image arrival, cached restart, edit invalidation and compact windows. A matched native-window benchmark separates initial complete-map preparation, cold/warm CPU submission and input-handler latency on a generated 4,000-file / four-million-line fixture. These measurements are not GPU time or FPS and do not substitute for a real Chromium/Fuchsia checkout on user hardware.

Headless 50-million-line and 100,000-file fixtures continue to validate indexing separately. Linux GUI CI uses Ubuntu software OpenGL.

See [rendering](docs/RENDERING.md), [architecture](docs/ARCHITECTURE.md), [design](docs/DESIGN.md), [metrics](docs/METRICS.md) and [language](docs/LANGUAGE.md). AST subdivision, 3D navigation and filesystem watching are not implemented.
