# Scope

A local, language-independent spatial codebase explorer built with Rust and Makepad. Scope is independent of Makepad Scope and is not specific to Wave.

## Run

```sh
git clone https://github.com/LunaStev/scope.git
cd scope
cargo run --release -- /path/to/repository
```

Update an existing checkout with `git pull --ff-only origin master`. The executable remains `target/release/scope`. Use a release build for interactive exploration.

For headless reports use `--scan` or `--json` (which implies `--scan`). `cargo run --no-default-features -- --json .` needs no display.

## Compact workspace

A small repository toolbar, navigation/search row and one-line summary leave the main window for source exploration. Details live in a narrow, independently scrollable inspector. Numeric values are exact, grouped and aligned by measured text width. The **Details** button hides the inspector for a larger map.

- Wheel: cursor-anchored zoom. Drag: pan.
- Double-click a file: enlarge the same source under the pointer to a readable 12 px target.
- `F` / **Read**: focus the selected file from its beginning; fit directories.
- `Home` / **Overview**: fit the repository. `Backspace` / **Up**: focus the parent.
- **Area**: cycle non-blank lines, physical lines, classified code lines and text bytes.
- **Source**: toggle source visibility manually; it never switches automatically with zoom.

### Closely packed code columns

Each file's column arrangement is computed once in world space. A column takes the width of its own longest line, followed by **three character cells** of separation. Spare rectangle width is not distributed between columns. An unusually long line does not widen every other column. Line height is 1.65 times the nominal font size. No source lines are truncated or rearranged during zoom.

## Retained source rendering

Actual source is rendered at overview scale, not replaced with line bars, thumbnails or a different reading view. Zoom changes the scale of those same characters.

Source is prepared in fixed blocks of up to 128 lines. Each block retains its Makepad draw list and glyph instances. Warm pan/zoom reuses those lists and updates camera/clipping uniforms, instead of walking every lexical run, slicing strings and rebuilding glyph instances every frame. Files, columns and blocks outside the viewport are culled.

Cold blocks are prepared progressively, yielding between blocks after an approximately 8 ms CPU preparation budget. This is a soft budget, not a frame-time guarantee. The footer indicates pending preparation. It is independent of zoom level and never substitutes another source representation. A completed idle view does not request continuous redraws.

Geometry is invalidated when the scene, display DPI or live renderer style changes. Snapshot source and cached glyph geometry occupy memory; this is not a fixed-memory renderer. The inspector's **Source allocation** measures retained CPU source data, not total process or GPU memory. See [rendering details](docs/RENDERING.md).

## Root modules

```text
scope/
  app/          Process entry, command-line arguments, text/JSON reporting
  model/        Shared tree, source-document and metric types
  analysis/     Read-only scanning, ignore policy and lexical analysis
  layout/       Treemap, camera, hit tests and packed source geometry
  runtime/      Background indexing and versioned scene snapshots
  render/       Retained GPU source batches, clipping and map presentation
  ui/           Compact shell, summary, inspector and input handling
  docs/
  scripts/
  Cargo.toml
```

Directories, package names and imports use these names without a project prefix. The executable package alone is named `scope`. Independent modules remain a Cargo workspace, with `app` as the default member. Analysis, model, layout and runtime have no GUI dependencies.

## Measurements

```text
physical lines = code + comment-only + blank + unclassified
```

The summary is repository-wide; the inspector's first section is selected-node data. Language shares explicitly use workspace physical lines. Tokei 12.1.2 performs lexical classification; unknown non-blank text stays unclassified. Counts are never inflated to match the minimum visual area of empty files. Units adapt between B/KiB/MiB/GiB. See [metric definitions](docs/METRICS.md).

The footer reports retained source batches reused and CPU map submission time. Neither is GPU completion time, monitor FPS or a sustained performance guarantee. The matched release benchmark reports its fixture, sample count and environment separately.

## Indexing policy

All repository access is local and read-only. `.git`, `.hg`, `.svn` and symbolic links are excluded. `.gitignore`, `.ignore` and `.scopeignore` are respected by default; build/dependency folders are excluded by default. Options: `--no-ignore`, `--include-build`, repeatable `--exclude NAME`, `--max-file-mib N` (1–1024, default 8).

Only accepted UTF-8 text is indexed. Re-index to load edits made after snapshot creation. Oversized, unreadable and binary/non-UTF-8 files are reported separately.

## Development and validation

```sh
cargo test --workspace
cargo test -p scope --no-default-features
cargo build --release -p scope
python3 scripts/check_architecture.py
```

Linux builds need a current stable Rust toolchain and the X11/OpenGL/audio development packages required by Makepad. CI uses Ubuntu with software OpenGL, not Fedora hardware validation.

GUI regression tests check all 1,200 fixture lines at overview scale before zoom, direct readable double-click, reuse without rebuilding on camera movement, blank-file behavior and a compact window. CI also runs a matched release-build CPU submission benchmark against the previous renderer and preserves real screenshots and measurements as artifacts.

See [architecture](docs/ARCHITECTURE.md), [design](docs/DESIGN.md) and [contributing](CONTRIBUTING.md). AST subdivision, a 3D camera, persistent indexing and incremental filesystem watching are not implemented yet.
