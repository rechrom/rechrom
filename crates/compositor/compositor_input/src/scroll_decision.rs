//! Compositor-thread scroll consumption and clamping policy.

use compositor_tree::input_geometry::{PointF, Vector2dF};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollByDecision {
    pub adjusted_delta: Vector2dF,
    pub new_offset: PointF,
    pub unused_delta: Vector2dF,
    pub horizontal_axis_filtered: bool,
    pub vertical_axis_filtered: bool,
    pub clamped: bool,
}

pub fn DecideScrollBy(
    current_offset: PointF,
    requested_delta: Vector2dF,
    max_scroll_offset: PointF,
    user_scrollable_horizontal: bool,
    user_scrollable_vertical: bool,
) -> ScrollByDecision {
    let mut adjusted_delta = requested_delta;
    if !user_scrollable_horizontal {
        adjusted_delta.set_x(0.0);
    }
    if !user_scrollable_vertical {
        adjusted_delta.set_y(0.0);
    }
    let unclamped_offset = current_offset + adjusted_delta;
    let mut new_offset = unclamped_offset;
    new_offset.SetToMin(max_scroll_offset);
    new_offset.SetToMax(PointF::default());
    ScrollByDecision {
        adjusted_delta,
        new_offset,
        unused_delta: current_offset + requested_delta - new_offset,
        horizontal_axis_filtered: !user_scrollable_horizontal && requested_delta.x() != 0.0,
        vertical_axis_filtered: !user_scrollable_vertical && requested_delta.y() != 0.0,
        clamped: new_offset != unclamped_offset,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_locked_axis_but_reports_requested_delta_unused() {
        let decision = DecideScrollBy(
            PointF::new(10.0, 10.0),
            Vector2dF::new(5.0, 7.0),
            PointF::new(100.0, 100.0),
            false,
            true,
        );
        assert_eq!(decision.adjusted_delta, Vector2dF::new(0.0, 7.0));
        assert_eq!(decision.new_offset, PointF::new(10.0, 17.0));
        assert_eq!(decision.unused_delta, Vector2dF::new(5.0, 0.0));
        assert!(decision.horizontal_axis_filtered);
        assert!(!decision.vertical_axis_filtered);
    }

    #[test]
    fn clamps_at_both_scroll_limits() {
        let upper = DecideScrollBy(
            PointF::new(8.0, 9.0),
            Vector2dF::new(5.0, 5.0),
            PointF::new(10.0, 10.0),
            true,
            true,
        );
        assert_eq!(upper.new_offset, PointF::new(10.0, 10.0));
        assert_eq!(upper.unused_delta, Vector2dF::new(3.0, 4.0));
        assert!(upper.clamped);

        let lower = DecideScrollBy(
            PointF::new(2.0, 3.0),
            Vector2dF::new(-5.0, -7.0),
            PointF::new(10.0, 10.0),
            true,
            true,
        );
        assert_eq!(lower.new_offset, PointF::default());
        assert_eq!(lower.unused_delta, Vector2dF::new(-3.0, -4.0));
        assert!(lower.clamped);
    }
}
