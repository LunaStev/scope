# Scope

A local, language-independent spatial codebase explorer. Point it at a directory, see its folders and text files as a nested map, then zoom into actual source code.

Scope is an independent application inspired by spatial source-code visualizers. It is not Makepad Scope or a feature of Makepad Director. The native interface uses the published `makepad-widgets` 1.0.0 release; a separate Makepad checkout is not required.

## Run

```sh
cargo run --release -- /path/to/repository
```

The executable is `target/release/scope`. With no path, it reads the current directory. A Git repository is not required. Quote paths containing spaces.

```sh
cargo run --release -- .
cargo run --release -- --scan /path/to/repository
cargo run --release -- --include-build --exclude vendor /path/to/repository
```

Linux requires the development libraries used by Makepad (X11/OpenGL and audio libraries). An existing working Makepad build normally means these dependencies are already present. Rust stable is used for CI.

## First implementation

- Background directory scanning, stable hierarchical squarified layout and cursor-anchored wheel zoom.
- Drag to pan, click to select, double-click to focus; Home, Parent and Fit selection controls.
- Folder/file labels, filename/path search highlighting and an Inspector with actual file counts and line counts.
- Miniature traces sampled from real file contents; sufficiently close views load and draw actual source text in the map.
- Language labels based on filenames, with arbitrary UTF-8 text as a fallback. No compiler integration or language-specific AST is required.
- Read-only operation. No code execution, cleanup, file rewriting, server or source upload.

**Area is non-blank physical lines, including comments**, with a minimum visible weight for empty text files. It is not language-aware SLOC, complexity, function count or disk usage.

## Exclusions and limits

`.git`, `.hg` and `.svn` are always excluded, including `.git` pointer files in worktrees. Symlinks are not followed. Binary/non-UTF-8 files are skipped and counted.

By default Scope respects `.gitignore`, `.ignore` and `.scopeignore`, even for copied source trees without a `.git` directory. Parent/global ignore files are not consulted. It also excludes directories named `target`, `node_modules`, `build`, `dist`, `.venv`, `venv`, `__pycache__` and `.cache`.

`--include-build` disables the built-in build/dependency exclusions. `--no-ignore` disables ignore-file rules. `--exclude NAME` adds a basename exclusion and can be repeated. Use `.scopeignore` for path/glob rules.

The scanner reads at most 8 MiB per file by default (`--max-file-mib N` changes this). Source display is loaded on demand, with up to three reads in flight, eight cached documents and a 32 MiB document-cache budget. The source viewer has a separate 16 MiB per-file maximum. Oversized files and read errors are reported rather than silently included in totals.

## Controls

| Action | Control |
| --- | --- |
| Open another source tree | Path field, then Open |
| Rescan current tree | Refresh |
| Zoom around pointer | Mouse wheel |
| Pan | Drag inside the map |
| Select | Single click |
| Focus a file or folder | Double click / Fit selection / F |
| Fit whole tree | Home button / Home key |
| Focus parent | Parent button / Backspace |
| Highlight paths | Search field |
| Toggle code traces/text | Source button |

Keyboard shortcuts are active after clicking the map. Source text is read from the current file; use Refresh after editing to recompute layout and statistics.

## Project structure

```text
src/
  main.rs      CLI and native entry point
  app.rs       Window, toolbar and search
  view.rs      GPU drawing, interaction, async scan/load and source cache
  scanner.rs   Ignore-aware, read-only file inventory
  source.rs    Bounded UTF-8 reads, line index and lexical display hints
  model.rs     Language-independent tree and statistics
  layout.rs    Squarified layout and hierarchical hit testing
  camera.rs    Coordinate transforms and pointer-anchored zoom
```

```sh
cargo test --no-default-features
cargo build
cargo run -- --scan .
```

The core tests do not depend on a window system. CI builds the native app and includes an Xvfb startup smoke test.

## Next milestones

Function/type regions through optional language adapters, a persistent index, smooth animated camera transitions, incremental filesystem updates and additional 3D/tilted presentation modes. The first version is a 2D zoomable code map: it does not yet provide AST symbols, dependency edges or a 3D fly-through. Lexical colors are language-neutral hints, not a language-accurate highlighter.

No project license has been selected yet. Makepad and other dependencies retain their own licenses.
