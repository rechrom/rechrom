// C++: layoutng_inline/line_break_candidate.h/.cc.
#![allow(non_snake_case)]

use std::fmt;
use std::ops::{Deref, DerefMut};

use foundation::{LayoutUnit, RuntimeEnabledFeatures, WtfSizeT};
use layoutng::internal::inline_item::{InlineItem, InlineItemType};
use layoutng::internal::inline_item_result::InlineItemResult;
use layoutng::internal::inline_item_text_index::InlineItemTextIndex;

use crate::line_break_point::LineBreakPoint;
use crate::line_info::LineInfo;

// The source LineBreaker owns this behavior. The trait passes AppendLine's
// callback into the line breaker's translated candidate collector without
// moving the algorithm into this file.
pub trait CandidateLineBreaker {
    fn AppendCandidates(
        &mut self,
        item_result: &InlineItemResult,
        line_info: &LineInfo,
        context: &mut LineBreakCandidateContext<'_>,
    );
}

// cpp: layoutng_inline/line_break_candidate.h:23-58
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct LineBreakCandidate {
    pub base: LineBreakPoint,
    pub pos_no_break: f32,
    pub pos_if_break: f32,
    pub penalty: f32,
}

impl Deref for LineBreakCandidate {
    type Target = LineBreakPoint;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for LineBreakCandidate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl LineBreakCandidate {
    // cpp: layoutng_inline/line_break_candidate.h:29-41
    pub fn new(
        offset: InlineItemTextIndex,
        end: InlineItemTextIndex,
        pos_no_break: f32,
        pos_if_break: f32,
        penalty: f32,
        is_hyphenated: bool,
    ) -> Self {
        Self {
            base: LineBreakPoint::new(offset, end, is_hyphenated),
            pos_no_break,
            pos_if_break,
            penalty,
        }
    }

    pub fn at_position(offset: InlineItemTextIndex, position: f32) -> Self {
        Self::new(offset, offset, position, position, 0.0, false)
    }

    // cpp: layoutng_inline/line_break_candidate.h:56
    pub const kInlineCapacity: usize = 128;
}

// cpp: layoutng_inline/line_break_candidate.h:43-47
impl PartialEq for LineBreakCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base
            && self.pos_no_break == other.pos_no_break
            && self.pos_if_break == other.pos_if_break
            && self.penalty == other.penalty
    }
}

// cpp: layoutng_inline/line_break_candidate.h:59-68
// C++ Vector's inline capacity is a storage hint. Rust Vec keeps the same
// element order and a caller may reserve kInlineCapacity when useful.
pub type LineBreakCandidates = Vec<LineBreakCandidate>;

// cpp: layoutng_inline/line_break_candidate.h:72-136
pub struct LineBreakCandidateContext<'a> {
    position_no_snap_: f32,
    state_: State,
    last_item_: *const InlineItem,
    last_end_offset_: WtfSizeT,
    hyphen_penalty_: f32,
    candidates_: &'a mut LineBreakCandidates,
    #[cfg(feature = "expensive_dchecks")]
    first_offset_: InlineItemTextIndex,
}

// cpp: layoutng_inline/line_break_candidate.h:82
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    kBreak,
    kMidWord,
}

impl<'a> LineBreakCandidateContext<'a> {
    // cpp: layoutng_inline/line_break_candidate.h:77-79
    pub fn new(candidates: &'a mut LineBreakCandidates) -> Self {
        Self {
            position_no_snap_: 0.0,
            state_: State::kBreak,
            last_item_: std::ptr::null(),
            last_end_offset_: 0,
            hyphen_penalty_: 0.0,
            candidates_: candidates,
            #[cfg(feature = "expensive_dchecks")]
            first_offset_: InlineItemTextIndex::default(),
        }
    }

    // cpp: layoutng_inline/line_break_candidate.h:80-105
    pub fn HyphenPenalty(&self) -> f32 {
        self.hyphen_penalty_
    }
    pub fn SetHyphenPenalty(&mut self, penalty: f32) {
        self.hyphen_penalty_ = penalty;
    }
    pub fn GetState(&self) -> State {
        self.state_
    }
    pub fn Position(&self) -> f32 {
        self.position_no_snap_
    }
    pub fn SnappedPosition(&self) -> LayoutUnit {
        LayoutUnit::FromFloatCeil(self.position_no_snap_)
    }
    pub fn Candidates(&self) -> &LineBreakCandidates {
        &*self.candidates_
    }
    pub fn LastItem(&self) -> *const InlineItem {
        self.last_item_
    }
    pub fn LastEndOffset(&self) -> WtfSizeT {
        self.last_end_offset_
    }
    pub fn SetLast(&mut self, item: &InlineItem, offset: WtfSizeT) {
        self.last_item_ = item;
        self.last_end_offset_ = offset;
    }

    // cpp: layoutng_inline/line_break_candidate.cc:30-77
    pub fn Append(
        &mut self,
        new_state: State,
        offset: InlineItemTextIndex,
        end: InlineItemTextIndex,
        pos_no_break: f32,
        pos_if_break: f32,
        penalty: f32,
        is_hyphenated: bool,
    ) {
        #[cfg(feature = "expensive_dchecks")]
        {
            debug_assert!(offset.GreaterOrEqual(&end));
            if let Some(last_candidate) = self.candidates_.last() {
                if self.state_ == State::kBreak {
                    debug_assert!(offset.GreaterThan(&last_candidate.offset));
                } else {
                    debug_assert!(offset.GreaterOrEqual(&last_candidate.offset));
                }
                debug_assert!(end.GreaterOrEqual(&last_candidate.end));
                if self.position_no_snap_.is_nan() {
                    debug_assert!(last_candidate.pos_no_break.is_nan());
                } else if self.position_no_snap_ < LayoutUnit::NearlyMax().ToFloat() {
                    debug_assert_eq!(self.position_no_snap_, last_candidate.pos_no_break);
                    debug_assert!(pos_no_break >= last_candidate.pos_no_break);
                }
            }
        }

        match self.state_ {
            State::kBreak => self.candidates_.push(LineBreakCandidate::new(
                offset,
                end,
                pos_no_break,
                pos_if_break,
                penalty,
                is_hyphenated,
            )),
            State::kMidWord => {
                let last_candidate = self.candidates_.last_mut().expect("mid-word candidate");
                last_candidate.offset = offset;
                last_candidate.end = end;
                last_candidate.pos_no_break = pos_no_break;
                last_candidate.pos_if_break = pos_if_break;
                last_candidate.penalty = penalty;
                last_candidate.is_hyphenated = is_hyphenated;
            }
        }
        self.position_no_snap_ = pos_no_break;
        self.state_ = new_state;
    }

    // cpp: layoutng_inline/line_break_candidate.cc:79-83
    pub fn AppendAtPosition(
        &mut self,
        new_state: State,
        offset: InlineItemTextIndex,
        position: f32,
    ) {
        self.Append(new_state, offset, offset, position, position, 0.0, false);
    }

    // cpp: layoutng_inline/line_break_candidate.cc:85-96
    pub fn AppendTrailingSpaces(
        &mut self,
        new_state: State,
        offset: InlineItemTextIndex,
        pos_no_break: f32,
    ) {
        let last_candidate = self.candidates_.last_mut().expect("candidate sentinel");
        debug_assert!(offset.GreaterOrEqual(&last_candidate.offset));
        debug_assert_eq!(self.position_no_snap_, last_candidate.pos_no_break);
        last_candidate.offset = offset;
        last_candidate.pos_no_break = pos_no_break;
        self.position_no_snap_ = pos_no_break;
        self.state_ = new_state;
    }

    // cpp: layoutng_inline/line_break_candidate.cc:17-27
    fn LastNonOutOfFlowPositionedItemResult(line_info: &LineInfo) -> Option<usize> {
        line_info.Results().iter().rposition(|result| {
            let item = unsafe { &*result.item.Get() };
            item.Type() != InlineItemType::kOutOfFlowPositioned
        })
    }

    // cpp: layoutng_inline/line_break_candidate.cc:98-175
    // The source const_casts the last InlineItemResult to set can_break_after.
    // Rust takes &mut LineInfo explicitly for that real mutation.
    pub fn AppendLine(
        &mut self,
        line_info: &mut LineInfo,
        line_breaker: &mut dyn CandidateLineBreaker,
    ) -> bool {
        let last_index = if RuntimeEnabledFeatures::SkipOofItemForBreakCandidateEnabled() {
            Self::LastNonOutOfFlowPositionedItemResult(line_info)
        } else {
            line_info.Results().len().checked_sub(1)
        };
        if let Some(index) = last_index {
            if !line_info.Results()[index].can_break_after {
                line_info.MutableResults()[index].can_break_after = true;
            }
        }

        for item_result in line_info.Results() {
            if item_result.inline_size < LayoutUnit::default() {
                return false;
            }
            let item = unsafe { &*item_result.item.Get() };
            match item.Type() {
                InlineItemType::kText => {
                    line_breaker.AppendCandidates(item_result, line_info, self);
                }
                InlineItemType::kControl => {
                    self.AppendTrailingSpaces(
                        if item_result.can_break_after {
                            State::kBreak
                        } else {
                            State::kMidWord
                        },
                        InlineItemTextIndex {
                            item_index: item_result.item_index,
                            text_offset: item_result.EndOffset(),
                        },
                        (self.SnappedPosition() + item_result.inline_size).ToFloat(),
                    );
                    self.SetLast(item, item_result.EndOffset());
                }
                InlineItemType::kOutOfFlowPositioned
                    if RuntimeEnabledFeatures::SkipOofItemForBreakCandidateEnabled() => {}
                _ => {
                    let new_state = if item_result.can_break_after {
                        State::kBreak
                    } else if self.state_ == State::kBreak {
                        State::kMidWord
                    } else {
                        self.state_
                    };
                    let offset = InlineItemTextIndex {
                        item_index: item_result.item_index + 1,
                        text_offset: item_result.EndOffset(),
                    };
                    let end_position = (self.SnappedPosition() + item_result.inline_size).ToFloat();
                    if item.Length() == 0 {
                        let last_candidate = self.candidates_.last().expect("candidate sentinel");
                        let end = last_candidate.end;
                        let pos_if_break = last_candidate.pos_if_break;
                        self.Append(
                            new_state,
                            offset,
                            end,
                            end_position,
                            pos_if_break,
                            0.0,
                            false,
                        );
                    } else {
                        self.AppendAtPosition(new_state, offset, end_position);
                    }
                    self.SetLast(item, item_result.EndOffset());
                }
            }
        }

        #[cfg(feature = "expensive_dchecks")]
        {
            self.CheckConsistency();
            debug_assert_eq!(self.state_, State::kBreak);
            let last_candidate = self.candidates_.last().expect("candidate sentinel");
            if let Some(index) = last_index {
                let last_result = &line_info.Results()[index];
                debug_assert!(last_candidate.offset.item_index >= last_result.item_index);
                debug_assert!(last_candidate.offset.item_index <= last_result.item_index + 1);
                debug_assert!(last_candidate.offset.text_offset >= last_result.EndOffset());
                debug_assert!(last_candidate.offset.text_offset <= line_info.EndTextOffset());
            }
        }
        true
    }

    // cpp: layoutng_inline/line_break_candidate.cc:175-183
    pub fn EnsureFirstSentinel(&mut self, first_line_info: &LineInfo) {
        debug_assert!(self.candidates_.is_empty());
        let first_item_result = first_line_info.Results().first().expect("first line item");
        self.candidates_.push(LineBreakCandidate::at_position(
            first_item_result.Start(),
            0.0,
        ));
        #[cfg(feature = "expensive_dchecks")]
        {
            self.first_offset_ = first_item_result.Start();
        }
    }

    // cpp: layoutng_inline/line_break_candidate.cc:185-202
    pub fn EnsureLastSentinel(&self, last_line_info: &LineInfo) {
        #[cfg(feature = "expensive_dchecks")]
        {
            let last_index = if RuntimeEnabledFeatures::SkipOofItemForBreakCandidateEnabled() {
                Self::LastNonOutOfFlowPositionedItemResult(last_line_info)
            } else {
                last_line_info.Results().len().checked_sub(1)
            };
            if let Some(index) = last_index {
                let last_result = &last_line_info.Results()[index];
                debug_assert!(last_result.can_break_after);
                let candidate = self.candidates_.last().expect("last candidate");
                debug_assert!(
                    candidate.offset == last_result.End()
                        || candidate.offset == last_line_info.End()
                );
            }
            debug_assert_eq!(self.state_, State::kBreak);
            self.CheckConsistency();
            debug_assert!(self.candidates_.len() >= 2);
            debug_assert_eq!(self.candidates_[0].offset, self.first_offset_);
        }
        #[cfg(not(feature = "expensive_dchecks"))]
        let _ = last_line_info;
    }

    // cpp: layoutng_inline/line_break_candidate.cc:205-219
    #[cfg(feature = "expensive_dchecks")]
    fn CheckConsistency(&self) {
        for index in 1..self.candidates_.len() {
            let candidate = &self.candidates_[index];
            debug_assert!(candidate.offset.GreaterOrEqual(&candidate.end));
            let previous = &self.candidates_[index - 1];
            debug_assert!(candidate.offset.GreaterThan(&previous.offset));
            debug_assert!(candidate.end.GreaterOrEqual(&previous.end));
            if candidate.pos_no_break.is_nan()
                || candidate.pos_no_break >= LayoutUnit::NearlyMax().ToFloat()
            {
                continue;
            }
            debug_assert!(candidate.pos_no_break >= previous.pos_no_break);
        }
    }
}

// cpp: layoutng_inline/line_break_candidate.cc:221-227
impl fmt::Display for LineBreakCandidate {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            output,
            "{}/{} {}/{} penalty={}{}",
            self.offset,
            self.end,
            self.pos_no_break,
            self.pos_if_break,
            self.penalty,
            if self.is_hyphenated {
                " (hyphenated)"
            } else {
                ""
            },
        )
    }
}
