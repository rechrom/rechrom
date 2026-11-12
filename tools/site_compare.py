#!/usr/bin/env python3
"""Capture Rechrom/Chrome at fixed scroll positions and run strict pixel diffs."""

from __future__ import annotations

import argparse
import base64
import io
import json
import os
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
    inherit_stderr = os.environ.get("SITE_COMPARE_INHERIT_RECHROM_STDERR") == "1"
    process = subprocess.Popen(
        [
            str(binary), "--headless", "--url", "about:blank",
            "--width", str(width), "--height", str(height),
            "--force-device-scale-factor", str(dpr), "--devtools-port", "0",
            "--force-prefers-reduced-motion",
        ],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=None if inherit_stderr else subprocess.PIPE,
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

    commits = [
        event for event in instants
        if event.get("name") == "DocumentCommitted"
        and page_target(event)
    ]
    if not commits:
        return {}

    # Tracing can begin while the startup about:blank request is still open.
    # Select the final top-level commit produced by this Page.navigate, then
    # pair it with the nearest preceding OpenRequest.  Using the first request
    # incorrectly charges startup/about:blank work to the measured document.
    commit = max(commits, key=lambda event: event["start_ns"])
    committed_at = commit["start_ns"]
    starts = [
        event["start_ns"]
        for event in spans
        if event.get("name") == "OpenRequest"
        and page_target(event)
        and event.get("start_ns", 0) <= committed_at
    ]
    if not starts:
        return {}
    origin = max(starts)

    generation = None
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


def wait_for_new_offscreen_frame(
    base: str,
    recording: str,
    after: tuple[int, int],
    timeout: float = 15.0,
) -> bool:
    """Wait until a queued DOM/style mutation reaches the offscreen surface."""
    deadline = time.monotonic() + timeout
    latest = after[0]
    quiet_since = None
    while time.monotonic() < deadline:
        snapshot = api(base, "Performance.snapshot", {"recording_id": recording})
        published_at, _ = last_offscreen_frame(snapshot)
        if published_at > latest:
            latest = published_at
            quiet_since = time.monotonic()
        elif quiet_since is not None and time.monotonic() - quiet_since >= 0.05:
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


def screenshot_bytes_valid(data: str) -> dict:
    image = Image.open(io.BytesIO(base64.b64decode(data))).convert("RGB")
    stat = ImageStat.Stat(image)
    extrema = stat.extrema
    return {
        "size": list(image.size),
        "channel_extrema": extrema,
        "nonblank": any(low != high for low, high in extrema),
    }


def runtime_string(base: str, expression: str) -> str | None:
    """Read a primitive from Rechrom's fire-and-forget Runtime.evaluate bridge."""
    marker = "__RECHROM_VALUE__"
    try:
        api(
            base,
            "Runtime.evaluate",
            {
                "expression": (
                    "throw new Error(" + json.dumps(marker) + "+String(" + expression + "))"
                )
            },
        )
    except RuntimeError as error:
        message = str(error)
        if marker in message:
            return message.split(marker, 1)[1]
    return None


def wait_for_window_scroll(base: str, expected_y: int, timeout: float = 5.0) -> float | None:
    """Wait for the root programmatic-scroll promise, ignoring nested scroll noise."""
    deadline = time.monotonic() + timeout
    actual = None
    while time.monotonic() < deadline:
        value = runtime_string(base, "window.scrollY")
        try:
            actual = float(value) if value is not None else None
        except ValueError:
            actual = None
        if actual is not None and abs(actual - expected_y) < 0.5:
            return actual
        time.sleep(0.02)
    return actual


def wait_for_document_fonts(base: str, timeout: float = 15.0) -> dict:
    deadline = time.monotonic() + timeout
    status = None
    while time.monotonic() < deadline:
        status = runtime_string(
            base,
            "document.fonts ? document.fonts.status : 'unsupported'",
        )
        if status in {"loaded", "unsupported"}:
            return {"font_status": status, "timed_out": False}
        time.sleep(0.1)
    return {"font_status": status, "timed_out": True}


def wait_for_visible_images(base: str, timeout: float | None = None) -> dict:
    if timeout is None:
        timeout = float(os.environ.get("RECHROM_VISIBLE_IMAGE_TIMEOUT", "15"))
    expression = """(() => {
      const visible = image => {
        const r=image.getBoundingClientRect();
        return r.width > 0 && r.height > 0 && r.bottom > 0 && r.right > 0 && r.top < innerHeight && r.left < innerWidth;
      };
      const images=Array.from(document.images).filter(visible);
      return JSON.stringify({
        visible:images.length,
        complete:images.filter(image=>image.complete).length,
        decoded:images.filter(image=>image.complete&&image.naturalWidth>0).length,
        pending:images.filter(image=>!image.complete).length,
        images:images.map(image=>{const r=image.getBoundingClientRect();return {
          src:image.currentSrc||image.src,
          complete:image.complete,
          naturalWidth:image.naturalWidth,
          naturalHeight:image.naturalHeight,
          rect:[r.x,r.y,r.width,r.height],
        }}),
      });
    })()"""
    deadline = time.monotonic() + timeout
    state = None
    while time.monotonic() < deadline:
        raw = runtime_string(base, expression)
        if raw:
            try:
                state = json.loads(raw)
            except json.JSONDecodeError:
                state = None
        if state is not None and state.get("pending") == 0:
            state["timed_out"] = False
            return state
        time.sleep(0.1)
    state = state or {}
    state["timed_out"] = True
    return state


def capture_stable_screenshot(
    base: str,
    *,
    target_id: int = 1,
    timeout: float = 5.0,
) -> tuple[str, bool, int]:
    deadline = time.monotonic() + timeout
    previous = None
    repeats = 0
    latest = None
    attempts = 0
    while time.monotonic() < deadline:
        latest = api(base, "Page.captureScreenshot", {"target_id": target_id})["data"]
        attempts += 1
        if latest == previous:
            repeats += 1
            if repeats >= 2:
                return latest, True, attempts
        else:
            previous = latest
            repeats = 0
        time.sleep(0.25)
    if latest is None:
        latest = api(base, "Page.captureScreenshot", {"target_id": target_id})["data"]
        attempts += 1
    return latest, False, attempts


def capture_rechrom(
    sites: list[dict],
    out: Path,
    binary: Path,
    width: int,
    height: int,
    dpr: float,
    minimum_navigation_ms: float = 0,
    stabilize_style: bool = True,
    after_settle_script: str = "",
):
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
                first_nonblank_ms = None
                current_timing = {}
                navigation_limit = max(45.0, minimum_navigation_ms / 1000.0 + 5.0)
                while time.monotonic() - navigation_started < navigation_limit:
                    time.sleep(0.5)
                    current = api(base, "Performance.snapshot", {"recording_id": recording})
                    current_timing = trace_timing(current)
                    if (
                        current_timing.get("committed_ms") is not None
                        and current_timing.get("first_artifact_ms") is not None
                    ):
                        validation = screenshot_bytes_valid(
                            api(base, "Page.captureScreenshot", {"target_id": 1})["data"]
                        )
                        if validation["nonblank"]:
                            first_nonblank_ms = first_nonblank_ms or (
                                (time.monotonic() - navigation_started) * 1000
                            )
                            ready_since = ready_since or time.monotonic()
                            # Load can remain open for intentionally long-lived
                            # resources. A stable nonblank surface is the visual
                            # readiness boundary; never accept a blank first
                            # artifact as a completed capture.
                            minimum_elapsed = (
                                (time.monotonic() - navigation_started) * 1000
                                >= minimum_navigation_ms
                            )
                            if (
                                minimum_elapsed
                                and current_timing.get("load_finished_ms") is not None
                            ):
                                break
                            if minimum_elapsed and time.monotonic() - ready_since >= 1.5:
                                break
                        else:
                            ready_since = None
                if current_timing.get("committed_ms") is None:
                    raise RuntimeError("navigation did not commit a new document")
                if current_timing.get("first_artifact_ms") is None:
                    raise RuntimeError("committed document did not produce a paint artifact")
                if first_nonblank_ms is None:
                    raise RuntimeError("committed document did not publish a nonblank frame")
                current_timing["first_nonblank_ms"] = first_nonblank_ms
                row["render_readiness"] = wait_for_document_fonts(base)
                if stabilize_style:
                    before_settle_style = api(
                        base,
                        "Performance.snapshot",
                        {"recording_id": recording},
                    )
                    previous_frame = last_offscreen_frame(before_settle_style)
                    api(base, "Runtime.evaluate", {"expression": "(() => { const root=document.documentElement; if(!root)return; let s=document.getElementById('__rechrom_compare_style'); if(!s){s=document.createElement('style');s.id='__rechrom_compare_style';root.appendChild(s)} s.textContent='*,*::before,*::after{animation:none!important;transition:none!important;caret-color:transparent!important;scroll-behavior:auto!important}' })()"})
                    wait_for_new_offscreen_frame(base, recording, previous_frame)
                if after_settle_script:
                    before_after_settle = api(
                        base,
                        "Performance.snapshot",
                        {"recording_id": recording},
                    )
                    previous_frame = last_offscreen_frame(before_after_settle)
                    api(
                        base,
                        "Runtime.evaluate",
                        {"expression": after_settle_script},
                    )
                    wait_for_new_offscreen_frame(base, recording, previous_frame)
                    probe = runtime_string(
                        base,
                        "document.documentElement.dataset.rechromCompareProbe",
                    )
                    if probe is not None:
                        row["after_settle_probe"] = probe
                row["render_readiness"]["after_style"] = wait_for_document_fonts(base)
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
                    actual_root_y = wait_for_window_scroll(base, int(y))
                    if int(y) != 0:
                        wait_for_offscreen_frame(
                            base,
                            recording,
                            previous_frame,
                            previous_scroll_applied,
                        )
                    # Keep the Rechrom capture phase identical to
                    # chrome_site_capture.mjs:219-220. Chromium's macOS overlay
                    # scrollbar opacity is time-dependent after ScrollOffsetChanged,
                    # so comparing an immediate Rechrom frame with Chrome at 200ms
                    # measures two different animation instants.
                    time.sleep(0.2)
                    settled = api(base, "Performance.snapshot", {"recording_id": recording})
                    position = last_scroll_position(settled, previous_scroll_applied)
                    row["scroll_positions"][str(y)] = {
                        "requested_y": int(y),
                        "actual_y": actual_root_y,
                        "scroll_node": position.get("scroll_node") if position else None,
                        "scroll_height": runtime_string(
                            base, "document.documentElement.scrollHeight"
                        ),
                        "client_height": runtime_string(
                            base, "document.documentElement.clientHeight"
                        ),
                    }
                    row["scroll_positions"][str(y)]["visible_images"] = (
                        wait_for_visible_images(base)
                    )
                    # Scrolling can make a previously unused @font-face become
                    # selected during layout. Wait on the browser's real
                    # FontFaceSet state at every capture position, not only at
                    # initial navigation readiness.
                    row["scroll_positions"][str(y)]["font_readiness"] = (
                        wait_for_document_fonts(base)
                    )
                    post_scroll_settle_ms = float(
                        os.environ.get("RECHROM_COMPARE_POST_SCROLL_SETTLE_MS", "0")
                    )
                    if post_scroll_settle_ms > 0:
                        time.sleep(post_scroll_settle_ms / 1000)
                    if os.environ.get(
                        "SITE_COMPARE_FORCE_FULL_REPAINT_AFTER_SCROLL"
                    ) == "1":
                        before_hidden = api(
                            base,
                            "Performance.snapshot",
                            {"recording_id": recording},
                        )
                        previous_frame = last_offscreen_frame(before_hidden)
                        api(
                            base,
                            "Runtime.evaluate",
                            {
                                "expression": "(() => { const root=document.documentElement; root.dataset.rechromPreviousVisibility=root.style.visibility; root.style.visibility='hidden'; })()"
                            },
                        )
                        wait_for_new_offscreen_frame(base, recording, previous_frame)
                        before_visible = api(
                            base,
                            "Performance.snapshot",
                            {"recording_id": recording},
                        )
                        previous_frame = last_offscreen_frame(before_visible)
                        api(
                            base,
                            "Runtime.evaluate",
                            {
                                "expression": "(() => { const root=document.documentElement; root.style.visibility=root.dataset.rechromPreviousVisibility || ''; delete root.dataset.rechromPreviousVisibility; })()"
                            },
                        )
                        wait_for_new_offscreen_frame(base, recording, previous_frame)
                        repaint_settle_ms = float(
                            os.environ.get(
                                "SITE_COMPARE_FORCE_REPAINT_SETTLE_MS", "1000"
                            )
                        )
                        if repaint_settle_ms > 0:
                            time.sleep(repaint_settle_ms / 1000)
                    data, stable_frame, stability_attempts = capture_stable_screenshot(base)
                    row["scroll_positions"][str(y)]["stable_frame"] = stable_frame
                    row["scroll_positions"][str(y)]["stability_attempts"] = stability_attempts
                    probe = runtime_string(
                        base,
                        "document.documentElement.dataset.rechromCompareProbe",
                    )
                    if probe is not None:
                        row["scroll_positions"][str(y)]["probe"] = probe
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
                row["timing"]["first_nonblank_ms"] = first_nonblank_ms
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
    parser.add_argument(
        "--rechrom-minimum-navigation-ms",
        type=float,
        default=0,
        help="do not capture Rechrom before this navigation age",
    )
    parser.add_argument(
        "--skip-stabilize-style",
        action="store_true",
        help="capture without injecting the animation/transition stabilization style",
    )
    parser.add_argument(
        "--rechrom-after-settle-script",
        default="",
        help="evaluate JavaScript after visual readiness and before capture",
    )
    parser.add_argument(
        "--after-settle-script-file",
        type=Path,
        help="evaluate the same JavaScript file in both browsers before capture",
    )
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
    after_settle_script = (
        args.after_settle_script_file.read_text()
        if args.after_settle_script_file
        else args.rechrom_after_settle_script
    )

    chrome_dir = args.chrome_reference or (args.output / "chrome")
    rechrom_dir = args.output / "rechrom"
    if args.chrome_reference is None:
        chrome_environment = os.environ.copy()
        if args.skip_stabilize_style:
            chrome_environment["CHROME_PRESERVE_MOTION"] = "1"
        if after_settle_script:
            chrome_environment["CHROME_AFTER_SETTLE_SCRIPT"] = after_settle_script
        subprocess.run(
            ["node", str(ROOT / "tools/chrome_site_capture.mjs"), str(selected_manifest), str(chrome_dir), str(args.width), str(args.height), str(args.dpr)],
            cwd=ROOT, check=True, env=chrome_environment,
        )
    elif not (chrome_dir / "results.json").is_file():
        raise FileNotFoundError(f"Chrome reference has no results.json: {chrome_dir}")
    rechrom = capture_rechrom(
        sites,
        rechrom_dir,
        args.rechrom,
        args.width,
        args.height,
        args.dpr,
        args.rechrom_minimum_navigation_ms,
        not args.skip_stabilize_style,
        after_settle_script,
    )
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
