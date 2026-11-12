//! Call-scoped native position queries for the source SelectionController.
//! Every result is a position or source cursor; resident DOM/Layout are borrowed
//! only during the query, never across selectstart or focus dispatch.
use super::*;
use interaction_default::core::editing::{
    position_iterator::PositionIterator,
    selection_adjuster::{self, WordInclusion},
    text_granularity::TextGranularity,
    visible_selection,
};
use std::ffi::CString;

pub enum MouseSelectionGeometryQuery {
    Misspelled {
        position: EditingPositionWithAffinity,
        owner: CompositionMarkerOwner,
        focused: Option<u64>,
        page_default: bool,
    },
    MarkerGroup {
        position: EditingPositionWithAffinity,
        owner: CompositionMarkerOwner,
    },
    Annotation {
        position: EditingPositionWithAffinity,
        owner: CompositionMarkerOwner,
        has_inner_node: bool,
        has_frame: bool,
    },
    Equivalent(EditingPosition, EditingPosition),
    NodeDraggable(u64),
    AdjustBidi(EditingPositionWithAffinity, EditingPositionWithAffinity),
    SelectionFocus {
        selection: Option<DOMSelection>,
        focused: Option<u64>,
    },
    EditingBoundary {
        anchor: EditingPosition,
        target: u64,
        hit: EditingPositionWithAffinity,
        local_point: foundation::PhysicalOffset,
        options: layoutng_assembly::EditingPositionOptions,
    },
    ParentAnchored(EditingPositionWithAffinity),
    FirstInNode(u64),
    LastInNode(u64),
    EnclosingBlock(u64),
    RootEditableHasChildren(u64),
    Expand {
        selection: DOMSelection,
        granularity: TextGranularity,
    },
    Start(EditingPositionWithAffinity, TextGranularity),
    End(
        EditingPositionWithAffinity,
        EditingPositionWithAffinity,
        TextGranularity,
    ),
    Previous(EditingPositionWithAffinity),
    IteratorStart(EditingPositionWithAffinity),
    IteratorStep(PositionIterator<u64>, bool),
}
pub struct MouseSelectionCursor {
    pub iterator: PositionIterator<u64>,
    pub position: Option<EditingPositionWithAffinity>,
    pub node: Option<u64>,
    pub at_start: bool,
    pub at_end: bool,
}
pub enum MouseSelectionGeometryResult {
    Bounds(Option<(EditingPositionWithAffinity, EditingPositionWithAffinity)>),
    Annotation(
        Option<interaction_default::core::editing::selection_hit_test::MarkerAnnotationType>,
    ),
    Boolean(bool),
    OptionalBoolean(Option<bool>),
    Position(Option<EditingPositionWithAffinity>),
    Selection(Option<DOMSelection>),
    Node(Option<u64>),
    Cursor(MouseSelectionCursor),
}
pub(super) fn QueryMouseSelectionGeometry(
    document: &Document,
    layout: LayoutOutput<'_>,
    query: MouseSelectionGeometryQuery,
    locale: &CString,
) -> Option<MouseSelectionGeometryResult> {
    use MouseSelectionGeometryQuery as Q;
    use MouseSelectionGeometryResult as V;
    let _lifecycle = document.Lifecycle().DisallowTransition();
    let ctx = EditingPositionContext {
        document,
        layout: Some(layout),
        flat: true,
        ui_locale: Some(locale.as_c_str()),
        behavior: None,
    };
    let flat = |p| {
        let p = SourcePosition(p);
        PositionWithAffinity {
            position: position_conversion::ToPositionInFlatTree(&ctx.Dom(), &ctx, &p.position),
            affinity: p.affinity,
        }
    };
    let native = |p: &Position<u64>, a| {
        NativePosition(
            &position_conversion::ToPositionInDOMTree(&ctx.Dom(), &ctx, p),
            a,
        )
    };
    let downstream = |p| {
        flat(EditingPositionWithAffinity {
            position: p,
            affinity: EditingTextAffinity::Downstream,
        })
    };
    let cursor = |iterator: PositionIterator<u64>| {
        let p = iterator.ComputePosition(&ctx);
        V::Cursor(MouseSelectionCursor {
            position: native(&p, TextAffinity::Downstream),
            node: iterator.GetNode().copied(),
            at_start: iterator.AtStart(&ctx),
            at_end: iterator.AtEnd(&ctx),
            iterator,
        })
    };
    Some(match query {
        Q::Misspelled{position,owner,focused,page_default}=>V::Boolean(interaction_default::core::editing::selection_hit_test::HitTestResultIsMisspelled(&MouseSpellCheckingContext{context:ctx.Dom(),focused},&ctx,owner.SourceController(),&SourcePosition(position).position,page_default)),
        Q::MarkerGroup{position,owner}=>V::Bounds(owner.SourceController().FirstMarkerGroupAroundPosition(&ctx,&flat(position).position,interaction_default::core::editing::markers::document_marker_controller::MarkerTypes::Misspelling()).and_then(|g|g.Bounds(&ctx.Dom())).and_then(|(a,b)|Some((NativePosition(&a,TextAffinity::Downstream)?,NativePosition(&b,TextAffinity::Downstream)?)))),
        Q::Annotation{position,owner,has_inner_node,has_frame}=>V::Annotation(interaction_default::core::editing::selection_hit_test::AnnotationAtHitPosition(&ctx.Dom(),&ctx,owner.SourceController(),has_inner_node,has_frame,&SourcePosition(position).position)),
        Q::NodeDraggable(n)=>V::Boolean(ctx.Layout(&n).is_some_and(|n|n.EditingSelectionStartFacts().2==foundation::EUserDrag::kElement)),
        Q::AdjustBidi(a,b)=> {
            let (a,b)=interaction_default::core::editing::bidi_adjustment::AdjustForRangeSelection(&ctx,&flat(a),&flat(b));
            V::Selection(Some(DOMSelection {anchor:native(&a,TextAffinity::Downstream)?.position,focus:native(&b,TextAffinity::Downstream)?.position,affinity:EditingTextAffinity::Downstream}))
        },
        Q::SelectionFocus {selection,focused}=> V::Boolean(SelectionFocus(ctx, selection, focused)),
        Q::EditingBoundary{anchor,target,hit,local_point,options}=> {
            let hit=MouseSelectionHit{target,position:flat(hit),local_point,options};
            let position=interaction_default::core::editing::selection_hit_test::PositionRespectingEditingBoundary(&ctx,&downstream(anchor).position,&hit);
            V::Position(native(&position.position,position.affinity))
        },
        Q::Equivalent(a,b)=> V::Boolean(downstream(a).position.IsEquivalent(&ctx,&downstream(b).position)),
        Q::ParentAnchored(p)=> { let p=flat(p); V::Position(native(&p.position.ParentAnchoredEquivalent(&ctx),p.affinity)) },
        Q::FirstInNode(n)=>V::Position(native(&Position::FirstPositionInNode(n),TextAffinity::Downstream)),
        Q::LastInNode(n)=>V::Position(native(&Position::LastPositionInNode(&ctx,&n),TextAffinity::Downstream)),
        Q::EnclosingBlock(n)=>V::Node(interaction_default::core::editing::position_units_paragraph::ParagraphContext::EnclosingBlock(&ctx,&Position::FirstPositionInOrBeforeNode(&ctx,&n),Rule::CanCross)),
        Q::RootEditableHasChildren(n)=>V::OptionalBoolean(EditableBoundaryContext::RootEditableElementOf(&ctx,&Position::FirstPositionInOrBeforeNode(&ctx,&n)).map(|root| PositionTree::ChildCount(&ctx.Dom(),&root)>0)),
        Q::Expand{selection,granularity}=> {
            let(a,f,affinity)=super::selection_geometry::FlatSelection(&ctx,selection);
            V::Selection(super::selection_geometry::DomSelection(&ctx, visible_selection::ComputeVisibleSelectionInFlatTree(&ctx,&a,&f,affinity,granularity,WordInclusion::Default)))
        },
        Q::Start(p,g)=> { let p=flat(p); V::Position(native(&selection_adjuster::ComputeStartRespectingGranularity(&ctx,&p,g,WordInclusion::Default),p.affinity)) },
        Q::End(start,p,g)=> { let start=flat(start);let p=flat(p); V::Position(native(&selection_adjuster::ComputeEndRespectingGranularity(&ctx,&start.position,&p,g,WordInclusion::Default),p.affinity)) },
        Q::Previous(p)=> { let p=flat(p); let p=visible_navigation::PreviousPositionOf(&ctx,&p.position,Rule::CanCross);V::Position(native(p.DeepEquivalent(),p.ToPositionWithAffinity().affinity)) },
        Q::IteratorStart(p)=>cursor(PositionIterator::new(&ctx,&flat(p).position)),
        Q::IteratorStep(mut iterator,forward)=> { if forward { iterator.Increment(&ctx) } else { iterator.Decrement(&ctx) };cursor(iterator) },
    })
}

/// FrameSelection::MoveCaretSelection, visible_units.cc:490-505 and
/// frame_selection.cc:231-243. The hit has already borrowed the resident paint
/// layers and queried LayoutObject::PositionForPoint; only source positions
/// cross this synchronous adapter boundary.
pub fn CaretSelectionForContentsHit(
    document: &Document,
    layout: LayoutOutput<'_>,
    selection: DOMSelection,
    target: u64,
    hit: Option<interaction_hit_test::core::layout::hit_test_result::HitTestPositionData>,
    local_point: foundation::PhysicalOffset,
    options: layoutng_assembly::EditingPositionOptions,
    locale: &std::ffi::CStr,
) -> Option<DOMSelection> {
    use interaction_hit_test::core::layout::hit_test_result::{
        HitTestPositionAnchor as A, HitTestTextAffinity as F,
    };
    if hit.is_some_and(|hit| hit.document != document.RootHandle().Identity()) {
        return None;
    }
    let ctx = EditingPositionContext {
        document,
        layout: Some(layout),
        flat: false,
        ui_locale: Some(locale),
        behavior: None,
    };
    let source = |position| {
        SourcePosition(EditingPositionWithAffinity {
            position,
            affinity: selection.affinity,
        })
    };
    let anchor = source(selection.anchor);
    let focus = source(selection.focus);
    let visible = visible_selection::CreateVisibleSelection(
        &ctx,
        &anchor.position,
        &focus.position,
        anchor.affinity,
    );
    let start = if EditableBoundaryContext::ComparePositions(&ctx, &visible.anchor, &visible.focus)
        == Some(Ordering::Greater)
    {
        &visible.focus
    } else {
        &visible.anchor
    };
    let hit = MouseSelectionHit {
        target,
        position: hit
            .and_then(|hit| {
                Some(SourcePosition(EditingPositionWithAffinity {
                    position: EditingPosition::WithAnchor(
                        hit.node_id?,
                        hit.offset,
                        match hit.anchor {
                            A::OffsetInAnchor => EditingPositionAnchorType::OffsetInAnchor,
                            A::BeforeAnchor => EditingPositionAnchorType::BeforeAnchor,
                            A::AfterAnchor => EditingPositionAnchorType::AfterAnchor,
                            A::AfterChildren => EditingPositionAnchorType::AfterChildren,
                        },
                    ),
                    affinity: match hit.affinity {
                        F::Downstream => EditingTextAffinity::Downstream,
                        F::Upstream => EditingTextAffinity::Upstream,
                    },
                }))
            })
            .unwrap_or_default(),
        local_point,
        options,
    };
    let position =
        interaction_default::core::editing::selection_hit_test::PositionRespectingEditingBoundary(
            &ctx, start, &hit,
        );
    let visible = VisiblePosition::Create(&ctx, position);
    let position = visible.ToPositionWithAffinity();
    NativeDOMSelection(&position.position, &position.position, position.affinity)
}

impl interaction_default::core::editing::bidi_adjustment::BidiAdjustmentContext
    for EditingPositionContext<'_>
{
    fn AdjustInlineBidiRange(
        &self,
        a: &PositionWithAffinity<u64>,
        b: &PositionWithAffinity<u64>,
    ) -> interaction_default::core::editing::bidi_adjustment::BidiRangeAdjustment<u64> {
        use interaction_default::core::editing::bidi_adjustment::BidiRangeAdjustment as R;
        use layoutng_assembly::layout_engine::EditingBidiRangeAdjustment as N;
        let dom = |p: &PositionWithAffinity<u64>| {
            NativePosition(
                &position_conversion::ToPositionInDOMTree(&self.Dom(), self, &p.position),
                p.affinity,
            )
        };
        let (Some(a), Some(b)) = (dom(a), dom(b)) else {
            return R::Unchanged;
        };
        let native = self.layout.unwrap().AdjustBidiRange(
            &a,
            &b,
            layoutng_assembly::EditingPositionOptions {
                bidi_caret_affinity: self.BidiCaretAffinityEnabled(),
                ..Default::default()
            },
        );
        let flat = |p| {
            position_conversion::ToPositionInFlatTree(
                &self.Dom(),
                self,
                &SourcePosition(EditingPositionWithAffinity {
                    position: p,
                    affinity: EditingTextAffinity::Downstream,
                })
                .position,
            )
        };
        match native {
            N::Unchanged => R::Unchanged,
            N::AdjustedBase(p) => R::AdjustedBase(flat(p)),
            N::AdjustedExtent(p) => R::AdjustedExtent(flat(p)),
        }
    }
}
struct MouseSelectionHit {
    target: u64,
    position: PositionWithAffinity<u64>,
    local_point: foundation::PhysicalOffset,
    options: layoutng_assembly::EditingPositionOptions,
}
impl
    interaction_default::core::editing::selection_hit_test::SelectionHitTestContext<
        MouseSelectionHit,
    > for EditingPositionContext<'_>
{
    fn TargetNode(&self, h: &MouseSelectionHit) -> Option<u64> {
        Some(h.target)
    }
    fn HasLayoutObject(&self, n: &u64) -> bool {
        self.Layout(n).is_some()
    }
    fn IsSelectable(&self, n: &u64) -> bool {
        self.Layout(n).is_some_and(|n| n.EditingIsSelectable())
    }
    fn TextControlForPosition(&self, p: &Position<u64>) -> Option<u64> {
        text_offset_mapping::EnclosingTextControl(
            &self.Dom(),
            &position_conversion::ToPositionInDOMTree(&self.Dom(), self, p),
        )
    }
    fn PlaceholderElement(&self, n: &u64) -> Option<u64> {
        self.document
            .TextControlPlaceholderElement(self.Index(n)?)
            .map(|i| self.document.Node(i).Id())
    }
    fn InnerEditorElement(&self, n: &u64) -> Option<u64> {
        self.document
            .TextControlInnerEditorElement(self.Index(n)?)
            .map(|i| self.document.Node(i).Id())
    }
    fn HaveSameOutOfFlowAncestor(&self, a: &u64, b: &u64) -> bool {
        self.layout
            .unwrap()
            .EditingHaveSameOutOfFlowAncestor(*a, *b)
    }
    fn VisibleToHitTesting(&self, n: &u64) -> bool {
        self.Layout(n)
            .is_some_and(|n| n.EditingVisibleToHitTesting())
    }
    fn HitPosition(&self, h: &MouseSelectionHit) -> PositionWithAffinity<u64> {
        h.position.clone()
    }
    fn PositionForPointInContainBoundary(
        &self,
        h: &MouseSelectionHit,
        target: &u64,
        editable: &u64,
    ) -> PositionWithAffinity<u64> {
        self.layout
            .unwrap()
            .EditingPositionForPointInContainBoundary(*target, *editable, &h.local_point, h.options)
            .map(|p| {
                let p = SourcePosition(p);
                PositionWithAffinity {
                    position: if self.flat {
                        position_conversion::ToPositionInFlatTree(&self.Dom(), self, &p.position)
                    } else {
                        p.position
                    },
                    affinity: p.affinity,
                }
            })
            .unwrap_or_default()
    }
    fn NoExtendSelectionToUserSelectNoneOutOfFlowEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::NoExtendSelectionToUserSelectNoneOutOfFlowEnabled()
    }
    fn NoExtendSelectionToUserSelectNoneOutOfFlowUnlessEditableEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::NoExtendSelectionToUserSelectNoneOutOfFlowUnlessEditableEnabled()
    }
}
struct MouseSelectionFocus<'a> {
    caret: CaretSelectionFocus<'a>,
    selection: DOMSelection,
    focused: Option<u64>,
}
impl interaction_default::core::editing::selection_focus::SelectionFocusDOM
    for MouseSelectionFocus<'_>
{
    type Node = u64;
    type Scope = u64;
    type Position = Position<u64>;
    fn VisibleFlatSelection(
        &self,
    ) -> Option<
        interaction_default::core::editing::selection_focus::SelectionFocusRange<Position<u64>>,
    > {
        let (a, f, affinity) =
            super::selection_geometry::FlatSelection(&self.caret.context, self.selection);
        let s=interaction_default::core::editing::visible_selection::CreateVisibleCharacterSelectionInFlatTree(&self.caret.context.Flat(),&a,&f,affinity);
        let (start, end) = if EditableBoundaryContext::ComparePositions(
            &self.caret.context.Flat(),
            &s.anchor,
            &s.focus,
        ) == Some(Ordering::Greater)
        {
            (s.focus, s.anchor)
        } else {
            (s.anchor, s.focus)
        };
        (!start.IsNull()).then_some(
            interaction_default::core::editing::selection_focus::SelectionFocusRange { start, end },
        )
    }
    fn FocusedElement(&self) -> Option<u64> {
        self.focused
    }
    fn UpdateStyleAndLayoutForSelection(&self) {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::UpdateStyleAndLayoutForSelection(&self.caret)
    }
    fn ContainerNode(&self, position: &Self::Position) -> Option<Self::Node> {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::ContainerNode(
            &self.caret,
            position,
        )
    }
    fn DocumentElement(&self) -> Option<Self::Node> {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::DocumentElement(
            &self.caret,
        )
    }
    fn IsScrollControlOrMarkerGroupPseudo(&self, node: Self::Node) -> bool {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::IsScrollControlOrMarkerGroupPseudo(&self.caret ,node)
    }
    fn IsTextControl(&self, node: Self::Node) -> bool {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::IsTextControl(
            &self.caret,
            node,
        )
    }
    fn ContainsIncludingHostElements(&self, ancestor: Self::Node, node: Self::Node) -> bool {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::ContainsIncludingHostElements(&self.caret ,ancestor,node)
    }
    fn FirstPositionInNode(&self, node: Self::Node) -> Self::Position {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::FirstPositionInNode(
            &self.caret,
            node,
        )
    }
    fn ComparePositions(
        &self,
        first: &Self::Position,
        second: &Self::Position,
    ) -> Option<Ordering> {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::ComparePositions(
            &self.caret,
            first,
            second,
        )
    }
    fn SelectionAndFocusedVisiblePositionMatchEnabled(&self) -> bool {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::SelectionAndFocusedVisiblePositionMatchEnabled(&self.caret)
    }
    fn IsFocusable(&self, node: Self::Node) -> bool {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::IsFocusable(
            &self.caret,
            node,
        )
    }
    fn IsEditable(&self, node: Self::Node) -> bool {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::IsEditable(
            &self.caret,
            node,
        )
    }
    fn VisiblePositionDeepEquivalent(&self, position: &Self::Position) -> Option<Self::Position> {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::VisiblePositionDeepEquivalent(&self.caret ,position)
    }
    fn TreeScope(&self, node: Self::Node) -> Self::Scope {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::TreeScope(
            &self.caret,
            node,
        )
    }
    fn IsElement(&self, node: Self::Node) -> bool {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::IsElement(
            &self.caret,
            node,
        )
    }
    fn DelegatesFocus(&self, node: Self::Node) -> bool {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::DelegatesFocus(
            &self.caret,
            node,
        )
    }
    fn GetFocusableArea(&self, node: Self::Node) -> Option<Self::Node> {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::GetFocusableArea(
            &self.caret,
            node,
        )
    }
    fn ParentIsShadowRoot(&self, node: Self::Node) -> bool {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::ParentIsShadowRoot(
            &self.caret,
            node,
        )
    }
    fn FlatTreeParent(&self, node: Self::Node) -> Option<Self::Node> {
        interaction_default::core::editing::selection_focus::SelectionFocusDOM::FlatTreeParent(
            &self.caret,
            node,
        )
    }
}

struct MouseSpellCheckingContext<'a> {
    context: EditingPositionContext<'a>,
    focused: Option<u64>,
}
impl interaction_default::core::editing::spellcheck::spell_checker::SpellCheckingContext
    for MouseSpellCheckingContext<'_>
{
    fn EnclosingTextControl(&self, p: &Position<u64>) -> Option<u64> {
        text_offset_mapping::EnclosingTextControl(&self.context, p)
    }
    fn IsInput(&self, n: &u64) -> bool {
        self.context
            .Index(n)
            .is_some_and(|i| self.context.document.Node(i).IsHTMLElement("input"))
    }
    fn IsFocusedElementInDocument(&self, n: &u64) -> bool {
        self.focused == Some(*n)
    }
    fn IsHTMLElement(&self, n: &u64) -> bool {
        self.context.Index(n).is_some_and(|i| {
            let n = self.context.document.Node(i);
            n.Type() == DOMNodeType::kElement && n.Namespace() == DOMNamespace::kHTML
        })
    }
    fn IsSpellCheckingEnabled(&self, n: &u64, page_default: bool) -> bool {
        self.context.Index(n).is_some_and(|i| {
            self.context
                .document
                .IsSpellCheckingEnabledForElement(i, Some(page_default))
        })
    }
    fn IsEditable(&self, n: &u64) -> bool {
        self.context.IsEditableNodeValue(n)
    }
}
impl PositionTree for MouseSpellCheckingContext<'_> {
    type Node = u64;
    fn Parent(&self, node: &Self::Node) -> Option<Self::Node> {
        PositionTree::Parent(&self.context, node)
    }
    fn ChildAt(&self, node: &Self::Node, index: usize) -> Option<Self::Node> {
        PositionTree::ChildAt(&self.context, node, index)
    }
    fn ChildCount(&self, node: &Self::Node) -> usize {
        PositionTree::ChildCount(&self.context, node)
    }
    fn NodeIndex(&self, node: &Self::Node) -> usize {
        PositionTree::NodeIndex(&self.context, node)
    }
    fn PreviousSibling(&self, node: &Self::Node) -> Option<Self::Node> {
        PositionTree::PreviousSibling(&self.context, node)
    }
    fn NextSibling(&self, node: &Self::Node) -> Option<Self::Node> {
        PositionTree::NextSibling(&self.context, node)
    }
    fn IsText(&self, node: &Self::Node) -> bool {
        PositionTree::IsText(&self.context, node)
    }
    fn CharacterDataLength(&self, node: &Self::Node) -> Option<usize> {
        PositionTree::CharacterDataLength(&self.context, node)
    }
}
impl PositionDOM for MouseSpellCheckingContext<'_> {
    fn EditingIgnoresContent(&self, node: &Self::Node) -> bool {
        PositionDOM::EditingIgnoresContent(&self.context, node)
    }
    fn IsDisplayInsideTable(&self, node: &Self::Node) -> bool {
        PositionDOM::IsDisplayInsideTable(&self.context, node)
    }
    fn IsConnected(&self, node: &Self::Node) -> bool {
        PositionDOM::IsConnected(&self.context, node)
    }
    fn DocumentIdentity(&self, node: &Self::Node) -> u128 {
        PositionDOM::DocumentIdentity(&self.context, node)
    }
    fn LastDOMOffsetForEditing(&self, n: &u64) -> usize {
        PositionDOM::LastDOMOffsetForEditing(&self.context, n)
    }
    fn LastOffsetForEditing(&self, n: &u64) -> usize {
        PositionDOM::LastOffsetForEditing(&self.context, n)
    }
}

impl PositionTreeScope for MouseSpellCheckingContext<'_> {
    fn IsShadowRoot(&self, node: &Self::Node) -> bool {
        PositionTreeScope::IsShadowRoot(&self.context, node)
    }
    fn ShadowHost(&self, root: &Self::Node) -> Option<Self::Node> {
        PositionTreeScope::ShadowHost(&self.context, root)
    }
    fn TreeScopeRoot(&self, node: &Self::Node) -> Self::Node {
        PositionTreeScope::TreeScopeRoot(&self.context, node)
    }
    fn IsShadowHost(&self, node: &Self::Node) -> bool {
        PositionTreeScope::IsShadowHost(&self.context, node)
    }
}

pub(super) fn QuerySelectionFocus(
    document: &Document,
    layout: LayoutOutput<'_>,
    selection: Option<DOMSelection>,
    focused: Option<u64>,
) -> bool {
    SelectionFocus(
        EditingPositionContext {
            document,
            layout: Some(layout),
            flat: true,
            ui_locale: None,
            behavior: None,
        },
        selection,
        focused,
    )
}
fn SelectionFocus(
    ctx: EditingPositionContext<'_>,
    selection: Option<DOMSelection>,
    focused: Option<u64>,
) -> bool {
    let document = ctx.document;
    use interaction_default::core::editing::selection_focus;
    selection.is_some_and(|selection| {
        let fallback = focused
            .or_else(|| document.DocumentElement().map(|i| document.Node(i).Id()))
            .unwrap_or(0);
        selection_focus::SelectionHasFocus(&MouseSelectionFocus {
            caret: CaretSelectionFocus {
                context: ctx,
                position: SourcePosition(EditingPositionWithAffinity {
                    position: selection.anchor,
                    affinity: selection.affinity,
                })
                .position,
                affinity: SourcePosition(EditingPositionWithAffinity {
                    position: selection.anchor,
                    affinity: selection.affinity,
                })
                .affinity,
                focused: fallback,
            },
            selection,
            focused,
        })
    })
}
