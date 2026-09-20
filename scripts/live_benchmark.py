"""Native readable-source CPU submission and OS input receipt, not GPU time/FPS."""
from pathlib import Path
import json, math, os, re, statistics, subprocess as sp, sys, tempfile, time
OUT=Path('evidence');OUT.mkdir(exist_ok=True)
BINARY=str(Path(sys.argv[1]).resolve())
def xdo(*args):return sp.check_output(['xdotool',*map(str,args)],text=True).strip()
def frames(log):return [dict(re.findall(r'(\w+)=([^ ]+)',s)) for s in log.read_text(errors='replace').splitlines() if s.startswith('scope frame:')]
def wait(proc,fn,timeout=180):
    end=time.monotonic()+timeout
    while time.monotonic()<end:
        if proc.poll() is not None:raise AssertionError('Scope exited')
        found=fn()
        if found:return found
        time.sleep(.02)
    raise TimeoutError('No completed native source')
def summary(values):
    values=sorted(values)
    return dict(samples=len(values),median_ms=statistics.median(values),p95_ms=values[math.ceil(.95*len(values))-1],max_ms=max(values))
with tempfile.TemporaryDirectory(prefix='scope-live-bench-') as directory:
    root=Path(directory)/'source';root.mkdir()
    (root/'main.wave').write_text(''.join(f'fun function_{i}() {{ let x = {i}; }} // source {i}\n' for i in range(12000)))
    log=OUT/'live-rendering.log'
    env=dict(os.environ,LIBGL_ALWAYS_SOFTWARE='1',SCOPE_TRACE='1',SCOPE_INPUT_TRACE='1',SCOPE_MAP_CACHE_DIR=str(Path(directory)/'cache'))
    with log.open('w') as file:
        proc=sp.Popen([BINARY,str(root)],env=env,stdout=file,stderr=sp.STDOUT)
        try:
            wait(proc,lambda:frames(log))
            wid=xdo('search','--pid',proc.pid,'--name','Scope').splitlines()[0]
            xdo('windowmove',wid,0,0);xdo('mousemove',580,460)
            begin=time.monotonic();xdo('click','--repeat',2,'--delay',100,1)
            first=wait(proc,lambda: next((f for f in reversed(frames(log)) if int(f.get('native_glyphs',0))>0),None))
            first_native_ms=(time.monotonic()-begin)*1000
            wait(proc,lambda: frames(log)[-1] if frames(log)[-1].get('native_pending')=='0' and frames(log)[-1].get('redraw')=='0' else None)
            native_resident=int(frames(log)[-1]['native_resident']);mark=len(frames(log));latencies=[]
            for i in range(24):
                old=len(re.findall(r'scope input: kind=scroll timestamp_ns=(\d+)',log.read_text()))
                sent=time.time_ns();xdo('click',4 if i%2==0 else 5)
                received=wait(proc,lambda: re.findall(r'scope input: kind=scroll timestamp_ns=(\d+)',log.read_text())[old:],30)
                latencies.append((int(received[0])-sent)/1e6);time.sleep(.08)
            records=frames(log)[mark:]
            assert records and all(int(r['image_tiles'])>0 and r['map_ready']=='1' for r in records)
            assert all(int(r['built'])==0 for r in records),'Warm native glyphs were rebuilt'
            assert all(int(r['native_glyphs'])>0 for r in records)
            assert all(int(r['native_resident'])==native_resident for r in records)
            sp.run(['import','-window',wid,str(OUT/'live-native.png')],check=True)
            report=dict(fixture=dict(files=1,lines=12000,language='Wave'),environment='Ubuntu CI / Xvfb / software OpenGL / release',
                scope='Native readable-source CPU submission and OS input receipt; not GPU time or FPS. Synthetic fixture, not Fuchsia.',
                jump_to_first_native_ms=first_native_ms,warm_cpu=summary([float(r['cpu_ms']) for r in records]),
                input_latency=summary(latencies),native_resident_glyphs=native_resident,new_warm_glyph_tiles=sum(int(r['built']) for r in records))
            (OUT/'live-performance.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
        finally:
            proc.terminate()
            try:proc.wait(timeout=10)
            except sp.TimeoutExpired:proc.kill();proc.wait()
