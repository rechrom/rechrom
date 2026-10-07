#![allow(non_snake_case)]

use foundation::graphics_types;
use std::cell::Cell;
use std::ffi::c_char;
use std::ops::{Deref, DerefMut};

use super::algorithm_forward::{OffsetMapping, TextDiffRange};
use font_engine::Font;
use foundation::{
    gfx, ETextSecurity, ETextTransform, LayoutUnit, PhysicalOffset, PhysicalRect, String, Vector,
    Visitor,
};
use graphics_types::graphics::dom_node_id::{kInvalidDOMNodeId, DOMNodeId};
use layoutng_fragment_tree::inline_items_data::InlineItemsData;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::style_difference::StyleDifference;

use super::editing::forward::{Position, PositionWithAffinity};
use super::hit_test_phase::HitTestPhase;
use super::inline_item_span::InlineItemSpan;
use super::layout_box_model_object::LayoutBoxModelObject;
use super::layout_node_metadata::{Node, Text};
use super::layout_object::{
    BoxQuadType, HitTestLocation, HitTestResult, IncludeDescendants, LayoutObject, PaintInfo,
    PaintInvalidatorContext, StyleChangeContext,
};
use super::map_coordinates_flags::MapCoordinatesFlags;
use super::variable_length_transform_result::VariableLengthTransformResult;

// The constructor and most non-inline methods belong to //src/layoutng_inline.
unsafe extern "Rust" {
    fn LayoutTextNew(node: *mut Node, text: String) -> LayoutText;
    fn LayoutTextTrace(this: &LayoutText, visitor: &mut Visitor);
    fn LayoutTextIsWordBreak(this: &LayoutText) -> bool;
    fn LayoutTextScaledFont(this: &LayoutText) -> *const Font;
    fn LayoutTextOriginalText(this: &LayoutText) -> String;
    fn LayoutTextOriginalTextLength(this: &LayoutText) -> u32;
    fn LayoutTextHasInlineFragments(this: &LayoutText) -> bool;
    fn LayoutTextClearFirstInlineFragmentItemIndex(this: &mut LayoutText);
    fn LayoutTextSetFirstInlineFragmentItemIndex(this: &mut LayoutText, index: usize);
    fn LayoutTextPlainText(this: &LayoutText) -> String;
    fn LayoutTextGetVariableLengthTransformResult(
        this: &LayoutText,
    ) -> VariableLengthTransformResult;
    fn LayoutTextClearHasVariableLengthTransform(this: &mut LayoutText);
    fn LayoutTextQuadsInAncestorInternal(
        this: &LayoutText,
        quads: &mut Vector<gfx::QuadF>,
        ancestor: *const LayoutBoxModelObject,
        flags: MapCoordinatesFlags,
        quad_type: BoxQuadType,
    );
    fn LayoutTextAbsoluteQuadsForRange(
        this: &LayoutText,
        quads: &mut Vector<gfx::QuadF>,
        start_offset: u32,
        end_offset: u32,
    );
    fn LayoutTextLocalBoundingBoxRectForAccessibility(
        this: &LayoutText,
        include_descendants: IncludeDescendants,
    ) -> gfx::RectF;
    fn LayoutTextLocalQuadsInFlippedBlocksDirection(
        this: &LayoutText,
        quads: &mut Vector<gfx::QuadF>,
        option: ClippingOption,
    );
    fn LayoutTextPositionForPoint(
        this: &LayoutText,
        point: &PhysicalOffset,
    ) -> PositionWithAffinity;
    fn LayoutTextFirstCharacterAfterWhitespaceCollapsing(this: &LayoutText) -> foundation::UChar32;
    fn LayoutTextLastCharacterAfterWhitespaceCollapsing(this: &LayoutText) -> foundation::UChar32;
    fn LayoutTextPhysicalLinesBoundingBox(this: &LayoutText) -> PhysicalRect;
    fn LayoutTextVisualOverflowRect(this: &LayoutText) -> PhysicalRect;
    fn LayoutTextInvalidateVisualOverflow(this: &mut LayoutText);
    fn LayoutTextFirstLineBoxTopLeft(this: &LayoutText) -> PhysicalOffset;
    fn LayoutTextSetTextIfNeeded(this: &mut LayoutText, text: String);
    fn LayoutTextForceSetText(this: &mut LayoutText, text: String);
    fn LayoutTextSetTextInternal(this: &mut LayoutText, text: String);
    fn LayoutTextTransformAndSecureOriginalText(this: &mut LayoutText);
    fn LayoutTextTransformAndSecureText(
        this: &LayoutText,
        original: &String,
        offset_map: &mut foundation::TextOffsetMap,
    ) -> String;
    fn LayoutTextLocalSelectionVisualRect(this: &LayoutText) -> PhysicalRect;
    fn LayoutTextGetTextBoxInfo(this: &LayoutText) -> Vector<TextBoxInfo>;
    fn LayoutTextPositionForCaretOffset(this: &LayoutText, offset: u32) -> Position;
    fn LayoutTextCaretOffsetForPosition(this: &LayoutText, position: &Position) -> Option<u32>;
    fn LayoutTextContainsCaretOffset(this: &LayoutText, offset: i32) -> bool;
    fn LayoutTextIsBeforeNonCollapsedCharacter(this: &LayoutText, offset: u32) -> bool;
    fn LayoutTextIsAfterNonCollapsedCharacter(this: &LayoutText, offset: u32) -> bool;
    fn LayoutTextCaretMinOffset(this: &LayoutText) -> usize;
    fn LayoutTextCaretMaxOffset(this: &LayoutText) -> usize;
    fn LayoutTextResolvedTextLength(this: &LayoutText) -> u32;
    fn LayoutTextHasNonCollapsedText(this: &LayoutText) -> bool;
    fn LayoutTextSetTextWithOffset(this: &mut LayoutText, text: String, diff: &TextDiffRange);
    fn LayoutTextLocalCaretRect(
        this: &LayoutText,
        caret_offset: i32,
        shape: super::caret_rect::CaretShape,
    ) -> PhysicalRect;
    fn LayoutTextMomentarilyRevealLastTypedCharacter(this: &mut LayoutText, offset: u32);
    fn LayoutTextIsAllCollapsibleWhitespace(this: &LayoutText) -> bool;
    fn LayoutTextRemoveAndDestroyTextBoxes(this: &mut LayoutText);
    fn LayoutTextFirstAbstractInlineTextBox(this: &mut LayoutText) -> *mut AbstractInlineTextBox;
    fn LayoutTextDebugRect(this: &LayoutText) -> PhysicalRect;
    fn LayoutTextPreviousCharacter(this: &LayoutText) -> foundation::UChar;
    fn LayoutTextGetOffsetMapping(this: &LayoutText) -> *const OffsetMapping;
    fn LayoutTextMapDOMOffsetToTextContentOffset(
        this: &LayoutText,
        mapping: &OffsetMapping,
        start: &mut u32,
        end: &mut u32,
    ) -> bool;
    fn LayoutTextEnsureNodeId(this: &mut LayoutText) -> DOMNodeId;
    fn LayoutTextSetInlineItems(
        this: &mut LayoutText,
        data: *mut InlineItemsData,
        begin: usize,
        size: usize,
    );
    fn LayoutTextClearInlineItems(this: &mut LayoutText);
    fn LayoutTextInlineItems(this: &LayoutText) -> *const InlineItemSpan;
    fn LayoutTextInvalidateSubtreeLayoutForFontUpdates(this: &mut LayoutText);
    fn LayoutTextLogicalStartingPointAndHeight(
        this: &LayoutText,
        point: &mut LogicalOffset,
        height: &mut LayoutUnit,
    );
    fn LayoutTextSetPreviousLogicalStartingPoint(this: &LayoutText, point: &LogicalOffset);
    #[cfg(debug_assertions)]
    fn LayoutTextRecalcVisualOverflow(this: &mut LayoutText);
    fn LayoutTextWillBeDestroyed(this: &mut LayoutText);
    fn LayoutTextStyleDidChange(
        this: &mut LayoutText,
        difference: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        context: &StyleChangeContext,
    );
    fn LayoutTextInLayoutNGInlineFormattingContextWillChange(this: &mut LayoutText, value: bool);
    fn LayoutTextTextDidChange(this: &mut LayoutText);
    fn LayoutTextInvalidatePaint(this: &LayoutText, context: &PaintInvalidatorContext);
    fn LayoutTextInvalidateDisplayItemClients(
        this: &LayoutText,
        reason: foundation::PaintInvalidationReason,
    );
    fn LayoutTextSetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(
        this: &mut LayoutText,
        reason: *const c_char,
    );
    fn LayoutTextTextDidChangeWithoutInvalidation(this: &mut LayoutText);
    fn LayoutTextCollectLineBoxRects(
        this: &LayoutText,
        collector: &dyn Fn(&PhysicalRect),
        option: ClippingOption,
    );
    fn LayoutTextDeleteTextBoxes(this: &mut LayoutText);
    fn LayoutTextSecureText(
        this: &LayoutText,
        plain: &String,
        mask: foundation::UChar,
    ) -> (String, foundation::TextOffsetMap);
    fn LayoutTextSetVariableLengthTransformResult(
        this: &mut LayoutText,
        original_length: usize,
        offset_map: &foundation::TextOffsetMap,
    );
    fn LayoutTextGetSelectionDisplayItemClient(
        this: &LayoutText,
    ) -> *const graphics_types::graphics::paint::display_item_client::DisplayItemClient;
    fn LayoutTextGetOrResetContentCaptureManager(
        this: &mut LayoutText,
    ) -> *mut ContentCaptureManager;
    fn LayoutTextNonCollapsedCaretMaxOffset(this: &LayoutText) -> u32;
}

// Forward-declared in the source; concrete ownership is outside this package.
pub enum ContentCaptureManager {}
pub enum AbstractInlineTextBox {}

// cpp: layoutng/internal/layout_text.h:162-164
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClippingOption {
    kNoClipping,
    kClipToEllipsis,
}

// cpp: layoutng/internal/layout_text.h:204-210
pub struct TextBoxInfo {
    pub local_rect: PhysicalRect,
    pub dom_start_offset: u32,
    pub dom_length: u32,
}

// cpp: layoutng/internal/layout_text.h:65-488
#[repr(C)]
pub struct LayoutText {
    object_: LayoutObject,
    pub(crate) valid_ng_items_: bool,
    pub(crate) has_no_control_items_: bool,
    pub(crate) has_bidi_control_items_: bool,
    pub(crate) is_text_fragment_: bool,
    pub(crate) has_abstract_inline_text_box_: bool,
    pub(crate) has_variable_length_transform_: bool,
    ignore_whitespace_for_accessibility_: Cell<bool>,
    has_cached_ignore_whitespace_for_accessibility_: Cell<bool>,
    node_id_: DOMNodeId,
    pub(crate) text_: String,
    pub(crate) previous_logical_starting_point_: Cell<LogicalOffset>,
    pub(crate) inline_items_: InlineItemSpan,
    pub(crate) first_fragment_item_index_: usize,
}

// cpp: layoutng/internal/layout_text.h:509-513
impl foundation::DowncastFrom<LayoutObject> for LayoutText {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsText()
    }
}

const _: () = assert!(std::mem::offset_of!(LayoutText, object_) == 0);

impl LayoutText {
    // Construction is defined in //src/layoutng_inline/layout_text.cc. The
    // shared assembly invokes this base initializer from that source module.
    // cpp: layoutng_inline/layout_text.cc:31-41
    pub(crate) fn new_base_for_inline(node: *mut Node, text: String) -> Self {
        debug_assert!(!text.IsNull());
        debug_assert!(node.is_null() || !unsafe { &*node }.IsDocumentNode());
        let result = Self {
            object_: LayoutObject::new_base(node),
            valid_ng_items_: false,
            has_no_control_items_: false,
            has_bidi_control_items_: false,
            is_text_fragment_: false,
            has_abstract_inline_text_box_: false,
            has_variable_length_transform_: false,
            ignore_whitespace_for_accessibility_: Cell::new(false),
            has_cached_ignore_whitespace_for_accessibility_: Cell::new(false),
            node_id_: kInvalidDOMNodeId,
            text_: text,
            previous_logical_starting_point_: Cell::new(Self::UninitializedLogicalStartingPoint()),
            inline_items_: InlineItemSpan::default(),
            first_fragment_item_index_: 0,
        };
        result
            .object_
            .SetRuntimeClass(super::layout_object::LayoutObjectClass::Text);
        result
    }

    // cpp: layoutng/internal/layout_text.h:67-77
    pub fn new(node: *mut Node, text: String) -> Self {
        unsafe { LayoutTextNew(node, text) }
    }
    pub fn Trace(&self, visitor: &mut Visitor) {
        unsafe { LayoutTextTrace(self, visitor) }
    }
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        "LayoutText"
    }

    // cpp: layoutng/internal/layout_text.h:79-88
    pub fn IsTextFragment(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.is_text_fragment_
    }
    // cpp: layoutng_inline/layout_text_fragment_layout.cc:28-28
    // The derived constructor owns this one-time base-class tag update.
    pub fn SetIsTextFragmentForDerived(&mut self) {
        self.is_text_fragment_ = true;
    }
    pub fn IsWordBreak(&self) -> bool {
        unsafe { LayoutTextIsWordBreak(self) }
    }
    pub fn ScaledFont(&self) -> &Font {
        unsafe { &*LayoutTextScaledFont(self) }
    }
    pub fn SvgScalingFactor(&self) -> f32 {
        if self.RuntimeClass() == super::layout_object::LayoutObjectClass::SvgInlineText {
            let objects = super::layout_pass_scope::LayoutObjectFactoryScope::Objects();
            let factor = unsafe { objects.as_ref() }
                .and_then(|objects| objects.svg_inline_text_scaling_factor)
                .expect("SVG inline text scaling factor is not installed");
            return factor(self);
        }
        1.0
    }

    // cpp: layoutng/internal/layout_text.h:90-100
    pub fn OriginalText(&self) -> String {
        unsafe { LayoutTextOriginalText(self) }
    }
    pub fn OriginalTextLength(&self) -> u32 {
        unsafe { LayoutTextOriginalTextLength(self) }
    }
    pub fn HasInlineFragments(&self) -> bool {
        unsafe { LayoutTextHasInlineFragments(self) }
    }
    pub fn ClearFirstInlineFragmentItemIndex(&mut self) {
        unsafe { LayoutTextClearFirstInlineFragmentItemIndex(self) }
    }
    pub fn SetFirstInlineFragmentItemIndex(&mut self, index: usize) {
        unsafe { LayoutTextSetFirstInlineFragmentItemIndex(self, index) }
    }

    // cpp: layoutng/internal/layout_text.h:102-120
    pub fn TransformedText(&self) -> &String {
        self.CheckIsNotDestroyed();
        &self.text_
    }
    pub fn TransformedTextLength(&self) -> u32 {
        self.CheckIsNotDestroyed();
        self.text_.length()
    }
    pub fn TextStartOffset(&self) -> u32 {
        self.CheckIsNotDestroyed();
        0
    }
    pub fn PlainText(&self) -> String {
        unsafe { LayoutTextPlainText(self) }
    }

    // cpp: layoutng/internal/layout_text.h:123-136
    pub fn HasVariableLengthTransform(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.has_variable_length_transform_
    }
    pub fn GetVariableLengthTransformResult(&self) -> VariableLengthTransformResult {
        unsafe { LayoutTextGetVariableLengthTransformResult(self) }
    }
    pub fn ClearHasVariableLengthTransform(&mut self) {
        unsafe { LayoutTextClearHasVariableLengthTransform(self) }
    }
    pub fn GetFirstLetterPart(&self) -> *mut LayoutText {
        self.CheckIsNotDestroyed();
        std::ptr::null_mut()
    }

    // cpp: layoutng/internal/layout_text.h:148-160
    pub fn IgnoreWhitespaceForAccessibility(&self) -> Option<bool> {
        self.CheckIsNotDestroyed();
        if self.has_cached_ignore_whitespace_for_accessibility_.get() {
            Some(self.ignore_whitespace_for_accessibility_.get())
        } else {
            None
        }
    }
    pub fn SetIgnoreWhitespaceForAccessibility(&self, value: bool) {
        self.CheckIsNotDestroyed();
        self.ignore_whitespace_for_accessibility_.set(value);
        self.has_cached_ignore_whitespace_for_accessibility_
            .set(true);
    }

    // cpp: layoutng/internal/layout_text.h:168-171
    pub fn HasEmptyText(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.text_.empty()
    }

    // cpp: layoutng/internal/layout_text.h:239-247
    pub fn IsSecure(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().TextSecurity() != ETextSecurity::kNone
    }
    pub fn HasTextTransform(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().TextTransform() != ETextTransform::kNone
    }

    // cpp: layoutng/internal/layout_text.h:258-275
    pub fn HasAbstractInlineTextBox(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.has_abstract_inline_text_box_
    }
    pub fn SetHasAbstractInlineTextBox(&mut self) {
        self.CheckIsNotDestroyed();
        self.has_abstract_inline_text_box_ = true;
    }
    pub fn AutosizingMultiplerChanged(&mut self) {
        self.CheckIsNotDestroyed();
        self.valid_ng_items_ = false;
        self.SetNeedsCollectInlines();
    }

    // cpp: layoutng/internal/layout_text.h:290-301
    pub fn HasNodeId(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.node_id_ != kInvalidDOMNodeId
    }
    pub fn HasValidInlineItems(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.valid_ng_items_
    }

    // cpp: layoutng/internal/layout_text.h:303-342
    pub fn InvalidateInlineItems(&mut self) {
        self.CheckIsNotDestroyed();
        self.valid_ng_items_ = false;
    }
    pub fn HasNoControlItems(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.has_no_control_items_
    }
    pub fn SetHasNoControlItems(&mut self) {
        self.CheckIsNotDestroyed();
        self.has_no_control_items_ = true;
    }
    pub fn ClearHasNoControlItems(&mut self) {
        self.CheckIsNotDestroyed();
        self.has_no_control_items_ = false;
    }
    pub fn HasBidiControlInlineItems(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.has_bidi_control_items_
    }
    pub fn SetHasBidiControlInlineItems(&mut self) {
        self.CheckIsNotDestroyed();
        self.has_bidi_control_items_ = true;
    }
    pub fn ClearHasBidiControlInlineItems(&mut self) {
        self.CheckIsNotDestroyed();
        self.has_bidi_control_items_ = false;
    }
    pub fn GetInlineItems(&self) -> &InlineItemSpan {
        self.CheckIsNotDestroyed();
        &self.inline_items_
    }
    pub fn GetInlineItemsMut(&mut self) -> &mut InlineItemSpan {
        self.CheckIsNotDestroyed();
        &mut self.inline_items_
    }

    // cpp: layoutng/internal/layout_text.h:353-363
    pub fn PreviousLogicalStartingPoint(&self) -> LogicalOffset {
        self.CheckIsNotDestroyed();
        self.previous_logical_starting_point_.get()
    }
    pub fn UninitializedLogicalStartingPoint() -> LogicalOffset {
        LogicalOffset::new(LayoutUnit::Max(), LayoutUnit::Max())
    }

    // cpp: layoutng/internal/layout_text.h:392-395
    pub fn CanBeSelectionLeafInternal(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    // cpp: layoutng/internal/layout_text.h:431-434
    pub fn IsText(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    // cpp: layoutng/internal/layout_text.h:490-507
    pub fn FirstInlineFragmentItemIndex(&self) -> usize {
        self.CheckIsNotDestroyed();
        if !self.IsInLayoutNGInlineFormattingContext() {
            return 0;
        }
        self.first_fragment_item_index_
    }
    pub fn DetachAxHooksIfNeeded(&mut self) {
        self.CheckIsNotDestroyed();
        if self.has_abstract_inline_text_box_ {
            self.DetachAxHooks();
        }
        if !self.IsInLayoutNGInlineFormattingContext() {
            return;
        }
        self.ClearBlockFlowCachedData();
    }
}

impl LayoutText {
    // cpp: layoutng/internal/layout_text.h:138-146
    pub fn QuadsInAncestorInternal(
        &self,
        quads: &mut Vector<gfx::QuadF>,
        ancestor: *const LayoutBoxModelObject,
        flags: MapCoordinatesFlags,
        quad_type: BoxQuadType,
    ) {
        unsafe { LayoutTextQuadsInAncestorInternal(self, quads, ancestor, flags, quad_type) }
    }
    pub fn AbsoluteQuadsForRange(
        &self,
        quads: &mut Vector<gfx::QuadF>,
        start_offset: u32,
        end_offset: u32,
    ) {
        unsafe { LayoutTextAbsoluteQuadsForRange(self, quads, start_offset, end_offset) }
    }
    pub fn AbsoluteQuadsForRangeDefault(&self, quads: &mut Vector<gfx::QuadF>) {
        self.AbsoluteQuadsForRange(quads, 0, i32::MAX as u32)
    }
    pub fn LocalBoundingBoxRectForAccessibility(
        &self,
        include_descendants: IncludeDescendants,
    ) -> gfx::RectF {
        unsafe { LayoutTextLocalBoundingBoxRectForAccessibility(self, include_descendants) }
    }

    // cpp: layoutng/internal/layout_text.h:162-166
    pub fn LocalQuadsInFlippedBlocksDirection(
        &self,
        quads: &mut Vector<gfx::QuadF>,
        option: ClippingOption,
    ) {
        unsafe { LayoutTextLocalQuadsInFlippedBlocksDirection(self, quads, option) }
    }
    pub fn LocalQuadsInFlippedBlocksDirectionDefault(&self, quads: &mut Vector<gfx::QuadF>) {
        self.LocalQuadsInFlippedBlocksDirection(quads, ClippingOption::kNoClipping)
    }
    pub fn PositionForPoint(&self, point: &PhysicalOffset) -> PositionWithAffinity {
        unsafe { LayoutTextPositionForPoint(self, point) }
    }

    // cpp: layoutng/internal/layout_text.h:173-192
    pub fn FirstCharacterAfterWhitespaceCollapsing(&self) -> foundation::UChar32 {
        unsafe { LayoutTextFirstCharacterAfterWhitespaceCollapsing(self) }
    }
    pub fn LastCharacterAfterWhitespaceCollapsing(&self) -> foundation::UChar32 {
        unsafe { LayoutTextLastCharacterAfterWhitespaceCollapsing(self) }
    }
    pub fn PhysicalLinesBoundingBox(&self) -> PhysicalRect {
        unsafe { LayoutTextPhysicalLinesBoundingBox(self) }
    }
    pub fn VisualOverflowRect(&self) -> PhysicalRect {
        unsafe { LayoutTextVisualOverflowRect(self) }
    }
    pub fn InvalidateVisualOverflow(&mut self) {
        unsafe { LayoutTextInvalidateVisualOverflow(self) }
    }
    pub fn FirstLineBoxTopLeft(&self) -> PhysicalOffset {
        unsafe { LayoutTextFirstLineBoxTopLeft(self) }
    }
    pub fn SetTextIfNeeded(&mut self, text: String) {
        unsafe { LayoutTextSetTextIfNeeded(self, text) }
    }
    pub fn ForceSetText(&mut self, text: String) {
        unsafe { LayoutTextForceSetText(self, text) }
    }
    pub fn SetTextInternal(&mut self, text: String) {
        unsafe { LayoutTextSetTextInternal(self, text) }
    }

    // cpp: layoutng/internal/layout_text.h:194-210
    pub fn TransformAndSecureOriginalText(&mut self) {
        unsafe { LayoutTextTransformAndSecureOriginalText(self) }
    }
    pub fn TransformAndSecureText(
        &self,
        original: &String,
        offset_map: &mut foundation::TextOffsetMap,
    ) -> String {
        unsafe { LayoutTextTransformAndSecureText(self, original, offset_map) }
    }
    pub fn LocalSelectionVisualRect(&self) -> PhysicalRect {
        unsafe { LayoutTextLocalSelectionVisualRect(self) }
    }
    pub fn GetTextBoxInfo(&self) -> Vector<TextBoxInfo> {
        unsafe { LayoutTextGetTextBoxInfo(self) }
    }

    // cpp: layoutng/internal/layout_text.h:212-237
    pub fn PositionForCaretOffset(&self, offset: u32) -> Position {
        unsafe { LayoutTextPositionForCaretOffset(self, offset) }
    }
    pub fn CaretOffsetForPosition(&self, position: &Position) -> Option<u32> {
        unsafe { LayoutTextCaretOffsetForPosition(self, position) }
    }
    pub fn ContainsCaretOffset(&self, offset: i32) -> bool {
        unsafe { LayoutTextContainsCaretOffset(self, offset) }
    }
    pub fn IsBeforeNonCollapsedCharacter(&self, offset: u32) -> bool {
        unsafe { LayoutTextIsBeforeNonCollapsedCharacter(self, offset) }
    }
    pub fn IsAfterNonCollapsedCharacter(&self, offset: u32) -> bool {
        unsafe { LayoutTextIsAfterNonCollapsedCharacter(self, offset) }
    }
    pub fn CaretMinOffset(&self) -> usize {
        unsafe { LayoutTextCaretMinOffset(self) }
    }
    pub fn CaretMaxOffset(&self) -> usize {
        unsafe { LayoutTextCaretMaxOffset(self) }
    }
    pub fn ResolvedTextLength(&self) -> u32 {
        unsafe { LayoutTextResolvedTextLength(self) }
    }
    pub fn HasNonCollapsedText(&self) -> bool {
        unsafe { LayoutTextHasNonCollapsedText(self) }
    }
}

impl LayoutText {
    // cpp: layoutng/internal/layout_text.h:189-202
    pub fn SetTextWithOffset(&mut self, text: String, diff: &TextDiffRange) {
        unsafe { LayoutTextSetTextWithOffset(self, text, diff) }
    }
    pub fn LocalCaretRect(
        &self,
        caret_offset: i32,
        shape: super::caret_rect::CaretShape,
    ) -> PhysicalRect {
        unsafe { LayoutTextLocalCaretRect(self, caret_offset, shape) }
    }

    // cpp: layoutng/internal/layout_text.h:249-268
    pub fn MomentarilyRevealLastTypedCharacter(&mut self, offset: u32) {
        unsafe { LayoutTextMomentarilyRevealLastTypedCharacter(self, offset) }
    }
    pub fn IsAllCollapsibleWhitespace(&self) -> bool {
        unsafe { LayoutTextIsAllCollapsibleWhitespace(self) }
    }
    pub fn RemoveAndDestroyTextBoxes(&mut self) {
        unsafe { LayoutTextRemoveAndDestroyTextBoxes(self) }
    }
    pub fn FirstAbstractInlineTextBox(&mut self) -> *mut AbstractInlineTextBox {
        unsafe { LayoutTextFirstAbstractInlineTextBox(self) }
    }
    pub fn DebugRect(&self) -> PhysicalRect {
        unsafe { LayoutTextDebugRect(self) }
    }

    // cpp: layoutng/internal/layout_text.h:277-302
    pub fn PreviousCharacter(&self) -> foundation::UChar {
        unsafe { LayoutTextPreviousCharacter(self) }
    }
    pub fn GetOffsetMapping(&self) -> *const OffsetMapping {
        unsafe { LayoutTextGetOffsetMapping(self) }
    }
    pub fn MapDOMOffsetToTextContentOffset(
        &self,
        mapping: &OffsetMapping,
        start: &mut u32,
        end: &mut u32,
    ) -> bool {
        unsafe { LayoutTextMapDOMOffsetToTextContentOffset(self, mapping, start, end) }
    }
    pub fn EnsureNodeId(&mut self) -> DOMNodeId {
        unsafe { LayoutTextEnsureNodeId(self) }
    }
    pub fn SetInlineItems(&mut self, data: *mut InlineItemsData, begin: usize, size: usize) {
        unsafe { LayoutTextSetInlineItems(self, data, begin, size) }
    }
    pub fn ClearInlineItems(&mut self) {
        unsafe { LayoutTextClearInlineItems(self) }
    }
    pub fn InlineItems(&self) -> &InlineItemSpan {
        unsafe { &*LayoutTextInlineItems(self) }
    }

    // cpp: layoutng/internal/layout_text.h:344-367
    pub fn InvalidateSubtreeLayoutForFontUpdates(&mut self) {
        unsafe { LayoutTextInvalidateSubtreeLayoutForFontUpdates(self) }
    }
    pub fn LogicalStartingPointAndHeight(
        &self,
        point: &mut LogicalOffset,
        height: &mut LayoutUnit,
    ) {
        unsafe { LayoutTextLogicalStartingPointAndHeight(self, point, height) }
    }
    pub fn SetPreviousLogicalStartingPoint(&self, point: &LogicalOffset) {
        unsafe { LayoutTextSetPreviousLogicalStartingPoint(self, point) }
    }
    #[cfg(debug_assertions)]
    pub fn RecalcVisualOverflow(&mut self) {
        unsafe { LayoutTextRecalcVisualOverflow(self) }
    }
}

impl LayoutText {
    // cpp: layoutng/internal/layout_text.h:369-390
    pub fn WillBeDestroyed(&mut self) {
        unsafe { LayoutTextWillBeDestroyed(self) }
    }
    pub fn StyleWillChange(
        &self,
        _difference: StyleDifference,
        _old_style: *const ComputedStyle,
        _new_style: &ComputedStyle,
        _context: &mut StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
    }
    pub fn StyleDidChange(
        &mut self,
        difference: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        context: &StyleChangeContext,
    ) {
        unsafe { LayoutTextStyleDidChange(self, difference, old_style, new_style, context) }
    }
    pub fn InLayoutNGInlineFormattingContextWillChange(&mut self, value: bool) {
        unsafe { LayoutTextInLayoutNGInlineFormattingContextWillChange(self, value) }
    }
    pub fn TextDidChange(&mut self) {
        unsafe { LayoutTextTextDidChange(self) }
    }
    pub fn InvalidatePaint(&self, context: &PaintInvalidatorContext) {
        unsafe { LayoutTextInvalidatePaint(self, context) }
    }
    pub fn InvalidateDisplayItemClients(&self, reason: foundation::PaintInvalidationReason) {
        unsafe { LayoutTextInvalidateDisplayItemClients(self, reason) }
    }

    // cpp: layoutng/internal/layout_text.h:397-434
    pub fn SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(
        &mut self,
        reason: *const c_char,
    ) {
        unsafe {
            LayoutTextSetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(self, reason)
        }
    }
    pub fn TextDidChangeWithoutInvalidation(&mut self) {
        unsafe { LayoutTextTextDidChangeWithoutInvalidation(self) }
    }
    pub fn CollectLineBoxRects<F: Fn(&PhysicalRect)>(&self, collector: &F, option: ClippingOption) {
        unsafe { LayoutTextCollectLineBoxRects(self, collector, option) }
    }
    pub fn CollectLineBoxRectsDefault<F: Fn(&PhysicalRect)>(&self, collector: &F) {
        self.CollectLineBoxRects(collector, ClippingOption::kNoClipping)
    }
    pub fn Paint(&self, _info: &PaintInfo) -> ! {
        self.CheckIsNotDestroyed();
        unreachable!("LayoutText::Paint is unreachable; painting uses line fragments")
    }
    pub fn NodeAtPoint(
        &self,
        _result: &mut HitTestResult,
        _location: &HitTestLocation,
        _offset: &PhysicalOffset,
        _phase: HitTestPhase,
    ) -> ! {
        self.CheckIsNotDestroyed();
        unreachable!("LayoutText::NodeAtPoint is unreachable; hit testing uses line fragments")
    }
    pub fn DeleteTextBoxes(&mut self) {
        unsafe { LayoutTextDeleteTextBoxes(self) }
    }
    pub fn SecureText(
        &self,
        plain: &String,
        mask: foundation::UChar,
    ) -> (String, foundation::TextOffsetMap) {
        unsafe { LayoutTextSecureText(self, plain, mask) }
    }
    pub fn SetVariableLengthTransformResult(
        &mut self,
        original_length: usize,
        offset_map: &foundation::TextOffsetMap,
    ) {
        unsafe { LayoutTextSetVariableLengthTransformResult(self, original_length, offset_map) }
    }

    // cpp: layoutng/internal/layout_text.h:436-459
    pub fn GetSelectionDisplayItemClient(
        &self,
    ) -> *const graphics_types::graphics::paint::display_item_client::DisplayItemClient {
        unsafe { LayoutTextGetSelectionDisplayItemClient(self) }
    }
    pub fn GetOrResetContentCaptureManager(&mut self) -> *mut ContentCaptureManager {
        unsafe { LayoutTextGetOrResetContentCaptureManager(self) }
    }
    pub fn NonCollapsedCaretMaxOffset(&self) -> u32 {
        unsafe { LayoutTextNonCollapsedCaretMaxOffset(self) }
    }
}

impl Deref for LayoutText {
    type Target = LayoutObject;
    fn deref(&self) -> &Self::Target {
        &self.object_
    }
}
impl DerefMut for LayoutText {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object_
    }
}

// cpp: layoutng/internal/layout_text.h:72-72
impl foundation::Traceable for LayoutText {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        LayoutText::Trace(self, visitor);
    }
}

// cpp: layoutng/internal/layout_text.h:509-516
// cpp: layoutng/internal/layout_node_metadata.h:258-258
impl Text {
    pub fn GetLayoutObject(&self) -> *mut LayoutText {
        let object = self.node.GetLayoutObject();
        debug_assert!(object.is_null() || unsafe { &*object }.IsText());
        object.cast()
    }
}
