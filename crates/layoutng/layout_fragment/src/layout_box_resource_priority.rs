#![allow(non_snake_case)]

use super::*;

// cpp: core/layout/layout_box.cc:2231-2263
pub(super) fn LayoutBoxComputeResourcePriorityProvider(box_: &LayoutBox) -> ResourcePriority {
    box_.CheckIsNotDestroyed();
    let view = box_.View();
    assert!(!view.is_null());
    let view_bounds = unsafe { &*view }.ViewRect();
    let mut object_bounds = box_.PhysicalContentBoxRect();
    let object: &LayoutObject = box_;
    object_bounds.Move(&object.LocalToAbsolutePointPhysical(
        &PhysicalOffset::default(),
        MapCoordinatesMode::kIgnoreTransforms.into(),
    ));

    let is_visible = if !object_bounds.IsEmpty() {
        view_bounds.IntersectsRect(&object_bounds)
    } else {
        view_bounds.ContainsRect(&object_bounds)
    };

    let mut screen_rect = PhysicalRect::default();
    if !object_bounds.IsEmpty() {
        screen_rect = view_bounds;
        screen_rect.Intersect(&object_bounds);
    }
    let screen_area = if !screen_rect.IsEmpty() && is_visible {
        (screen_rect.Width() * screen_rect.Height()).ToInt()
    } else {
        0
    };
    ResourcePriority::new(
        if is_visible {
            VisibilityStatus::kVisible
        } else {
            VisibilityStatus::kNotVisible
        },
        screen_area,
    )
}
