"""Generate a deterministic mixed-language 5M-line test; no network downloads."""
from pathlib import Path
import sys
root=Path(sys.argv[1]);root.mkdir(parents=True,exist_ok=True)
for directory in range(50):
    folder=root/f'module-{directory:03}';folder.mkdir(exist_ok=True)
    for file in range(100):
        ext='wave' if file%4==0 else 'rs'
        name='fun' if ext=='wave' else 'fn'
        text=''.join(f'{name} f_{directory}_{file}_{i}() {{ let value = {i}; }} // source {i}\n' for i in range(1000))
        (folder/f'unit-{file:03}.{ext}').write_text(text)
print('5000 files / 5,000,000 lines generated')
