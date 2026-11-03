//! Chromium ScrollbarTheme thumb geometry projection from retained Scrollbar
//! position/proportion state.

#![allow(non_snake_case)]

use super::*;
use crate::internal::scrollbar::Scrollbar;

// Source: core/scroll/scrollbar_theme.cc:195-217 and
//         cc/input/scroll_utils.cc:59-73.
#[inline(never)]
pub(super) fn SetScrollbarThumb(
    axis: &mut ScrollbarPaintAxis,
    scrollbar: &Scrollbar,
    has_buttons: bool,
    minimum_thumb_length: f64,
) {
    let track_length = if axis.horizontal {
        axis.track_size.width
    } else {
        axis.track_size.height
    };
    let cross_size = if axis.horizontal {
        axis.track_size.height
    } else {
        axis.track_size.width
    };
    axis.has_buttons = has_buttons && track_length >= 2.0 * cross_size + minimum_thumb_length;
    let button_extent = if axis.has_buttons {
        2.0 * cross_size
    } else {
        0.0
    };
    let available = (track_length - button_extent).max(0.0);
    let total_size = scrollbar.TotalSize();
    let visible_size = scrollbar.VisibleSize();
    let ratio = if total_size > 0 {
        (visible_size as f64 / total_size as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    // Source: cc/input/scroll_utils.cc:59-73.
    let thumb_length = available.min(minimum_thumb_length.max((available * ratio).round()));
    let range = total_size - visible_size;
    let position = if range > 0 {
        (scrollbar.CurrentPos().max(0.0) as f64 / range as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let start = button_extent / 2.0 + (available - thumb_length) * position;
    axis.thumb_offset = axis.track_offset;
    axis.thumb_size = axis.track_size;
    if axis.horizontal {
        axis.thumb_offset.x += start;
        axis.thumb_size.width = thumb_length;
    } else {
        axis.thumb_offset.y += start;
        axis.thumb_size.height = thumb_length;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr::null_mut;

    #[test]
    fn retained_scrollbar_state_drives_thumb_geometry() {
        let mut scrollbar = Scrollbar::new(
            null_mut(),
            crate::internal::scrollbar_orientation::ScrollbarOrientation::kHorizontalScrollbar,
            null_mut(),
            false,
            false,
            Default::default(),
            foundation::EScrollbarWidth::kAuto,
            10,
            25.0,
        );
        scrollbar.SetProportion(50, 100);
        let mut axis = ScrollbarPaintAxis {
            horizontal: true,
            track_size: Size {
                width: 100.0,
                height: 10.0,
            },
            ..Default::default()
        };
        SetScrollbarThumb(&mut axis, &scrollbar, false, 10.0);
        assert_eq!(axis.thumb_offset.x, 25.0);
        assert_eq!(axis.thumb_size.width, 50.0);

        assert!(scrollbar.Enabled());
        scrollbar.DisplayItemClient().ValidateForCommittedPaint();
        scrollbar.SetEnabled(false);
        assert!(!scrollbar.Enabled());
        assert_eq!(
            scrollbar.DisplayItemClient().GetPaintInvalidationReason(),
            foundation::PaintInvalidationReason::kScrollControl
        );

        scrollbar.OffsetDidChange(50.0);
        SetScrollbarThumb(&mut axis, &scrollbar, false, 10.0);
        assert_eq!(axis.thumb_offset.x, 50.0);
    }
}
