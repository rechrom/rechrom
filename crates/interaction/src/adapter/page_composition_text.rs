//! PlainTextRange uses the translated TextIterator over live DOM and resident
//! OffsetMapping, including rendered whitespace, BRs and atomic inline objects.
//! Source: editing/plain_text_range.cc:59-205.
use super::*;
use dom::persistent_document::DOMShadowRootMode;
use dom::{DOMAutofillState, DOMStaticRange};
use interaction_default::core::editing::markers::{
    document_marker_controller::{DocumentMarkerController, MarkerEffect, MarkerType, MarkerTypes},
    styleable_marker::StyleableMarker,
};
use interaction_default::core::editing::{
    commands::structural_editing_context::{
        StructuralBooleanQuery, StructuralNodeFacts, StructuralPositionQuery,
    },
    position_units_paragraph::{self, ParagraphContext, ParagraphLayoutFacts, ParagraphTextFacts},
};
use interaction_default::core::editing::{
    ime::input_method_controller::PlainTextRange,
    iterators::{
        text_iterator::{
            self, TextIterator, TextIteratorContext, TextIteratorLayoutFacts, TextIteratorTextRun,
        },
        text_iterator_behavior::TextIteratorBehavior,
    },
};
use layoutng_assembly::editing_state::DocumentCompositionMarker;
use std::rc::Rc;

impl ParagraphContext for EditingPositionContext<'_> {
    fn ParagraphLayout(&self, node: &u64) -> Option<ParagraphLayoutFacts> {
        let facts = self.Layout(node)?.EditingParagraphLayoutFacts();
        Some(ParagraphLayoutFacts {
            visible: facts.visible,
            br: facts.br,
            selectable: facts.selectable,
            user_select_all: facts.user_select_all,
            text: facts.text.map(|text| ParagraphTextFacts {
                transformed_text: text.transformed_text,
                preserve_breaks: text.preserve_breaks,
                resolved_text_length: text.resolved_text_length,
                caret_max_offset: text.caret_max_offset,
                text_start_offset: text.text_start_offset,
            }),
        })
    }
    fn IsRenderedAsNonInlineTableImageOrHR(&self, node: &u64) -> bool {
        self.Layout(node).is_some_and(|node| {
            node.EditingParagraphLayoutFacts()
                .is_rendered_as_non_inline_table_image_or_hr
        })
    }
    fn EnclosingBlock(&self, position: &Position<u64>, rule: Rule) -> Option<u64> {
        let mut node = position.ComputeContainerNode(self);
        while let Some(current) = node {
            if self
                .Layout(&current)
                .is_some_and(|layout| layout.EditingParagraphLayoutFacts().is_enclosing_block)
            {
                return Some(current);
            }
            if rule == Rule::CannotCross && !self.IsEditableNodeValue(&current) {
                return None;
            }
            node = PositionTree::Parent(self, &current);
        }
        None
    }
    fn HighestEditableRoot(&self, position: &Position<u64>) -> Option<u64> {
        EditableBoundaryContext::HighestEditableRoot(self, position)
    }
    fn IsEditable(&self, node: &u64) -> bool {
        self.IsEditableNodeValue(node)
    }
    fn IsEnclosingBlock(&self, node: &u64) -> bool {
        self.Layout(node)
            .is_some_and(|layout| layout.EditingParagraphLayoutFacts().is_enclosing_block)
    }
    fn IsDOMDescendantOf(&self, node: &u64, ancestor: Option<&u64>) -> bool {
        ancestor.is_some_and(|ancestor| self.Dom().IsDescendant(node, ancestor))
    }
}

/// Opaque Interaction-owned source marker controller. The host only routes a
/// typed operation; it cannot inspect or mutate marker storage.
#[derive(Clone)]
pub struct CompositionMarkerOwner(Rc<DocumentMarkerController<u64>>);
impl CompositionMarkerOwner {
    pub(crate) fn SourceController(&self) -> &DocumentMarkerController<u64> {
        &self.0
    }
    pub(crate) fn new(document: u128) -> Self {
        Self(Rc::new(DocumentMarkerController::new(document)))
    }
    pub(crate) fn MoveMarkers(
        &self,
        from: u64,
        length: usize,
        to: u64,
    ) -> (Rc<[DocumentCompositionMarker]>, Vec<u64>) {
        let effects = self.0.MoveMarkers(&from, length, &to);
        let mut ranges: Vec<_> = self
            .0
            .CompositionMarkerRanges()
            .into_iter()
            .map(|(node, start, end)| DocumentCompositionMarker {
                node_id: node,
                start: start.min(u32::MAX as usize) as u32,
                end: end.min(u32::MAX as usize) as u32,
            })
            .collect();
        ranges.sort_unstable_by_key(|marker| (marker.node_id, marker.start, marker.end));
        (
            Rc::from(ranges),
            effects
                .into_iter()
                .filter_map(|effect| match effect {
                    MarkerEffect::InvalidatePaint(node) => Some(node),
                    _ => None,
                })
                .collect(),
        )
    }
}
pub struct CompositionMarkerUpdate {
    owner: CompositionMarkerOwner,
    operation: CompositionMarkerOperation,
}
enum CompositionMarkerOperation {
    ReplaceComposition(Option<DOMStaticRange>),
    RemoveSuggestionsOnFinish(DOMStaticRange),
    RemoveAutocorrect,
    AddImeSpans {
        root: u64,
        offset: usize,
        spans: Vec<interaction_default::core::editing::ime::ime_text_span::ImeTextSpan>,
    },
}
impl CompositionMarkerUpdate {
    pub(crate) fn new(owner: CompositionMarkerOwner, range: Option<DOMStaticRange>) -> Self {
        Self {
            owner,
            operation: CompositionMarkerOperation::ReplaceComposition(range),
        }
    }
    pub(crate) fn RemoveSuggestionsOnFinish(
        owner: CompositionMarkerOwner,
        range: DOMStaticRange,
    ) -> Self {
        Self {
            owner,
            operation: CompositionMarkerOperation::RemoveSuggestionsOnFinish(range),
        }
    }
    pub(crate) fn RemoveAutocorrect(owner: CompositionMarkerOwner) -> Self {
        Self {
            owner,
            operation: CompositionMarkerOperation::RemoveAutocorrect,
        }
    }
    pub(crate) fn AddImeSpans(
        owner: CompositionMarkerOwner,
        root: u64,
        offset: usize,
        spans: Vec<interaction_default::core::editing::ime::ime_text_span::ImeTextSpan>,
    ) -> Self {
        Self {
            owner,
            operation: CompositionMarkerOperation::AddImeSpans {
                root,
                offset,
                spans,
            },
        }
    }
}

impl interaction_default::core::editing::ime::input_method_controller::ImeTextSpanContext
    for EditingPositionContext<'_>
{
    type FlatContext = Self;
    fn FlatContext(&self) -> Self {
        Self {
            document: self.document,
            layout: self.layout,
            flat: true,
            ui_locale: self.ui_locale,
            behavior: self.behavior,
        }
    }
    fn ToFlatTreePosition(&self, flat: &Self, position: &Position<u64>) -> Position<u64> {
        position_conversion::ToPositionInFlatTree(self, flat, position)
    }
    fn ImeSpanRange(
        &self,
        root: &u64,
        start: usize,
        end: usize,
    ) -> Option<(Position<u64>, Position<u64>)> {
        let range = CreateRange(
            self,
            *root,
            PlainTextRange {
                start: start.try_into().ok()?,
                end: end.try_into().ok()?,
            },
        )?;
        Some(RangePositions(range))
    }
    fn SpellCheckingEnabledAt(&self, position: &Position<u64>) -> bool {
        position
            .ComputeContainerNode(self)
            .and_then(|node| self.Index(&node))
            .is_some_and(|index| {
                self.document
                    .IsSpellCheckingEnabledForElement(index, Some(true))
            })
    }
}

impl interaction_default::core::editing::visible_units_line::EndOfLineContext
    for EditingPositionContext<'_>
{
    fn NativeEndOfLine(&self, position: &PositionWithAffinity<u64>) -> PositionWithAffinity<u64> {
        self.NativeCaretPosition(position)
            .and_then(|native| {
                self.Layout(&native.position.node_id?)?
                    .EndOfLine(&native, Default::default())
            })
            .map(|native| self.SourceCaretPosition(native))
            .unwrap_or_default()
    }
}
impl interaction_default::core::editing::visible_units_commands::CommandPositionContext
    for EditingPositionContext<'_>
{
    type CaretQuad = (foundation::gfx::QuadF, bool);
    fn AbsoluteCaretBounds(&self, position: &PositionWithAffinity<u64>) -> foundation::gfx::Rect {
        let Some(native) = NativePosition(&position.position, position.affinity) else {
            return Default::default();
        };
        let Some(node) = position
            .position
            .AnchorNode()
            .and_then(|node| self.Layout(node))
        else {
            return Default::default();
        };
        let (quad, empty) = node.CaretQuadAndEmpty(
            &native,
            Default::default(),
            position
                .position
                .ComputeEditingOffset(self)
                .min(i32::MAX as usize) as i32,
            node.CaretBarWidth(),
        );
        if empty {
            Default::default()
        } else {
            foundation::gfx::ToEnclosingRect(quad.BoundingBox())
        }
    }
    fn IsHorizontalWritingMode(&self, node: &u64) -> bool {
        self.Layout(node).is_some_and(|node| {
            node.EditingCaretFacts().writing_mode == foundation::WritingMode::kHorizontalTb
        })
    }
    fn IsHTMLBR(&self, node: &u64) -> bool {
        self.Index(node)
            .is_some_and(|index| self.document.Node(index).IsHTMLElement("br"))
    }
    fn IsLayoutText(&self, node: &u64) -> bool {
        self.Layout(node)
            .is_some_and(|node| node.EditingCaretFacts().is_text)
    }
    fn ShouldCollapseBreaks(&self, node: &u64) -> bool {
        self.Layout(node)
            .is_some_and(|node| node.EditingParagraphLayoutFacts().collapse_breaks)
    }
    fn TextCodeUnit(&self, node: &u64, offset: usize) -> Option<u16> {
        let index = self.Index(node)?;
        PositionTree::IsText(self, node)
            .then(|| self.document.CharacterDataUTF16(index).get(offset).copied())
            .flatten()
    }
    fn EditingUseDomPositionApiEnabled(&self) -> bool {
        false
    }
    fn IsBeforeNonCollapsedContent(&self, position: &Position<u64>) -> bool {
        position
            .AnchorNode()
            .and_then(|node| self.Layout(node))
            .and_then(|node| {
                node.EditingTextFacts(position.ComputeOffsetInContainerNode(self), false)
            })
            .is_some_and(|facts| facts.is_before_non_collapsed_character)
    }
    fn CaretQuad(&self, position: &Position<u64>) -> Option<Self::CaretQuad> {
        let native = NativePosition(position, TextAffinity::Downstream)?;
        let node = self.Layout(position.AnchorNode()?)?;
        Some(node.CaretQuadAndEmpty(
            &native,
            Default::default(),
            position.ComputeEditingOffset(self).min(i32::MAX as usize) as i32,
            node.CaretBarWidth(),
        ))
    }
    fn IsLayoutBlockFlow(&self, node: &u64) -> bool {
        self.Layout(node)
            .is_some_and(|node| node.EditingIsBlockFlow())
    }
}
pub enum CompositionTextQuery {
    NodeWritingMode(u64),
    EditorSelectionGeometry {query:super::EditorSelectionGeometryQuery,behavior:EditingBehaviorType,ui_locale:std::ffi::CString},
    SelectAllFacts { selection: Option<DOMSelection>, focused: Option<u64> },
    ModifySelection {
        selection: DOMSelection,
        state: interaction_default::core::editing::frame_selection_modify::SelectionModificationState,
        alteration: interaction_default::core::editing::selection_modifier::SelectionModifyAlteration,
        direction: interaction_default::core::editing::commands::editor_command_input_type::SelectionModifyDirection,
        granularity: interaction_default::core::editing::text_granularity::TextGranularity,
        behavior: interaction_default::core::editing::editing_behavior::EditingBehaviorType,
        ui_locale: std::ffi::CString,
    },
    EditorCommandTargetRange {
        selection: Option<DOMSelection>,
        directional: bool,
        extend: Option<(interaction_default::core::editing::commands::editor_command_input_type::SelectionModifyDirection, interaction_default::core::editing::text_granularity::TextGranularity)>,
        behavior: interaction_default::core::editing::editing_behavior::EditingBehaviorType,
        ui_locale: std::ffi::CString,
    },
    TypingDeletion {
        query: super::typing_deletion_geometry::TypingDeletionGeometryQuery,
        behavior: interaction_default::core::editing::editing_behavior::EditingBehaviorType,
        ui_locale: std::ffi::CString,
    },
    ApplyStyleGeometry(super::apply_style_geometry::ApplyStyleGeometryQuery),
    VisiblePositionIndex(EditingPosition),
    MovingParagraphRangeLength { start: EditingPosition, end: EditingPosition },
    MovingParagraphPositionForIndex(usize),
    PositionForVisibleIndex { scope: u64, index: usize },
    SelectionFocus {
        selection: Option<DOMSelection>,
        focused: Option<u64>,
    },
    ReplacementCanHaveChildren(u64),
    SimplifyMarkupInline(u64),
    ReplacementVisibleText(u64),
    ReplacementRawLineBreak(EditingPosition),
    ReplacementListItemNumber(u64),
    ReplacementBlockPlaceholder(u64),
    ReplacementMoveGrapheme {
        position: EditingPosition,
        forward: bool,
    },
    ReplacementBoundaryCodePoint {
        position: EditingPositionWithAffinity,
        before: bool,
    },
    ReplacementTestRenderingText(u64),
    ReplacementBoundaryCharacter {
        position: EditingPositionWithAffinity,
        before: bool,
    },
    SelectedTextForClipboard {
        selection: DOMSelection,
        include_image_alt_text: bool,
    },
    FragmentFromTextContext(DOMSelection),
    ClipboardSelection {
        selection: Option<DOMSelection>,
        focused: Option<u64>,
        has_focus: bool,
    },
    PotentialSuggestionTap {
        owner: CompositionMarkerOwner,
        position: EditingPositionWithAffinity,
    },
    NonLocatedGeometry(super::nonlocated_context_menu_geometry::NonLocatedGeometryQuery),
    MouseSelectionGeometry {
        query: super::mouse_selection_geometry::MouseSelectionGeometryQuery,
        ui_locale: std::ffi::CString,
    },
    SelectionGeometry {
        query: super::selection_geometry::SelectionGeometryQuery,
        ui_locale: std::ffi::CString,
    },
    CanonicalSelection(DOMSelection),
    StructuralBoolean {
        position: EditingPosition,
        other: Option<EditingPosition>,
        query: StructuralBooleanQuery,
    },
    StructuralPosition {
        position: EditingPositionWithAffinity,
        query: StructuralPositionQuery,
    },
    StructuralBlock(EditingPosition),
    StructuralNode(u64),
    Markers(CompositionMarkerUpdate),
    SelectionRange(DOMSelection),
    AnchorFirst(DOMSelection),
    Offsets {
        root: u64,
        range: DOMStaticRange,
    },
    Range {
        root: u64,
        offsets: PlainTextRange,
    },
    Text {
        range: DOMStaticRange,
        original: bool,
    },
    Length {
        root: u64,
    },
}
pub enum CompositionTextResult {
    NodeWritingMode(Option<foundation::WritingMode>),
    EditorSelectionGeometry(super::EditorSelectionGeometryResult),
    SelectAllFacts(interaction_default::core::editing::frame_selection_select_all::SelectAllFacts),
    ModifiedSelection(interaction_default::core::editing::frame_selection_modify::PreparedSelectionModification<super::super::page_frame_selection::FrameSelectionValue>),
    EditorCommandTargetRange(Option<DOMStaticRange>),
    TypingDeletion(super::typing_deletion_geometry::TypingDeletionGeometryResult),
    ApplyStyleGeometry(super::apply_style_geometry::ApplyStyleGeometryResult),
    VisiblePositionIndex(Option<(u64, usize)>),
    SelectionFocus(bool),
    ReplacementCanHaveChildren(bool),
    SimplifyMarkupInline(Option<bool>),
    ReplacementVisibleText(bool),
    ReplacementListItemNumber(Option<i32>),
    ReplacementBlockPlaceholder(interaction_default::core::editing::commands::break_blockquote_command::BlockPlaceholderFacts),
    ReplacementBoundaryCodePoint(u32),
    ReplacementBoundaryCharacter(u16),
    FragmentFromTextContext(
        interaction_default::core::editing::serializers::serialization::TextFragmentContext<
            u64,
        >,
    ),
    ClipboardSelection {
        target: Option<u64>,
        richly_editable: bool,
    },
    PotentialSuggestionTap(bool),
    NonLocatedGeometry(super::nonlocated_context_menu_geometry::NonLocatedGeometryResult),
    MouseSelectionGeometry(super::mouse_selection_geometry::MouseSelectionGeometryResult),
    SelectionGeometry(super::selection_geometry::SelectionGeometryResult),
    CanonicalSelection(DOMSelection),
    StructuralBoolean(bool),
    StructuralPosition(EditingPositionWithAffinity),
    StructuralBlock(Option<u64>),
    StructuralNode(StructuralNodeFacts),
    AnchorFirst(bool),
    Markers {
        ranges: Rc<[DocumentCompositionMarker]>,
        invalidated: Vec<u64>,
    },
    Range(DOMStaticRange),
    Offsets(PlainTextRange),
    Text(Vec<u16>),
    Length(usize),
}

impl<'a> TextIteratorContext for EditingPositionContext<'a> {
    type VisibleContext = Self;
    type TextCursor = layoutng_assembly::layout_engine::EditingTextRunCursor<'a>;
    fn VisibleContext(&self) -> &Self {
        self
    }
    fn AssertTextIteratorLifecyclePrepared(&self) {
        assert!(self.layout.unwrap().FragmentRoot().is_some());
    }
    fn IsFlatTreeStrategy(&self) -> bool {
        self.flat
    }
    fn OpenShadowRoot(&self, node: &u64) -> Option<u64> {
        let root = self.document.ShadowRoot(self.Index(node)?)?;
        (self.document.ShadowRootMode(root) == Some(DOMShadowRootMode::kOpen))
            .then(|| self.document.Node(root).Id())
    }
    fn UserAgentShadowRoot(&self, node: &u64) -> Option<u64> {
        let root = self.document.ShadowRoot(self.Index(node)?)?;
        (self.document.ShadowRootMode(root) == Some(DOMShadowRootMode::kUserAgent))
            .then(|| self.document.Node(root).Id())
    }
    fn IsOpenShadowRoot(&self, node: &u64) -> bool {
        self.Index(node)
            .is_some_and(|i| self.document.ShadowRootMode(i) == Some(DOMShadowRootMode::kOpen))
    }
    fn TextTreeScopeRoot(&self, node: &u64) -> u64 {
        PositionTreeScope::TreeScopeRoot(self, node)
    }
    fn IsAtomicInline(&self, node: &u64) -> bool {
        self.Layout(node).is_some_and(|n| n.EditingIsAtomicInline())
    }
    fn CountShadowTreeUse(&self, _selection_to_string: bool, _window_find: bool) {}
    fn LayoutFacts(&self, node: &u64) -> Option<TextIteratorLayoutFacts> {
        let facts = self.Layout(node)?.EditingTextIteratorObjectFacts();
        Some(TextIteratorLayoutFacts {
            visible: facts.visible,
            selectable: facts.selectable,
            is_text: facts.is_text,
            is_box: facts.is_box,
            is_scroll_container: facts.is_scroll_container,
            is_layout_view: facts.is_layout_view,
            size_empty: facts.size_empty,
            out_of_flow: facts.out_of_flow,
            image: facts.image,
            embedded: facts.embedded,
            text_control: facts.text_control,
            br: facts.br,
            hr: facts.hr,
            table: facts.table,
            table_cell: facts.table_cell,
            table_row: facts.table_row,
            inline: facts.inline,
            block: facts.block,
            block_flow: facts.block_flow,
            floating_or_out_of_flow: facts.floating_or_out_of_flow,
            body: facts.body,
            has_height: facts.has_height,
            table_exists: facts.table_exists,
            table_first_cell: facts.table_first_cell,
            table_inline: facts.table_inline,
        })
    }
    fn IsHTMLElement(&self, node: &u64, tag: &str) -> bool {
        self.Index(node)
            .is_some_and(|i| self.document.Node(i).IsHTMLElement(tag))
    }
    fn HasDisplayContents(&self, node: &u64) -> bool {
        self.Index(node)
            .and_then(|i| self.document.ResolvedStyleFor(i))
            .and_then(|style| style.ComputedStyle())
            .is_some_and(|style| style.Display() == foundation::EDisplay::kContents)
    }
    fn IsShadowRoot(&self, node: &u64) -> bool {
        PositionTreeScope::IsShadowRoot(self, node)
    }
    fn ShadowHost(&self, node: &u64) -> Option<u64> {
        PositionTreeScope::ShadowHost(self, node)
    }
    fn LockedInclusiveAncestorPreventingLayout(&self, node: &u64) -> bool {
        let Some(mut index) = self.Index(node) else {
            return false;
        };
        loop {
            if self
                .document
                .DisplayLockContentState(index)
                .is_some_and(|state| state.is_locked && !state.should_layout_children)
            {
                return true;
            }
            let Some(parent) = self.document.FlatTreeParent(index) else {
                return false;
            };
            index = parent;
        }
    }
    fn DisplayLockAllowsSelectionTraversal(&self, node: &u64) -> bool {
        self.Index(node)
            .and_then(|i| self.document.DisplayLockContentState(i))
            .is_none_or(|state| !state.is_locked || state.should_layout_children)
    }
    fn IsFormControl(&self, node: &u64) -> bool {
        self.Index(node)
            .is_some_and(|i| self.document.FormControlClass(i).is_some())
    }
    fn IsTextControl(&self, node: &u64) -> bool {
        self.Index(node).is_some_and(|i| {
            matches!(
                self.document.FormControlClass(i),
                Some(DOMFormControlClass::Input | DOMFormControlClass::TextArea)
            )
        })
    }
    fn EnclosingAutofilledOrPreviewedTextControl(&self, node: &u64) -> bool {
        text_offset_mapping::EnclosingTextControl(self, &Position::new(*node, 0))
            .and_then(|n| self.Index(&n))
            .is_some_and(|i| self.document.AutofillState(i) != DOMAutofillState::kNotFilled)
    }
    fn AltText(&self, node: &u64) -> foundation::String {
        self.Index(node)
            .and_then(|i| self.document.Node(i).FindAttribute("alt"))
            .map_or_else(foundation::String::default, |a| {
                foundation::String::FromUtf8(a.value.as_bytes())
            })
    }
    fn IsImageInput(&self, node: &u64) -> bool {
        self.Index(node)
            .is_some_and(|i| self.document.InputType(i) == Some(dom::DOMInputType::kImage))
    }
    fn CreateTextCursor(
        &self,
        node: &u64,
        start: usize,
        end: usize,
        behavior: TextIteratorBehavior,
    ) -> Option<Self::TextCursor> {
        self.Layout(node)?.EditingTextRuns(
            start.try_into().ok()?,
            end.min(u32::MAX as usize) as u32,
            layoutng_assembly::layout_engine::EditingTextIteratorBehavior {
                ignores_style_visibility: behavior.IgnoresStyleVisibility(),
                ignores_css_text_transforms: behavior.IgnoresCssTextTransforms(),
                emits_original_text: behavior.EmitsOriginalText(),
                emits_space_for_nbsp: behavior.EmitsSpaceForNbsp(),
            },
        )
    }
    fn NextTextRun(&self, cursor: &mut Self::TextCursor) -> Option<TextIteratorTextRun> {
        let run = cursor.next()?;
        Some(TextIteratorTextRun {
            dom_start: run.dom_start as usize,
            dom_end: run.dom_end as usize,
            text: run.text,
            text_start: run.text_start as usize,
            text_end: run.text_end as usize,
            is_text_security: run.is_text_security,
        })
    }
}
fn Boundary(
    context: &EditingPositionContext<'_>,
    position: &Position<u64>,
) -> Option<DOMBoundaryPoint> {
    Some(DOMBoundaryPoint {
        node_id: position.ComputeContainerNode(context)?,
        offset: position.ComputeOffsetInContainerNode(context),
    })
}
fn RangePositions(range: DOMStaticRange) -> (Position<u64>, Position<u64>) {
    (
        Position::new(range.start.node_id, range.start.offset),
        Position::new(range.end.node_id, range.end.offset),
    )
}
fn PositionInTextRun(
    context: &EditingPositionContext<'_>,
    offset: usize,
    start: &Position<u64>,
    end: &Position<u64>,
) -> Position<u64> {
    if start
        .ComputeContainerNode(context)
        .is_some_and(|node| context.IsText(&node))
    {
        Position::new(
            start.ComputeContainerNode(context).unwrap(),
            start.ComputeOffsetInContainerNode(context) + offset,
        )
    } else if offset == 0 {
        start.clone()
    } else {
        end.clone()
    }
}
fn CreateRange(
    context: &EditingPositionContext<'_>,
    root: u64,
    offsets: PlainTextRange,
) -> Option<DOMStaticRange> {
    CreateRangeWithBehavior(
        context,
        root,
        offsets,
        TextIteratorBehavior::EmitsObjectReplacementCharacterBehavior(),
    )
}
fn CreateRangeWithBehavior(
    context: &EditingPositionContext<'_>,
    root: u64,
    offsets: PlainTextRange,
    behavior: TextIteratorBehavior,
) -> Option<DOMStaticRange> {
    if offsets.start < 0 || offsets.end < offsets.start {
        return None;
    }
    let start = Position::FirstPositionInNode(root);
    let end = Position::LastPositionInNode(context, &root);
    let mut iterator = TextIterator::new(context, &start, &end, behavior);
    if offsets.start == 0 && offsets.end == 0 && iterator.AtEnd() {
        let point = DOMBoundaryPoint {
            node_id: iterator.CurrentContainer(),
            offset: 0,
        };
        return Some(DOMStaticRange {
            start: point,
            end: point,
        });
    }
    let mut count = 0;
    let mut result_start = None;
    let mut last_end = None;
    while !iterator.AtEnd() {
        let len = iterator.length();
        let run_start = iterator
            .StartPositionInCurrentContainer()
            .ToOffsetInAnchor(context);
        let mut run_end = iterator
            .EndPositionInCurrentContainer()
            .ToOffsetInAnchor(context);
        let found_start =
            (offsets.start as usize) >= count && (offsets.start as usize) <= count + len;
        let found_end = (offsets.end as usize) >= count && (offsets.end as usize) <= count + len;
        let special = found_end
            && len == 1
            && (matches!(iterator.CharacterAt(0), 10 | 9)
                || iterator.IsInsideAtomicInlineElement());
        if special {
            iterator.Advance();
            if !iterator.AtEnd() {
                run_end = iterator.StartPositionInCurrentContainer();
            } else {
                let visible = VisiblePosition::Create(
                    context,
                    PositionWithAffinity {
                        position: run_start.clone(),
                        affinity: TextAffinity::Downstream,
                    },
                );
                let next = visible_navigation::NextPositionOf(
                    context,
                    &visible.ToPositionWithAffinity(),
                    Rule::CanCross,
                );
                if !next.IsNull() {
                    run_end = next.DeepEquivalent().clone();
                }
            }
        }
        if found_start {
            result_start = Some(PositionInTextRun(
                context,
                offsets.start as usize - count,
                &run_start,
                &run_end,
            ));
        }
        if found_end {
            let result_end =
                PositionInTextRun(context, offsets.end as usize - count, &run_start, &run_end);
            return Some(DOMStaticRange {
                start: Boundary(context, &result_start?)?,
                end: Boundary(context, &result_end)?,
            });
        }
        count += len;
        last_end = Some(run_end);
        iterator.Advance();
    }
    Some(DOMStaticRange {
        start: Boundary(context, &result_start?)?,
        end: Boundary(context, &last_end?)?,
    })
}
pub fn QueryCompositionText(
    document: &Document,
    layout: LayoutOutput<'_>,
    query: CompositionTextQuery,
) -> Option<CompositionTextResult> {
    let _lifecycle = document.Lifecycle().DisallowTransition();
    let context = EditingPositionContext {
        document,
        layout: Some(layout),
        flat: false,
        ui_locale: None,
        behavior: None,
    };
    match query {
        CompositionTextQuery::EditorSelectionGeometry {
            query,
            behavior,
            ui_locale,
        } => Some(CompositionTextResult::EditorSelectionGeometry(
            super::editor_selection_geometry::Query(document, layout, &ui_locale, behavior, query),
        )),
        // editing_utilities.cc:1561-1587,1632-1648. Preserve the native
        // TextIterator's selection indexing behavior instead of DOM offsets.
        CompositionTextQuery::VisiblePositionIndex(position) => {
            let visible = VisiblePosition::Create(
                &context,
                SourcePosition(EditingPositionWithAffinity {
                    position,
                    affinity: EditingTextAffinity::Downstream,
                }),
            );
            let p = visible.DeepEquivalent();
            let node = document.FindNodeById(*p.AnchorNode()?)?;
            let scope = document
                .ContainingShadowRoot(node)
                .or_else(|| document.DocumentElement())?;
            let scope = document.Node(scope).Id();
            let mut behavior = TextIteratorBehavior::AllVisiblePositionsRangeLengthBehavior();
            behavior.SetSuppressesExtraNewlineEmission(true);
            let index = text_iterator::RangeLength(
                &context,
                &Position::FirstPositionInNode(scope),
                &p.ToOffsetInAnchor(&context),
                behavior,
            );
            Some(CompositionTextResult::VisiblePositionIndex(Some((
                scope, index,
            ))))
        }
        CompositionTextQuery::MovingParagraphRangeLength { start, end } => {
            let start = SourcePosition(EditingPositionWithAffinity {
                position: start,
                affinity: EditingTextAffinity::Downstream,
            })
            .position;
            let end = SourcePosition(EditingPositionWithAffinity {
                position: end,
                affinity: EditingTextAffinity::Downstream,
            })
            .position;
            Some(CompositionTextResult::Length(text_iterator::RangeLength(
                &context,
                &start,
                &end,
                TextIteratorBehavior::AllVisiblePositionsIncludingShadowRootRangeLengthBehavior(),
            )))
        }
        CompositionTextQuery::MovingParagraphPositionForIndex(index) => {
            let root = document.Node(document.DocumentElement()?).Id();
            let index = index.try_into().ok()?;
            let range = CreateRange(
                &context,
                root,
                PlainTextRange {
                    start: index,
                    end: index,
                },
            )?;
            let visible = VisiblePosition::Create(
                &context,
                PositionWithAffinity {
                    position: Position::new(range.start.node_id, range.start.offset),
                    affinity: TextAffinity::Downstream,
                },
            );
            NativePosition(visible.DeepEquivalent(), TextAffinity::Downstream)
                .map(CompositionTextResult::StructuralPosition)
        }
        CompositionTextQuery::PositionForVisibleIndex { scope, index } => {
            let mut behavior = TextIteratorBehavior::AllVisiblePositionsRangeLengthBehavior();
            behavior.SetSuppressesExtraNewlineEmission(true);
            let index = index.try_into().ok()?;
            let range = CreateRangeWithBehavior(
                &context,
                scope,
                PlainTextRange {
                    start: index,
                    end: index,
                },
                behavior,
            )?;
            let visible = VisiblePosition::Create(
                &context,
                PositionWithAffinity {
                    position: Position::new(range.start.node_id, range.start.offset),
                    affinity: TextAffinity::Downstream,
                },
            );
            NativePosition(visible.DeepEquivalent(), TextAffinity::Downstream)
                .map(CompositionTextResult::StructuralPosition)
        }
        CompositionTextQuery::SelectAllFacts { selection, focused } => {
            let selection = CreateVisibleDOMSelectionForFocus(document, layout, selection);
            let start = selection.map(|s| DOMSelectionStart(&context, s));
            let has_focus = super::mouse_selection_geometry::QuerySelectionFocus(
                document, layout, selection, focused,
            );
            let hidden = interaction_default::core::editing::frame_selection::SelectionIsHidden(
                has_focus,
                start.as_ref().and_then(|s| s.AnchorNode()).copied(),
                selection.is_some_and(|s| s.anchor != s.focus),
                |node| {
                    text_offset_mapping::EnclosingTextControl(
                        &context,
                        &Position::FirstPositionInNode(*node),
                    )
                    .is_some()
                },
            );
            // Node::NonBoundaryShadowTreeRootNode, node.cc:2126-2138.
            let shadow_root = start
                .as_ref()
                .and_then(|s| s.AnchorNode())
                .and_then(|node| document.FindNodeById(*node))
                .and_then(|mut node| {
                    let shadow = document.ContainingShadowRoot(node)?;
                    if node == shadow {
                        return None;
                    }
                    loop {
                        let parent = document.Node(node).Parent()?;
                        if parent == shadow {
                            return Some(node);
                        }
                        node = parent;
                    }
                });
            let editable_root = start
                .as_ref()
                .filter(|s| {
                    s.AnchorNode()
                        .is_some_and(|node| context.IsEditableNodeValue(node))
                })
                .and_then(|s| EditableBoundaryContext::HighestEditableRoot(&context, s));
            let editable_root_is_empty = editable_root
                .and_then(|root| document.FindNodeById(root))
                .is_some_and(|root| {
                    let children = document.Node(root).Children();
                    if children.is_empty() {
                        return true;
                    }
                    if children.len() != 1 {
                        return false;
                    }
                    let child = document.Node(children[0]);
                    child.IsHTMLElement("br")
                        || (child.Type() == DOMNodeType::kText
                            && !layout
                                .Node(child.Id())
                                .and_then(|node| node.EditingTextFacts(0, false))
                                .is_some_and(|facts| facts.has_non_collapsed_text))
                });
            Some(CompositionTextResult::SelectAllFacts(
                interaction_default::core::editing::frame_selection_select_all::SelectAllFacts {
                    visible_selection_is_none: selection.is_none(),
                    hidden,
                    editable_root,
                    editable_root_is_empty,
                    shadow_root: shadow_root.map(|node| document.Node(node).Id()),
                    shadow_host: shadow_root
                        .and_then(|node| document.OwnerShadowHost(node))
                        .map(|node| document.Node(node).Id()),
                    document_element: document
                        .DocumentElement()
                        .map(|node| document.Node(node).Id()),
                    body: document.BodyElement().map(|node| document.Node(node).Id()),
                },
            ))
        }
        CompositionTextQuery::ModifySelection {
            selection,
            state,
            alteration,
            direction,
            granularity,
            behavior,
            ui_locale,
        } => {
            let flat = EditingPositionContext::NewForSelectionModifier(
                document, layout, &ui_locale, behavior,
            );
            let (anchor, focus, affinity) =
                super::selection_geometry::FlatSelection(&flat, selection);
            Some(CompositionTextResult::ModifiedSelection(
                interaction_default::core::editing::frame_selection_modify::PrepareSelectionModification(
                    &flat, VisibleSelection { anchor, focus, affinity }, state, alteration, direction, granularity,
                    |selection| super::super::page_frame_selection::FrameSelectionValue(
                        NativePosition(&selection.anchor, selection.affinity).zip(NativePosition(&selection.focus, selection.affinity)).map(|(anchor, focus)| DOMSelection { anchor: anchor.position, focus: focus.position, affinity: anchor.affinity })
                    ),
                )
            ))
        }
        CompositionTextQuery::EditorCommandTargetRange {
            selection,
            directional,
            extend,
            behavior,
            ui_locale,
        } => Some(CompositionTextResult::EditorCommandTargetRange(
            super::editor_target_range_geometry::Query(
                document,
                layout,
                &ui_locale,
                behavior,
                selection,
                directional,
                extend,
            ),
        )),
        CompositionTextQuery::TypingDeletion {
            query,
            behavior,
            ui_locale,
        } => Some(CompositionTextResult::TypingDeletion(
            super::typing_deletion_geometry::Query(document, layout, &ui_locale, behavior, query),
        )),
        CompositionTextQuery::ApplyStyleGeometry(query) => {
            super::apply_style_geometry::Query(document, layout, query)
                .map(CompositionTextResult::ApplyStyleGeometry)
        }
        CompositionTextQuery::ReplacementRawLineBreak(position) => {
            let p = SourcePosition(EditingPositionWithAffinity {
                position,
                affinity: EditingTextAffinity::Downstream,
            })
            .position;
            Some(CompositionTextResult::StructuralBoolean(interaction_default::core::editing::visible_units_commands::LineBreakExistsAtPosition(&context,&p)))
        }
        CompositionTextQuery::ReplacementVisibleText(node) => Some(
            CompositionTextResult::ReplacementVisibleText(context.Layout(&node).is_some_and(|n| {
                n.EditingParagraphLayoutFacts()
                    .text
                    .is_some_and(|t| t.resolved_text_length > 0)
            })),
        ),
        CompositionTextQuery::ReplacementListItemNumber(node) => {
            Some(CompositionTextResult::ReplacementListItemNumber(
                context.Layout(&node).and_then(|n| n.EditingListItemValue()),
            ))
        }
        CompositionTextQuery::ReplacementBlockPlaceholder(node) => {
            use interaction_default::core::editing::commands::break_blockquote_command::BlockPlaceholderFacts;
            let facts = context
                .Layout(&node)
                .and_then(|n| n.EditingBlockFlowPlaceholderFacts());
            Some(CompositionTextResult::ReplacementBlockPlaceholder(
                facts
                    .map(|f| BlockPlaceholderFacts {
                        block_flow: true,
                        height_zero: f.stitched_height_is_zero,
                        empty_list_item: f.is_layout_list_item && !f.has_layout_child,
                    })
                    .unwrap_or_default(),
            ))
        }
        CompositionTextQuery::ReplacementMoveGrapheme { position, forward } => {
            let p = SourcePosition(EditingPositionWithAffinity {
                position,
                affinity: EditingTextAffinity::Downstream,
            })
            .position;
            let p = if forward {
                interaction_default::core::editing::position_iterator::NextGraphemePosition(
                    &context, &p,
                )
            } else {
                interaction_default::core::editing::position_iterator::PreviousGraphemePosition(
                    &context, &p,
                )
            };
            NativePosition(&p, TextAffinity::Downstream)
                .map(CompositionTextResult::StructuralPosition)
        }
        CompositionTextQuery::SimplifyMarkupInline(node) => {
            Some(CompositionTextResult::SimplifyMarkupInline(
                context
                    .Layout(&node)
                    .and_then(|n| n.EditingInlineAlwaysCreateLineBoxes()),
            ))
        }
        CompositionTextQuery::ReplacementBoundaryCodePoint { position, before } => {
            Some(CompositionTextResult::ReplacementBoundaryCodePoint(
                super::replace_selection_geometry::CharacterCodePointAtVisibleBoundary(
                    document, layout, position, before,
                ),
            ))
        }
        CompositionTextQuery::ReplacementCanHaveChildren(node) => {
            Some(CompositionTextResult::ReplacementCanHaveChildren(
                layout
                    .Node(node)
                    .is_some_and(|n| n.EditingCanHaveChildren()),
            ))
        }
        CompositionTextQuery::ReplacementBoundaryCharacter { position, before } => {
            Some(CompositionTextResult::ReplacementBoundaryCharacter(
                super::replace_selection_geometry::CharacterAtVisibleBoundary(
                    document, layout, position, before,
                ),
            ))
        }
        CompositionTextQuery::SelectedTextForClipboard {
            selection,
            include_image_alt_text,
        } => {
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
            let flat = context.Flat();
            let Some((start,end)) = interaction_default::core::editing::serializers::serialization::ComputeRangeForSerialization(&context, &flat, &anchor, &focus) else { return Some(CompositionTextResult::Text(Vec::new())); };
            let mut behavior = TextIteratorBehavior::default();
            behavior
                .SetEmitsImageAltText(include_image_alt_text)
                .SetSkipsUnselectableContent(true)
                .SetEntersTextControls(true)
                .SetIgnoresCssTextTransforms(true);
            let mut text = text_iterator::PlainText(&flat, &start, &end, behavior);
            text.retain(|unit| *unit != 0);
            Some(CompositionTextResult::Text(text))
        }
        CompositionTextQuery::FragmentFromTextContext(selection) => {
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
            let start = if compare_positions::CompareDOMPositions(&context, &anchor, &focus)?
                == Ordering::Greater
            {
                focus
            } else {
                anchor
            };
            let first = text_offset_mapping::NodeAsRangeFirstNode(&context, &start);
            let preserve_newlines = first
                .filter(|node| context.Layout(node).is_some())
                .or_else(|| start.AnchorNode().copied())
                .filter(|node| context.Layout(node).is_some())
                .and_then(|node| context.Index(&node))
                .and_then(|node| document.ResolvedStyleFor(node))
                .and_then(|style| style.ComputedStyle())
                .is_some_and(|style| style.ShouldPreserveBreaks());
            let block = first.and_then(|node| {
                ParagraphContext::EnclosingBlock(
                    &context,
                    &Position::FirstPositionInNode(node),
                    Rule::CanCross,
                )
            });
            let root = editing_boundaries::RootEditableElementOf(&context, &start);
            let block = block.filter(|node| {
                Some(*node) != root
                    && context.Index(node).is_some_and(|index| {
                        let n = document.Node(index);
                        !n.IsHTMLElement("body") && !n.IsHTMLElement("html")
                    })
            });
            Some(CompositionTextResult::FragmentFromTextContext(interaction_default::core::editing::serializers::serialization::TextFragmentContext {
                richly_editable: {
                    let mut node = *start.AnchorNode()?;
                    if context.IsDisplayInsideTable(&node) { node = PositionTree::Parent(&context, &node)?; }
                    foundation::editable::IsRichlyEditable(&dom::persistent_document::DOMEditableFacts(document), node, Default::default())
                },
                preserve_newlines,
                enclosing_block_to_clone: block,
                default_paragraph_separator: interaction_default::core::editing::editor::EditorParagraphSeparator::kIsDiv,
            }))
        }
        CompositionTextQuery::NodeWritingMode(node) => {
            Some(CompositionTextResult::NodeWritingMode(
                layout
                    .Node(node)
                    .map(|node| node.EditingCaretFacts().writing_mode),
            ))
        }
        CompositionTextQuery::SelectionFocus { selection, focused } => {
            Some(CompositionTextResult::SelectionFocus(
                super::mouse_selection_geometry::QuerySelectionFocus(
                    document, layout, selection, focused,
                ),
            ))
        }
        CompositionTextQuery::ClipboardSelection {
            selection,
            focused,
            has_focus,
        } => {
            let (target, richly_editable) = super::clipboard_geometry::QueryClipboardSelection(
                document, layout, selection, focused, has_focus,
            );
            Some(CompositionTextResult::ClipboardSelection {
                target,
                richly_editable,
            })
        }
        CompositionTextQuery::NonLocatedGeometry(query) => {
            super::nonlocated_context_menu_geometry::QueryNonLocatedGeometry(
                document, layout, query,
            )
            .map(CompositionTextResult::NonLocatedGeometry)
        }
        CompositionTextQuery::PotentialSuggestionTap { owner, position } => {
            let flat = context.Flat();
            let caret = position_conversion::ToPositionInFlatTree(
                &context,
                &flat,
                &SourcePosition(position).position,
            );
            Some(CompositionTextResult::PotentialSuggestionTap(interaction_default::core::editing::text_suggestion_controller::HasPotentialSuggestionTap(&flat,owner.SourceController(),&caret)))
        }
        CompositionTextQuery::MouseSelectionGeometry { query, ui_locale } => {
            super::mouse_selection_geometry::QueryMouseSelectionGeometry(
                document, layout, query, &ui_locale,
            )
            .map(CompositionTextResult::MouseSelectionGeometry)
        }
        CompositionTextQuery::SelectionGeometry { query, ui_locale } => {
            super::selection_geometry::QuerySelectionGeometry(document, layout, query, &ui_locale)
                .map(CompositionTextResult::SelectionGeometry)
        }
        CompositionTextQuery::CanonicalSelection(selection) => {
            let base = VisiblePosition::Create(
                &context,
                SourcePosition(EditingPositionWithAffinity {
                    position: selection.anchor,
                    affinity: EditingTextAffinity::Downstream,
                }),
            );
            let extent = VisiblePosition::Create(
                &context,
                SourcePosition(EditingPositionWithAffinity {
                    position: selection.focus,
                    affinity: EditingTextAffinity::Downstream,
                }),
            );
            let visible =
                interaction_default::core::editing::visible_selection::CreateVisibleSelection(
                    &context,
                    base.DeepEquivalent(),
                    extent.DeepEquivalent(),
                    base.ToPositionWithAffinity().affinity,
                );
            NativeDOMSelection(&visible.anchor, &visible.focus, visible.affinity)
                .map(CompositionTextResult::CanonicalSelection)
        }
        CompositionTextQuery::StructuralBoolean {
            position,
            other,
            query,
        } => {
            use interaction_default::core::editing::visible_units_commands as commands;
            let source = SourcePosition(EditingPositionWithAffinity {
                position,
                affinity: EditingTextAffinity::Downstream,
            })
            .position;
            let visible = VisiblePosition::Create(
                &context,
                PositionWithAffinity {
                    position: source.clone(),
                    affinity: TextAffinity::Downstream,
                },
            );
            let value = match query {
                StructuralBooleanQuery::RenderedCharacter => {
                    commands::IsRenderedCharacter(&context, &source)
                }
                StructuralBooleanQuery::VisuallyEquivalentCandidate => {
                    visible_units::IsVisuallyEquivalentCandidate(&context, &source)
                }
                StructuralBooleanQuery::RendersInDifferentPosition => other.is_some_and(|other| {
                    commands::RendersInDifferentPosition(
                        &context,
                        &source,
                        &SourcePosition(EditingPositionWithAffinity {
                            position: other,
                            affinity: EditingTextAffinity::Downstream,
                        })
                        .position,
                    )
                }),
                StructuralBooleanQuery::StartOfLine => commands::IsStartOfLine(&context, &visible),
                StructuralBooleanQuery::LineBreakExists => {
                    commands::LineBreakExistsAtPosition(&context, visible.DeepEquivalent())
                }
            };
            Some(CompositionTextResult::StructuralBoolean(value))
        }
        CompositionTextQuery::StructuralPosition { position, query } => {
            let source = SourcePosition(position);
            let result = match query {
                StructuralPositionQuery::Visible | StructuralPositionQuery::VisibleUpstream => {
                    VisiblePosition::Create(
                        &context,
                        PositionWithAffinity {
                            position: source.position.clone(),
                            affinity: if matches!(query, StructuralPositionQuery::VisibleUpstream) {
                                TextAffinity::Upstream
                            } else {
                                source.affinity
                            },
                        },
                    )
                    .DeepEquivalent()
                    .clone()
                }
                StructuralPositionQuery::PreviousCandidate => {
                    visible_units::PreviousCandidate(&context, &source.position)
                }
                StructuralPositionQuery::Backward => visible_units::MostBackwardCaretPosition(
                    &context,
                    &source.position,
                    Rule::CannotCross,
                    SnapToClient::Editing,
                ),
                StructuralPositionQuery::Forward => visible_units::MostForwardCaretPosition(
                    &context,
                    &source.position,
                    Rule::CannotCross,
                    SnapToClient::Editing,
                ),
                StructuralPositionQuery::Previous
                | StructuralPositionQuery::PreviousWithinEditable
                | StructuralPositionQuery::Next
                | StructuralPositionQuery::NextWithinEditable => {
                    let visible = VisiblePosition::Create(&context, source.clone());
                    if matches!(
                        query,
                        StructuralPositionQuery::Previous
                            | StructuralPositionQuery::PreviousWithinEditable
                    ) {
                        visible_navigation::PreviousPositionOf(
                            &context,
                            visible.DeepEquivalent(),
                            if matches!(query, StructuralPositionQuery::PreviousWithinEditable) {
                                Rule::CannotCross
                            } else {
                                Rule::CanCross
                            },
                        )
                        .DeepEquivalent()
                        .clone()
                    } else {
                        visible_navigation::NextPositionOf(
                            &context,
                            visible.ToPositionWithAffinity(),
                            if matches!(query, StructuralPositionQuery::NextWithinEditable) {
                                Rule::CannotCross
                            } else {
                                Rule::CanCross
                            },
                        )
                        .DeepEquivalent()
                        .clone()
                    }
                }
                StructuralPositionQuery::ParagraphStart => {
                    position_units_paragraph::StartOfParagraph(
                        &context,
                        &source.position,
                        Rule::CannotCross,
                    )
                }
                StructuralPositionQuery::ParagraphEnd => position_units_paragraph::EndOfParagraph(
                    &context,
                    &source.position,
                    Rule::CannotCross,
                ),
                StructuralPositionQuery::BlockStart | StructuralPositionQuery::BlockEnd => {
                    let block = ParagraphContext::EnclosingBlock(
                        &context,
                        &source.position,
                        Rule::CannotCross,
                    )?;
                    if matches!(query, StructuralPositionQuery::BlockStart) {
                        Position::FirstPositionInNode(block)
                    } else {
                        Position::LastPositionInNode(&context, &block)
                    }
                }
            };
            Some(CompositionTextResult::StructuralPosition(
                NativePosition(&result, source.affinity).unwrap_or_default(),
            ))
        }
        CompositionTextQuery::StructuralBlock(position) => {
            let source = SourcePosition(EditingPositionWithAffinity {
                position,
                affinity: EditingTextAffinity::Downstream,
            });
            Some(CompositionTextResult::StructuralBlock(
                ParagraphContext::EnclosingBlock(&context, &source.position, Rule::CannotCross),
            ))
        }
        CompositionTextQuery::StructuralNode(node) => {
            let (html_element, link) =
                document.FindNodeById(node).map_or((false, false), |index| {
                    let element = document.Node(index);
                    (
                        element.Type() == dom::DOMNodeType::kElement
                            && element.Namespace() == dom::DOMNamespace::kHTML,
                        (element.IsHTMLElement("a") || element.IsHTMLElement("area"))
                            && element.FindAttribute("href").is_some(),
                    )
                });
            let Some(layout) = layout.Node(node) else {
                return Some(CompositionTextResult::StructuralNode(StructuralNodeFacts {
                    special_element: html_element && link,
                    richly_editable: foundation::editable::IsRichlyEditable(
                        &dom::persistent_document::DOMEditableFacts(document),
                        node,
                        Default::default(),
                    ),
                    ..Default::default()
                }));
            };
            let facts = layout.EditingParagraphLayoutFacts();
            let inline = layout.EditingInlineAdjustmentFacts();
            let text = layout.EditingTextFacts(0, false);
            Some(CompositionTextResult::StructuralNode(StructuralNodeFacts {
                has_layout: true,
                special_element: html_element && (link || facts.display_table || facts.is_floating),
                list_item: layout.EditingIsListItem(),
                richly_editable: layout.EditingCaretFacts().used_user_modify
                    == foundation::EUserModify::kReadWrite,
                insignificant_text: facts.text.as_ref().is_some_and(|text| {
                    !text.has_inline_fragments && text.resolved_text_length == 0
                }),
                block: facts.is_enclosing_block,
                block_flow: layout.EditingIsBlockFlow(),
                inline: inline.is_inline,
                table_cell: facts.table_cell,
                table_structure: facts.is_table_structure_node,
                br: facts.br,
                atomic_block: facts.is_rendered_as_non_inline_table_image_or_hr,
                preserve_breaks: !facts.collapse_breaks,
                preserve_spaces: facts.preserve_white_spaces,
                visible: facts.visible,
                caret_min: text.as_ref().map_or(0, |text| text.caret_min_offset),
                caret_max: text.as_ref().map_or_else(
                    || context.LastDOMOffsetForEditing(&node),
                    |text| text.caret_max_offset,
                ),
            }))
        }
        CompositionTextQuery::AnchorFirst(selection) => {
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
            Some(CompositionTextResult::AnchorFirst(
                compare_positions::CompareDOMPositions(&context, &anchor, &focus)?
                    != Ordering::Greater,
            ))
        }
        CompositionTextQuery::Markers(update) => {
            let effects = match update.operation {
                CompositionMarkerOperation::ReplaceComposition(range) => {
                    let mut effects = update.owner.0.RemoveMarkersOfTypes(MarkerTypes::Single(MarkerType::Composition));
                    if let Some(range) = range {
                        let (start, end) = RangePositions(range);
                        effects.extend(update.owner.0.AddCompositionMarker(&context, &start, &end, &StyleableMarker::default()));
                    }
                    effects
                }
                CompositionMarkerOperation::RemoveSuggestionsOnFinish(range) => {
                    let (start, end) = RangePositions(range);
                    update.owner.0.RemoveSuggestionMarkerInRangeOnFinish(&context, &start, &end)
                }
                CompositionMarkerOperation::RemoveAutocorrect => update.owner.0.RemoveSuggestionMarkerByType(interaction_default::core::editing::markers::styleable_marker::SuggestionType::Autocorrect),
                CompositionMarkerOperation::AddImeSpans { root, offset, spans } => interaction_default::core::editing::ime::input_method_controller::AddImeTextSpans(&context, &update.owner.0, &spans, &root, offset),
            };
            let mut ranges: Vec<_> = update
                .owner
                .0
                .CompositionMarkerRanges()
                .into_iter()
                .map(|(node, start, end)| DocumentCompositionMarker {
                    node_id: node,
                    start: start.min(u32::MAX as usize) as u32,
                    end: end.min(u32::MAX as usize) as u32,
                })
                .collect();
            ranges.sort_unstable_by_key(|marker| (marker.node_id, marker.start, marker.end));
            let mut invalidated: Vec<_> = effects
                .into_iter()
                .filter_map(|effect| match effect {
                    MarkerEffect::InvalidatePaint(node) => Some(node),
                    _ => None,
                })
                .collect();
            invalidated.sort_unstable();
            invalidated.dedup();
            Some(CompositionTextResult::Markers {
                ranges: Rc::from(ranges),
                invalidated,
            })
        }
        CompositionTextQuery::SelectionRange(selection) => {
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
            let (start, end) = if compare_positions::CompareDOMPositions(&context, &anchor, &focus)?
                == Ordering::Greater
            {
                (focus, anchor)
            } else {
                (anchor, focus)
            };
            Some(CompositionTextResult::Range(DOMStaticRange {
                start: Boundary(&context, &start)?,
                end: Boundary(&context, &end)?,
            }))
        }
        CompositionTextQuery::Range { root, offsets } => {
            CreateRange(&context, root, offsets).map(CompositionTextResult::Range)
        }
        CompositionTextQuery::Offsets { root, range } => {
            for point in [range.start, range.end] {
                if point.node_id != root && !context.IsDescendant(&point.node_id, &root) {
                    return None;
                }
            }
            let (start, end) = RangePositions(range);
            let origin = Position::FirstPositionInNode(root);
            let behavior = TextIteratorBehavior::DefaultRangeLengthBehavior();
            let start_offset = text_iterator::RangeLength(&context, &origin, &start, behavior);
            let end_offset = if range.Collapsed() {
                start_offset
            } else {
                text_iterator::RangeLength(&context, &origin, &end, behavior)
            };
            Some(CompositionTextResult::Offsets(PlainTextRange {
                start: start_offset.try_into().ok()?,
                end: end_offset.try_into().ok()?,
            }))
        }
        CompositionTextQuery::ReplacementTestRenderingText(holder) => {
            let selection = CreateVisibleDOMSelectionForFocus(
                document,
                layout,
                Some(DOMSelection {
                    anchor: EditingPosition::new(holder, 0),
                    focus: EditingPosition::new(holder, context.ChildCount(&holder)),
                    affinity: EditingTextAffinity::Downstream,
                }),
            )?;
            let start = DOMSelectionStart(&context, selection);
            let end = SourcePosition(EditingPositionWithAffinity {
                position: selection.focus,
                affinity: selection.affinity,
            })
            .position;
            let mut behavior = TextIteratorBehavior::default();
            behavior
                .SetEmitsOriginalText(true)
                .SetIgnoresStyleVisibility(true);
            Some(CompositionTextResult::Text(text_iterator::PlainText(
                &context, &start, &end, behavior,
            )))
        }
        CompositionTextQuery::Text { range, original } => {
            let (start, end) = RangePositions(range);
            let mut behavior = TextIteratorBehavior::default();
            behavior.SetEmitsOriginalText(original);
            Some(CompositionTextResult::Text(text_iterator::PlainText(
                &context, &start, &end, behavior,
            )))
        }
        CompositionTextQuery::Length { root } => {
            Some(CompositionTextResult::Length(text_iterator::RangeLength(
                &context,
                &Position::FirstPositionInNode(root),
                &Position::LastPositionInNode(&context, &root),
                TextIteratorBehavior::AllVisiblePositionsRangeLengthBehavior(),
            )))
        }
    }
}
