//! Invalidation-owned admission of retained native input descriptors.

use foundation::Persistent;
use layoutng_style::style::computed_style::ComputedStyle as NativeComputedStyle;

use super::{ComputedStyle, ElementData, NativeNodeConstructionData};

// Native theme state and scroll translations affect paint, not box geometry.
// Keep their new values in live metadata, excluding them only from comparison.
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
            normalized.scroll_offset = old.scroll_offset;
            normalized == *old
        }
        _ => false,
    }
}

// Native form-control interaction state changes theme painting without changing
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

// Compare attachment metadata without constructing another owned style. Native
// ComputedStyle differences are checked independently by the caller, as in
// LayoutObject::SetStyle. Keep this destructuring exhaustive when inputs grow.
#[inline(never)]
pub(super) fn NonStyleInputsEqual(
    new: &NativeNodeConstructionData,
    old: &NativeNodeConstructionData,
) -> bool {
    let NativeNodeConstructionData {
        id,
        kind,
        style_generated,
        style: _,
        text,
        element,
        debug_name,
        first_line_style,
    } = new;
    *id == old.id
        && *kind == old.kind
        && *style_generated == old.style_generated
        && *text == old.text
        && *debug_name == old.debug_name
        && *first_line_style == old.first_line_style
        && GeometryMetadataEqual(element, &old.element)
}

#[inline(never)]
pub(super) fn RetainInputStyles(
    input: &NativeNodeConstructionData,
    effective: &ComputedStyle,
) -> Vec<Persistent<NativeComputedStyle>> {
    let mut roots: Vec<Persistent<NativeComputedStyle>> = Vec::new();
    for style in [
        input.style.native_style,
        effective.native_style,
        input
            .first_line_style
            .as_ref()
            .map_or(std::ptr::null(), |style| style.native_style),
    ] {
        if !style.is_null() && !roots.iter().any(|root| root.Get().cast_const() == style) {
            roots.push(Persistent::from_ptr(style.cast_mut()));
        }
    }
    roots
}
