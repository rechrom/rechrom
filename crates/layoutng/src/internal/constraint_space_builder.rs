#![allow(non_snake_case)]

use foundation::{
    kIndefiniteSize, AtomicString, IsParallelWritingMode, LayoutUnit, MakeGarbageCollected,
    MarginStrut, Member, WritingDirectionMode, WritingMode,
};
use layoutng_geometry::geometry::bfc_offset::BfcOffset;
use layoutng_geometry::geometry::box_sides::LogicalBoxSides;
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_geometry::geometry::logical_size::LogicalSize;

use std::sync::Arc;

use super::break_appeal::BreakAppeal;
use super::constraint_space::{
    AdjoiningObjectTypes, AutoSizeBehavior, BaselineAlgorithmType, ConstraintSpace,
    DecorationPercentageResolutionType, FragmentationType, LayoutResultCacheSlot,
    MathTargetStretchBlockSizes, RareData,
};
use super::exclusions::exclusion_space::ExclusionSpace;
use super::grid_layout_data::GridLayoutSubtree;
use super::line_clamp_data::{LineClampAncestorChain, LineClampData};
use super::table_constraint_space_data::TableConstraintSpaceData;

// cpp: layoutng/internal/constraint_space_builder.h:24-25
// cpp: layoutng/internal/constraint_space_builder.h:682-724
pub struct ConstraintSpaceBuilder {
    space_: ConstraintSpace,
    rare_data_: *mut RareData,
    orthogonal_fallback_inline_size_: LayoutUnit,
    is_in_parallel_flow_: bool,
    is_new_fc_: bool,
    adjust_inline_size_if_needed_: bool,
    force_orthogonal_writing_mode_root_: bool,
    #[cfg(debug_assertions)]
    pub(crate) is_custom_layout_data_set_: bool,
    #[cfg(debug_assertions)]
    is_hidden_for_paint_set_: bool,
    #[cfg(debug_assertions)]
    is_available_size_set_: bool,
    #[cfg(debug_assertions)]
    is_percentage_resolution_size_set_: bool,
    #[cfg(debug_assertions)]
    is_fragmentainer_block_size_set_: bool,
    #[cfg(debug_assertions)]
    is_fragmentainer_offset_set_: bool,
    #[cfg(debug_assertions)]
    is_block_direction_fragmentation_type_set_: bool,
    #[cfg(debug_assertions)]
    is_margin_strut_set_: bool,
    #[cfg(debug_assertions)]
    is_optimistic_bfc_block_offset_set_: bool,
    #[cfg(debug_assertions)]
    is_forced_bfc_block_offset_set_: bool,
    #[cfg(debug_assertions)]
    is_clearance_offset_set_: bool,
    #[cfg(debug_assertions)]
    is_table_cell_borders_set_: bool,
    #[cfg(debug_assertions)]
    is_table_cell_alignment_baseline_set_: bool,
    #[cfg(debug_assertions)]
    is_table_cell_column_index_set_: bool,
    #[cfg(debug_assertions)]
    is_table_cell_with_collapsed_borders_set_: bool,
    #[cfg(debug_assertions)]
    is_line_clamp_data_set_: bool,
    #[cfg(debug_assertions)]
    is_table_row_data_set_: bool,
    #[cfg(debug_assertions)]
    is_table_section_data_set_: bool,
    #[cfg(debug_assertions)]
    is_grid_layout_subtree_set_: bool,
    #[cfg(debug_assertions)]
    to_constraint_space_called_: bool,
}

impl ConstraintSpaceBuilder {
    // cpp: layoutng/internal/constraint_space_builder.h:29-42
    pub fn new(
        parent_space: &ConstraintSpace,
        writing_direction: WritingDirectionMode,
        is_new_fc: bool,
    ) -> Self {
        Self::new_with_parent_writing_mode(
            parent_space,
            parent_space.GetWritingMode(),
            writing_direction,
            is_new_fc,
            true,
            false,
        )
    }

    pub fn new_with_inline_size_adjustment(
        parent_space: &ConstraintSpace,
        writing_direction: WritingDirectionMode,
        is_new_fc: bool,
        adjust_inline_size_if_needed: bool,
    ) -> Self {
        Self::new_with_parent_writing_mode(
            parent_space,
            parent_space.GetWritingMode(),
            writing_direction,
            is_new_fc,
            adjust_inline_size_if_needed,
            false,
        )
    }

    // cpp: layoutng/internal/constraint_space_builder.h:45-65
    pub fn new_with_parent_writing_mode(
        parent_space: &ConstraintSpace,
        parent_writing_mode: WritingMode,
        writing_direction: WritingDirectionMode,
        is_new_fc: bool,
        adjust_inline_size_if_needed: bool,
        force_orthogonal_writing_mode_root: bool,
    ) -> Self {
        let mut builder = Self::new_internal(
            writing_direction,
            IsParallelWritingMode(parent_writing_mode, writing_direction.GetWritingMode()),
            is_new_fc,
            adjust_inline_size_if_needed,
            force_orthogonal_writing_mode_root,
        );
        if parent_space.ShouldPropagateChildBreakValues() {
            builder.SetShouldPropagateChildBreakValues(true);
        }
        if parent_space.ShouldRepeat() {
            builder.SetShouldRepeat(true);
        }
        builder.SetIsInsideRepeatableContent(parent_space.IsInsideRepeatableContent());
        builder.SetContainsAnnotations(parent_space.ContainsAnnotations());
        builder
    }

    // cpp: layoutng/internal/constraint_space_builder.h:68-79
    pub fn new_without_parent_space(
        parent_writing_mode: WritingMode,
        writing_direction: WritingDirectionMode,
        is_new_fc: bool,
        adjust_inline_size_if_needed: bool,
        force_orthogonal_writing_mode_root: bool,
    ) -> Self {
        Self::new_internal(
            writing_direction,
            IsParallelWritingMode(parent_writing_mode, writing_direction.GetWritingMode()),
            is_new_fc,
            adjust_inline_size_if_needed,
            force_orthogonal_writing_mode_root,
        )
    }

    // cpp: layoutng/internal/constraint_space_builder.h:667-681
    fn new_internal(
        writing_direction: WritingDirectionMode,
        is_in_parallel_flow: bool,
        is_new_fc: bool,
        adjust_inline_size_if_needed: bool,
        force_orthogonal_writing_mode_root: bool,
    ) -> Self {
        let mut space = ConstraintSpace::new(writing_direction);
        space.bitfields_.set_bit(8, is_new_fc);
        space.bitfields_.set_bit(
            9,
            !is_in_parallel_flow || force_orthogonal_writing_mode_root,
        );
        Self {
            space_: space,
            rare_data_: std::ptr::null_mut(),
            orthogonal_fallback_inline_size_: kIndefiniteSize,
            is_in_parallel_flow_: is_in_parallel_flow,
            is_new_fc_: is_new_fc,
            adjust_inline_size_if_needed_: adjust_inline_size_if_needed,
            force_orthogonal_writing_mode_root_: force_orthogonal_writing_mode_root,
            #[cfg(debug_assertions)]
            is_custom_layout_data_set_: false,
            #[cfg(debug_assertions)]
            is_hidden_for_paint_set_: false,
            #[cfg(debug_assertions)]
            is_available_size_set_: false,
            #[cfg(debug_assertions)]
            is_percentage_resolution_size_set_: false,
            #[cfg(debug_assertions)]
            is_fragmentainer_block_size_set_: false,
            #[cfg(debug_assertions)]
            is_fragmentainer_offset_set_: false,
            #[cfg(debug_assertions)]
            is_block_direction_fragmentation_type_set_: false,
            #[cfg(debug_assertions)]
            is_margin_strut_set_: false,
            #[cfg(debug_assertions)]
            is_optimistic_bfc_block_offset_set_: false,
            #[cfg(debug_assertions)]
            is_forced_bfc_block_offset_set_: false,
            #[cfg(debug_assertions)]
            is_clearance_offset_set_: false,
            #[cfg(debug_assertions)]
            is_table_cell_borders_set_: false,
            #[cfg(debug_assertions)]
            is_table_cell_alignment_baseline_set_: false,
            #[cfg(debug_assertions)]
            is_table_cell_column_index_set_: false,
            #[cfg(debug_assertions)]
            is_table_cell_with_collapsed_borders_set_: false,
            #[cfg(debug_assertions)]
            is_line_clamp_data_set_: false,
            #[cfg(debug_assertions)]
            is_table_row_data_set_: false,
            #[cfg(debug_assertions)]
            is_table_section_data_set_: false,
            #[cfg(debug_assertions)]
            is_grid_layout_subtree_set_: false,
            #[cfg(debug_assertions)]
            to_constraint_space_called_: false,
        }
    }

    // cpp: layoutng/internal/constraint_space_builder.h:685-691
    pub(crate) fn EnsureRareData(&mut self) -> *mut RareData {
        if self.rare_data_.is_null() {
            self.rare_data_ = MakeGarbageCollected(RareData::default());
            self.space_.rare_data_ = Member::from_ptr(self.rare_data_);
        }
        self.rare_data_
    }

    // cpp: layoutng/internal/constraint_space_builder.h:84-92
    pub fn AdjustInlineSizeIfNeeded(&mut self, inline_size: &mut LayoutUnit) {
        debug_assert!(!self.is_in_parallel_flow_);
        debug_assert!(self.adjust_inline_size_if_needed_);
        if *inline_size != kIndefiniteSize {
            return;
        }
        debug_assert_ne!(self.orthogonal_fallback_inline_size_, kIndefiniteSize);
        *inline_size = self.orthogonal_fallback_inline_size_;
        unsafe { &mut *self.EnsureRareData() }.uses_orthogonal_fallback_inline_size = true;
    }

    // cpp: layoutng/internal/constraint_space_builder.h:95-110
    pub fn SetAvailableSize(&mut self, mut available_size: LogicalSize) {
        #[cfg(debug_assertions)]
        {
            self.is_available_size_set_ = true;
            debug_assert!(!self.is_percentage_resolution_size_set_);
        }
        if self.is_in_parallel_flow_ {
            self.space_.available_size_ = available_size;
            self.space_.percentage_size_ = available_size;
        } else {
            if self.adjust_inline_size_if_needed_ {
                self.AdjustInlineSizeIfNeeded(&mut available_size.block_size);
            }
            let converted = LogicalSize::new(available_size.block_size, available_size.inline_size);
            self.space_.available_size_ = converted;
            self.space_.percentage_size_ = converted;
        }
    }

    // cpp: layoutng/internal/constraint_space_builder.h:113-126
    pub fn SetPercentageResolutionSize(&mut self, mut percentage_size: LogicalSize) {
        #[cfg(debug_assertions)]
        {
            self.is_percentage_resolution_size_set_ = true;
        }
        if self.is_in_parallel_flow_ {
            self.space_.percentage_size_ = percentage_size;
        } else {
            if self.adjust_inline_size_if_needed_ {
                self.AdjustInlineSizeIfNeeded(&mut percentage_size.block_size);
            }
            self.space_.percentage_size_ =
                LogicalSize::new(percentage_size.block_size, percentage_size.inline_size);
        }
    }

    // cpp: layoutng/internal/constraint_space_builder.h:132-150
    pub fn SetReplacedChildPercentageResolutionSize(&mut self, size: LogicalSize) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(self.is_percentage_resolution_size_set_);
            debug_assert!(self.is_in_parallel_flow_);
        }
        debug_assert_eq!(
            size.inline_size,
            self.space_.PercentageResolutionInlineSize()
        );
        debug_assert_ne!(size.block_size, kIndefiniteSize);
        debug_assert_ne!(size.block_size, self.space_.PercentageResolutionBlockSize());
        unsafe { &mut *self.EnsureRareData() }.replaced_child_percentage_resolution_block_size =
            size.block_size;
    }

    // cpp: layoutng/internal/constraint_space_builder.h:153-156
    pub fn SetOrthogonalFallbackInlineSize(&mut self, size: LayoutUnit) {
        self.orthogonal_fallback_inline_size_ = size;
    }

    // cpp: layoutng/internal/constraint_space_builder.h:158-162
    pub fn SetPageName(&mut self, name: &AtomicString) {
        if name.IsNull() && self.space_.rare_data_.Get().is_null() {
            return;
        }
        unsafe { &mut *self.EnsureRareData() }.page_name = name.clone();
    }

    // cpp: layoutng/internal/constraint_space_builder.h:164-171
    pub fn SetFragmentainerBlockSize(&mut self, size: LayoutUnit) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_fragmentainer_block_size_set_);
            self.is_fragmentainer_block_size_set_ = true;
        }
        if size != kIndefiniteSize {
            unsafe { &mut *self.EnsureRareData() }.fragmentainer_block_size = size;
        }
    }

    // cpp: layoutng/internal/constraint_space_builder.h:175-180
    pub fn SetFragmentainerBlockSizeFromAvailableSize(&mut self) {
        #[cfg(debug_assertions)]
        debug_assert!(self.is_available_size_set_);
        self.SetFragmentainerBlockSize(self.space_.AvailableSize().block_size);
    }

    // cpp: layoutng/internal/constraint_space_builder.h:187-200
    pub fn ReserveSpaceInFragmentainer(&mut self, space: LayoutUnit) {
        if !self.space_.HasBlockFragmentation() {
            return;
        }
        #[cfg(debug_assertions)]
        debug_assert!(self.is_fragmentainer_block_size_set_);
        let rare_data = unsafe { &mut *self.rare_data_ };
        rare_data.fragmentainer_block_size -= space;
        rare_data.fragmentainer_block_size =
            rare_data.fragmentainer_block_size.ClampNegativeToZero();
    }

    // cpp: layoutng/internal/constraint_space_builder.h:202-211
    pub fn SetFragmentainerOffset(&mut self, offset: LayoutUnit) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_fragmentainer_offset_set_);
            self.is_fragmentainer_offset_set_ = true;
        }
        if offset != LayoutUnit::default() {
            unsafe { &mut *self.EnsureRareData() }.fragmentainer_offset = offset;
        }
    }

    // cpp: layoutng/internal/constraint_space_builder.h:211-228
    pub fn SetIsAtFragmentainerStart(&mut self) {
        unsafe { &mut *self.EnsureRareData() }.is_at_fragmentainer_start = true;
    }

    pub fn SetShouldRepeat(&mut self, value: bool) {
        unsafe { &mut *self.EnsureRareData() }.should_repeat = value;
    }

    pub fn SetIsInsideRepeatableContent(&mut self, value: bool) {
        if !value && self.space_.rare_data_.Get().is_null() {
            return;
        }
        unsafe { &mut *self.EnsureRareData() }.is_inside_repeatable_content = value;
    }

    pub fn SetIsInsideBreakAvoid(&mut self, value: bool) {
        if !value && self.space_.rare_data_.Get().is_null() {
            return;
        }
        unsafe { &mut *self.EnsureRareData() }.is_inside_break_avoid = value;
    }

    // cpp: layoutng/internal/constraint_space_builder.h:230-238
    pub fn DisableFurtherFragmentation(&mut self) {
        if self.space_.HasBlockFragmentation() {
            let data = unsafe { &mut *self.rare_data_ };
            data.block_direction_fragmentation_type = FragmentationType::kFragmentNone;
            data.is_block_fragmentation_forced_off = true;
        }
    }

    pub fn DisableMonolithicOverflowPropagation(&mut self) {
        unsafe { &mut *self.EnsureRareData() }.is_monolithic_overflow_propagation_disabled = true;
    }

    // cpp: layoutng/internal/constraint_space_builder.h:240-248
    pub fn SetIsHiddenForPaint(&mut self, is_hidden_for_paint: bool) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_hidden_for_paint_set_);
            self.is_hidden_for_paint_set_ = true;
        }
        if is_hidden_for_paint {
            self.space_.bitfields_.set_bit(11, true);
        }
    }

    // cpp: layoutng/internal/constraint_space_builder.h:250-304
    pub fn SetIsFixedInlineSize(&mut self, value: bool) {
        self.space_
            .bitfields_
            .set_bit(if self.is_in_parallel_flow_ { 25 } else { 26 }, value);
    }

    pub fn SetIsFixedBlockSize(&mut self, value: bool) {
        self.space_
            .bitfields_
            .set_bit(if self.is_in_parallel_flow_ { 26 } else { 25 }, value);
    }

    pub fn SetIsInitialBlockSizeIndefinite(&mut self, value: bool) {
        if self.is_in_parallel_flow_ {
            self.space_.bitfields_.set_bit(27, value);
        }
    }

    pub fn SetInlineAutoBehavior(&mut self, value: AutoSizeBehavior) {
        self.space_.bitfields_.set_bits(
            if self.is_in_parallel_flow_ { 21 } else { 23 },
            2,
            value as u32,
        );
    }

    pub fn SetBlockAutoBehavior(&mut self, value: AutoSizeBehavior) {
        self.space_.bitfields_.set_bits(
            if self.is_in_parallel_flow_ { 23 } else { 21 },
            2,
            value as u32,
        );
    }

    pub fn SetIsPaintedAtomically(&mut self, value: bool) {
        self.space_.bitfields_.set_bit(10, value);
    }

    // cpp: layoutng/internal/constraint_space_builder.h:306-345
    pub fn SetFragmentationType(&mut self, value: FragmentationType) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_block_direction_fragmentation_type_set_);
            self.is_block_direction_fragmentation_type_set_ = true;
        }
        if value != FragmentationType::kFragmentNone {
            unsafe { &mut *self.EnsureRareData() }.block_direction_fragmentation_type = value;
        }
    }

    pub fn SetPaperEdgeAdjacentSides(&mut self, sides: LogicalBoxSides) {
        let rare_data = unsafe { &mut *self.EnsureRareData() };
        rare_data.is_adjacent_to_paper_edge_inline_start = sides.inline_start;
        rare_data.is_adjacent_to_paper_edge_inline_end = sides.inline_end;
        rare_data.is_adjacent_to_paper_edge_block_start = sides.block_start;
        rare_data.is_adjacent_to_paper_edge_block_end = sides.block_end;
    }

    pub fn SetSafePrintableInset(&mut self, inset: LayoutUnit) {
        unsafe { &mut *self.EnsureRareData() }.safe_printable_inset = inset;
    }

    pub fn SetRequiresContentBeforeBreaking(&mut self, value: bool) {
        if value || !self.rare_data_.is_null() {
            unsafe { &mut *self.EnsureRareData() }.requires_content_before_breaking = value;
        }
    }

    pub fn SetIsInsideBalancedColumns(&mut self) {
        unsafe { &mut *self.EnsureRareData() }.is_inside_balanced_columns = true;
    }

    pub fn SetShouldIgnoreForcedBreaks(&mut self) {
        unsafe { &mut *self.EnsureRareData() }.should_ignore_forced_breaks = true;
    }

    pub fn SetIsInColumnBfc(&mut self) {
        unsafe { &mut *self.EnsureRareData() }.is_in_column_bfc = true;
    }

    pub fn SetIsPastBreak(&mut self) {
        unsafe { &mut *self.EnsureRareData() }.is_past_break = true;
    }

    // cpp: layoutng/internal/constraint_space_builder.h:347-435
    pub fn SetMinBlockSizeShouldEncompassIntrinsicSize(&mut self) {
        unsafe { &mut *self.EnsureRareData() }.min_block_size_should_encompass_intrinsic_size =
            true;
    }

    pub fn SetMinBreakAppeal(&mut self, value: BreakAppeal) {
        if value != BreakAppeal::kBreakAppealLastResort || !self.rare_data_.is_null() {
            unsafe { &mut *self.EnsureRareData() }.min_break_appeal = value;
        }
    }

    pub fn SetShouldPropagateChildBreakValues(&mut self, value: bool) {
        if value || !self.rare_data_.is_null() {
            unsafe { &mut *self.EnsureRareData() }.propagate_child_break_values = value;
        }
    }

    pub fn SetIsTableCell(&mut self, _is_table_cell: bool) {
        unsafe { &mut *self.EnsureRareData() }.SetIsTableCell();
    }

    pub fn SetIsRestrictedBlockSizeTableCell(&mut self, value: bool) {
        debug_assert!(self.space_.IsTableCell());
        if value || !self.rare_data_.is_null() {
            unsafe { &mut *self.EnsureRareData() }.is_restricted_block_size_table_cell = value;
        }
    }

    pub fn SetHideTableCellIfEmpty(&mut self, value: bool) {
        if value || !self.rare_data_.is_null() {
            unsafe { &mut *self.EnsureRareData() }.hide_table_cell_if_empty = value;
        }
    }

    pub fn SetIsAnonymous(&mut self, value: bool) {
        self.space_.bitfields_.set_bit(7, value);
    }

    pub fn SetUseFirstLineStyle(&mut self, value: bool) {
        self.space_.bitfields_.set_bit(12, value);
    }

    pub fn SetAdjoiningObjectTypes(&mut self, value: AdjoiningObjectTypes) {
        if !self.is_new_fc_ {
            self.space_.bitfields_.set_bits(0, 3, value as u32);
        }
    }

    pub fn SetAncestorHasClearancePastAdjoiningFloats(&mut self) {
        self.space_.bitfields_.set_bit(13, true);
    }

    pub fn SetBaselineAlgorithmType(&mut self, value: BaselineAlgorithmType) {
        self.space_.bitfields_.set_bit(19, value as u8 != 0);
    }

    pub fn SetCacheSlot(&mut self, value: LayoutResultCacheSlot) {
        self.space_.bitfields_.set_bit(20, value as u8 != 0);
    }

    pub fn SetContainsAnnotations(&mut self, value: bool) {
        self.space_.bitfields_.set_bit(14, value);
    }

    pub fn SetBlockStartAnnotationSpace(&mut self, space: LayoutUnit) {
        if space != LayoutUnit::default() {
            unsafe { &mut *self.EnsureRareData() }.SetBlockStartAnnotationSpace(space);
        }
    }

    pub fn SetPreviousSiblingBlockEndAnnotationSpace(&mut self, space: LayoutUnit) {
        if space != LayoutUnit::default() {
            unsafe { &mut *self.EnsureRareData() }.SetPreviousSiblingBlockEndAnnotationSpace(space);
        } else {
            debug_assert_eq!(space, self.space_.PreviousSiblingBlockEndAnnotationSpace());
        }
    }

    // cpp: layoutng/internal/constraint_space_builder.h:437-472
    pub fn SetIgnoreMarginsForStretch(
        &mut self,
        parent_direction: WritingDirectionMode,
        sides: LogicalBoxSides,
    ) {
        if sides.IsEmpty() {
            return;
        }
        let converted = sides
            .ToPhysical(parent_direction)
            .ToLogical(self.space_.GetWritingDirection());
        self.space_.bitfields_.set_bit(15, converted.inline_start);
        self.space_.bitfields_.set_bit(16, converted.inline_end);
        self.space_.bitfields_.set_bit(17, converted.block_start);
        self.space_.bitfields_.set_bit(18, converted.block_end);
    }

    pub fn SetMarginStrut(&mut self, margin_strut: &MarginStrut) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_margin_strut_set_);
            self.is_margin_strut_set_ = true;
        }
        if !self.is_new_fc_ && *margin_strut != MarginStrut::default() {
            unsafe { &mut *self.EnsureRareData() }.SetMarginStrut(margin_strut);
        }
    }

    pub fn SetBfcOffset(&mut self, offset: BfcOffset) {
        if !self.is_new_fc_ {
            self.space_.bfc_offset_ = offset;
        }
    }

    pub fn SetOptimisticBfcBlockOffset(&mut self, offset: LayoutUnit) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_optimistic_bfc_block_offset_set_);
            self.is_optimistic_bfc_block_offset_set_ = true;
        }
        if !self.is_new_fc_ {
            unsafe { &mut *self.EnsureRareData() }.SetOptimisticBfcBlockOffset(offset);
        }
    }

    pub fn SetForcedBfcBlockOffset(&mut self, offset: LayoutUnit) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_forced_bfc_block_offset_set_);
            self.is_forced_bfc_block_offset_set_ = true;
        }
        debug_assert!(!self.is_new_fc_);
        unsafe { &mut *self.EnsureRareData() }.SetForcedBfcBlockOffset(offset);
    }

    // cpp: layoutng/internal/constraint_space_builder.h:474-476
    pub fn ExpectedBfcBlockOffset(&self) -> LayoutUnit {
        self.space_.ExpectedBfcBlockOffset()
    }

    // cpp: layoutng/internal/constraint_space_builder.h:478-485
    pub fn SetClearanceOffset(&mut self, offset: LayoutUnit) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_clearance_offset_set_);
            self.is_clearance_offset_set_ = true;
        }
        if !self.is_new_fc_ && offset != LayoutUnit::Min() {
            unsafe { &mut *self.EnsureRareData() }.SetClearanceOffset(offset);
        }
    }

    // cpp: layoutng/internal/constraint_space_builder.h:487-535
    pub fn SetTableCellBorders(
        &mut self,
        borders: &BoxStrut,
        cell_direction: WritingDirectionMode,
        table_direction: WritingDirectionMode,
    ) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_table_cell_borders_set_);
            self.is_table_cell_borders_set_ = true;
        }
        if *borders != BoxStrut::default() {
            let converted = borders
                .ConvertToPhysical(table_direction)
                .ConvertToLogical(cell_direction);
            unsafe { &mut *self.EnsureRareData() }.SetTableCellBorders(&converted);
        }
    }

    pub fn SetTableCellAlignmentBaseline(&mut self, baseline: Option<LayoutUnit>) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_table_cell_alignment_baseline_set_);
            self.is_table_cell_alignment_baseline_set_ = true;
        }
        if self.is_in_parallel_flow_ {
            if let Some(baseline) = baseline {
                unsafe { &mut *self.EnsureRareData() }.SetTableCellAlignmentBaseline(baseline);
            }
        }
    }

    pub fn SetTableCellColumnIndex(&mut self, index: u32) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_table_cell_column_index_set_);
            self.is_table_cell_column_index_set_ = true;
        }
        unsafe { &mut *self.EnsureRareData() }.SetTableCellColumnIndex(index);
    }

    pub fn SetIsTableCellWithCollapsedBorders(&mut self, value: bool) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_table_cell_with_collapsed_borders_set_);
            self.is_table_cell_with_collapsed_borders_set_ = true;
        }
        if value {
            unsafe { &mut *self.EnsureRareData() }.SetIsTableCellWithCollapsedBorders(value);
        }
    }

    pub fn SetIsTableCellChild(&mut self, value: bool) {
        self.space_.bitfields_.set_bit(28, value);
    }

    pub fn SetIsRestrictedBlockSizeTableCellChild(&mut self) {
        self.space_.bitfields_.set_bit(29, true);
    }

    // cpp: layoutng/internal/constraint_space_builder.h:537-581
    pub fn SetExclusionSpace(&mut self, exclusion_space: &ExclusionSpace) {
        if !self.is_new_fc_ {
            self.space_.exclusion_space_ = exclusion_space.clone();
        }
    }

    pub fn SetTableRowData(&mut self, table_data: Arc<TableConstraintSpaceData>, row_index: u32) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_table_row_data_set_);
            self.is_table_row_data_set_ = true;
        }
        unsafe { &mut *self.EnsureRareData() }.SetTableRowData(table_data, row_index);
    }

    pub fn SetTableSectionData(
        &mut self,
        table_data: Arc<TableConstraintSpaceData>,
        section_index: u32,
    ) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_table_section_data_set_);
            self.is_table_section_data_set_ = true;
        }
        unsafe { &mut *self.EnsureRareData() }.SetTableSectionData(table_data, section_index);
    }

    pub fn SetLineClampData(&mut self, data: LineClampData) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_line_clamp_data_set_);
            self.is_line_clamp_data_set_ = true;
        }
        debug_assert!(!self.is_new_fc_);
        if data.IsLineClampContext() {
            unsafe { &mut *self.EnsureRareData() }.SetLineClampData(data);
        }
    }

    pub fn SetLineClampAncestorChain(&mut self, data: *const LineClampAncestorChain) {
        if !data.is_null() {
            unsafe { &mut *self.EnsureRareData() }.SetLineClampAncestorChain(data);
        }
    }

    // cpp: layoutng/internal/constraint_space_builder.h:583-645
    pub fn SetShouldTextBoxTrimNodeStart(&mut self, value: bool) {
        if value || !self.rare_data_.is_null() {
            unsafe { &mut *self.EnsureRareData() }.should_text_box_trim_node_start = value;
        }
    }

    pub fn SetShouldTextBoxTrimNodeEnd(&mut self, value: bool) {
        if value || !self.rare_data_.is_null() {
            unsafe { &mut *self.EnsureRareData() }.should_text_box_trim_node_end = value;
        }
    }

    pub fn SetShouldTextBoxTrimFragmentainerStart(&mut self, value: bool) {
        if value || !self.rare_data_.is_null() {
            unsafe { &mut *self.EnsureRareData() }.should_text_box_trim_fragmentainer_start = value;
        }
    }

    pub fn SetShouldTextBoxTrimFragmentainerEnd(&mut self, value: bool) {
        if value || !self.rare_data_.is_null() {
            unsafe { &mut *self.EnsureRareData() }.should_text_box_trim_fragmentainer_end = value;
        }
    }

    pub fn SetShouldTextBoxTrimInsideWhenLineClamp(&mut self, value: bool) {
        if value || !self.rare_data_.is_null() {
            unsafe { &mut *self.EnsureRareData() }.should_text_box_trim_inside_when_line_clamp =
                value;
        }
    }

    pub fn SetShouldForceTextBoxTrimEnd(&mut self) {
        unsafe { &mut *self.EnsureRareData() }.should_force_text_box_trim_end = true;
    }

    pub fn SetShouldForceMarginTrimEnd(&mut self) {
        unsafe { &mut *self.EnsureRareData() }.should_force_margin_trim_end = true;
    }

    pub fn SetDecorationPercentageResolutionType(
        &mut self,
        value: DecorationPercentageResolutionType,
    ) {
        unsafe { &mut *self.EnsureRareData() }.decoration_percentage_resolution_type = value;
    }

    pub fn SetIsPushedByFloats(&mut self) {
        unsafe { &mut *self.EnsureRareData() }.is_pushed_by_floats = true;
    }

    pub fn SetTargetStretchInlineSize(&mut self, size: LayoutUnit) {
        debug_assert!(size >= LayoutUnit::default());
        unsafe { &mut *self.EnsureRareData() }.SetTargetStretchInlineSize(size);
    }

    pub fn SetTargetStretchBlockSizes(&mut self, sizes: MathTargetStretchBlockSizes) {
        debug_assert!(sizes.ascent >= LayoutUnit::default());
        debug_assert!(sizes.descent >= LayoutUnit::default());
        unsafe { &mut *self.EnsureRareData() }.SetTargetStretchBlockSizes(sizes);
    }

    // cpp: layoutng/internal/constraint_space_builder.h:638-645
    pub fn SetGridLayoutSubtree(&mut self, subtree: *const GridLayoutSubtree) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_grid_layout_subtree_set_);
            self.is_grid_layout_subtree_set_ = true;
        }
        unsafe { &mut *self.EnsureRareData() }.SetGridLayoutSubtree(subtree);
    }

    // cpp: layoutng/internal/constraint_space_builder.h:647-664
    pub fn ToConstraintSpace(mut self) -> ConstraintSpace {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.to_constraint_space_called_);
            self.to_constraint_space_called_ = true;
        }
        debug_assert!(!self.is_new_fc_ || self.space_.bitfields_.0 & 0b111 == 0);
        debug_assert_eq!(
            self.space_.bitfields_.is_orthogonal_writing_mode_root(),
            !self.is_in_parallel_flow_ || self.force_orthogonal_writing_mode_root_
        );
        debug_assert!(!self.force_orthogonal_writing_mode_root_ || self.is_in_parallel_flow_);
        self.space_
    }
}
