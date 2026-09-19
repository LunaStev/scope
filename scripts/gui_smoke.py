"""Exercise real source, retained geometry and compact UI under Xvfb."""
from pathlib import Path
import os
import re
import subprocess as sp
import tempfile
import time

OUT=Path('evidence');OUT.mkdir(exist_ok=True)
BINARY=str(Path('target/debug/scope').resolve())
ENV=dict(os.environ,LIBGL_ALWAYS_SOFTWARE='1',SCOPE_TRACE='1')
def xdo(*args):return sp.check_output(['xdotool',*map(str,args)],text=True).strip()
def frames(path):
    return [dict(re.findall(r'(\w+)=([^ ]+)',line)) for line in path.read_text(errors='replace').splitlines() if line.startswith('scope frame:')]
def wait_ready(proc,log,minimum=1,timeout=60):
    end=time.monotonic()+timeout
    while time.monotonic()<end:
        if proc.poll() is not None:raise AssertionError(log.read_text(errors='replace'))
        r=frames(log)
        if r and int(r[-1].get('lines',0))>=minimum and r[-1].get('pending')=='0':return r[-1]
        time.sleep(.1)
    raise AssertionError('Geometry did not complete: '+log.read_text(errors='replace')[-4000:])
def launch(root,name,minimum):
    log=OUT/(name+'.log');file=log.open('w')
    proc=sp.Popen([BINARY,str(root)],env=ENV,stdout=file,stderr=sp.STDOUT)
    try:
        record=wait_ready(proc,log,minimum)
        wid=xdo('search','--pid',proc.pid,'--name','Scope').splitlines()[0]
        xdo('windowmove',wid,0,0)
        # The trace precedes presentation. Allow a completed warm redraw before
        # capturing, rather than saving a still-pending previous frame.
        xdo('mousemove',580,450);time.sleep(.5)
        record=wait_ready(proc,log,minimum);time.sleep(.15)
        return proc,file,log,wid,record
    except BaseException:
        proc.terminate();proc.wait(timeout=10);file.close();raise

def stop(proc,file):
    proc.terminate()
    try:proc.wait(timeout=10)
    except sp.TimeoutExpired:proc.kill();proc.wait()
    file.close()

with tempfile.TemporaryDirectory(prefix='scope-gui-') as folder:
    root=Path(folder)
    (root/'main.rs').write_text(''.join(f'fn f_{i}() {{ let x = {i}; }} // line {i}\n' for i in range(1200)))
    proc,file,log,wid,before=launch(root,'source',1200)
    try:
        assert int(before['lines'])==1200 and 0<float(before['font_min'])<7
        sp.run(['import','-window',wid,str(OUT/'scope-source-overview.png')],check=True)
        xdo('mousemove',580,480,'click','--repeat',2,'--delay',100,1)
        time.sleep(1);read=wait_ready(proc,log)
        assert float(read['font_max'])>=11.9,read
        assert any(r['reused']!='0' and r['built']=='0' for r in frames(log)), 'Camera must reuse geometry'
        sp.run(['import','-window',wid,str(OUT/'scope-source-read.png')],check=True)
        xdo('key','Home');time.sleep(.5)
        xdo('windowsize',wid,960,640);time.sleep(1)
        sp.run(['import','-window',wid,str(OUT/'scope-compact.png')],check=True)
    finally:stop(proc,file)
    (root/'blank.rs').write_text('\n'*300)
    proc,file,log,wid,_=launch(root,'blank',1500)
    try:
        xdo('mousemove',450,350,'mousedown',1,'mousemove',460,350,'mouseup',1);time.sleep(.5)
        assert proc.poll() is None
    finally:stop(proc,file)

proc,file,log,wid,_=launch(Path.cwd(),'repository',1)
try:sp.run(['import','-window',wid,str(OUT/'scope-overview.png')],check=True)
finally:stop(proc,file)
for log in OUT.glob('*.log'):
    value=log.read_text(errors='replace')
    assert not re.search(r'panicked|error expanding|error applying|shader compilation failed|Mismatch in drawlist',value,re.I),value
print('Actual overview source, cached pan/zoom, readable double-click, blank source and compact UI passed.')
