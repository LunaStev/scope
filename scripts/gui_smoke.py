"""Complete backing, native SDF source and cache reuse in a real window."""
from pathlib import Path
import os,re,subprocess as sp,tempfile,time
OUT=Path('evidence');OUT.mkdir(exist_ok=True)
BINARY=str(Path('target/debug/scope').resolve())
ENV=dict(os.environ,LIBGL_ALWAYS_SOFTWARE='1',SCOPE_TRACE='1',SCOPE_MAP_CACHE_DIR=str((OUT/'map-cache').resolve()))
def xdo(*args):return sp.check_output(['xdotool',*map(str,args)],text=True).strip()
def frames(path):return [dict(re.findall(r'(\w+)=([^ ]+)',line)) for line in path.read_text(errors='replace').splitlines() if line.startswith('scope frame:')]
def wait_ready(proc,log,minimum=1,timeout=180,settled=True):
    end=time.monotonic()+timeout
    while time.monotonic()<end:
        if proc.poll() is not None:raise AssertionError(log.read_text(errors='replace')[-6000:])
        records=frames(log)
        if records:
            r=records[-1]
            if r.get('map_ready')=='1' and int(r.get('lines',0))>=minimum and (not settled or (r.get('map_pending')=='0' and r.get('native_pending','0')=='0' and r.get('redraw')=='0')):return r
        time.sleep(.1)
    raise AssertionError('Map did not complete: '+log.read_text(errors='replace')[-6000:])
def launch(root,name,minimum):
    log=OUT/(name+'.log');file=log.open('w');proc=sp.Popen([BINARY,str(root)],env=ENV,stdout=file,stderr=sp.STDOUT)
    try:
        record=wait_ready(proc,log,minimum);wid=xdo('search','--pid',proc.pid,'--name','Scope').splitlines()[0]
        xdo('windowmove',wid,0,0);xdo('mousemove',580,450);time.sleep(.25)
        return proc,file,log,wid,wait_ready(proc,log,minimum)
    except BaseException:proc.terminate();proc.wait(timeout=10);file.close();raise

def stop(proc,file):
    proc.terminate()
    try:proc.wait(timeout=10)
    except sp.TimeoutExpired:proc.kill();proc.wait()
    file.close()
with tempfile.TemporaryDirectory(prefix='scope-map-gui-') as folder:
    root=Path(folder);(root/'main.wave').write_text(''.join(f'fun f_{i}() {{ let x = {i}; }} // line {i}\n' for i in range(1200)))
    proc,file,log,wid,before=launch(root,'source',1200)
    try:
        assert int(before['lines'])==1200 and before['map_errors']=='0'
        assert before['representation']=='hybrid-sdf'
        assert before['image_tiles']=='1',before
        sp.run(['import','-window',wid,str(OUT/'scope-source-overview.png')],check=True)
        n=len(frames(log));xdo('mousemove',580,480,'click','--repeat',2,'--delay',100,1);time.sleep(.25)
        pending=wait_ready(proc,log,1200,settled=False)
        assert pending['map_ready']=='1' and int(pending['image_tiles'])>=1
        read=wait_ready(proc,log,1200)
        assert float(read['font_max'])>=11.9,read
        assert int(read['native_glyphs'])>0,'Readable source must use GPU glyphs, not only an image'
        assert int(read['native_resident'])>0
        mark=len(frames(log));xdo('click',4);time.sleep(.25);closer=wait_ready(proc,log,1200)
        assert int(closer['native_reused'])>0
        assert all(int(r['built'])==0 for r in frames(log)[mark:]),'Warm inward zoom rebuilt glyphs'
        xdo('click',5);time.sleep(.25);read=wait_ready(proc,log,1200)
        assert all(r.get('map_ready')=='1' and int(r.get('image_tiles',0))>=1 for r in frames(log)[n:])
        sp.run(['import','-window',wid,str(OUT/'scope-source-read.png')],check=True)
        xdo('key','Home');time.sleep(.3);restored=wait_ready(proc,log,1200);assert int(restored['lines'])==1200
        xdo('mousemove',580,480,'click',5);time.sleep(.2);r=wait_ready(proc,log,1200)
        assert r['built']=='0' and r['glyphs']=='0'
        xdo('key','Home');time.sleep(.3)
    finally:stop(proc,file)
    proc,file,log,wid,warm=launch(root,'warm',1200)
    try:
        assert warm['map_cache_hit']=='1',warm
        xdo('windowsize',wid,960,640);time.sleep(.5);wait_ready(proc,log,1200)
        sp.run(['import','-window',wid,str(OUT/'scope-compact.png')],check=True)
    finally:stop(proc,file)
    with (root/'main.wave').open('a') as f:f.write('// changed\n')
    proc,file,log,wid,changed=launch(root,'changed',1201)
    try:assert changed['map_cache_hit']=='0'
    finally:stop(proc,file)
proc,file,log,wid,_=launch(Path.cwd(),'repository',1)
try:sp.run(['import','-window',wid,str(OUT/'scope-overview.png')],check=True)
finally:stop(proc,file)
for log in OUT.glob('*.log'):
    value=log.read_text(errors='replace')
    assert not re.search(r'panicked|error expanding|error applying|shader compilation failed|Mismatch in drawlist',value,re.I),value[-5000:]
print('Complete backing, live GPU source, warm glyph reuse, cached restart and edit invalidation passed.')
