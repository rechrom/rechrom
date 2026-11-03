//! Reuse-owned admission for retaining an existing source LayoutObject.

use super::*;

impl LayoutObjectTreeStorage {
    #[inline(always)]
    pub(super) fn Compatible(
        old: &ManagedObject,
        parent: *mut LayoutObject,
        input: &NodeConstructionData,
        effective: &ComputedStyle,
    ) -> bool {
        managed_object_compatible(old, parent, input, effective)
    }

    #[inline(always)]
    pub(super) fn StyleUpdateCompatible(
        old: &ManagedObject,
        parent: *mut LayoutObject,
        input: &NodeConstructionData,
        effective: &ComputedStyle,
    ) -> bool {
        managed_style_update_compatible(old, parent, input, effective)
    }
}

#[inline(never)]
fn object_kind_compatible(
    old: &ManagedObject,
    parent: *mut LayoutObject,
    input: &NodeConstructionData,
    effective: &ComputedStyle,
) -> bool {
    let canonical_kind_compatible =
        match (unsafe { old.effective_computed_style.as_ref() }, unsafe {
            effective.style.as_ref()
        }) {
            (Some(old_style), Some(new_style)) => {
                let old_animations = unsafe { old_style.Animations().Get().as_ref() };
                let new_animations = unsafe { new_style.Animations().Get().as_ref() };
                old_style.Display() == new_style.Display()
                    && old_style.InitialLetter() == new_style.InitialLetter()
                    && match (old_animations, new_animations) {
                        (Some(old), Some(new)) => old.TimelineTriggerNamesMatch(new),
                        (None, None) => true,
                        (Some(old), None) | (None, Some(old)) => old
                            .TimelineTriggerNameList()
                            .iter()
                            .all(|name| name.Get().is_null()),
                    }
            }
            _ => {
                old.effective_style.display == effective.display
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
        };
    old.parent == parent
        && old.input.kind == input.kind
        && canonical_kind_compatible
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
}

#[inline(never)]
fn managed_object_compatible(
    old: &ManagedObject,
    parent: *mut LayoutObject,
    input: &NodeConstructionData,
    effective: &ComputedStyle,
) -> bool {
    if !object_kind_compatible(old, parent, input, effective) {
        return false;
    }
    match (unsafe { old.effective_computed_style.as_ref() }, unsafe {
        effective.style.as_ref()
    }) {
        (Some(old), Some(new)) => {
            old.GetPosition() == new.GetPosition() && old.Floating() == new.Floating()
        }
        _ => {
            old.effective_style.position == effective.position
                && old.effective_style.floating == effective.floating
        }
    }
}

// cpp: ComputedStyle::NeedsReinsertLayoutTree; Element::RecalcOwnStyle.
// Position changes do not change the LayoutObject class. Blink updates the
// resident object through SetStyle, reinserting it only when it crosses the
// in-flow/out-of-flow boundary (or changes float participation in-flow).
#[inline(never)]
fn managed_style_update_compatible(
    old: &ManagedObject,
    parent: *mut LayoutObject,
    input: &NodeConstructionData,
    effective: &ComputedStyle,
) -> bool {
    if !object_kind_compatible(old, parent, input, effective) {
        return false;
    }
    let old_style = unsafe { old.effective_computed_style.as_ref() };
    let new_style = unsafe { effective.style.as_ref() };
    match (old_style, new_style) {
        (Some(old_style), Some(new_style)) => {
            !StyleComputedStyle::NeedsReinsertLayoutTree(old_style, new_style)
        }
        _ => {
            old.effective_style.position == effective.position
                && old.effective_style.floating == effective.floating
        }
    }
}
