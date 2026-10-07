#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use font_engine::Font;
use foundation::{
    IsHorizontalTypographicMode, Member, PhysicalOffset, PhysicalRect, PhysicalSize, String,
    Visitor, WritingMode,
};
use layoutng_fragment_tree::fragment_item::FragmentItem;
use layoutng_fragment_tree::inline_cursor::InlineCursor;
use layoutng_style::style::computed_style::ComputedStyle;

use super::layout_block_flow::LayoutBlockFlow;
use super::layout_object::{LayoutObject, LayoutObjectClass};
use super::layout_text::LayoutText;

// These non-inline bodies belong to //src/layoutng_inline.
unsafe extern "Rust" {
    fn LayoutTextCombineNew() -> LayoutTextCombine;
    fn LayoutTextCombineDesiredWidth(this: &LayoutTextCombine) -> f32;
    fn LayoutTextCombineGetTextContent(this: &LayoutTextCombine) -> String;
    fn LayoutTextCombineSetCompressedFont(this: &mut LayoutTextCombine, font: *const Font);
    fn LayoutTextCombineAdjustOffsetForHitTest(
        this: &LayoutTextCombine,
        offset: &PhysicalOffset,
    ) -> PhysicalOffset;
    fn LayoutTextCombineAdjustOffsetForLocalCaretRect(
        this: &LayoutTextCombine,
        offset: &PhysicalOffset,
    ) -> PhysicalOffset;
    fn LayoutTextCombineAdjustRectForBoundingBox(
        this: &LayoutTextCombine,
        rect: &PhysicalRect,
    ) -> PhysicalRect;
    fn LayoutTextCombineComputeTextBoundsRectForHitTest(
        this: &LayoutTextCombine,
        text_item: &FragmentItem,
        offset: &PhysicalOffset,
    ) -> PhysicalRect;
    fn LayoutTextCombineRecalcContentsInkOverflow(
        this: &LayoutTextCombine,
        cursor: &InlineCursor,
    ) -> PhysicalRect;
    fn LayoutTextCombineResetLayout(this: &mut LayoutTextCombine);
    fn LayoutTextCombineSetScaleX(this: &mut LayoutTextCombine, new_scale_x: f32);
    fn LayoutTextCombineAssertStyleIsValid(style: &ComputedStyle);
    fn LayoutTextCombineCreateAnonymous(text_child: *mut LayoutText) -> *mut LayoutTextCombine;
    fn LayoutTextCombineApplyScaleXOffset(
        this: &LayoutTextCombine,
        offset: &PhysicalOffset,
    ) -> PhysicalOffset;
    fn LayoutTextCombineApplyScaleXRect(
        this: &LayoutTextCombine,
        rect: &PhysicalRect,
    ) -> PhysicalRect;
    fn LayoutTextCombineApplyScaleXSize(
        this: &LayoutTextCombine,
        size: &PhysicalSize,
    ) -> PhysicalSize;
    fn LayoutTextCombineUnapplyScaleX(
        this: &LayoutTextCombine,
        offset: &PhysicalOffset,
    ) -> PhysicalOffset;
    fn LayoutTextCombineComputeInlineSpacing(this: &LayoutTextCombine) -> f32;
    fn LayoutTextCombineUsingSyntheticOblique(this: &LayoutTextCombine) -> bool;
    fn DispatchIsLayoutTextCombine(object: &LayoutObject) -> bool;
}

// cpp: layoutng/internal/layout_text_combine.h:23-115
#[repr(C)]
pub struct LayoutTextCombine {
    block_flow_: LayoutBlockFlow,
    scale_x_: Option<f32>,
    compressed_font_: Member<Font>,
}

impl LayoutTextCombine {
    // C++ constructs this derived object in //src/layoutng_inline. Keep its
    // private storage in the owning crate and expose only the state operations
    // needed by that source implementation.
    // cpp: layoutng/internal/layout_text_combine.h:84-88
    pub fn FromInlineBase(block_flow: LayoutBlockFlow) -> Self {
        let value = Self {
            block_flow_: block_flow,
            scale_x_: None,
            compressed_font_: Member::default(),
        };
        value.SetRuntimeClass(LayoutObjectClass::TextCombine);
        value
    }

    pub fn ScaleXForInline(&self) -> Option<f32> {
        self.scale_x_
    }

    pub fn ResetScaleAndFontForInline(&mut self) {
        self.compressed_font_ = Member::default();
        self.scale_x_ = None;
    }

    pub fn SetScaleXForInline(&mut self, value: f32) {
        self.scale_x_ = Some(value);
    }

    pub fn SetCompressedFontForInline(&mut self, font: *const Font) {
        self.compressed_font_ = Member::from_ptr(font.cast_mut());
    }

    // cpp: layoutng/internal/layout_text_combine.h:27-28
    pub fn new() -> Self {
        unsafe { LayoutTextCombineNew() }
    }

    // cpp: layoutng/internal/layout_text_combine.h:30-33
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.compressed_font_);
        self.block_flow_.Trace(visitor);
    }

    // cpp: layoutng/internal/layout_text_combine.h:35-42
    pub fn DesiredWidth(&self) -> f32 {
        unsafe { LayoutTextCombineDesiredWidth(self) }
    }
    pub fn GetTextContent(&self) -> String {
        unsafe { LayoutTextCombineGetTextContent(self) }
    }
    pub fn CompressedFont(&self) -> *const Font {
        self.CheckIsNotDestroyed();
        self.compressed_font_.Get()
    }
    pub fn SetCompressedFont(&mut self, font: *const Font) {
        unsafe { LayoutTextCombineSetCompressedFont(self, font) }
    }

    // cpp: layoutng/internal/layout_text_combine.h:46-70
    pub fn AdjustOffsetForHitTest(&self, offset: &PhysicalOffset) -> PhysicalOffset {
        unsafe { LayoutTextCombineAdjustOffsetForHitTest(self, offset) }
    }
    pub fn AdjustOffsetForLocalCaretRect(&self, offset: &PhysicalOffset) -> PhysicalOffset {
        unsafe { LayoutTextCombineAdjustOffsetForLocalCaretRect(self, offset) }
    }
    pub fn AdjustRectForBoundingBox(&self, rect: &PhysicalRect) -> PhysicalRect {
        unsafe { LayoutTextCombineAdjustRectForBoundingBox(self, rect) }
    }
    pub fn ComputeTextBoundsRectForHitTest(
        &self,
        text_item: &FragmentItem,
        offset: &PhysicalOffset,
    ) -> PhysicalRect {
        unsafe { LayoutTextCombineComputeTextBoundsRectForHitTest(self, text_item, offset) }
    }
    pub fn RecalcContentsInkOverflow(&self, cursor: &InlineCursor) -> PhysicalRect {
        unsafe { LayoutTextCombineRecalcContentsInkOverflow(self, cursor) }
    }

    // cpp: layoutng/internal/layout_text_combine.h:72-82
    pub fn ResetLayout(&mut self) {
        unsafe { LayoutTextCombineResetLayout(self) }
    }
    pub fn SetScaleX(&mut self, new_scale_x: f32) {
        unsafe { LayoutTextCombineSetScaleX(self, new_scale_x) }
    }
    pub fn UsesScaleX(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.scale_x_.is_some()
    }
    pub fn AssertStyleIsValid(style: &ComputedStyle) {
        unsafe { LayoutTextCombineAssertStyleIsValid(style) }
    }
    pub fn CreateAnonymous(text_child: *mut LayoutText) -> *mut Self {
        unsafe { LayoutTextCombineCreateAnonymous(text_child) }
    }

    // cpp: layoutng/internal/layout_text_combine.h:91-107
    pub fn IsLayoutTextCombine(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        "LayoutTextCombine"
    }
    pub fn ApplyScaleXOffset(&self, offset: &PhysicalOffset) -> PhysicalOffset {
        unsafe { LayoutTextCombineApplyScaleXOffset(self, offset) }
    }
    pub fn ApplyScaleXRect(&self, rect: &PhysicalRect) -> PhysicalRect {
        unsafe { LayoutTextCombineApplyScaleXRect(self, rect) }
    }
    pub fn ApplyScaleXSize(&self, size: &PhysicalSize) -> PhysicalSize {
        unsafe { LayoutTextCombineApplyScaleXSize(self, size) }
    }
    pub fn UnapplyScaleX(&self, offset: &PhysicalOffset) -> PhysicalOffset {
        unsafe { LayoutTextCombineUnapplyScaleX(self, offset) }
    }
    pub fn ComputeInlineSpacing(&self) -> f32 {
        unsafe { LayoutTextCombineComputeInlineSpacing(self) }
    }
    pub fn UsingSyntheticOblique(&self) -> bool {
        unsafe { LayoutTextCombineUsingSyntheticOblique(self) }
    }

    // cpp: layoutng/internal/layout_text_combine.h:117-129
    pub fn IsSupportedMode(mode: WritingMode) -> bool {
        !IsHorizontalTypographicMode(mode)
    }
    pub fn ShouldBeParentOf(object: &LayoutObject) -> bool {
        if !Self::IsSupportedMode(object.StyleRef().GetWritingMode())
            || !object.IsText()
            || object.IsSVGInlineText()
        {
            return false;
        }
        object.StyleRef().HasTextCombine()
    }

    // cpp: layoutng/internal/layout_text_combine.h:131-136
    pub fn AllowFrom(object: &LayoutObject) -> bool {
        unsafe { DispatchIsLayoutTextCombine(object) }
    }
}

// cpp: layoutng/internal/layout_text_combine.h:131-136
impl foundation::DowncastFrom<LayoutObject> for LayoutTextCombine {
    fn AllowFrom(object: &LayoutObject) -> bool {
        LayoutTextCombine::AllowFrom(object)
    }
}

impl foundation::DowncastFrom<LayoutBlockFlow> for LayoutTextCombine {
    fn AllowFrom(flow: &LayoutBlockFlow) -> bool {
        flow.IsLayoutTextCombine()
    }
}

impl Deref for LayoutTextCombine {
    type Target = LayoutBlockFlow;
    fn deref(&self) -> &Self::Target {
        &self.block_flow_
    }
}
impl DerefMut for LayoutTextCombine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.block_flow_
    }
}
