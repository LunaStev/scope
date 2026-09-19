"""A complete 50-million-line map in a native window, not a headless index.
The dataset is generated and OS file caches are not flushed. Separate cold map
creation from persistent image reuse; never present CPU timings as GPU FPS.
"""
from pathlib import Path
import json
import math
import os
import re
import statistics
import subprocess as sp
import sys
import tempfile
import time

OUT=Path('evidence');OUT.mkdir(exist_ok=True)
BINARY=str(Path(sys.argv[1]).resolve());ROOT=Path(sys.argv[2]).resolve()
EXPECTED=int(sys.argv[3])

def frame_records(path):
    return [dict(re.findall(r'(\w+)=([^ ]+)',line)) for line in path.read_text(errors='replace').splitlines() if line.startswith('scope frame:')]
def inputs(path):
    return [int(value) for value in re.findall(r'^scope input: kind=scroll timestamp_ns=(\d+)',path.read_text(errors='replace'),re.M)]
def xdo(*args):return sp.check_output(['xdotool',*map(str,args)],text=True).strip()
def wait(proc,predicate,timeout):
    deadline=time.monotonic()+timeout
    while time.monotonic()<deadline:
        if proc.poll() is not None:raise AssertionError(f'Application exited: {proc.returncode}')
        value=predicate()
        if value:return value
        time.sleep(.04)
    raise TimeoutError('Native map test timed out')
def stats(values):
    values=sorted(values)
    return {'samples':len(values),'median_ms':statistics.median(values),'p95_ms':values[max(0,math.ceil(len(values)*.95)-1)],'max_ms':max(values)}
def run(label,env):
    log=OUT/f'map-50m-{label}.log'
    with log.open('w') as file:
        start=time.monotonic();proc=sp.Popen([BINARY,'--threads','4',str(ROOT)],env=env,stdout=file,stderr=sp.STDOUT)
        try:
            records=wait(proc,lambda:frame_records(log),600)
            elapsed=time.monotonic()-start;first=records[0]
            assert first['map_ready']=='1' and int(first['lines'])==EXPECTED and first['map_errors']=='0',first
            wid=xdo('search','--pid',proc.pid,'--name','Scope').splitlines()[0]
            xdo('windowmove',wid,0,0)
            latencies=[]
            for i in range(12):
                xdo('mousemove',580,450);old=len(inputs(log));sent=time.time_ns()
                xdo('click',5 if i%2==0 else 4)
                received=wait(proc,lambda:inputs(log)[old:],30)[0]
                latencies.append((received-sent)/1e6);time.sleep(.1)
            all_frames=frame_records(log)
            assert all(f['map_ready']=='1' and int(f['image_tiles'])>=1 for f in all_frames)
            assert all(f['built']=='0' and f['glyphs']=='0' and int(f['uploads'])<=1 for f in all_frames)
            sp.run(['import','-window',wid,str(OUT/f'map-50m-{label}.png')],check=True)
            rss=None
            status=Path(f'/proc/{proc.pid}/status')
            if status.exists():
                match=re.search(r'^VmHWM:\s+(\d+) kB',status.read_text(),re.M)
                if match:rss=int(match[1])*1024
            return {'startup_to_complete_map_seconds':elapsed,'map_prepare_ms':int(first['map_prepare_ms']),
                    'map_cache_hit':first['map_cache_hit']=='1','covered_physical_lines':int(first['lines']),
                    'map_errors':int(first['map_errors']),'peak_process_rss_bytes':rss,
                    'map_cpu_submission':stats([float(f['cpu_ms']) for f in all_frames]),
                    'scroll_handler_latency':stats(latencies),'raw_scroll_latency_ms':latencies,
                    'max_image_draws':max(int(f['image_tiles']) for f in all_frames),'raw_log':log.name}
        finally:
            proc.terminate()
            try:proc.wait(timeout=10)
            except sp.TimeoutExpired:proc.kill();proc.wait()

with tempfile.TemporaryDirectory(prefix='scope-scale-cache-') as directory:
    env=dict(os.environ,LIBGL_ALWAYS_SOFTWARE='1',SCOPE_TRACE='1',SCOPE_INPUT_TRACE='1',SCOPE_MAP_CACHE_DIR=directory)
    report={'fixture':{'source':'generated five-language fixture','physical_lines':EXPECTED},
            'environment':'Ubuntu CI, Xvfb, software OpenGL, release build, four index workers; OS cache not flushed',
            'scope':'Native complete-map coverage, startup and CPU/input costs; not GPU time or FPS',
            'cold':run('cold',env),'warm':run('warm',env)}
    assert not report['cold']['map_cache_hit'] and report['warm']['map_cache_hit']
    (OUT/'map-50m-validation.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))
