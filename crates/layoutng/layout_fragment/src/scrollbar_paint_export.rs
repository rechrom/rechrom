//! Chromium scrollbar paint payload projection.
//!
//! Kept as a distinct fragment owner so scrollbar-heavy layout publication
//! has a direct package-qualified sampling frame.

#![allow(non_snake_case)]

use super::*;
use crate::internal::layout_scrollable_area::ResizerHitTestType;
use crate::internal::scrollbar_orientation::ScrollbarOrientation;

// Package-boundary projection of the source overflow-control state from
// core/paint/paint_layer_scrollable_area.cc:2141-2184,2283-2308 and the
// scrollbar identities from core/scroll/scrollbar.{h,cc}. The projection is
// consumed by Paint without recreating Blink's in-process LayoutObject graph.
#[inline(never)]
pub(super) fn ExportScrollbarPaintData(
    box_fragment: &PhysicalBoxFragment,
    paint: &mut PaintProperties,
) {
    let layout_box = DynamicTo::<LayoutBox>(box_fragment.GetLayoutObject());
    if layout_box.is_null() {
        return;
    }
    let layout_box = unsafe { &*layout_box };
    let area = layout_box.GetScrollableArea();
    if area.is_null()
        || (!unsafe { &*area }.HasHorizontalScrollbar()
            && !unsafe { &*area }.HasVerticalScrollbar()
            && !layout_box.CanResize())
    {
        return;
    }
    // Source: paint_property_tree_builder.cc:4172-4175. Position the source
    // controls before exporting their immutable paint payload.
    unsafe { &mut *area }.PositionOverflowControlsForPrePaint();
    let area = unsafe { &*area };
    let theme = layout_box.ScrollbarThemeForLayout();
    let uses_overlay_scrollbars = area.HasOverlayScrollbars();
    let should_paint_as_overlay = area.ShouldOverflowControlsPaintAsOverlay();
    let vertical_on_left = area.ShouldPlaceVerticalScrollbarOnLeft();

    let corner_client = area.ScrollCornerDisplayItemClient();
    let mut data = ScrollbarPaintData {
        corner_client_invalidation_reason: corner_client
            .map(|client| client.GetPaintInvalidationReason()),
        corner_client_id: corner_client.map_or(0, |client| client.Id() as u64),
        corner_client_is_cacheable: corner_client.is_some_and(|client| client.IsCacheable()),
        corner_client_is_just_created: corner_client
            .is_some_and(|client| client.IsJustCreated())
            .into(),
        should_paint_as_overlay,
        uses_overlay_scrollbars,
        is_solid_color: theme.is_solid_color,
        uses_macos_painted_scrollbars: theme.uses_macos_painted_scrollbars,
        theme_disallows_hit_test: !theme.allows_hit_test,
        minimum_thumb_length: theme.minimum_thumb_length,
        resizer_on_left: vertical_on_left,
        resizer_paint_scale: area.ScaleFromDIP(),
        needs_scroll_corner: area.NeedsScrollCorner(),
        track_color: theme.track_color,
        thumb_color: theme.thumb_color,
        button_color: theme.button_color,
        corner_color: theme.corner_color,
        ..ScrollbarPaintData::default()
    };
    let corner_rect = area.ScrollCornerRect();
    let resizer_pointer_rect = area.ResizerCornerRect(ResizerHitTestType::kResizerForPointer);
    let resizer_touch_rect = area.ResizerCornerRect(ResizerHitTestType::kResizerForTouch);

    if area.HasHorizontalScrollbar() {
        let scrollbar = area.HorizontalScrollbar();
        let frame = unsafe { &*scrollbar }.FrameRect();
        let client = (!scrollbar.is_null()).then(|| unsafe { &*scrollbar }.DisplayItemClient());
        let mut axis = ScrollbarPaintAxis {
            compositor_element_id: foundation::CompositorElementIdFromUniqueObjectId(
                layout_box.UniqueId(),
                foundation::CompositorElementIdNamespace::kHorizontalScrollbar,
            ),
            display_item_client_id: client.map_or(0, |client| client.Id() as u64),
            display_item_client_invalidation_reason: client
                .map(|client| client.GetPaintInvalidationReason()),
            // ScrollbarDisplayItem::NeedsUpdateDisplay is cc::Scrollbar state;
            // DisplayItemClient validity is a different lifecycle signal.  The
            // source Scrollbar subset does not export that state yet, so leave
            // it explicitly unavailable.  Paint's consumer conservatively
            // treats None as needing an update.
            needs_update_display: None,
            display_item_client_is_cacheable: client.is_some_and(|client| client.IsCacheable()),
            display_item_client_is_just_created: client
                .is_some_and(|client| client.IsJustCreated())
                .into(),
            horizontal: true,
            left_side_vertical: false,
            painted_opacity: area
                .ScrollbarPaintedOpacity(ScrollbarOrientation::kHorizontalScrollbar),
            painted_thumb_width: area.ScrollbarPaintedThumbWidth(unsafe { &*scrollbar }),
            track_offset: Offset {
                x: frame.x() as f64,
                y: frame.y() as f64,
            },
            track_size: Size {
                width: frame.width().max(0) as f64,
                height: frame.height().max(0) as f64,
            },
            ..ScrollbarPaintAxis::default()
        };
        SetScrollbarThumb(&mut axis, unsafe { &*scrollbar });
        data.horizontal = Some(axis);
    }
    if area.HasVerticalScrollbar() {
        let scrollbar = area.VerticalScrollbar();
        let frame = unsafe { &*scrollbar }.FrameRect();
        let client = (!scrollbar.is_null()).then(|| unsafe { &*scrollbar }.DisplayItemClient());
        let mut axis = ScrollbarPaintAxis {
            compositor_element_id: foundation::CompositorElementIdFromUniqueObjectId(
                layout_box.UniqueId(),
                foundation::CompositorElementIdNamespace::kVerticalScrollbar,
            ),
            display_item_client_id: client.map_or(0, |client| client.Id() as u64),
            display_item_client_invalidation_reason: client
                .map(|client| client.GetPaintInvalidationReason()),
            // See the horizontal branch above. Do not reconstruct cc scrollbar
            // update state from DisplayItemClient validity.
            needs_update_display: None,
            display_item_client_is_cacheable: client.is_some_and(|client| client.IsCacheable()),
            display_item_client_is_just_created: client
                .is_some_and(|client| client.IsJustCreated())
                .into(),
            horizontal: false,
            left_side_vertical: vertical_on_left,
            painted_opacity: area.ScrollbarPaintedOpacity(ScrollbarOrientation::kVerticalScrollbar),
            painted_thumb_width: area.ScrollbarPaintedThumbWidth(unsafe { &*scrollbar }),
            track_offset: Offset {
                x: frame.x() as f64,
                y: frame.y() as f64,
            },
            track_size: Size {
                width: frame.width().max(0) as f64,
                height: frame.height().max(0) as f64,
            },
            ..ScrollbarPaintAxis::default()
        };
        SetScrollbarThumb(&mut axis, unsafe { &*scrollbar });
        data.vertical = Some(axis);
    }
    if !corner_rect.IsEmpty() {
        data.corner_offset = Offset {
            x: corner_rect.x() as f64,
            y: corner_rect.y() as f64,
        };
        data.corner_size = Size {
            width: corner_rect.width() as f64,
            height: corner_rect.height() as f64,
        };
    }
    if !resizer_touch_rect.IsEmpty() {
        data.resizer_touch_offset = Offset {
            x: resizer_touch_rect.x() as f64,
            y: resizer_touch_rect.y() as f64,
        };
        data.resizer_touch_size = Size {
            width: resizer_touch_rect.width() as f64,
            height: resizer_touch_rect.height() as f64,
        };
    }
    if !resizer_pointer_rect.IsEmpty() {
        data.resizer_pointer_offset = Offset {
            x: resizer_pointer_rect.x() as f64,
            y: resizer_pointer_rect.y() as f64,
        };
        data.resizer_pointer_size = Size {
            width: resizer_pointer_rect.width() as f64,
            height: resizer_pointer_rect.height() as f64,
        };
    }
    paint.scrollbars = Some(Arc::new(data));
}
