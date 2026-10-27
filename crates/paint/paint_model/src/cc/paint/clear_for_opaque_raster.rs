// Translation status: complete
// Copyright 2020 The Chromium Authors. BSD-style license in LICENSE.
// Source: cc/paint/clear_for_opaque_raster.h:1-35;
//         cc/paint/clear_for_opaque_raster.cc:1-15,82-83.
use foundation::gfx::{Point, Rect, Size, Vector2dF};
// Source: cc/paint/clear_for_opaque_raster.h:18-31;
//         cc/paint/clear_for_opaque_raster.cc:16-80.
#[allow(non_snake_case)]
pub fn CalculateClearForOpaqueRasterRects(
    translation: &Vector2dF,
    scale: &Vector2dF,
    content_size: &Size,
    canvas_bitmap_rect: &Rect,
    canvas_playback_rect: &Rect,
    outer_rect: &mut Rect,
    inner_rect: &mut Rect,
) -> bool {
    debug_assert!(translation.x() >= 0.0);
    debug_assert!(translation.y() >= 0.0);
    debug_assert!(translation.x() < 1.0);
    debug_assert!(translation.y() < 1.0);
    let left_opaque = translation.x() == 0.0;
    let top_opaque = translation.y() == 0.0;
    let right_opaque = scale.x() == 1.0;
    let bottom_opaque = scale.y() == 1.0;
    if left_opaque && top_opaque && right_opaque && bottom_opaque {
        return false;
    }
    *outer_rect = Rect::new(Point::default(), *content_size);
    *inner_rect = *outer_rect;
    outer_rect.InsetTLBR(
        0,
        0,
        if bottom_opaque { 0 } else { -1 },
        if right_opaque { 0 } else { -1 },
    );
    inner_rect.InsetTLBR(
        if top_opaque { 0 } else { 1 },
        if left_opaque { 0 } else { 1 },
        if bottom_opaque { 0 } else { 1 },
        if right_opaque { 0 } else { 1 },
    );
    let touches_left_edge = !left_opaque && canvas_playback_rect.x() == 0;
    let touches_top_edge = !top_opaque && canvas_playback_rect.y() == 0;
    let touches_right_edge = !right_opaque && content_size.width() == canvas_playback_rect.right();
    let touches_bottom_edge =
        !bottom_opaque && content_size.height() == canvas_playback_rect.bottom();
    let mut adjusted_playback_rect = *canvas_playback_rect;
    adjusted_playback_rect.InsetTLBR(
        if touches_top_edge { -1 } else { 0 },
        if touches_left_edge { -1 } else { 0 },
        if touches_bottom_edge { -1 } else { 0 },
        if touches_right_edge { -1 } else { 0 },
    );
    if inner_rect.Contains(adjusted_playback_rect) {
        return false;
    }
    if !outer_rect.Intersects(adjusted_playback_rect) {
        return false;
    }
    outer_rect.Intersect(adjusted_playback_rect);
    inner_rect.Intersect(adjusted_playback_rect);
    outer_rect.Offset(-canvas_bitmap_rect.x(), -canvas_bitmap_rect.y());
    inner_rect.Offset(-canvas_bitmap_rect.x(), -canvas_bitmap_rect.y());
    inner_rect != outer_rect
}
