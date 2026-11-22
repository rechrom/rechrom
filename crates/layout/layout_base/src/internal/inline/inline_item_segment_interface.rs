#![allow(non_snake_case, unused_unsafe)]

use font_engine::{
    Font, FontFallbackPriority, HarfBuzzShaper, RenderOrientation, RunSegmenter, RunSegmenterRange,
    ShapeOptions, ShapeResult,
};
use foundation::{HeapVector, MakeGarbageCollected, Member, String, TextDirection, TextOffsetMap, Vector, Visitor};

use super::super::inline_item::InlineItem;

// R5: the record members of InlineItemSegment(s) (packing, construction,
// clone; snapshot layoutng_inline/inline_item_segment.cc) are defined here,
// with the fragment-tree data that stores them. Segmentation, orientation,
// splitting and shaping are inline-algorithm code:
// `layout_algo_inline::inline_item_segment::InlineItemSegmentsAlgorithms`.
// cpp: layoutng_inline/inline_item_segment.cc:18-34
const SCRIPT_BITS: u32 = InlineItemSegment::kScriptBits;
const PRIORITY_BITS: u32 = InlineItemSegment::kFontFallbackPriorityBits;
const ORIENTATION_BITS: u32 = InlineItemSegment::kRenderOrientationBits;
const SCRIPT_MASK: u32 = (1 << SCRIPT_BITS) - 1;
const PRIORITY_MASK: u32 = (1 << PRIORITY_BITS) - 1;
const ORIENTATION_MASK: u32 = (1 << ORIENTATION_BITS) - 1;
const _: () =
    assert!(InlineItemSegment::kSegmentDataBits == SCRIPT_BITS + ORIENTATION_BITS + PRIORITY_BITS);

// cpp: layoutng_inline/inline_item_segment.cc:48-50
fn InlineItemSegmentNewFromRange(range: &RunSegmenterRange) -> InlineItemSegment {
    InlineItemSegment::new(range.end, InlineItemSegment::PackSegmentData(range))
}

// cpp: layoutng_inline/inline_item_segment.cc:52-54
fn InlineItemSegmentNewFromItem(
    end_offset: u32,
    item: &InlineItem,
) -> InlineItemSegment {
    InlineItemSegment::new(end_offset, item.SegmentData())
}

// cpp: layoutng_inline/inline_item_segment.cc:56-72
fn InlineItemSegmentPackSegmentData(range: &RunSegmenterRange) -> u32 {
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
fn InlineItemSegmentUnpackSegmentData(
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
fn InlineItemSegmentToRunSegmenterRange(
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
fn InlineItemSegmentsClone(
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


// cpp: layoutng/internal/inline_item_segment.h:37-77
#[derive(Clone)]
pub struct InlineItemSegment {
    pub end_offset_: u32,
    pub segment_data_: u32,
}

impl InlineItemSegment {
    // cpp: layoutng/internal/inline_item_segment.h:41-47
    pub fn new(end_offset: u32, segment_data: u32) -> Self {
        Self {
            end_offset_: end_offset,
            segment_data_: segment_data & Self::segment_data_mask(),
        }
    }

    pub fn new_from_range(range: &RunSegmenterRange) -> Self {
        unsafe { InlineItemSegmentNewFromRange(range) }
    }

    pub fn new_from_item(end_offset: u32, item: &InlineItem) -> Self {
        unsafe { InlineItemSegmentNewFromItem(end_offset, item) }
    }

    pub fn ToRunSegmenterRangeWithEnd(
        &self,
        start_offset: u32,
        end_offset: u32,
    ) -> RunSegmenterRange {
        unsafe { InlineItemSegmentToRunSegmenterRange(self, start_offset, end_offset) }
    }

    // cpp: layoutng/internal/inline_item_segment.h:49-54
    pub fn ToRunSegmenterRange(&self, start_offset: u32) -> RunSegmenterRange {
        self.ToRunSegmenterRangeWithEnd(start_offset, self.end_offset_)
    }

    pub fn EndOffset(&self) -> u32 {
        self.end_offset_
    }

    // cpp: layoutng/internal/inline_item_segment.h:56-68
    pub const kScriptBits: u32 = u32::BITS - (icu_bidi::USCRIPT_CODE_LIMIT as u32).leading_zeros();
    pub const kFontFallbackPriorityBits: u32 =
        u32::BITS - (FontFallbackPriority::kMaxEnumValue as u32).leading_zeros();
    pub const kRenderOrientationBits: u32 =
        u32::BITS - (RenderOrientation::kMaxEnumValue as u32).leading_zeros();
    pub const kSegmentDataBits: u32 =
        Self::kScriptBits + Self::kFontFallbackPriorityBits + Self::kRenderOrientationBits;

    const fn segment_data_mask() -> u32 {
        if Self::kSegmentDataBits == 32 {
            u32::MAX
        } else {
            (1u32 << Self::kSegmentDataBits) - 1
        }
    }

    // cpp: layoutng/internal/inline_item_segment.h:70-72
    pub fn PackSegmentData(range: &RunSegmenterRange) -> u32 {
        unsafe { InlineItemSegmentPackSegmentData(range) }
    }

    pub fn UnpackSegmentData(start_offset: u32, end_offset: u32, value: u32) -> RunSegmenterRange {
        unsafe { InlineItemSegmentUnpackSegmentData(start_offset, end_offset, value) }
    }
}

// cpp: layoutng/internal/inline_item_segment.h:117-117
pub type RunSegmenterRanges = Vector<RunSegmenterRange>;

// cpp: layoutng/internal/inline_item_segment.h:93-177
#[derive(Default)]
pub struct InlineItemSegments {
    pub segments_: HeapVector<InlineItemSegment>,
    pub items_to_segments_: HeapVector<u32>,
}

impl InlineItemSegments {
    // cpp: layoutng/internal/inline_item_segment.h:96-110
    pub fn Clone(&self) -> *mut Self {
        unsafe { InlineItemSegmentsClone(self) }
    }

    pub fn size(&self) -> u32 {
        self.segments_.len() as u32
    }

    pub fn IsEmpty(&self) -> bool {
        self.segments_.is_empty()
    }


    pub fn EndOffset(&self) -> u32 {
        self.segments_
            .last()
            .expect("empty InlineItemSegments")
            .EndOffset()
    }

    pub fn ReserveCapacity(&mut self, capacity: u32) {
        let additional = (capacity as usize).saturating_sub(self.segments_.len());
        self.segments_.reserve(additional);
    }

    // C++ variadic emplace_back becomes a constructed value, preserving each
    // constructor through InlineItemSegment's named constructors above.
    // cpp: layoutng/internal/inline_item_segment.h:112-115
    pub fn Append(&mut self, segment: InlineItemSegment) {
        self.segments_.push(segment);
    }

    // cpp: layoutng/internal/inline_item_segment.h:119-177: ComputeSegments,
    // AppendMixedFontOrientation, AdjustOffsets, ComputeItemIndex, Ranges,
    // ShapeText, PopulateItemsFromFontOrientation, Split, CheckOffset and
    // OffsetForSegment are `InlineItemSegmentsAlgorithms` (layout_algo_inline).

    // cpp: layoutng/internal/inline_item_segment.h:157-160
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.segments_);
        visitor.Trace(&self.items_to_segments_);
    }

}

// cpp: layoutng/internal/inline_item_segment.h:123-149,180-212
pub struct InlineItemSegmentsIterator<'a> {
    range_: RunSegmenterRange,
    span_: &'a [InlineItemSegment],
    segment_index_: u32,
    start_offset_: u32,
    end_offset_: u32,
}

impl<'a> InlineItemSegmentsIterator<'a> {
    pub fn new(
        start_offset: u32,
        end_offset: u32,
        span: &'a [InlineItemSegment],
        segment_index: u32,
    ) -> Self {
        debug_assert!(start_offset < end_offset);
        debug_assert!(start_offset < span[segment_index as usize].EndOffset());
        let range =
            span[segment_index as usize].ToRunSegmenterRangeWithEnd(start_offset, end_offset);
        Self {
            range_: range,
            span_: span,
            segment_index_: segment_index,
            start_offset_: start_offset,
            end_offset_: end_offset,
        }
    }

    pub fn IsDone(&self) -> bool {
        self.range_.start == self.end_offset_
    }

    pub fn begin(&self) -> &Self {
        self
    }

    pub fn end(&self) -> &Self {
        self
    }

    pub fn NotEqual(&self, _other: &Self) -> bool {
        !self.IsDone()
    }

    pub fn Get(&self) -> &RunSegmenterRange {
        &self.range_
    }

    // cpp: layoutng/internal/inline_item_segment.h:214-224
    pub fn Increment(&mut self) {
        debug_assert!(self.range_.end <= self.end_offset_);
        if self.range_.end == self.end_offset_ {
            self.range_.start = self.end_offset_;
            return;
        }
        self.start_offset_ = self.range_.end;
        self.segment_index_ += 1;
        self.range_ = self.span_[self.segment_index_ as usize]
            .ToRunSegmenterRangeWithEnd(self.start_offset_, self.end_offset_);
    }
}
