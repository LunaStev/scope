#!/usr/bin/env bash
set -euo pipefail
xvfb-run -a -s '-screen 0 1600x1050x24' python3 scripts/gui_smoke.py
python3 - <<'PY'
from pathlib import Path
import re
# The actual renderer must be known on this native Linux test, not merely a
# successful empty GpuInfo getter. This checks the same data displayed by the UI.
text = Path('evidence/source.log').read_text()
names = re.findall(r'^scope graphics: renderer=(.*?) vendor=', text, re.M)
assert any(name not in ('', 'unknown', 'not reported') for name in names), names
print('Active renderer name verified:', names[-1])
PY
