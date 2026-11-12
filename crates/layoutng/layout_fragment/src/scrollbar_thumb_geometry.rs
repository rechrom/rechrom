//! Chromium ScrollbarTheme thumb geometry projection from retained Scrollbar
//! position/proportion state.

#![allow(non_snake_case)]

use super::*;
use crate::internal::scrollbar::Scrollbar;

// Source: core/scroll/scrollbar_theme.cc:195-217 and
//         cc/input/scroll_utils.cc:59-73.
#[inline(never)]
pub(super) fn SetScrollbarThumb(axis: &mut ScrollbarPaintAxis, scrollbar: &Scrollbar) {
    // Read the same native theme geometry as Scrollbar input. In particular,
    // ThumbPosition preserves ScrollbarTheme's saturated/truncated position.
    let geometry = scrollbar.ThemeGeometryAt(scrollbar.CurrentPos());
    axis.has_buttons = geometry.has_buttons;
    axis.thumb_offset = Offset {
        x: geometry.thumb_rect.x() as f64,
        y: geometry.thumb_rect.y() as f64,
    };
    axis.thumb_size = Size {
        width: geometry.thumb_rect.width() as f64,
        height: geometry.thumb_rect.height() as f64,
    };
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
        scrollbar.SetFrameRect(&foundation::gfx::Rect::new(
            foundation::gfx::Point::new(0, 0),
            foundation::gfx::Size::new(100, 10),
        ));
        let mut axis = ScrollbarPaintAxis {
            horizontal: true,
            track_size: Size {
                width: 100.0,
                height: 10.0,
            },
            ..Default::default()
        };
        SetScrollbarThumb(&mut axis, &scrollbar);
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
        SetScrollbarThumb(&mut axis, &scrollbar);
        assert_eq!(axis.thumb_offset.x, 50.0);
    }
}
