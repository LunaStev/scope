"""Matched native-window comparison: distinguish full-map preparation from
interactive image submission. Never label CPU/input measurements as GPU FPS."""
from pathlib import Path
import json,math,os,platform,re,statistics,subprocess as sp,sys,tempfile,time
OUT=Path('evidence');OUT.mkdir(exist_ok=True)
ENV=dict(os.environ,LIBGL_ALWAYS_SOFTWARE='1',SCOPE_TRACE='1',SCOPE_INPUT_TRACE='1')
def xdo(*args):return sp.check_output(['xdotool',*map(str,args)],text=True).strip()
def records(path):
    frames,inputs,clock=[],[],0
    for line in path.read_text(errors='replace').splitlines():
        if line.startswith('scope clock:'):clock=int(line.split('timestamp_ns=')[1])
        elif line.startswith('scope frame:'):
            item=dict(re.findall(r'(\w+)=([^ ]+)',line));item['timestamp_ns']=clock;frames.append(item)
        elif line.startswith('scope input:'):inputs.append(int(line.split('timestamp_ns=')[1]))
    return frames,inputs

def wait(proc,predicate,timeout=180):
    deadline=time.monotonic()+timeout
    while time.monotonic()<deadline:
        if proc.poll() is not None:raise AssertionError(f'Application exited: {proc.returncode}')
        value=predicate()
        if value:return value
        time.sleep(.02)
    raise TimeoutError('Map preparation did not finish')
def summary(samples):
    v=sorted(samples)
    return {'samples':len(v),'median_ms':statistics.median(v),'p95_ms':v[max(0,math.ceil(len(v)*.95)-1)],'max_ms':max(v)}
def run(binary,root,label,enforce):
    log=OUT/f'reveal-{label}.log'
    with log.open('w') as handle:
        launched=time.monotonic();proc=sp.Popen([str(Path(binary).resolve()),str(root)],env=ENV,stdout=handle,stderr=sp.STDOUT)
        latencies=[]
        try:
            wait(proc,lambda:records(log)[0]);ready_seconds=time.monotonic()-launched
            first=records(log)[0][0]
            wid=xdo('search','--pid',proc.pid,'--name','Scope').splitlines()[0];xdo('windowmove',wid,0,0)
            for i in range(24):
                xdo('mousemove',580,450);old=len(records(log)[1]);sent=time.time_ns();xdo('click',4 if i%2==0 else 5)
                received=wait(proc,lambda:records(log)[1][old:],30)[0];latencies.append((received-sent)/1e6);time.sleep(.08)
            frames,_=records(log)
            if enforce:
                assert all(f['map_ready']=='1' and int(f['image_tiles'])>=1 for f in frames), 'No camera frame may lose its backing image'
                assert all(int(f['uploads'])<=1 for f in frames), 'Upload admission must remain bounded'
                assert all(int(f['glyphs'])==0 and f['built']=='0' for f in frames), 'Source glyphs must not be constructed during navigation'
                assert int(first['lines'])==4_000_000 and first['map_errors']=='0', 'First disclosed map must cover the whole fixture'
            sp.run(['import','-window',wid,str(OUT/f'reveal-{label}.png')],check=True)
            value=log.read_text(errors='replace')
            assert not re.search(r'panicked|error parsing live|error expanding|error applying|shader compilation failed|Mismatch in drawlist',value,re.I),value[-5000:]
            return {'startup_to_first_map_seconds':ready_seconds,'first_map_is_complete':enforce,
                    'map_cache_hit':first.get('map_cache_hit'),'map_prepare_ms':first.get('map_prepare_ms'),
                    'first_map_submission_ms':float(first['cpu_ms']),
                    'first_30_map_submissions':summary([float(f['cpu_ms']) for f in frames[:30]]),
                    'all_map_submissions':summary([float(f['cpu_ms']) for f in frames]),
                    'input_handler_latency':summary(latencies),'input_handler_samples_ms':latencies,
                    'rendered_source_lines_max':max(int(f['lines']) for f in frames),
                    'max_image_draws':max(int(f.get('image_tiles',0)) for f in frames),'raw_log':log.name}
        finally:
            proc.terminate()
            try:proc.wait(timeout=10)
            except sp.TimeoutExpired:proc.kill();proc.wait()
with tempfile.TemporaryDirectory(prefix='scope-map-perf-') as folder:
    root=Path(folder)/'input';root.mkdir();ENV['SCOPE_MAP_CACHE_DIR']=str(Path(folder)/'cache')
    template=''.join(f'fun function_{i}() {{ let value = {i}; }} // code\n' for i in range(1000))
    for i in range(4000):
        directory=root/f'module_{i//40:03}';directory.mkdir(exist_ok=True);(directory/f'file_{i:05}.wave').write_text(template)
    report={'schema_version':2,'fixture':{'files':4000,'physical_lines':4_000_000,'language':'Wave'},
            'environment':{'os':platform.platform(),'renderer':'LIBGL_ALWAYS_SOFTWARE=1, Xvfb','build':'release; same host'},
            'scope':'Preparation wall time, CPU map submission and native input latency; not GPU time or FPS. Before shows a partial map, after waits for a complete map.',
            'before':run(sys.argv[1],root,'before',False),
            'after_cold':run(sys.argv[2],root,'after-cold',True),
            'after_warm':run(sys.argv[2],root,'after-warm',True)}
    assert report['after_warm']['map_cache_hit']=='1', 'Second process must reuse the persistent map'
    (OUT/'first-paint-performance.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
