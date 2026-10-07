use std::fmt;
use std::ops::{Index, IndexMut};

use font_engine::{Font, FontHeight, ShapeResultView};
use foundation::{
    DirectionFromLevel, DynamicTo, HeapVector, LayoutUnit, Member, StrCat, String,
    TextDirection, Visitor, WritingDirectionMode, WritingMode,
};
use icu_bidi::UBiDiLevel;
use layoutng::internal::inline_item::{InlineItem, InlineItemType};
use layoutng::internal::layout_node_metadata::Node;
use layoutng::internal::inline_item_result::{InlineItemResult, OptionalPositionedFloat};
use layoutng::internal::inline_item_text_index::InlineItemTextIndex;
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_text::LayoutText;
use layoutng::internal::style_variant::StyleVariant;
use layoutng::internal::text_fit_scale::TextFitScale;
use layoutng::internal::text_offset_range::TextOffsetRange;
use layoutng::internal::used_font::UsedFont;
use layoutng_geometry::geometry::bfc_offset::BfcOffset;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::logical_size::LogicalSize;
use layoutng_style::style::computed_style::ComputedStyle;

use crate::layout_result::LayoutResult;
use crate::physical_fragment::PhysicalFragment;

// All fields are public as in the C++ struct. Member<T> keeps each GC edge,
// and the source's default bidi sentinel is set explicitly below.
// cpp: layoutng_fragment_tree/logical_line_item.h:25-28
// cpp: layoutng_fragment_tree/logical_line_item.h:230-284
pub struct LogicalLineItem {
    pub inline_item: Member<InlineItem>,
    pub shape_result: Member<ShapeResultView>,
    pub text_offset: TextOffsetRange,
    pub text_content: String,
    pub layout_result: Member<LayoutResult>,
    pub layout_object: Member<LayoutObject>,
    pub style_variant: StyleVariant,
    pub out_of_flow_positioned_box: Member<LayoutObject>,
    pub unpositioned_float: Member<LayoutObject>,
    pub item_index: InlineItemTextIndex,
    pub rect: LogicalRect,
    pub bfc_offset: BfcOffset,
    pub inline_size: LayoutUnit,
    pub margin_line_left: LayoutUnit,
    pub box_data_index: u32,
    pub children_count: u32,
    pub bidi_level: UBiDiLevel,
    pub container_writing_direction: WritingDirectionMode,
    pub text_fit_scale: Member<TextFitScale>,
    pub has_only_bidi_trailing_spaces: bool,
    pub is_hidden_for_paint: bool,
    pub has_over_annotation: bool,
    pub has_under_annotation: bool,
    pub annotation_metrics: FontHeight,
}

impl Default for LogicalLineItem {
    // cpp: layoutng_fragment_tree/logical_line_item.h:30-31
    fn default() -> Self {
        Self {
            inline_item: Member::default(),
            shape_result: Member::default(),
            text_offset: TextOffsetRange::default(),
            text_content: String::default(),
            layout_result: Member::default(),
            layout_object: Member::default(),
            style_variant: StyleVariant::kStandard,
            out_of_flow_positioned_box: Member::default(),
            unpositioned_float: Member::default(),
            item_index: InlineItemTextIndex::default(),
            rect: LogicalRect::default(),
            bfc_offset: BfcOffset::default(),
            inline_size: LayoutUnit::default(),
            margin_line_left: LayoutUnit::default(),
            box_data_index: 0,
            children_count: 0,
            bidi_level: 0xff,
            container_writing_direction: WritingDirectionMode::new(
                WritingMode::kHorizontalTb,
                TextDirection::kLtr,
            ),
            text_fit_scale: Member::default(),
            has_only_bidi_trailing_spaces: false,
            is_hidden_for_paint: false,
            has_over_annotation: false,
            has_under_annotation: false,
            annotation_metrics: FontHeight::default(),
        }
    }
}

#[allow(non_snake_case)]
impl LogicalLineItem {
    // cpp: layoutng_fragment_tree/logical_line_item.h:32-35
    pub fn placeholder(block_offset: LayoutUnit, block_size: LayoutUnit) -> Self {
        Self {
            rect: LogicalRect::from_units(
                LayoutUnit::default(),
                block_offset,
                LayoutUnit::default(),
                block_size,
            ),
            ..Self::default()
        }
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:36-38
    pub fn bidi_control(bidi_level: UBiDiLevel) -> Self {
        Self {
            bidi_level,
            ..Self::default()
        }
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:39-47
    pub fn from_layout_result_rect(
        layout_result: *const LayoutResult,
        rect: &LogicalRect,
        children_count: u32,
        bidi_level: UBiDiLevel,
    ) -> Self {
        Self {
            layout_result: Member::from_ptr(layout_result as *mut LayoutResult),
            rect: *rect,
            children_count,
            bidi_level,
            ..Self::default()
        }
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:48-57
    pub fn from_layout_result_offset(
        layout_result: *const LayoutResult,
        offset: LogicalOffset,
        inline_size: LayoutUnit,
        children_count: u32,
        bidi_level: UBiDiLevel,
    ) -> Self {
        Self {
            layout_result: Member::from_ptr(layout_result as *mut LayoutResult),
            rect: LogicalRect::new(offset, LogicalSize::default()),
            inline_size,
            children_count,
            bidi_level,
            ..Self::default()
        }
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:58-74
    pub fn from_item_result(
        inline_item: &InlineItem,
        item_result: &mut InlineItemResult,
        text_offset: &TextOffsetRange,
        block_offset: LayoutUnit,
        inline_size: LayoutUnit,
        text_height: LayoutUnit,
        bidi_level: UBiDiLevel,
    ) -> Self {
        Self {
            inline_item: Member::from_ptr(inline_item as *const _ as *mut _),
            shape_result: item_result.shape_result.clone(),
            text_offset: *text_offset,
            rect: LogicalRect::from_units(
                LayoutUnit::default(),
                block_offset,
                LayoutUnit::default(),
                text_height,
            ),
            inline_size,
            bidi_level,
            text_fit_scale: item_result.text_fit_scale.clone(),
            has_only_bidi_trailing_spaces: item_result.has_only_bidi_trailing_spaces,
            ..Self::default()
        }
    }

    // C++ defaults text_fit_scale to null in this overload. Rust callers pass
    // None for that case rather than relying on an implicit default argument.
    // cpp: layoutng_fragment_tree/logical_line_item.h:75-89
    pub fn from_text_fragment(
        inline_item: &InlineItem,
        shape_result: *const ShapeResultView,
        text_offset: &TextOffsetRange,
        block_offset: LayoutUnit,
        inline_size: LayoutUnit,
        text_height: LayoutUnit,
        bidi_level: UBiDiLevel,
        text_fit_scale: Option<&TextFitScale>,
    ) -> Self {
        Self {
            inline_item: Member::from_ptr(inline_item as *const _ as *mut _),
            shape_result: Member::from_ptr(shape_result as *mut ShapeResultView),
            text_offset: *text_offset,
            rect: LogicalRect::from_units(
                LayoutUnit::default(),
                block_offset,
                LayoutUnit::default(),
                text_height,
            ),
            inline_size,
            bidi_level,
            text_fit_scale: Member::from_ptr(
                text_fit_scale.map_or(std::ptr::null_mut(), |value| {
                    value as *const TextFitScale as *mut TextFitScale
                }),
            ),
            ..Self::default()
        }
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:90-106
    pub fn generated_text(
        inline_item: &InlineItem,
        shape_result: &ShapeResultView,
        text_content: &String,
        text_fit_scale: *const TextFitScale,
        block_offset: LayoutUnit,
        inline_size: LayoutUnit,
        text_height: LayoutUnit,
        bidi_level: UBiDiLevel,
    ) -> Self {
        Self {
            inline_item: Member::from_ptr(inline_item as *const _ as *mut _),
            shape_result: Member::from_ptr(shape_result as *const _ as *mut _),
            text_offset: TextOffsetRange::new(shape_result.StartIndex(), shape_result.EndIndex()),
            text_content: text_content.clone(),
            rect: LogicalRect::from_units(
                LayoutUnit::default(),
                block_offset,
                LayoutUnit::default(),
                text_height,
            ),
            inline_size,
            bidi_level,
            text_fit_scale: Member::from_ptr(text_fit_scale as *mut TextFitScale),
            ..Self::default()
        }
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:107-122
    pub fn ellipsis(
        layout_object: &LayoutObject,
        style_variant: StyleVariant,
        shape_result: &ShapeResultView,
        text_content: &String,
        rect: &LogicalRect,
        bidi_level: UBiDiLevel,
    ) -> Self {
        Self {
            shape_result: Member::from_ptr(shape_result as *const _ as *mut _),
            text_offset: TextOffsetRange::new(shape_result.StartIndex(), shape_result.EndIndex()),
            text_content: text_content.clone(),
            layout_object: Member::from_ptr(layout_object as *const _ as *mut _),
            style_variant,
            rect: *rect,
            inline_size: rect.size.inline_size,
            bidi_level,
            ..Self::default()
        }
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:123-132
    pub fn reshaped_text(
        source_item: &Self,
        shape_result: &ShapeResultView,
        text_offset: &TextOffsetRange,
    ) -> Self {
        Self {
            inline_item: source_item.inline_item.clone(),
            shape_result: Member::from_ptr(shape_result as *const _ as *mut _),
            text_offset: *text_offset,
            text_content: source_item.text_content.clone(),
            rect: source_item.rect,
            inline_size: shape_result.SnappedWidth(),
            bidi_level: source_item.bidi_level,
            ..Self::default()
        }
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:133-135
    // cpp: layoutng_fragment_tree/logical_line_item.h:372-380
    pub fn OutOfFlowPositioned(
        inline_item: &InlineItem,
        container_writing_direction: WritingDirectionMode,
    ) -> Self {
        let mut line_item = Self::default();
        line_item.out_of_flow_positioned_box = Member::from_ptr(inline_item.GetLayoutObject());
        line_item.bidi_level = inline_item.BidiLevel();
        line_item.container_writing_direction = container_writing_direction;
        line_item
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:136-138
    // cpp: layoutng_fragment_tree/logical_line_item.h:382-391
    pub fn PositionedFloat(
        inline_item: &InlineItem,
        positioned_float: &OptionalPositionedFloat,
    ) -> Self {
        let positioned = positioned_float.Get();
        debug_assert!(!unsafe { &*positioned }.layout_result.Get().is_null());
        let mut line_item = Self::default();
        line_item.layout_result = unsafe { &*positioned }.layout_result.clone();
        line_item.bfc_offset = unsafe { &*positioned }.bfc_offset;
        line_item.bidi_level = inline_item.BidiLevel();
        line_item
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:139-140
    // cpp: layoutng_fragment_tree/logical_line_item.h:393-401
    pub fn UnpositionedFloat(inline_item: &InlineItem, item_index: InlineItemTextIndex) -> Self {
        let mut line_item = Self::default();
        line_item.unpositioned_float = Member::from_ptr(inline_item.GetLayoutObject());
        line_item.bidi_level = inline_item.BidiLevel();
        line_item.item_index = item_index;
        line_item
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:142-144
    pub fn IsItemType(&self, item_type: InlineItemType) -> bool {
        let item = self.inline_item.Get();
        !item.is_null() && unsafe { &*item }.Type() == item_type
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:145-147
    pub fn IsNotItemType(&self, item_type: InlineItemType) -> bool {
        let item = self.inline_item.Get();
        !item.is_null() && unsafe { &*item }.Type() != item_type
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:148-150
    pub fn IsFloating(&self) -> bool {
        let result = self.layout_result.Get();
        !result.is_null() && unsafe { &*result }.GetPhysicalFragment().IsFloating()
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:151-154
    pub fn IsAtomicInline(&self) -> bool {
        let result = self.layout_result.Get();
        !result.is_null() && unsafe { &*result }.GetPhysicalFragment().IsAtomicInline()
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:155-158
    pub fn IsInitialLetterBox(&self) -> bool {
        let result = self.layout_result.Get();
        !result.is_null()
            && unsafe { &*result }
                .GetPhysicalFragment()
                .IsInitialLetterBox()
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:159-161
    pub fn IsInlineBox(&self) -> bool {
        let result = self.layout_result.Get();
        !result.is_null() && unsafe { &*result }.GetPhysicalFragment().IsInlineBox()
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:162-166
    pub fn HasInFlowFragment(&self) -> bool {
        self.IsNotItemType(InlineItemType::kRubyLinePlaceholder)
            || (!self.layout_result.Get().is_null() && !self.IsFloating())
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:167-170
    pub fn HasInFlowOrFloatingFragment(&self) -> bool {
        self.IsNotItemType(InlineItemType::kRubyLinePlaceholder)
            || !self.layout_result.Get().is_null()
            || !self.layout_object.Get().is_null()
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:171-173
    pub fn HasOutOfFlowFragment(&self) -> bool {
        !self.out_of_flow_positioned_box.Get().is_null()
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:174-176
    pub fn HasFragment(&self) -> bool {
        self.HasInFlowOrFloatingFragment() || self.HasOutOfFlowFragment()
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:177-180
    pub fn IsControl(&self) -> bool {
        self.IsItemType(InlineItemType::kControl)
    }

    pub fn CanCreateFragmentItem(&self) -> bool {
        self.HasInFlowOrFloatingFragment()
    }

    pub fn HasBidiLevel(&self) -> bool {
        self.bidi_level != 0xff
    }

    pub fn IsPlaceholder(&self) -> bool {
        !self.HasFragment() && !self.HasBidiLevel()
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:181-195
    pub fn IsOpaqueToBidiReordering(&self) -> bool {
        if self.IsPlaceholder() {
            return true;
        }
        let result = self.layout_result.Get();
        if !result.is_null() {
            let object = unsafe { &*result }.GetPhysicalFragment().GetLayoutObject();
            debug_assert!(!object.is_null());
            if unsafe { &*object }.IsLayoutInline() {
                return true;
            }
        }
        false
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:196-198
    pub fn IsRubyLinePlaceholder(&self) -> bool {
        self.IsItemType(InlineItemType::kRubyLinePlaceholder)
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:200-205
    pub fn Offset(&self) -> &LogicalOffset {
        &self.rect.offset
    }

    pub fn InlineOffset(&self) -> LayoutUnit {
        self.rect.offset.inline_offset
    }

    pub fn BlockOffset(&self) -> LayoutUnit {
        self.rect.offset.block_offset
    }

    pub fn BlockEndOffset(&self) -> LayoutUnit {
        self.rect.BlockEndOffset()
    }

    pub fn Size(&self) -> &LogicalSize {
        &self.rect.size
    }

    pub fn MarginSize(&self) -> LogicalSize {
        LogicalSize::new(self.inline_size, self.Size().block_size)
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:207-211
    pub fn GetPhysicalFragment(&self) -> *const PhysicalFragment {
        let result = self.layout_result.Get();
        if !result.is_null() {
            return unsafe { &*result }.GetPhysicalFragment() as *const PhysicalFragment;
        }
        std::ptr::null()
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:212
    // cpp: layoutng_fragment_tree/logical_line_item.cc:60-67
    pub fn GetLayoutObject(&self) -> *const LayoutObject {
        let item = self.inline_item.Get();
        if !item.is_null() {
            return unsafe { &*item }.GetLayoutObject();
        }
        let fragment = self.GetPhysicalFragment();
        if !fragment.is_null() {
            return unsafe { &*fragment }.GetLayoutObject();
        }
        std::ptr::null()
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:213
    // cpp: layoutng_fragment_tree/logical_line_item.cc:69-76
    pub fn GetMutableLayoutObject(&self) -> *mut LayoutObject {
        let item = self.inline_item.Get();
        if !item.is_null() {
            return unsafe { &*item }.GetLayoutObject();
        }
        let fragment = self.GetPhysicalFragment();
        if !fragment.is_null() {
            return unsafe { &*fragment }.GetMutableLayoutObject();
        }
        std::ptr::null_mut()
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:214
    // cpp: layoutng_fragment_tree/logical_line_item.cc:78-82
    pub fn GetNode(&self) -> *const Node {
        let object = self.GetLayoutObject();
        if !object.is_null() {
            return unsafe { &*object }.GetNode();
        }
        std::ptr::null()
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:215
    // cpp: layoutng_fragment_tree/logical_line_item.cc:84-91
    pub fn Style(&self) -> *const ComputedStyle {
        let fragment = self.GetPhysicalFragment();
        if !fragment.is_null() {
            return unsafe { &*fragment }.Style() as *const ComputedStyle;
        }
        let item = self.inline_item.Get();
        if !item.is_null() {
            return unsafe { &*item }.Style();
        }
        std::ptr::null()
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:216
    // cpp: layoutng_fragment_tree/logical_line_item.cc:93-104
    pub fn GetUsedFont(&self) -> UsedFont {
        let mut font: *const Font = unsafe { &*self.Style() }.GetFont();
        debug_assert!(!font.is_null());
        let mut scale = 1.0f32;
        let text_fit_scale = self.text_fit_scale.Get();
        if !text_fit_scale.is_null() {
            let scaled_font = unsafe { &*text_fit_scale }.font.Get();
            if !scaled_font.is_null() {
                font = scaled_font;
            }
            scale = unsafe { &*text_fit_scale }.scale;
        }
        UsedFont::new(unsafe { &*font }, scale)
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:218-219
    pub fn StartOffset(&self) -> u32 {
        self.text_offset.start
    }

    pub fn EndOffset(&self) -> u32 {
        self.text_offset.end
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:221-226
    pub fn ResolvedDirection(&self) -> TextDirection {
        debug_assert!(self.HasBidiLevel() || self.IsInlineBox());
        if self.HasBidiLevel() {
            DirectionFromLevel(self.bidi_level as u32)
        } else {
            TextDirection::kLtr
        }
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:228
    // cpp: layoutng_fragment_tree/logical_line_item.cc:184-192
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.inline_item);
        visitor.Trace(&self.shape_result);
        visitor.Trace(&self.layout_result);
        visitor.Trace(&self.layout_object);
        visitor.Trace(&self.out_of_flow_positioned_box);
        visitor.Trace(&self.unpositioned_float);
        visitor.Trace(&self.text_fit_scale);
    }
}

// The C++ definitions are members of InlineItem, which is owned by the later
// layoutng crate. That crate can forward its two methods to these helpers.
// cpp: layoutng_fragment_tree/logical_line_item.cc:15-47
#[allow(non_snake_case)]
pub fn InlineItemTypeToString(value: InlineItemType) -> &'static str {
    match value {
        InlineItemType::kText => "Text",
        InlineItemType::kControl => "Control",
        InlineItemType::kAtomicInline => "AtomicInline",
        InlineItemType::kBlockInInline => "BlockInInline",
        InlineItemType::kOpenTag => "OpenTag",
        InlineItemType::kCloseTag => "CloseTag",
        InlineItemType::kFloating => "Floating",
        InlineItemType::kOutOfFlowPositioned => "OutOfFlowPositioned",
        InlineItemType::kInitialLetterBox => "InitialLetterBox",
        InlineItemType::kListMarker => "ListMarker",
        InlineItemType::kBidiControl => "BidiControl",
        InlineItemType::kOpenRubyColumn => "OpenRubyColumn",
        InlineItemType::kCloseRubyColumn => "CloseRubyColumn",
        InlineItemType::kRubyLinePlaceholder => "RubyLinePlaceholder",
    }
}

// cpp: layoutng_fragment_tree/logical_line_item.cc:49-58
#[allow(non_snake_case)]
pub fn InlineItemToString(item: &InlineItem) -> String {
    let object = item.GetLayoutObject();
    let layout_text = DynamicTo::<LayoutText>(object);
    let object_info = if !layout_text.is_null() {
        unsafe { &*layout_text }
            .TransformedText()
            .EncodeForDebugging()
    } else if !object.is_null() {
        unsafe { &*object }.ToString()
    } else {
        String::default()
    };
    StrCat(&[
        String::from("InlineItem "),
        String::from(InlineItemTypeToString(item.Type())),
        String::from(". "),
        object_info,
    ])
}

// cpp: layoutng_fragment_tree/logical_line_item.h:286-287
// cpp: layoutng_fragment_tree/logical_line_item.cc:106-121
impl fmt::Display for LogicalLineItem {
    fn fmt(&self, stream: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(stream, "LogicalLineItem(")?;
        if self.IsPlaceholder() {
            write!(stream, " placeholder")?;
        }
        write!(stream, " inline_size={}", self.inline_size)?;
        let inline_item = self.inline_item.Get();
        if !inline_item.is_null() {
            write!(stream, " {}", InlineItemToString(unsafe { &*inline_item }))?;
        }
        let fragment = self.GetPhysicalFragment();
        if !fragment.is_null() {
            write!(stream, " Fragment={}", unsafe { &*fragment })?;
        }
        let object = self.GetLayoutObject();
        if !object.is_null() {
            write!(stream, " LayoutObject={}", unsafe { &*object })?;
        }
        write!(stream, ")")
    }
}

// cpp: layoutng_fragment_tree/logical_line_item.h:289-293
// cpp: layoutng_fragment_tree/logical_line_item.h:368-370
pub struct LogicalLineItems {
    children_: HeapVector<LogicalLineItem, 16>,
    was_propagated_: bool,
}

impl Default for LogicalLineItems {
    // cpp: layoutng_fragment_tree/logical_line_item.h:294
    fn default() -> Self {
        Self {
            children_: HeapVector::default(),
            was_propagated_: false,
        }
    }
}

impl Drop for LogicalLineItems {
    // cpp: layoutng_fragment_tree/logical_line_item.h:295
    fn drop(&mut self) {
        debug_assert!(self.IsEmpty());
    }
}

impl Index<usize> for LogicalLineItems {
    type Output = LogicalLineItem;

    // cpp: layoutng_fragment_tree/logical_line_item.h:300-301
    fn index(&self, index: usize) -> &Self::Output {
        &self.children_[index]
    }
}

impl IndexMut<usize> for LogicalLineItems {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.children_[index]
    }
}

#[allow(non_snake_case)]
impl LogicalLineItems {
    // cpp: layoutng_fragment_tree/logical_line_item.h:296-298
    pub fn AssignMoved(&mut self, mut other: Self) {
        self.children_ = std::mem::take(&mut other.children_);
        // C++ assigns only children_; was_propagated_ stays unchanged.
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:303-310
    pub fn size(&self) -> u32 {
        self.children_.len() as u32
    }

    pub fn clear(&mut self) {
        self.children_.clear();
    }

    pub fn IsEmpty(&self) -> bool {
        self.children_.is_empty()
    }

    pub fn ReserveInitialCapacity(&mut self, capacity: u32) {
        self.children_.ReserveInitialCapacity(capacity);
    }

    pub fn Shrink(&mut self, size: u32) {
        self.children_.Shrink(size);
    }

    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.children_, &mut other.children_);
    }

    // C++ converts to a mutable span and exposes forward/reverse iterators.
    // Slices and standard iterator adapters retain the same traversal order.
    // cpp: layoutng_fragment_tree/logical_line_item.h:312-327
    pub fn AsSpan(&mut self) -> &mut [LogicalLineItem] {
        &mut self.children_
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &LogicalLineItem> {
        self.children_.iter()
    }

    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut LogicalLineItem> {
        self.children_.iter_mut()
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:329
    // cpp: layoutng_fragment_tree/logical_line_item.cc:123-129
    pub fn FirstInFlowChild(&mut self) -> *mut LogicalLineItem {
        for child in &mut self.children_ {
            if child.HasInFlowFragment() {
                return child;
            }
        }
        std::ptr::null_mut()
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:330
    // cpp: layoutng_fragment_tree/logical_line_item.cc:131-137
    pub fn LastInFlowChild(&mut self) -> *mut LogicalLineItem {
        for child in self.children_.iter_mut().rev() {
            if child.HasInFlowFragment() {
                return child;
            }
        }
        std::ptr::null_mut()
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:332
    // cpp: layoutng_fragment_tree/logical_line_item.cc:139-147
    pub fn BlockInInlineLayoutResult(&self) -> *const LayoutResult {
        for item in &self.children_ {
            let result = item.layout_result.Get();
            if !result.is_null() && unsafe { &*result }.GetPhysicalFragment().IsBlockInInline() {
                return result;
            }
        }
        std::ptr::null()
    }

    // The C++ variadic emplace constructor becomes an explicit item factory
    // followed by a value append; source constructor overloads remain above.
    // cpp: layoutng_fragment_tree/logical_line_item.h:334-338
    pub fn AddChild(&mut self, item: LogicalLineItem) {
        self.children_.push(item);
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:339-342
    pub fn InsertChild(&mut self, index: u32, item: LogicalLineItem) {
        self.WillInsertChild(index);
        self.children_.insert(index as usize, item);
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:343-351
    pub fn InsertChildLayoutResult(
        &mut self,
        index: u32,
        layout_result: *const LayoutResult,
        rect: &LogicalRect,
        children_count: u32,
    ) {
        self.WillInsertChild(index);
        self.children_.insert(
            index as usize,
            LogicalLineItem::from_layout_result_rect(layout_result, rect, children_count, 0),
        );
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:353
    // cpp: layoutng_fragment_tree/logical_line_item.cc:160-163
    pub fn MoveInInlineDirection(&mut self, delta: LayoutUnit) {
        for child in &mut self.children_ {
            child.rect.offset.inline_offset += delta;
        }
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:354
    // cpp: layoutng_fragment_tree/logical_line_item.cc:165-170
    pub fn MoveInInlineDirectionRange(&mut self, delta: LayoutUnit, start: u32, end: u32) {
        for index in start..end {
            self.children_[index as usize].rect.offset.inline_offset += delta;
        }
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:355
    // cpp: layoutng_fragment_tree/logical_line_item.cc:172-175
    pub fn MoveInBlockDirection(&mut self, delta: LayoutUnit) {
        for child in &mut self.children_ {
            child.rect.offset.block_offset += delta;
        }
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:356
    // cpp: layoutng_fragment_tree/logical_line_item.cc:177-182
    pub fn MoveInBlockDirectionRange(&mut self, delta: LayoutUnit, start: u32, end: u32) {
        for index in start..end {
            self.children_[index as usize].rect.offset.block_offset += delta;
        }
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:358-361
    pub fn SetPropagated(&mut self) {
        self.was_propagated_ = true;
    }

    pub fn WasPropagated(&self) -> bool {
        self.was_propagated_
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:363
    // cpp: layoutng_fragment_tree/logical_line_item.cc:194-196
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.children_);
    }

    // cpp: layoutng_fragment_tree/logical_line_item.h:365-366
    // cpp: layoutng_fragment_tree/logical_line_item.cc:149-158
    fn WillInsertChild(&mut self, insert_before: u32) {
        let mut index = 0;
        for child in &mut self.children_ {
            if index >= insert_before {
                break;
            }
            if child.children_count != 0 && index + child.children_count > insert_before {
                child.children_count += 1;
            }
            index += 1;
        }
    }
}
