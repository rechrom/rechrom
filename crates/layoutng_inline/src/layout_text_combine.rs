// C++: layoutng_inline/layout_text_combine.cc. The value type and virtual
// interface are owned by //src/layoutng/internal/layout_text_combine.h.
#![allow(non_snake_case)]

use font_engine::fonts::font_orientation::IsVerticalBaseline;
use font_engine::{Font, FontOrientation};
use foundation::String;
use foundation::{
    EDisplay, LayoutUnit, MakeGarbageCollected, PhysicalOffset, PhysicalRect, PhysicalSize,
    TextDecorationLine, TextDirection, TextEmphasisMark, WritingDirectionMode, WritingMode,
};
use layoutng::internal::layout_block_flow::LayoutBlockFlow;
use layoutng::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng::internal::layout_text::LayoutText;
use layoutng::internal::layout_text_combine::LayoutTextCombine;
use layoutng::internal::text_combine_style_service::AdjustStyleForTextCombine;
use layoutng::internal::used_font::UsedFont;
use layoutng_fragment_tree::inline_cursor::InlineCursor;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;
use layoutng_style::style::anonymous_style::CreateAnonymousStyleBuilderWithDisplay;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::shadow_list::ShadowList;

// The owning layoutng InkOverflow text methods are declared but have no
// implementation in the supplied package. Keep these typed dependencies
// unresolved until that owner supplies them; no local paint approximation.
unsafe extern "Rust" {
    fn InkOverflowComputeDecorationOverflowForTextCombine(
        cursor: &InlineCursor,
        style: &ComputedStyle,
        used_font: &UsedFont,
        offset: &PhysicalOffset,
        overflow: &LogicalRect,
        inline_context: *const (),
        writing_mode: WritingMode,
    ) -> LogicalRect;
    fn InkOverflowComputeEmphasisMarkOverflowForTextCombine(
        style: &ComputedStyle,
        size: &PhysicalSize,
        overflow: &LogicalRect,
    ) -> LogicalRect;
    fn InkOverflowExpandForShadowOverflowForTextCombine(
        overflow: &mut LogicalRect,
        shadow: &ShadowList,
        writing_mode: WritingMode,
    );
}

// cpp: layoutng_inline/layout_text_combine.cc:23-25
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineNew() -> LayoutTextCombine {
    LayoutTextCombine::FromInlineBase(LayoutBlockFlow::new(std::ptr::null_mut()))
}

// cpp: layoutng_inline/layout_text_combine.cc:27-41
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineCreateAnonymous(
    text_child: *mut LayoutText,
) -> *mut LayoutTextCombine {
    debug_assert!(LayoutTextCombine::ShouldBeParentOf(unsafe { &*text_child }));
    let layout_object = MakeGarbageCollected(LayoutTextCombine::new());
    let combined = unsafe { &mut *layout_object };
    combined.SetInputOwnerForAnonymous(unsafe { &*text_child });
    let child_style = unsafe { &*text_child }.StyleRef();
    let mut builder = CreateAnonymousStyleBuilderWithDisplay(
        child_style,
        EDisplay::kInlineBlock,
        child_style.AppliedTextDecorationData(),
    );
    unsafe { AdjustStyleForTextCombine(&mut builder) };
    combined.SetStyle(builder.TakeStyle());
    combined.AddChildDefault(text_child.cast::<LayoutObject>());
    LayoutTextCombine::AssertStyleIsValid(unsafe { &*text_child }.StyleRef());
    layout_object
}

// cpp: layoutng_inline/layout_text_combine.cc:43-47
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineGetTextContent(this: &LayoutTextCombine) -> String {
    this.CheckIsNotDestroyed();
    debug_assert!(!this.NeedsCollectInlines() && !this.GetInlineNodeData().is_null());
    unsafe { &*this.GetInlineNodeData() }
        .ItemsData(false)
        .text_content
        .clone()
}

// cpp: layoutng_inline/layout_text_combine.cc:49-65
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineDesiredWidth(this: &LayoutTextCombine) -> f32 {
    this.CheckIsNotDestroyed();
    let style = this.StyleRef();
    debug_assert_eq!(
        unsafe { &*style.GetFont() }
            .GetFontDescription()
            .Orientation(),
        FontOrientation::kHorizontal
    );
    let one_em = style.ComputedFontSize();
    let parent = unsafe { &*this.Parent() };
    let decorated = parent.StyleRef().TextDecorationsInEffect()
        & (TextDecorationLine::kUnderline | TextDecorationLine::kOverline);
    if decorated != TextDecorationLine::kNone {
        return one_em;
    }
    one_em * 1.1
}

// cpp: layoutng_inline/layout_text_combine.cc:67-74
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineComputeInlineSpacing(this: &LayoutTextCombine) -> f32 {
    this.CheckIsNotDestroyed();
    debug_assert_eq!(
        unsafe { &*this.StyleRef().GetFont() }
            .GetFontDescription()
            .Orientation(),
        FontOrientation::kHorizontal
    );
    debug_assert!(this.ScaleXForInline().is_some());
    let line_height = this
        .StyleRef()
        .GetFontHeightForDefaultBaseline()
        .LineHeight();
    (line_height.ToFloat() - this.DesiredWidth()) / 2.0
}

// cpp: layoutng_inline/layout_text_combine.cc:76-83
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineApplyScaleXOffset(
    this: &LayoutTextCombine,
    offset: &PhysicalOffset,
) -> PhysicalOffset {
    this.CheckIsNotDestroyed();
    let scale = this.ScaleXForInline().expect("scale_x is set by layout");
    let spacing = this.ComputeInlineSpacing();
    PhysicalOffset::new(
        LayoutUnit::FromFloatRound(offset.left.ToFloat() * scale + spacing),
        offset.top,
    )
}

// cpp: layoutng_inline/layout_text_combine.cc:85-89
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineApplyScaleXRect(
    this: &LayoutTextCombine,
    rect: &PhysicalRect,
) -> PhysicalRect {
    this.CheckIsNotDestroyed();
    debug_assert!(this.ScaleXForInline().is_some());
    PhysicalRect::new(
        this.ApplyScaleXOffset(&rect.offset),
        this.ApplyScaleXSize(&rect.size),
    )
}

// cpp: layoutng_inline/layout_text_combine.cc:91-95
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineApplyScaleXSize(
    this: &LayoutTextCombine,
    size: &PhysicalSize,
) -> PhysicalSize {
    this.CheckIsNotDestroyed();
    let scale = this.ScaleXForInline().expect("scale_x is set by layout");
    PhysicalSize::new(
        LayoutUnit::FromFloatRound(size.width.ToFloat() * scale),
        size.height,
    )
}

// cpp: layoutng_inline/layout_text_combine.cc:97-104
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineUnapplyScaleX(
    this: &LayoutTextCombine,
    offset: &PhysicalOffset,
) -> PhysicalOffset {
    this.CheckIsNotDestroyed();
    let scale = this.ScaleXForInline().expect("scale_x is set by layout");
    let spacing = this.ComputeInlineSpacing();
    PhysicalOffset::new(
        LayoutUnit::FromFloatRound((offset.left.ToFloat() - spacing) / scale),
        offset.top,
    )
}

// cpp: layoutng_inline/layout_text_combine.cc:106-113
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineAdjustOffsetForHitTest(
    this: &LayoutTextCombine,
    offset: &PhysicalOffset,
) -> PhysicalOffset {
    this.CheckIsNotDestroyed();
    if this.ScaleXForInline().is_none() {
        return *offset;
    }
    this.UnapplyScaleX(offset)
}

// cpp: layoutng_inline/layout_text_combine.cc:115-122
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineAdjustOffsetForLocalCaretRect(
    this: &LayoutTextCombine,
    offset: &PhysicalOffset,
) -> PhysicalOffset {
    this.CheckIsNotDestroyed();
    if this.ScaleXForInline().is_none() {
        return *offset;
    }
    this.ApplyScaleXOffset(offset)
}

// cpp: layoutng_inline/layout_text_combine.cc:124-132
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineAdjustRectForBoundingBox(
    this: &LayoutTextCombine,
    rect: &PhysicalRect,
) -> PhysicalRect {
    this.CheckIsNotDestroyed();
    if this.ScaleXForInline().is_none() {
        return *rect;
    }
    this.ApplyScaleXRect(rect)
}

// cpp: layoutng_inline/layout_text_combine.cc:134-143
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineComputeTextBoundsRectForHitTest(
    this: &LayoutTextCombine,
    text_item: &layoutng_fragment_tree::fragment_item::FragmentItem,
    inline_root_offset: &PhysicalOffset,
) -> PhysicalRect {
    debug_assert!(text_item.IsText());
    let mut rect = text_item.SelfInkOverflowRect();
    rect.Move(text_item.OffsetInContainerFragment());
    rect = this.AdjustRectForBoundingBox(&rect);
    rect.Move(inline_root_offset);
    rect
}

// cpp: layoutng_inline/layout_text_combine.cc:151-197
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineRecalcContentsInkOverflow(
    this: &LayoutTextCombine,
    cursor: &InlineCursor,
) -> PhysicalRect {
    this.CheckIsNotDestroyed();
    let style = unsafe { &*this.Parent() }.StyleRef();
    let font = unsafe { &*style.GetFont() };
    debug_assert!(IsVerticalBaseline(font.GetFontDescription().Orientation()));

    let one_em = style.ComputedFontSizeAsFixedValue();
    let text_metrics = style.GetFontHeightForDefaultBaseline();
    let text_rect = PhysicalRect::new(
        PhysicalOffset::default(),
        PhysicalSize::new(one_em, text_metrics.LineHeight()),
    );
    let mut ink_overflow = LogicalRect::from_units(
        text_rect.offset.left,
        text_rect.offset.top,
        text_rect.size.width,
        text_rect.size.height,
    );

    let writing_mode = style.GetWritingMode();
    if style.HasAppliedTextDecorations() {
        let decoration = unsafe {
            InkOverflowComputeDecorationOverflowForTextCombine(
                cursor,
                style,
                &UsedFont::new(font, 1.0),
                &PhysicalOffset::default(),
                &ink_overflow,
                std::ptr::null(),
                writing_mode,
            )
        };
        ink_overflow.Unite(&decoration);
    }
    if style.GetTextEmphasisMark() != TextEmphasisMark::kNone {
        ink_overflow = unsafe {
            InkOverflowComputeEmphasisMarkOverflowForTextCombine(
                style,
                &text_rect.size,
                &ink_overflow,
            )
        };
    }
    let text_shadow = style.TextShadow();
    if !text_shadow.is_null() {
        unsafe {
            InkOverflowExpandForShadowOverflowForTextCombine(
                &mut ink_overflow,
                &*text_shadow,
                writing_mode,
            );
        }
    }

    let mut local_ink_overflow = WritingModeConverter::new(
        WritingDirectionMode::new(writing_mode, TextDirection::kLtr),
        text_rect.size,
    )
    .ToPhysicalRect(ink_overflow);
    local_ink_overflow.ExpandEdgesToPixelBoundaries();
    local_ink_overflow
}

// cpp: layoutng_inline/layout_text_combine.cc:145-149
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineResetLayout(this: &mut LayoutTextCombine) {
    this.CheckIsNotDestroyed();
    this.ResetScaleAndFontForInline();
}

// cpp: layoutng_inline/layout_text_combine.cc:199-207
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineSetScaleX(this: &mut LayoutTextCombine, scale: f32) {
    this.CheckIsNotDestroyed();
    debug_assert!(scale > 0.0);
    debug_assert!(this.ScaleXForInline().is_none());
    debug_assert!(this.CompressedFont().is_null());
    this.SetScaleXForInline(scale);
}

// cpp: layoutng_inline/layout_text_combine.cc:209-214
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineSetCompressedFont(
    this: &mut LayoutTextCombine,
    font: *const Font,
) {
    this.CheckIsNotDestroyed();
    debug_assert!(this.CompressedFont().is_null());
    debug_assert!(this.ScaleXForInline().is_none());
    this.SetCompressedFontForInline(font);
}

// cpp: layoutng_inline/layout_text_combine.cc:216-223
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineUsingSyntheticOblique(this: &LayoutTextCombine) -> bool {
    this.CheckIsNotDestroyed();
    unsafe { &*unsafe { &*this.Parent() }.StyleRef().GetFont() }
        .GetFontDescription()
        .IsSyntheticOblique()
}

// The C++ dynamic cast is virtual; the owner records the most-derived type
// because Rust does not have a C++ vtable for LayoutObject.
// cpp: layoutng/internal/layout_text_combine.h:131-136
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchIsLayoutTextCombine(object: &LayoutObject) -> bool {
    object.RuntimeClass() == LayoutObjectClass::TextCombine
}
