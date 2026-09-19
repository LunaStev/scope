# Scope

A local, language-independent codebase explorer. Navigate an entire repository as a spatial map, then zoom into real source text.

Scope is an independent project, not Makepad Scope or a Wave-specific tool. Its native interface uses Rust and Makepad. Repository reads are local and read-only.

## Run

```sh
git clone https://github.com/LunaStev/scope.git
cd scope
cargo run --release -- /path/to/repository
```

The executable is `target/release/scope`. The repository root is a Cargo workspace; its default member is the desktop/CLI application in `apps/scope`.

### Linux prerequisites

A current stable Rust toolchain and the X11/OpenGL/audio development libraries used by Makepad are required. On Fedora:

```sh
sudo dnf install gcc gcc-c++ pkgconf-pkg-config libX11-devel libXcursor-devel libXrandr-devel libXi-devel libXinerama-devel alsa-lib-devel pulseaudio-libs-devel mesa-libGL-devel mesa-libEGL-devel
```

CI builds and exercises the GUI on Ubuntu with software OpenGL. Other environments are not implied to be tested by that result.

## Workspace

| Package | Responsibility |
| --- | --- |
| `scope-core` | Tree model, metric contracts, report types, number formatting |
| `scope-analysis` | Filesystem traversal, ignore policy, bounded reads, language classification, source documents |
| `scope-layout` | Squarified treemaps, metric weights, camera and hit-testing |
| `scope-runtime` | Background jobs, snapshot replacement, selection/search state and bounded source cache |
| `scope-render` | GPU map drawing, source LOD, text rendering and shared color tokens |
| `scope-ui` | Application shell, dashboard, inspector, controls and input routing |
| `scope` | Thin CLI entry point and JSON/text reporting |

Core analysis, layout and runtime have no dependency on Makepad. The CLI and GUI consume the same measurements. `scripts/check_architecture.py` enforces internal dependency direction.

## Interface

- Four overview cards: text files, physical lines, code lines and adaptive text-size units.
- Spatial map with language colors, path highlighting, source previews and actual text at readable zoom levels.
- Scrollable inspector with exact counts, line composition, workspace language distribution and index health.
- Area control cycles through non-blank lines, physical lines, classified code lines and text bytes.
- Overview, parent, selection focus, source visibility and inspector visibility controls.
- Footer reports visible nodes, CPU draw-submission time and source-cache memory. It does **not** label submission time as GPU frame time or FPS.

Wheel zooms at the cursor; drag pans; click selects; double-click focuses. `Home` fits the repository, `F` fits the selection, and `Backspace` focuses the parent. Wheel over the inspector scrolls its contents instead of zooming the map.

## Metrics

`physical lines = code + comment-only + blank + unclassified`

Recognized language classification uses pinned Tokei 12.1.2. Unknown non-blank text is explicitly unclassified, never guessed to be source code. Classification is lexical, not an AST, and documentation/configuration languages follow their respective classifier rules. See [metric definitions](docs/METRICS.md).

`Area: non-blank` remains the default. Comments and unknown text participate in that metric. Zero-weight files receive a small visibility floor; reported statistics are never inflated to match that floor. Directory weights sum the effective weights of their files.

### CLI

```sh
cargo run --release -- --scan /path/to/repository
cargo run --release -- --json /path/to/repository
cargo run --no-default-features -- --scan /path/to/repository
```

`--json` implies `--scan`, needs no display, and emits exact integers with schema version 1.

Other options: `--no-ignore`, `--include-build`, repeatable `--exclude NAME`, and `--max-file-mib N` (1–1024; default 8).

## Indexing policy

`.git`, `.hg`, `.svn` and symbolic links are always excluded. `.gitignore`, `.ignore` and `.scopeignore` are respected by default, including outside a Git repository. Build/dependency directories such as `target`, `node_modules`, `build`, `dist` and virtual environments are excluded by default. Additional exact basenames can be excluded with `--exclude`.

Only UTF-8 text is indexed. Oversized, binary/non-UTF-8, unreadable files and explicitly pruned entries are reported separately. Ignored subtrees are not traversed just to count their descendants. Text sizes are bytes read from indexed files, not allocated disk usage or Git history size.

Source previews are sampled; full text is loaded on demand. The source cache is bounded to eight documents and 32 MiB, with at most three concurrent source reads per active cache. Rendering culls off-screen/subpixel nodes and has explicit node/glyph submission budgets.

## Development

```sh
cargo test -p scope-core -p scope-analysis -p scope-layout -p scope-runtime
cargo test -p scope --no-default-features
cargo build -p scope
python3 scripts/check_architecture.py
```

CI also validates JSON accounting, launches the GUI, exercises zoom until real source glyphs are submitted, and captures full-size/compact screenshots.

See [architecture](docs/ARCHITECTURE.md), [design system](docs/DESIGN.md) and [contributing](CONTRIBUTING.md).

## Current scope

Implemented: 2D spatial exploration, language-aware line accounting, repository-wide summaries and real source text. Not yet implemented: AST/symbol subdivision, 3D camera, incremental filesystem watching or a persistent index. These can be added at the corresponding module boundaries rather than to the application entry point.
