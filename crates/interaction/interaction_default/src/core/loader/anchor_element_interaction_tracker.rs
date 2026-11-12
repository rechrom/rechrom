// Copyright The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: anchor_element_interaction_tracker.h:99-184; .cc:369-487,535-605,
// 615-640,775-791. Timers and browser speculative loading are injected services.
use interaction_dispatch::core::{
    dom::events::event::{EventInterface, EventTargetRef},
    events::pointer_event::PointerEventRef,
};
use interaction_input::base::time::time::TimeTicks;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpeculationEagerness {
    Eager,
    Moderate,
}
#[derive(Clone, Copy, Debug)]
pub struct AnchorPointerData {
    pub is_mouse: bool,
    pub velocity: f64,
    pub tangential_acceleration: f64,
}
#[derive(Clone, Copy)]
struct HoverCandidate {
    is_mouse: bool,
    anchor_id: u32,
    timestamp: TimeTicks,
}

pub trait AnchorInteractionEnvironment {
    /// The same MouseMotionEstimator that supplies MouseMotion below receives
    /// screen positions here. Its clock, samples and task timer stay retained
    /// with the source estimator, not with an EventHandler invocation.
    fn OnMouseMoveEvent(&self, screen_position: foundation::gfx::PointF);
    fn IsNode(&self, target: &EventTargetRef) -> bool;
    fn FirstAnchorElementIncludingSelf(&self, target: &EventTargetRef) -> Option<u64>;
    fn Href(&self, anchor: u64) -> String;
    fn DocumentURL(&self) -> String;
    fn NoSamePageFragmentAnchorTracking(&self) -> bool;
    fn RecordPointerDown(&self, event: &PointerEventRef);
    fn StopModerateViewportTimer(&self);
    fn ReportAnchorPointerEvent(&self, anchor: u64, event: &PointerEventRef);
    fn InteractionHostBound(&self) -> bool;
    fn RendererSideHeuristics(&self) -> bool;
    fn OnPointerDownHeuristic(&self, url: &str) -> bool;
    fn NotifyPointerDown(&self, url: &str, renderer_enacted: bool);
    fn PreloadingEligibilityCheckOnRenderer(&self) -> bool;
    fn SaveDataEnabled(&self) -> bool;
    fn BatterySaverEnabled(&self) -> bool;
    fn PreloadingDisabled(&self) -> bool;
    fn EagerHoverHeuristics(&self) -> bool;
    fn EagerHoverDwellSeconds(&self) -> f64;
    fn AnchorId(&self, anchor: u64) -> u32;
    fn Now(&self) -> TimeTicks;
    fn StartHoverTimer(&self, delay_seconds: f64);
    fn MouseMotion(&self) -> (f64, f64);
    fn ReportAnchorPointerData(&self, anchor: u32, data: AnchorPointerData);
    fn OnHoverHeuristic(&self, url: &str, eagerness: SpeculationEagerness) -> bool;
    fn NotifyPointerHover(
        &self,
        url: &str,
        eagerness: SpeculationEagerness,
        data: AnchorPointerData,
        renderer_enacted: bool,
    );
}

pub struct AnchorElementInteractionTracker {
    environment: Rc<dyn AnchorInteractionEnvironment>,
    last_pointer_down_locations: Cell<[Option<f64>; 2]>,
    hover_candidates: RefCell<HashMap<(String, SpeculationEagerness), HoverCandidate>>,
    hover_timer_active: Cell<bool>,
}
impl AnchorElementInteractionTracker {
    pub fn new(environment: Rc<dyn AnchorInteractionEnvironment>) -> Self {
        Self {
            environment,
            last_pointer_down_locations: Cell::new([None, None]),
            hover_candidates: RefCell::new(HashMap::new()),
            hover_timer_active: Cell::new(false),
        }
    }
    // Source .cc:369-372.
    pub fn OnMouseMoveEvent(
        &self,
        event: &interaction_input::public::common::input::web_mouse_event::WebMouseEvent,
    ) {
        let point = event.pointer_properties.PositionInScreen();
        self.environment
            .OnMouseMoveEvent(foundation::gfx::PointF::new(point.x(), point.y()));
    }
    fn Eligible(&self) -> bool {
        let e = &self.environment;
        !e.PreloadingEligibilityCheckOnRenderer()
            || !(e.SaveDataEnabled() || e.BatterySaverEnabled() || e.PreloadingDisabled())
    }
    fn HrefEligible(&self, anchor: u64) -> Option<String> {
        let e = &self.environment;
        let raw = e.Href(anchor);
        let url = url::Url::parse(&raw).ok()?;
        if !matches!(url.scheme(), "http" | "https") {
            return None;
        }
        if e.NoSamePageFragmentAnchorTracking() && url.fragment().is_some() {
            if let Ok(mut document) = url::Url::parse(&e.DocumentURL()) {
                let mut candidate = url.clone();
                candidate.set_fragment(None);
                document.set_fragment(None);
                if candidate == document {
                    return None;
                }
            }
        }
        Some(raw)
    }
    // Source .cc:374-487. Do not retain a concrete PointerEvent RefCell borrow
    // through metrics, speculation rules or browser host calls.
    pub fn OnPointerEvent(&self, target: &EventTargetRef, event: &PointerEventRef) {
        let e = &self.environment;
        if !e.IsNode(target) {
            return;
        }
        let (primary, kind, screen_y, link_button, is_mouse) = {
            let event = event.borrow();
            (
                event.isPrimary(),
                event.BaseEvent().r#type().to_owned(),
                event.screenY(),
                event.IsLinkClickButton(),
                event.pointerType() == "mouse",
            )
        };
        if !primary {
            return;
        }
        if kind == "pointerdown" {
            let old = self.last_pointer_down_locations.get();
            self.last_pointer_down_locations
                .set([Some(screen_y), old[0]]);
            e.RecordPointerDown(event);
            e.StopModerateViewportTimer();
        }
        let Some(anchor) = e.FirstAnchorElementIncludingSelf(target) else {
            return;
        };
        let Some(url) = self.HrefEligible(anchor) else {
            return;
        };
        e.ReportAnchorPointerEvent(anchor, event);
        if !e.InteractionHostBound() {
            return;
        }
        if kind == "pointerdown" {
            if !link_button {
                return;
            }
            let enacted = e.RendererSideHeuristics() && e.OnPointerDownHeuristic(&url);
            e.NotifyPointerDown(&url, enacted);
            return;
        }
        if kind == "pointerover" && self.Eligible() {
            let now = e.Now().ToInternalValue();
            let anchor_id = e.AnchorId(anchor);
            let mut candidates = self.hover_candidates.borrow_mut();
            if e.EagerHoverHeuristics() {
                candidates.insert(
                    (url.clone(), SpeculationEagerness::Eager),
                    HoverCandidate {
                        is_mouse,
                        anchor_id,
                        timestamp: TimeTicks::FromInternalValue(
                            now.saturating_add((e.EagerHoverDwellSeconds() * 1e6) as i64),
                        ),
                    },
                );
            }
            candidates.insert(
                (url, SpeculationEagerness::Moderate),
                HoverCandidate {
                    is_mouse,
                    anchor_id,
                    timestamp: TimeTicks::FromInternalValue(now.saturating_add(200_000)),
                },
            );
            drop(candidates);
            if !self.hover_timer_active.replace(true) {
                e.StartHoverTimer(if e.EagerHoverHeuristics() {
                    e.EagerHoverDwellSeconds()
                } else {
                    0.2
                });
            }
        } else if kind == "pointerout" {
            let mut candidates = self.hover_candidates.borrow_mut();
            candidates.remove(&(url.clone(), SpeculationEagerness::Eager));
            candidates.remove(&(url, SpeculationEagerness::Moderate));
        }
    }
    // Source .cc:535-605. Copy only due source candidate values before calls,
    // since the service may synchronously update document speculation rules.
    pub fn HoverTimerFired(&self) {
        self.hover_timer_active.set(false);
        let e = &self.environment;
        if !e.InteractionHostBound() {
            return;
        }
        let now = e.Now();
        let candidates: Vec<_> = self
            .hover_candidates
            .borrow()
            .iter()
            .map(|(key, value)| (key.clone(), *value))
            .collect();
        let mut erased = Vec::new();
        let mut next: Option<TimeTicks> = None;
        for ((url, eagerness), candidate) in candidates {
            if now >= candidate.timestamp {
                let (velocity, tangential_acceleration) = e.MouseMotion();
                let data = AnchorPointerData {
                    is_mouse: candidate.is_mouse,
                    velocity,
                    tangential_acceleration,
                };
                if candidate.is_mouse {
                    e.ReportAnchorPointerData(candidate.anchor_id, data);
                }
                let enacted = e.RendererSideHeuristics() && e.OnHoverHeuristic(&url, eagerness);
                e.NotifyPointerHover(&url, eagerness, data, enacted);
                erased.push((url, eagerness));
            } else {
                next = Some(next.map_or(candidate.timestamp, |next| next.min(candidate.timestamp)));
            }
        }
        let mut candidates = self.hover_candidates.borrow_mut();
        for key in erased {
            candidates.remove(&key);
        }
        drop(candidates);
        if let Some(next) = next {
            self.hover_timer_active.set(true);
            e.StartHoverTimer((next.ToInternalValue() - now.ToInternalValue()) as f64 / 1e6);
        }
    }
    pub fn LastPointerDownLocations(&self) -> [Option<f64>; 2] {
        self.last_pointer_down_locations.get()
    }
}
