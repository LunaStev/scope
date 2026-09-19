# Contributing

Choose the owning module before implementing a feature. See [architecture](docs/ARCHITECTURE.md).

- Models, units and invariants belong in `scope-core`.
- Indexing and language analysis belong in `scope-analysis`.
- Spatial math belongs in `scope-layout`.
- Background coordination and caching belong in `scope-runtime`.
- GPU map passes belong in `scope-render`.
- The shell, dashboard and inspector belong in `scope-ui`.
- CLI parsing/reporting belong in `apps/scope`.

Do not add a filesystem walk to a draw method, a Makepad dependency to a headless crate, or independent line-count logic to a UI widget. New internal dependencies must be consciously reflected in `scripts/check_architecture.py`.

Run:

```sh
cargo test -p scope-core -p scope-analysis -p scope-layout -p scope-runtime
cargo test -p scope --no-default-features
cargo build -p scope
cargo fmt --all
python3 scripts/check_architecture.py
cargo run -- --json . > inventory.json
python3 scripts/check_report.py inventory.json
```

Test metric changes with unknown languages, comments inside strings, mixed comment/code lines, CRLF, empty files and embedded languages. Test cache changes for replacement, eviction and stale results. Test layout changes for stable ordering, finite geometry, empty inputs and area conservation. Include an actual GUI capture for UI changes.

Keep changes focused. Splitting files is not sufficient on its own: dependencies, ownership and testable contracts must stay clear.
