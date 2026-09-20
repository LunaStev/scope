"""Cold readable-source A/B, plus warm replay. This is not an FPS benchmark.

Both paths run this commit with the same source, geometry and camera. The
legacy path uses conventional DrawText; atlas uses cached ASCII templates.
A 60K-line file stays below native prefetch density at overview, ensuring that
neither implementation silently warms foreground glyphs before measurement.
"""
from pathlib import Path
import json, math, os, re, statistics, subprocess as sp, sys, tempfile, time
OUT=Path('evidence');OUT.mkdir(exist_ok=True)
BINARY=str(Path(sys.argv[1]).resolve())

def xdo(*args):
    return sp.check_output(['xdotool',*map(str,args)],text=True).strip()

def frames(log):
    return [dict(re.findall(r'(\w+)=([^ ]+)',line)) for line in log.read_text(errors='replace').splitlines() if line.startswith('scope frame:')]

def wait(proc,fn,timeout=120):
    end=time.monotonic()+timeout
    while time.monotonic()<end:
        if proc.poll() is not None:raise AssertionError('Scope exited during sharpness test')
        result=fn()
        if result:return result
        time.sleep(.01)
    raise TimeoutError('Source did not become ready')

def summary(values):
    values=sorted(values)
    return dict(samples=len(values),median=statistics.median(values),p95=values[math.ceil(.95*len(values))-1],maximum=max(values))

def run(root,cache,path,trial):
    log=OUT/f'sharp-{path}-{trial}.log'
    env=dict(os.environ,LIBGL_ALWAYS_SOFTWARE='1',SCOPE_TRACE='1',SCOPE_GLYPH_PATH=path,SCOPE_MAP_CACHE_DIR=str(cache))
    with log.open('w') as output:
        proc=sp.Popen([BINARY,str(root)],env=env,stdout=output,stderr=sp.STDOUT)
        try:
            def overview():
                records=frames(log)
                if not records:return False
                r=records[-1]
                return r if r.get('map_ready')=='1' and r.get('native_pending')=='0' and r.get('redraw')=='0' else False
            before=wait(proc,overview)
            assert int(before['native_resident'])==0,'Benchmark must begin without source glyphs'
            if path=='atlas':assert before['atlas_ready']=='1'
            wid=xdo('search','--pid',proc.pid,'--name','Scope').splitlines()[0]
            xdo('windowmove',wid,0,0);xdo('mousemove',580,480);time.sleep(.1)
            mark=len(frames(log));begin=time.monotonic()
            xdo('click','--repeat',2,'--delay',100,1)
            def completed():
                records=frames(log)[mark:]
                if not records:return False
                r=records[-1]
                return r if float(r.get('font_max',0))>=11.9 and int(r.get('native_glyphs',0))>1000 and r.get('native_pending')=='0' else False
            read=wait(proc,completed)
            foreground_ms=(time.monotonic()-begin)*1000
            cold=frames(log)[mark:]
            assert all(r.get('map_ready')=='1' and int(r['image_tiles'])>0 for r in cold)
            if path=='legacy':assert sum(int(r['fallback_builds']) for r in cold)>0
            else:
                assert int(read['atlas_instances'])>0
                assert sum(int(r['fallback_builds']) for r in cold)==0
                assert all(int(r['atlas_builds'])<=12 for r in cold)
            # Submission traces precede GL presentation. Keep this race-visible
            # capture distinct; never present it as the settled sharp output.
            sp.run(['import','-window',wid,str(OUT/f'submitted-{path}.png')],check=True)
            wait(proc,lambda: frames(log)[-1].get('redraw')=='0' and frames(log)[-1].get('map_pending')=='0')
            time.sleep(.15)
            sp.run(['import','-window',wid,str(OUT/f'sharp-{path}.png')],check=True)
            warm_start=len(frames(log))
            for step in range(8):
                xdo('click',4 if step%2==0 else 5);time.sleep(.1)
            warm=frames(log)[warm_start:]
            assert warm and all(int(r['built'])==0 for r in warm),'Warm foreground rebuilt'
            assert all(int(r.get('atlas_uploads',0))==0 for r in warm),'Atlas was uploaded on zoom'
            assert all(int(r['native_glyphs'])>0 for r in warm)
            return dict(path=path,trial=trial,foreground_complete_ms=foreground_ms,
                glyph_build_cpu_ms=sum(float(r['build_cpu_ms']) for r in cold),
                build_frames=sum(int(r['built'])>0 for r in cold),character_slots=sum(int(r['glyphs']) for r in cold),
                atlas_builds=sum(int(r['atlas_builds']) for r in cold),fallback_builds=sum(int(r['fallback_builds']) for r in cold),
                visible_atlas_instances=int(read.get('atlas_instances',0)),warm_cpu_ms=summary([float(r['cpu_ms']) for r in warm]))
        finally:
            proc.terminate()
            try:proc.wait(timeout=10)
            except sp.TimeoutExpired:proc.kill();proc.wait()

with tempfile.TemporaryDirectory(prefix='scope-sharp-') as directory:
    directory=Path(directory);root=directory/'source';root.mkdir()
    (root/'main.wave').write_text(''.join(f'fun function_{i}() {{ let x = {i}; }} // source {i}\n' for i in range(60000)))
    results=[]
    for trial in range(3):
        for path in (('legacy','atlas') if trial%2==0 else ('atlas','legacy')):
            results.append(run(root,directory/'cache',path,trial))
    assert len({r['character_slots'] for r in results})==1,results
    report=dict(fixture=dict(files=1,physical_lines=60000,language='Wave',alphabet='ASCII'),
        environment='Ubuntu CI / Xvfb / software OpenGL / release build',
        comparison='Same commit: per-run DrawText vs shared SDF glyph templates. Identical source-character slots.',
        scope='Foreground completion ends at CPU submission and includes document preparation, scheduling and 100ms double-click injection; it excludes initial overview/atlas preparation. Glyph build CPU is only foreground assembly. Screenshots are taken after settling separately. No GPU completion time, display latency or FPS.',
        trials=results,aggregate={path:{'foreground_complete_ms':summary([r['foreground_complete_ms'] for r in results if r['path']==path]),
            'glyph_build_cpu_ms':summary([r['glyph_build_cpu_ms'] for r in results if r['path']==path])} for path in ('legacy','atlas')})
    (OUT/'sharp-performance.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
for path in OUT.glob('sharp-*.log'):
    text=path.read_text(errors='replace')
    assert not re.search(r'panicked|error expanding|error applying|shader compilation failed|Mismatch in drawlist|scope atlas: .*failed',text,re.I),text[-5000:]
