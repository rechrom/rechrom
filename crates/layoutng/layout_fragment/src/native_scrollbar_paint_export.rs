//! Native scrollbar paint payload projection.
//!
//! Kept as a distinct fragment owner so scrollbar-heavy layout publication
//! has a direct package-qualified sampling frame.

#![allow(non_snake_case)]

use super::*;

// cpp: layoutng/internal/boundary/layout_boundary.cc:1138-1278
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
            && !unsafe { &*area }.HasVerticalScrollbar())
    {
        return;
    }
    let area = unsafe { &*area };
    let theme = layout_box.ScrollbarThemeForLayout();
    let themed_thickness =
        ScrollbarThemeThickness(theme, box_fragment.Style().UsedScrollbarWidth()) as f64;
    let borders = box_fragment.Borders();
    let reserved = box_fragment.Scrollbar();
    let border_top = Number(borders.top);
    let border_right = Number(borders.right);
    let border_bottom = Number(borders.bottom);
    let border_left = Number(borders.left);
    // The reserved strut is not a placement signal: scrollbar-gutter:
    // stable both-edges deliberately reserves both inline sides. Use Layout's
    // canonical writing-direction decision so an LTR both-edges gutter does
    // not move the actual vertical scrollbar and scroll corner to the left.
    let vertical_on_left = layout_box.ShouldPlaceVerticalScrollbarOnLeft();
    let vertical_thickness = if area.HasVerticalScrollbar() {
        if theme.uses_overlay_scrollbars {
            themed_thickness
        } else {
            Number(reserved.left).max(Number(reserved.right))
        }
    } else {
        0.0
    };
    let horizontal_thickness = if area.HasHorizontalScrollbar() {
        if theme.uses_overlay_scrollbars {
            themed_thickness
        } else {
            Number(reserved.bottom)
        }
    } else {
        0.0
    };
    if (area.HasVerticalScrollbar() && vertical_thickness <= 0.0)
        || (area.HasHorizontalScrollbar() && horizontal_thickness <= 0.0)
    {
        return;
    }

    let corner_client = area.ScrollCornerDisplayItemClient();
    let mut data = ScrollbarPaintData {
        corner_client_invalidation_reason: corner_client
            .map(|client| client.GetPaintInvalidationReason()),
        corner_client_id: corner_client.map_or(0, |client| client.Id() as u64),
        corner_client_is_cacheable: corner_client.is_some_and(|client| client.IsCacheable()),
        corner_client_is_just_created: corner_client
            .is_some_and(|client| client.IsJustCreated())
            .into(),
        uses_overlay_scrollbars: theme.uses_overlay_scrollbars,
        minimum_thumb_length: theme.minimum_thumb_length,
        track_color: theme.track_color,
        thumb_color: theme.thumb_color,
        button_color: theme.button_color,
        corner_color: theme.corner_color,
        ..ScrollbarPaintData::default()
    };
    let visible = area.VisibleContentRect(IncludeScrollbarsInRect::kExcludeScrollbars);
    let contents = area.ContentsSize();
    let minimum = area.MinimumScrollOffsetInt();
    let maximum = area.MaximumScrollOffsetInt();
    let current = area.GetScrollOffset();
    let corner_width = if !theme.uses_overlay_scrollbars && area.HasVerticalScrollbar() {
        vertical_thickness
    } else {
        0.0
    };
    let corner_height = if !theme.uses_overlay_scrollbars && area.HasHorizontalScrollbar() {
        horizontal_thickness
    } else {
        0.0
    };

    if area.HasHorizontalScrollbar() {
        let scrollbar = area.HorizontalScrollbar();
        let client = (!scrollbar.is_null()).then(|| unsafe { &*scrollbar }.DisplayItemClient());
        let mut axis = ScrollbarPaintAxis {
            compositor_element_id: foundation::CompositorElementIdFromUniqueObjectId(
                layout_box.UniqueId(),
                foundation::CompositorElementIdNamespace::kHorizontalScrollbar,
            ),
            display_item_client_id: client.map_or(0, |client| client.Id() as u64),
            display_item_client_invalidation_reason: client
                .map(|client| client.GetPaintInvalidationReason()),
            // Source: ScrollbarDisplayItem::NeedsUpdateDisplay delegates to
            // cc::Scrollbar. This native owner uses the same persistent
            // DisplayItemClient invalidation lifecycle as its update bit.
            needs_update_display: Some(client.is_none_or(|client| !client.IsValid())),
            display_item_client_is_cacheable: client.is_some_and(|client| client.IsCacheable()),
            display_item_client_is_just_created: client
                .is_some_and(|client| client.IsJustCreated())
                .into(),
            horizontal: true,
            left_side_vertical: false,
            track_offset: Offset {
                x: border_left
                    + if !theme.uses_overlay_scrollbars && vertical_on_left {
                        corner_width
                    } else {
                        0.0
                    },
                y: Number(box_fragment.Size().height) - border_bottom - horizontal_thickness,
            },
            track_size: Size {
                width: (Number(box_fragment.Size().width)
                    - border_left
                    - border_right
                    - corner_width)
                    .max(0.0),
                height: horizontal_thickness,
            },
            ..ScrollbarPaintAxis::default()
        };
        SetScrollbarThumb(
            &mut axis,
            visible.width() as f64,
            contents.width() as f64,
            current.x() as f64,
            minimum.x() as f64,
            maximum.x() as f64,
            theme.has_buttons,
            theme.minimum_thumb_length as f64,
        );
        data.horizontal = Some(axis);
    }
    if area.HasVerticalScrollbar() {
        let scrollbar = area.VerticalScrollbar();
        let client = (!scrollbar.is_null()).then(|| unsafe { &*scrollbar }.DisplayItemClient());
        let mut axis = ScrollbarPaintAxis {
            compositor_element_id: foundation::CompositorElementIdFromUniqueObjectId(
                layout_box.UniqueId(),
                foundation::CompositorElementIdNamespace::kVerticalScrollbar,
            ),
            display_item_client_id: client.map_or(0, |client| client.Id() as u64),
            display_item_client_invalidation_reason: client
                .map(|client| client.GetPaintInvalidationReason()),
            needs_update_display: Some(client.is_none_or(|client| !client.IsValid())),
            display_item_client_is_cacheable: client.is_some_and(|client| client.IsCacheable()),
            display_item_client_is_just_created: client
                .is_some_and(|client| client.IsJustCreated())
                .into(),
            horizontal: false,
            left_side_vertical: vertical_on_left,
            track_offset: Offset {
                x: if vertical_on_left {
                    border_left
                } else {
                    Number(box_fragment.Size().width) - border_right - vertical_thickness
                },
                y: border_top,
            },
            track_size: Size {
                width: vertical_thickness,
                height: (Number(box_fragment.Size().height)
                    - border_top
                    - border_bottom
                    - corner_height)
                    .max(0.0),
            },
            ..ScrollbarPaintAxis::default()
        };
        SetScrollbarThumb(
            &mut axis,
            visible.height() as f64,
            contents.height() as f64,
            current.y() as f64,
            minimum.y() as f64,
            maximum.y() as f64,
            theme.has_buttons,
            theme.minimum_thumb_length as f64,
        );
        data.vertical = Some(axis);
    }
    if corner_width > 0.0 && corner_height > 0.0 {
        data.corner_offset = Offset {
            x: if vertical_on_left {
                border_left
            } else {
                Number(box_fragment.Size().width) - border_right - corner_width
            },
            y: Number(box_fragment.Size().height) - border_bottom - corner_height,
        };
        data.corner_size = Size {
            width: corner_width,
            height: corner_height,
        };
    }
    paint.scrollbars = Some(Arc::new(data));
}
