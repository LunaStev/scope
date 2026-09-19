"""Enforce root-level module names and dependency direction."""
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[1]
allowed = {
    "model": set(),
    "analysis": {"model"},
    "layout": {"model"},
    "runtime": {"model", "analysis", "layout"},
    "render": {"model", "layout", "runtime"},
    "ui": {"model", "analysis", "layout", "runtime", "render"},
    "scope": {"model", "analysis", "ui"},
}
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=root))
packages = {p["name"]: p for p in metadata["packages"]}
assert set(packages) == set(allowed), "Register new modules in the dependency contract."
for name, package in packages.items():
    directory = "app" if name == "scope" else name
    assert Path(package["manifest_path"]).resolve() == root / directory / "Cargo.toml", name
    for dependency in package["dependencies"]:
        target = dependency["name"]
        if target in allowed:
            assert target in allowed[name], f"Forbidden dependency: {name} -> {target}"
        if name in {"model", "analysis", "layout", "runtime"}:
            assert not target.startswith("makepad"), f"GUI dependency leaked into {name}"
print("Root module names, manifest paths and dependency boundaries verified.")
