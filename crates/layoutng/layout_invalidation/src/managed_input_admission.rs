//! Invalidation-owned admission of retained source input descriptors.

use foundation::Persistent;
use layoutng_style::style::computed_style::ComputedStyle as StyleComputedStyle;

use super::{ComputedStyle, ElementData, NodeConstructionData};
use crate::internal::{layout_box::LayoutBox, layout_object::LayoutObject};

// Chromium theme state, image-resource paint facts and scroll translations affect
// paint, not box geometry. Keep their new values in live metadata, excluding
// them only from the geometry comparison.
#[inline(never)]
pub(super) fn GeometryMetadataEqual(new: &Option<ElementData>, old: &Option<ElementData>) -> bool {
    if new == old {
        return true;
    }
    match (new, old) {
        (Some(new), Some(old)) => {
            let mut normalized = new.clone();
            normalized.control_hovered = old.control_hovered;
            normalized.control_active = old.control_active;
            normalized.control_focused = old.control_focused;
            normalized.image_resource_is_loaded = old.image_resource_is_loaded;
            normalized.image_resource_error = old.image_resource_error;
            normalized.image_resource_known_to_be_opaque = old.image_resource_known_to_be_opaque;
            normalized.image_resource_maybe_animated = old.image_resource_maybe_animated;
            normalized.focused_image_map_area = old.focused_image_map_area.clone();
            normalized.scroll_offset = old.scroll_offset;
            normalized == *old
        }
        _ => false,
    }
}

// Source: core/layout/layout_image.cc:139-153,187-193,282-290. Resource
// availability and decoded alpha can change without changing intrinsic
// dimensions. LayoutImage updates the live resource and invalidates its image
// display item; it does not turn those facts into a geometry change.
#[inline(always)]
pub(super) fn ImagePaintStateChanged(new: &Option<ElementData>, old: &Option<ElementData>) -> bool {
    match (new, old) {
        (Some(new), Some(old)) => {
            new.image_resource_is_loaded != old.image_resource_is_loaded
                || new.image_resource_error != old.image_resource_error
                || new.image_resource_known_to_be_opaque != old.image_resource_known_to_be_opaque
                || new.image_resource_maybe_animated != old.image_resource_maybe_animated
                || new.focused_image_map_area != old.focused_image_map_area
        }
        _ => false,
    }
}

#[inline(always)]
fn NaturalSizeChanged(new: &ElementData, old: &ElementData) -> bool {
    new.natural_width != old.natural_width
        || new.natural_height != old.natural_height
        || new.natural_aspect_ratio != old.natural_aspect_ratio
        || new.image_device_pixel_ratio != old.image_device_pixel_ratio
}

#[inline(always)]
pub(super) fn LayoutImageNaturalSizeChanged(
    object: *const LayoutObject,
    new: &Option<ElementData>,
    old: &Option<ElementData>,
) -> bool {
    let (Some(new), Some(old)) = (new, old) else {
        return false;
    };
    !object.is_null() && unsafe { &*object }.IsLayoutImage() && NaturalSizeChanged(new, old)
}

// Source: core/layout/layout_image.cc:208-221,224-239,271-279. A fixed-size,
// non-flex LayoutImage keeps its box geometry when decoded natural dimensions
// change. It still refreshes intrinsic-width state, paint properties and its
// image display item; callers perform those mutations after admission.
#[inline(always)]
pub(super) fn LayoutImageNaturalSizeChangeIsPaintOnly(
    object: *const LayoutObject,
    new: &Option<ElementData>,
    old: &Option<ElementData>,
    new_style: *const StyleComputedStyle,
) -> bool {
    let (Some(new), Some(old)) = (new, old) else {
        return false;
    };
    if object.is_null() || !unsafe { &*object }.IsLayoutImage() || !NaturalSizeChanged(new, old) {
        return false;
    }

    let mut normalized = new.clone();
    normalized.natural_width = old.natural_width;
    normalized.natural_height = old.natural_height;
    normalized.natural_aspect_ratio = old.natural_aspect_ratio;
    normalized.image_device_pixel_ratio = old.image_device_pixel_ratio;
    if !GeometryMetadataEqual(&Some(normalized), &Some(old.clone())) {
        return false;
    }

    let box_ = unsafe { &*object.cast::<LayoutBox>() };
    if box_.IsFlexItem() {
        return false;
    }
    // Complete tree updates can change style and decoded dimensions in the
    // same transaction. Chromium evaluates NeedsLayoutOnNaturalSizeChange()
    // against the newly installed style, not the retained old one.
    let style = if new_style.is_null() {
        unsafe { &*object }.StyleRef()
    } else {
        unsafe { &*new_style }
    };
    style.LogicalWidth().IsFixed()
        && style.LogicalHeight().IsFixed()
        && (style.LogicalMinWidth().IsFixed() || style.LogicalMinWidth().IsAuto())
        && (style.LogicalMaxWidth().IsFixed() || style.LogicalMaxWidth().IsNone())
}

// Chromium form-control interaction state changes theme painting without changing
// box geometry. LayoutObject::SetStyle normally turns the corresponding visual
// diff into a full paint invalidation; the paint-only metadata path must make
// the same DisplayItemClient transition before replacing its retained input.
#[inline(always)]
pub(super) fn ControlPaintStateChanged(
    new: &Option<ElementData>,
    old: &Option<ElementData>,
) -> bool {
    match (new, old) {
        (Some(new), Some(old)) => {
            new.control_hovered != old.control_hovered
                || new.control_active != old.control_active
                || new.control_focused != old.control_focused
        }
        _ => false,
    }
}

// Compare attachment metadata without constructing another owned style. Chromium
// ComputedStyle differences are checked independently by the caller, as in
// LayoutObject::SetStyle. Keep this destructuring exhaustive when inputs grow.
#[inline(never)]
pub(super) fn NonStyleInputsEqual(
    new: &NodeConstructionData,
    old: &NodeConstructionData,
    natural_size_change_is_paint_only: bool,
) -> bool {
    let NodeConstructionData {
        id,
        kind,
        style_generated,
        computed_style: _,
        style: _,
        resources: _,
        text,
        element,
        debug_name,
        first_line_computed_style: _,
        first_line_style,
        first_line_resources: _,
    } = new;
    *id == old.id
        && *kind == old.kind
        && *style_generated == old.style_generated
        && *text == old.text
        && *debug_name == old.debug_name
        && *first_line_style == old.first_line_style
        && (GeometryMetadataEqual(element, &old.element) || natural_size_change_is_paint_only)
}

#[inline(never)]
pub(super) fn RetainInputStyles(
    input: &NodeConstructionData,
    effective_computed_style: *const StyleComputedStyle,
) -> Vec<Persistent<StyleComputedStyle>> {
    let mut roots: Vec<Persistent<StyleComputedStyle>> = Vec::new();
    for style in [
        input.computed_style,
        effective_computed_style,
        input.first_line_computed_style,
    ] {
        if !style.is_null() && !roots.iter().any(|root| root.Get().cast_const() == style) {
            roots.push(Persistent::from_ptr(style.cast_mut()));
        }
    }
    roots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_resource_paint_facts_do_not_change_geometry() {
        let old = Some(ElementData::default());
        let mut loaded = ElementData::default();
        loaded.image_resource_is_loaded = true;
        loaded.image_resource_error = false;
        loaded.image_resource_known_to_be_opaque = true;
        loaded.image_resource_maybe_animated = true;
        let loaded = Some(loaded);

        assert!(GeometryMetadataEqual(&loaded, &old));
        assert!(ImagePaintStateChanged(&loaded, &old));

        let mut resized = loaded.clone().unwrap();
        resized.natural_width = Some(40.0);
        assert!(!GeometryMetadataEqual(&Some(resized), &old));
    }
}
