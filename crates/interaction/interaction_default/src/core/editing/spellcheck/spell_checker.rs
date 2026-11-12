// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source core/editing/spellcheck/spell_checker.cc:740-752.
use crate::core::editing::position::{Position, PositionDOM};
use crate::core::editing::visible_position::DownstreamVisiblePositionContext;
pub trait SpellCheckingContext: PositionDOM {
    fn EnclosingTextControl(&self, position: &Position<Self::Node>) -> Option<Self::Node>;
    fn IsInput(&self, node: &Self::Node) -> bool;
    fn IsFocusedElementInDocument(&self, node: &Self::Node) -> bool;
    fn IsHTMLElement(&self, node: &Self::Node) -> bool;
    fn IsSpellCheckingEnabled(&self, node: &Self::Node, page_default: bool) -> bool;
    fn IsEditable(&self, node: &Self::Node) -> bool;
}
pub fn IsSpellCheckingEnabledAt<C: SpellCheckingContext>(
    ctx: &C,
    position: &Position<C::Node>,
    page_default: bool,
) -> bool {
    let Some(anchor) = position.AnchorNode() else {
        return false;
    };
    if let Some(control) = ctx.EnclosingTextControl(position) {
        if ctx.IsInput(&control) && !ctx.IsFocusedElementInDocument(&control) {
            return false;
        }
    }
    let mut node = Some(anchor.clone());
    while let Some(current) = node {
        if ctx.IsHTMLElement(&current) {
            return ctx.IsSpellCheckingEnabled(&current, page_default) && ctx.IsEditable(&current);
        }
        node = ctx.Parent(&current);
    }
    false
}

/// Source spell_checker.cc:77-79,428-468. Despite its historical name this
/// operation reads the original marker group and DOM PlainText synchronously;
/// the platform text checker is consulted separately for missing suggestions.
pub fn SelectMisspellingAsync<D, F>(
    dom: &D,
    flat: &F,
    markers: &super::super::markers::document_marker_controller::DocumentMarkerController<D::Node>,
    mut compute_selection: impl FnMut() -> super::super::visible_selection::VisibleSelection<D::Node>,
) -> (Vec<u16>, Vec<u16>)
where
    D: super::super::iterators::text_iterator::TextIteratorContext
        + super::super::visible_position::VisiblePositionContext
        + super::super::editing_boundaries::EditableBoundaryContext
        + super::super::position_conversion::PositionTreeScope,
    D::Node: Eq + std::hash::Hash + 'static,
    F: super::super::iterators::text_iterator::TextIteratorContext<Node = D::Node>
        + super::super::text_offset_mapping::InlineContentsContext
        + super::super::position_units_word::WordBreakContext
        + super::super::editing_boundaries::EditableBoundaryContext,
{
    use super::super::{
        compare_positions::CompareDOMPositions,
        iterators::{text_iterator::PlainText, text_iterator_behavior::TextIteratorBehavior},
        markers::document_marker_controller::MarkerTypes,
        visible_selection::NormalizeSelectionRange,
    };
    (|| {
        let selection = compute_selection();
        if selection.anchor.IsNull() {
            return None;
        }
        let (start, end) = if CompareDOMPositions(dom, &selection.anchor, &selection.focus)?
            == std::cmp::Ordering::Greater
        {
            (&selection.focus, &selection.anchor)
        } else {
            (&selection.anchor, &selection.focus)
        };
        let (start, end) = (
            start.ParentAnchoredEquivalent(dom),
            end.ParentAnchoredEquivalent(dom),
        );
        let group = markers.FirstMarkerGroupIntersectingEphemeralRange(
            dom,
            flat,
            &start,
            &end,
            MarkerTypes::Misspelling(),
        )?;
        let selection = compute_selection();
        let (start, end) = NormalizeSelectionRange(dom, &selection.anchor, &selection.focus)?;
        let (marker_start, marker_end) = group.Bounds(dom)?;
        let marked_text = PlainText(
            dom,
            &marker_start,
            &marker_end,
            TextIteratorBehavior::default(),
        );
        let selected_text = PlainText(dom, &start, &end, TextIteratorBehavior::default());
        let strip = |text: &[u16]| {
            let predicate = |c: &u16| {
                foundation::unicode::IsSpaceOrNewline(*c) || foundation::unicode::IsPunct(*c as i32)
            };
            let first = text
                .iter()
                .position(|c| !predicate(c))
                .unwrap_or(text.len());
            let end = text
                .iter()
                .rposition(|c| !predicate(c))
                .map_or(first, |i| i + 1);
            (first, end)
        };
        let (a, b) = strip(&marked_text);
        let (c, d) = strip(&selected_text);
        if marked_text[a..b] != selected_text[c..d] {
            return None;
        }
        let node = start
            .ComputeContainerNode(dom)
            .expect("normalized marker selection Text");
        assert!(dom.IsText(&node));
        let marker = group
            .GetMarkerForText(&node)
            .expect("selection Text belongs to original spellcheck group");
        Some((marked_text, marker.Description().to_vec()))
    })()
    .unwrap_or_default()
}

/// Source WebTextCheckClient::CheckSpelling (web_text_check_client.h) and
/// context_menu_controller.cc:806-818. The optional native checker is injected
/// as its original owner; lack of a checker is represented by Option, never a
/// fabricated successful spellcheck result.
pub struct SpellingCheckResult {
    pub misspelled_offset: usize,
    pub misspelled_length: usize,
    pub suggestions: Vec<Vec<u16>>,
}
pub trait TextCheckerClient {
    fn CheckSpelling(&self, text: &[u16]) -> SpellingCheckResult;
    // Original optional WebTextCheckClient is shared by ContextMenu and the
    // retained requester. It is the host service, not a spellcheck algorithm.
    fn IsSpellCheckingEnabled(&self) -> bool;
    fn RequestCheckingOfText(
        &self,
        text: std::rc::Rc<[u16]>,
        markers: Vec<super::text_checking::WebSpellingMarker>,
        should_force_refresh: bool,
        completion: std::rc::Rc<dyn super::text_checking::TextCheckingCompletion>,
    );
}

// SpellChecker.cc:400-425. This is separate from the asynchronous requester:
// ending text-field editing removes existing markers from original Text nodes.
pub trait SpellCheckerEditingLifecycleContext: PositionDOM {
    fn TextControlInnerEditor(&self, element: &Self::Node) -> Self::Node;
    fn IsEditableNode(&self, node: &Self::Node) -> bool;
    fn UpdateStyleAndLayoutTreeForSpellCheckElement(&self, element: &Self::Node);
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ElementsType {
    All,
    OnlyNonEditable,
}
pub fn RemoveSpellingAndGrammarMarkers<C>(
    ctx: &C,
    markers: &super::super::markers::document_marker_controller::DocumentMarkerController<C::Node>,
    element: &C::Node,
    elements: ElementsType,
) -> Vec<super::super::markers::document_marker_controller::MarkerEffect<C::Node>>
where
    C: SpellCheckerEditingLifecycleContext,
    C::Node: Eq + std::hash::Hash + 'static,
{
    use super::super::{
        markers::document_marker_controller::MarkerTypes, text_offset_mapping::NextNode,
    };
    if elements == ElementsType::OnlyNonEditable {
        ctx.UpdateStyleAndLayoutTreeForSpellCheckElement(element);
    }
    let mut node = Some(element.clone());
    let mut effects = Vec::new();
    while let Some(current) = node {
        node = NextNode(ctx, &current, Some(element));
        if (elements == ElementsType::All || !ctx.IsEditableNode(&current)) && ctx.IsText(&current)
        {
            effects.extend(markers.RemoveMarkersForNode(&current, MarkerTypes::Misspelling()));
        }
    }
    effects
}
pub fn DidEndEditingOnTextField<C>(
    ctx: &C,
    markers: &super::super::markers::document_marker_controller::DocumentMarkerController<C::Node>,
    element: &C::Node,
) -> Vec<super::super::markers::document_marker_controller::MarkerEffect<C::Node>>
where
    C: SpellCheckerEditingLifecycleContext,
    C::Node: Eq + std::hash::Hash + 'static,
{
    RemoveSpellingAndGrammarMarkers(
        ctx,
        markers,
        &ctx.TextControlInnerEditor(element),
        ElementsType::All,
    )
}
