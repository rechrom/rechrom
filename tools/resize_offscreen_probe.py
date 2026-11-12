#!/usr/bin/env python3
"""Measure continuous viewport resize through Rechrom's offscreen protocol path."""

from __future__ import annotations

import argparse
import base64
import hashlib
import io
import json
import subprocess
import threading
import time
import urllib.request
from pathlib import Path

from PIL import Image, ImageStat


ROOT = Path(__file__).resolve().parents[1]


def api(base: str, method: str, params: dict | None = None) -> dict:
    request = urllib.request.Request(
        base + "/api",
        data=json.dumps({"id": 1, "method": method, "params": params or {}}).encode(),
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(request, timeout=60) as response:
        value = json.load(response)
    if "error" in value:
        raise RuntimeError(value["error"])
    return value.get("result", {})


def start_rechrom(binary: Path, width: int, height: int, dpr: float):
    process = subprocess.Popen(
        [
            str(binary),
            "--headless",
            "--url",
            "about:blank",
            "--width",
            str(width),
            "--height",
            str(height),
            "--force-device-scale-factor",
            str(dpr),
            "--devtools-port",
            "0",
        ],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
    )
    deadline = time.monotonic() + 20
    while time.monotonic() < deadline:
        line = process.stdout.readline()
        if line.startswith("devtools-url\t"):
            return process, line.strip().split("\t", 1)[1].rstrip("/")
        if process.poll() is not None:
            break
    stderr = process.stderr.read() if process.stderr else ""
    raise RuntimeError(f"Rechrom failed to expose DevTools: {stderr[-2000:]}")


def decode_shot(result: dict) -> tuple[Image.Image, bytes]:
    png = base64.b64decode(result["data"])
    return Image.open(io.BytesIO(png)).convert("RGB"), png


def image_metrics(image: Image.Image, png: bytes) -> dict:
    stat = ImageStat.Stat(image)
    extrema = stat.extrema
    return {
        "width": image.width,
        "height": image.height,
        "uniform": all(low == high for low, high in extrema),
        "stddev_mean": round(sum(stat.stddev) / len(stat.stddev), 4),
        "sha256": hashlib.sha256(png).hexdigest(),
    }


def capture(base: str, target_id: int, output: Path | None = None) -> dict:
    started = time.monotonic_ns()
    image, png = decode_shot(api(base, "Page.captureScreenshot", {"target_id": target_id}))
    finished = time.monotonic_ns()
    if output is not None:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_bytes(png)
    return {
        "start_ns": started,
        "finish_ns": finished,
        "capture_ms": (finished - started) / 1e6,
        **image_metrics(image, png),
    }


def wait_for_size(base: str, expected: tuple[int, int], timeout: float = 10) -> dict:
    deadline = time.monotonic() + timeout
    last = None
    while time.monotonic() < deadline:
        last = capture(base, 1)
        if (last["width"], last["height"]) == expected:
            return last
        time.sleep(0.002)
    raise RuntimeError(f"page target did not reach {expected}; last={last}")


def percentile(values: list[float], fraction: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    index = min(len(ordered) - 1, int((len(ordered) - 1) * fraction + 0.5))
    return ordered[index]


def scalar_sweep(high: int, low: int, step: int) -> list[int]:
    down = list(range(high, low - 1, -step))
    if down[-1] != low:
        down.append(low)
    up = list(range(low + step, high + 1, step))
    if not up or up[-1] != high:
        up.append(high)
    return down + up


def dimensions(args: argparse.Namespace) -> list[tuple[int, int]]:
    if args.axis == "vertical":
        return [
            (args.width, value)
            for value in scalar_sweep(args.height, args.low_height, args.height_step)
        ]
    # Keep content height fixed: this isolates horizontal responsive reflow and
    # makes every screenshot dimension directly attributable to one request.
    return [
        (value, args.height)
        for value in scalar_sweep(args.width, args.low_width, args.step)
    ]


def run(args: argparse.Namespace) -> dict:
    process, base = start_rechrom(args.binary, args.width, args.height, args.dpr)
    samples: list[dict] = []
    requests: list[dict] = []
    stop_sampling = threading.Event()
    sampling_error: list[str] = []
    trace_id = None
    try:
        api(base, "Page.navigate", {"url": args.url})
        time.sleep(args.settle_seconds)
        initial = wait_for_size(base, (round(args.width * args.dpr), round(args.height * args.dpr)))
        initial_toolbar = capture(
            base,
            2,
            args.screenshot_dir / "toolbar-initial.png" if args.screenshot_dir else None,
        )
        if args.screenshot_dir:
            capture(base, 1, args.screenshot_dir / "initial.png")
        trace_id = api(
            base,
            "Tracing.start",
            {
                "max_events": 500_000,
                "included_categories": [
                    "resize",
                    "frame",
                    "paint",
                    "layout",
                    "compositor.frame",
                    "present",
                    "devtools",
                ],
            },
        )["recording_id"]

        def sampler():
            while not stop_sampling.is_set():
                try:
                    samples.append(capture(base, 1))
                except Exception as error:  # Report, but preserve the trace.
                    sampling_error.append(repr(error))
                    return

        thread = None
        if not args.no_concurrent_capture:
            thread = threading.Thread(target=sampler, name="offscreen-frame-sampler", daemon=True)
            thread.start()
        sequence = dimensions(args)
        for width, height in sequence:
            sent_ns = time.monotonic_ns()
            api(
                base,
                "Emulation.setDeviceMetricsOverride",
                {"width": width, "height": height, "deviceScaleFactor": args.dpr},
            )
            requests.append({"sent_ns": sent_ns, "width": width, "height": height})
            time.sleep(args.interval_ms / 1000)
        if not args.no_concurrent_capture:
            final_expected = (
                round(sequence[-1][0] * args.dpr),
                round(sequence[-1][1] * args.dpr),
            )
            wait_for_size(base, final_expected)
        time.sleep(args.post_seconds)
        stop_sampling.set()
        if thread is not None:
            thread.join(timeout=20)

        api(base, "Tracing.end")
        snapshot = api(base, "Performance.snapshot", {"recording_id": trace_id})
        summary = api(base, "Performance.summary", {"recording_id": trace_id})
        trace_id = None
        if args.trace_output:
            args.trace_output.parent.mkdir(parents=True, exist_ok=True)
            args.trace_output.write_text(json.dumps(snapshot))
        page = capture(base, 1, args.screenshot_dir / "page.png" if args.screenshot_dir else None)
        toolbar = capture(base, 2, args.screenshot_dir / "toolbar.png" if args.screenshot_dir else None)

        expected_sizes = {
            (round(request["width"] * args.dpr), round(request["height"] * args.dpr))
            for request in requests
        }
        impossible = [
            sample for sample in samples
            if (sample["width"], sample["height"]) not in expected_sizes
        ]
        blank = [sample for sample in samples if sample["uniform"]]
        stale = []
        for sample in samples:
            latest = None
            for request in requests:
                if request["sent_ns"] <= sample["start_ns"]:
                    latest = request
                else:
                    break
            if latest is not None and (
                sample["width"] != round(latest["width"] * args.dpr)
                or sample["height"] != round(latest["height"] * args.dpr)
            ):
                stale.append(sample)

        resize_spans = [
            span for span in snapshot.get("spans", [])
            if span.get("name") == "ResizeToPresentationComplete"
        ]
        resize_ms = [span.get("duration_ns", 0) / 1e6 for span in resize_spans]
        offscreen = [
            event for event in snapshot.get("instants", [])
            if event.get("name") == "OffscreenFramePublished"
            and event.get("context", {}).get("target_id") == 1
        ]
        offscreen_intervals = [
            (right["start_ns"] - left["start_ns"]) / 1e6
            for left, right in zip(offscreen, offscreen[1:])
        ]
        # Content provenance can lag behind an already resized root. Measure
        # the first root publication separately using the existing frame parent
        # IDs and Host queue timestamps, all in the trace's monotonic clock.
        host_resizes = sorted(
            (span for span in snapshot.get("spans", [])
             if span.get("name") == "Host.Resize" and span.get("fields", {}).get("accepted")),
            key=lambda span: span["start_ns"],
        )
        host_resize_tasks = {
            span.get("context", {}).get("task_id"): span
            for span in host_resizes
        }
        ingress = {
            span.get("context", {}).get("task_id"): span["start_ns"]
            for span in snapshot.get("spans", []) if span.get("name") == "ResizeHostQueue"
        }
        # DisplayPageFrameReceived retains the resize task provenance from the
        # Page that produced the Surface. Matching it to Host.Resize measures
        # the exact-size Surface arrival independently of root presentation.
        # tab_id=0 is browser chrome (toolbar); tab_id=1 is WebContents.
        surface_arrival_ms: dict[str, list[float]] = {
            "toolbar": [],
            "content": [],
        }
        surface_arrival_tasks: dict[str, set[int]] = {
            "toolbar": set(),
            "content": set(),
        }
        for event in snapshot.get("instants", []):
            if event.get("name") != "DisplayPageFrameReceived":
                continue
            task_id = event.get("context", {}).get("task_id")
            if task_id not in host_resize_tasks:
                continue
            tab_id = int(event.get("fields", {}).get("tab_id", -1))
            page_kind = "toolbar" if tab_id == 0 else "content" if tab_id == 1 else None
            if page_kind is None:
                continue
            queued = ingress.get(task_id, host_resize_tasks[task_id]["start_ns"])
            surface_arrival_ms[page_kind].append((event["start_ns"] - queued) / 1e6)
            surface_arrival_tasks[page_kind].add(task_id)
        prepared = {
            span["parent_id"]: span for span in snapshot.get("spans", [])
            if span.get("name") == "Display.PrepareFrame"
        }
        published_resizes = set()
        root_resize_ms = []
        for event in offscreen:
            prepare = prepared.get(event.get("parent_id"))
            if prepare is None:
                continue
            latest = next((span for span in reversed(host_resizes)
                           if span["start_ns"] <= prepare["start_ns"]), None)
            if latest is None or latest["id"] in published_resizes:
                continue
            if any(latest["fields"].get(key) != prepare["fields"].get(key)
                   for key in ("width", "height", "device_scale")):
                continue
            published_resizes.add(latest["id"])
            queued = ingress.get(latest.get("context", {}).get("task_id"), latest["start_ns"])
            root_resize_ms.append((event["start_ns"] - queued) / 1e6)
        draw_age_ms = [
            span["fields"]["draw_start_age_ms"] for span in snapshot.get("spans", [])
            if span.get("name") == "FrameCycle" and span.get("fields", {}).get("submitted")
            and host_resizes and host_resizes[0]["start_ns"] <= span["start_ns"]
            <= host_resizes[-1]["start_ns"]
        ]
        resize_window_offscreen = []
        resize_window_intervals = []
        stale_surface_histogram: dict[str, int] = {}
        if resize_spans:
            resize_window_start = min(span["start_ns"] for span in resize_spans)
            resize_window_end = max(
                span["start_ns"] + span.get("duration_ns", 0)
                for span in resize_spans
            )
            resize_window_offscreen = [
                event for event in offscreen
                if resize_window_start <= event["start_ns"] <= resize_window_end
            ]
            resize_window_intervals = [
                (right["start_ns"] - left["start_ns"]) / 1e6
                for left, right in zip(resize_window_offscreen, resize_window_offscreen[1:])
            ]
            for span in snapshot.get("spans", []):
                if (
                    span.get("name") == "Display.PrepareFrame"
                    and resize_window_start <= span["start_ns"] <= resize_window_end
                ):
                    stale_count = str(int(span.get("fields", {}).get("stale_surfaces", -1)))
                    stale_surface_histogram[stale_count] = stale_surface_histogram.get(stale_count, 0) + 1
        content_revisions = [
            event.get("fields", {}).get("content_recording_revision")
            for event in resize_window_offscreen
        ]
        same_content_revision_frames = sum(
            left == right for left, right in zip(content_revisions, content_revisions[1:])
        )
        capture_ms = [sample["capture_ms"] for sample in samples]
        return {
            "configuration": {
                "url": args.url,
                "dpr": args.dpr,
                "sequence": [[w, h] for w, h in sequence],
                "interval_ms": args.interval_ms,
            },
            "requests": len(requests),
            "captured_screenshots": len(samples),
            "unique_captured_sizes": sorted({(s["width"], s["height"]) for s in samples}),
            "blank_screenshots": len(blank),
            "impossible_dimension_screenshots": len(impossible),
            "stale_at_capture_start": len(stale),
            "sampling_errors": sampling_error,
            "capture_cost_ms": {
                "p50": percentile(capture_ms, 0.50),
                "p95": percentile(capture_ms, 0.95),
                "max": max(capture_ms, default=0),
            },
            "trace_resize": summary.get("resize", {}),
            "trace_stages": summary.get("stages", {}),
            "resize_spans": len(resize_spans),
            "root_geometry_latency_ms": {
                "count": len(root_resize_ms),
                "p50": percentile(root_resize_ms, 0.50),
                "p95": percentile(root_resize_ms, 0.95),
                "max": max(root_resize_ms, default=0),
            },
            "exact_surface_arrival_latency_ms": {
                page: {
                    "count": len(values),
                    "request_coverage": len(surface_arrival_tasks[page]),
                    "p50": percentile(values, 0.50),
                    "p95": percentile(values, 0.95),
                    "max": max(values, default=0),
                }
                for page, values in surface_arrival_ms.items()
            },
            "display_draw_start_age_ms": {
                "p50": percentile(draw_age_ms, 0.50),
                "p95": percentile(draw_age_ms, 0.95),
                "max": max(draw_age_ms, default=0),
            },
            "resize_latency_ms": {
                "p50": percentile(resize_ms, 0.50),
                "p95": percentile(resize_ms, 0.95),
                "max": max(resize_ms, default=0),
                "values": resize_ms,
            },
            "offscreen_frames": len(offscreen),
            "offscreen_frame_intervals_ms": {
                "p50": percentile(offscreen_intervals, 0.50),
                "p95": percentile(offscreen_intervals, 0.95),
                "max": max(offscreen_intervals, default=0),
                "over_16_7_ms": sum(value > 16.7 for value in offscreen_intervals),
                "values": offscreen_intervals,
            },
            "resize_window_offscreen": {
                "frames": len(resize_window_offscreen),
                "content_revision_advances": max(
                    0, len(resize_window_offscreen) - same_content_revision_frames
                ),
                "same_content_revision_frames": same_content_revision_frames,
                "unattributed_frames": sum(
                    event.get("context", {}).get("task_id", 0) == 0
                    for event in resize_window_offscreen
                ),
                "stale_surface_histogram": stale_surface_histogram,
                "intervals_ms": {
                    "p50": percentile(resize_window_intervals, 0.50),
                    "p95": percentile(resize_window_intervals, 0.95),
                    "max": max(resize_window_intervals, default=0),
                    "over_16_7_ms": sum(value > 16.7 for value in resize_window_intervals),
                    "values": resize_window_intervals,
                },
            },
            "final_targets": {"page": page, "toolbar": toolbar},
            "toolbar_roundtrip_same_pixels": (
                initial_toolbar["sha256"] == toolbar["sha256"]
            ),
            "toolbar_pair_valid": (
                page["width"] == toolbar["width"]
                and toolbar["height"] == round(86 * args.dpr)
                and not page["uniform"]
                and not toolbar["uniform"]
            ),
        }
    finally:
        stop_sampling.set()
        if trace_id is not None:
            try:
                api(base, "Tracing.end")
            except Exception:
                pass
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/rechrom")
    parser.add_argument("--url", default="http://127.0.0.1:9877/python-docs/index.html")
    parser.add_argument("--output", type=Path, default=ROOT / "work/resize-offscreen-probe.json")
    parser.add_argument("--trace-output", type=Path)
    parser.add_argument("--screenshot-dir", type=Path)
    parser.add_argument("--width", type=int, default=1280)
    parser.add_argument("--low-width", type=int, default=600)
    parser.add_argument("--height", type=int, default=720)
    parser.add_argument("--step", type=int, default=20)
    parser.add_argument("--axis", choices=("horizontal", "vertical"), default="horizontal")
    parser.add_argument("--low-height", type=int, default=360)
    parser.add_argument("--height-step", type=int, default=20)
    parser.add_argument("--dpr", type=float, default=1.0)
    parser.add_argument("--interval-ms", type=float, default=8.0)
    parser.add_argument("--settle-seconds", type=float, default=2.0)
    parser.add_argument("--post-seconds", type=float, default=0.5)
    parser.add_argument("--no-concurrent-capture", action="store_true")
    args = parser.parse_args()
    result = run(args)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2))
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
