//! Read-only EditorCommand target ranges over the resident selection geometry.
//! Chromium: editor_command.cc:221-259; editing_utilities.cc:1750-1764.
use super::*;
use interaction_default::core::editing::{
    commands::{
        editor_command_input_type::SelectionModifyDirection,
        editor_command_target_ranges::ComputeTargetRangeSelection,
    },
    text_granularity::TextGranularity,
};
use std::ffi::CStr;

pub(super) fn Query(
    document: &Document,
    layout: LayoutOutput<'_>,
    locale: &CStr,
    behavior: EditingBehaviorType,
    selection: Option<DOMSelection>,
    directional: bool,
    extend: Option<(SelectionModifyDirection, TextGranularity)>,
) -> Option<dom::DOMStaticRange> {
    let selection = selection?;
    let flat = EditingPositionContext::NewForSelectionModifier(document, layout, locale, behavior);
    let dom = flat.Dom();
    let selection = if let Some((direction, granularity)) = extend {
        let (anchor, focus, affinity) = selection_geometry::FlatSelection(&flat, selection);
        ComputeTargetRangeSelection(
            &flat,
            &dom,
            VisibleSelection {
                anchor,
                focus,
                affinity,
            },
            directional,
            direction,
            granularity,
        )?
    } else {
        let anchor = SourcePosition(EditingPositionWithAffinity {
            position: selection.anchor,
            affinity: selection.affinity,
        });
        let focus = SourcePosition(EditingPositionWithAffinity {
            position: selection.focus,
            affinity: selection.affinity,
        });
        visible_selection::CreateVisibleSelection(
            &dom,
            &anchor.position,
            &focus.position,
            anchor.affinity,
        )
    };
    // Selection::ComputeRange orders anchor/focus; Range's container offsets
    // preserve UTF-16 CharacterData offsets and DOM child offsets separately.
    let (start, end) = match dom.ComparePositions(&selection.anchor, &selection.focus)? {
        Ordering::Greater => (&selection.focus, &selection.anchor),
        _ => (&selection.anchor, &selection.focus),
    };
    Some(dom::DOMStaticRange {
        start: DOMBoundaryPoint {
            node_id: start.ComputeContainerNode(&dom)?,
            offset: start.ComputeOffsetInContainerNode(&dom),
        },
        end: DOMBoundaryPoint {
            node_id: end.ComputeContainerNode(&dom)?,
            offset: end.ComputeOffsetInContainerNode(&dom),
        },
    })
}
