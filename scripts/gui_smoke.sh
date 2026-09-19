#!/usr/bin/env bash
set -euo pipefail
xvfb-run -a -s '-screen 0 1600x1050x24' python3 scripts/gui_smoke.py
