//! Scroll-update consumption and overscroll routing policy.

use crate::overscroll_behavior::{OverscrollBehavior, Type as OverscrollType};
use compositor_tree::input_geometry::Vector2dF;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollConsumption {
    pub consumed_any_delta: bool,
    pub scrolled_content: bool,
    pub did_scroll: bool,
}

pub fn ClassifyScrollConsumption(
    residual_delta: Vector2dF,
    resolved_delta: Vector2dF,
    caused_scroll_x: bool,
    caused_scroll_y: bool,
    browser_controls_moved: bool,
    epsilon: f32,
) -> ScrollConsumption {
    let consumed_any_delta = (residual_delta.x() - resolved_delta.x()).abs() > epsilon
        || (residual_delta.y() - resolved_delta.y()).abs() > epsilon;
    let scrolled_content = caused_scroll_x || caused_scroll_y;
    ScrollConsumption {
        consumed_any_delta,
        scrolled_content,
        did_scroll: scrolled_content || browser_controls_moved,
    }
}

pub trait OverscrollRoutingContext {
    fn inner_viewport_user_scrollable_delta(&mut self, delta: Vector2dF) -> Vector2dF;
    fn non_root_effects_enabled(&mut self) -> bool;
    fn non_root_has_x_border_effect(&mut self) -> bool;
    fn non_root_has_y_border_effect(&mut self) -> bool;
    fn active_tree_overscroll_behavior(&mut self) -> OverscrollBehavior;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverscrollRoutingInput {
    pub residual_delta: Vector2dF,
    pub accumulated_root_overscroll: Vector2dF,
    pub caused_scroll_x: bool,
    pub caused_scroll_y: bool,
    pub is_root_scroller: bool,
    pub scroll_chain_cut: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OverscrollRoutingResult {
    pub unused_scroll_delta: Vector2dF,
    pub accumulated_root_overscroll: Vector2dF,
    pub did_overscroll_root: bool,
    pub overscroll_behavior: OverscrollBehavior,
}

pub fn ResolveOverscrollRouting<C: OverscrollRoutingContext>(
    context: &mut C,
    input: OverscrollRoutingInput,
) -> OverscrollRoutingResult {
    let mut trace = browser_tracing::span(
        "compositor.input",
        "Compositor.Input.ResolveOverscrollRouting",
    );
    let mut accumulated = input.accumulated_root_overscroll;
    if input.caused_scroll_x {
        accumulated.set_x(0.0);
    }
    if input.caused_scroll_y {
        accumulated.set_y(0.0);
    }

    let mut unused = input.residual_delta;
    if input.is_root_scroller {
        unused = context.inner_viewport_user_scrollable_delta(unused);
        accumulated += unused;
    }
    if !context.non_root_effects_enabled() && !input.is_root_scroller {
        unused = Vector2dF::default();
    }
    if context.non_root_effects_enabled() && !input.is_root_scroller {
        if !context.non_root_has_x_border_effect() {
            unused.set_x(0.0);
        }
        if !context.non_root_has_y_border_effect() {
            unused.set_y(0.0);
        }
    }

    let overscroll_behavior = if input.scroll_chain_cut {
        OverscrollBehavior::FromType(OverscrollType::kNone)
    } else {
        context.active_tree_overscroll_behavior()
    };
    let result = OverscrollRoutingResult {
        unused_scroll_delta: unused,
        accumulated_root_overscroll: accumulated,
        did_overscroll_root: input.is_root_scroller && unused != Vector2dF::default(),
        overscroll_behavior,
    };
    trace.set("root_scroller", input.is_root_scroller as u8 as f64);
    trace.set(
        "did_overscroll_root",
        result.did_overscroll_root as u8 as f64,
    );
    trace.set("unused_x", result.unused_scroll_delta.x() as f64);
    trace.set("unused_y", result.unused_scroll_delta.y() as f64);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Context {
        events: Vec<&'static str>,
        x_effect: bool,
        y_effect: bool,
    }

    impl OverscrollRoutingContext for Context {
        fn inner_viewport_user_scrollable_delta(&mut self, mut delta: Vector2dF) -> Vector2dF {
            self.events.push("inner");
            delta.set_x(0.0);
            delta
        }
        fn non_root_has_x_border_effect(&mut self) -> bool {
            self.events.push("x-effect");
            self.x_effect
        }
        fn non_root_effects_enabled(&mut self) -> bool {
            self.events.push("feature");
            true
        }
        fn non_root_has_y_border_effect(&mut self) -> bool {
            self.events.push("y-effect");
            self.y_effect
        }
        fn active_tree_overscroll_behavior(&mut self) -> OverscrollBehavior {
            self.events.push("behavior");
            OverscrollBehavior::default()
        }
    }

    #[test]
    fn classifies_content_and_controls_independently() {
        let controls = ClassifyScrollConsumption(
            Vector2dF::new(2.0, 3.0),
            Vector2dF::new(2.0, 3.0),
            false,
            false,
            true,
            0.1,
        );
        assert!(!controls.consumed_any_delta);
        assert!(!controls.scrolled_content);
        assert!(controls.did_scroll);
    }

    #[test]
    fn root_path_filters_inner_axis_accumulates_and_avoids_non_root_queries() {
        let mut context = Context::default();
        let result = ResolveOverscrollRouting(
            &mut context,
            OverscrollRoutingInput {
                residual_delta: Vector2dF::new(4.0, 5.0),
                accumulated_root_overscroll: Vector2dF::new(1.0, 2.0),
                caused_scroll_x: true,
                caused_scroll_y: false,
                is_root_scroller: true,
                scroll_chain_cut: false,
            },
        );
        assert_eq!(result.unused_scroll_delta, Vector2dF::new(0.0, 5.0));
        assert_eq!(result.accumulated_root_overscroll, Vector2dF::new(0.0, 7.0));
        assert_eq!(context.events, ["inner", "feature", "feature", "behavior"]);
    }

    #[test]
    fn non_root_effects_filter_each_axis_and_chain_cut_is_lazy() {
        let mut context = Context {
            x_effect: true,
            y_effect: false,
            ..Default::default()
        };
        let result = ResolveOverscrollRouting(
            &mut context,
            OverscrollRoutingInput {
                residual_delta: Vector2dF::new(4.0, 5.0),
                accumulated_root_overscroll: Vector2dF::default(),
                caused_scroll_x: false,
                caused_scroll_y: false,
                is_root_scroller: false,
                scroll_chain_cut: true,
            },
        );
        assert_eq!(result.unused_scroll_delta, Vector2dF::new(4.0, 0.0));
        assert_eq!(
            context.events,
            ["feature", "feature", "x-effect", "y-effect"]
        );
    }
}
