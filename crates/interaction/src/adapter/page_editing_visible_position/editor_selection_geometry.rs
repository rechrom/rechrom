//! Native leaves for Editor selection commands, over the resident owners.
use super::*;
use interaction_default::core::editing::{
    commands::{editor_selection_commands as commands, move_commands},
    selection_modifier::{
        NoXPosForVerticalArrowNavigation, SelectionModifier, SelectionModifyAlteration,
        SelectionModifyVerticalDirection,
    },
    text_granularity::TextGranularity,
};
pub enum EditorSelectionGeometryQuery {
    Expand {
        selection: DOMSelection,
        granularity: TextGranularity,
    },
    Union {
        selection: DOMSelection,
        mark: DOMSelection,
    },
    Transpose(DOMSelection),
    Page {
        selection: DOMSelection,
        directional: bool,
        focused: Option<u64>,
        alteration: SelectionModifyAlteration,
        direction: SelectionModifyVerticalDirection,
    },
}
pub enum EditorSelectionGeometryResult {
    Selection(Option<DOMSelection>),
    Range(Option<dom::DOMStaticRange>),
}
fn Source(ctx: &EditingPositionContext<'_>, s: DOMSelection) -> VisibleSelection<u64> {
    let anchor = SourcePosition(EditingPositionWithAffinity {
        position: s.anchor,
        affinity: s.affinity,
    });
    let focus = SourcePosition(EditingPositionWithAffinity {
        position: s.focus,
        affinity: s.affinity,
    });
    visible_selection::CreateVisibleSelection(
        ctx,
        &anchor.position,
        &focus.position,
        anchor.affinity,
    )
}
fn Native(
    anchor: Position<u64>,
    focus: Position<u64>,
    affinity: TextAffinity,
) -> Option<DOMSelection> {
    let anchor = NativePosition(&anchor, affinity)?;
    let focus = NativePosition(&focus, affinity)?;
    Some(DOMSelection {
        anchor: anchor.position,
        focus: focus.position,
        affinity: anchor.affinity,
    })
}
pub(super) fn Query(
    document: &Document,
    layout: LayoutOutput<'_>,
    locale: &std::ffi::CStr,
    behavior: EditingBehaviorType,
    query: EditorSelectionGeometryQuery,
) -> EditorSelectionGeometryResult {
    use EditorSelectionGeometryQuery as Q;
    use EditorSelectionGeometryResult as R;
    let flat = EditingPositionContext::NewForSelectionModifier(document, layout, locale, behavior);
    let dom = flat.Dom();
    match query {
        Q::Expand {
            selection,
            granularity,
        } => R::Selection(
            commands::ExpandSelectionToGranularity(&dom, &Source(&dom, selection), granularity)
                .and_then(|(a, f)| Native(a, f, TextAffinity::Downstream)),
        ),
        Q::Union { selection, mark } => {
            let selection = Source(&dom, selection);
            let mark = Source(&dom, mark);
            let range = visible_selection::NormalizeSelectionRange(
                &dom,
                &selection.anchor,
                &selection.focus,
            )
            .zip(visible_selection::NormalizeSelectionRange(
                &dom,
                &mark.anchor,
                &mark.focus,
            ))
            .and_then(|(a, b)| commands::UnionEphemeralRanges(&dom, a, b));
            R::Selection(range.and_then(|(a, f)| Native(a, f, TextAffinity::Downstream)))
        }
        Q::Transpose(selection) => R::Range(
            commands::ComputeRangeForTranspose(&dom, &Source(&dom, selection)).and_then(
                |(a, f)| {
                    Some(dom::DOMStaticRange {
                        start: DOMBoundaryPoint {
                            node_id: a.ComputeContainerNode(&dom)?,
                            offset: a.ComputeOffsetInContainerNode(&dom),
                        },
                        end: DOMBoundaryPoint {
                            node_id: f.ComputeContainerNode(&dom)?,
                            offset: f.ComputeOffsetInContainerNode(&dom),
                        },
                    })
                },
            ),
        ),
        Q::Page {
            selection,
            directional,
            focused,
            alteration,
            direction,
        } => {
            let distance = focused
                .and_then(|id| {
                    layout
                        .Node(id)
                        .and_then(|node| node.EditingPageMovementFacts())
                        .map(|(overflow, padding, height)| {
                            move_commands::VerticalScrollDistance(
                                overflow,
                                dom.IsEditableNodeValue(&id),
                                padding,
                                height,
                            )
                        })
                })
                .unwrap_or(0);
            if distance == 0 {
                return R::Selection(None);
            }
            let (anchor, focus, affinity) =
                super::selection_geometry::FlatSelection(&flat, selection);
            let mut modifier = SelectionModifier::new(
                VisibleSelection {
                    anchor,
                    focus,
                    affinity,
                },
                NoXPosForVerticalArrowNavigation(),
            );
            modifier.SetSelectionIsDirectional(directional);
            if !modifier.ModifyWithPageGranularity(&flat, alteration, distance, direction) {
                return R::Selection(None);
            }
            let selection = modifier.Selection(&flat);
            R::Selection(Native(
                selection.anchor,
                selection.focus,
                selection.affinity,
            ))
        }
    }
}
