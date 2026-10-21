#![allow(non_snake_case)]

use cssom::css_property_value_set::CSSPropertyValueSet;
use foundation::{
    AtomicString, HashSet, LayoutUnit, Member, PhysicalOffset, String as BlinkString, Visitor,
    WeakHeapHashMap, WeakMember,
};
use layoutng_style::style::position_try_fallbacks::{
    kNoTryTactics, PositionTryFallbacks, TryTacticList,
};

use super::successful_position_fallback::SuccessfulPositionFallback;
use crate::internal::layout_box::LayoutBox;
use crate::internal::layout_node_metadata::Element;
use crate::internal::layout_object::LayoutObject;
use crate::internal::node_rare_data_field::NodeRareDataField;

// cpp: layoutng/internal/css/out_of_flow_data.h:21-21
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RememberedScrollOffsetType {
    kLayout,
    kRangeAdjustment,
}

// cpp: layoutng/internal/css/out_of_flow_data.h:30-40
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollOffsetPair {
    pub scroll_offset_for_layout: PhysicalOffset,
    pub scroll_offset_for_range_adjustment: PhysicalOffset,
}

// cpp: layoutng/internal/css/out_of_flow_data.h:42-90
#[derive(Default)]
pub struct RememberedScrollOffsets {
    offsets_: WeakHeapHashMap<Element, ScrollOffsetPair>,
}

impl RememberedScrollOffsets {
    // cpp: layoutng/internal/css/out_of_flow_data.h:47-55
    pub fn GetOffsetsForAnchor(&self, anchor: *const Element) -> Option<ScrollOffsetPair> {
        if anchor.is_null() {
            return None;
        }
        self.offsets_
            .get(&WeakMember::from_ptr(anchor as *mut Element))
            .cloned()
    }

    // cpp: layoutng/internal/css/out_of_flow_data.h:56-75
    pub fn GetOffset(
        &self,
        anchor: *const Element,
        kind: RememberedScrollOffsetType,
        needs_x: bool,
        needs_y: bool,
    ) -> Option<PhysicalOffset> {
        let offsets = self.GetOffsetsForAnchor(anchor)?;
        let mut result = match kind {
            RememberedScrollOffsetType::kLayout => offsets.scroll_offset_for_layout,
            RememberedScrollOffsetType::kRangeAdjustment => {
                offsets.scroll_offset_for_range_adjustment
            }
        };
        if !needs_x {
            result.left = LayoutUnit::default();
        }
        if !needs_y {
            result.top = LayoutUnit::default();
        }
        Some(result)
    }

    // cpp: layoutng/internal/css/out_of_flow_data.h:76-79
    pub fn SetOffsetsForAnchor(&mut self, anchor: *const Element, offsets: &ScrollOffsetPair) {
        self.offsets_.Set(
            WeakMember::from_ptr(anchor as *mut Element),
            offsets.clone(),
        );
    }

    // cpp: layoutng/internal/css/out_of_flow_data.h:85-85
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.offsets_);
    }
}

// cpp: layoutng/internal/css/out_of_flow_data.h:81-83
impl PartialEq for RememberedScrollOffsets {
    fn eq(&self, other: &Self) -> bool {
        self.offsets_ == other.offsets_
    }
}

// cpp: layoutng/internal/css/out_of_flow_data.h:93-165
pub struct OutOfFlowData {
    rare_data_: NodeRareDataField,
    last_successful_position_fallback_: SuccessfulPositionFallback,
    new_successful_position_fallback_: SuccessfulPositionFallback,
    remembered_scroll_offsets_: Member<RememberedScrollOffsets>,
    pending_remembered_scroll_offsets_: Member<RememberedScrollOffsets>,
}

impl Default for OutOfFlowData {
    fn default() -> Self {
        Self {
            rare_data_: NodeRareDataField,
            last_successful_position_fallback_: SuccessfulPositionFallback::default(),
            new_successful_position_fallback_: SuccessfulPositionFallback::default(),
            remembered_scroll_offsets_: Member::default(),
            pending_remembered_scroll_offsets_: Member::default(),
        }
    }
}

impl OutOfFlowData {
    // cpp: layoutng_out_of_flow/out_of_flow_element_data.cc:17-22
    pub fn HasStaleFallbackData(&self, box_: &LayoutBox) -> bool {
        !self.last_successful_position_fallback_.IsEmpty()
            && !foundation::ValuesEquivalent(
                &self
                    .last_successful_position_fallback_
                    .position_try_fallbacks_,
                box_.StyleRef().GetPositionTryFallbacks(),
            )
    }

    // cpp: layoutng_out_of_flow/out_of_flow_element_data.cc:24-27
    pub fn GetRememberedScrollOffsets(&self) -> *const RememberedScrollOffsets {
        self.remembered_scroll_offsets_.Get()
    }

    // cpp: layoutng_out_of_flow/out_of_flow_element_data.cc:29-39
    pub fn SetPendingRememberedScrollOffsets(
        &mut self,
        offsets: *const RememberedScrollOffsets,
    ) -> bool {
        if offsets.is_null() {
            self.pending_remembered_scroll_offsets_ = Member::default();
            return self.remembered_scroll_offsets_.Get().is_null();
        }
        let remembered = self.remembered_scroll_offsets_.Get();
        if !remembered.is_null() && unsafe { &*remembered } == unsafe { &*offsets } {
            return false;
        }
        self.pending_remembered_scroll_offsets_ = Member::from_ptr(offsets as *mut _);
        true
    }

    // cpp: layoutng/internal/css/out_of_flow_data.h:111-114
    pub fn HasLastSuccessfulPositionFallback(&self) -> bool {
        !self
            .last_successful_position_fallback_
            .position_try_fallbacks_
            .Get()
            .is_null()
    }

    // cpp: layoutng/internal/css/out_of_flow_data.h:121-123
    pub fn GetLastSuccessfulTrySet(&self) -> *const CSSPropertyValueSet {
        self.last_successful_position_fallback_.try_set_.Get()
    }

    // cpp: layoutng/internal/css/out_of_flow_data.h:125-127
    pub fn GetLastSuccessfulTryTactics(&self) -> &TryTacticList {
        &self.last_successful_position_fallback_.try_tactics_
    }

    // cpp: layoutng/internal/css/out_of_flow_data.h:129-131
    pub fn GetLastSuccessfulIndex(&self) -> Option<usize> {
        self.last_successful_position_fallback_.index_
    }

    // cpp: layoutng/internal/css/out_of_flow_data.h:133-138
    pub fn GetNewSuccessfulPositionFallbackIndex(&self) -> Option<usize> {
        self.new_successful_position_fallback_
            .index_
            .or(self.last_successful_position_fallback_.index_)
    }

    // cpp: layoutng/internal/css/out_of_flow_data.h:148-151
    pub fn ClearRememberedScrollOffsets(&mut self) {
        self.remembered_scroll_offsets_ = Member::default();
        self.pending_remembered_scroll_offsets_ = Member::default();
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.cc:38-42
    pub fn GetSpeculativeRememberedScrollOffsets(&self) -> *const RememberedScrollOffsets {
        let pending = self.pending_remembered_scroll_offsets_.Get();
        if pending.is_null() {
            self.remembered_scroll_offsets_.Get()
        } else {
            pending
        }
    }

    // cpp: layoutng/internal/layout_node_metadata.cc:73-79
    pub fn Trace(&self, visitor: &mut Visitor) {
        self.rare_data_.Trace(visitor);
        self.last_successful_position_fallback_.Trace(visitor);
        self.new_successful_position_fallback_.Trace(visitor);
        visitor.Trace(&self.remembered_scroll_offsets_);
        visitor.Trace(&self.pending_remembered_scroll_offsets_);
    }
}

// cpp: layoutng/internal/css/out_of_flow_data.h:87-87
// RememberedScrollOffsets::ToString is declared without a definition in the
// supplied source tree. The remaining declarations below are implemented in
// other source units or likewise have no supplied definition.
#[allow(non_snake_case)]
pub trait OutOfFlowDataExternal {
    // cpp: layoutng/internal/css/out_of_flow_data.h:95-99
    fn SetPendingSuccessfulPositionFallback(
        &mut self,
        fallbacks: *const PositionTryFallbacks,
        try_set: *const CSSPropertyValueSet,
        try_tactics: &TryTacticList,
        index: Option<usize>,
    ) -> bool;

    // cpp: layoutng/internal/css/out_of_flow_data.h:101-104
    fn ClearPendingSuccessfulPositionFallback(&mut self) -> bool {
        self.SetPendingSuccessfulPositionFallback(
            std::ptr::null(),
            std::ptr::null(),
            &kNoTryTactics,
            None,
        )
    }

    // cpp: layoutng/internal/css/out_of_flow_data.h:108-109
    fn ApplyPendingSuccessfulPositionFallbackAndAnchorScrollShift(
        &mut self,
        layout_object: *mut LayoutObject,
    ) -> bool;

    // cpp: layoutng/internal/css/out_of_flow_data.h:119-119
    fn InvalidatePositionTryNames(&mut self, try_names: &HashSet<AtomicString>) -> bool;

    // cpp: layoutng/internal/css/out_of_flow_data.h:142-142
    fn HasStaleFallbackData(&self, box_: &LayoutBox) -> bool;

    // cpp: layoutng/internal/css/out_of_flow_data.h:144-146
    fn GetRememberedScrollOffsets(&self) -> *const RememberedScrollOffsets;
    fn SetPendingRememberedScrollOffsets(
        &mut self,
        offsets: *const RememberedScrollOffsets,
    ) -> bool;

    // cpp: layoutng/internal/css/out_of_flow_data.h:156-156
    fn ResetAnchorData(&mut self);
}

#[allow(non_snake_case)]
pub trait RememberedScrollOffsetsExternal {
    // cpp: layoutng/internal/css/out_of_flow_data.h:87-87
    fn ToString(&self) -> BlinkString;
}
