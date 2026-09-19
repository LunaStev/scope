"""Matched release-build CPU submission benchmark, not GPU time/FPS.
Run under Xvfb: performance.py BEFORE_BINARY AFTER_BINARY.
"""
import json
import math
import os
from pathlib import Path
import re
import statistics
import subprocess as sp
import sys
import tempfile
import time
out=Path('evidence');out.mkdir(exist_ok=True)
env=dict(os.environ,LIBGL_ALWAYS_SOFTWARE='1',SCOPE_TRACE='1')
def xdo(*args):return sp.check_output(['xdotool',*map(str,args)],text=True).strip()
def records(log):
    result=[];cpu=None
    for line in log.read_text(errors='replace').splitlines():
        if line.startswith('scope perf '):cpu=float(re.search(r'cpu_ms=([\d.]+)',line)[1])
        if line.startswith('scope frame:'):
            r=dict(re.findall(r'(\w+)=([^ ]+)',line));r['cpu_ms']=float(r.get('cpu_ms',cpu or 0));result.append(r)
    return result

def wait(proc,log,predicate):
    end=time.monotonic()+90
    while time.monotonic()<end:
        assert proc.poll() is None,log.read_text(errors='replace')[-5000:]
        r=records(log)
        if r and predicate(r):return r
        time.sleep(.05)
    raise AssertionError('Timed out: '+log.read_text(errors='replace')[-2000:])

def run(binary,label,root):
    log=out/f'performance-{label}.log'
    with log.open('w') as file:
        proc=sp.Popen([str(Path(binary).resolve()),str(root)],env=env,stdout=file,stderr=sp.STDOUT)
        try:
            wait(proc,log,lambda r:int(r[-1]['lines'])==12000 and r[-1].get('pending','0')=='0')
            wid=xdo('search','--pid',proc.pid,'--name','Scope').splitlines()[0]
            xdo('windowmove',wid,0,0);xdo('windowsize',wid,1440,900);time.sleep(.5)
            wait(proc,log,lambda r:r[-1].get('pending','0')=='0')
            sp.run(['import','-window',wid,str(out/f'performance-{label}.png')],check=True)
            xdo('mousemove',570,450,'mousedown',1);time.sleep(.3)
            samples=[]
            for step in range(1,25):
                count=len(records(log));xdo('mousemove',570+(step%2)*3,450+(step%3))
                r=wait(proc,log,lambda r:len(r)>count and r[-1].get('pending','0')=='0');samples.append(r[-1])
            xdo('mouseup',1)
            assert all(int(r['lines'])==12000 for r in samples), 'Different source populations compared'
            if label=='after':assert all(r['built']=='0' and int(r['reused'])>0 for r in samples),samples
            values=sorted(r['cpu_ms'] for r in samples)
            return {'samples':len(values),'physical_lines':12000,'median_cpu_ms':statistics.median(values),'p95_cpu_ms':values[math.ceil(.95*len(values))-1],'min_cpu_ms':min(values),'max_cpu_ms':max(values),'rebuilt_chunks':sum(int(r.get('built',0)) for r in samples)}
        finally:
            proc.terminate()
            try:proc.wait(timeout=10)
            except sp.TimeoutExpired:proc.kill();proc.wait()

with tempfile.TemporaryDirectory(prefix='scope-perf-') as folder:
    root=Path(folder)
    for i in range(24):
        path=root/f'group_{i//6}'/f'module_{i:02}.rs';path.parent.mkdir(exist_ok=True)
        path.write_text(''.join(f'fn f_{j}() {{ let x = {j}; }} // actual line {j}\n' for j in range(500)))
    before=run(sys.argv[1],'before',root);after=run(sys.argv[2],'after',root)
report={'environment':'same Ubuntu Actions runner, release builds, Xvfb, Mesa software OpenGL','measurement':'CPU map submission only, warmed glyph geometry, 24 paired small pan events; not GPU frame time or FPS','p95_method':'nearest rank','before_commit':'455a32084163cd17b42fb219781d7197df322a49','before':before,'after':after}
(out/'performance.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2))
