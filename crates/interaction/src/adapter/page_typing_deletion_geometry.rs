//! Read-only stages of TypingCommand deletion borrow the current resident
//! geometry for one query. Original modifier state survives as an opaque source
//! owner; it contains positions, never Document/Layout borrows or copied trees.
use super::*;
use interaction_default::core::editing::{
    commands::{
        typing_command::TypingSelection,
        typing_deletion::{
            self as source, BackwardDeletionGeometry, DeletionGeometryAction,
            TypingDeletionGeometryContext,
        },
    },
    editing_behavior::EditingBehaviorType,
    frame_selection::SelectionValue,
    text_granularity::TextGranularity,
    visible_selection::VisibleSelection,
};
use std::ffi::CStr;

pub enum TypingDeletionGeometryQuery {
    BeginBackward {
        ending: DOMSelection,
        directional: bool,
        granularity: TextGranularity,
        kill_ring: bool,
    },
    NeedsEmptyRoot(BackwardDeletionGeometry<u64>),
    FinishBackward {
        geometry: BackwardDeletionGeometry<u64>,
        ending: DOMSelection,
        starting: DOMSelection,
        granularity: TextGranularity,
    },
    Forward {
        ending: DOMSelection,
        starting: DOMSelection,
        directional: bool,
        granularity: TextGranularity,
        kill_ring: bool,
    },
}
pub enum TypingDeletionGeometryResult {
    Backward(BackwardDeletionGeometry<u64>),
    NeedsEmptyRoot(bool),
    Action(DeletionGeometryAction<DOMSelection>),
}
#[derive(Clone, PartialEq)]
struct Selection(DOMSelection);
impl TypingSelection for Selection {
    fn IsCaret(&self) -> bool {
        self.0.anchor == self.0.focus
    }
}
impl SelectionValue for Selection {
    fn IsNone(&self) -> bool {
        self.0.anchor.node_id.is_none()
    }
}
struct Geometry<'a> {
    flat: EditingPositionContext<'a>,
    dom: EditingPositionContext<'a>,
}
fn Raw(selection: DOMSelection) -> VisibleSelection<u64> {
    let anchor = SourcePosition(EditingPositionWithAffinity {
        position: selection.anchor,
        affinity: selection.affinity,
    });
    let focus = SourcePosition(EditingPositionWithAffinity {
        position: selection.focus,
        affinity: selection.affinity,
    });
    VisibleSelection {
        anchor: anchor.position,
        focus: focus.position,
        affinity: anchor.affinity,
    }
}
impl<'a> TypingDeletionGeometryContext<Selection> for Geometry<'a> {
    type Geometry = EditingPositionContext<'a>;
    fn Geometry(&self) -> &Self::Geometry {
        &self.flat
    }
    fn DOMGeometry(&self) -> &Self::Geometry {
        &self.dom
    }
    fn RawDOMSelection(&self, selection: &Selection) -> VisibleSelection<u64> {
        Raw(selection.0)
    }
    fn ConvertRawDOMSelectionToFlat(&self, selection: &Selection) -> VisibleSelection<u64> {
        let (anchor, focus, affinity) = selection_geometry::FlatSelection(&self.flat, selection.0);
        VisibleSelection {
            anchor,
            focus,
            affinity,
        }
    }
    fn ComputeVisibleDOMSelection(&self, selection: &Selection) -> VisibleSelection<u64> {
        CreateVisibleDOMSelectionForFocus(
            self.dom.document,
            self.dom.layout.expect("resident deletion geometry"),
            Some(selection.0),
        )
        .map(Raw)
        .unwrap_or_default()
    }
    fn BuildDOMSelection(&self, selection: &VisibleSelection<u64>) -> Selection {
        Selection(
            NativeDOMSelection(&selection.anchor, &selection.focus, selection.affinity)
                .expect("nonnull source deletion selection"),
        )
    }
    fn EnclosingTableCell(&self, position: &Position<u64>) -> Option<u64> {
        use interaction_default::core::editing::selection_adjuster::WordGranularityContext;
        WordGranularityContext::EnclosingTableCell(&self.dom, position)
    }
    fn TableElementJustBefore(&self, position: &VisiblePosition<u64>) -> Option<u64> {
        use interaction_default::core::editing::selection_adjuster::WordGranularityContext;
        WordGranularityContext::TableElementJustBefore(&self.dom, position)
    }
    fn TableElementJustAfter(&self, position: &VisiblePosition<u64>) -> Option<u64> {
        let downstream = visible_units::MostForwardCaretPosition(
            &self.dom,
            position.DeepEquivalent(),
            Rule::CannotCross,
            SnapToClient::Editing,
        );
        let node = *downstream.AnchorNode()?;
        (PositionDOM::IsDisplayInsideTable(&self.dom, &node)
            && downstream.AtFirstEditingPositionForNode(&self.dom))
        .then_some(node)
    }
    fn CaretMinOffset(&self, node: &u64) -> usize {
        self.dom
            .Layout(node)
            .and_then(|node| node.EditingTextFacts(0, false))
            .map_or(0, |facts| facts.caret_min_offset)
    }
    fn AfterBlockIfBeforeAnonymousPlaceholder(
        &self,
        position: &Position<u64>,
    ) -> Option<Position<u64>> {
        source::AfterBlockIfBeforeAnonymousPlaceholder(&self.dom, position)
    }
}
impl source::AnonymousPlaceholderContext for EditingPositionContext<'_> {
    fn AnonymousPlaceholderFacts(
        &self,
        node: &u64,
    ) -> Option<source::AnonymousPlaceholderFacts<u64>> {
        let facts = self.Layout(node)?.EditingAnonymousPlaceholderFacts();
        Some(source::AnonymousPlaceholderFacts {
            is_br: facts.is_br,
            has_layout_sibling: facts.has_layout_sibling,
            parent_is_anonymous: facts.parent_is_anonymous,
            previous_parent_non_pseudo_node: facts.previous_parent_non_pseudo_node,
        })
    }
}
fn Action(action: DeletionGeometryAction<Selection>) -> DeletionGeometryAction<DOMSelection> {
    match action {
        DeletionGeometryAction::None => DeletionGeometryAction::None,
        DeletionGeometryAction::Select(selection) => DeletionGeometryAction::Select(selection.0),
        DeletionGeometryAction::Delete {
            selection,
            after_undo,
        } => DeletionGeometryAction::Delete {
            selection: selection.0,
            after_undo: after_undo.0,
        },
    }
}
pub(super) fn Query(
    document: &Document,
    layout: LayoutOutput<'_>,
    locale: &CStr,
    behavior: EditingBehaviorType,
    query: TypingDeletionGeometryQuery,
) -> TypingDeletionGeometryResult {
    let flat = EditingPositionContext::NewForSelectionModifier(document, layout, locale, behavior);
    let ctx = Geometry {
        flat,
        dom: flat.Dom(),
    };
    match query {
        TypingDeletionGeometryQuery::BeginBackward {
            ending,
            directional,
            granularity,
            kill_ring,
        } => TypingDeletionGeometryResult::Backward(source::BeginBackwardDeletionGeometry(
            &ctx,
            &Selection(ending),
            directional,
            granularity,
            kill_ring,
        )),
        TypingDeletionGeometryQuery::NeedsEmptyRoot(geometry) => {
            TypingDeletionGeometryResult::NeedsEmptyRoot(source::BackwardDeletionNeedsEmptyRoot(
                &ctx, &geometry,
            ))
        }
        TypingDeletionGeometryQuery::FinishBackward {
            geometry,
            ending,
            starting,
            granularity,
        } => TypingDeletionGeometryResult::Action(Action(source::FinishBackwardDeletionGeometry(
            &ctx,
            geometry,
            &Selection(ending),
            &Selection(starting),
            granularity,
        ))),
        TypingDeletionGeometryQuery::Forward {
            ending,
            starting,
            directional,
            granularity,
            kill_ring,
        } => TypingDeletionGeometryResult::Action(Action(source::ForwardDeletionGeometry(
            &ctx,
            &Selection(ending),
            &Selection(starting),
            directional,
            granularity,
            kill_ring,
        ))),
    }
}
