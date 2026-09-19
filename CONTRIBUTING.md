# Contributing

Use the owning root module: `model`, `analysis`, `layout`, `runtime`, `render`, `ui` or `app`. Dependencies follow [architecture](docs/ARCHITECTURE.md). Do not add filesystem access or lexical parsing to drawing code, or GUI dependencies to the model/analysis/layout/runtime layers.

```sh
cargo test -p model -p analysis -p layout -p runtime
cargo test -p scope --no-default-features
cargo build -p scope
cargo fmt --all
python3 scripts/check_architecture.py
cargo run -- --json . > /tmp/scope-inventory.json
python3 scripts/check_report.py /tmp/scope-inventory.json
```

Rendering changes must preserve a single source representation. Test tiny source at overview, direct readable focus, cursor-anchored zoom, long lines, Unicode and snapshot consistency. No font-size gate or sampled preview may be reintroduced. Keep page geometry independent of the camera.

Metric changes need comment/string, unknown-language, CRLF, empty-file and embedded-language tests. Layout changes need finite geometry, stable ordering and area conservation tests. Add actual GUI captures for UI changes.
