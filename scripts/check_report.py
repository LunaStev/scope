"""Validate that all CLI/UI physical-line categories have the same denominator."""
import json
import sys
with open(sys.argv[1], encoding="utf-8") as file:
    report = json.load(file)
assert report["schema_version"] == 1
metrics = report["metrics"]
assert metrics["lines"] == sum(metrics[k] for k in ("code", "comments", "blanks", "unclassified"))
for key in metrics:
    assert metrics[key] == sum(lang["stats"][key] for lang in report["languages"]), key
print(f"Accounting contract passed for {metrics['files']} files / {metrics['lines']} lines.")
