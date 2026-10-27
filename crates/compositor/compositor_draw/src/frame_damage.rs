//! Frame-level damage decision over the active draw tree.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameDamageReason {
    #[default]
    None,
    VisibilityChanged,
    ViewportDamage,
    ReferencedSurfacesChanged,
    LocalSurfaceChanged,
    UnsubmittedUnboundedSurface,
    MetadataSequenceChanged,
    UnboundedSurfaceDamage,
    RootSurfaceDamage,
    CopyRequest,
    AnimatedHud,
    ViewTransitionRequest,
}

pub trait FrameDamageContext {
    fn visibility_changed(&mut self) -> bool;
    fn viewport_has_damage(&mut self) -> bool;
    fn referenced_surfaces_changed(&mut self) -> bool;
    fn local_surface_changed(&mut self) -> bool;
    fn has_unsubmitted_unbounded_surface(&mut self) -> bool;
    fn metadata_sequence_changed(&mut self) -> bool;
    fn unbounded_elements_enabled(&mut self) -> bool;
    fn render_surface_count(&mut self) -> usize;
    fn render_surface_has_visible_unbounded_damage(&mut self, index: usize) -> bool;
    fn root_surface_has_visible_damage(&mut self) -> bool;
    fn has_copy_requests(&mut self) -> bool;
    fn hud_wants_to_draw(&mut self) -> bool;
    fn has_view_transition_requests(&mut self) -> bool;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameDamageDecision {
    pub has_damage: bool,
    pub reason: FrameDamageReason,
    pub surfaces_checked: usize,
}

pub fn HasFrameDamage<C: FrameDamageContext>(context: &mut C) -> FrameDamageDecision {
    let mut trace = browser_tracing::span("compositor.draw", "Compositor.Draw.HasFrameDamage");
    let mut decision = FrameDamageDecision::default();
    macro_rules! damaged {
        ($reason:expr) => {{
            decision.has_damage = true;
            decision.reason = $reason;
            trace.set("has_damage", 1.0);
            trace.set("reason", decision.reason as u8 as f64);
            trace.set("surfaces_checked", decision.surfaces_checked as f64);
            return decision;
        }};
    }
    if context.visibility_changed() {
        damaged!(FrameDamageReason::VisibilityChanged);
    }
    if context.viewport_has_damage() {
        damaged!(FrameDamageReason::ViewportDamage);
    }
    if context.referenced_surfaces_changed() {
        damaged!(FrameDamageReason::ReferencedSurfacesChanged);
    }
    if context.local_surface_changed() {
        damaged!(FrameDamageReason::LocalSurfaceChanged);
    }
    if context.has_unsubmitted_unbounded_surface() {
        damaged!(FrameDamageReason::UnsubmittedUnboundedSurface);
    }
    if context.metadata_sequence_changed() {
        damaged!(FrameDamageReason::MetadataSequenceChanged);
    }
    if context.unbounded_elements_enabled() {
        let count = context.render_surface_count();
        for index in 0..count {
            decision.surfaces_checked += 1;
            if context.render_surface_has_visible_unbounded_damage(index) {
                damaged!(FrameDamageReason::UnboundedSurfaceDamage);
            }
        }
    }
    if context.root_surface_has_visible_damage() {
        damaged!(FrameDamageReason::RootSurfaceDamage);
    }
    if context.has_copy_requests() {
        damaged!(FrameDamageReason::CopyRequest);
    }
    if context.hud_wants_to_draw() {
        damaged!(FrameDamageReason::AnimatedHud);
    }
    if context.has_view_transition_requests() {
        damaged!(FrameDamageReason::ViewTransitionRequest);
    }
    trace.set("has_damage", 0.0);
    trace.set("reason", FrameDamageReason::None as u8 as f64);
    trace.set("surfaces_checked", decision.surfaces_checked as f64);
    decision
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Context {
        flags: [bool; 10],
        surfaces: Vec<bool>,
        calls: Vec<&'static str>,
    }

    impl FrameDamageContext for Context {
        fn visibility_changed(&mut self) -> bool {
            self.calls.push("visibility");
            self.flags[0]
        }
        fn viewport_has_damage(&mut self) -> bool {
            self.calls.push("viewport");
            self.flags[1]
        }
        fn referenced_surfaces_changed(&mut self) -> bool {
            self.calls.push("references");
            self.flags[2]
        }
        fn local_surface_changed(&mut self) -> bool {
            self.calls.push("local");
            self.flags[3]
        }
        fn has_unsubmitted_unbounded_surface(&mut self) -> bool {
            self.calls.push("unsubmitted");
            self.flags[4]
        }
        fn metadata_sequence_changed(&mut self) -> bool {
            self.calls.push("metadata");
            self.flags[5]
        }
        fn unbounded_elements_enabled(&mut self) -> bool {
            self.calls.push("unbounded-enabled");
            self.flags[6]
        }
        fn render_surface_count(&mut self) -> usize {
            self.surfaces.len()
        }
        fn render_surface_has_visible_unbounded_damage(&mut self, index: usize) -> bool {
            self.calls.push("surface");
            self.surfaces[index]
        }
        fn root_surface_has_visible_damage(&mut self) -> bool {
            self.calls.push("root");
            self.flags[7]
        }
        fn has_copy_requests(&mut self) -> bool {
            self.calls.push("copy");
            self.flags[8]
        }
        fn hud_wants_to_draw(&mut self) -> bool {
            self.calls.push("hud");
            self.flags[9]
        }
        fn has_view_transition_requests(&mut self) -> bool {
            self.calls.push("transition");
            false
        }
    }

    #[test]
    fn short_circuits_in_source_priority_order() {
        let mut context = Context::default();
        context.flags[1] = true;
        context.flags[7] = true;
        let decision = HasFrameDamage(&mut context);
        assert_eq!(decision.reason, FrameDamageReason::ViewportDamage);
        assert_eq!(context.calls, ["visibility", "viewport"]);
    }

    #[test]
    fn attributes_unbounded_surface_scan_and_stops_at_first_damage() {
        let mut context = Context::default();
        context.flags[6] = true;
        context.surfaces = vec![false, true, true];
        let decision = HasFrameDamage(&mut context);
        assert_eq!(decision.reason, FrameDamageReason::UnboundedSurfaceDamage);
        assert_eq!(decision.surfaces_checked, 2);
        assert!(!context.calls.contains(&"root"));
    }

    #[test]
    fn no_damage_checks_late_sources_after_surface_scan() {
        let mut context = Context::default();
        context.flags[6] = true;
        context.surfaces = vec![false, false];
        let decision = HasFrameDamage(&mut context);
        assert!(!decision.has_damage);
        assert_eq!(decision.surfaces_checked, 2);
        assert_eq!(context.calls.last(), Some(&"transition"));
    }
}
