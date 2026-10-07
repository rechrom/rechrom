#![allow(non_snake_case, non_camel_case_types)]

use font_engine::TextFragmentPaintInfo;
use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use foundation::{gfx, LayoutUnit, PhysicalOffset, PhysicalRect, PhysicalSize, WritingMode};
use layoutng_fragment_tree::fragment_item::FragmentItem;
use layoutng_fragment_tree::inline_cursor::InlineCursor;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_style::style::applied_text_decoration::AppliedTextDecoration;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::shadow_list::ShadowList;
#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicU32, Ordering};

use super::document_marker::{DocumentMarkerVector, MarkerType};
use super::layout_node_metadata::Text;
use super::text_offset_range::TextOffsetRange;
use super::used_font::UsedFont;

// cpp: layoutng/internal/ink_overflow.h:190-198
pub struct ReadUnsetAsNoneScope;

#[cfg(debug_assertions)]
// cpp: layoutng/internal/ink_overflow.h:287
// cpp: layoutng/internal/ink_overflow.cc:39
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

// cpp: layoutng/internal/ink_overflow.h:34-41
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

// cpp: layoutng/internal/ink_overflow.h:45-58
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

// cpp: layoutng/internal/ink_overflow.h:69-81
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

// cpp: layoutng/internal/ink_overflow.h:254-282
#[repr(C)]
#[derive(Clone, Copy)]
union InkOverflowStorage {
    single_: *mut SingleInkOverflow,
    container_: *mut ContainerInkOverflow,
    outsets_: [SmallRawValue; 4],
}

// cpp: layoutng/internal/ink_overflow.h:67-90,254-287
#[repr(C)]
pub struct InkOverflow {
    storage_: InkOverflowStorage,
    #[cfg(debug_assertions)]
    type_: InkOverflowType,
}

// cpp: layoutng/internal/ink_overflow.cc:14-20
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
    // cpp: layoutng/internal/ink_overflow.h:84
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
    // cpp: layoutng/internal/ink_overflow.cc:41-46
    fn drop(&mut self) {
        debug_assert!(matches!(
            self.type_,
            InkOverflowType::kNotSet | InkOverflowType::kNone | InkOverflowType::kInvalidated
        ));
    }
}

// cpp: layoutng/internal/ink_overflow.cc:23-29
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
    // cpp: layoutng/internal/ink_overflow.h:82
    pub const kTypeBits: i32 = 3;

    // cpp: layoutng/internal/ink_overflow.cc:50-74
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

    // cpp: layoutng/internal/ink_overflow.cc:76-102
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

    // cpp: layoutng/internal/ink_overflow.h:289-304
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

    // cpp: layoutng/internal/ink_overflow.h:107-109
    pub fn Reset(&mut self, type_: InkOverflowType) -> InkOverflowType {
        self.ResetTo(type_, InkOverflowType::kNone)
    }

    pub fn Invalidate(&mut self, type_: InkOverflowType) -> InkOverflowType {
        self.ResetTo(type_, InkOverflowType::kInvalidated)
    }

    // cpp: layoutng/internal/ink_overflow.cc:104-124
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

    // cpp: layoutng/internal/ink_overflow.cc:126-132
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

    // cpp: layoutng/internal/ink_overflow.cc:134-156
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

    // cpp: layoutng/internal/ink_overflow.cc:158-181
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

    // cpp: layoutng/internal/ink_overflow.cc:183-207
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

    // cpp: layoutng/internal/ink_overflow.cc:211-233
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

    // cpp: layoutng/internal/ink_overflow.cc:235-275
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

    // cpp: layoutng/internal/ink_overflow.cc:277-284
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

    // cpp: layoutng/internal/ink_overflow.cc:286-294
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

    // cpp: layoutng/internal/ink_overflow.cc:296-330
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
}

// The supplied source tree declares these functions without defining them.
// The associated type preserves the opaque InlinePaintContext boundary until
// the text-paint owner supplies its actual type and implementations.
// cpp: layoutng/internal/ink_overflow.h:127-187
pub trait InkOverflowTextMethods {
    type InlinePaintContext;

    // cpp: layoutng/internal/ink_overflow.h:128-134
    fn SetTextInkOverflow(
        &mut self,
        type_: InkOverflowType,
        cursor: &InlineCursor,
        text_info: &TextFragmentPaintInfo,
        style: &ComputedStyle,
        rect_in_container: &PhysicalRect,
        inline_context: Option<&Self::InlinePaintContext>,
        ink_overflow_out: *mut PhysicalRect,
    ) -> InkOverflowType;

    // cpp: layoutng/internal/ink_overflow.h:139-147
    fn SetSvgTextInkOverflow(
        &mut self,
        type_: InkOverflowType,
        cursor: &InlineCursor,
        text_info: &TextFragmentPaintInfo,
        style: &ComputedStyle,
        rect: &gfx::RectF,
        scaling_factor: f32,
        length_adjust_scale: f32,
        transform: &AffineTransform,
        ink_overflow_out: *mut PhysicalRect,
    ) -> InkOverflowType;

    // cpp: layoutng/internal/ink_overflow.h:149-154
    fn ComputeTextInkOverflow(
        cursor: &InlineCursor,
        text_info: &TextFragmentPaintInfo,
        style: &ComputedStyle,
        rect_in_container: &PhysicalRect,
        inline_context: Option<&Self::InlinePaintContext>,
    ) -> Option<PhysicalRect>;

    // cpp: layoutng/internal/ink_overflow.h:160-163
    fn ComputeEmphasisMarkOverflow(
        style: &ComputedStyle,
        size: &PhysicalSize,
        ink_overflow: &LogicalRect,
    ) -> LogicalRect;

    // cpp: layoutng/internal/ink_overflow.h:166-168
    fn ExpandForShadowOverflow(
        ink_overflow: &mut LogicalRect,
        text_shadow: &ShadowList,
        writing_mode: WritingMode,
    );

    // cpp: layoutng/internal/ink_overflow.h:175-182
    fn ComputeDecorationOverflow(
        cursor: &InlineCursor,
        style: &ComputedStyle,
        used_font: &UsedFont,
        container_offset: &PhysicalOffset,
        ink_overflow: &LogicalRect,
        inline_context: Option<&Self::InlinePaintContext>,
        writing_mode: WritingMode,
    ) -> LogicalRect;

    // cpp: layoutng/internal/ink_overflow.h:185-187
    fn ComputeCaretOverflow(
        cursor: &InlineCursor,
        style: &ComputedStyle,
        ink_overflow_in: &LogicalRect,
    ) -> LogicalRect;
}

// Private C++ helpers are kept crate-local. Their implementations are absent
// from the supplied source just like the public text-overflow methods.
// cpp: layoutng/internal/ink_overflow.h:201-241
pub(crate) trait InkOverflowTextInternals: InkOverflowTextMethods {
    type LayoutObject;

    // cpp: layoutng/internal/ink_overflow.h:206-213
    fn ComputeAppliedDecorationOverflow(
        style: &ComputedStyle,
        used_font: &UsedFont,
        offset_in_container: &PhysicalOffset,
        ink_overflow: &LogicalRect,
        inline_context: Option<&Self::InlinePaintContext>,
        fragment_cursor: Option<&InlineCursor>,
        decoration_override: Option<&AppliedTextDecoration>,
    ) -> LogicalRect;

    // cpp: layoutng/internal/ink_overflow.h:217-229
    fn ComputeMarkerOverflow(
        markers: &DocumentMarkerVector,
        marker_type: MarkerType,
        fragment_item: Option<&FragmentItem>,
        fragment_dom_offsets: &TextOffsetRange,
        node: *mut Text,
        layout_object: &Self::LayoutObject,
        style: &ComputedStyle,
        used_font: &UsedFont,
        offset_in_container: &PhysicalOffset,
        ink_overflow: &LogicalRect,
        inline_context: Option<&Self::InlinePaintContext>,
        writing_mode: WritingMode,
    ) -> LogicalRect;

    // cpp: layoutng/internal/ink_overflow.h:231-241
    fn ComputeCustomHighlightOverflow(
        markers: &DocumentMarkerVector,
        fragment_item: Option<&FragmentItem>,
        fragment_dom_offsets: &TextOffsetRange,
        text_node: *mut Text,
        layout_object: &Self::LayoutObject,
        style: &ComputedStyle,
        used_font: &UsedFont,
        offset_in_container: &PhysicalOffset,
        ink_overflow: &LogicalRect,
        inline_context: Option<&Self::InlinePaintContext>,
    ) -> LogicalRect;
}
