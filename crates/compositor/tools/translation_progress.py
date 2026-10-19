#!/usr/bin/env python3
"""Report physical Chromium source-line coverage from Source annotations."""
from __future__ import annotations
from collections import defaultdict
from pathlib import Path
import re
import tomllib

crate = Path(__file__).resolve().parents[1]
with (crate / "SOURCE_BASELINE.toml").open("rb") as f:
    baseline = tomllib.load(f)
source_root = Path(baseline["source_root"])
source_paths = [
    line.strip()
    for line in (crate / "SOURCE_FILES.txt").read_text().splitlines()
    if line.strip() and not line.lstrip().startswith("#")
]
source_re = re.compile(
    r"Source:\s+((?:cc|third_party/blink/renderer/platform/graphics/(?:compositing|paint))/[^:]+\.(?:h|cc)):(\d+)(?:-(\d+))?"
)

def target_for(source: str) -> Path:
    path = Path(source)
    return crate / "src" / path.with_suffix(".rs")

coverage: dict[str, set[int]] = defaultdict(set)
targets = {source: target_for(source) for source in source_paths}
for target in sorted(set(targets.values())):
    if not target.exists():
        continue
    text = target.read_text(errors="ignore")
    target_complete = "// Translation status: complete" in text
    if target_complete:
        for source in source_paths:
            if targets[source] == target:
                total = len((source_root / source).read_text(errors="ignore").splitlines())
                coverage[source].update(range(1, total + 1))
    for match in source_re.finditer(text):
        source, begin, end = match.group(1), int(match.group(2)), match.group(3)
        end_line = int(end) if end else begin
        source_total = len((source_root / source).read_text(errors="ignore").splitlines())
        # A file-level provenance header identifies the source but does not
        # claim completion. Only the explicit complete marker can do that.
        if not target_complete and begin == 1 and end_line == source_total:
            continue
        coverage[source].update(range(begin, end_line + 1))

rows = []
for source in source_paths:
    path = source_root / source
    total = len(path.read_text(errors="ignore").splitlines())
    covered = len({line for line in coverage[source] if 1 <= line <= total})
    rows.append((source, total, covered, total - covered, targets[source]))

target_rows = defaultdict(list)
for row in rows:
    target_rows[row[4]].append(row)
complete_targets = [
    target for target, source_rows in target_rows.items()
    if target.exists() and all(row[3] == 0 for row in source_rows)
]
partial_targets = [
    (target, sum(r[3] for r in source_rows))
    for target, source_rows in target_rows.items()
    if target.exists() and any(r[3] for r in source_rows)
]
missing_targets = [
    target for target in target_rows
    if not target.exists()
]
expected = baseline.get("expected", {})
actual = {"source_files": len(rows), "rust_files": len(target_rows),
          "source_lines": sum(r[1] for r in rows)}
for key, value in expected.items():
    if actual[key] != value:
        raise SystemExit(f"baseline {key}: expected {value}, got {actual[key]}")
print(f"source_files={len(rows)}")
print(f"rust_files={len(target_rows)}")
print(f"source_lines={sum(r[1] for r in rows)}")
print(f"translated_lines={sum(r[2] for r in rows)}")
print(f"remaining_lines={sum(r[3] for r in rows)}")
print(f"complete_source_files={sum(r[3] == 0 for r in rows)}")
print(f"complete_rust_files={len(complete_targets)}")
print(f"missing_rust_files={len(missing_targets)}")
print("completed:")
for target in complete_targets:
    print(f"  {target.relative_to(crate)}")
print("in_progress:")
for target, remaining in partial_targets:
    print(f"  {target.relative_to(crate)} remaining={remaining}")
print("missing:")
for target in missing_targets:
    print(f"  {target.relative_to(crate)}")
