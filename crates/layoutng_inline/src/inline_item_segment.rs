// C++: layoutng_inline/inline_item_segment.cc. Non-inline implementations of
// types owned by //src/layoutng; linked in the shared assembly.
#![allow(non_snake_case)]

use font_engine::fonts::orientation_iterator::OrientationIterator;
use font_engine::{
    Font, FontFallbackPriority, FontOrientation, HarfBuzzShaper, RenderOrientation, RunSegmenter,
    RunSegmenterRange, ShapeOptions, ShapeResult,
};
use foundation::{HeapVector, MakeGarbageCollected, Member, String, TextDirection, TextOffsetMap};
use layoutng::internal::inline_item::InlineItem;
use layoutng::internal::inline_item_segment::{
    InlineItemSegment, InlineItemSegments, InlineItemSegmentsIterator, RunSegmenterRanges,
};

// cpp: layoutng_inline/inline_item_segment.cc:18-34
const SCRIPT_BITS: u32 = InlineItemSegment::kScriptBits;
const PRIORITY_BITS: u32 = InlineItemSegment::kFontFallbackPriorityBits;
const ORIENTATION_BITS: u32 = InlineItemSegment::kRenderOrientationBits;
const SCRIPT_MASK: u32 = (1 << SCRIPT_BITS) - 1;
const PRIORITY_MASK: u32 = (1 << PRIORITY_BITS) - 1;
const ORIENTATION_MASK: u32 = (1 << ORIENTATION_BITS) - 1;
const _: () =
    assert!(InlineItemSegment::kSegmentDataBits == SCRIPT_BITS + ORIENTATION_BITS + PRIORITY_BITS);

// cpp: layoutng_inline/inline_item_segment.cc:36-44
fn SetRenderOrientation(value: u32, orientation: RenderOrientation) -> u32 {
    debug_assert_ne!(orientation, RenderOrientation::kOrientationInvalid);
    (value & !ORIENTATION_MASK) | u32::from(orientation != RenderOrientation::kOrientationKeep)
}

// cpp: layoutng_inline/inline_item_segment.cc:48-50
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSegmentNewFromRange(range: &RunSegmenterRange) -> InlineItemSegment {
    InlineItemSegment::new(range.end, InlineItemSegment::PackSegmentData(range))
}

// cpp: layoutng_inline/inline_item_segment.cc:52-54
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSegmentNewFromItem(
    end_offset: u32,
    item: &InlineItem,
) -> InlineItemSegment {
    InlineItemSegment::new(end_offset, item.SegmentData())
}

// cpp: layoutng_inline/inline_item_segment.cc:56-72
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSegmentPackSegmentData(range: &RunSegmenterRange) -> u32 {
    debug_assert!(range.script == -1 || (range.script >= 0 && range.script as u32 <= SCRIPT_MASK));
    debug_assert!((range.font_fallback_priority as u32) <= PRIORITY_MASK);
    debug_assert!((range.render_orientation as u32) <= ORIENTATION_MASK);
    let mut value = if range.script == -1 {
        SCRIPT_MASK
    } else {
        range.script as u32
    };
    value <<= PRIORITY_BITS;
    value |= range.font_fallback_priority as u32;
    value <<= ORIENTATION_BITS;
    value |= range.render_orientation as u32;
    value
}

// cpp: layoutng_inline/inline_item_segment.cc:74-89
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSegmentUnpackSegmentData(
    start_offset: u32,
    end_offset: u32,
    mut value: u32,
) -> RunSegmenterRange {
    let orientation = value & ORIENTATION_MASK;
    value >>= ORIENTATION_BITS;
    let priority = value & PRIORITY_MASK;
    value >>= PRIORITY_BITS;
    let script = value & SCRIPT_MASK;
    RunSegmenterRange {
        start: start_offset,
        end: end_offset,
        script: if script == SCRIPT_MASK {
            -1
        } else {
            script as i32
        },
        render_orientation: match orientation {
            0 => RenderOrientation::kOrientationKeep,
            1 => RenderOrientation::kOrientationRotateSideways,
            _ => panic!("invalid packed render orientation"),
        },
        font_fallback_priority: match priority {
            0 => FontFallbackPriority::kText,
            1 => FontFallbackPriority::kEmojiText,
            2 => FontFallbackPriority::kEmojiTextWithVS,
            3 => FontFallbackPriority::kEmojiEmoji,
            4 => FontFallbackPriority::kEmojiEmojiWithVS,
            5 => FontFallbackPriority::kInvalid,
            _ => panic!("invalid packed font fallback priority"),
        },
    }
}

// cpp: layoutng_inline/inline_item_segment.cc:91-98
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSegmentToRunSegmenterRange(
    segment: &InlineItemSegment,
    start_offset: u32,
    end_offset: u32,
) -> RunSegmenterRange {
    debug_assert!(start_offset < end_offset);
    debug_assert!(start_offset < segment.end_offset_);
    InlineItemSegment::UnpackSegmentData(
        start_offset,
        end_offset.min(segment.end_offset_),
        segment.segment_data_,
    )
}

// cpp: layoutng_inline/inline_item_segment.cc:100-105
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSegmentsClone(
    segments: &InlineItemSegments,
) -> *mut InlineItemSegments {
    let clone = MakeGarbageCollected(InlineItemSegments::default());
    unsafe { &mut *clone }
        .segments_
        .extend_from_slice(&segments.segments_);
    unsafe { &mut *clone }
        .items_to_segments_
        .extend_from_slice(&segments.items_to_segments_);
    clone
}

// cpp: layoutng_inline/inline_item_segment.cc:107-110
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSegmentsOffsetForSegment(
    segments: &InlineItemSegments,
    segment: &InlineItemSegment,
) -> u32 {
    if std::ptr::eq(segment, segments.segments_.as_ptr()) {
        0
    } else {
        unsafe { &*(segment as *const InlineItemSegment).sub(1) }.EndOffset()
    }
}

// cpp: layoutng_inline/inline_item_segment.cc:112-120
#[cfg(debug_assertions)]
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSegmentsCheckOffset(
    segments: &InlineItemSegments,
    offset: u32,
    segment: *const InlineItemSegment,
) {
    debug_assert!(segment.addr() >= segments.segments_.as_ptr().addr());
    debug_assert!(segment.addr() < segments.segments_.as_ptr_range().end.addr());
    debug_assert!(offset >= segments.OffsetForSegment(unsafe { &*segment }));
    debug_assert!(offset < unsafe { &*segment }.EndOffset());
}

// cpp: layoutng_inline/inline_item_segment.cc:122-129
pub fn InlineItemSegmentsToRanges(segments: &InlineItemSegments, ranges: &mut RunSegmenterRanges) {
    ranges.reserve(segments.segments_.len());
    let mut start_offset = 0;
    for segment in segments.segments_.iter() {
        ranges.push(segment.ToRunSegmenterRange(start_offset));
        start_offset = segment.EndOffset();
    }
}

// cpp: layoutng_inline/inline_item_segment.cc:131-160
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSegmentsRanges(
    segments: &InlineItemSegments,
    start_offset: u32,
    end_offset: u32,
    item_index: u32,
) -> InlineItemSegmentsIterator<'_> {
    debug_assert!(start_offset < end_offset);
    debug_assert!(end_offset <= segments.EndOffset());
    let mut segment_index = segments.items_to_segments_[item_index as usize] as usize;
    let segment = &segments.segments_[segment_index];
    debug_assert!(start_offset >= segments.OffsetForSegment(segment));
    let span = segments.segments_.as_slice();
    if start_offset < segment.EndOffset() {
        return InlineItemSegmentsIterator::new(
            start_offset,
            end_offset,
            span,
            segment_index as u32,
        );
    }
    let end_segment_index = if (item_index as usize + 1) < segments.items_to_segments_.len() {
        segments.items_to_segments_[item_index as usize + 1] as usize
    } else {
        segments.segments_.len()
    };
    assert!(end_segment_index > segment_index);
    assert!(end_segment_index <= segments.segments_.len());
    let relative = span[segment_index..end_segment_index]
        .partition_point(|segment| segment.EndOffset() <= start_offset);
    segment_index += relative;
    segments.CheckOffset(start_offset, &span[segment_index]);
    InlineItemSegmentsIterator::new(start_offset, end_offset, span, segment_index as u32)
}

// cpp: layoutng_inline/inline_item_segment.cc:162-169
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSegmentsComputeSegments(
    segments: &mut InlineItemSegments,
    segmenter: *mut RunSegmenter,
    range: *mut RunSegmenterRange,
) {
    segments.segments_.truncate(0);
    loop {
        segments
            .segments_
            .push(InlineItemSegment::new_from_range(unsafe { &*range }));
        if !unsafe { &mut *segmenter }.Consume(unsafe { &mut *range }) {
            break;
        }
    }
}

// cpp: layoutng_inline/inline_item_segment.cc:171-189
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSegmentsAppendMixedFontOrientation(
    segments: &mut InlineItemSegments,
    text_content: &String,
    mut start_offset: u32,
    mut end_offset: u32,
    mut segment_index: u32,
) -> u32 {
    debug_assert!(start_offset < end_offset);
    let text = text_content.Span16().expect("text content is null");
    let mut iterator = OrientationIterator::new(
        &text[start_offset as usize..end_offset as usize],
        FontOrientation::kVerticalMixed,
    );
    let original_start_offset = start_offset;
    let mut orientation = RenderOrientation::kOrientationInvalid;
    while iterator.Consume(&mut end_offset, &mut orientation) {
        end_offset += original_start_offset;
        segment_index = segments.PopulateItemsFromFontOrientation(
            start_offset,
            end_offset,
            orientation,
            segment_index,
        );
        start_offset = end_offset;
    }
    segment_index
}

// cpp: layoutng_inline/inline_item_segment.cc:191-223
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSegmentsPopulateItemsFromFontOrientation(
    segments: &mut InlineItemSegments,
    start_offset: u32,
    end_offset: u32,
    orientation: RenderOrientation,
    mut segment_index: u32,
) -> u32 {
    debug_assert!(start_offset < end_offset);
    debug_assert!(end_offset <= segments.segments_.last().unwrap().EndOffset());
    while start_offset >= segments.segments_[segment_index as usize].EndOffset() {
        segment_index += 1;
    }
    let previous_end = if segment_index == 0 {
        0
    } else {
        segments.segments_[(segment_index - 1) as usize].EndOffset()
    };
    if start_offset != previous_end {
        segments.Split(segment_index, start_offset);
        segment_index += 1;
    }
    loop {
        let current_end = segments.segments_[segment_index as usize].EndOffset();
        let value = segments.segments_[segment_index as usize].segment_data_;
        segments.segments_[segment_index as usize].segment_data_ =
            SetRenderOrientation(value, orientation);
        if end_offset == current_end {
            segment_index += 1;
            break;
        }
        if end_offset < current_end {
            segments.Split(segment_index, end_offset);
            segment_index += 1;
            break;
        }
        segment_index += 1;
    }
    segment_index
}

// cpp: layoutng_inline/inline_item_segment.cc:225-232
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSegmentsSplit(
    segments: &mut InlineItemSegments,
    index: u32,
    offset: u32,
) {
    let segment = &mut segments.segments_[index as usize];
    debug_assert!(offset < segment.EndOffset());
    let end_offset = segment.EndOffset();
    segment.end_offset_ = offset;
    let data = segment.segment_data_;
    segments
        .segments_
        .insert(index as usize + 1, InlineItemSegment::new(end_offset, data));
}

// cpp: layoutng_inline/inline_item_segment.cc:234-238
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSegmentsAdjustOffsets(
    segments: &mut InlineItemSegments,
    map: &TextOffsetMap,
) {
    for segment in segments.segments_.iter_mut() {
        segment.end_offset_ = map.MapOffset(segment.end_offset_);
    }
}

// cpp: layoutng_inline/inline_item_segment.cc:240-253
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSegmentsComputeItemIndex(
    segments: &mut InlineItemSegments,
    items: &HeapVector<Member<InlineItem>>,
) {
    debug_assert_eq!(
        unsafe { &*items.last().expect("empty inline items").Get() }.EndOffset(),
        segments.EndOffset()
    );
    let mut segment_index = 0;
    let mut item_index = 0;
    segments.items_to_segments_.resize(items.len(), 0);
    for item_ptr in items.iter() {
        let item = unsafe { &*item_ptr.Get() };
        while segment_index < segments.segments_.len()
            && item.StartOffset() >= segments.segments_[segment_index].EndOffset()
        {
            segment_index += 1;
        }
        segments.items_to_segments_[item_index] = segment_index as u32;
        item_index += 1;
    }
}

// cpp: layoutng_inline/inline_item_segment.cc:255-273
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineItemSegmentsShapeText(
    segments: &InlineItemSegments,
    shaper: *const HarfBuzzShaper,
    font: *const Font,
    direction: TextDirection,
    start_offset: u32,
    end_offset: u32,
    item_index: u32,
    options: ShapeOptions,
) -> *mut ShapeResult {
    let mut ranges = Vec::<RunSegmenterRange>::new();
    let mut iterator = segments.Ranges(start_offset, end_offset, item_index);
    while !iterator.IsDone() {
        ranges.push(*iterator.Get());
        iterator.Increment();
    }
    let result = unsafe { &*shaper }.ShapeWithRanges(
        font,
        direction,
        start_offset,
        end_offset,
        &ranges,
        options,
    );
    debug_assert!(!result.is_null());
    debug_assert_eq!(unsafe { &*result }.StartIndex(), start_offset);
    debug_assert_eq!(unsafe { &*result }.EndIndex(), end_offset);
    result
}
