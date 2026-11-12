#!/usr/bin/env python3
"""Report verified Chromium-to-Rechrom Paint source-line coverage."""

from __future__ import annotations

import argparse
import json
import subprocess
from collections import defaultdict
from pathlib import Path


REPO = Path(__file__).resolve().parents[1]
DEFAULT_MANIFEST = REPO / "crates/paint/chromium_translation.json"
DEFAULT_CHROMIUM = REPO.parent / "src"

PACKAGE_RESPONSIBILITIES = (
    ("layout_fragment/prepaint", "exports source Layout fragments, style facts, scroll facts, and retained revisions"),
    ("layoutng_foundation/paint_model", "owns shared paint identities and Chromium compositing-reason constants"),
    ("paint_model", "owns property-tree, geometry-mapping, cull-rect, clip, and filter value types"),
    ("prepaint", "builds paint properties, updates cull rects, and computes paint invalidation"),
    ("paint_walk", "walks paint layers/fragments in paint order and selects paint phases"),
    ("paint_record", "records display items and drawing operations through PaintController and painters"),
    ("paint_chunk", "groups display items by property state and carries hit-test/capture metadata"),
    ("paint_reuse", "compares retained pre-paint state and admits reusable paint output"),
    ("paint_commit", "assembles the PaintArtifact and publishes scroll paint updates"),
    ("paint", "orchestrates the complete Paint phase across the responsibility packages above"),
)


def code_lines(path: Path) -> set[int]:
    result: set[int] = set()
    in_block_comment = False
    for number, raw in enumerate(path.read_text(errors="strict").splitlines(), 1):
        line = raw
        output: list[str] = []
        index = 0
        while index < len(line):
            if in_block_comment:
                end = line.find("*/", index)
                if end < 0:
                    index = len(line)
                else:
                    in_block_comment = False
                    index = end + 2
                continue
            block = line.find("/*", index)
            single = line.find("//", index)
            if single >= 0 and (block < 0 or single < block):
                output.append(line[index:single])
                break
            if block >= 0:
                output.append(line[index:block])
                in_block_comment = True
                index = block + 2
                continue
            output.append(line[index:])
            break
        if "".join(output).strip():
            result.add(number)
    return result


def expand_ranges(ranges: list[list[int]], maximum: int) -> set[int]:
    expanded: set[int] = set()
    for start, end in ranges:
        if start < 1 or end < start or end > maximum:
            raise ValueError(f"invalid source range {start}-{end}; file has {maximum} lines")
        expanded.update(range(start, end + 1))
    return expanded


def is_in_scope(path: Path, chromium: Path, scope: dict) -> bool:
    relative = path.relative_to(chromium)
    relative_text = relative.as_posix()
    if path.suffix not in scope["extensions"]:
        return False
    if any(
        relative_text == excluded or relative_text.startswith(excluded + "/")
        for excluded in scope.get("excluded_directories", [])
    ):
        return False
    return not any(
        path.stem.endswith(suffix)
        for suffix in scope.get("excluded_filename_stem_suffixes", [])
    )


def discover_scope(chromium: Path, scope: dict) -> list[Path]:
    paths: set[Path] = set()
    for root_text in scope["roots"]:
        root = chromium / root_text
        if not root.is_dir():
            raise SystemExit(f"missing Chromium source root: {root}")
        paths.update(
            path
            for path in root.rglob("*")
            if path.is_file() and is_in_scope(path, chromium, scope)
        )
    return sorted(paths)


def source_area(relative: str) -> str:
    if relative.startswith("third_party/blink/renderer/core/paint/compositing/"):
        return "core/paint/compositing"
    if relative.startswith("third_party/blink/renderer/core/paint/"):
        return "core/paint"
    if relative.startswith("third_party/blink/renderer/platform/graphics/paint/"):
        return "platform/graphics/paint"
    raise ValueError(f"Paint source outside known areas: {relative}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--chromium", type=Path, default=DEFAULT_CHROMIUM)
    parser.add_argument("--write", type=Path)
    parser.add_argument("--write-inventory", type=Path)
    args = parser.parse_args()

    manifest = json.loads(args.manifest.read_text())
    revision = subprocess.check_output(
        ["git", "-C", str(args.chromium), "rev-parse", "HEAD"], text=True
    ).strip()
    if revision != manifest["chromium_revision"]:
        raise SystemExit(
            f"Chromium revision mismatch: expected {manifest['chromium_revision']}, got {revision}"
        )

    rows: list[tuple[str, str, str, int, int, int, int, int]] = []
    selected_totals = defaultdict(int)
    package_totals: dict[str, dict[str, int]] = defaultdict(lambda: defaultdict(int))
    line_states: dict[str, tuple[set[int], set[int], set[int], set[int]]] = {}
    for source in manifest["sources"]:
        path = args.chromium / source["path"]
        if not path.is_file():
            raise SystemExit(f"missing Chromium source: {path}")
        physical_count = len(path.read_text(errors="strict").splitlines())
        code = code_lines(path)
        translated: set[int] = set()
        unsupported: set[int] = set()
        not_applicable: set[int] = set()
        for mapping in source.get("mappings", []):
            selected = expand_ranges(mapping["lines"], physical_count) & code
            if mapping["status"] == "translated":
                translated.update(selected)
            elif mapping["status"] == "unsupported":
                unsupported.update(selected)
            elif mapping["status"] == "not_applicable":
                not_applicable.update(selected)
            else:
                raise ValueError(f"unknown status {mapping['status']!r}")
        overlap = (
            (translated & unsupported)
            | (translated & not_applicable)
            | (unsupported & not_applicable)
        )
        if overlap:
            raise ValueError(f"line-state overlap in {source['path']}: {sorted(overlap)}")
        remaining = code - translated - unsupported - not_applicable
        line_states[source["path"]] = (code, translated, unsupported, not_applicable)
        rows.append(
            (
                source["entity"],
                source["package"],
                source["path"],
                len(code),
                len(translated),
                len(unsupported),
                len(not_applicable),
                len(remaining),
            )
        )
        selected_totals["code"] += len(code)
        selected_totals["translated"] += len(translated)
        selected_totals["unsupported"] += len(unsupported)
        selected_totals["not_applicable"] += len(not_applicable)
        selected_totals["remaining"] += len(remaining)
        package_counts = package_totals[source["package"]]
        package_counts["files"] += 1
        package_counts["code"] += len(code)
        package_counts["translated"] += len(translated)
        package_counts["unsupported"] += len(unsupported)
        package_counts["not_applicable"] += len(not_applicable)
        package_counts["remaining"] += len(remaining)

    scope_rows: dict[str, dict[str, int]] = defaultdict(lambda: defaultdict(int))
    scope_totals = defaultdict(int)
    inventory_rows: list[tuple[str, str, int, int, int, int, int]] = []
    packages_by_path = {source["path"]: source["package"] for source in manifest["sources"]}
    scope_paths = discover_scope(args.chromium, manifest["scope"])
    scope_relative_paths = {path.relative_to(args.chromium).as_posix() for path in scope_paths}
    outside_scope = sorted(set(line_states) - scope_relative_paths)
    for path in scope_paths:
        relative = path.relative_to(args.chromium).as_posix()
        if relative in line_states:
            code, translated, unsupported, not_applicable = line_states[relative]
        else:
            code = code_lines(path)
            translated = set()
            unsupported = set()
            not_applicable = set()
        counts = {
            "files": 1,
            "audited_files": int(relative in line_states),
            "unassigned_files": int(relative not in line_states),
            "code": len(code),
            "translated": len(translated),
            "unsupported": len(unsupported),
            "not_applicable": len(not_applicable),
            "remaining": len(code - translated - unsupported - not_applicable),
        }
        area = source_area(relative)
        inventory_rows.append(
            (
                relative,
                packages_by_path.get(relative, "unassigned"),
                counts["code"],
                counts["translated"],
                counts["unsupported"],
                counts["not_applicable"],
                counts["remaining"],
            )
        )
        for key, count in counts.items():
            scope_rows[area][key] += count
            scope_totals[key] += count

    lines = [
        f"Chromium revision: `{revision}`",
        f"Metric: {manifest['metric']}",
        "",
        "## Full production directory scope",
        "",
        manifest["scope"]["description"] + ".",
        "",
        "| Chromium area | Files | Audited files | Unassigned files | Code | Translated | Unsupported | N/A | Remaining |",
        "|---|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for area in ("core/paint", "core/paint/compositing", "platform/graphics/paint"):
        counts = scope_rows.get(area)
        if not counts:
            continue
        lines.append(
            f"| `{area}` | {counts['files']} | {counts['audited_files']} | "
            f"{counts['unassigned_files']} | {counts['code']} | {counts['translated']} | "
            f"{counts['unsupported']} | {counts['not_applicable']} | {counts['remaining']} |"
        )
    lines.extend(
        [
            f"| **Total** | **{scope_totals['files']}** | **{scope_totals['audited_files']}** | "
            f"**{scope_totals['unassigned_files']}** | **{scope_totals['code']}** | "
            f"**{scope_totals['translated']}** | **{scope_totals['unsupported']}** | "
            f"**{scope_totals['not_applicable']}** | **{scope_totals['remaining']}** |",
            "",
            "Audited files have a manually verified source-range entry. Unassigned files count "
            "entirely as Remaining; a verified package subtotal therefore describes only its "
            "listed backbone files, not every Chromium file that may ultimately belong to that package.",
            "`core/paint/timing` is a separate performance-observation subsystem and is not included "
            "in the rendering-pipeline total above.",
            (
                "Supporting sources outside the two Paint directories remain in the backbone detail "
                f"and outside the full-directory total: {', '.join(f'`{path}`' for path in outside_scope)}."
                if outside_scope
                else ""
            ),
            "",
            "## Rechrom Paint package responsibility map",
            "",
            "These are responsibility boundaries, not claims that every corresponding Chromium source file has been audited.",
            "",
            "| Rechrom package | Responsibility |",
            "|---|---|",
            *(
                f"| `{package}` | {responsibility} |"
                for package, responsibility in PACKAGE_RESPONSIBILITIES
            ),
            "",
            "## Verified backbone by Rechrom package",
            "",
            "This table groups only source files with manually assigned Rechrom ownership. It does "
            "not assign the still-unmapped Chromium files to a package by filename guess.",
            "",
            "| Rechrom package | Files | Code | Translated | Unsupported | N/A | Remaining |",
            "|---|---:|---:|---:|---:|---:|---:|",
        ]
    )
    for package in sorted(package_totals):
        counts = package_totals[package]
        lines.append(
            f"| `{package}` | {counts['files']} | {counts['code']} | "
            f"{counts['translated']} | {counts['unsupported']} | "
            f"{counts['not_applicable']} | {counts['remaining']} |"
        )
    lines.extend(
        [
            "",
            "## Verified backbone detail",
            "",
        "| Entity | Rechrom package | Chromium source | Code | Translated | Unsupported | N/A | Remaining |",
        "|---|---|---|---:|---:|---:|---:|---:|",
        ]
    )
    for entity, package, path, code, translated, unsupported, not_applicable, remaining in rows:
        lines.append(
            f"| {entity} | `{package}` | `{path}` | {code} | {translated} | {unsupported} | {not_applicable} | {remaining} |"
        )
    lines.extend(
        [
            f"| **Selected backbone total** | | | **{selected_totals['code']}** | "
            f"**{selected_totals['translated']}** | **{selected_totals['unsupported']}** | "
            f"**{selected_totals['not_applicable']}** | **{selected_totals['remaining']}** |",
            "",
            "Only manually verified source ranges count as translated. Existing Rust code without a "
            "verified source-range entry remains in Remaining until audited. This table is the detailed "
            "backbone subset; the first table is the authoritative full-directory total.",
        ]
    )
    report = "\n".join(lines) + "\n"
    if args.write:
        output = args.write if args.write.is_absolute() else REPO / args.write
        output.write_text(report)
    if args.write_inventory:
        output = (
            args.write_inventory
            if args.write_inventory.is_absolute()
            else REPO / args.write_inventory
        )
        inventory = [
            f"Chromium revision: `{revision}`",
            f"Metric: {manifest['metric']}",
            "",
            "# Paint source-file translation inventory",
            "",
            "Every production source file in the configured Chromium Paint scope appears here.",
            "`unassigned` means that no source-range audit has assigned the file to a Rechrom package yet;",
            "it does not by itself mean that no equivalent Rust behavior exists.",
            "",
            "| Chromium source | Rechrom package | Code | Translated | Unsupported | N/A | Remaining |",
            "|---|---|---:|---:|---:|---:|---:|",
        ]
        for path, package, code, translated, unsupported, not_applicable, remaining in inventory_rows:
            inventory.append(
                f"| `{path}` | `{package}` | {code} | {translated} | {unsupported} | "
                f"{not_applicable} | {remaining} |"
            )
        inventory.extend(
            [
                f"| **Total** | | **{scope_totals['code']}** | **{scope_totals['translated']}** | "
                f"**{scope_totals['unsupported']}** | **{scope_totals['not_applicable']}** | "
                f"**{scope_totals['remaining']}** |",
                "",
            ]
        )
        output.write_text("\n".join(inventory))
    print(report, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
