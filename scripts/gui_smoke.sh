#!/usr/bin/env bash
set -euo pipefail
fixture=$(mktemp -d /tmp/scope-source-fixture.XXXXXX)
trap 'rm -rf "$fixture"' EXIT
python3 - "$fixture" <<'PY'
from pathlib import Path
import sys
root = Path(sys.argv[1])
text = ''.join(f'fn function_{i}() {{ let value = {i}; }} // actual source line {i}\n' for i in range(1200))
(root / 'main.rs').write_text(text)
PY
export SCOPE_FIXTURE="$fixture"
xvfb-run -a -s '-screen 0 1600x1050x24' bash -e -c '
    export LIBGL_ALWAYS_SOFTWARE=1 SCOPE_TRACE=1
    ./target/debug/scope "$SCOPE_FIXTURE" > /tmp/scope-fixture.log 2>&1 &
    pid=$!
    trap "kill $pid 2>/dev/null || true" EXIT
    sleep 8
    kill -0 "$pid"
    wid=$(xdotool search --pid "$pid" --name "Scope" | head -1)
    xdotool windowmove "$wid" 0 0
    cp /tmp/scope-fixture.log /tmp/scope-before-zoom.log
    import -window "$wid" scope-source-overview.png
    xdotool mousemove 580 500 click --repeat 2 --delay 100 1
    sleep 2
    kill -0 "$pid"
    import -window "$wid" scope-source-read.png
    xdotool key Home
    xdotool click 4
    sleep 1
    xdotool key Home
    sleep 1
    kill "$pid"
    wait "$pid" 2>/dev/null || true
    trap - EXIT
    ./target/debug/scope . > /tmp/scope-repository.log 2>&1 &
    pid=$!
    trap "kill $pid 2>/dev/null || true" EXIT
    sleep 8
    kill -0 "$pid"
    wid=$(xdotool search --pid "$pid" --name "Scope" | head -1)
    import -window "$wid" scope-overview.png
'
python3 - <<'PY'
import re
from pathlib import Path
before = Path('/tmp/scope-before-zoom.log').read_text()
after = Path('/tmp/scope-fixture.log').read_text()
pattern = r'scope source: lines=(\d+) runs=(\d+) font=([\d.]+) representation=source'
b = [(int(a), int(r), float(f)) for a,r,f in re.findall(pattern,before)]
a = [(int(n), int(r), float(f)) for n,r,f in re.findall(pattern,after)]
assert any(n == 1200 and r > 0 and 0 < f < 7 for n,r,f in b), 'All actual source lines must render BEFORE zoom, below the former threshold'
assert any(r > 0 and f >= 11.9 for _,r,f in a), 'Double-click must immediately reach readable source'
for name in ('/tmp/scope-fixture.log','/tmp/scope-repository.log'):
    log=Path(name).read_text()
    assert not re.search(r'panicked|error expanding|error applying|shader compilation failed',log,re.I), log
print('Real source at overview + no threshold + direct readable double-click: passed.')
PY
cp /tmp/scope-fixture.log gui.log
cp /tmp/scope-before-zoom.log overview.log
cp /tmp/scope-repository.log repository.log
