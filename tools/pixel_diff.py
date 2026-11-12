#!/usr/bin/env python3
"""Strict, alpha-safe browser screenshot comparison.

Exit 0 means every RGBA channel of every pixel is identical. Exit 1 means the
images differ; exit 2 means they cannot be compared. The mask is constructed
from all four channels explicitly because Pillow's RGBA getbbox behavior can
hide RGB differences when the difference image has a zero alpha channel.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from functools import reduce
from pathlib import Path

from PIL import Image, ImageChops, ImageEnhance, ImageStat


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("reference", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--diff", type=Path)
    parser.add_argument(
        "--max-details",
        type=int,
        default=64,
        help="maximum number of changed pixels and RGBA transitions to report",
    )
    parser.add_argument(
        "--crop",
        type=int,
        nargs=4,
        metavar=("X", "Y", "WIDTH", "HEIGHT"),
        help="compare the same pixel crop in both images",
    )
    args = parser.parse_args()
    if args.max_details < 0:
        parser.error("--max-details must be non-negative")

    reference = Image.open(args.reference).convert("RGBA")
    candidate = Image.open(args.candidate).convert("RGBA")
    if reference.size != candidate.size:
        print(
            json.dumps(
                {
                    "identical": False,
                    "error": "size mismatch",
                    "reference_size": reference.size,
                    "candidate_size": candidate.size,
                }
            )
        )
        return 2
    origin_x = 0
    origin_y = 0
    if args.crop is not None:
        x, y, width, height = args.crop
        if x < 0 or y < 0 or width <= 0 or height <= 0:
            parser.error("crop coordinates must be non-negative and dimensions positive")
        if x + width > reference.width or y + height > reference.height:
            parser.error("crop exceeds image bounds")
        box = (x, y, x + width, y + height)
        reference = reference.crop(box)
        candidate = candidate.crop(box)
        origin_x = x
        origin_y = y

    difference = ImageChops.difference(reference, candidate)
    mask = reduce(ImageChops.lighter, difference.split())
    histogram = mask.histogram()
    total = reference.width * reference.height
    changed = total - histogram[0]
    bbox = mask.getbbox()
    stats = ImageStat.Stat(difference)
    changed_details = []
    transitions: Counter[tuple[tuple[int, ...], tuple[int, ...]]] = Counter()
    transition_scan_limit = max(args.max_details, 65536)
    transition_pixels_scanned = 0
    if bbox is not None and args.max_details:
        reference_pixels = reference.load()
        candidate_pixels = candidate.load()
        mask_pixels = mask.load()
        left, top, right, bottom = bbox
        for y in range(top, bottom):
            for x in range(left, right):
                if not mask_pixels[x, y]:
                    continue
                reference_rgba = tuple(reference_pixels[x, y])
                candidate_rgba = tuple(candidate_pixels[x, y])
                if transition_pixels_scanned < transition_scan_limit:
                    transitions[(reference_rgba, candidate_rgba)] += 1
                    transition_pixels_scanned += 1
                if len(changed_details) < args.max_details:
                    changed_details.append(
                        {
                            "x": x + origin_x,
                            "y": y + origin_y,
                            "reference_rgba": reference_rgba,
                            "candidate_rgba": candidate_rgba,
                            "delta_rgba": tuple(
                                candidate_channel - reference_channel
                                for reference_channel, candidate_channel in zip(
                                    reference_rgba, candidate_rgba
                                )
                            ),
                        }
                    )
                if (
                    len(changed_details) >= args.max_details
                    and transition_pixels_scanned >= transition_scan_limit
                ):
                    break
            if (
                len(changed_details) >= args.max_details
                and transition_pixels_scanned >= transition_scan_limit
            ):
                break
    transition_details = [
        {
            "count": count,
            "reference_rgba": reference_rgba,
            "candidate_rgba": candidate_rgba,
        }
        for (reference_rgba, candidate_rgba), count in transitions.most_common(
            args.max_details
        )
    ]
    result = {
        "identical": changed == 0,
        "size": reference.size,
        "changed_pixels": changed,
        "total_pixels": total,
        "changed_ratio": changed / total if total else 0.0,
        "bbox": bbox,
        "mean_absolute_rgba": stats.mean,
        "extrema_rgba": stats.extrema,
        "changed_pixels_detail": changed_details,
        "rgba_transitions": transition_details,
        "rgba_transitions_complete": transition_pixels_scanned == changed,
        "details_truncated": changed > len(changed_details),
    }
    print(json.dumps(result, separators=(",", ":")))

    if args.diff is not None:
        args.diff.parent.mkdir(parents=True, exist_ok=True)
        rgb = Image.merge("RGB", difference.split()[:3])
        ImageEnhance.Brightness(rgb).enhance(4.0).save(args.diff)
    return 0 if changed == 0 else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as error:
        print(json.dumps({"identical": False, "error": str(error)}))
        raise SystemExit(2)
