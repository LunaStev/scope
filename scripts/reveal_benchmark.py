"""Measure the interactive post-index path, not headless index throughput.

Both binaries run on the same generated fixture with software OpenGL. Trace
clocks mark CPU submission, NOT GPU completion. Input latency ends at the native
scroll handler, not compositor presentation. Every raw sample is preserved.
"""
from pathlib import Path
import json
import math
import os
import platform
import re
import statistics
import subprocess as sp
import sys
import tempfile
import time

OUT = Path('evidence')
OUT.mkdir(exist_ok=True)
ENV = dict(os.environ, LIBGL_ALWAYS_SOFTWARE='1', SCOPE_TRACE='1', SCOPE_INPUT_TRACE='1')

def xdo(*args):
    return sp.check_output(['xdotool', *map(str,args)], text=True).strip()

def records(path):
    frames, inputs, clock = [], [], 0
    for line in path.read_text(errors='replace').splitlines():
        if line.startswith('scope clock:'):
            clock = int(line.split('timestamp_ns=')[1])
        elif line.startswith('scope frame:'):
            item = dict(re.findall(r'(\w+)=([^ ]+)',line))
            item['timestamp_ns'] = clock
            frames.append(item)
        elif line.startswith('scope input:'):
            inputs.append(int(line.split('timestamp_ns=')[1]))
    return frames, inputs

def wait(proc, predicate, timeout=30):
    deadline = time.monotonic()+timeout
    while time.monotonic()<deadline:
        if proc.poll() is not None:
            raise AssertionError(f'Application exited: {proc.returncode}')
        value = predicate()
        if value:
            return value
        time.sleep(.015)
    raise TimeoutError('No progress while measuring first paint')

def summary(samples):
    values = sorted(samples)
    return {'samples': len(values), 'median_ms': statistics.median(values),
            'p95_ms': values[max(0, math.ceil(len(values)*.95)-1)], 'max_ms': max(values)}

def run(binary, root, label, enforce):
    log = OUT/f'reveal-{label}.log'
    with log.open('w') as handle:
        proc = sp.Popen([str(Path(binary).resolve()), str(root)], env=ENV,
                        stdout=handle, stderr=sp.STDOUT)
        latencies = []
        try:
            # Do not wait for all glyphs: interact as soon as the map appears.
            first = wait(proc, lambda: records(log)[0])
            wid = xdo('search','--pid',proc.pid,'--name','Scope').splitlines()[0]
            xdo('windowmove',wid,0,0)
            for i in range(24):
                # First exercise input while the original view is filling;
                # then alternate zoom directions on the map itself.
                x = 1340 if i<16 else 580
                xdo('mousemove',x,450)
                old = len(records(log)[1])
                sent = time.time_ns()
                xdo('click',4 if i%2==0 else 5)
                received = wait(proc,lambda: records(log)[1][old:],timeout=30)[0]
                latencies.append((received-sent)/1_000_000)
                time.sleep(.08)
            time.sleep(.5)
            frames, _ = records(log)
            sp.run(['import','-window',wid,str(OUT/f'reveal-{label}.png')],check=True)
            assert frames and all(float(f['cpu_ms'])>=0 for f in frames)
            if enforce:
                for f in frames:
                    assert int(f['node_visits'])<=2048, f
                    assert int(f['label_visits'])<=256, f
                    assert int(f['checks'])<=128, f
                    assert int(f['built'])<=2, f
                    assert int(f['new_glyphs'])<=8192, f
                assert any(f['layers_built']=='0' and int(f['layers_reused'])>0 for f in frames)
                assert any(int(f['nodes'])>3000 for f in frames), 'Map discovery must finish, not stay blank'
                assert any(int(f['lines'])>0 for f in frames), 'Real source must still be rendered'
            value = log.read_text(errors='replace')
            assert not re.search(r'panicked|error parsing live|error expanding|error applying|shader compilation failed|Mismatch in drawlist',value,re.I),value[-5000:]
            first_time = frames[0]['timestamp_ns']
            largest_count = max(int(f['nodes']) for f in frames)
            full = next(f for f in frames if int(f['nodes'])==largest_count)
            return {
                'first_map_submission_ms': float(frames[0]['cpu_ms']),
                'first_30_map_submissions': summary([float(f['cpu_ms']) for f in frames[:30]]),
                'all_map_submissions': summary([float(f['cpu_ms']) for f in frames]),
                'input_handler_latency': summary(latencies),
                'input_handler_samples_ms': latencies,
                'largest_visible_node_count': largest_count,
                'ms_from_first_partial_map_to_largest_node_set': (full['timestamp_ns']-first_time)/1_000_000,
                'rendered_source_lines_max': max(int(f['lines']) for f in frames),
                'max_resident_glyphs': max(int(f['glyphs']) for f in frames),
                'raw_log': log.name,
            }
        finally:
            proc.terminate()
            try: proc.wait(timeout=10)
            except sp.TimeoutExpired: proc.kill();proc.wait()

with tempfile.TemporaryDirectory(prefix='scope-reveal-') as folder:
    root = Path(folder)
    template = ''.join(f'fun function_{i}() {{ let value = {i}; }} // code\n' for i in range(1000))
    for i in range(4000):
        directory = root/f'module_{i//40:03}'
        directory.mkdir(exist_ok=True)
        (directory/f'file_{i:05}.wave').write_text(template)
    report = {
        'schema_version': 1,
        'fixture': {'files':4000,'physical_lines':4_000_000,'language':'Wave',
                    'description':'Generated equal-size code files in 100 directories'},
        'environment': {'os':platform.platform(),'renderer':'LIBGL_ALWAYS_SOFTWARE=1, Xvfb',
                        'build':'release; same lockfile and host'},
        'scope':'CPU map submission and native input-handler latency after indexing; not GPU time or FPS',
        'before':run(sys.argv[1],root,'before',False),
        'after':run(sys.argv[2],root,'after',True),
    }
    (OUT/'first-paint-performance.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))
