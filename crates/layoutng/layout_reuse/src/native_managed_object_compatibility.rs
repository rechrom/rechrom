//! Reuse-owned admission for retaining an existing native LayoutObject.

use super::*;

impl LayoutObjectTreeStorage {
    #[inline(always)]
    pub(super) fn Compatible(
        old: &ManagedObject,
        parent: *mut LayoutObject,
        input: &NativeNodeConstructionData,
        effective: &ComputedStyle,
    ) -> bool {
        managed_object_compatible(old, parent, input, effective)
    }

    #[inline(always)]
    pub(super) fn StyleUpdateCompatible(
        old: &ManagedObject,
        parent: *mut LayoutObject,
        input: &NativeNodeConstructionData,
        effective: &ComputedStyle,
    ) -> bool {
        managed_style_update_compatible(old, parent, input, effective)
    }
}

#[inline(never)]
fn object_kind_compatible(
    old: &ManagedObject,
    parent: *mut LayoutObject,
    input: &NativeNodeConstructionData,
    effective: &ComputedStyle,
) -> bool {
    old.parent == parent
        && old.input.kind == input.kind
        && old.effective_style.display == effective.display
        && old
            .input
            .element
            .as_ref()
            .and_then(|element| element.form_control_type)
            == input
                .element
                .as_ref()
                .and_then(|element| element.form_control_type)
        && old.input.element.as_ref().map(|element| {
            (
                element.html_image,
                element.image_map_area,
                element.first_letter_pseudo,
                element.text_control_inner_editor,
                element.text_control_spin_button,
                element.select_uses_menu_list,
            )
        }) == input.element.as_ref().map(|element| {
            (
                element.html_image,
                element.image_map_area,
                element.first_letter_pseudo,
                element.text_control_inner_editor,
                element.text_control_spin_button,
                element.select_uses_menu_list,
            )
        })
        && old
            .effective_style
            .extended
            .as_ref()
            .map(|extended| extended.initial_letter)
            == effective
                .extended
                .as_ref()
                .map(|extended| extended.initial_letter)
        && old
            .effective_style
            .extended
            .as_ref()
            .map(|extended| &extended.timeline_trigger_names)
            == effective
                .extended
                .as_ref()
                .map(|extended| &extended.timeline_trigger_names)
}

#[inline(never)]
fn managed_object_compatible(
    old: &ManagedObject,
    parent: *mut LayoutObject,
    input: &NativeNodeConstructionData,
    effective: &ComputedStyle,
) -> bool {
    object_kind_compatible(old, parent, input, effective)
        && old.effective_style.position == effective.position
        && old.effective_style.floating == effective.floating
}

// cpp: ComputedStyle::NeedsReinsertLayoutTree; Element::RecalcOwnStyle.
// Position changes do not change the LayoutObject class. Blink updates the
// resident object through SetStyle, reinserting it only when it crosses the
// in-flow/out-of-flow boundary (or changes float participation in-flow).
#[inline(never)]
fn managed_style_update_compatible(
    old: &ManagedObject,
    parent: *mut LayoutObject,
    input: &NativeNodeConstructionData,
    effective: &ComputedStyle,
) -> bool {
    if !object_kind_compatible(old, parent, input, effective) {
        return false;
    }
    let old_native = unsafe { old.effective_style.native_style.as_ref() };
    let new_native = unsafe { effective.native_style.as_ref() };
    match (old_native, new_native) {
        (Some(old_native), Some(new_native)) => {
            !NativeComputedStyle::NeedsReinsertLayoutTree(old_native, new_native)
        }
        _ => {
            old.effective_style.position == effective.position
                && old.effective_style.floating == effective.floating
        }
    }
}
