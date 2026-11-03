#!/usr/bin/env python3
"""Capture Rechrom/Chrome at fixed scroll positions and run strict pixel diffs."""

from __future__ import annotations

import argparse
import base64
import json
import subprocess
import time
import urllib.request
from pathlib import Path

from PIL import Image, ImageStat


ROOT = Path(__file__).resolve().parents[1]


def api(base: str, method: str, params: dict) -> dict:
    request = urllib.request.Request(
        base + "/api",
        data=json.dumps({"id": 1, "method": method, "params": params}).encode(),
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(request, timeout=60) as response:
        value = json.load(response)
    if "error" in value:
        raise RuntimeError(value["error"])
    return value.get("result", {})


def start_rechrom(binary: Path, width: int, height: int, dpr: float) -> tuple[subprocess.Popen, str]:
    process = subprocess.Popen(
        [
            str(binary), "--headless", "--url", "about:blank",
            "--width", str(width), "--height", str(height),
            "--force-device-scale-factor", str(dpr), "--devtools-port", "0",
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


def trace_timing(snapshot: dict) -> dict:
    spans = snapshot.get("spans", [])
    instants = snapshot.get("instants", [])
    def page_target(event: dict) -> bool:
        return event.get("context", {}).get("target_id") == 1

    starts = [
        event["start_ns"]
        for event in spans
        if event.get("name") == "OpenRequest" and page_target(event)
    ]
    if not starts:
        return {}
    origin = min(starts)

    commits = [
        event for event in instants
        if event.get("name") == "DocumentCommitted"
        and page_target(event)
        and event.get("start_ns", 0) >= origin
    ]
    generation = None
    committed_at = None
    if commits:
        commit = min(commits, key=lambda event: event["start_ns"])
        committed_at = commit["start_ns"]
        generation = commit.get("fields", {}).get("document_generation")

    def first(
        events: list[dict],
        name: str,
        *,
        successful: bool = False,
        same_document: bool = False,
        content: bool = False,
    ):
        values = []
        for event in events:
            if committed_at is None:
                continue
            if event.get("name") != name or event.get("start_ns", 0) < committed_at:
                continue
            if not page_target(event):
                continue
            if successful and event.get("fields", {}).get("succeeded") != 1.0:
                continue
            if same_document and generation is not None and event.get("fields", {}).get("document_generation") != generation:
                continue
            if content and event.get("fields", {}).get("has_content") != 1.0:
                continue
            values.append(event.get("start_ns", 0) + event.get("duration_ns", 0))
        return (min(values) - origin) / 1e6 if values else None

    return {
        "committed_ms": (committed_at - origin) / 1e6 if committed_at is not None else None,
        "first_artifact_ms": first(instants, "DocumentPaintArtifact", same_document=True),
        "first_frame_submit_ms": first(spans, "Compositor.Frame.Submit"),
        "first_offscreen_frame_ms": first(
            instants,
            "OffscreenFramePublished",
            same_document=True,
            content=True,
        ),
        "first_present_ms": first(spans, "PresentReturn", successful=True),
        "load_finished_ms": first(instants, "DocumentLoadFinished"),
        "dropped_events": snapshot.get("dropped_events", 0),
    }


def last_offscreen_frame(snapshot: dict) -> tuple[int, int]:
    frames = [
        event
        for event in snapshot.get("instants", [])
        if event.get("name") == "OffscreenFramePublished"
    ]
    if not frames:
        return (0, 0)
    frame = max(frames, key=lambda event: event.get("start_ns", 0))
    return (
        frame.get("start_ns", 0),
        int(frame.get("fields", {}).get("compositor_scroll_sequence", 0)),
    )


def last_scroll_applied(snapshot: dict) -> int:
    return max(
        (
            event.get("start_ns", 0)
            for event in snapshot.get("instants", [])
            if event.get("name") == "ScrollApplied"
        ),
        default=0,
    )


def last_scroll_position(snapshot: dict, after: int = 0) -> dict | None:
    events = [
        event
        for event in snapshot.get("instants", [])
        if event.get("name") == "ScrollApplied"
        and event.get("start_ns", 0) > after
    ]
    if not events:
        return None
    event = max(events, key=lambda value: value.get("start_ns", 0))
    fields = event.get("fields", {})
    return {
        "event_ns": event.get("start_ns", 0),
        "scroll_node": fields.get("scroll_node"),
        "x": fields.get("offset_x"),
        "y": fields.get("offset_y"),
    }


def wait_for_offscreen_frame(
    base: str,
    recording: str,
    after: tuple[int, int],
    after_scroll_applied: int,
    timeout: float = 5.0,
) -> bool:
    deadline = time.monotonic() + timeout
    latest = after[0]
    applied_at = None
    quiet_since = None
    while time.monotonic() < deadline:
        snapshot = api(base, "Performance.snapshot", {"recording_id": recording})
        current_applied = last_scroll_applied(snapshot)
        if current_applied > after_scroll_applied:
            applied_at = current_applied
        published_at, _ = last_offscreen_frame(snapshot)
        if applied_at is not None and published_at > max(latest, applied_at):
            latest = published_at
            quiet_since = time.monotonic()
        elif (
            applied_at is not None
            and quiet_since is not None
            and time.monotonic() - quiet_since >= 0.05
        ):
            return True
        time.sleep(0.02)
    return False


def screenshot_valid(path: Path) -> dict:
    image = Image.open(path).convert("RGB")
    stat = ImageStat.Stat(image)
    extrema = stat.extrema
    return {
        "size": list(image.size),
        "channel_extrema": extrema,
        "nonblank": any(low != high for low, high in extrema),
    }


def capture_rechrom(sites: list[dict], out: Path, binary: Path, width: int, height: int, dpr: float):
    out.mkdir(parents=True, exist_ok=True)
    process = None
    base = None
    results = []
    try:
        for site in sites:
            if process is None or process.poll() is not None:
                process, base = start_rechrom(binary, width, height, dpr)
            row = {"id": site["id"], "url": site["url"], "screenshots": {}, "error": None}
            try:
                recording = api(base, "Tracing.start", {
                    "max_events": 500_000,
                    "included_categories": [
                        "navigation",
                        "input",
                        "paint",
                        "compositor.frame",
                        "frame",
                        "present",
                    ],
                })["recording_id"]
                api(base, "Page.navigate", {"url": site["url"]})
                navigation_started = time.monotonic()
                ready_since = None
                current_timing = {}
                while time.monotonic() - navigation_started < 15:
                    time.sleep(0.25)
                    current = api(base, "Performance.snapshot", {"recording_id": recording})
                    current_timing = trace_timing(current)
                    if (
                        current_timing.get("committed_ms") is not None
                        and current_timing.get("first_artifact_ms") is not None
                    ):
                        ready_since = ready_since or time.monotonic()
                        if current_timing.get("load_finished_ms") is not None:
                            break
                        if time.monotonic() - ready_since >= 1.5:
                            break
                if current_timing.get("committed_ms") is None:
                    raise RuntimeError("navigation did not commit a new document")
                if current_timing.get("first_artifact_ms") is None:
                    raise RuntimeError("committed document did not produce a paint artifact")
                api(base, "Runtime.evaluate", {"expression": "(() => { const root=document.documentElement; if(!root)return; let s=document.getElementById('__rechrom_compare_style'); if(!s){s=document.createElement('style');s.id='__rechrom_compare_style';root.appendChild(s)} s.textContent='*,*::before,*::after{animation:none!important;transition:none!important;caret-color:transparent!important;scroll-behavior:auto!important}' })()"})
                # Resource completion can continue publishing replacement
                # artifacts after load. Let the initial root-scroller geometry
                # settle before measuring fixed scroll positions; otherwise the
                # first scrollTo may be superseded by that replacement commit.
                time.sleep(1.5)
                row["scroll_positions"] = {}
                for y in site["scrolls"]:
                    before = api(base, "Performance.snapshot", {"recording_id": recording})
                    previous_frame = last_offscreen_frame(before)
                    previous_scroll_applied = last_scroll_applied(before)
                    api(base, "Runtime.evaluate", {"expression": f"window.scrollTo(0,{int(y)});"})
                    if int(y) != 0:
                        wait_for_offscreen_frame(
                            base,
                            recording,
                            previous_frame,
                            previous_scroll_applied,
                        )
                    settled = api(base, "Performance.snapshot", {"recording_id": recording})
                    position = last_scroll_position(settled, previous_scroll_applied)
                    row["scroll_positions"][str(y)] = {
                        "requested_y": int(y),
                        "actual_y": 0 if int(y) == 0 and position is None else (
                            position.get("y") if position else None
                        ),
                        "scroll_node": position.get("scroll_node") if position else None,
                    }
                    data = api(base, "Page.captureScreenshot", {"target_id": 1})["data"]
                    filename = f"{site['id']}__y{y}.png"
                    target = out / filename
                    target.write_bytes(base64.b64decode(data))
                    row["screenshots"][str(y)] = filename
                    validation = screenshot_valid(target)
                    row.setdefault("validation", {})[str(y)] = validation
                    if not validation["nonblank"]:
                        raise RuntimeError(f"blank screenshot at scroll {y}")
                api(base, "Tracing.end", {})
                final = api(base, "Performance.snapshot", {"recording_id": recording})
                row["timing"] = trace_timing(final)
            except Exception as error:
                row["error"] = repr(error)
                if process.poll() is None:
                    try:
                        api(base, "Tracing.end", {})
                    except Exception:
                        pass
            results.append(row)
            (out / "results.json").write_text(json.dumps(results, indent=2))
    finally:
        if process is not None and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
    return results


def run_diff(reference: Path, candidate: Path, diff: Path) -> dict:
    process = subprocess.run(
        ["python3", str(ROOT / "tools/pixel_diff.py"), str(reference), str(candidate), "--diff", str(diff)],
        cwd=ROOT, text=True, capture_output=True,
    )
    if not process.stdout.strip():
        return {"identical": False, "error": process.stderr.strip() or f"exit {process.returncode}"}
    return json.loads(process.stdout)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, default=ROOT / "tools/site_compare_sites.json")
    parser.add_argument("--output", type=Path, default=ROOT / "work/site_compare")
    parser.add_argument("--limit", type=int)
    parser.add_argument("--width", type=int, default=1280)
    parser.add_argument("--height", type=int, default=720)
    parser.add_argument("--dpr", type=float, default=2.0)
    parser.add_argument("--rechrom", type=Path, default=ROOT / "target/Rechrom.app/Contents/MacOS/rechrom")
    parser.add_argument(
        "--chrome-reference",
        type=Path,
        help="reuse a compatible Chrome capture directory instead of recapturing it",
    )
    args = parser.parse_args()
    sites = json.loads(args.manifest.read_text())
    if args.limit:
        sites = sites[: args.limit]
    args.output.mkdir(parents=True, exist_ok=True)
    selected_manifest = args.output / "sites.json"
    selected_manifest.write_text(json.dumps(sites, indent=2))

    chrome_dir = args.chrome_reference or (args.output / "chrome")
    rechrom_dir = args.output / "rechrom"
    if args.chrome_reference is None:
        subprocess.run(
            ["node", str(ROOT / "tools/chrome_site_capture.mjs"), str(selected_manifest), str(chrome_dir), str(args.width), str(args.height), str(args.dpr)],
            cwd=ROOT, check=True,
        )
    elif not (chrome_dir / "results.json").is_file():
        raise FileNotFoundError(f"Chrome reference has no results.json: {chrome_dir}")
    rechrom = capture_rechrom(sites, rechrom_dir, args.rechrom, args.width, args.height, args.dpr)
    chrome = {row["id"]: row for row in json.loads((chrome_dir / "results.json").read_text())}
    comparisons = []
    for row in rechrom:
        reference = chrome.get(row["id"], {})
        for y in next(site for site in sites if site["id"] == row["id"])["scrolls"]:
            chrome_name = reference.get("screenshots", {}).get(str(y))
            rechrom_name = row.get("screenshots", {}).get(str(y))
            if not chrome_name or not rechrom_name:
                comparisons.append({"id": row["id"], "scroll_y": y, "error": "missing screenshot"})
                continue
            result = run_diff(chrome_dir / chrome_name, rechrom_dir / rechrom_name, args.output / "diff" / f"{row['id']}__y{y}.png")
            comparisons.append({"id": row["id"], "scroll_y": y, **result})
    ratios = [row["changed_ratio"] for row in comparisons if "changed_ratio" in row]
    first_screen = []
    for row in rechrom:
        reference = chrome.get(row["id"], {})
        chrome_paints = reference.get("timing", {}).get("paints", {})
        chrome_fcp = chrome_paints.get("first-contentful-paint") or chrome_paints.get("first-paint")
        rechrom_timing = row.get("timing", {})
        rechrom_first_frame = (
            rechrom_timing.get("first_offscreen_frame_ms")
            or rechrom_timing.get("first_present_ms")
            or rechrom_timing.get("first_frame_submit_ms")
        )
        if chrome_fcp is None or rechrom_first_frame is None:
            continue
        first_screen.append({
            "id": row["id"],
            "chrome_fcp_ms": chrome_fcp,
            "rechrom_first_frame_ms": rechrom_first_frame,
            "delta_ms": rechrom_first_frame - chrome_fcp,
            "ratio": rechrom_first_frame / chrome_fcp if chrome_fcp else None,
        })
    summary = {
        "sites": len(sites),
        "positions": len(comparisons),
        "compared": len(ratios),
        "zero_pixel_positions": sum(row.get("changed_pixels") == 0 for row in comparisons),
        "mean_changed_ratio": sum(ratios) / len(ratios) if ratios else None,
        "max_changed_ratio": max(ratios) if ratios else None,
        "first_screen": first_screen,
        "comparisons": comparisons,
        "chrome": list(chrome.values()),
        "rechrom": rechrom,
    }
    (args.output / "summary.json").write_text(json.dumps(summary, indent=2))
    print(json.dumps({key: value for key, value in summary.items() if key not in {"comparisons", "chrome", "rechrom"}}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
