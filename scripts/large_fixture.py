"""Deterministic mixed-language stress input. This is not Chromium or Fuchsia."""
from pathlib import Path
import argparse
import json

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('root', type=Path)
parser.add_argument('--files', type=int, default=5000)
parser.add_argument('--lines', type=int, default=1000, help='Physical lines per file')
args = parser.parse_args()
if args.files < 1 or args.lines < 1:
    parser.error('files and lines must be positive')
if args.root.exists() and any(args.root.iterdir()):
    parser.error('use a new or empty fixture directory; no files will be deleted')
args.root.mkdir(parents=True, exist_ok=True)
extensions = ('wave', 'rs', 'cc', 'py', 'ts')
for index in range(args.files):
    directory = args.root / f'module-{index // 100:05}'
    directory.mkdir(exist_ok=True)
    extension = extensions[index % len(extensions)]
    if extension in ('wave', 'rs'):
        keyword = 'fun' if extension == 'wave' else 'fn'
        text = ''.join(f'{keyword} f_{index}_{line}() {{ let value = {line}; }} // source {line}\n' for line in range(args.lines))
    elif extension == 'cc':
        text = ''.join(f'int f_{index}_{line}() {{ return {line}; }} // source {line}\n' for line in range(args.lines))
    elif extension == 'py':
        text = ''.join(f'def f_{index}_{line}(): return {line}  # source {line}\n' for line in range(args.lines))
    else:
        text = ''.join(f'export const f_{index}_{line} = () => {line}; // source {line}\n' for line in range(args.lines))
    (directory / f'unit-{index:07}.{extension}').write_text(text, encoding='utf-8')
print(json.dumps({'fixture': 'generated source, not a real repository', 'files': args.files,
                  'lines_per_file': args.lines, 'physical_lines': args.files * args.lines,
                  'languages': extensions}))
