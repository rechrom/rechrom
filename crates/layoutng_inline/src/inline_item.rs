// C++: layoutng_inline/inline_item.cc. Non-inline methods of the InlineItem
// type owned by //src/layoutng; linked in the shared assembly.
#![allow(non_snake_case)]

use font_engine::{Font, RunSegmenterRange, ShapeResult};
use foundation::{MakeGarbageCollected, Member, String, To, Visitor};
use layoutng::internal::inline_item::{InlineItem, InlineItemType, InlineItems};
use layoutng::internal::inline_item_segment::InlineItemSegment;
use layoutng::internal::layout_invalidation_reason;
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_text::LayoutText;
use layoutng::internal::text_item_type::TextItemType;
use layoutng_style::style::computed_style::ComputedStyle;

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
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemNew(
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

// cpp: layoutng_inline/inline_item.cc:85-108
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemCloneAdjusted(
    item: &InlineItem,
    start: u32,
    end: u32,
    shape_result: *const ShapeResult,
) -> InlineItem {
    let clone = InlineItem::clone_adjusted_base_for_inline(item, start, end, shape_result);
    debug_assert!(end >= start);
    clone
}

// cpp: layoutng_inline/inline_item.cc:110-114
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemClone(item: &InlineItem) -> InlineItem {
    InlineItemCloneAdjusted(
        item,
        item.start_offset_,
        item.end_offset_,
        item.shape_result_.Get(),
    )
}

// cpp: layoutng_inline/inline_item.cc:116-116
// Rust's field-owning drop is the source's default destructor.

// cpp: layoutng_inline/inline_item.cc:118-151
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemComputeBoxProperties(item: &mut InlineItem) {
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
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSetSegmentData(range: &RunSegmenterRange, items: &mut InlineItems) {
    let segment_data = InlineItemSegment::PackSegmentData(range);
    for item_ptr in items.iter_mut() {
        let item = unsafe { &mut *item_ptr.Get() };
        if item.Type() == InlineItemType::kText {
            item.segment_data_ = segment_data;
        }
    }
}

// cpp: layoutng_inline/inline_item.cc:164-220
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSetBidiLevelForItems(
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

// cpp: layoutng_inline/inline_item.cc:222-229
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemFontWithSvgScaling(item: &InlineItem) -> *const Font {
    let object = item.layout_object_.Get();
    if !object.is_null() && unsafe { &*object }.IsSVGInlineText() {
        let text = To::<LayoutText>(object);
        return unsafe { &*text }.ScaledFont();
    }
    unsafe { &*item.Style() }.GetFont()
}

// cpp: layoutng_inline/inline_item.cc:231-240
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemIndexInItems(
    item: &InlineItem,
    items: &[Member<InlineItem>],
) -> u32 {
    let mut index = 0;
    for candidate in items {
        if std::ptr::eq(item, unsafe { &*candidate.Get() }) {
            return index;
        }
        index += 1;
    }
    index
}

// cpp: layoutng_inline/inline_item.cc:242-247
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemUpdateIndex(items: &mut [Member<InlineItem>]) {
    let mut index = 0;
    for item in items {
        unsafe { &mut *item.Get() }.index_ = index;
        index += 1;
    }
}

// cpp: layoutng_inline/inline_item.cc:249-257
#[cfg(feature = "expensive_dchecks")]
pub fn InlineItemCheckIndex(items: &mut [Member<InlineItem>]) {
    let mut index = 0;
    for item in items {
        debug_assert_eq!(unsafe { &*item.Get() }.index_, index);
        index += 1;
    }
}

// cpp: layoutng_inline/inline_item.cc:259-274
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSplit(items: &mut InlineItems, index: u32, offset: u32) {
    let source = items[index as usize].Get();
    debug_assert!(offset > unsafe { &*source }.start_offset_);
    debug_assert!(offset < unsafe { &*source }.end_offset_);
    unsafe { &mut *source }.shape_result_ = Member::default();
    let copy = MakeGarbageCollected(unsafe { &*source }.clone());
    unsafe { &mut *source }.end_offset_ = offset;
    unsafe { &mut *copy }.start_offset_ = offset;
    items.insert(index as usize + 1, Member::from_ptr(copy));
}

// cpp: layoutng_inline/inline_item.cc:276-307
#[cfg(debug_assertions)]
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemCheckTextType(item: &InlineItem, text_content: &String) {
    let character = if item.Length() != 0 {
        text_content.Span16().expect("text content is null")[item.StartOffset() as usize]
    } else {
        0
    };
    match character {
        0x000a => {
            debug_assert_eq!(item.Length(), 1);
            debug_assert_eq!(item.Type(), InlineItemType::kControl);
            debug_assert_eq!(item.TextType(), TextItemType::kForcedLineBreak);
        }
        0x0009 => {
            debug_assert_eq!(item.Type(), InlineItemType::kControl);
            debug_assert_eq!(item.TextType(), TextItemType::kFlowControl);
        }
        0x000d | 0x000c | 0x200b => {
            if item.Type() == InlineItemType::kControl {
                debug_assert_eq!(item.Length(), 1);
                debug_assert_eq!(item.TextType(), TextItemType::kFlowControl);
            } else {
                debug_assert_eq!(item.Type(), InlineItemType::kText);
                debug_assert_eq!(item.TextType(), TextItemType::kNormal);
            }
        }
        _ => {
            debug_assert_ne!(item.Type(), InlineItemType::kControl);
            debug_assert!(matches!(
                item.TextType(),
                TextItemType::kNormal | TextItemType::kSymbolMarker
            ));
        }
    }
}

// cpp: layoutng_inline/inline_item.cc:309-312
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemTrace(item: &InlineItem, visitor: &mut Visitor<'_>) {
    visitor.Trace(&item.shape_result_);
    visitor.Trace(&item.layout_object_);
}
