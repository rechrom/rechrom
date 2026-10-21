#![allow(non_snake_case)]

use font_engine::{
    Font, FontFallbackPriority, HarfBuzzShaper, RenderOrientation, RunSegmenter, RunSegmenterRange,
    ShapeOptions, ShapeResult,
};
use foundation::{HeapVector, Member, String, TextDirection, TextOffsetMap, Vector, Visitor};

use super::inline_item::InlineItem;

// Non-inline bodies are owned by //src/layoutng_inline/inline_item_segment.cc.
unsafe extern "Rust" {
    fn InlineItemSegmentNewFromRange(range: &RunSegmenterRange) -> InlineItemSegment;
    fn InlineItemSegmentNewFromItem(end_offset: u32, item: &InlineItem) -> InlineItemSegment;
    fn InlineItemSegmentToRunSegmenterRange(
        segment: &InlineItemSegment,
        start_offset: u32,
        end_offset: u32,
    ) -> RunSegmenterRange;
    fn InlineItemSegmentPackSegmentData(range: &RunSegmenterRange) -> u32;
    fn InlineItemSegmentUnpackSegmentData(
        start_offset: u32,
        end_offset: u32,
        value: u32,
    ) -> RunSegmenterRange;
    fn InlineItemSegmentsClone(segments: &InlineItemSegments) -> *mut InlineItemSegments;
    fn InlineItemSegmentsOffsetForSegment(
        segments: &InlineItemSegments,
        segment: &InlineItemSegment,
    ) -> u32;
    fn InlineItemSegmentsComputeSegments(
        segments: &mut InlineItemSegments,
        segmenter: *mut RunSegmenter,
        range: *mut RunSegmenterRange,
    );
    fn InlineItemSegmentsAppendMixedFontOrientation(
        segments: &mut InlineItemSegments,
        text_content: &String,
        start_offset: u32,
        end_offset: u32,
        segment_index: u32,
    ) -> u32;
    fn InlineItemSegmentsAdjustOffsets(
        segments: &mut InlineItemSegments,
        offset_map: &TextOffsetMap,
    );
    fn InlineItemSegmentsComputeItemIndex(
        segments: &mut InlineItemSegments,
        items: &HeapVector<Member<InlineItem>>,
    );
    fn InlineItemSegmentsToRanges(segments: &InlineItemSegments, ranges: &mut RunSegmenterRanges);
    fn InlineItemSegmentsRanges<'a>(
        segments: &'a InlineItemSegments,
        start_offset: u32,
        end_offset: u32,
        item_index: u32,
    ) -> InlineItemSegmentsIterator<'a>;
    fn InlineItemSegmentsShapeText(
        segments: &InlineItemSegments,
        shaper: *const HarfBuzzShaper,
        font: *const Font,
        direction: TextDirection,
        start_offset: u32,
        end_offset: u32,
        item_index: u32,
        options: ShapeOptions,
    ) -> *mut ShapeResult;
    fn InlineItemSegmentsPopulateItemsFromFontOrientation(
        segments: &mut InlineItemSegments,
        start_offset: u32,
        end_offset: u32,
        orientation: RenderOrientation,
        segment_index: u32,
    ) -> u32;
    fn InlineItemSegmentsSplit(segments: &mut InlineItemSegments, index: u32, offset: u32);
    #[cfg(debug_assertions)]
    fn InlineItemSegmentsCheckOffset(
        segments: &InlineItemSegments,
        offset: u32,
        segment: *const InlineItemSegment,
    );
}

// cpp: layoutng/internal/inline_item_segment.h:37-77
#[derive(Clone)]
pub struct InlineItemSegment {
    pub(crate) end_offset_: u32,
    pub(crate) segment_data_: u32,
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
    pub(crate) segments_: HeapVector<InlineItemSegment>,
    pub(crate) items_to_segments_: HeapVector<u32>,
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

    pub fn OffsetForSegment(&self, segment: &InlineItemSegment) -> u32 {
        unsafe { InlineItemSegmentsOffsetForSegment(self, segment) }
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

    // cpp: layoutng/internal/inline_item_segment.h:119-177
    pub fn ComputeSegments(&mut self, segmenter: *mut RunSegmenter, range: *mut RunSegmenterRange) {
        unsafe { InlineItemSegmentsComputeSegments(self, segmenter, range) }
    }

    pub fn AppendMixedFontOrientation(
        &mut self,
        text_content: &String,
        start_offset: u32,
        end_offset: u32,
        segment_index: u32,
    ) -> u32 {
        unsafe {
            InlineItemSegmentsAppendMixedFontOrientation(
                self,
                text_content,
                start_offset,
                end_offset,
                segment_index,
            )
        }
    }

    pub fn AdjustOffsets(&mut self, offset_map: &TextOffsetMap) {
        unsafe { InlineItemSegmentsAdjustOffsets(self, offset_map) }
    }

    pub fn ComputeItemIndex(&mut self, items: &HeapVector<Member<InlineItem>>) {
        unsafe { InlineItemSegmentsComputeItemIndex(self, items) }
    }

    pub fn ToRanges(&self, ranges: &mut RunSegmenterRanges) {
        unsafe { InlineItemSegmentsToRanges(self, ranges) }
    }

    pub fn Ranges(
        &self,
        start_offset: u32,
        end_offset: u32,
        item_index: u32,
    ) -> InlineItemSegmentsIterator<'_> {
        unsafe { InlineItemSegmentsRanges(self, start_offset, end_offset, item_index) }
    }

    pub fn ShapeText(
        &self,
        shaper: *const HarfBuzzShaper,
        font: *const Font,
        direction: TextDirection,
        start_offset: u32,
        end_offset: u32,
        item_index: u32,
        options: ShapeOptions,
    ) -> *mut ShapeResult {
        unsafe {
            InlineItemSegmentsShapeText(
                self,
                shaper,
                font,
                direction,
                start_offset,
                end_offset,
                item_index,
                options,
            )
        }
    }

    pub fn ShapeTextDefault(
        &self,
        shaper: *const HarfBuzzShaper,
        font: *const Font,
        direction: TextDirection,
        start_offset: u32,
        end_offset: u32,
        item_index: u32,
    ) -> *mut ShapeResult {
        self.ShapeText(
            shaper,
            font,
            direction,
            start_offset,
            end_offset,
            item_index,
            ShapeOptions::default(),
        )
    }

    // cpp: layoutng/internal/inline_item_segment.h:157-160
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.segments_);
        visitor.Trace(&self.items_to_segments_);
    }

    pub fn PopulateItemsFromFontOrientation(
        &mut self,
        start_offset: u32,
        end_offset: u32,
        orientation: RenderOrientation,
        segment_index: u32,
    ) -> u32 {
        unsafe {
            InlineItemSegmentsPopulateItemsFromFontOrientation(
                self,
                start_offset,
                end_offset,
                orientation,
                segment_index,
            )
        }
    }

    pub fn Split(&mut self, index: u32, offset: u32) {
        unsafe { InlineItemSegmentsSplit(self, index, offset) }
    }

    #[cfg(debug_assertions)]
    pub fn CheckOffset(&self, offset: u32, segment: *const InlineItemSegment) {
        unsafe { InlineItemSegmentsCheckOffset(self, offset, segment) }
    }

    #[cfg(not(debug_assertions))]
    pub fn CheckOffset(&self, _offset: u32, _segment: *const InlineItemSegment) {}
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
