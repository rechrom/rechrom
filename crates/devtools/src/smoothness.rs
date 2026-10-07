//! Observation-based flow analysis. No engine objects and no inferred scanout.
use browser_tracing::{RecordedSpan, Snapshot};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

type Key = (u64, u64, u64); // target, source, sequence
fn key(e: &RecordedSpan) -> Key {
    (e.context.target_id, e.context.source_id, e.context.frame_id)
}
fn end(e: &RecordedSpan) -> u64 {
    e.start_ns.saturating_add(e.duration_ns)
}
fn number(e: &RecordedSpan, name: &str) -> Option<f64> {
    e.fields.get(name).copied().filter(|n| n.is_finite())
}
fn interval(e: &RecordedSpan) -> Option<u64> {
    number(e, "interval_ms")
        .filter(|n| *n > 0.0)
        .map(|n| (n * 1e6) as u64)
}
fn milliseconds(n: u64) -> f64 {
    n as f64 / 1e6
}
#[derive(Default)]
struct Frame<'a> {
    decision: Option<&'a RecordedSpan>,
    admission: Option<&'a RecordedSpan>,
    pulse: Option<&'a RecordedSpan>,
    cycle: Option<&'a RecordedSpan>,
    queue: Option<&'a RecordedSpan>,
    commit: Option<&'a RecordedSpan>,
    scroll: bool,
    input_count: usize,
    wheel_input_count: usize,
    cancelled_wheel: usize,
    raster_tasks: Option<f64>,
    reused_tiles: Option<f64>,
    stages: Vec<u64>,
}
fn period(frame: Option<&Frame<'_>>) -> Option<u64> {
    frame.and_then(|f| {
        f.pulse
            .and_then(interval)
            .or_else(|| f.cycle.and_then(interval))
    })
}
fn boundary(f: &Frame<'_>) -> Option<u64> {
    f.pulse
        .map(|p| p.start_ns)
        .or_else(|| f.queue.map(|p| p.start_ns))
        .or_else(|| {
            f.cycle.and_then(|c| {
                number(c, "start_age_ms").map(|age| c.start_ns.saturating_sub((age * 1e6) as u64))
            })
        })
}
fn percentiles(mut a: Vec<f64>) -> Value {
    a.retain(|n| n.is_finite());
    a.sort_by(f64::total_cmp);
    let p = |n: usize| {
        if a.is_empty() {
            Value::Null
        } else {
            json!(a[(a.len() * n).div_ceil(100).saturating_sub(1)])
        }
    };
    json!({"count":a.len(),"p50_ms":p(50),"p95_ms":p(95),"max_ms":a.last()})
}
fn issue(
    issues: &mut Vec<Value>,
    kind: &str,
    label: &str,
    start: u64,
    finish: u64,
    anchor: &RecordedSpan,
    related: Vec<u64>,
    evidence: Value,
) {
    issues.push(json!({"kind":kind,"label":label,"start_ns":start,"duration_ns":finish.saturating_sub(start),
        "event_id":anchor.id,"related_event_ids":related,"context":anchor.context,"evidence":evidence}));
}

pub fn analyze(s: &Snapshot) -> Value {
    let mut frames: BTreeMap<Key, Frame<'_>> = BTreeMap::new();
    let mut commits: BTreeMap<(u64, u64), Vec<&RecordedSpan>> = BTreeMap::new();
    let mut wheels: BTreeMap<u64, Vec<&RecordedSpan>> = BTreeMap::new();
    let mut motion: BTreeMap<(u64, u64), Vec<&RecordedSpan>> = BTreeMap::new();
    let mut wheel_ids = BTreeSet::new();
    for e in &s.instants {
        match e.name.as_str() {
            "VSync" => frames.entry(key(e)).or_default().pulse = Some(e),
            "ScrollFrameDecision" => frames.entry(key(e)).or_default().decision = Some(e),
            "LateScrollAdmissionDecision" => frames.entry(key(e)).or_default().admission = Some(e),
            "WheelSample" => {
                wheels.entry(e.context.target_id).or_default().push(e);
                wheel_ids.insert(e.context.input_id);
            }
            "ScrollApplied" => {
                frames.entry(key(e)).or_default().scroll = true;
                motion
                    .entry((
                        e.context.target_id,
                        number(e, "scroll_node").unwrap_or(0.) as u64,
                    ))
                    .or_default()
                    .push(e);
            }
            _ => {}
        }
    }
    let mut issues = Vec::new();
    let mut latencies = Vec::new();
    let mut owner_wait = Vec::new();
    let mut input_wait = Vec::new();
    let mut present_wait = Vec::new();
    let mut failures = 0;
    let mut failure_keys = BTreeSet::new();
    let mut no_frame_inputs = 0;
    for e in &s.spans {
        if e.incomplete {
            continue;
        }
        let frame = frames.entry(key(e)).or_default();
        if let Some(n) = number(e, "raster_tasks") {
            frame.raster_tasks = Some(frame.raster_tasks.unwrap_or(0.).max(n));
        }
        if let Some(n) = number(e, "reused_tiles") {
            frame.reused_tiles = Some(frame.reused_tiles.unwrap_or(0.).max(n));
        }
        if matches!(
            e.name.as_str(),
            "Page.DispatchFrameInput"
                | "RasterTiles"
                | "ComposeTiles"
                | "LayoutEngine.Layout"
                | "PaintEngine.Paint"
                | "Paint.RecordPaint"
                | "PresentReturn"
        ) {
            frame.stages.push(e.id);
        }
        if e.name == "Page.DispatchFrameInput" {
            frame.input_count += 1;
            if number(e, "kind") == Some(1.) {
                frame.wheel_input_count += 1;
                if number(e, "default_prevented") == Some(1.) {
                    frame.cancelled_wheel += 1;
                }
            }
        }
        match e.name.as_str() {
            "FrameCycle" => frame.cycle = Some(e),
            "BeginFrameQueue" => frame.queue = Some(e),
            "PresentReturn" if number(e, "succeeded") == Some(1.) => {
                frame.commit = Some(e);
                commits
                    .entry((e.context.target_id, e.context.source_id))
                    .or_default()
                    .push(e);
            }
            "Page.ApplyPendingScrollUpdates" => {
                frame.scroll |= number(e, "pending_scrolls").unwrap_or(0.) > 0.
                    && number(e, "resolved_noop") != Some(1.)
            }
            "InputToPresentReturn" => latencies.push(milliseconds(e.duration_ns)),
            "InputOwnerQueue" => owner_wait.push(milliseconds(e.duration_ns)),
            "InputQueue" => input_wait.push(milliseconds(e.duration_ns)),
            "NativePresentQueue" => present_wait.push(milliseconds(e.duration_ns)),
            "InputNoFrame" => {
                if wheel_ids.contains(&e.context.input_id) {
                    no_frame_inputs += 1;
                }
            }
            "InputPresentationFailed" => {
                if failure_keys.insert(if e.context.frame_id == 0 {
                    (e.context.target_id, e.context.source_id, e.id)
                } else {
                    key(e)
                }) {
                    failures += 1;
                    issue(
                        &mut issues,
                        "presentation_failure",
                        "Presentation failed",
                        e.start_ns,
                        end(e),
                        e,
                        vec![],
                        json!({"measured":"platform commit error"}),
                    );
                }
            }
            _ => {}
        }
        if e.name == "PresentReturn"
            && number(e, "succeeded") == Some(0.)
            && failure_keys.insert(if e.context.frame_id == 0 {
                (e.context.target_id, e.context.source_id, e.id)
            } else {
                key(e)
            })
        {
            failures += 1;
            issue(
                &mut issues,
                "presentation_failure",
                "Presentation failed",
                e.start_ns,
                end(e),
                e,
                vec![],
                json!({"measured":"platform commit error"}),
            );
        }
    }
    for list in wheels.values_mut() {
        list.sort_by_key(|e| e.start_ns);
    }
    for list in commits.values_mut() {
        list.sort_by_key(|e| end(e));
    }
    // These are native *estimated* display opportunities, not presentation
    // feedback. Keep callback time and display time separate: the target of
    // an earlier callback can precede this callback's own output target.
    let mut display_targets: BTreeMap<(u64, u64), Vec<u64>> = BTreeMap::new();
    for (k, f) in &frames {
        if let Some(pulse) = f.pulse {
            if let Some(offset) = number(pulse, "display_target_offset_ms").filter(|n| *n > 0.) {
                display_targets
                    .entry((k.0, k.1))
                    .or_default()
                    .push(pulse.start_ns.saturating_add((offset * 1e6) as u64));
            }
        }
    }
    for targets in display_targets.values_mut() {
        targets.sort_unstable();
        targets.dedup();
    }
    let mut estimated_opportunities = BTreeMap::new();
    let mut display_phase_gaps = 0;
    let mut display_phase_collisions = 0;
    for (source, list) in &commits {
        let Some(targets) = display_targets.get(source) else {
            continue;
        };
        let mut previous: Option<(&RecordedSpan, u64)> = None;
        for commit in list {
            let time = end(commit);
            let Some(&target) = targets.get(targets.partition_point(|t| *t < time)) else {
                continue;
            };
            estimated_opportunities.insert(key(commit), target);
            if let Some((old, old_target)) = previous {
                let old_frame = frames.get(&key(old));
                let frame = frames.get(&key(commit));
                if let Some(p) = period(frame).or_else(|| period(old_frame)) {
                    let commit_gap = time.saturating_sub(end(old));
                    // Diagnose phase crossings even when submission cadence
                    // is healthy; gesture pauses are handled separately.
                    if commit_gap <= p + p / 2
                        && old_frame.is_some_and(|f| f.scroll)
                        && frame.is_some_and(|f| f.scroll)
                    {
                        let display_gap = target.saturating_sub(old_target);
                        let anomaly = if display_gap > p + p / 2 {
                            display_phase_gaps += 1;
                            Some(("estimated_display_gap", "Estimated display opportunity gap"))
                        } else if display_gap < p / 2 {
                            display_phase_collisions += 1;
                            Some((
                                "estimated_display_collision",
                                "Commits share an estimated display opportunity",
                            ))
                        } else {
                            None
                        };
                        if let Some((kind, label)) = anomaly {
                            issue(
                                &mut issues,
                                kind,
                                label,
                                end(old),
                                time,
                                commit,
                                vec![old.id, commit.id],
                                json!({
                                "commit_interval_ms":milliseconds(commit_gap),
                                "estimated_display_interval_ms":milliseconds(display_gap),
                                "previous_estimated_display_ns":old_target,"estimated_display_ns":target,
                                "submit_to_estimated_display_ms":milliseconds(target-time),
                                "model":"Earliest recorded native display target at or after commit; CA latch time is unobserved",
                                "physical_display":"unknown"}),
                            );
                        }
                    }
                }
            }
            previous = Some((commit, target));
        }
    }
    let mut ledger = Vec::new();
    let mut no_submit = 0;
    let mut idle = 0;
    let mut unobserved = 0;
    let mut deadline_misses = 0;
    let mut source_periods: BTreeMap<u64, Vec<(u64, u64)>> = BTreeMap::new();
    for (k, f) in &frames {
        if k.2 == 0 || (f.pulse.is_none() && f.cycle.is_none()) {
            continue;
        }
        let Some(time) = boundary(f) else {
            continue;
        };
        let p = period(Some(f));
        if let Some(p) = p {
            source_periods.entry(k.0).or_default().push((time, p));
        }
        let missed = f
            .cycle
            .is_some_and(|e| number(e, "deadline_missed") == Some(1.));
        let status = if f.commit.is_some() {
            "committed"
        } else if f.cycle.is_some_and(|e| number(e, "submitted") == Some(0.)) {
            no_submit += 1;
            "no_submit"
        } else if f.cycle.is_some() {
            "commit_unobserved"
        } else if f.pulse.is_some_and(|e| number(e, "requested") == Some(0.)) {
            idle += 1;
            "idle"
        } else {
            unobserved += 1;
            "not_observed"
        };
        let anchor = f.cycle.or(f.pulse).unwrap();
        if status == "not_observed"
            && f.pulse.is_some_and(|e| number(e, "requested") == Some(1.))
            && p.is_some_and(|p| time.saturating_add(p.saturating_mul(2)) <= s.duration_ns)
        {
            issue(
                &mut issues,
                "unhandled_pulse",
                "Requested VSync not handled",
                time,
                time.saturating_add(p.unwrap()),
                anchor,
                vec![],
                json!({"requested":true,"physical_display":"unknown","handling":"no recorded cycle"}),
            );
        }
        let cycle_end = f.cycle.map(end);
        let next_wheel = cycle_end
            .and_then(|finish| {
                wheels.get(&k.0).and_then(|samples| {
                    samples.get(samples.partition_point(|e| e.start_ns < finish))
                })
            })
            .filter(|e| p.is_some_and(|p| e.start_ns < time.saturating_add(p)));
        let no_submit_reason = if status != "no_submit" {
            None
        } else if f.scroll {
            Some("Scroll changed without submission")
        } else if f.wheel_input_count > 0 {
            Some("Wheel dispatched without scroll mutation")
        } else if next_wheel.is_some() {
            Some("Wheel arrived after cycle closed")
        } else if f.input_count > 0 {
            Some("Non-wheel input without visual change")
        } else {
            Some("No scroll mutation in cycle")
        };
        let input_deadline_ms = f.decision.and_then(|e| number(e, "input_deadline_ms"));
        let draw_deadline_ms = f
            .cycle
            .and_then(|e| number(e, "deadline_ms"))
            .or_else(|| f.decision.and_then(|e| number(e, "draw_deadline_ms")));
        let source_deadline_ms = f
            .cycle
            .and_then(|e| number(e, "source_deadline_ms"))
            .or_else(|| f.decision.and_then(|e| number(e, "source_deadline_ms")));
        let draw_start_age_ms = f.cycle.and_then(|e| number(e, "draw_start_age_ms"));
        let input_disposition = if f
            .decision
            .is_some_and(|e| number(e, "has_wheel") == Some(1.))
        {
            "ready_at_begin"
        } else if f
            .admission
            .is_some_and(|e| number(e, "admitted") == Some(1.))
        {
            "admitted_late"
        } else if f
            .admission
            .is_some_and(|e| number(e, "admitted") == Some(0.))
        {
            "deferred"
        } else if f
            .decision
            .is_some_and(|e| number(e, "late_window_open") == Some(1.))
        {
            "window_expired_without_input"
        } else {
            "not_waiting"
        };
        ledger.push(json!({"start_ns":time,"interval_ns":p,"target_id":k.0,"source_id":k.1,"frame_id":k.2,
            "status":status,"event_id":anchor.id,"commit_event_id":f.commit.map(|e|e.id),"commit_ns":f.commit.map(|e|end(e)),
            "estimated_display_ns":estimated_opportunities.get(k),
            "submit_to_estimated_display_ms":f.commit.zip(estimated_opportunities.get(k)).map(|(c,t)|milliseconds(t.saturating_sub(end(c)))),
            "work_ns":f.cycle.map(|e| e.duration_ns.saturating_sub((number(e,"late_scroll_wait_ms").unwrap_or(0.)*1e6) as u64)),
            "wait_ms":f.cycle.and_then(|e|number(e,"late_scroll_wait_ms")),"deadline_missed":missed,
            "input_count":f.input_count,"cancelled_wheel":f.cancelled_wheel,"raster_tasks":f.raster_tasks,"reused_tiles":f.reused_tiles,"related_event_ids":f.stages,
            "scroll_active":f.cycle.and_then(|e|number(e,"scroll_active")),"scroll_changed":f.scroll,
            "wheel_dispatch_count":f.wheel_input_count,"no_submit_reason":no_submit_reason,
            "next_wheel_event_id":next_wheel.map(|e|e.id),
            "next_wheel_after_close_ms":next_wheel.zip(cycle_end).map(|(e,t)|milliseconds(e.start_ns.saturating_sub(t))),
            "input_deadline_ns":input_deadline_ms.map(|v|time.saturating_add((v*1e6) as u64)),
            "draw_deadline_ns":draw_deadline_ms.map(|v|time.saturating_add((v*1e6) as u64)),
            "source_deadline_ns":source_deadline_ms.map(|v|time.saturating_add((v*1e6) as u64)),
            "draw_start_ns":draw_start_age_ms.map(|v|time.saturating_add((v*1e6) as u64)),
            "input_disposition":input_disposition,
            "decision_event_id":f.decision.map(|e|e.id),"decision":f.decision.map(|e|&e.fields),
            "admission_event_id":f.admission.map(|e|e.id),"admission":f.admission.map(|e|&e.fields)}));
        if missed {
            deadline_misses += 1;
            issue(
                &mut issues,
                "deadline",
                "Frame deadline exceeded",
                time,
                end(anchor),
                anchor,
                vec![],
                json!({"interval_ms":p.map(milliseconds),"start_age_ms":number(anchor,"start_age_ms"),"cycle_ms":milliseconds(anchor.duration_ns),"wait_ms":number(anchor,"late_scroll_wait_ms"),"physical_display":"unknown"}),
            );
        }
        if let (Some(q), Some(p)) = (f.queue, p) {
            if q.duration_ns > p {
                issue(
                    &mut issues,
                    "owner_delay",
                    "BeginFrame owner delay",
                    q.start_ns,
                    end(q),
                    q,
                    vec![anchor.id],
                    json!({"queue_ms":milliseconds(q.duration_ns),"budget_ms":milliseconds(p)}),
                );
            }
        }
    }
    ledger.sort_by_key(|e| e["start_ns"].as_u64().unwrap_or(0));
    for samples in source_periods.values_mut() {
        samples.sort_unstable();
    }
    let mut gaps = Vec::new();
    let mut gap_count = 0;
    let mut intervals = Vec::new();
    for ((target, source), list) in &commits {
        for pair in list.windows(2) {
            let a = pair[0];
            let b = pair[1];
            let gap = end(b).saturating_sub(end(a));
            let bf = frames.get(&key(b));
            let af = frames.get(&key(a));
            let Some(p) = period(bf).or_else(|| period(af)) else {
                continue;
            };
            // A still page is not a dropped frame. Restrict cadence diagnostics
            // to nearby scroll commits or explicitly active gestures.
            let nearby_scroll = gap <= p.saturating_mul(4)
                && (af.is_some_and(|f| f.scroll) || bf.is_some_and(|f| f.scroll));
            let active = af
                .and_then(|f| f.cycle)
                .is_some_and(|e| number(e, "scroll_active") == Some(1.))
                && bf
                    .and_then(|f| f.cycle)
                    .is_some_and(|e| number(e, "scroll_active") == Some(1.));
            if !nearby_scroll && !active {
                continue;
            }
            gaps.push(milliseconds(gap));
            intervals.push(
                json!({"start_ns":end(a),"duration_ns":gap,"interval_ns":p,"context":b.context}),
            );
            if gap <= p + p / 2 {
                continue;
            }
            gap_count += 1;
            let mut related = vec![a.id, b.id];
            let mut empty = Vec::new();
            let mut reason = "Unclassified commit gap";
            for (k, f) in frames.range(
                (*target, *source, a.context.frame_id.saturating_add(1))
                    ..(
                        *target,
                        *source,
                        b.context.frame_id.max(a.context.frame_id.saturating_add(1)),
                    ),
            ) {
                let Some(c) = f.cycle else {
                    continue;
                };
                related.push(c.id);
                if number(c, "submitted") != Some(0.) {
                    continue;
                }
                let Some(t) = boundary(f) else {
                    continue;
                };
                let next = wheels
                    .get(target)
                    .and_then(|samples| samples.get(samples.partition_point(|e| e.start_ns < t)))
                    .filter(|e| e.start_ns <= t.saturating_add(p));
                let delay = next.map(|e| milliseconds(e.start_ns.saturating_sub(t)));
                let wait = number(c, "late_scroll_wait_ms").unwrap_or(0.);
                if wait > 0. && delay.is_some_and(|delay| delay > wait) {
                    reason = "Input arrived after late-scroll wait";
                } else if next.is_some_and(|e| e.start_ns > end(c)) {
                    reason = "Wheel arrived after cycle closed";
                } else if reason == "Unclassified commit gap" {
                    reason = "No submission in intervening cycle";
                }
                empty.push(json!({"frame_id":k.2,"event_id":c.id,"wait_ms":wait,"next_wheel_after_ms":delay,"cycle_closed_after_ms":milliseconds(end(c).saturating_sub(t)),"next_wheel_event_id":next.map(|e|e.id)}));
            }
            issue(
                &mut issues,
                "commit_gap",
                "Scroll commit gap",
                end(a),
                end(b),
                b,
                related,
                json!({"gap_ms":milliseconds(gap),"interval_ms":milliseconds(p),"vsync_steps":b.context.frame_id.saturating_sub(a.context.frame_id),"reason":reason,"empty_cycles":empty,"physical_display":"unknown"}),
            );
        }
    }
    // Per-event budgets follow the recorded source; no assumed 60 Hz threshold.
    for e in &s.spans {
        if e.incomplete {
            continue;
        }
        let p = period(frames.get(&key(e))).or_else(|| {
            source_periods
                .get(&e.context.target_id)
                .and_then(|samples| {
                    samples.get(
                        samples
                            .partition_point(|(t, _)| *t <= e.start_ns)
                            .saturating_sub(1),
                    )
                })
                .map(|(_, p)| *p)
        });
        let Some(p) = p else {
            continue;
        };
        let (kind, label) = match e.name.as_str() {
            "InputToPresentReturn"
                if wheel_ids.is_empty() || wheel_ids.contains(&e.context.input_id) =>
            {
                ("input_latency", "Input to commit > one interval")
            }
            "InputOwnerQueue" => ("input_owner_wait", "Native input owner wait"),
            "InputQueue" => ("input_frame_wait", "Frame-aligned input wait"),
            "NativePresentQueue" => ("native_present_wait", "Window-thread presentation wait"),
            "ForegroundTasks" | "BackgroundTasks" => {
                ("long_task", "Main-thread task > one interval")
            }
            "RasterTiles" | "ComposeTiles" | "Renderer.Raster" | "Renderer.Compose" | "Raster"
            | "Compose" => ("stage_budget", "Render stage > one interval"),
            _ => continue,
        };
        if e.duration_ns > p {
            issue(
                &mut issues,
                kind,
                label,
                e.start_ns,
                end(e),
                e,
                vec![],
                json!({"duration_ms":milliseconds(e.duration_ns),"interval_ms":milliseconds(p),"stage":e.name}),
            );
        }
    }
    let mut series = Vec::new();
    for ((target, node), mut points) in motion {
        points.sort_by_key(|e| e.start_ns);
        let all_commits: Vec<_> = commits
            .iter()
            .filter(|((t, _), _)| *t == target)
            .flat_map(|(_, list)| list.iter().copied())
            .collect();
        let mut all_commits = all_commits;
        all_commits.sort_by_key(|e| end(e));
        let mut grouped: Vec<(&RecordedSpan, Option<&RecordedSpan>)> = Vec::new();
        for e in points {
            let commit = all_commits
                .get(all_commits.partition_point(|c| end(c) < e.start_ns))
                .copied();
            if commit.is_some()
                && grouped
                    .last()
                    .is_some_and(|(_, last)| last.map(|c| c.id) == commit.map(|c| c.id))
            {
                *grouped.last_mut().unwrap() = (e, commit);
            } else {
                grouped.push((e, commit));
            }
        }
        let mut last: Option<(u64, f64, f64)> = None;
        let mut velocities = Vec::new();
        let data: Vec<_> = grouped
            .iter()
            .map(|(e, commit)| {
                // Link state mutations to the next successful submission. A failed
                // or absent submission leaves the visual relationship unknown.
                let time = commit.map(end);
                let x = number(e, "offset_x");
                let y = number(e, "offset_y");
                let velocity = match (last, time, x, y) {
                    (Some((t, px, py)), Some(now), Some(x), Some(y)) if now > t => {
                        Some(((x - px).hypot(y - py)) / (milliseconds(now - t) / 1000.))
                    }
                    _ => None,
                };
                if let Some(v) = velocity {
                    velocities.push(v);
                }
                if let (Some(t), Some(x), Some(y)) = (time, x, y) {
                    last = Some((t, x, y));
                }
                json!({"event_id":e.id,"start_ns":e.start_ns,"context":e.context,"fields":e.fields,
                "commit_ns":time,"commit_event_id":commit.map(|e|e.id),"speed_css_px_s":velocity})
            })
            .collect();
        series.push(json!({"target_id":target,"scroll_node":node,"points":data,"coordinate_unit":"CSS px",
            "speed_range_css_px_s":{"min":velocities.iter().copied().reduce(f64::min),"max":velocities.iter().copied().reduce(f64::max)}}));
    }
    issues.sort_by_key(|e| e["start_ns"].as_u64().unwrap_or(0));
    for (i, e) in issues.iter_mut().enumerate() {
        e["id"] = json!(i + 1);
        e["confidence"] = json!(if s.dropped_events > 0 {
            "partial capture"
        } else if e["kind"]
            .as_str()
            .is_some_and(|k| k.starts_with("estimated_display_"))
        {
            "estimated"
        } else {
            "observed"
        });
    }
    let complete = s.dropped_events == 0 && !s.spans.iter().any(|e| e.incomplete);
    json!({"version":1,"coverage":{
        "capture":if complete {"complete"} else {"partial"},
        "vsync":if s.instants.iter().any(|e|e.name=="VSync") {"source pulses"} else {"handled boundaries only"},
        "wheel_samples":if wheel_ids.is_empty() {"unavailable"} else {"native UI enqueue"},
        "scroll_motion":if series.is_empty() {"unavailable"} else {"logical CSS offsets at commit"},
        "native_to_ui_latency":"unavailable","presentation":if present_wait.is_empty() {"commit API return only; window-thread receipt unobserved"} else {"window-thread receipt and transaction return; not scanout"},
        "physical_display":"unavailable","displayed_scroll_motion":"unavailable",
        "display_opportunities":if display_targets.is_empty() {"unavailable"} else {"native estimated targets; not physical presentation feedback"},
        "js":"instrumented callbacks and tasks; no sampled JS stack"},
        "metrics":{"vsync_pulses":s.instants.iter().filter(|e|e.name=="VSync").count(),"handled_cycles":ledger.iter().filter(|e|e["work_ns"].is_number()).count(),
        "commits":commits.values().map(Vec::len).sum::<usize>(),"no_submit_cycles":no_submit,"idle_pulses":idle,"unobserved_pulses":unobserved,
        "deadline_misses":deadline_misses,"scroll_commit_gaps":gap_count,"presentation_failures":failures,"wheel_no_frame_outcomes":no_frame_inputs,"cancelled_wheel_updates":frames.values().map(|f|f.cancelled_wheel).sum::<usize>(),
        "estimated_display_gaps":display_phase_gaps,"estimated_display_collisions":display_phase_collisions,
        "scroll_commit_interval":percentiles(gaps),"input_to_commit":percentiles(latencies),"input_owner_wait":percentiles(owner_wait),"input_frame_wait":percentiles(input_wait),"native_present_wait":percentiles(present_wait)},
        "frames":ledger,"issues":issues,"scroll_series":series,"commit_intervals":intervals})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(
        id: u64,
        name: &str,
        start: u64,
        duration: u64,
        sequence: u64,
        fields: &[(&str, f64)],
    ) -> RecordedSpan {
        RecordedSpan {
            id,
            parent_id: 0,
            thread_id: 1,
            category: "frame".into(),
            name: name.into(),
            start_ns: start,
            duration_ns: duration,
            incomplete: false,
            context: browser_tracing::Context {
                target_id: 1,
                source_id: 7,
                frame_id: sequence,
                input_id: if name == "WheelSample" { 8 } else { 0 },
                ..Default::default()
            },
            fields: fields.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        }
    }
    fn snapshot(spans: Vec<RecordedSpan>, instants: Vec<RecordedSpan>) -> Snapshot {
        Snapshot {
            version: 1,
            recording_id: 1,
            active: false,
            duration_ns: 2_000_000_000,
            dropped_events: 0,
            spans,
            instants,
        }
    }
    #[test]
    fn short_frames_can_cross_then_share_native_display_opportunities() {
        let mut instants: Vec<_> = (0..5)
            .map(|i| {
                event(
                    10 + i,
                    "VSync",
                    i * 10_000_000,
                    0,
                    i + 1,
                    &[
                        ("interval_ms", 10.),
                        ("display_target_offset_ms", 14.),
                        ("requested", 1.),
                    ],
                )
            })
            .collect();
        for i in 0..3 {
            instants.push(event(
                20 + i,
                "ScrollApplied",
                i * 10_000_000 + 1_000_000,
                0,
                i + 1,
                &[("offset_y", i as f64 * 10.)],
            ));
        }
        let spans = vec![
            event(
                1,
                "FrameCycle",
                0,
                3_000_000,
                1,
                &[("interval_ms", 10.), ("submitted", 1.)],
            ),
            event(
                2,
                "FrameCycle",
                10_000_000,
                5_000_000,
                2,
                &[("interval_ms", 10.), ("submitted", 1.)],
            ),
            event(
                3,
                "FrameCycle",
                20_000_000,
                3_000_000,
                3,
                &[("interval_ms", 10.), ("submitted", 1.)],
            ),
            event(
                4,
                "PresentReturn",
                2_900_000,
                100_000,
                1,
                &[("succeeded", 1.)],
            ),
            event(
                5,
                "PresentReturn",
                14_900_000,
                100_000,
                2,
                &[("succeeded", 1.)],
            ),
            event(
                6,
                "PresentReturn",
                22_900_000,
                100_000,
                3,
                &[("succeeded", 1.)],
            ),
        ];
        // Cover the preceding callback's target, which is the first
        // opportunity here. This must not become a synthetic VSync pulse.
        instants.push(event(
            9,
            "VSync",
            0,
            0,
            99,
            &[
                ("interval_ms", 10.),
                ("display_target_offset_ms", 4.),
                ("requested", 0.),
            ],
        ));
        let r = analyze(&snapshot(spans, instants));
        assert_eq!(r["metrics"]["deadline_misses"], 0);
        assert_eq!(r["metrics"]["scroll_commit_gaps"], 0);
        assert_eq!(r["metrics"]["estimated_display_gaps"], 1);
        assert_eq!(r["metrics"]["estimated_display_collisions"], 1);
        assert_eq!(r["coverage"]["physical_display"], "unavailable");
        let gap = r["issues"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["kind"] == "estimated_display_gap")
            .unwrap();
        assert_eq!(gap["confidence"], "estimated");
        assert_eq!(gap["evidence"]["commit_interval_ms"], 12.);
        assert_eq!(gap["evidence"]["estimated_display_interval_ms"], 20.);
    }
    #[test]
    fn idle_source_pulses_are_not_labeled_as_dropped_scroll_frames() {
        let pulses = (0..100)
            .map(|i| {
                event(
                    i + 1,
                    "VSync",
                    i * 10_000_000,
                    0,
                    i + 1,
                    &[("interval_ms", 10.), ("requested", 0.)],
                )
            })
            .collect();
        let r = analyze(&snapshot(vec![], pulses));
        assert_eq!(r["metrics"]["idle_pulses"], 100);
        assert_eq!(r["metrics"]["scroll_commit_gaps"], 0);
        assert_eq!(r["issues"].as_array().unwrap().len(), 0);
        assert_eq!(r["coverage"]["physical_display"], "unavailable");
    }
    #[test]
    fn fast_work_can_have_a_commit_gap_and_late_input_at_non_60hz() {
        let instants = vec![
            event(
                10,
                "VSync",
                0,
                0,
                1,
                &[("interval_ms", 10.), ("requested", 1.)],
            ),
            event(
                11,
                "VSync",
                10_000_000,
                0,
                2,
                &[("interval_ms", 10.), ("requested", 1.)],
            ),
            event(
                12,
                "VSync",
                20_000_000,
                0,
                3,
                &[("interval_ms", 10.), ("requested", 1.)],
            ),
            event(13, "WheelSample", 14_000_000, 0, 0, &[("delta_y", 5.)]),
        ];
        let spans = vec![
            event(
                1,
                "FrameCycle",
                0,
                6_000_000,
                1,
                &[
                    ("interval_ms", 10.),
                    ("submitted", 1.),
                    ("scroll_active", 1.),
                ],
            ),
            event(
                2,
                "FrameCycle",
                10_000_000,
                3_000_000,
                2,
                &[
                    ("interval_ms", 10.),
                    ("submitted", 0.),
                    ("scroll_active", 1.),
                    ("late_scroll_wait_ms", 3.),
                ],
            ),
            event(
                3,
                "FrameCycle",
                20_000_000,
                6_000_000,
                3,
                &[
                    ("interval_ms", 10.),
                    ("submitted", 1.),
                    ("scroll_active", 1.),
                ],
            ),
            event(
                4,
                "PresentReturn",
                5_000_000,
                100_000,
                1,
                &[("succeeded", 1.)],
            ),
            event(
                5,
                "PresentReturn",
                25_000_000,
                100_000,
                3,
                &[("succeeded", 1.)],
            ),
        ];
        let r = analyze(&snapshot(spans, instants));
        assert_eq!(r["metrics"]["scroll_commit_gaps"], 1);
        assert_eq!(r["metrics"]["no_submit_cycles"], 1);
        let gap = r["issues"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["kind"] == "commit_gap")
            .unwrap();
        assert_eq!(gap["evidence"]["gap_ms"], 20.);
        assert_eq!(
            gap["evidence"]["reason"],
            "Input arrived after late-scroll wait"
        );
        assert_eq!(
            gap["evidence"]["empty_cycles"][0]["next_wheel_after_ms"],
            4.
        );
    }
}
