#![allow(non_snake_case)]

use foundation::{
    EBlockEllipsis, LayoutUnit, MakeGarbageCollected, MarginStrut, Member, RuntimeEnabledFeatures,
    Visitor,
};

use super::layout_object::LayoutObject;
use super::min_max_sizes::MinMaxSizes;

// cpp: layoutng/internal/line_clamp_data.h:30-53
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    kDisabled,
    kClampByLines,
    kClampAfterLayoutObject,
    kMeasureLinesUntilBfcOffset,
    kCountLines,
    kClampByLinesWithBfcOffset,
}

// cpp: layoutng/internal/line_clamp_data.h:21-24
// cpp: layoutng/internal/line_clamp_data.h:113-144
#[repr(C)]
pub struct LineClampData {
    pub lines_until_clamp: i32,
    pub clamp_bfc_offset: LayoutUnit,
    // UntracedMember is used only for identity comparison in the source.
    pub clamp_after_layout_object: *const LayoutObject,
    pub state: State,
    pub block_ellipsis: EBlockEllipsis,
}

impl Default for LineClampData {
    fn default() -> Self {
        Self {
            lines_until_clamp: 0,
            clamp_bfc_offset: LayoutUnit::default(),
            clamp_after_layout_object: std::ptr::null(),
            state: State::kDisabled,
            block_ellipsis: EBlockEllipsis::kEllipsis,
        }
    }
}

impl Clone for LineClampData {
    // cpp: layoutng/internal/line_clamp_data.h:26-26
    // cpp: layoutng/internal/line_clamp_data.cc:29-48
    fn clone(&self) -> Self {
        let mut copy = Self {
            state: self.state,
            ..Self::default()
        };
        match self.state {
            State::kDisabled => {}
            State::kClampByLines | State::kCountLines => {
                copy.lines_until_clamp = self.lines_until_clamp;
            }
            State::kClampAfterLayoutObject => {
                copy.clamp_after_layout_object = self.clamp_after_layout_object;
            }
            State::kMeasureLinesUntilBfcOffset | State::kClampByLinesWithBfcOffset => {
                copy.lines_until_clamp = self.lines_until_clamp;
                copy.clamp_bfc_offset = self.clamp_bfc_offset;
            }
        }
        copy.block_ellipsis = self.block_ellipsis;
        copy
    }
}

impl LineClampData {
    // cpp: layoutng/internal/line_clamp_data.h:54-54
    pub fn IsLineClampContext(&self) -> bool {
        self.state != State::kDisabled
    }

    // cpp: layoutng/internal/line_clamp_data.h:57-60
    pub fn IsClampByLines(&self) -> bool {
        matches!(
            self.state,
            State::kClampByLines | State::kClampByLinesWithBfcOffset
        )
    }

    // cpp: layoutng/internal/line_clamp_data.h:63-66
    pub fn IsMeasureUntilBfcOffset(&self) -> bool {
        matches!(
            self.state,
            State::kMeasureLinesUntilBfcOffset | State::kClampByLinesWithBfcOffset
        )
    }

    // cpp: layoutng/internal/line_clamp_data.h:69-72
    pub fn IsCountLines(&self) -> bool {
        matches!(
            self.state,
            State::kMeasureLinesUntilBfcOffset | State::kCountLines
        )
    }

    // cpp: layoutng/internal/line_clamp_data.h:77-82
    pub fn LinesUntilClamp(&self, show_measured_lines: bool) -> Option<i32> {
        if self.IsClampByLines() || (show_measured_lines && self.IsCountLines()) {
            Some(self.lines_until_clamp)
        } else {
            None
        }
    }

    // cpp: layoutng/internal/line_clamp_data.h:84-86
    pub fn IsAtClampPoint(&self) -> bool {
        self.IsClampByLines() && self.lines_until_clamp == 1
    }

    // cpp: layoutng/internal/line_clamp_data.h:88-90
    pub fn IsPastClampPoint(&self) -> bool {
        self.IsClampByLines() && self.lines_until_clamp <= 0
    }

    // cpp: layoutng/internal/line_clamp_data.h:92-92
    // cpp: layoutng/internal/line_clamp_data.cc:12-14
    pub fn ShouldHideForPaint(&self) -> bool {
        RuntimeEnabledFeatures::CSSLineClampEnabled() && self.IsPastClampPoint()
    }

    // cpp: layoutng/internal/line_clamp_data.h:28-28
    // cpp: layoutng/internal/line_clamp_data.cc:50-74
    pub fn Assign(&mut self, other: &Self) -> &mut Self {
        if self.state == State::kClampAfterLayoutObject
            && other.state != State::kClampAfterLayoutObject
        {
            self.clamp_after_layout_object = std::ptr::null();
        }
        self.state = other.state;
        match self.state {
            State::kDisabled => {}
            State::kClampByLines | State::kCountLines => {
                self.lines_until_clamp = other.lines_until_clamp;
            }
            State::kClampAfterLayoutObject => {
                self.clamp_after_layout_object = other.clamp_after_layout_object;
            }
            State::kMeasureLinesUntilBfcOffset | State::kClampByLinesWithBfcOffset => {
                self.lines_until_clamp = other.lines_until_clamp;
                self.clamp_bfc_offset = other.clamp_bfc_offset;
            }
        }
        self.block_ellipsis = other.block_ellipsis;
        self
    }
}

impl PartialEq for LineClampData {
    // cpp: layoutng/internal/line_clamp_data.h:94-111
    fn eq(&self, other: &Self) -> bool {
        if self.state != other.state || self.block_ellipsis != other.block_ellipsis {
            return false;
        }
        match self.state {
            State::kClampByLines | State::kCountLines => {
                self.lines_until_clamp == other.lines_until_clamp
            }
            State::kClampAfterLayoutObject => {
                self.clamp_after_layout_object == other.clamp_after_layout_object
            }
            State::kMeasureLinesUntilBfcOffset | State::kClampByLinesWithBfcOffset => {
                self.lines_until_clamp == other.lines_until_clamp
                    && self.clamp_bfc_offset == other.clamp_bfc_offset
            }
            State::kDisabled => true,
        }
    }
}

impl Eq for LineClampData {}

// cpp: layoutng/internal/line_clamp_data.cc:18-25
#[repr(C)]
struct SameSizeAsLineClampData {
    lines_until_clamp: i32,
    clamp_bfc_offset: LayoutUnit,
    clamp_after_layout_object: *const LayoutObject,
    states: [u8; 2],
}
const _: () =
    assert!(std::mem::size_of::<LineClampData>() == std::mem::size_of::<SameSizeAsLineClampData>());

// cpp: layoutng/internal/line_clamp_data.h:160-223
pub struct LineClampAncestorChain {
    bfc_offset_: Option<LayoutUnit>,
    end_border_padding_: LayoutUnit,
    end_margin_: LayoutUnit,
    block_min_max_sizes_: MinMaxSizes,
    parent_: Member<LineClampAncestorChain>,
}

// cpp: layoutng/internal/line_clamp_data.h:207-211
impl PartialEq for LineClampAncestorChain {
    fn eq(&self, other: &Self) -> bool {
        self.bfc_offset_ == other.bfc_offset_
            && self.end_border_padding_ == other.end_border_padding_
            && self.end_margin_ == other.end_margin_
            && self.parent_.Get() == other.parent_.Get()
    }
}

impl Eq for LineClampAncestorChain {}

impl LineClampAncestorChain {
    // cpp: layoutng/internal/line_clamp_data.h:163-167
    pub fn new_root(end_border_padding: LayoutUnit) -> Self {
        Self {
            bfc_offset_: Some(LayoutUnit::default()),
            end_border_padding_: end_border_padding,
            end_margin_: LayoutUnit::default(),
            block_min_max_sizes_: MinMaxSizes {
                min_size: LayoutUnit::default(),
                max_size: LayoutUnit::Max(),
            },
            parent_: Member::default(),
        }
    }

    // cpp: layoutng/internal/line_clamp_data.h:168-179
    pub fn new_child(
        bfc_offset: Option<LayoutUnit>,
        end_border_padding: LayoutUnit,
        end_margin: LayoutUnit,
        block_size_constraints: MinMaxSizes,
        parent: *const Self,
    ) -> Self {
        debug_assert!(!parent.is_null());
        Self {
            bfc_offset_: bfc_offset,
            end_border_padding_: end_border_padding,
            end_margin_: end_margin,
            block_min_max_sizes_: block_size_constraints,
            parent_: Member::from_ptr(parent as *mut Self),
        }
    }

    // cpp: layoutng/internal/line_clamp_data.h:181-181
    pub fn HasBfcOffset(&self) -> bool {
        self.bfc_offset_.is_some()
    }

    // cpp: layoutng/internal/line_clamp_data.h:183-193
    pub fn WithResolvedBfcOffset(&self, new_bfc_offset: LayoutUnit) -> *const Self {
        if let Some(existing) = self.bfc_offset_ {
            debug_assert_eq!(existing, new_bfc_offset);
            self
        } else {
            MakeGarbageCollected(Self::new_child(
                Some(new_bfc_offset),
                self.end_border_padding_,
                self.end_margin_,
                self.block_min_max_sizes_,
                self.parent_.Get(),
            ))
        }
    }

    // cpp: layoutng/internal/line_clamp_data.h:198-203
    pub fn FinalLineClampBlockSize(
        &self,
        inflow_block_offset: LayoutUnit,
        margin_strut: MarginStrut,
    ) -> LayoutUnit {
        debug_assert!(self.bfc_offset_.is_some());
        self.InnerFinalLineClampBlockSize(
            self.bfc_offset_
                .expect("line-clamp BFC offset must be resolved"),
            inflow_block_offset,
            margin_strut,
        )
    }

    // cpp: layoutng/internal/line_clamp_data.h:205-205
    // cpp: layoutng/internal/line_clamp_data.cc:116-118
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.parent_);
    }

    // cpp: layoutng/internal/line_clamp_data.h:207-211
    pub fn Equals(&self, other: &Self) -> bool {
        self.bfc_offset_ == other.bfc_offset_
            && self.end_border_padding_ == other.end_border_padding_
            && self.end_margin_ == other.end_margin_
            && self.parent_.Get() == other.parent_.Get()
    }

    // cpp: layoutng/internal/line_clamp_data.h:214-216
    // cpp: layoutng/internal/line_clamp_data.cc:76-114
    fn InnerFinalLineClampBlockSize(
        &self,
        bfc_offset_override: LayoutUnit,
        inflow_block_offset: LayoutUnit,
        mut margin_strut: MarginStrut,
    ) -> LayoutUnit {
        let mut block_size = inflow_block_offset;
        let parent = self.parent_.Get();
        if self.end_border_padding_ != LayoutUnit::default() || parent.is_null() {
            block_size += margin_strut.Sum() + self.end_border_padding_;
            margin_strut = MarginStrut::default();
        }

        if !parent.is_null() {
            let clamped_size = self.block_min_max_sizes_.ClampSizeToMinAndMax(block_size);
            if clamped_size != block_size || self.block_min_max_sizes_.max_size == block_size {
                margin_strut = MarginStrut::default();
            }
            margin_strut.Append(&self.end_margin_, false);

            let bfc_offset = self.bfc_offset_.unwrap_or(bfc_offset_override);
            let parent = unsafe { &*parent };
            let parent_bfc_offset = parent.bfc_offset_.unwrap_or(bfc_offset_override);
            parent.InnerFinalLineClampBlockSize(
                bfc_offset,
                bfc_offset + clamped_size - parent_bfc_offset,
                margin_strut,
            )
        } else {
            debug_assert!(self.bfc_offset_.is_some());
            debug_assert_eq!(self.bfc_offset_, Some(LayoutUnit::default()));
            debug_assert!(margin_strut.IsEmpty());
            block_size
        }
    }
}
