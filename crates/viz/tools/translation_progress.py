#!/usr/bin/env python3
from __future__ import annotations

import re
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHROMIUM = (ROOT / "../../../src").resolve()
MANIFEST = ROOT / "SOURCE_FILES.txt"
ANNOTATION = re.compile(r"// Source: (?P<path>components/viz/[^:]+):(.*)")
RANGE = re.compile(r"(\d+)(?:-(\d+))?")
# A whole-file marker can only prove that somebody typed a comment.  Keep
# source regions small enough that the Rust immediately below can be reviewed
# against the cited Chromium block.  This is deliberately stricter than line
# accounting used by older translated packages.
MAX_REVIEWABLE_REGION = 160

sources = [line.strip() for line in MANIFEST.read_text().splitlines() if line.strip()]
covered: dict[str, set[int]] = defaultdict(set)
targets: dict[str, set[Path]] = defaultdict(set)
oversized: list[tuple[Path, str, int, int]] = []
regions = 0
for rust in (ROOT / "src").rglob("*.rs"):
    for line in rust.read_text(errors="ignore").splitlines():
        match = ANNOTATION.search(line)
        if not match:
            continue
        path = match.group("path")
        for token in RANGE.finditer(match.group(2)):
            start = int(token.group(1))
            end = int(token.group(2) or start)
            if end < start or end - start + 1 > MAX_REVIEWABLE_REGION:
                oversized.append((rust.relative_to(ROOT), path, start, end))
                continue
            regions += 1
            covered[path].update(range(start, end + 1))
            targets[path].add(rust.relative_to(ROOT))

source_lines = translated = 0
complete = []
missing = []
for path in sources:
    physical = sum(1 for _ in (CHROMIUM / path).open(errors="ignore"))
    source_lines += physical
    count = len({line for line in covered[path] if 1 <= line <= physical})
    translated += count
    if count == physical:
        complete.append(path)
    else:
        missing.append((path, physical - count))

target_files = set().union(*targets.values()) if targets else set()
source_pairs = {re.sub(r"\.(?:h|cc)$", "", path) for path in sources}
print(f"source_files={len(sources)}")
print(f"source_pairs={len(source_pairs)}")
print(f"rust_files={len(target_files)}")
print(f"rust_lines={sum(len((ROOT / path).read_text(errors='ignore').splitlines()) for path in target_files)}")
print(f"reviewable_source_regions={regions}")
print(f"source_lines={source_lines}")
print(f"translated_lines={translated}")
print(f"remaining_lines={source_lines - translated}")
print(f"complete_source_files={len(complete)}")
print(f"oversized_unreviewable_regions={len(oversized)}")
print("in_progress:")
for path, remaining in missing:
    print(f"  {path}: {remaining} lines")
if oversized:
    print("oversized_regions:")
    for target, path, start, end in oversized:
        print(f"  {target}: {path}:{start}-{end}")
