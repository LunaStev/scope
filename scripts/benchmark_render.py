"""A/B CPU-submission benchmark. Run under Xvfb with two release executables.

No FPS claim: CPU paint timing excludes driver completion and display latency.
Both executables use the same source fixture, window size and input sequence.
"""
from pathlib import Path
import json
import os
import re
import statistics
import subprocess
import sys
import tempfile
import time

FRAME = re.compile(r"scope perf: cpu_ms=([\d.]+) built=(\d+) reused=(\d+) pending=(\d+) submitted_runs=(\d+) geometry_bytes=(\d+)")

def frames(path):
    text = path.read_text(errors="replace") if path.exists() else ""
    return [dict(zip(("cpu_ms", "built", "reused", "pending", "submitted_runs", "geometry_bytes"), (float(a), int(b), int(c), int(d), int(e), int(f)))) for a,b,c,d,e,f in FRAME.findall(text)]

def command(*args):
    return subprocess.check_output(["xdotool", *map(str,args)], stderr=subprocess.DEVNULL, text=True).strip()

def measure(binary, fixture, label):
    env = dict(os.environ, LIBGL_ALWAYS_SOFTWARE="1", SCOPE_PERF="1")
    env.pop("SCOPE_TRACE", None)
    path = Path(f"perf-{label}.log")
    with path.open("w") as log:
        process = subprocess.Popen([str(Path(binary).resolve()), str(fixture)], stdout=log, stderr=log, env=env)
        try:
            started=time.monotonic()
            while True:
                if process.poll() is not None: raise AssertionError(path.read_text())
                data=frames(path)
                if data and data[-1]["pending"] == 0: break
                if time.monotonic()-started > 180: raise TimeoutError(f"{label}: source preparation timeout")
                time.sleep(0.25)
            preparation=time.monotonic()-started
            wid=command("search", "--pid", process.pid, "--name", "Scope").splitlines()[0]
            command("windowmove", wid, 0, 0)
            command("windowsize", wid, 1440, 900)
            command("mousemove", "--window", wid, 580, 500)
            time.sleep(2)
            # Warm all geometry at overview after the resize; baseline warms its
            # internal text-layout caches through the exact same input events.
            for _ in range(3):
                command("click", 4); time.sleep(0.3)
                command("click", 5); time.sleep(0.3)
            time.sleep(3)
            offset=len(frames(path))
            for i in range(24):
                command("mousemove", "--window", wid, 580+(i%2)*8, 500)
                command("click", 4 if i%2==0 else 5)
                time.sleep(0.25)
            time.sleep(3)
            data=frames(path)[offset:]
            warm=[f for f in data if f["pending"]==0 and f["built"]==0]
            if len(warm)<8: raise AssertionError(f"{label}: only {len(warm)} warm samples")
            before=len(frames(path));time.sleep(2);idle=len(frames(path))-before
            values=sorted(f["cpu_ms"] for f in warm)
            result={"samples":len(warm),"cpu_median_ms":statistics.median(values),"cpu_p95_ms":values[min(len(values)-1,int(len(values)*0.95))],"initial_ready_seconds":round(preparation,3),"idle_redraws_in_2_seconds":idle,"max_warm_submitted_runs":max(f["submitted_runs"] for f in warm),"max_geometry_bytes":max(f["geometry_bytes"] for f in data)}
            if label=="after":
                assert result["max_warm_submitted_runs"]==0, "Camera motion rebuilt source glyphs"
                assert any(f["reused"]>0 for f in warm), "No retained geometry was reused"
                assert idle<=2, "Idle rendering loop did not settle"
            return result
        finally:
            process.terminate()
            try: process.wait(timeout=8)
            except subprocess.TimeoutExpired: process.kill();process.wait()

with tempfile.TemporaryDirectory(prefix="scope-render-bench-") as directory:
    root=Path(directory)
    for file in range(40):
        folder=root/f"module_{file//5}";folder.mkdir(exist_ok=True)
        (folder/f"source_{file}.rs").write_text("".join(f"fn function_{line}() {{ let x = {line}; }} // source {file}\n" for line in range(500)))
    report={"baseline_commit":"455a32084163cd17b42fb219781d7197df322a49","profile":"release","renderer":"Mesa software OpenGL under Xvfb","window":[1440,900],"fixture":{"files":40,"physical_lines":20000},"scope":"CPU map submission only; not end-to-end frame time or FPS"}
    report["before"]=measure(sys.argv[1],root,"before")
    report["after"]=measure(sys.argv[2],root,"after")
    report["median_speedup"]=report["before"]["cpu_median_ms"]/max(report["after"]["cpu_median_ms"],0.000001)
    Path("performance.json").write_text(json.dumps(report,indent=2)+"\n")
    print(json.dumps(report,indent=2))
