"""Enforce workspace dependency direction without importing GUI code."""
import json
import subprocess

allowed = {
    "scope-core": set(),
    "scope-analysis": {"scope-core"},
    "scope-layout": {"scope-core"},
    "scope-runtime": {"scope-core", "scope-analysis", "scope-layout"},
    "scope-render": {"scope-core", "scope-analysis", "scope-layout", "scope-runtime"},
    "scope-ui": {"scope-core", "scope-analysis", "scope-layout", "scope-runtime", "scope-render"},
    "scope": {"scope-core", "scope-analysis", "scope-ui"},
}
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"]))
packages = {p["name"]: p for p in metadata["packages"]}
assert set(packages) == set(allowed), "Register new modules and boundaries in this check."
for name, package in packages.items():
    for dependency in package["dependencies"]:
        target = dependency["name"]
        if target in allowed:
            assert target in allowed[name], f"Forbidden dependency: {name} -> {target}"
        if name in {"scope-core", "scope-analysis", "scope-layout", "scope-runtime"}:
            assert not target.startswith("makepad"), f"GUI dependency leaked into {name}"
print("Seven workspace packages; dependency boundaries verified.")
