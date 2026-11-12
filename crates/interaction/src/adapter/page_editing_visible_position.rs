//! Call-scoped adapters for backward character movement. The translated
//! VisibleUnits/VisiblePosition owners traverse the live DOM and resident
//! LayoutObjects; this module retains neither positions nor a projected tree.
use super::page_editing_context::PreparedEditingGeometry;
use dom::{DOMBoundaryPoint, DOMFormControlClass, DOMNamespace, DOMNodeType, Document};
use foundation::{
    editable::{self, EditableFeatures},
    LayoutUnit, TextDirection, WritingMode,
};
use interaction_default::core::editing::{
    compare_positions,
    editing_behavior::EditingBehaviorType,
    editing_boundaries::{self, EditableBoundaryContext, EditingPositionRootContext},
    inline_box_position::{InlineAdjustmentContext, InlineAdjustmentFacts},
    position::{
        Position, PositionAnchorType, PositionDOM, PositionTree, PositionWithAffinity, TextAffinity,
    },
    position_conversion::{self, PositionTreeScope},
    position_iterator::PositionIteratorContext,
    position_units_sentence::{self, SentenceTrailingSpaceBehavior},
    position_units_word::{self, PlatformWordBehavior},
    selection_modifier::{SelectionModifierContext, VisualCaretMoveResult},
    selection_modifier_line::{self, LineMovementContext, LinePositionResult},
    state_machines::{NextGraphemeBoundary, PreviousGraphemeBoundary},
    text_offset_mapping::{self, TextOffsetMappingContext},
    visible_navigation,
    visible_position::{DownstreamVisiblePositionContext, VisiblePosition, VisiblePositionContext},
    visible_selection::{self, VisibleSelection},
    visible_units::{
        self, AssociatedCaretFacts, CaretObjectFacts, EditingBoundaryCrossingRule as Rule,
        SnapToClient, TextCaretFacts, VisibleUnitsContext,
    },
    visible_units_line::{self, StartOfLineContext, VisibleLineContext},
};
use layoutng_assembly::{
    editing_state::DOMSelection,
    layout_engine::{EditingInlineContentsBlockIdentity, LayoutNode, LayoutOutput},
    EditingPosition, EditingPositionAnchorType, EditingPositionWithAffinity, EditingTextAffinity,
};
use std::cmp::Ordering;

#[path = "page_nonlocated_context_menu_geometry.rs"]
mod nonlocated_context_menu_geometry;
pub use nonlocated_context_menu_geometry::{
    NonLocatedGeometryQuery, NonLocatedGeometryResult, QueryNonLocatedGeometry,
};
#[path = "page_clipboard_geometry.rs"]
mod clipboard_geometry;
#[path = "page_markup_geometry.rs"]
mod markup_geometry;
pub use markup_geometry::{
    SerializeMarkupBody, SerializeMarkupRange, SerializeMarkupRangeWithAncestor,
};
#[path = "page_apply_style_geometry.rs"]
pub(super) mod apply_style_geometry;
#[path = "page_composition_text.rs"]
mod composition_text;
#[path = "page_editor_selection_geometry.rs"]
mod editor_selection_geometry;
#[path = "page_editor_target_range_geometry.rs"]
mod editor_target_range_geometry;
pub use editor_selection_geometry::{EditorSelectionGeometryQuery, EditorSelectionGeometryResult};
#[path = "page_mouse_selection_geometry.rs"]
mod mouse_selection_geometry;
#[path = "page_replace_selection_geometry.rs"]
mod replace_selection_geometry;
#[path = "page_selection_geometry.rs"]
mod selection_geometry;
#[path = "page_typing_deletion_geometry.rs"]
pub(super) mod typing_deletion_geometry;
pub use composition_text::{
    CompositionMarkerOwner, CompositionMarkerUpdate, CompositionTextQuery, CompositionTextResult,
    QueryCompositionText,
};
pub use mouse_selection_geometry::{
    CaretSelectionForContentsHit, MouseSelectionCursor, MouseSelectionGeometryQuery,
    MouseSelectionGeometryResult,
};
pub use selection_geometry::{SelectionGeometryQuery, SelectionGeometryResult};
pub use typing_deletion_geometry::{TypingDeletionGeometryQuery, TypingDeletionGeometryResult};

#[derive(Clone, Copy)]
pub(super) struct EditingPositionContext<'a> {
    document: &'a Document,
    layout: Option<LayoutOutput<'a>>,
    flat: bool,
    ui_locale: Option<&'a std::ffi::CStr>,
    behavior: Option<EditingBehaviorType>,
}
impl<'a> EditingPositionContext<'a> {
    /// Called only after the host's synchronous editing lifecycle barrier.
    /// Platform policy comes from the current Page Settings owner.
    pub(super) fn NewForSelectionModifier(
        document: &'a Document,
        layout: LayoutOutput<'a>,
        ui_locale: &'a std::ffi::CStr,
        behavior: EditingBehaviorType,
    ) -> Self {
        Self {
            document,
            layout: Some(layout),
            flat: true,
            ui_locale: Some(ui_locale),
            behavior: Some(behavior),
        }
    }
    fn NativeCaretPosition(
        &self,
        p: &PositionWithAffinity<u64>,
    ) -> Option<EditingPositionWithAffinity> {
        NativePosition(&self.DomPosition(&p.position), p.affinity)
    }
    fn SourceCaretPosition(&self, p: EditingPositionWithAffinity) -> PositionWithAffinity<u64> {
        let mut source = SourcePosition(p);
        if self.flat {
            source.position =
                position_conversion::ToPositionInFlatTree(&self.Dom(), self, &source.position);
        }
        source
    }

    pub(super) fn Dom(&self) -> Self {
        Self {
            flat: false,
            ..*self
        }
    }
    pub(super) fn Flat(&self) -> Self {
        Self {
            flat: true,
            ..*self
        }
    }
    fn Index(&self, node: &u64) -> Option<usize> {
        self.document.FindNodeById(*node)
    }
    fn Layout(&self, node: &u64) -> Option<LayoutNode<'a>> {
        self.layout?.Node(*node)
    }
    fn IsEditableNodeValue(&self, node: &u64) -> bool {
        editable::IsEditable(
            &dom::persistent_document::DOMEditableFacts(self.document),
            *node,
            EditableFeatures::default(),
        )
    }
    fn DomPosition(&self, p: &Position<u64>) -> Position<u64> {
        if self.flat {
            position_conversion::ToPositionInDOMTree(&self.Dom(), self, p)
        } else {
            p.clone()
        }
    }
    fn IsDocumentValue(&self, node: &u64) -> bool {
        self.Index(node)
            .is_some_and(|i| self.document.Node(i).Type() == DOMNodeType::kDocument)
    }
    fn IsDescendant(&self, node: &u64, ancestor: &u64) -> bool {
        text_offset_mapping::IsDescendantOf(self, node, ancestor)
    }
}
fn NativePosition(
    p: &Position<u64>,
    affinity: TextAffinity,
) -> Option<EditingPositionWithAffinity> {
    Some(EditingPositionWithAffinity {
        position: EditingPosition::WithAnchor(
            *p.AnchorNode()?,
            p.OffsetInAnchor(),
            match p.AnchorType() {
                PositionAnchorType::OffsetInAnchor => EditingPositionAnchorType::OffsetInAnchor,
                PositionAnchorType::BeforeAnchor => EditingPositionAnchorType::BeforeAnchor,
                PositionAnchorType::AfterAnchor => EditingPositionAnchorType::AfterAnchor,
                PositionAnchorType::AfterChildren => EditingPositionAnchorType::AfterChildren,
            },
        ),
        affinity: match affinity {
            TextAffinity::Downstream => EditingTextAffinity::Downstream,
            TextAffinity::Upstream => EditingTextAffinity::Upstream,
        },
    })
}
fn SourcePosition(p: EditingPositionWithAffinity) -> PositionWithAffinity<u64> {
    let Some(node) = p.position.node_id else {
        return PositionWithAffinity::default();
    };
    let position = match p.position.anchor {
        EditingPositionAnchorType::OffsetInAnchor => Position::new(node, p.position.offset),
        EditingPositionAnchorType::BeforeAnchor => Position::BeforeNode(node),
        EditingPositionAnchorType::AfterAnchor => Position::AfterNode(node),
        EditingPositionAnchorType::AfterChildren => Position::AfterChildren(node),
    };
    PositionWithAffinity {
        position,
        affinity: match p.affinity {
            EditingTextAffinity::Downstream => TextAffinity::Downstream,
            EditingTextAffinity::Upstream => TextAffinity::Upstream,
        },
    }
}
impl PositionTree for EditingPositionContext<'_> {
    type Node = u64;
    fn Parent(&self, node: &u64) -> Option<u64> {
        let i = self.Index(node)?;
        let parent = if self.flat {
            self.document.FlatTreeParent(i)
        } else {
            self.document.Node(i).Parent()
        }?;
        Some(self.document.Node(parent).Id())
    }
    fn ChildAt(&self, node: &u64, index: usize) -> Option<u64> {
        let i = self.Index(node)?;
        let child = if self.flat {
            *self.document.FlatTreeChildren(i).get(index)?
        } else {
            *self.document.Node(i).Children().get(index)?
        };
        Some(self.document.Node(child).Id())
    }
    fn ChildCount(&self, node: &u64) -> usize {
        self.Index(node).map_or(0, |i| {
            if self.flat {
                self.document.FlatTreeChildren(i).len()
            } else {
                self.document.Node(i).Children().len()
            }
        })
    }
    fn NodeIndex(&self, node: &u64) -> usize {
        let Some(parent) = self.Parent(node) else {
            return 0;
        };
        let i = self.Index(node).expect("live child identity");
        let p = self.Index(&parent).expect("live parent identity");
        if self.flat {
            self.document
                .FlatTreeChildren(p)
                .iter()
                .position(|c| *c == i)
        } else {
            self.document
                .Node(p)
                .Children()
                .iter()
                .position(|c| *c == i)
        }
        .expect("source traversal child belongs to parent")
    }
    fn PreviousSibling(&self, node: &u64) -> Option<u64> {
        let parent = self.Parent(node)?;
        self.NodeIndex(node)
            .checked_sub(1)
            .and_then(|i| self.ChildAt(&parent, i))
    }
    fn NextSibling(&self, node: &u64) -> Option<u64> {
        self.ChildAt(&self.Parent(node)?, self.NodeIndex(node) + 1)
    }
    fn IsText(&self, node: &u64) -> bool {
        self.Index(node)
            .is_some_and(|i| self.document.Node(i).Type() == DOMNodeType::kText)
    }
    fn CharacterDataLength(&self, node: &u64) -> Option<usize> {
        let i = self.Index(node)?;
        matches!(
            self.document.Node(i).Type(),
            DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
        )
        .then(|| self.document.CharacterDataUTF16(i).len())
    }
}
impl PositionDOM for EditingPositionContext<'_> {
    fn EditingIgnoresContent(&self, node: &u64) -> bool {
        self.Index(node)
            .is_some_and(|i| self.document.EditingIgnoresContent(i))
    }
    fn IsDisplayInsideTable(&self, node: &u64) -> bool {
        self.Layout(node).is_some()
            && self
                .Index(node)
                .is_some_and(|i| self.document.IsHTMLTableElementForEditing(i))
    }
    fn IsConnected(&self, node: &u64) -> bool {
        self.Index(node)
            .is_some_and(|i| self.document.IsConnectedToDocument(i))
    }
    fn DocumentIdentity(&self, node: &u64) -> u128 {
        self.Index(node)
            .map_or(0, |i| self.document.NodeDocumentHandle(i).Identity())
    }
    fn LastDOMOffsetForEditing(&self, node: &u64) -> usize {
        self.Index(node)
            .map_or(0, |i| self.document.LastOffsetForEditing(i))
    }
}
impl PositionTreeScope for EditingPositionContext<'_> {
    fn IsShadowRoot(&self, node: &u64) -> bool {
        self.Index(node)
            .is_some_and(|i| self.document.ShadowHost(i).is_some())
    }
    fn ShadowHost(&self, node: &u64) -> Option<u64> {
        self.document
            .ShadowHost(self.Index(node)?)
            .map(|i| self.document.Node(i).Id())
    }
    fn TreeScopeRoot(&self, node: &u64) -> u64 {
        let i = self.Index(node).expect("live tree-scope node");
        self.document.Node(self.document.TreeScopeRoot(i)).Id()
    }
    fn IsShadowHost(&self, node: &u64) -> bool {
        self.Index(node)
            .is_some_and(|i| self.document.ShadowRoot(i).is_some())
    }
}
impl PositionIteratorContext for EditingPositionContext<'_> {
    fn IsUserSelectContain(&self, node: &u64) -> bool {
        self.Index(node).is_some_and(|i| {
            matches!(
                self.document.FormControlClass(i),
                Some(
                    DOMFormControlClass::Input
                        | DOMFormControlClass::TextArea
                        | DOMFormControlClass::Select
                )
            )
        })
    }
    fn HasLayoutObject(&self, node: &u64) -> bool {
        self.Layout(node).is_some()
    }
    fn NextGraphemeBoundary(&self, node: &u64, offset: usize) -> usize {
        if self.IsText(node) {
            NextGraphemeBoundary(
                &self
                    .document
                    .CharacterDataUTF16(self.Index(node).expect("live text")),
                offset,
            )
        } else {
            offset + 1
        }
    }
    fn LastOffsetForEditingInDOMTree(&self, node: &u64) -> usize {
        self.Index(node)
            .map_or(0, |i| self.document.LastOffsetForEditing(i))
    }
    fn PreviousGraphemeBoundary(&self, node: &u64, offset: usize) -> usize {
        if self.IsText(node) {
            PreviousGraphemeBoundary(
                &self
                    .document
                    .CharacterDataUTF16(self.Index(node).expect("live text")),
                offset,
            )
        } else {
            offset
                .checked_sub(1)
                .expect("previous boundary after node start")
        }
    }
}
impl VisibleUnitsContext for EditingPositionContext<'_> {
    type WritingMode = WritingMode;
    fn IsEditable(&self, node: &u64) -> bool {
        self.IsEditableNodeValue(node)
    }
    fn EndsAreVisuallyDistinct(&self, node: &u64) -> bool {
        let Some(index) = self.Index(node) else {
            return false;
        };
        self.Layout(node).is_some_and(|n| {
            n.EndsOfNodeAreVisuallyDistinctPositions(
                !self.IsText(node) && self.document.CanContainRangeEndPoint(index),
                self.document.IsHTMLTableElementForEditing(index),
                self.document.IsHTMLMarqueeElementForEditing(index),
            )
        })
    }
    fn CanHaveCaretPosition(&self, node: &u64) -> bool {
        let Some(i) = self.Index(node) else {
            return false;
        };
        visible_units::CanHaveCaretPosition(
            self.document.Node(i).Namespace() == DOMNamespace::kSVG,
            self.document.IsSVGTextElementForEditing(i),
            self.document.IsSVGForeignObjectElementForEditing(i),
        )
    }
    fn CaretFacts(
        &self,
        node: &u64,
        offset: usize,
        first_letter: bool,
    ) -> Option<AssociatedCaretFacts<WritingMode>> {
        let n = self.Layout(node)?;
        let base = n.EditingCaretFacts();
        if let Some(text) = n.EditingTextFacts(offset, first_letter) {
            return Some(AssociatedCaretFacts {
                visible: text.visible,
                blocked_by_display_lock: text.blocked_by_display_lock,
                writing_mode: text.writing_mode,
                text: Some(TextCaretFacts {
                    start_offset: text.start_offset as usize,
                    min_offset: text.caret_min_offset,
                    max_offset: text.caret_max_offset,
                    has_non_collapsed_text: text.has_non_collapsed_text,
                    contains_caret_offset: text.contains_caret_offset,
                    before_non_collapsed_character: text.is_before_non_collapsed_character,
                    after_non_collapsed_character: text.is_after_non_collapsed_character,
                }),
            });
        }
        Some(AssociatedCaretFacts {
            visible: base.visible,
            blocked_by_display_lock: base.blocked_by_display_lock,
            writing_mode: base.writing_mode,
            text: None,
        })
    }
    fn LayoutCaretFacts(&self, node: &u64) -> Option<CaretObjectFacts<WritingMode>> {
        let f = self.Layout(node)?.EditingCaretFacts();
        Some(CaretObjectFacts {
            visible: f.visible,
            blocked_by_display_lock: f.blocked_by_display_lock,
            writing_mode: f.writing_mode,
            is_text: f.is_text,
        })
    }
    fn CaretMaxOffset(&self, node: &u64) -> usize {
        self.Layout(node)
            .and_then(|n| n.EditingTextFacts(0, false))
            .map_or_else(
                || self.LastOffsetForEditing(node),
                |f| f.start_offset as usize + f.caret_max_offset,
            )
    }
    fn IsBody(&self, node: &u64) -> bool {
        self.Index(node) == self.document.BodyElement()
    }
    fn IsDocumentElement(&self, node: &u64) -> bool {
        self.Index(node) == self.document.DocumentElement()
    }
    fn IsDocument(&self, node: &u64) -> bool {
        self.IsDocumentValue(node)
    }
    fn IsSelectable(&self, node: &u64) -> bool {
        self.Layout(node).is_some_and(|n| n.EditingIsSelectable())
    }
    fn IsBR(&self, node: &u64) -> bool {
        self.Layout(node).is_some_and(|n| n.EditingIsBR())
    }
    fn IsSVG(&self, node: &u64) -> bool {
        self.Index(node)
            .is_some_and(|i| self.document.Node(i).Namespace() == DOMNamespace::kSVG)
    }
    fn IsBlockContainer(&self, node: &u64) -> bool {
        self.Layout(node)
            .is_some_and(|n| n.EditingIsBlockContainer())
    }
    fn IsBlockFlowElement(&self, node: &u64) -> bool {
        self.Layout(node).is_some_and(|n| n.EditingIsBlockFlow())
    }
    fn HasLogicalHeight(&self, node: &u64) -> bool {
        self.Layout(node)
            .is_some_and(|n| n.EditingHasLogicalHeight())
    }
    fn HasRenderedNonAnonymousDescendantsWithHeight(&self, node: &u64) -> bool {
        self.Layout(node)
            .is_some_and(|n| n.HasRenderedNonAnonymousDescendantsWithHeight())
    }
    fn BoundaryCaretPosition(
        &self,
        p: &Position<u64>,
        rule: Rule,
        backward: bool,
    ) -> Position<u64> {
        if backward {
            visible_units::MostBackwardCaretPosition(self, p, rule, SnapToClient::Editing)
        } else {
            visible_units::MostForwardCaretPosition(self, p, rule, SnapToClient::Editing)
        }
    }
}
impl EditingPositionRootContext for EditingPositionContext<'_> {
    fn RootEditableElement(&self, node: &u64) -> Option<u64> {
        editable::RootEditableElement(
            &dom::persistent_document::DOMEditableFacts(self.document),
            *node,
            EditableFeatures::default(),
        )
    }
    fn IsEditableNode(&self, node: &u64) -> bool {
        self.IsEditableNodeValue(node)
    }
    fn IsDocumentNode(&self, node: &u64) -> bool {
        self.IsDocumentValue(node)
    }
}
impl EditableBoundaryContext for EditingPositionContext<'_> {
    fn RootEditableElementOf(&self, p: &Position<u64>) -> Option<u64> {
        editing_boundaries::RootEditableElementOf(&self.Dom(), &self.DomPosition(p))
    }
    fn HighestEditableRoot(&self, p: &Position<u64>) -> Option<u64> {
        editing_boundaries::HighestEditableRoot(&self.Dom(), &self.DomPosition(p))
    }
    fn IsEditablePosition(&self, p: &Position<u64>) -> bool {
        editing_boundaries::IsEditablePosition(&self.Dom(), &self.DomPosition(p))
    }
    fn IsAtomicNodeInDOMTree(&self, node: &u64) -> bool {
        editing_boundaries::IsAtomicNode(&self.Dom(), node)
    }
    fn ComparePositions(&self, a: &Position<u64>, b: &Position<u64>) -> Option<Ordering> {
        if self.flat {
            compare_positions::CompareFlatPositions(self, a, b)
        } else {
            compare_positions::CompareDOMPositions(self, a, b)
        }
    }
    fn DOMContains(&self, a: &u64, n: &u64) -> bool {
        a == n || self.Dom().IsDescendant(n, a)
    }
}
impl DownstreamVisiblePositionContext for EditingPositionContext<'_> {
    fn MostBackwardCaretPosition(&self, p: &Position<u64>) -> Position<u64> {
        visible_units::MostBackwardCaretPosition(self, p, Rule::CanCross, SnapToClient::Editing)
    }
    fn MostForwardCaretPosition(&self, p: &Position<u64>) -> Position<u64> {
        visible_units::MostForwardCaretPosition(self, p, Rule::CanCross, SnapToClient::Editing)
    }
    fn IsVisuallyEquivalentCandidate(&self, p: &Position<u64>) -> bool {
        visible_units::IsVisuallyEquivalentCandidate(self, p)
    }
    fn NextCandidate(&self, p: &Position<u64>) -> Position<u64> {
        visible_units::NextCandidate(self, p)
    }
    fn PreviousCandidate(&self, p: &Position<u64>) -> Position<u64> {
        visible_units::PreviousCandidate(self, p)
    }
    fn IsEditable(&self, node: &u64) -> bool {
        self.IsEditableNodeValue(node)
    }
    fn RootEditableElement(&self, p: &Position<u64>) -> Option<u64> {
        EditableBoundaryContext::RootEditableElementOf(self, p)
    }
    fn IsDescendantOf(&self, n: &u64, a: &u64) -> bool {
        self.IsDescendant(n, a)
    }
    fn EnclosingBlockFlowElement(&self, node: &u64) -> Option<u64> {
        visible_units::EnclosingBlockFlowElement(self, node)
    }
    fn DocumentElement(&self, _: &u64) -> Option<u64> {
        self.document
            .DocumentElement()
            .map(|i| self.document.Node(i).Id())
    }
    fn Body(&self, _: &u64) -> Option<u64> {
        self.document
            .BodyElement()
            .map(|i| self.document.Node(i).Id())
    }
    fn IsDocument(&self, node: &u64) -> bool {
        self.IsDocumentValue(node)
    }
    fn UsePositionIfIsVisuallyEquivalentCandidateEnabled(&self) -> bool {
        true
    } // Stable feature in pinned RuntimeEnabledFeatures.json5.
    fn HasLayoutObject(&self, node: &u64) -> bool {
        self.Layout(node).is_some()
    }
    fn InSameLine(&self, a: &PositionWithAffinity<u64>, b: &PositionWithAffinity<u64>) -> bool {
        visible_units_line::InSameLine(self, a, b)
    }
    fn InlineFormattingContext(&self, p: &Position<u64>) -> Option<u64> {
        VisibleLineContext::InlineFormattingContextForPosition(self, p).map(|f| f.0)
    }
    fn CompareFlatPositions(&self, a: &Position<u64>, b: &Position<u64>) -> Option<Ordering> {
        compare_positions::CompareFlatPositions(&self.Flat(), a, b)
    }
    fn StartOfLine(&self, p: &PositionWithAffinity<u64>) -> PositionWithAffinity<u64> {
        visible_units_line::StartOfLine(self, p)
    }
}

impl VisiblePositionContext for EditingPositionContext<'_> {
    fn BidiCaretAffinityEnabled(&self) -> bool {
        // runtime_enabled_features.json5 leaves BidiCaretAffinity disabled in
        // the pinned default configuration.
        false
    }

    fn CaretBoundsEqual(
        &self,
        first: &PositionWithAffinity<u64>,
        second: &PositionWithAffinity<u64>,
    ) -> bool {
        let bounds = |position: &PositionWithAffinity<u64>| {
            let layout = self.layout?;
            let native = self.NativeCaretPosition(position)?;
            let node = layout.Node(native.position.node_id?)?;
            // AbsoluteCaretBoundsOf compares the integer-enclosing default
            // bar-caret rectangles. The exact width does not affect which
            // bidi boundary owns the caret; one CSS pixel is the source
            // default when no platform override is installed.
            node.CaretBounds(&native, Default::default(), 1.0)
        };
        bounds(first) == bounds(second)
    }
}

impl TextOffsetMappingContext for EditingPositionContext<'_> {
    type InlineContents = EditingInlineContentsBlockIdentity;
    fn ComputeInlineContentsFromNode(&self, n: &u64) -> Option<Self::InlineContents> {
        self.Layout(n)?.EditingInlineContentsBlockIdentity()
    }
    fn IsTextControl(&self, n: &u64) -> bool {
        self.Index(n).is_some_and(|i| {
            matches!(
                self.document.FormControlClass(i),
                Some(DOMFormControlClass::Input | DOMFormControlClass::TextArea)
            )
        })
    }
    fn HasUserAgentShadowRoot(&self, n: &u64) -> bool {
        self.Index(n)
            .and_then(|i| self.document.ShadowRoot(i))
            .is_some_and(|r| {
                self.document.ShadowRootMode(r)
                    == Some(dom::persistent_document::DOMShadowRootMode::kUserAgent)
            })
    }
    fn UserAgentShadowHost(&self, n: &u64) -> Option<u64> {
        let root = self.document.ContainingShadowRoot(self.Index(n)?)?;
        if self.document.ShadowRootMode(root)
            != Some(dom::persistent_document::DOMShadowRootMode::kUserAgent)
        {
            return None;
        }
        self.document
            .ShadowHost(root)
            .map(|h| self.document.Node(h).Id())
    }
}
impl VisibleLineContext for EditingPositionContext<'_> {
    fn InlineFormattingContextForPosition(&self, p: &Position<u64>) -> Option<(u64, bool)> {
        let native = NativePosition(&self.DomPosition(p), TextAffinity::Downstream)?;
        self.Layout(&native.position.node_id?)?
            .EditingInlineFormattingContextForPosition(&native.position)
    }
    fn InSameNGLineBox(
        &self,
        a: &PositionWithAffinity<u64>,
        b: &PositionWithAffinity<u64>,
    ) -> bool {
        let line = |p: &PositionWithAffinity<u64>| {
            let native = self.NativeCaretPosition(p)?;
            self.Layout(&native.position.node_id?)?
                .EditingLine(&native, Default::default())
        };
        line(a).zip(line(b)).is_some_and(|(a, b)| a == b)
    }
    fn ToFlatPosition(&self, p: &Position<u64>) -> Position<u64> {
        if self.flat {
            p.clone()
        } else {
            position_conversion::ToPositionInFlatTree(self, &self.Flat(), p)
        }
    }
    fn RootEditableElementOf(&self, p: &Position<u64>) -> Option<u64> {
        EditableBoundaryContext::RootEditableElementOf(self, p)
    }
    fn StartOfLine(&self, p: &PositionWithAffinity<u64>) -> PositionWithAffinity<u64> {
        visible_units_line::StartOfLine(self, p)
    }
    fn CanonicalPositionOf(&self, p: &Position<u64>) -> Position<u64> {
        interaction_default::core::editing::visible_position::CanonicalPositionOf(self, p)
    }
    fn ForwardInlineContentsInFlatTree(&self, p: &Position<u64>) -> Option<Self::InlineContents> {
        text_offset_mapping::FindForwardInlineContents(&self.Flat(), p)
    }
}
impl InlineAdjustmentContext for EditingPositionContext<'_> {
    fn InlineAdjustmentFacts(&self, p: &Position<u64>) -> Option<InlineAdjustmentFacts> {
        let n = self.Layout(p.AnchorNode()?)?;
        let f = n.EditingInlineAdjustmentFacts();
        Some(InlineAdjustmentFacts {
            is_text: f.is_text,
            is_inline: f.is_inline,
            is_atomic_inline: f.is_atomic_inline,
            in_ng_inline_context: f.is_in_ng_inline_context,
            is_block_flow: f.is_block_flow,
            has_rendered_descendants: n.HasRenderedNonAnonymousDescendantsWithHeight(),
        })
    }
    fn CanHaveChildrenForEditing(&self, n: &u64) -> bool {
        !self.IsText(n)
            && self
                .Index(n)
                .is_some_and(|i| self.document.CanContainRangeEndPoint(i))
    }
    fn LocalCaretEquivalent(&self, p: &Position<u64>, rule: Rule, backward: bool) -> Position<u64> {
        if backward {
            visible_units::MostBackwardCaretPosition(self, p, rule, SnapToClient::LocalCaretRect)
        } else {
            visible_units::MostForwardCaretPosition(self, p, rule, SnapToClient::LocalCaretRect)
        }
    }
}
impl StartOfLineContext for EditingPositionContext<'_> {
    fn NativeStartOfLine(&self, position: &PositionWithAffinity<u64>) -> PositionWithAffinity<u64> {
        self.NativeCaretPosition(position)
            .and_then(|native| {
                self.Layout(&native.position.node_id?)?
                    .StartOfLine(&native, Default::default())
            })
            .map(|native| self.SourceCaretPosition(native))
            .unwrap_or_default()
    }
    fn IsLayoutBlock(&self, n: &u64) -> bool {
        self.Layout(n)
            .is_some_and(|n| n.EditingInlineAdjustmentFacts().is_layout_block)
    }
}

fn NativeDOMSelection(
    anchor: &Position<u64>,
    focus: &Position<u64>,
    affinity: TextAffinity,
) -> Option<DOMSelection> {
    let anchor = NativePosition(anchor, affinity)?;
    let focus = NativePosition(focus, affinity)?;
    Some(DOMSelection {
        anchor: anchor.position,
        focus: focus.position,
        affinity: anchor.affinity,
    })
}

/// Element::UpdateSelectionOnFocus's collapsed DOM selection, before the
/// optional CreateVisibleSelection step. The caller supplies the same live
/// document after its synchronous lifecycle barrier.
pub fn SelectionAtFirstPositionInOrBeforeNodeForFocus(
    document: &Document,
    node_id: u64,
) -> Option<DOMSelection> {
    document.FindNodeById(node_id)?;
    let context = EditingPositionContext {
        document,
        layout: None,
        flat: false,
        ui_locale: None,
        behavior: None,
    };
    let position = Position::FirstPositionInOrBeforeNode(&context, &node_id);
    NativeDOMSelection(&position, &position, TextAffinity::Downstream)
}

/// A transient projection through the translated VisibleSelection algorithm;
/// this never publishes or retains selection state.
pub fn CreateVisibleDOMSelectionForFocus(
    document: &Document,
    layout: LayoutOutput<'_>,
    selection: Option<DOMSelection>,
) -> Option<DOMSelection> {
    let selection = selection?;
    let context = EditingPositionContext {
        document,
        layout: Some(layout),
        flat: false,
        ui_locale: None,
        behavior: None,
    };
    let anchor = SourcePosition(EditingPositionWithAffinity {
        position: selection.anchor,
        affinity: selection.affinity,
    });
    let focus = SourcePosition(EditingPositionWithAffinity {
        position: selection.focus,
        affinity: selection.affinity,
    });
    let visible = interaction_default::core::editing::visible_selection::CreateVisibleSelection(
        &context,
        &anchor.position,
        &focus.position,
        anchor.affinity,
    );
    NativeDOMSelection(&visible.anchor, &visible.focus, visible.affinity)
}

/// FrameSelection::SelectFrameElementInParentIfFullySelected reads the same
/// visible DOM selection and translated visible caret navigation as editing.
pub fn VisibleDOMSelectionCoversDocumentForFocus(
    document: &Document,
    layout: LayoutOutput<'_>,
    selection: Option<DOMSelection>,
) -> bool {
    let Some(selection) = CreateVisibleDOMSelectionForFocus(document, layout, selection) else {
        return false;
    };
    let context = EditingPositionContext {
        document,
        layout: Some(layout),
        flat: false,
        ui_locale: None,
        behavior: None,
    };
    let anchor = SourcePosition(EditingPositionWithAffinity {
        position: selection.anchor,
        affinity: selection.affinity,
    });
    let focus = SourcePosition(EditingPositionWithAffinity {
        position: selection.focus,
        affinity: selection.affinity,
    });
    let is_range = anchor.position != focus.position;
    let (mut start, mut end) =
        if compare_positions::CompareDOMPositions(&context, &anchor.position, &focus.position)
            == Some(Ordering::Greater)
        {
            (focus, anchor)
        } else {
            (anchor, focus)
        };
    // VisibleSelection::VisibleStart/VisibleEnd use opposite affinities for
    // ranges, preserving the source line-wrap endpoint interpretation.
    if is_range {
        start.affinity = TextAffinity::Downstream;
        end.affinity = TextAffinity::Upstream;
    }
    let start = VisiblePosition::Create(&context, start);
    let end = VisiblePosition::Create(&context, end);
    visible_units::IsStartOfDocument(&context, &start)
        && visible_units::IsEndOfDocument(&context, &end)
}

/// FrameSelection::Contains's original read-only geometry leaves. Child
/// Frames retain the same LayoutEditingState/SelectionEditor endpoints as
/// mutation paths; only the native read-only active hit is supplied lazily.
pub fn ResidentSelectionContains(
    document: &Document,
    layout: LayoutOutput<'_>,
    selections: &layoutng_assembly::editing_state::SelectionState,
    locale: &std::ffi::CStr,
    point: &foundation::PhysicalOffset,
    hit: impl FnOnce() -> Option<EditingPositionWithAffinity>,
) -> bool {
    use super::page_frame_selection::FrameSelectionValue;
    use interaction_default::core::editing::frame_selection::{
        ContainsSelection, SelectionContainmentSource,
    };
    #[derive(Clone)]
    struct Comparable<'a> {
        context: EditingPositionContext<'a>,
        position: Position<u64>,
    }
    impl PartialEq for Comparable<'_> {
        fn eq(&self, other: &Self) -> bool {
            self.partial_cmp(other) == Some(Ordering::Equal)
        }
    }
    impl PartialOrd for Comparable<'_> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            compare_positions::CompareFlatPositions(&self.context, &self.position, &other.position)
        }
    }
    struct Source<'a, F> {
        context: EditingPositionContext<'a>,
        hit: std::cell::RefCell<Option<F>>,
    }
    impl<'a, F: FnOnce() -> Option<EditingPositionWithAffinity>>
        SelectionContainmentSource<FrameSelectionValue> for Source<'a, F>
    {
        type Position = Comparable<'a>;
        fn HasLayoutView(&self) -> bool {
            self.context.layout.is_some()
        }
        fn ComputeVisibleSelectionInDOMTreeIsNonNull(
            &self,
            selection: &FrameSelectionValue,
        ) -> bool {
            CreateVisibleDOMSelectionForFocus(
                self.context.document,
                self.context.layout.unwrap(),
                selection.0,
            )
            .is_some()
        }
        fn VisibleFlatSelectionEndpoints(
            &self,
            selection: &FrameSelectionValue,
        ) -> Option<(Self::Position, Self::Position)> {
            let selection = selection.0?;
            let anchor = SourcePosition(EditingPositionWithAffinity {
                position: selection.anchor,
                affinity: selection.affinity,
            });
            let focus = SourcePosition(EditingPositionWithAffinity {
                position: selection.focus,
                affinity: selection.affinity,
            });
            let anchor = position_conversion::ToPositionInFlatTree(
                &self.context.Dom(),
                &self.context,
                &anchor.position,
            );
            let focus = position_conversion::ToPositionInFlatTree(
                &self.context.Dom(),
                &self.context,
                &focus.position,
            );
            let visible =
                interaction_default::core::editing::visible_selection::CreateVisibleSelection(
                    &self.context,
                    &anchor,
                    &focus,
                    SourcePosition(EditingPositionWithAffinity {
                        position: selection.anchor,
                        affinity: selection.affinity,
                    })
                    .affinity,
                );
            let a = Comparable {
                context: self.context,
                position: visible.anchor,
            };
            let f = Comparable {
                context: self.context,
                position: visible.focus,
            };
            Some(if a.partial_cmp(&f) == Some(Ordering::Greater) {
                (f, a)
            } else {
                (a, f)
            })
        }
        fn HitTestPositionReadOnlyActive(
            &self,
            _: &foundation::PhysicalOffset,
        ) -> Option<Self::Position> {
            let hit = (self.hit.borrow_mut().take()?)()?;
            let p = SourcePosition(hit);
            let position = position_conversion::ToPositionInFlatTree(
                &self.context.Dom(),
                &self.context,
                &p.position,
            );
            Some(Comparable {
                context: self.context,
                position,
            })
        }
    }
    let source = Source {
        context: EditingPositionContext {
            document,
            layout: Some(layout),
            flat: true,
            ui_locale: Some(locale),
            behavior: None,
        },
        hit: std::cell::RefCell::new(Some(hit)),
    };
    ContainsSelection(
        &source,
        || {
            FrameSelectionValue(super::page_frame_selection::CurrentDOMSelectionForFocus(
                document, selections,
            ))
        },
        point,
    )
}

/// VisibleSelection::RootEditableElement uses Start(), rather than the raw
/// anchor. Read the caller's canonical frame endpoints without a selection copy.
pub fn VisibleDOMSelectionRootEditableElementForFocus(
    document: &Document,
    layout: LayoutOutput<'_>,
    selection: Option<DOMSelection>,
) -> Option<u64> {
    let visible = CreateVisibleDOMSelectionForFocus(document, layout, selection)?;
    let context = EditingPositionContext {
        document,
        layout: Some(layout),
        flat: false,
        ui_locale: None,
        behavior: None,
    };
    let start = DOMSelectionStart(&context, visible);
    EditableBoundaryContext::RootEditableElementOf(&context, &start)
}

fn DOMSelectionStart(
    context: &EditingPositionContext<'_>,
    selection: DOMSelection,
) -> Position<u64> {
    let anchor = SourcePosition(EditingPositionWithAffinity {
        position: selection.anchor,
        affinity: selection.affinity,
    });
    let focus = SourcePosition(EditingPositionWithAffinity {
        position: selection.focus,
        affinity: selection.affinity,
    });
    if compare_positions::CompareDOMPositions(context, &anchor.position, &focus.position)
        == Some(Ordering::Greater)
    {
        focus.position
    } else {
        anchor.position
    }
}

/// Start().AnchorNode() of a previously computed visible DOM selection. This
/// preserves neighbor anchors and uses the source DOM position comparator.
pub fn VisibleDOMSelectionStartAnchorForFocus(
    document: &Document,
    selection: Option<DOMSelection>,
) -> Option<u64> {
    let context = EditingPositionContext {
        document,
        layout: None,
        flat: false,
        ui_locale: None,
        behavior: None,
    };
    DOMSelectionStart(&context, selection?)
        .AnchorNode()
        .copied()
}

/// EnclosingTextControl(selection.Anchor()) from the original DOM selection.
pub fn EnclosingTextControlForSelectionForFocus(
    document: &Document,
    selection: Option<DOMSelection>,
) -> Option<u64> {
    let selection = selection?;
    let context = EditingPositionContext {
        document,
        layout: None,
        flat: false,
        ui_locale: None,
        behavior: None,
    };
    let anchor = SourcePosition(EditingPositionWithAffinity {
        position: selection.anchor,
        affinity: selection.affinity,
    });
    text_offset_mapping::EnclosingTextControl(&context, &anchor.position)
}

/// The source ScopedForcedUpdate and optional temporary live Range. It owns
/// no DOM borrow, so native assembly may complete layout between Begin/End.
#[must_use = "end the forced selection scope on its original Document"]
pub struct ForcedSelectionLayoutScope {
    document: dom::persistent_document::DOMOwnerHandle,
    scope: dom::persistent_document::DisplayLockForcedScopeHandle,
    range: Option<dom::DOMRangeHandle>,
}

/// FrameSelection::ComputeVisibleSelectionInDomTreeDeprecated, cc:211-228.
/// The caller supplies only the embedding's current script-context flag; all
/// selection range/node choices remain with the source package.
pub fn BeginDisplayLockForcedSelectionForFocus(
    document: &mut Document,
    selection: Option<DOMSelection>,
    in_script_context: bool,
) -> ForcedSelectionLayoutScope {
    let context = EditingPositionContext {
        document,
        layout: None,
        flat: false,
        ui_locale: None,
        behavior: None,
    };
    let anchor = selection
        .map(|selection| {
            SourcePosition(EditingPositionWithAffinity {
                position: selection.anchor,
                affinity: selection.affinity,
            })
            .position
        })
        .unwrap_or_default();
    let focus = selection
        .map(|selection| {
            SourcePosition(EditingPositionWithAffinity {
                position: selection.focus,
                affinity: selection.affinity,
            })
            .position
        })
        .unwrap_or_default();
    let containers = anchor
        .ComputeContainerNode(&context)
        .zip(focus.ComputeContainerNode(&context));
    let range_points = containers.filter(|_| anchor != focus).map(|(start, end)| {
        (
            DOMBoundaryPoint {
                node_id: start,
                offset: anchor.ComputeOffsetInContainerNode(&context),
            },
            DOMBoundaryPoint {
                node_id: end,
                offset: focus.ComputeOffsetInContainerNode(&context),
            },
        )
    });
    let anchor_node = anchor
        .AnchorNode()
        .and_then(|node| document.FindNodeById(*node));
    let (scope, range) = if let Some((start, end)) = range_points {
        // The native Range constructor performs setStart then setEnd. Do not
        // pre-sort backward anchor/focus: source Range collapses to its end.
        let range = document
            .CreateRange(start, end)
            .expect("canonical selection has valid Range boundaries");
        let scope = document.BeginRangeDisplayLockForcedScope(
            range.clone(),
            dom::persistent_document::DisplayLockForcedPhase::Layout,
            in_script_context,
        );
        (scope, Some(range))
    } else {
        (
            document.BeginNodeDisplayLockForcedScope(
                anchor_node,
                dom::persistent_document::DisplayLockForcedPhase::Layout,
                false,
                false,
                true,
                in_script_context,
            ),
            None,
        )
    };
    ForcedSelectionLayoutScope {
        document: document.RootHandle(),
        scope,
        range,
    }
}

pub fn EndDisplayLockForcedSelectionForFocus(
    document: &mut Document,
    forced: ForcedSelectionLayoutScope,
) {
    assert_eq!(
        document.RootHandle(),
        forced.document,
        "forced selection scope belongs to its original Document"
    );
    document.EndDisplayLockForcedUpdate(forced.scope);
    if let Some(range) = forced.range {
        document.DisposeRange(&range);
    }
}

fn DOMSelectionRange(
    context: &EditingPositionContext<'_>,
    selection: Option<DOMSelection>,
) -> interaction_default::core::editing::ephemeral_range::EphemeralRange<u64> {
    use interaction_default::core::editing::ephemeral_range::EphemeralRange;
    let Some(selection) = selection else {
        return EphemeralRange::default();
    };
    let anchor = SourcePosition(EditingPositionWithAffinity {
        position: selection.anchor,
        affinity: selection.affinity,
    })
    .position;
    let focus = SourcePosition(EditingPositionWithAffinity {
        position: selection.focus,
        affinity: selection.affinity,
    })
    .position;
    if compare_positions::CompareDOMPositions(context, &anchor, &focus) == Some(Ordering::Greater) {
        EphemeralRange::new(focus, anchor)
    } else {
        EphemeralRange::new(anchor, focus)
    }
}

struct DisplayLockSelectionContexts<'a>(&'a Document);
impl interaction_default::core::display_lock::display_lock_utilities::SelectionChangedContext
    for DisplayLockSelectionContexts<'_>
{
    type Node = u64;
    type Context = u64;
    fn InclusiveAncestorContexts(&self, node: u64) -> Vec<u64> {
        self.0
            .FindNodeById(node)
            .map_or_else(Vec::new, |node| self.0.DisplayLockSelectionAncestors(node))
    }
}

/// FrameSelection::NotifyDisplayLockForSelectionChange (cc:1184-1203),
/// followed by DisplayLockUtilities::SelectionChanged (cc:730-780). Selected
/// nodes are visited through the borrowed original flat-tree traversal only.
pub fn NotifyDisplayLockForSelectionChangeForFocus(
    document: &mut Document,
    old: Option<DOMSelection>,
    new: Option<DOMSelection>,
) {
    use interaction_default::core::{
        display_lock::display_lock_utilities,
        editing::ephemeral_range::{EphemeralRange, ToEphemeralRangeInFlatTree},
    };
    if document.DisplayLockCount() == 0 {
        return;
    }
    let old_valid = {
        let dom = EditingPositionContext {
            document,
            layout: None,
            flat: false,
            ui_locale: None,
            behavior: None,
        };
        let range = DOMSelectionRange(&dom, old);
        let identity = document.RootHandle().Identity();
        range.StartPosition().IsValidFor(&dom, identity)
            && range.EndPosition().IsValidFor(&dom, identity)
    };
    if !old_valid {
        document.DisplayLockSelectionRemoved();
    }
    let changed = {
        let dom = EditingPositionContext {
            document,
            layout: None,
            flat: false,
            ui_locale: None,
            behavior: None,
        };
        let flat = dom.Flat();
        let old_range = if old_valid {
            ToEphemeralRangeInFlatTree(&dom, &flat, &DOMSelectionRange(&dom, old))
        } else {
            EphemeralRange::default()
        };
        let new_range = ToEphemeralRangeInFlatTree(&dom, &flat, &DOMSelectionRange(&dom, new));
        display_lock_utilities::SelectionChanged(
            &DisplayLockSelectionContexts(document),
            old_range.Nodes(&flat),
            new_range.Nodes(&flat),
        )
    };
    for node in changed.lost {
        if let Some(node) = document.FindNodeById(node) {
            document.NotifyDisplayLockSubtreeLostSelection(node);
        }
    }
    for node in changed.gained {
        if let Some(node) = document.FindNodeById(node) {
            document.NotifyDisplayLockSubtreeGainedSelection(node);
        }
    }
}

/// LayoutSelection::CalcSelectionInFlatTree, layout_selection.cc:138-164,
/// followed by the native LayoutSelection commit. Pure DOM conversion has no
/// LayoutOutput borrow, so the original engine can commit its resident owners
/// while the node iterator borrows only the live document traversal.
pub fn CommitLayoutSelectionForFocus(
    document: &Document,
    engine: &mut layoutng_assembly::layout_engine::LayoutEngine,
    selection: Option<DOMSelection>,
    hidden: bool,
) -> bool {
    use interaction_default::core::editing::ephemeral_range::{
        EphemeralRange, ToEphemeralRangeInFlatTree,
    };
    let dom = EditingPositionContext {
        document,
        layout: None,
        flat: false,
        ui_locale: None,
        behavior: None,
    };
    let flat = dom.Flat();
    let raw = DOMSelectionRange(&dom, selection);
    let range = if raw.IsCollapsed() {
        EphemeralRange::default()
    } else {
        ToEphemeralRangeInFlatTree(&dom, &flat, &raw)
    };
    let native = if range.IsNull() || range.IsCollapsed() {
        None
    } else {
        Some(layoutng_assembly::layout_selection::LayoutSelectionRange {
            nodes: range.Nodes(&flat),
            start: NativePosition(range.StartPosition(), TextAffinity::Downstream)
                .expect("non-null flat selection start")
                .position,
            end: NativePosition(range.EndPosition(), TextAffinity::Downstream)
                .expect("non-null flat selection end")
                .position,
        })
    };
    engine.CommitLayoutSelection(native, hidden)
}

/// FrameCaret::CaretPosition (frame_caret.cc:107-114). VisibleSelection owns
/// canonicalization; a non-collapsed visible selection has no insertion caret.
pub fn FrameCaretPosition(
    document: &Document,
    layout: LayoutOutput<'_>,
    selection: Option<DOMSelection>,
) -> Option<EditingPositionWithAffinity> {
    let visible = CreateVisibleDOMSelectionForFocus(document, layout, selection)?;
    (visible.anchor == visible.focus).then_some(EditingPositionWithAffinity {
        position: visible.anchor,
        affinity: visible.affinity,
    })
}

struct CaretSelectionFocus<'a> {
    context: EditingPositionContext<'a>,
    position: Position<u64>,
    affinity: TextAffinity,
    focused: u64,
}
impl super::page_element_focusability::PageElementFocusLayout for CaretSelectionFocus<'_> {
    fn HasLayoutObject(&self, id: u64) -> bool {
        self.context.Layout(&id).is_some()
    }
    fn IsUserScrollable(&self, id: u64) -> bool {
        self.context
            .Layout(&id)
            .is_some_and(|node| node.FocusNavigationFacts().is_user_scrollable)
    }
    fn IsCanvasLayoutObject(&self, id: u64) -> bool {
        self.context
            .Layout(&id)
            .is_some_and(|node| node.FocusNavigationFacts().is_canvas)
    }
}
impl CaretSelectionFocus<'_> {
    fn Focusability(&self) -> super::page_element_focusability::PageElementFocusability<'_> {
        super::page_element_focusability::PageElementFocusability::new(
            self.context.document,
            self,
            Some(self.focused),
        )
    }
}
impl interaction_default::core::editing::selection_focus::SelectionFocusDOM
    for CaretSelectionFocus<'_>
{
    type Node = u64;
    type Scope = u64;
    type Position = Position<u64>;
    fn UpdateStyleAndLayoutForSelection(&self) {
        // The caller prepared the resident LayoutOutput before taking this borrow.
        assert!(self.context.layout.unwrap().FragmentRoot().is_some());
    }
    fn VisibleFlatSelection(
        &self,
    ) -> Option<
        interaction_default::core::editing::selection_focus::SelectionFocusRange<Self::Position>,
    > {
        let flat = position_conversion::ToPositionInFlatTree(
            &self.context.Dom(),
            &self.context.Flat(),
            &self.position,
        );
        let visible = VisiblePosition::Create(
            &self.context.Flat(),
            PositionWithAffinity {
                position: flat,
                affinity: self.affinity,
            },
        );
        (!visible.IsNull()).then(|| {
            interaction_default::core::editing::selection_focus::SelectionFocusRange {
                start: visible.DeepEquivalent().clone(),
                end: visible.DeepEquivalent().clone(),
            }
        })
    }
    fn ContainerNode(&self, position: &Self::Position) -> Option<u64> {
        position.ComputeContainerNode(&self.context.Flat())
    }
    fn FocusedElement(&self) -> Option<u64> {
        Some(self.focused)
    }
    fn DocumentElement(&self) -> Option<u64> {
        self.context
            .document
            .DocumentElement()
            .map(|i| self.context.document.Node(i).Id())
    }
    fn IsScrollControlOrMarkerGroupPseudo(&self, node: u64) -> bool {
        self.context
            .Index(&node)
            .is_some_and(|i| self.Focusability().IsScrollControlPseudoElement(i))
    }
    fn IsTextControl(&self, node: u64) -> bool {
        self.context.Index(&node).is_some_and(|i| {
            matches!(
                self.context.document.FormControlClass(i),
                Some(DOMFormControlClass::Input | DOMFormControlClass::TextArea)
            )
        })
    }
    fn ContainsIncludingHostElements(&self, ancestor: u64, mut node: u64) -> bool {
        loop {
            if node == ancestor {
                return true;
            }
            let Some(index) = self.context.Index(&node) else {
                return false;
            };
            let Some(parent) = self
                .context
                .document
                .Node(index)
                .Parent()
                .or_else(|| self.context.document.ShadowHost(index))
            else {
                return false;
            };
            node = self.context.document.Node(parent).Id();
        }
    }
    fn FirstPositionInNode(&self, node: u64) -> Self::Position {
        Position::FirstPositionInNode(node)
    }
    fn ComparePositions(
        &self,
        first: &Self::Position,
        second: &Self::Position,
    ) -> Option<Ordering> {
        compare_positions::CompareFlatPositions(&self.context.Flat(), first, second)
    }
    fn SelectionAndFocusedVisiblePositionMatchEnabled(&self) -> bool {
        // runtime_enabled_features.json5: SelectionAndFocusedVisiblePositionMatch
        // is experimental and disabled by the embedding's default feature set.
        false
    }
    fn IsFocusable(&self, node: u64) -> bool {
        self.context
            .Index(&node)
            .is_some_and(|i| self.Focusability().IsFocusable(i))
    }
    fn IsEditable(&self, node: u64) -> bool {
        self.context.IsEditableNodeValue(&node)
    }
    fn VisiblePositionDeepEquivalent(&self, position: &Self::Position) -> Option<Self::Position> {
        let visible = VisiblePosition::Create(
            &self.context.Flat(),
            PositionWithAffinity {
                position: position.clone(),
                affinity: TextAffinity::Downstream,
            },
        );
        (!visible.IsNull()).then(|| visible.DeepEquivalent().clone())
    }
    fn TreeScope(&self, node: u64) -> u64 {
        PositionTreeScope::TreeScopeRoot(&self.context, &node)
    }
    fn IsElement(&self, node: u64) -> bool {
        self.context
            .Index(&node)
            .is_some_and(|i| self.context.document.Node(i).Type() == DOMNodeType::kElement)
    }
    fn DelegatesFocus(&self, node: u64) -> bool {
        self.context
            .Index(&node)
            .is_some_and(|i| self.context.document.ShadowRootDelegatesFocus(i))
    }
    fn GetFocusableArea(&self, node: u64) -> Option<u64> {
        let root = self
            .context
            .document
            .ShadowRoot(self.context.Index(&node)?)?;
        let root_id = self.context.document.Node(root).Id();
        if self.ContainsIncludingHostElements(root_id, self.focused) {
            return Some(self.focused);
        }
        let mut stack: Vec<_> = self
            .context
            .document
            .Node(root)
            .Children()
            .iter()
            .rev()
            .copied()
            .collect();
        while let Some(index) = stack.pop() {
            if self.Focusability().IsFocusable(index) {
                return Some(self.context.document.Node(index).Id());
            }
            stack.extend(
                self.context
                    .document
                    .Node(index)
                    .Children()
                    .iter()
                    .rev()
                    .copied(),
            );
        }
        None
    }
    fn ParentIsShadowRoot(&self, node: u64) -> bool {
        PositionTree::Parent(&self.context.Dom(), &node)
            .is_some_and(|parent| PositionTreeScope::IsShadowRoot(&self.context, &parent))
    }
    fn FlatTreeParent(&self, node: u64) -> Option<u64> {
        PositionTree::Parent(&self.context.Flat(), &node)
    }
}

/// Source FrameCaret::ShouldShowCaret and CaretDisplayItemClient's native
/// painter update. Read canonical selection and live resident geometry only.
pub fn FrameCaretForSelection(
    document: &Document,
    layout: LayoutOutput<'_>,
    selections: &layoutng_assembly::editing_state::SelectionState,
    focused: Option<u64>,
) -> Option<layoutng_assembly::caret::TextCaret> {
    let _lifecycle = document.Lifecycle().DisallowTransition();
    let focused = focused?;
    if let Some(control) = selections.ActiveControl().filter(|id| *id == focused) {
        let index = document.FindNodeById(control)?;
        let node = document.Node(index);
        if node.FindAttribute("disabled").is_some() || node.FindAttribute("readonly").is_some() {
            return None;
        }
        let raw_selection = super::page_frame_selection::CurrentDOMSelectionForFocus(
            document,
            selections,
        )?;
        let position = FrameCaretPosition(document, layout, Some(raw_selection))?;
        let source_position = SourcePosition(position);
        let editing_offset = source_position
            .position
            .ComputeEditingOffset(&EditingPositionContext {
                document,
                layout: Some(layout),
                flat: false,
                ui_locale: None,
                behavior: None,
            })
            .min(i32::MAX as usize) as i32;
        let layout_node = layout.Node(position.position.node_id?)?;
        let composing = selections.Composition(control).is_some();
        let (node_id, native) = layout_node.CaretPainterGeometryForComposition(
            &position,
            editing_offset,
            layout_node.CaretBarWidth(),
            composing,
        )?;
        let mut caret =
            selections.CaretForUTF16Value(control, &document.ControlValueUTF16(index))?;
        caret.node_id = node_id;
        caret.native = Some(native);
        caret.control_block_text = caret.composition.is_none()
            && layout
                .Node(control)
                .is_some_and(|node| node.HasBlockCaretTextOverride());
        return Some(caret);
    }
    let context = EditingPositionContext {
        document,
        layout: Some(layout),
        flat: false,
        ui_locale: None,
        behavior: None,
    };
    let raw_selection = selections.DOMSelection()?;
    let root =
        VisibleDOMSelectionRootEditableElementForFocus(document, layout, Some(raw_selection))?;
    let composition = selections.Composition(root);
    let position = FrameCaretPosition(document, layout, Some(raw_selection)).or_else(|| {
        // A selected preedit has no visible insertion bar, but retains the
        // native focus rectangle for the platform candidate window.
        composition?;
        let visible = VisiblePosition::Create(
            &context,
            SourcePosition(EditingPositionWithAffinity {
                position: raw_selection.focus,
                affinity: raw_selection.affinity,
            }),
        );
        NativePosition(
            visible.DeepEquivalent(),
            visible.ToPositionWithAffinity().affinity,
        )
    })?;
    let source_position = SourcePosition(position);
    let source = source_position.position;
    EditableBoundaryContext::RootEditableElementOf(&context, &source)?;
    if !context.IsEditableNodeValue(source.AnchorNode()?)
        || !interaction_default::core::editing::selection_focus::SelectionHasFocus(
            &CaretSelectionFocus {
                context,
                position: source.clone(),
                affinity: source_position.affinity,
                focused,
            },
        )
    {
        return None;
    }
    let offset = source.ComputeEditingOffset(&context).min(i32::MAX as usize) as i32;
    let node = layout.Node(position.position.node_id?)?;
    let (node_id, native) = node.CaretPainterGeometryForComposition(
        &position,
        offset,
        node.CaretBarWidth(),
        composition.is_some(),
    )?;
    let selected_offsets = if raw_selection.anchor != raw_selection.focus && composition.is_some() {
        match QueryCompositionText(
            document,
            layout,
            CompositionTextQuery::SelectionRange(raw_selection),
        )? {
            CompositionTextResult::Range(range) => match QueryCompositionText(
                document,
                layout,
                CompositionTextQuery::Offsets { root, range },
            )? {
                CompositionTextResult::Offsets(range) => {
                    Some((range.start as u32, range.end as u32))
                }
                _ => None,
            },
            _ => None,
        }
    } else {
        None
    };
    Some(layoutng_assembly::caret::TextCaret {
        node_id,
        text_control_host: None,
        utf16_offset: offset as u32,
        empty: false,
        composition: composition.map(|range| {
            (
                range.Start().min(u32::MAX as usize) as u32,
                range.End().min(u32::MAX as usize) as u32,
            )
        }),
        selection: selected_offsets,
        native: Some(native),
        control_block_text: false,
    })
}

/// DragCaret::UpdateStyleAndLayoutIfNeeded resolves the retained native range
/// against the same live resident fragments and canonical anchor style.
pub fn DragCaretForSelection(
    document: &Document,
    layout: LayoutOutput<'_>,
    selection: Option<DOMSelection>,
) -> Option<layoutng_assembly::caret::TextCaret> {
    let _lifecycle = document.Lifecycle().DisallowTransition();
    let selection = selection?;
    VisibleDOMSelectionRootEditableElementForFocus(document, layout, Some(selection))?;
    let position = FrameCaretPosition(document, layout, Some(selection))?;
    let context = EditingPositionContext {
        document,
        layout: Some(layout),
        flat: false,
        ui_locale: None,
        behavior: None,
    };
    let source = SourcePosition(position).position;
    let offset = source.ComputeEditingOffset(&context).min(u32::MAX as usize) as u32;
    let node = layout.Node(position.position.node_id?)?;
    let (node_id, native) = node.CaretPainterGeometryForComposition(
        &position,
        offset as i32,
        node.CaretBarWidth(),
        false,
    )?;
    Some(layoutng_assembly::caret::TextCaret {
        node_id,
        text_control_host: None,
        utf16_offset: offset,
        empty: false,
        composition: None,
        selection: None,
        native: Some(native),
        control_block_text: false,
    })
}

/// FrameCaret::CaretPosition/AbsoluteCaretBounds (frame_caret.cc:107-114,
/// 216-223), or FrameSelection's visible focus extent (cc:1395-1398). The
/// resident Layout caret query owns local geometry and absolute transforms.
pub fn CaretBoundsForSelectionForFocus(
    document: &Document,
    layout: LayoutOutput<'_>,
    selection: Option<DOMSelection>,
    reveal_extent: interaction_default::core::editing::frame_selection::RevealExtentOption,
    bar_width: f64,
) -> foundation::gfx::Rect {
    use interaction_default::core::editing::frame_selection::RevealExtentOption;
    let _lifecycle = document.Lifecycle().DisallowTransition();
    let Some(selection) = selection else {
        return Default::default();
    };
    let context = EditingPositionContext {
        document,
        layout: Some(layout),
        flat: false,
        ui_locale: None,
        behavior: None,
    };
    let caret_position = FrameCaretPosition(document, layout, Some(selection));
    let is_caret = caret_position.is_some();
    let position = if is_caret {
        caret_position
    } else if reveal_extent == RevealExtentOption::kRevealExtent {
        let focus = SourcePosition(EditingPositionWithAffinity {
            position: selection.focus,
            affinity: EditingTextAffinity::Downstream,
        });
        let focus = VisiblePosition::Create(&context, focus);
        NativePosition(
            focus.DeepEquivalent(),
            focus.ToPositionWithAffinity().affinity,
        )
    } else {
        None
    };
    let Some(position) = position else {
        return Default::default();
    };
    let Some(node) = layout.Node(position.position.node_id.unwrap()) else {
        return Default::default();
    };
    let editing_offset = SourcePosition(position)
        .position
        .ComputeEditingOffset(&context)
        .min(i32::MAX as usize) as i32;
    // Source LocalCaretRect handles both inline positions and empty/atomic
    // boxes. CaretBounds' inline-only query cannot represent the latter.
    let (quad, empty) = if is_caret {
        node.InsertionCaretQuadAndEmpty(&position, editing_offset, bar_width)
    } else {
        node.CaretQuadAndEmpty(&position, Default::default(), editing_offset, bar_width)
    };
    if empty {
        Default::default()
    } else {
        foundation::gfx::ToEnclosingRect(quad.BoundingBox())
    }
}

/// SelectionModifier::ModifyExtendingBackwardInternal(kCharacter), source
/// selection_modifier.cc:654-664, with the original flat-tree visible focus.
/// The caller already owns a prepared Layout result and holds a live DOM read.
pub(crate) fn PreviousVisibleTextControlOffset(
    document: &Document,
    control: usize,
    caret: usize,
    geometry: PreparedEditingGeometry<'_>,
) -> Option<usize> {
    use dom::document_lifecycle::LifecycleState;
    let state = document.Lifecycle().GetState();
    if state < LifecycleState::LayoutClean || state >= LifecycleState::Stopping {
        return None;
    }
    geometry.AssertPrepared();
    let _lifecycle = document.Lifecycle().DisallowTransition();
    let context = EditingPositionContext {
        document,
        layout: Some(geometry.Output()),
        flat: true,
        ui_locale: None,
        behavior: None,
    };
    let dom = context.Dom();
    let point = document.TextControlInnerEditorPosition(control, caret)?;
    let raw = Position::new(point.node_id, point.offset);
    let flat = position_conversion::ToPositionInFlatTree(&dom, &context, &raw);
    let visible = VisiblePosition::CreateDownstream(
        &context,
        PositionWithAffinity {
            position: flat,
            affinity: TextAffinity::Downstream,
        },
    );
    if visible.IsNull() {
        return None;
    }
    let previous = visible_navigation::PreviousPositionOf(
        &context,
        visible.DeepEquivalent(),
        Rule::CanSkipOver,
    );
    // A null previous position means the caret is already at the editable edge.
    if previous.IsNull() {
        return Some(caret);
    }
    let previous =
        position_conversion::ToPositionInDOMTree(&dom, &context, previous.DeepEquivalent())
            .ToOffsetInAnchor(&dom);
    document.TextControlInnerEditorOffset(
        control,
        DOMBoundaryPoint {
            node_id: *previous.AnchorNode()?,
            offset: previous.OffsetInAnchor(),
        },
    )
}

// Source HitTestResult.cc:221-244,273-278: pseudo/default positions still use
// the canonical DOM VisibleUnits owner and current resident Layout facts.
pub(super) fn ResidentHitCaretPosition(
    document: &Document,
    layout: LayoutOutput<'_>,
    node: u64,
    query: interaction_hit_test::core::layout::hit_test_result::CaretPositionQuery,
    ui_locale: &std::ffi::CStr,
) -> Option<EditingPositionWithAffinity> {
    use interaction_hit_test::core::layout::hit_test_result::CaretPositionQuery as Query;
    document.FindNodeById(node)?;
    let context = EditingPositionContext {
        document,
        layout: Some(layout),
        flat: false,
        ui_locale: Some(ui_locale),
        behavior: None,
    };
    let position = match query {
        Query::NodeOffsetZeroDefaultAffinity => Position::new(node, 0),
        Query::FirstInOrBefore => Position::FirstPositionInOrBeforeNode(&context, &node),
        Query::FirstMostForward => visible_units::MostForwardCaretPosition(
            &context,
            &Position::FirstPositionInNode(node),
            Rule::CanCross,
            SnapToClient::Editing,
        ),
        Query::LastMostBackward => visible_units::MostBackwardCaretPosition(
            &context,
            &Position::LastPositionInNode(&context, &node),
            Rule::CanCross,
            SnapToClient::Editing,
        ),
    };
    NativePosition(&position, TextAffinity::Downstream)
}
pub(super) fn AdjustResidentPointPosition(
    document: &Document,
    layout: LayoutOutput<'_>,
    native: EditingPositionWithAffinity,
    ui_locale: &std::ffi::CStr,
) -> Option<EditingPositionWithAffinity> {
    let context = EditingPositionContext {
        document,
        layout: Some(layout),
        flat: false,
        ui_locale: Some(ui_locale),
        behavior: None,
    };
    let adjusted = visible_units::AdjustForEditingBoundary(&context, SourcePosition(native));
    NativePosition(&adjusted.position, adjusted.affinity)
}

impl interaction_default::core::editing::commands::editor_command_target_ranges::BackwardDeletionContext
    for EditingPositionContext<'_>
{
    fn CharacterDataUTF16ForDeletion(&self, node: &u64) -> std::rc::Rc<[u16]> {
        self.document.CharacterDataUTF16(self.Index(node).expect("live CharacterData deletion node"))
    }
}
impl visible_units_line::LogicalLineContext for EditingPositionContext<'_> {
    fn IsLayoutBlockFlowNode(&self, node: &u64) -> bool {
        self.Layout(node)
            .is_some_and(|node| node.EditingIsBlockFlow())
    }
    fn NativeLogicalStartOfLine(
        &self,
        position: &PositionWithAffinity<u64>,
    ) -> PositionWithAffinity<u64> {
        self.NativeCaretPosition(position)
            .and_then(|native| {
                self.Layout(&native.position.node_id?)?
                    .LogicalStartOfLine(&native, Default::default())
            })
            .map(|native| self.SourceCaretPosition(native))
            .unwrap_or_default()
    }
    fn NativeLogicalEndOfLine(
        &self,
        position: &PositionWithAffinity<u64>,
    ) -> PositionWithAffinity<u64> {
        self.NativeCaretPosition(position)
            .and_then(|native| {
                self.Layout(&native.position.node_id?)?
                    .LogicalEndOfLine(&native, Default::default())
            })
            .map(|native| self.SourceCaretPosition(native))
            .unwrap_or_default()
    }
}
impl LineMovementContext for EditingPositionContext<'_> {
    fn NativePositionOnLine(
        &self,
        position: &PositionWithAffinity<u64>,
        coordinate: LayoutUnit,
        adjacent: Option<bool>,
        only_editable: bool,
    ) -> LinePositionResult<u64> {
        use layoutng_assembly::layout_engine::EditingLinePositionResult as NativeResult;
        // AbstractLineBox::CreateFor normalizes in flat strategy, then resolves
        // the original DOM position on the resident LayoutObject.
        let adjusted =
            interaction_default::core::editing::inline_box_position::ComputeInlineAdjustedPosition(
                self,
                position.clone(),
                Rule::CanCross,
            );
        let Some(native) = self.NativeCaretPosition(&adjusted) else {
            return LinePositionResult::NoLine;
        };
        let Some(node) = native.position.node_id.and_then(|node| self.Layout(&node)) else {
            return LinePositionResult::NoLine;
        };
        let result = if let Some(next) = adjacent {
            node.EditingPositionOnAdjacentLine(
                &native,
                Default::default(),
                coordinate,
                next,
                only_editable,
            )
        } else {
            node.EditingPositionOnLine(&native, Default::default(), coordinate, only_editable)
        };
        match result {
            NativeResult::NoLine => LinePositionResult::NoLine,
            NativeResult::Line(position) => {
                LinePositionResult::Line(position.map(|p| self.SourceCaretPosition(p)))
            }
        }
    }
    fn AtomicInlineFormattingContextNode(&self, position: &Position<u64>) -> Option<u64> {
        let native = NativePosition(&self.DomPosition(position), TextAffinity::Downstream)?;
        self.Layout(&native.position.node_id?)?
            .EditingAtomicInlineFormattingContextNode(&native.position)
    }
    fn RootEditableElementOfNode(&self, node: &u64) -> Option<u64> {
        EditingPositionRootContext::RootEditableElement(&self.Dom(), node)
    }
    fn CaretMinOffset(&self, node: &u64) -> usize {
        self.Layout(node)
            .and_then(|node| node.EditingTextFacts(0, false))
            .map_or(0, |facts| {
                facts.start_offset as usize + facts.caret_min_offset
            })
    }
}
impl SelectionModifierContext for EditingPositionContext<'_> {
    type LifecycleGuard = dom::document_lifecycle::DisallowTransitionScope;
    fn AssertLayoutPrepared(&self) {
        use dom::document_lifecycle::LifecycleState;
        assert!(
            self.layout
                .is_some_and(|layout| layout.FragmentRoot().is_some()),
            "selection modification requires resident layout"
        );
        let state = self.document.Lifecycle().GetState();
        assert!(
            state >= LifecycleState::LayoutClean && state < LifecycleState::Stopping,
            "selection modification requires the editing lifecycle barrier"
        );
    }
    fn UpdateAllLifecyclePhasesExceptPaint(&self) {
        // The embedding completes the synchronous barrier before lending Layout.
        // No rendering engine is called through this call-scoped read adapter.
        self.AssertLayoutPrepared();
    }
    fn DisallowLifecycleTransitions(&self) -> Self::LifecycleGuard {
        self.document.Lifecycle().DisallowTransition()
    }
    fn Behavior(&self) -> EditingBehaviorType {
        self.behavior
            .expect("selection modifier requires current Page editing behavior")
    }
    fn IsSpatialNavigationEnabled(&self) -> bool {
        // Rechrom's Settings do not enable spatial navigation.
        false
    }
    fn CreateVisibleSelectionInFlatTree(
        &self,
        selection: &VisibleSelection<u64>,
    ) -> VisibleSelection<u64> {
        visible_selection::CreateVisibleCharacterSelectionInFlatTree(
            &self.Flat(),
            &selection.anchor,
            &selection.focus,
            selection.affinity,
        )
    }
    fn CreateVisibleSelectionInDOMTree(
        &self,
        selection: &VisibleSelection<u64>,
    ) -> VisibleSelection<u64> {
        let anchor =
            position_conversion::ToPositionInDOMTree(&self.Dom(), &self.Flat(), &selection.anchor);
        let focus =
            position_conversion::ToPositionInDOMTree(&self.Dom(), &self.Flat(), &selection.focus);
        visible_selection::CreateVisibleSelection(&self.Dom(), &anchor, &focus, selection.affinity)
    }
    fn LayoutTextDirection(&self, node: &u64) -> Option<TextDirection> {
        self.Layout(node)?;
        let mut index = self.Index(node)?;
        loop {
            if let Some(style) = self
                .document
                .ResolvedStyleFor(index)
                .and_then(|style| style.ComputedStyle())
            {
                return Some(style.Direction());
            }
            index = self.document.FlatTreeParent(index)?;
        }
    }
    fn NativeCaretDirections(
        &self,
        adjusted: &PositionWithAffinity<u64>,
    ) -> Option<(TextDirection, TextDirection)> {
        let native = self.NativeCaretPosition(adjusted)?;
        let directions = self
            .Layout(&native.position.node_id?)?
            .EditingCaretDirections(&native, Default::default())?;
        Some((
            directions.resolved_direction,
            directions.line_base_direction,
        ))
    }
    fn NativeMoveCaretVisually(
        &self,
        position: &PositionWithAffinity<u64>,
        level: Option<u8>,
        entered: bool,
        right: bool,
    ) -> VisualCaretMoveResult<u64> {
        use layoutng_assembly::layout_engine::VisualCaretMoveDirection;
        let native = self.NativeCaretPosition(position).and_then(|native| {
            Some(self.Layout(&native.position.node_id?)?.MoveCaretVisually(
                &native,
                level,
                entered,
                if right {
                    VisualCaretMoveDirection::Right
                } else {
                    VisualCaretMoveDirection::Left
                },
            ))
        });
        native.map_or_else(
            || VisualCaretMoveResult {
                position: Default::default(),
                bidi_level: None,
                entered_bidi_run: false,
            },
            |result| VisualCaretMoveResult {
                position: self.SourceCaretPosition(result.position),
                bidi_level: result.bidi_level,
                entered_bidi_run: result.entered_bidi_run,
            },
        )
    }
    fn CaretLineDirectionCoordinateIgnoringTransforms(
        &self,
        position: &PositionWithAffinity<u64>,
    ) -> Option<LayoutUnit> {
        let native = self.NativeCaretPosition(position)?;
        let node = self.Layout(&native.position.node_id?)?;
        node.EditingCaretLineDirectionCoordinateIgnoringTransforms(
            &native,
            Default::default(),
            node.CaretBarWidth(),
        )
    }
    fn AbsoluteCaretBounds(&self, position: &PositionWithAffinity<u64>) -> foundation::gfx::Rect {
        let dom = self.Dom();
        interaction_default::core::editing::visible_units_commands::CommandPositionContext::AbsoluteCaretBounds(
            &dom, &PositionWithAffinity { position: self.DomPosition(&position.position), affinity: position.affinity },
        )
    }
    fn WordPosition(
        &self,
        position: &Position<u64>,
        backward: bool,
        behavior: PlatformWordBehavior,
    ) -> PositionWithAffinity<u64> {
        if backward {
            position_units_word::PreviousWordPosition(self, position)
        } else {
            position_units_word::NextWordPosition(self, position, behavior)
        }
    }
    fn SentencePosition(&self, position: &Position<u64>, backward: bool) -> Position<u64> {
        if backward {
            position_units_sentence::PreviousSentencePosition(self, position)
        } else {
            position_units_sentence::NextSentencePosition(self, position)
        }
    }
    fn StartOfSentencePosition(&self, position: &Position<u64>) -> Position<u64> {
        position_units_sentence::StartOfSentencePosition(self, position)
    }
    fn EndOfSentence(&self, position: &Position<u64>) -> PositionWithAffinity<u64> {
        position_units_sentence::EndOfSentence(
            self,
            position,
            SentenceTrailingSpaceBehavior::IncludeSpace,
        )
    }
    fn LogicalLineBoundary(
        &self,
        position: &PositionWithAffinity<u64>,
        end: bool,
    ) -> PositionWithAffinity<u64> {
        visible_units_line::LogicalLineBoundary(self, position, end)
    }
    fn LinePosition(
        &self,
        position: &PositionWithAffinity<u64>,
        coordinate: LayoutUnit,
        next: bool,
    ) -> PositionWithAffinity<u64> {
        if next {
            selection_modifier_line::NextLinePosition(self, position, coordinate)
        } else {
            selection_modifier_line::PreviousLinePosition(self, position, coordinate)
        }
    }
}
