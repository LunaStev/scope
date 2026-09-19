# Scope

A local, language-independent spatial codebase explorer built with Rust and Makepad. Scope is independent of Makepad Scope and is not specific to Wave.

## Run

```sh
git clone https://github.com/LunaStev/scope.git
cd scope
cargo run --release -- /path/to/repository
```

Existing checkout:

```sh
git pull --ff-only origin master
cargo run --release -- /path/to/repository
```

The executable remains `target/release/scope`. For headless reports use `--scan` or `--json` (which implies `--scan`). `cargo run --no-default-features -- --json .` needs no window.

## Root modules

```text
scope/
  app/          Process entry, command-line arguments, text/JSON reporting
  model/        Shared tree, source-document and measurement types
  analysis/     Read-only scanning, ignore policy and lexical analysis
  layout/       Treemap, camera, hit tests and fixed source-page geometry
  runtime/      Background indexing and immutable scene snapshots
  render/       GPU drawing of the same real source at every scale
  ui/           Shell, dashboard, inspector and input handling
  docs/         Architecture, metrics and design contracts
  scripts/      Architecture and GUI regression tests
  Cargo.toml
```

Directories, package names and Rust imports use the names above, without a project prefix. The executable package alone is named `scope`. Independent modules remain a Cargo workspace, with `app` as the default member. Analysis, model, layout and runtime have no GUI dependencies.

## Continuous source rendering

The initial completed index already holds the actual source text and its prepared lexical runs. Every file receives one fixed world-space text layout. Zoom only changes camera scale and translation: no sampled line bars, preview-to-code transition, font-size gate or delayed read at a zoom threshold. Long lines are not truncated at 240 characters. Rendering clips off-screen geometry and text, but does not replace it with another representation.

At overview scale text is naturally tiny. Zoom enlarges the same characters at the same positions. Source may occupy several columns within a file tile; the column arrangement is computed once and never changes because of zoom.

Controls:

- Wheel: faster, cursor-anchored zoom (logarithmic sensitivity 0.022, previously 0.006).
- Double-click a file: enlarge the source under the pointer to a readable 12 px target without reflow.
- `F` or **Read / Focus**: show a file from its beginning at readable scale; fit directories.
- `Home` or **Overview**: fit the repository. `Backspace` or **Parent**: navigate upward.
- Drag: pan. Wheel over the inspector scrolls the inspector.
- Area button: cycle non-blank lines, physical lines, code lines and text bytes.
- Source and inspector buttons: toggle the respective layers, independently of zoom.

## Measurements

The dashboard shows text files, physical lines, classified code lines and adaptive B/KiB/MiB/GiB text sizes. The selected-node inspector separates code, comment-only, blank and unclassified lines. Workspace language shares use all indexed physical lines as the denominator.

```text
physical lines = code + comment-only + blank + unclassified
```

Lexical classification uses pinned Tokei 12.1.2. Unknown non-blank text stays explicitly unclassified. Rendering colors are display hints, not the authority for metrics. Counts are not adjusted to reflect the minimum visual area assigned to empty files. See [metric definitions](docs/METRICS.md).

## Indexing and memory

All repository access is local and read-only. `.git`, `.hg`, `.svn` and symbolic links are excluded. `.gitignore`, `.ignore` and `.scopeignore` are respected by default; build/dependency folders are excluded by default. Options include `--no-ignore`, `--include-build`, repeatable `--exclude NAME` and `--max-file-mib N` (1–1024, default 8).

Only accepted UTF-8 text is indexed. The active snapshot retains source and prepared text so zoom does not swap representations or reread a changed file. This consumes more RAM than an eight-file preview cache; that old cache has been removed. Source allocation is reported explicitly. It is not total process RAM, GPU memory, or disk allocation. Re-index to load edits made after a snapshot.

## Development

```sh
cargo test -p model -p analysis -p layout -p runtime
cargo test -p scope --no-default-features
cargo build -p scope
python3 scripts/check_architecture.py
```

CI builds the native app, checks metric accounting and module boundaries, and verifies that source is rendered **before any zoom input**, then that a double-click reaches readable scale. It also captures the repository overview.

Linux builds need a current stable Rust toolchain and the X11/OpenGL/audio development packages required by Makepad. GUI CI uses Ubuntu and software OpenGL, not Fedora hardware validation.

See [architecture](docs/ARCHITECTURE.md), [design](docs/DESIGN.md) and [contributing](CONTRIBUTING.md). AST subdivision, a 3D camera, persistent indexing and incremental filesystem watching are not implemented yet.
