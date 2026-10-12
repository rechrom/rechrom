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
from functools import reduce
from pathlib import Path

from PIL import Image, ImageChops, ImageEnhance, ImageStat


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("reference", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--diff", type=Path)
    args = parser.parse_args()

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

    difference = ImageChops.difference(reference, candidate)
    mask = reduce(ImageChops.lighter, difference.split())
    histogram = mask.histogram()
    total = reference.width * reference.height
    changed = total - histogram[0]
    bbox = mask.getbbox()
    stats = ImageStat.Stat(difference)
    result = {
        "identical": changed == 0,
        "size": reference.size,
        "changed_pixels": changed,
        "total_pixels": total,
        "changed_ratio": changed / total if total else 0.0,
        "bbox": bbox,
        "mean_absolute_rgba": stats.mean,
        "extrema_rgba": stats.extrema,
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
