// C++: layoutng_inline/inline_item.cc (Chromium core/layout/inline/inline_item.cc).
// The item-building members of base's InlineItem record (constructor with
// ComputeBoxProperties, Split, SetSegmentData, SetBidiLevelForItems). Rust
// cannot add inherent methods to a base type, so `InlineItemAlgorithms`
// carries them; the inline sources import it through `crate::algorithms`.
// The record members (copy, trace, index, checks) are base code (R5).
#![allow(non_snake_case)]

#[allow(unused_imports)]
use crate::algorithms::*;
use font_engine::RunSegmenterRange;
use foundation::{MakeGarbageCollected, Member};
use layout_base::internal::inline_item::{InlineItem, InlineItemType, InlineItems};
use layout_base::internal::inline_item_segment::InlineItemSegment;
use layout_base::internal::layout_invalidation_reason;
use layout_base::internal::layout_object::LayoutObject;
use layout_style::style::computed_style::ComputedStyle;

// cpp: layoutng_inline/inline_item.cc:17-24
// ASSERT_SIZE checks the C++ packed-bitfield ABI. InlineItem is not passed to
// C++; its Rust scalars preserve each field, while the GC uses its Rust size.

// cpp: layoutng_inline/inline_item.cc:26-46
fn IsInlineBoxStartEmpty(style: &ComputedStyle, object: &LayoutObject) -> bool {
    if style.BorderInlineStartWidth() != 0 || !style.PaddingInlineStart().IsZero() {
        return false;
    }
    if !style.MarginInlineStart().IsZero() && !object.InLineHeightQuirksModeForLayout() {
        return false;
    }
    true
}

// cpp: layoutng_inline/inline_item.cc:48-67
fn IsInlineBoxEndEmpty(style: &ComputedStyle, object: &LayoutObject) -> bool {
    if style.BorderInlineEndWidth() != 0 || !style.PaddingInlineEnd().IsZero() {
        return false;
    }
    if !style.MarginInlineEnd().IsZero() && !object.InLineHeightQuirksModeForLayout() {
        return false;
    }
    true
}

// cpp: layoutng_inline/inline_item.cc:71-83
fn InlineItemNew(
    type_: InlineItemType,
    start: u32,
    end: u32,
    layout_object: *mut LayoutObject,
) -> InlineItem {
    let mut item = InlineItem::new_base_for_inline(type_, start, end, layout_object);
    debug_assert!(end >= start);
    item.ComputeBoxProperties();
    item
}

// cpp: layoutng_inline/inline_item.cc:118-151
fn InlineItemComputeBoxProperties(item: &mut InlineItem) {
    debug_assert!(!item.is_empty_item_);
    if matches!(
        item.type_,
        InlineItemType::kText | InlineItemType::kAtomicInline | InlineItemType::kControl
    ) {
        return;
    }
    if item.type_ == InlineItemType::kInitialLetterBox {
        return;
    }
    if item.type_ == InlineItemType::kOpenTag {
        let object = item.layout_object_.Get();
        debug_assert!(!object.is_null() && unsafe { &*object }.IsLayoutInline());
        item.is_empty_item_ = IsInlineBoxStartEmpty(unsafe { &*item.Style() }, unsafe { &*object });
        return;
    }
    if item.type_ == InlineItemType::kCloseTag {
        let object = item.layout_object_.Get();
        debug_assert!(!object.is_null() && unsafe { &*object }.IsLayoutInline());
        item.is_empty_item_ = IsInlineBoxEndEmpty(unsafe { &*item.Style() }, unsafe { &*object });
        return;
    }
    if item.type_ == InlineItemType::kBlockInInline {
        return;
    }
    if matches!(
        item.type_,
        InlineItemType::kOutOfFlowPositioned | InlineItemType::kFloating
    ) {
        item.is_block_level_ = true;
    }
    item.is_empty_item_ = true;
}

// cpp: layoutng_inline/inline_item.cc:153-162
fn InlineItemSetSegmentData(range: &RunSegmenterRange, items: &mut InlineItems) {
    let segment_data = InlineItemSegment::PackSegmentData(range);
    for item_ptr in items.iter_mut() {
        let item = unsafe { &mut *item_ptr.Get() };
        if item.Type() == InlineItemType::kText {
            item.segment_data_ = segment_data;
        }
    }
}

// cpp: layoutng_inline/inline_item.cc:164-220
fn InlineItemSetBidiLevelForItems(
    items: &mut InlineItems,
    mut index: u32,
    end_offset: u32,
    level: u8,
    mut num_out_of_flow: u32,
) -> u32 {
    let item = loop {
        let item = unsafe { &mut *items[index as usize].Get() };
        item.SetBidiLevel(level);
        if num_out_of_flow != 0 && item.IsFloatingOrOutOfFlowPositioned() {
            num_out_of_flow -= 1;
        }
        if item.end_offset_ >= end_offset {
            break item as *mut InlineItem;
        }
        index += 1;
    };
    if unsafe { &*item }.end_offset_ > end_offset {
        let object = unsafe { &*item }.GetLayoutObject();
        if unsafe { &*object }.EverHadLayout() && !unsafe { &*object }.NeedsLayout() {
            unsafe { &mut *object }.SetNeedsLayout(unsafe {
                std::ptr::addr_of!(layout_invalidation_reason::kStyleChange)
            });
        }
        InlineItem::Split(items, index, end_offset);
        return index + 1;
    }
    debug_assert_eq!(end_offset, unsafe { &*item }.end_offset_);
    index += 1;
    while (index as usize) < items.len() {
        let item = unsafe { &mut *items[index as usize].Get() };
        let is_trailing = item.Length() == 0
            && (item.Type() == InlineItemType::kCloseTag || num_out_of_flow != 0);
        if !is_trailing {
            break;
        }
        item.SetBidiLevel(level);
        if num_out_of_flow != 0 && item.IsFloatingOrOutOfFlowPositioned() {
            num_out_of_flow -= 1;
        }
        index += 1;
    }
    debug_assert_eq!(num_out_of_flow, 0);
    index
}

// cpp: layoutng_inline/inline_item.cc:259-274
fn InlineItemSplit(items: &mut InlineItems, index: u32, offset: u32) {
    let source = items[index as usize].Get();
    debug_assert!(offset > unsafe { &*source }.start_offset_);
    debug_assert!(offset < unsafe { &*source }.end_offset_);
    unsafe { &mut *source }.shape_result_ = Member::default();
    let copy = MakeGarbageCollected(unsafe { &*source }.clone());
    unsafe { &mut *source }.end_offset_ = offset;
    unsafe { &mut *copy }.start_offset_ = offset;
    items.insert(index as usize + 1, Member::from_ptr(copy));
}


// cpp: layoutng/internal/inline_item.h:66-78,140-156,264-292
pub trait InlineItemAlgorithms {
    fn new(
        type_: InlineItemType,
        start: u32,
        end: u32,
        layout_object: *mut LayoutObject,
    ) -> InlineItem;
    fn ComputeBoxProperties(&mut self);
    fn Split(items: &mut InlineItems, index: u32, offset: u32);
    fn SetSegmentData(range: &RunSegmenterRange, items: &mut InlineItems);
    fn SetBidiLevelForItems(
        items: &mut InlineItems,
        index: u32,
        end_offset: u32,
        level: u8,
        num_out_of_flow: u32,
    ) -> u32;
    fn SetBidiLevelForItemsDefault(
        items: &mut InlineItems,
        index: u32,
        end_offset: u32,
        level: u8,
    ) -> u32;
}

impl InlineItemAlgorithms for InlineItem {
    fn new(
        type_: InlineItemType,
        start: u32,
        end: u32,
        layout_object: *mut LayoutObject,
    ) -> InlineItem {
        InlineItemNew(type_, start, end, layout_object)
    }
    fn ComputeBoxProperties(&mut self) {
        InlineItemComputeBoxProperties(self)
    }
    fn Split(items: &mut InlineItems, index: u32, offset: u32) {
        InlineItemSplit(items, index, offset)
    }
    fn SetSegmentData(range: &RunSegmenterRange, items: &mut InlineItems) {
        InlineItemSetSegmentData(range, items)
    }
    fn SetBidiLevelForItems(
        items: &mut InlineItems,
        index: u32,
        end_offset: u32,
        level: u8,
        num_out_of_flow: u32,
    ) -> u32 {
        InlineItemSetBidiLevelForItems(items, index, end_offset, level, num_out_of_flow)
    }
    fn SetBidiLevelForItemsDefault(
        items: &mut InlineItems,
        index: u32,
        end_offset: u32,
        level: u8,
    ) -> u32 {
        InlineItemSetBidiLevelForItems(items, index, end_offset, level, 0)
    }
}
