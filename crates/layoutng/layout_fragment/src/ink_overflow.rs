#![allow(non_snake_case, non_camel_case_types)]

use font_engine::TextFragmentPaintInfo;
use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use foundation::{
    gfx, Color, DecorationLineBounds, LayoutUnit, PhysicalOffset, PhysicalRect, PhysicalSize,
    TextDecorationLine, TextDirection, TextEmphasisMark, WritingDirectionMode, WritingMode,
};
use layout_fragment::inline_cursor::InlineCursor;
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::LineLogicalSide;
use layoutng_style::style::shadow_list::ShadowList;
#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicU32, Ordering};

use super::super::used_font::UsedFont;
use crate::inline_paint_context::InlinePaintContext;
use crate::line_relative_rect::LineRelativeOffset;
use crate::text_decoration_info::{
    ComputeTextDecorationFragmentContext, IsSvgText, TextDecorationFragmentContext,
    TextDecorationInfo,
};
use layoutng::internal::text_decoration_offset::TextDecorationOffset;
use layoutng_style::style::applied_text_decoration::AppliedTextDecoration;

pub struct ReadUnsetAsNoneScope;

#[cfg(debug_assertions)]
static READ_UNSET_AS_NONE: AtomicU32 = AtomicU32::new(0);

impl ReadUnsetAsNoneScope {
    pub fn new() -> Self {
        #[cfg(debug_assertions)]
        READ_UNSET_AS_NONE.fetch_add(1, Ordering::Relaxed);
        Self
    }

    pub fn IsActive() -> bool {
        #[cfg(debug_assertions)]
        {
            READ_UNSET_AS_NONE.load(Ordering::Relaxed) != 0
        }
        #[cfg(not(debug_assertions))]
        {
            false
        }
    }
}

impl Drop for ReadUnsetAsNoneScope {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(READ_UNSET_AS_NONE.load(Ordering::Relaxed) > 0);
            READ_UNSET_AS_NONE.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SingleInkOverflow {
    pub ink_overflow: PhysicalRect,
}

impl SingleInkOverflow {
    pub fn new(ink_overflow: &PhysicalRect) -> Self {
        Self {
            ink_overflow: *ink_overflow,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ContainerInkOverflow {
    pub base: SingleInkOverflow,
    pub contents_ink_overflow: PhysicalRect,
}

impl ContainerInkOverflow {
    pub fn new(self_rect: &PhysicalRect, contents: &PhysicalRect) -> Self {
        Self {
            base: SingleInkOverflow::new(self_rect),
            contents_ink_overflow: *contents,
        }
    }

    pub fn SelfAndContentsInkOverflow(&self) -> PhysicalRect {
        let mut result = self.base.ink_overflow;
        result.Unite(&self.contents_ink_overflow);
        result
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InkOverflowType {
    kNotSet,
    kInvalidated,
    kNone,
    kSmallSelf,
    kSelf,
    kSmallContents,
    kContents,
    kSelfAndContents,
}

#[cfg(target_pointer_width = "32")]
type SmallRawValue = u8;
#[cfg(target_pointer_width = "64")]
type SmallRawValue = u16;

#[repr(C)]
#[derive(Clone, Copy)]
union InkOverflowStorage {
    single_: *mut SingleInkOverflow,
    container_: *mut ContainerInkOverflow,
    outsets_: [SmallRawValue; 4],
}

#[repr(C)]
pub struct InkOverflow {
    storage_: InkOverflowStorage,
    #[cfg(debug_assertions)]
    type_: InkOverflowType,
}

#[repr(C)]
struct SameSizeAsInkOverflow {
    pointer: *mut (),
    #[cfg(debug_assertions)]
    type_: InkOverflowType,
}

const _: () = {
    assert!(std::mem::size_of::<InkOverflowStorage>() == std::mem::size_of::<*mut ()>());
    assert!(std::mem::size_of::<InkOverflow>() == std::mem::size_of::<SameSizeAsInkOverflow>());
};

impl Default for InkOverflow {
    fn default() -> Self {
        Self {
            storage_: InkOverflowStorage {
                single_: std::ptr::null_mut(),
            },
            #[cfg(debug_assertions)]
            type_: InkOverflowType::kNotSet,
        }
    }
}

#[cfg(debug_assertions)]
impl Drop for InkOverflow {
    fn drop(&mut self) {
        debug_assert!(matches!(
            self.type_,
            InkOverflowType::kNotSet | InkOverflowType::kNone | InkOverflowType::kInvalidated
        ));
    }
}

fn HasOverflow(rect: &PhysicalRect, size: &PhysicalSize) -> bool {
    if rect.IsEmpty() {
        return false;
    }
    rect.X() < LayoutUnit::default()
        || rect.Y() < LayoutUnit::default()
        || rect.Right() > size.width
        || rect.Bottom() > size.height
}

#[allow(non_snake_case)]
impl InkOverflow {
    pub const kTypeBits: i32 = 3;

    pub fn new(source_type: InkOverflowType, source: &InkOverflow) -> Self {
        source.CheckType(source_type);
        let mut result = Self::default();
        unsafe {
            match source_type {
                InkOverflowType::kNotSet
                | InkOverflowType::kInvalidated
                | InkOverflowType::kNone => {}
                InkOverflowType::kSmallSelf | InkOverflowType::kSmallContents => {
                    result.storage_.outsets_ = source.storage_.outsets_;
                }
                InkOverflowType::kSelf | InkOverflowType::kContents => {
                    let copied = *source.storage_.single_;
                    result.storage_.single_ = Box::into_raw(Box::new(copied));
                }
                InkOverflowType::kSelfAndContents => {
                    let copied = *source.storage_.container_;
                    result.storage_.container_ = Box::into_raw(Box::new(copied));
                }
            }
        }
        result.SetType(source_type);
        result
    }

    pub fn new_moved(source_type: InkOverflowType, source: &mut InkOverflow) -> Self {
        source.CheckType(source_type);
        let mut result = Self::default();
        unsafe {
            match source_type {
                InkOverflowType::kNotSet
                | InkOverflowType::kInvalidated
                | InkOverflowType::kNone => {}
                InkOverflowType::kSmallSelf | InkOverflowType::kSmallContents => {
                    result.storage_.outsets_ = source.storage_.outsets_;
                }
                InkOverflowType::kSelf | InkOverflowType::kContents => {
                    result.storage_.single_ = source.storage_.single_;
                    source.storage_.single_ = std::ptr::null_mut();
                }
                InkOverflowType::kSelfAndContents => {
                    result.storage_.container_ = source.storage_.container_;
                    source.storage_.container_ = std::ptr::null_mut();
                }
            }
        }
        result.SetType(source_type);
        result
    }

    fn CheckType(&self, type_: InkOverflowType) {
        #[cfg(debug_assertions)]
        debug_assert_eq!(type_, self.type_);
        #[cfg(not(debug_assertions))]
        let _ = type_;
    }

    fn SetType(&mut self, type_: InkOverflowType) -> InkOverflowType {
        #[cfg(debug_assertions)]
        {
            self.type_ = type_;
        }
        type_
    }

    pub fn Reset(&mut self, type_: InkOverflowType) -> InkOverflowType {
        self.ResetTo(type_, InkOverflowType::kNone)
    }

    pub fn Invalidate(&mut self, type_: InkOverflowType) -> InkOverflowType {
        self.ResetTo(type_, InkOverflowType::kInvalidated)
    }

    fn ResetTo(&mut self, type_: InkOverflowType, new_type: InkOverflowType) -> InkOverflowType {
        self.CheckType(type_);
        debug_assert!(matches!(
            new_type,
            InkOverflowType::kNotSet | InkOverflowType::kNone | InkOverflowType::kInvalidated
        ));
        unsafe {
            match type_ {
                InkOverflowType::kNotSet
                | InkOverflowType::kInvalidated
                | InkOverflowType::kNone
                | InkOverflowType::kSmallSelf
                | InkOverflowType::kSmallContents => {}
                InkOverflowType::kSelf | InkOverflowType::kContents => {
                    drop(Box::from_raw(self.storage_.single_));
                }
                InkOverflowType::kSelfAndContents => {
                    drop(Box::from_raw(self.storage_.container_));
                }
            }
        }
        self.SetType(new_type)
    }

    fn FromOutsets(&self, size: &PhysicalSize) -> PhysicalRect {
        let outsets = unsafe { self.storage_.outsets_ };
        let left = LayoutUnit::FromRawValue(outsets[0] as i32);
        let top = LayoutUnit::FromRawValue(outsets[1] as i32);
        PhysicalRect::new(
            PhysicalOffset::new(-left, -top),
            PhysicalSize::new(
                left + size.width + LayoutUnit::FromRawValue(outsets[2] as i32),
                top + size.height + LayoutUnit::FromRawValue(outsets[3] as i32),
            ),
        )
    }

    pub fn SelfRect(&self, type_: InkOverflowType, size: &PhysicalSize) -> PhysicalRect {
        self.CheckType(type_);
        match type_ {
            InkOverflowType::kNotSet
            | InkOverflowType::kInvalidated
            | InkOverflowType::kNone
            | InkOverflowType::kSmallContents
            | InkOverflowType::kContents => PhysicalRect::new(PhysicalOffset::default(), *size),
            InkOverflowType::kSmallSelf => self.FromOutsets(size),
            InkOverflowType::kSelf | InkOverflowType::kSelfAndContents => unsafe {
                (*self.storage_.single_).ink_overflow
            },
        }
    }

    pub fn Contents(&self, type_: InkOverflowType, size: &PhysicalSize) -> PhysicalRect {
        self.CheckType(type_);
        match type_ {
            InkOverflowType::kNotSet
            | InkOverflowType::kInvalidated
            | InkOverflowType::kNone
            | InkOverflowType::kSmallSelf
            | InkOverflowType::kSelf => PhysicalRect::default(),
            InkOverflowType::kSmallContents => self.FromOutsets(size),
            InkOverflowType::kContents => unsafe { (*self.storage_.single_).ink_overflow },
            InkOverflowType::kSelfAndContents => unsafe {
                (*self.storage_.container_).contents_ink_overflow
            },
        }
    }

    pub fn SelfAndContents(&self, type_: InkOverflowType, size: &PhysicalSize) -> PhysicalRect {
        self.CheckType(type_);
        match type_ {
            InkOverflowType::kNotSet | InkOverflowType::kInvalidated | InkOverflowType::kNone => {
                PhysicalRect::new(PhysicalOffset::default(), *size)
            }
            InkOverflowType::kSmallSelf | InkOverflowType::kSmallContents => self.FromOutsets(size),
            InkOverflowType::kSelf | InkOverflowType::kContents => unsafe {
                (*self.storage_.single_).ink_overflow
            },
            InkOverflowType::kSelfAndContents => unsafe {
                (*self.storage_.container_).SelfAndContentsInkOverflow()
            },
        }
    }

    fn TrySetOutsets(
        &mut self,
        type_: InkOverflowType,
        left_outset: LayoutUnit,
        top_outset: LayoutUnit,
        right_outset: LayoutUnit,
        bottom_outset: LayoutUnit,
    ) -> bool {
        self.CheckType(type_);
        let max_small_value = LayoutUnit::FromRawValue(SmallRawValue::MAX as i32);
        if left_outset > max_small_value {
            return false;
        }
        if top_outset > max_small_value {
            return false;
        }
        if right_outset > max_small_value {
            return false;
        }
        if bottom_outset > max_small_value {
            return false;
        }
        self.Reset(type_);
        self.storage_.outsets_ = [
            left_outset.RawValue() as SmallRawValue,
            top_outset.RawValue() as SmallRawValue,
            right_outset.RawValue() as SmallRawValue,
            bottom_outset.RawValue() as SmallRawValue,
        ];
        true
    }

    fn SetSingle(
        &mut self,
        type_: InkOverflowType,
        ink_overflow: &PhysicalRect,
        size: &PhysicalSize,
        new_type: InkOverflowType,
        new_small_type: InkOverflowType,
    ) -> InkOverflowType {
        self.CheckType(type_);
        debug_assert!(HasOverflow(ink_overflow, size));
        let left_outset = (-ink_overflow.X()).ClampNegativeToZero();
        let top_outset = (-ink_overflow.Y()).ClampNegativeToZero();
        let right_outset = (ink_overflow.Right() - size.width).ClampNegativeToZero();
        let bottom_outset = (ink_overflow.Bottom() - size.height).ClampNegativeToZero();
        if self.TrySetOutsets(type_, left_outset, top_outset, right_outset, bottom_outset) {
            return self.SetType(new_small_type);
        }
        let adjusted_ink_overflow = PhysicalRect::new(
            PhysicalOffset::new(-left_outset, -top_outset),
            PhysicalSize::new(
                left_outset + size.width + right_outset,
                top_outset + size.height + bottom_outset,
            ),
        );
        match type_ {
            InkOverflowType::kSelfAndContents => {
                self.Reset(type_);
                self.storage_.single_ =
                    Box::into_raw(Box::new(SingleInkOverflow::new(&adjusted_ink_overflow)));
                self.SetType(new_type)
            }
            InkOverflowType::kNotSet
            | InkOverflowType::kInvalidated
            | InkOverflowType::kNone
            | InkOverflowType::kSmallSelf
            | InkOverflowType::kSmallContents => {
                self.storage_.single_ =
                    Box::into_raw(Box::new(SingleInkOverflow::new(&adjusted_ink_overflow)));
                self.SetType(new_type)
            }
            InkOverflowType::kSelf | InkOverflowType::kContents => {
                unsafe { &mut *self.storage_.single_ }.ink_overflow = adjusted_ink_overflow;
                self.SetType(new_type)
            }
        }
    }

    pub fn SetSelf(
        &mut self,
        type_: InkOverflowType,
        ink_overflow: &PhysicalRect,
        size: &PhysicalSize,
    ) -> InkOverflowType {
        self.CheckType(type_);
        if !HasOverflow(ink_overflow, size) {
            return self.Reset(type_);
        }
        self.SetSingle(
            type_,
            ink_overflow,
            size,
            InkOverflowType::kSelf,
            InkOverflowType::kSmallSelf,
        )
    }

    pub fn SetContents(
        &mut self,
        type_: InkOverflowType,
        ink_overflow: &PhysicalRect,
        size: &PhysicalSize,
    ) -> InkOverflowType {
        self.CheckType(type_);
        if !HasOverflow(ink_overflow, size) {
            return self.Reset(type_);
        }
        self.SetSingle(
            type_,
            ink_overflow,
            size,
            InkOverflowType::kContents,
            InkOverflowType::kSmallContents,
        )
    }

    pub fn Set(
        &mut self,
        type_: InkOverflowType,
        self_rect: &PhysicalRect,
        contents: &PhysicalRect,
        size: &PhysicalSize,
    ) -> InkOverflowType {
        self.CheckType(type_);
        if !HasOverflow(self_rect, size) {
            if !HasOverflow(contents, size) {
                return self.Reset(type_);
            }
            return self.SetSingle(
                type_,
                contents,
                size,
                InkOverflowType::kContents,
                InkOverflowType::kSmallContents,
            );
        }
        if !HasOverflow(contents, size) {
            return self.SetSingle(
                type_,
                self_rect,
                size,
                InkOverflowType::kSelf,
                InkOverflowType::kSmallSelf,
            );
        }
        match type_ {
            InkOverflowType::kSelf | InkOverflowType::kContents => {
                self.Reset(type_);
                self.storage_.container_ =
                    Box::into_raw(Box::new(ContainerInkOverflow::new(self_rect, contents)));
                self.SetType(InkOverflowType::kSelfAndContents)
            }
            InkOverflowType::kNotSet
            | InkOverflowType::kInvalidated
            | InkOverflowType::kNone
            | InkOverflowType::kSmallSelf
            | InkOverflowType::kSmallContents => {
                self.storage_.container_ =
                    Box::into_raw(Box::new(ContainerInkOverflow::new(self_rect, contents)));
                self.SetType(InkOverflowType::kSelfAndContents)
            }
            InkOverflowType::kSelfAndContents => {
                let container = unsafe { &mut *self.storage_.container_ };
                container.base.ink_overflow = *self_rect;
                container.contents_ink_overflow = *contents;
                InkOverflowType::kSelfAndContents
            }
        }
    }

    // Source: core/layout/ink_overflow.cc:351-370.
    pub fn SetTextInkOverflow(
        &mut self,
        type_: InkOverflowType,
        cursor: &InlineCursor,
        text_info: &TextFragmentPaintInfo,
        style: &ComputedStyle,
        rect_in_container: &PhysicalRect,
        inline_context: Option<&InlinePaintContext>,
        ink_overflow_out: &mut PhysicalRect,
    ) -> InkOverflowType {
        self.CheckType(type_);
        debug_assert!(matches!(
            type_,
            InkOverflowType::kNotSet | InkOverflowType::kInvalidated
        ));
        let Some(mut ink_overflow) = Self::ComputeTextInkOverflow(
            cursor,
            text_info,
            style,
            rect_in_container,
            inline_context,
        ) else {
            *ink_overflow_out =
                PhysicalRect::new(PhysicalOffset::default(), rect_in_container.size);
            return self.Reset(type_);
        };
        ink_overflow.ExpandEdgesToPixelBoundaries();
        *ink_overflow_out = ink_overflow;
        self.SetSelf(type_, &ink_overflow, &rect_in_container.size)
    }

    // Source: core/layout/ink_overflow.cc:372-436.
    pub fn SetSvgTextInkOverflow(
        &mut self,
        type_: InkOverflowType,
        cursor: &InlineCursor,
        text_info: &TextFragmentPaintInfo,
        style: &ComputedStyle,
        rect: &gfx::RectF,
        scaling_factor: f32,
        length_adjust_scale: f32,
        transform: &AffineTransform,
        ink_overflow_out: &mut PhysicalRect,
    ) -> InkOverflowType {
        self.CheckType(type_);
        debug_assert!(matches!(
            type_,
            InkOverflowType::kNotSet | InkOverflowType::kInvalidated
        ));
        let item_size = if style.IsHorizontalWritingMode() {
            PhysicalSize::new(
                LayoutUnit::from_f32(rect.width() / length_adjust_scale),
                LayoutUnit::from_f32(rect.height()),
            )
        } else {
            PhysicalSize::new(
                LayoutUnit::from_f32(rect.width()),
                LayoutUnit::from_f32(rect.height() / length_adjust_scale),
            )
        };
        let mut ink_overflow = Self::ComputeTextInkOverflow(
            cursor,
            text_info,
            style,
            &PhysicalRect::new(PhysicalOffset::default(), item_size),
            None,
        );
        let needs_transform = scaling_factor != 1.0 || !transform.IsIdentity();
        let mut unscaled_size = PhysicalSize::FromSizeFRound(&rect.size());
        unscaled_size.ScaleFloat(1.0 / scaling_factor);

        if ink_overflow.is_none() {
            if needs_transform {
                let mut transformed_rect = transform.MapRect(*rect);
                transformed_rect.Offset(-rect.x(), -rect.y());
                transformed_rect.Scale(1.0 / scaling_factor, 1.0 / scaling_factor);
                *ink_overflow_out = PhysicalRect::EnclosingRect(&transformed_rect);
                ink_overflow_out.ExpandEdgesToPixelBoundaries();
                return self.SetSelf(type_, ink_overflow_out, &unscaled_size);
            }
            *ink_overflow_out = PhysicalRect::new(PhysicalOffset::default(), unscaled_size);
            ink_overflow_out.ExpandEdgesToPixelBoundaries();
            return self.Reset(type_);
        }

        let ink_overflow = ink_overflow.as_mut().expect("checked above");
        if style.IsHorizontalWritingMode() {
            ink_overflow.offset.left *= length_adjust_scale;
            ink_overflow.size.width *= length_adjust_scale;
        } else {
            ink_overflow.offset.top *= length_adjust_scale;
            ink_overflow.size.height *= length_adjust_scale;
        }
        if needs_transform {
            let mut transformed_rect = gfx::RectF::from(*ink_overflow);
            transformed_rect.Offset(rect.x(), rect.y());
            transformed_rect = transform.MapRect(transformed_rect);
            transformed_rect.Offset(-rect.x(), -rect.y());
            transformed_rect.Scale(1.0 / scaling_factor, 1.0 / scaling_factor);
            *ink_overflow_out = PhysicalRect::EnclosingRect(&transformed_rect);
            ink_overflow_out.ExpandEdgesToPixelBoundaries();
            return self.SetSelf(type_, ink_overflow_out, &unscaled_size);
        }
        *ink_overflow_out = *ink_overflow;
        ink_overflow_out.ExpandEdgesToPixelBoundaries();
        self.SetSelf(type_, ink_overflow, &unscaled_size)
    }

    // Source: core/layout/ink_overflow.cc:439-489. The Paint-owned
    // TextDecorationInfo sources are compiled once at the reciprocal assembly
    // boundary, so this Layout owner consumes their exact geometry and the
    // retained InlinePaintContext without copying either type.
    fn ComputeTextInkOverflow(
        cursor: &InlineCursor,
        text_info: &TextFragmentPaintInfo,
        style: &ComputedStyle,
        rect_in_container: &PhysicalRect,
        inline_context: Option<&InlinePaintContext>,
    ) -> Option<PhysicalRect> {
        let item = unsafe { &*cursor.CurrentItem() };
        let used_font = item.GetUsedFont();
        let text_ink_bounds = used_font.TextInkBounds(text_info);
        let mut ink_overflow = LogicalRect::EnclosingRect(&text_ink_bounds);
        let writing_mode = style.GetWritingMode();

        ink_overflow.offset.block_offset += used_font.FixedAscent();
        let stroke_width = style.TextStrokeWidth();
        if stroke_width != 0.0 {
            ink_overflow.Inflate(LayoutUnit::FromFloatCeil(stroke_width / 2.0));
        }

        let decoration_rect = ComputeDecorationOverflow(
            cursor,
            style,
            &used_font,
            &rect_in_container.offset,
            &ink_overflow,
            inline_context,
            writing_mode,
        );
        ink_overflow.Unite(&decoration_rect);

        if style.GetTextEmphasisMark() != TextEmphasisMark::kNone {
            ink_overflow =
                ComputeEmphasisMarkOverflow(style, &rect_in_container.size, &ink_overflow);
        }
        let text_shadow = style.TextShadow();
        if !text_shadow.is_null() {
            ExpandForShadowOverflow(&mut ink_overflow, unsafe { &*text_shadow }, writing_mode);
        }

        let mut local_ink_overflow = WritingModeConverter::new(
            WritingDirectionMode::new(writing_mode, TextDirection::kLtr),
            rect_in_container.size,
        )
        .ToPhysicalRect(ink_overflow);
        if !HasOverflow(&local_ink_overflow, &rect_in_container.size) {
            return None;
        }
        local_ink_overflow.Unite(&PhysicalRect::new(
            PhysicalOffset::default(),
            rect_in_container.size,
        ));
        Some(local_ink_overflow)
    }
}

// Source: core/layout/ink_overflow.cc:664-727.
fn ComputeAppliedDecorationOverflow(
    style: &ComputedStyle,
    used_font: &UsedFont,
    offset_in_container: &PhysicalOffset,
    ink_overflow: &LogicalRect,
    inline_context: Option<&InlinePaintContext>,
    fragment_cursor: Option<&InlineCursor>,
    decoration_override: Option<&AppliedTextDecoration>,
) -> LogicalRect {
    debug_assert!(style.HasAppliedTextDecorations() || decoration_override.is_some());

    let fragment_context = fragment_cursor.and_then(|cursor| {
        let item = cursor.CurrentItem();
        if !item.is_null()
            && unsafe { &*item }.IsText()
            && cursor.HasRoot()
            && TextDecorationInfo::NeedsFragmentContextForInset(style)
        {
            Some(ComputeTextDecorationFragmentContext(cursor))
        } else {
            None
        }
    });
    let is_svg_text = IsSvgText(decoration_override.is_some());
    let mut decoration_info = TextDecorationInfo::new(
        LineRelativeOffset::CreateFromBoxOrigin(offset_in_container),
        ink_overflow.size.inline_size,
        style,
        used_font,
        inline_context,
        TextDecorationLine::kNone,
        Color::default(),
        decoration_override,
        is_svg_text,
        1.0,
        fragment_context.unwrap_or_else(TextDecorationFragmentContext::default),
        fragment_context.is_none(),
    );
    let decoration_offset = TextDecorationOffset::new(style);
    let mut accumulated_bound = gfx::RectF::default();
    for index in 0..decoration_info.AppliedDecorationCount() {
        let decoration = decoration_info.ResolveDecorationAt(index);
        if !decoration.HasFontData() {
            continue;
        }
        if decoration.HasUnderline() {
            accumulated_bound.Union(DecorationLineBounds(
                &decoration_info.ComputeUnderlineLineData(&decoration, &decoration_offset),
            ));
        }
        if decoration.HasOverline() {
            accumulated_bound.Union(DecorationLineBounds(
                &decoration_info.ComputeOverlineLineData(&decoration, &decoration_offset),
            ));
        }
        if decoration.HasLineThrough() {
            accumulated_bound.Union(DecorationLineBounds(
                &decoration_info.ComputeLineThroughLineData(&decoration),
            ));
        }
        if decoration.HasSpellingOrGrammarError() {
            accumulated_bound.Union(DecorationLineBounds(
                &decoration_info
                    .ComputeSpellingOrGrammarErrorLineData(&decoration, &decoration_offset),
            ));
        }
    }
    accumulated_bound.Offset(
        -offset_in_container.left.ToFloat(),
        -offset_in_container.top.ToFloat(),
    );
    LogicalRect::EnclosingRect(&accumulated_bound)
}

// Source: core/layout/ink_overflow.cc:557-591. Marker-specific overflow from
// lines 593-660 remains separate until the document-marker inputs are exposed.
fn ComputeDecorationOverflow(
    cursor: &InlineCursor,
    style: &ComputedStyle,
    used_font: &UsedFont,
    container_offset: &PhysicalOffset,
    ink_overflow: &LogicalRect,
    inline_context: Option<&InlinePaintContext>,
    writing_mode: WritingMode,
) -> LogicalRect {
    let mut accumulated_bound = *ink_overflow;
    if used_font.PrimaryFont().is_null() {
        return accumulated_bound;
    }
    if style.HasAppliedTextDecorations() {
        accumulated_bound = ComputeAppliedDecorationOverflow(
            style,
            used_font,
            container_offset,
            ink_overflow,
            inline_context,
            Some(cursor),
            None,
        );
    }

    let layout_object = cursor.Current().GetLayoutObject();
    if !layout_object.is_null() && unsafe { &*layout_object }.IsSelected() {
        let selection_style = style.HighlightData().Selection();
        if !selection_style.is_null() {
            let selection_style = unsafe { &*selection_style };
            if selection_style.HasAppliedTextDecorations() {
                let selection_bound = ComputeAppliedDecorationOverflow(
                    selection_style,
                    used_font,
                    container_offset,
                    ink_overflow,
                    inline_context,
                    Some(cursor),
                    None,
                );
                accumulated_bound.Unite(&selection_bound);
            }
            let text_shadow = selection_style.TextShadow();
            if !text_shadow.is_null() {
                ExpandForShadowOverflow(
                    &mut accumulated_bound,
                    unsafe { &*text_shadow },
                    writing_mode,
                );
            }
        }
    }
    accumulated_bound
}

// Source: core/layout/ink_overflow.cc:519-541.
fn ComputeEmphasisMarkOverflow(
    style: &ComputedStyle,
    size: &PhysicalSize,
    ink_overflow_in: &LogicalRect,
) -> LogicalRect {
    let font = style.GetFont();
    debug_assert!(!font.is_null());
    let emphasis_mark_height = font_engine::fonts::font_services::FontEmphasisMarkHeight(
        unsafe { &*font },
        style.TextEmphasisMarkString(),
    );
    debug_assert!(emphasis_mark_height >= LayoutUnit::default());

    let mut ink_overflow = *ink_overflow_in;
    if style.GetTextEmphasisLineLogicalSide() == LineLogicalSide::kOver {
        ink_overflow
            .ShiftBlockStartEdgeTo(ink_overflow.offset.block_offset.min(-emphasis_mark_height));
    } else {
        let logical_height = if style.IsHorizontalWritingMode() {
            size.height
        } else {
            size.width
        };
        ink_overflow.ShiftBlockEndEdgeTo(
            ink_overflow
                .BlockEndOffset()
                .max(logical_height + emphasis_mark_height),
        );
    }
    ink_overflow
}

// Source: core/layout/ink_overflow.cc:543-554.
fn ExpandForShadowOverflow(
    ink_overflow: &mut LogicalRect,
    text_shadow: &ShadowList,
    writing_mode: WritingMode,
) {
    let outsets = PhysicalBoxStrut::Enclosing(&text_shadow.RectOutsetsIncludingOriginal())
        .ConvertToLineLogical(WritingDirectionMode::new(writing_mode, TextDirection::kLtr));
    ink_overflow.ExpandEdges(
        outsets.line_over.ClampNegativeToZero(),
        outsets.inline_end.ClampNegativeToZero(),
        outsets.line_under.ClampNegativeToZero(),
        outsets.inline_start.ClampNegativeToZero(),
    );
}

// Source: core/layout/ink_overflow.cc:519-541.
#[unsafe(no_mangle)]
pub extern "Rust" fn InkOverflowComputeEmphasisMarkOverflowForTextCombine(
    style: &ComputedStyle,
    size: &PhysicalSize,
    ink_overflow_in: &LogicalRect,
) -> LogicalRect {
    ComputeEmphasisMarkOverflow(style, size, ink_overflow_in)
}

// Source: core/layout/ink_overflow.cc:543-554.
#[unsafe(no_mangle)]
pub extern "Rust" fn InkOverflowExpandForShadowOverflowForTextCombine(
    ink_overflow: &mut LogicalRect,
    text_shadow: &ShadowList,
    writing_mode: WritingMode,
) {
    ExpandForShadowOverflow(ink_overflow, text_shadow, writing_mode);
}

// Source: core/layout/ink_overflow.cc:557-591,664-727.
#[unsafe(no_mangle)]
pub extern "Rust" fn InkOverflowComputeDecorationOverflowForTextCombine(
    cursor: &InlineCursor,
    style: &ComputedStyle,
    used_font: &UsedFont,
    offset: &PhysicalOffset,
    overflow: &LogicalRect,
    inline_context: *const (),
    writing_mode: WritingMode,
) -> LogicalRect {
    let inline_context = if inline_context.is_null() {
        None
    } else {
        Some(unsafe { &*inline_context.cast::<InlinePaintContext>() })
    };
    ComputeDecorationOverflow(
        cursor,
        style,
        used_font,
        offset,
        overflow,
        inline_context,
        writing_mode,
    )
}
