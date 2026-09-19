#!/usr/bin/env bash
set -euo pipefail
base=455a32084163cd17b42fb219781d7197df322a49
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
git fetch --no-tags --depth=1 origin "$base"
mkdir "$work/before"
git archive "$base" | tar -xf - -C "$work/before"
python3 - "$work/before/render/src/map.rs" <<'PY'
from pathlib import Path
import sys
path=Path(sys.argv[1]);text=path.read_text()
needle='stats.cpu_submit_ms = start.elapsed().as_secs_f64()*1000.0;'
assert text.count(needle)==1
trace='''
        if std::env::var_os("SCOPE_PERF").is_some() {
            eprintln!("scope perf: cpu_ms={:.4} built=0 reused=0 pending=0 submitted_runs={} geometry_bytes=0",stats.cpu_submit_ms,stats.glyph_runs);
        }
'''
path.write_text(text.replace(needle,needle+trace))
PY
export CARGO_TARGET_DIR="$work/target"
cargo build --release --manifest-path "$work/before/Cargo.toml"
cp "$work/target/release/scope" "$work/before-bin"
cargo build --release
cp "$work/target/release/scope" "$work/after-bin"
xvfb-run -a -s '-screen 0 1600x1050x24' python3 scripts/benchmark_render.py "$work/before-bin" "$work/after-bin"
