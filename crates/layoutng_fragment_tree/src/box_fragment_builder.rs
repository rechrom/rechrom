use std::ops::{Deref, DerefMut};

use foundation::{
    kIndefiniteSize, AtomicString, DynamicTo, EBoxDecorationBreak, EBreakBetween, EPosition,
    ETextBoxTrim, HeapVector, LayoutUnit, MakeGarbageCollected, MarginStrut, Member,
    PhysicalToLogical, RuntimeEnabledFeatures, ScopedRefPtr, To, ToLineWritingMode,
    WritingDirectionMode, WritingMode,
};
use layoutng::internal::block_node::BlockNode;
use layoutng::internal::break_appeal::{kBreakAppealPerfect, BreakAppeal};
use layoutng::internal::column_spanner_path::ColumnSpannerPath;
use layoutng::internal::constraint_space::ConstraintSpace;
use layoutng::internal::custom_layout_payload::SerializedScriptValue;
use layoutng::internal::devtools_flex_info::DevtoolsFlexInfo;
use layoutng::internal::disable_layout_side_effects_scope::DisableLayoutSideEffectsScope;
use layoutng::internal::early_break::EarlyBreak;
use layoutng::internal::fragmentation_utils::{
    BlockSizeForFragmentation, CalculateBreakAppealInsideDefault, FragmentainerSpaceLeft,
    IsAvoidBreakValue, IsBreakInside, JoinFragmentainerBreakValues, PageNameForChildFragment,
};
use layoutng::internal::frame_set_layout_data::FrameSetLayoutData;
use layoutng::internal::gap::gap_geometry::GapGeometry;
use layoutng::internal::grid_layout_data::GridLayoutData;
use layoutng::internal::inline_item_text_index::InlineItemTextIndex;
use layoutng::internal::inline_node::InlineNode;
use layoutng::internal::layout_input_node::LayoutInputNode;
use layoutng::internal::layout_node_metadata::Node;
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_pass_scope::LayoutPassScope;
use layoutng::internal::length_utils::CalculateChildAvailableSize;
use layoutng::internal::length_utils::ComputeMarginsForInlineSize;
use layoutng::internal::mathml_paint_info::MathMLPaintInfo;
use layoutng::internal::oof_positioned_node::{LogicalOofNodeForFragmentation, OofInlineContainer};
use layoutng::internal::relative_utils::ComputeRelativeOffsetForBoxFragment;
use layoutng::internal::table_borders::TableBorders;
use layoutng::internal::table_fragment_data::{
    CollapsedTableBordersGeometry, TableColumnGeometries,
};
use layoutng_geometry::geometry::box_sides::{LineLogicalBoxSides, LogicalBoxSides};
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_geometry::geometry::fragment_geometry::FragmentGeometry;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::logical_size::LogicalSize;
use layoutng_geometry::geometry::overflow_clip_axes::{kOverflowClipX, kOverflowClipY};
use layoutng_style::style::computed_style::ComputedStyle;

use crate::block_break_token::BlockBreakToken;
use crate::break_token::BreakToken;
use crate::break_token_algorithm_data::BreakTokenAlgorithmData;
use crate::fragment_builder::FragmentBuilder;
use crate::inline_break_token::{InlineBreakToken, InlineBreakTokenFlag};
use crate::layout_result::{EStatus, LayoutResult};
use crate::logical_box_fragment::LogicalBoxFragment;
use crate::logical_fragment::LogicalFragment;
use crate::logical_fragment_link::LogicalFragmentLink;
use crate::physical_box_fragment::PhysicalBoxFragment;
use crate::physical_fragment::{BoxType, PhysicalFragment};
use crate::physical_line_box_fragment::PhysicalLineBoxFragment;

// C++ derives from FragmentBuilder; the first field preserves that upcast.
// The remaining fields follow the source declaration and use owned Rust
// values where C++'s stack lifetime or unique_ptr guarantees ownership.
// cpp: layoutng_fragment_tree/box_fragment_builder.h:42-69
// cpp: layoutng_fragment_tree/box_fragment_builder.h:812-908
#[repr(C)]
pub struct BoxFragmentBuilder {
    pub(crate) base_: FragmentBuilder,
    pub(crate) initial_fragment_geometry_: *const FragmentGeometry,
    pub(crate) border_padding_: BoxStrut,
    pub(crate) border_scrollbar_padding_: BoxStrut,
    pub(crate) original_border_scrollbar_padding_block_start_: LayoutUnit,
    pub(crate) child_available_size_: LogicalSize,
    pub(crate) intrinsic_block_size_: LayoutUnit,
    pub(crate) inflow_bounds_: Option<LogicalRect>,
    pub(crate) is_fieldset_container_: bool,
    pub(crate) is_table_part_: bool,
    pub(crate) is_initial_block_size_indefinite_: bool,
    pub(crate) is_inline_formatting_context_: bool,
    pub(crate) is_known_to_fit_in_fragmentainer_: bool,
    pub(crate) is_block_size_for_fragmentation_clamped_: bool,
    pub(crate) is_monolithic_: bool,
    pub(crate) is_first_for_node_: bool,
    pub(crate) should_clone_box_end_decorations_: bool,
    pub(crate) should_prevent_break_before_block_end_decorations_: bool,
    pub(crate) did_break_self_: bool,
    pub(crate) has_inflow_child_break_inside_: bool,
    pub(crate) has_forced_break_: bool,
    pub(crate) has_seen_all_children_: bool,
    pub(crate) has_subsequent_children_: bool,
    pub(crate) has_column_spanner_: bool,
    pub(crate) is_empty_spanner_parent_: bool,
    pub(crate) should_force_same_fragmentation_flow_: bool,
    pub(crate) is_math_fraction_: bool,
    pub(crate) is_math_operator_: bool,
    pub(crate) is_at_block_end_: bool,
    pub(crate) is_truncated_by_fragmentation_line: bool,
    pub(crate) use_last_baseline_for_inline_baseline_: bool,
    pub(crate) has_moved_children_: bool,
    pub(crate) should_text_box_trim_node_start_: bool,
    pub(crate) should_text_box_trim_node_end_: bool,
    pub(crate) should_text_box_trim_fragmentainer_start_: bool,
    pub(crate) should_text_box_trim_fragmentainer_end_: bool,
    pub(crate) block_size_for_fragmentation_: LayoutUnit,
    pub(crate) consumed_block_size_: LayoutUnit,
    pub(crate) monolithic_overflow_: LayoutUnit,
    pub(crate) sequence_number_: u32,
    pub(crate) initial_break_before_: Option<EBreakBetween>,
    pub(crate) previous_break_after_: EBreakBetween,
    pub(crate) page_name_: AtomicString,
    pub(crate) first_baseline_: Option<LayoutUnit>,
    pub(crate) last_baseline_: Option<LayoutUnit>,
    pub(crate) math_italic_correction_: LayoutUnit,
    pub(crate) gap_geometry_: *const GapGeometry,
    pub(crate) table_grid_rect_: Option<LogicalRect>,
    pub(crate) table_column_geometries_: TableColumnGeometries,
    pub(crate) table_collapsed_borders_: *const TableBorders,
    pub(crate) table_collapsed_borders_geometry_: Option<Box<CollapsedTableBordersGeometry>>,
    pub(crate) table_column_count_: Option<u32>,
    pub(crate) table_cell_column_index_: Option<u32>,
    pub(crate) table_section_start_row_index_: u32,
    pub(crate) table_section_row_offsets_: Vec<LayoutUnit>,
    pub(crate) column_spanner_path_: *const ColumnSpannerPath,
    pub(crate) break_token_data_: *mut BreakTokenAlgorithmData,
    pub(crate) grid_layout_data_: *const GridLayoutData,
    pub(crate) flex_layout_data_: *const DevtoolsFlexInfo,
    pub(crate) frame_set_layout_data_: Option<Box<FrameSetLayoutData>>,
    pub(crate) reading_flow_nodes_: HeapVector<Member<Node>>,
    pub(crate) sides_to_include_: LineLogicalBoxSides,
    pub(crate) custom_layout_data_: ScopedRefPtr<SerializedScriptValue>,
    pub(crate) mathml_paint_info_: *const MathMLPaintInfo,
    #[cfg(debug_assertions)]
    pub(crate) block_size_is_for_all_fragments_: bool,
    #[cfg(debug_assertions)]
    pub(crate) needs_inflow_bounds_explicitly_set_: bool,
    #[cfg(debug_assertions)]
    pub(crate) needs_may_have_descendant_above_block_start_explicitly_set_: bool,
    #[cfg(debug_assertions)]
    pub(crate) is_inflow_bounds_explicitly_set_: bool,
}

impl Deref for BoxFragmentBuilder {
    type Target = FragmentBuilder;

    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}

impl DerefMut for BoxFragmentBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_
    }
}

#[allow(non_snake_case)]
impl BoxFragmentBuilder {
    // cpp: layoutng_out_of_flow/out_of_flow_fragment_builder.cc:25-53
    pub fn AdjustFragmentainerDescendant(
        &self,
        descendant: &mut LogicalOofNodeForFragmentation,
        only_fixedpos_containing_block: bool,
    ) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        let previous_token = self.PreviousBreakToken();
        let previous_consumed_block_size = if previous_token.is_null() {
            LayoutUnit::default()
        } else {
            unsafe { &*previous_token }.ConsumedBlockSize()
        };
        if !only_fixedpos_containing_block && descendant.containing_block.Fragment().is_null() {
            descendant
                .containing_block
                .IncreaseBlockOffset(-previous_consumed_block_size);
            descendant.IncreaseStaticPositionOffset(LogicalOffset::new(
                LayoutUnit::default(),
                previous_consumed_block_size,
            ));
        }
        if descendant.fixedpos_containing_block.Fragment().is_null()
            && (self.node_.IsFixedContainer()
                || !descendant.fixedpos_inline_container.Container().is_null())
        {
            descendant
                .fixedpos_containing_block
                .IncreaseBlockOffset(-previous_consumed_block_size);
        }
    }

    // cpp: layoutng_out_of_flow/out_of_flow_fragment_builder.cc:55-65
    pub fn AdjustFixedposContainingBlockForFragmentainerDescendants(&mut self) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        if !self.HasOutOfFlowFragmentainerDescendants() {
            return;
        }
        let mut descendants = std::mem::take(&mut self.oof_positioned_fragmentainer_descendants_);
        for descendant in &mut descendants {
            self.AdjustFragmentainerDescendant(descendant, true);
        }
        self.oof_positioned_fragmentainer_descendants_ = descendants;
    }

    // cpp: layoutng_out_of_flow/out_of_flow_fragment_builder.cc:67-88
    pub fn AdjustFixedposContainingBlockForInnerMulticols(&mut self) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        let previous_token = self.PreviousBreakToken();
        if !self.HasMulticolsWithPendingOOFs() || previous_token.is_null() {
            return;
        }
        let previous_consumed_block_size = unsafe { &*previous_token }.ConsumedBlockSize();
        let is_fixed_container = self.node_.IsFixedContainer();
        for (_, multicol) in self.multicols_with_pending_oofs_.iter() {
            let value = unsafe { &mut *multicol.Get() };
            if value.fixedpos_containing_block.Fragment().is_null()
                && (is_fixed_container || !value.fixedpos_inline_container.Container().is_null())
            {
                value
                    .fixedpos_containing_block
                    .IncreaseBlockOffset(-previous_consumed_block_size);
                value.multicol_offset.block_offset += previous_consumed_block_size;
            }
        }
    }

    pub(crate) fn base_mut(&mut self) -> &mut FragmentBuilder {
        &mut self.base_
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:47-59
    pub fn new(
        node: LayoutInputNode,
        style: *const ComputedStyle,
        space: &ConstraintSpace,
        writing_direction: WritingDirectionMode,
        previous_break_token: *const BlockBreakToken,
    ) -> Self {
        let is_inline_formatting_context = node.IsInline();
        let base = FragmentBuilder::new(
            node,
            style,
            space,
            writing_direction,
            previous_break_token as *const BreakToken,
        );
        Self::from_base(base, is_inline_formatting_context)
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:61-69
    pub fn new_for_layout_object(
        layout_object: *mut LayoutObject,
        style: *const ComputedStyle,
        space: &ConstraintSpace,
        writing_direction: WritingDirectionMode,
    ) -> Self {
        let mut base = FragmentBuilder::new(
            LayoutInputNode::null(),
            style,
            space,
            writing_direction,
            std::ptr::null(),
        );
        base.layout_object_ = layout_object;
        Self::from_base(base, true)
    }

    fn from_base(base: FragmentBuilder, is_inline_formatting_context: bool) -> Self {
        Self {
            base_: base,
            initial_fragment_geometry_: std::ptr::null(),
            border_padding_: BoxStrut::default(),
            border_scrollbar_padding_: BoxStrut::default(),
            original_border_scrollbar_padding_block_start_: LayoutUnit::default(),
            child_available_size_: LogicalSize::default(),
            intrinsic_block_size_: LayoutUnit::default(),
            inflow_bounds_: None,
            is_fieldset_container_: false,
            is_table_part_: false,
            is_initial_block_size_indefinite_: false,
            is_inline_formatting_context_: is_inline_formatting_context,
            is_known_to_fit_in_fragmentainer_: false,
            is_block_size_for_fragmentation_clamped_: false,
            is_monolithic_: true,
            is_first_for_node_: true,
            should_clone_box_end_decorations_: false,
            should_prevent_break_before_block_end_decorations_: false,
            did_break_self_: false,
            has_inflow_child_break_inside_: false,
            has_forced_break_: false,
            has_seen_all_children_: false,
            has_subsequent_children_: false,
            has_column_spanner_: false,
            is_empty_spanner_parent_: false,
            should_force_same_fragmentation_flow_: false,
            is_math_fraction_: false,
            is_math_operator_: false,
            is_at_block_end_: false,
            is_truncated_by_fragmentation_line: false,
            use_last_baseline_for_inline_baseline_: false,
            has_moved_children_: false,
            should_text_box_trim_node_start_: false,
            should_text_box_trim_node_end_: false,
            should_text_box_trim_fragmentainer_start_: false,
            should_text_box_trim_fragmentainer_end_: false,
            block_size_for_fragmentation_: LayoutUnit::default(),
            consumed_block_size_: LayoutUnit::default(),
            monolithic_overflow_: LayoutUnit::default(),
            sequence_number_: 0,
            initial_break_before_: None,
            previous_break_after_: EBreakBetween::kAuto,
            page_name_: AtomicString::default(),
            first_baseline_: None,
            last_baseline_: None,
            math_italic_correction_: LayoutUnit::default(),
            gap_geometry_: std::ptr::null(),
            table_grid_rect_: None,
            table_column_geometries_: TableColumnGeometries::default(),
            table_collapsed_borders_: std::ptr::null(),
            table_collapsed_borders_geometry_: None,
            table_column_count_: None,
            table_cell_column_index_: None,
            table_section_start_row_index_: 0,
            table_section_row_offsets_: Vec::new(),
            column_spanner_path_: std::ptr::null(),
            break_token_data_: std::ptr::null_mut(),
            grid_layout_data_: std::ptr::null(),
            flex_layout_data_: std::ptr::null(),
            frame_set_layout_data_: None,
            reading_flow_nodes_: HeapVector::default(),
            sides_to_include_: LineLogicalBoxSides::default(),
            custom_layout_data_: ScopedRefPtr::default(),
            mathml_paint_info_: std::ptr::null(),
            #[cfg(debug_assertions)]
            block_size_is_for_all_fragments_: false,
            #[cfg(debug_assertions)]
            needs_inflow_bounds_explicitly_set_: false,
            #[cfg(debug_assertions)]
            needs_may_have_descendant_above_block_start_explicitly_set_: false,
            #[cfg(debug_assertions)]
            is_inflow_bounds_explicitly_set_: false,
        }
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:148-150
    pub fn PreviousBreakToken(&self) -> *const BlockBreakToken {
        let token = self.previous_break_token_;
        if token.is_null() {
            return std::ptr::null();
        }
        assert!(unsafe { &*token }.IsBlockType());
        token as *const BlockBreakToken
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:164-180
    pub fn SetFragmentsTotalBlockSize(&mut self, block_size: LayoutUnit) {
        #[cfg(debug_assertions)]
        {
            self.block_size_is_for_all_fragments_ = true;
        }
        self.size_.block_size = block_size;
    }

    pub fn FragmentsTotalBlockSize(&self) -> LayoutUnit {
        #[cfg(debug_assertions)]
        {
            if self.space_.HasBlockFragmentation() {
                debug_assert!(self.block_size_is_for_all_fragments_);
            }
            debug_assert_ne!(self.size_.block_size, kIndefiniteSize);
        }
        self.size_.block_size
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:183-224
    pub fn SetFragmentBlockSize(&mut self, block_size: LayoutUnit) {
        #[cfg(debug_assertions)]
        {
            self.block_size_is_for_all_fragments_ = false;
        }
        self.size_.block_size = block_size;
    }

    pub fn FragmentBlockSize(&self) -> LayoutUnit {
        #[cfg(debug_assertions)]
        {
            debug_assert!(
                !self.block_size_is_for_all_fragments_
                    || !self.space_.HasBlockFragmentation()
                    || self.space_.IsInitialColumnBalancingPass()
            );
            debug_assert_ne!(self.size_.block_size, kIndefiniteSize);
        }
        self.size_.block_size
    }

    pub fn FragmentInlineSize(&self) -> LayoutUnit {
        debug_assert_ne!(self.size_.inline_size, kIndefiniteSize);
        self.size_.inline_size
    }

    pub fn SizeForAnchorQueries(&self) -> LogicalSize {
        let mut size = LogicalSize::new(self.InlineSize(), LayoutUnit::default());
        if self.HasBlockSize() {
            size.block_size = self.FragmentBlockSize();
        }
        size
    }

    pub fn SetIntrinsicBlockSize(&mut self, size: LayoutUnit) {
        self.intrinsic_block_size_ = size;
    }

    pub fn IntrinsicBlockSize(&self) -> LayoutUnit {
        self.intrinsic_block_size_
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:71-96
    pub fn SetInitialFragmentGeometry(&mut self, geometry: &FragmentGeometry) {
        self.initial_fragment_geometry_ = geometry;
        self.size_ = geometry.border_box_size;
        self.is_initial_block_size_indefinite_ = self.size_.block_size == kIndefiniteSize;
        self.border_padding_ = geometry.border + geometry.padding;
        self.border_scrollbar_padding_ = BoxStrut::default();
        if self.node_.IsNull() || (!self.node_.IsTableSection() && !self.node_.IsTableRow()) {
            self.border_scrollbar_padding_ = geometry.border + geometry.scrollbar;
            if self.node_.IsNull() || !self.node_.IsFieldsetContainer() {
                self.border_scrollbar_padding_ += geometry.padding;
            }
        }
        self.original_border_scrollbar_padding_block_start_ =
            self.border_scrollbar_padding_.block_start;
        if !self.node_.IsNull() {
            let node = BlockNode::from(self.node_.clone());
            let insets = self.border_padding_ + geometry.scrollbar;
            self.child_available_size_ =
                CalculateChildAvailableSize(&self.space_, &node, self.size_, &insets);
        }
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:98-101
    pub fn InitialFragmentGeometry(&self) -> &FragmentGeometry {
        debug_assert!(!self.initial_fragment_geometry_.is_null());
        unsafe { &*self.initial_fragment_geometry_ }
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:225-270
    pub fn Borders(&self) -> &BoxStrut {
        debug_assert_ne!(
            self.GetBoxType(),
            crate::physical_fragment::BoxType::kInlineBox
        );
        &self.InitialFragmentGeometry().border
    }

    pub fn Scrollbar(&self) -> &BoxStrut {
        &self.InitialFragmentGeometry().scrollbar
    }

    pub fn Padding(&self) -> &BoxStrut {
        &self.InitialFragmentGeometry().padding
    }

    pub fn InitialBorderBoxSize(&self) -> &LogicalSize {
        &self.InitialFragmentGeometry().border_box_size
    }

    pub fn ExcludedSidesTruncated(&self, strut: BoxStrut) -> BoxStrut {
        BoxStrut::new(
            strut.inline_start,
            strut.inline_end,
            if self.sides_to_include_.block_start {
                strut.block_start
            } else {
                LayoutUnit::default()
            },
            if self.sides_to_include_.block_end {
                strut.block_end
            } else {
                LayoutUnit::default()
            },
        )
    }

    pub fn ApplicableBorders(&self) -> BoxStrut {
        self.ExcludedSidesTruncated(self.InitialFragmentGeometry().border)
    }

    pub fn ApplicableScrollbar(&self) -> BoxStrut {
        self.ExcludedSidesTruncated(*self.Scrollbar())
    }

    pub fn ApplicablePadding(&self) -> BoxStrut {
        self.ExcludedSidesTruncated(*self.Padding())
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:282-326
    pub fn BorderPadding(&self) -> &BoxStrut {
        debug_assert!(!self.initial_fragment_geometry_.is_null());
        &self.border_padding_
    }

    pub fn BorderScrollbarPadding(&self) -> &BoxStrut {
        debug_assert!(!self.initial_fragment_geometry_.is_null());
        &self.border_scrollbar_padding_
    }

    pub fn OriginalBorderScrollbarPaddingBlockStart(&self) -> LayoutUnit {
        self.original_border_scrollbar_padding_block_start_
    }

    pub fn ClearBorderScrollbarPaddingBlockStart(&mut self) {
        self.border_scrollbar_padding_.block_start = LayoutUnit::default();
    }

    pub fn ClearBorderScrollbarPaddingBlockEnd(&mut self) {
        self.border_scrollbar_padding_.block_end = LayoutUnit::default();
    }

    pub fn ChildAvailableSize(&self) -> &LogicalSize {
        debug_assert!(!self.initial_fragment_geometry_.is_null());
        &self.child_available_size_
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:320-324
    pub fn Node(&self) -> BlockNode {
        debug_assert!(!self.node_.IsNull());
        BlockNode::from(self.node_.clone())
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:107-147
    pub fn ShouldTextBoxTrimStart(&self) -> bool {
        self.should_text_box_trim_node_start_ || self.should_text_box_trim_fragmentainer_start_
    }

    pub fn ShouldTextBoxTrimEnd(&self) -> bool {
        self.should_text_box_trim_node_end_ || self.should_text_box_trim_fragmentainer_end_
    }

    pub fn ShouldTextBoxTrim(&self) -> bool {
        self.ShouldTextBoxTrimStart() || self.ShouldTextBoxTrimEnd()
    }

    pub fn ClearShouldTextBoxTrimEnd(&mut self) {
        self.should_text_box_trim_node_end_ = false;
        self.should_text_box_trim_fragmentainer_end_ = false;
    }

    pub fn ClearShouldTextBoxTrimNodeStart(&mut self) {
        self.should_text_box_trim_node_start_ = false;
    }

    pub fn ShouldTextBoxTrimNodeStart(&self) -> bool {
        self.should_text_box_trim_node_start_
    }

    pub fn SetShouldTextBoxTrimNodeEnd(&mut self, value: bool) {
        self.should_text_box_trim_node_end_ = value;
    }

    pub fn ShouldTextBoxTrimNodeEnd(&self) -> bool {
        self.should_text_box_trim_node_end_
    }

    pub fn ClearShouldTextBoxTrimFragmentainerStart(&mut self) {
        self.should_text_box_trim_fragmentainer_start_ = false;
    }

    pub fn ShouldTextBoxTrimFragmentainerStart(&self) -> bool {
        self.should_text_box_trim_fragmentainer_start_
    }

    pub fn ShouldTextBoxTrimFragmentainerEnd(&self) -> bool {
        self.should_text_box_trim_fragmentainer_end_
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.cc:29-88
    pub fn SetInitialTextBoxTrim(&mut self) {
        self.should_text_box_trim_node_start_ = self.space_.ShouldTextBoxTrimNodeStart();
        self.should_text_box_trim_node_end_ = self.space_.ShouldTextBoxTrimNodeEnd();
        self.should_text_box_trim_fragmentainer_start_ =
            self.space_.ShouldTextBoxTrimFragmentainerStart();
        self.should_text_box_trim_fragmentainer_end_ =
            self.space_.ShouldTextBoxTrimFragmentainerEnd();
        if self.should_text_box_trim_node_start_
            && self.border_padding_.block_start != LayoutUnit::default()
        {
            self.should_text_box_trim_node_start_ = false;
        }
        if self.should_text_box_trim_node_end_
            && self.border_padding_.block_end != LayoutUnit::default()
        {
            self.should_text_box_trim_node_end_ = false;
        }
        let style = unsafe { &*(self.Style() as *const ComputedStyle) };
        if style.TextBoxTrim() == ETextBoxTrim::kNone {
            return;
        }
        if !self.space_.IsAnonymous() {
            self.should_text_box_trim_node_start_ |= style.ShouldTextBoxTrimStart();
            self.should_text_box_trim_node_end_ |= style.ShouldTextBoxTrimEnd();
        }
        if !self.space_.HasBlockFragmentation() || self.space_.IsPaginated() {
            self.should_text_box_trim_fragmentainer_start_ = false;
            self.should_text_box_trim_fragmentainer_end_ = false;
        } else {
            if IsBreakInside(self.PreviousBreakToken()) {
                self.should_text_box_trim_fragmentainer_start_ |=
                    self.should_text_box_trim_node_start_;
            } else {
                self.should_text_box_trim_fragmentainer_start_ = false;
            }
            self.should_text_box_trim_fragmentainer_end_ |= self.should_text_box_trim_node_end_;
            if !self.space_.IsAnonymous()
                && style.BoxDecorationBreak() != EBoxDecorationBreak::kClone
            {
                self.should_text_box_trim_fragmentainer_start_ &= !style.ShouldTextBoxTrimStart();
                self.should_text_box_trim_fragmentainer_end_ &= !style.ShouldTextBoxTrimEnd();
            }
        }
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:311
    // cpp: layoutng_fragment_tree/box_fragment_builder.cc:90-101
    pub fn UpdateBorderPaddingForClonedBoxDecorations(&mut self) {
        let token = self.PreviousBreakToken();
        if !IsBreakInside(token) {
            return;
        }
        let count = unsafe { &*token }.SequenceNumber() as i32 + 2;
        self.border_padding_.block_start = self.border_padding_.block_start * count;
        self.border_padding_.block_end = self.border_padding_.block_end * count;
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:723-732
    pub fn GetBreakTokenData(&self) -> *mut BreakTokenAlgorithmData {
        self.break_token_data_
    }

    pub fn SetBreakTokenData(&mut self, data: *mut BreakTokenAlgorithmData) {
        self.break_token_data_ = data;
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:761-774
    pub fn SetHasForcedBreak(&mut self) {
        self.has_forced_break_ = true;
        self.minimal_space_shortage_ = kIndefiniteSize;
    }

    pub fn HasForcedBreak(&self) -> bool {
        self.has_forced_break_
    }

    pub fn LastChildBreakToken(&self) -> *const BreakToken {
        debug_assert!(!self.child_break_tokens_.is_empty());
        self.child_break_tokens_
            .last()
            .expect("child break token")
            .Get()
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:784-803
    pub fn ShouldCalculateScrollableOverflow(&self) -> bool {
        !self.node_.IsNull()
            && !self.node_.IsReplaced()
            && (!self.node_.IsPaginatedRoot() || self.IsFragmentainerBoxType())
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:508-530
    pub fn HasInflowChildBreakInside(&self) -> bool {
        self.has_inflow_child_break_inside_
    }

    pub fn SetInitialBreakBefore(&mut self, break_before: EBreakBetween) {
        self.initial_break_before_ = Some(break_before);
    }

    pub fn SetInitialBreakBeforeIfNeeded(&mut self, break_before: EBreakBetween) {
        if self.initial_break_before_.is_none() {
            self.initial_break_before_ = Some(break_before);
        }
    }

    pub fn PreviousBreakAfter(&self) -> EBreakBetween {
        self.previous_break_after_
    }

    pub fn SetPreviousBreakAfter(&mut self, break_after: EBreakBetween) {
        self.previous_break_after_ = break_after;
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:526-531
    pub fn SetPageNameIfNeeded(&mut self, name: AtomicString) {
        if self.page_name_.IsNull() {
            self.page_name_ = name;
        }
    }

    pub fn PageName(&self) -> &AtomicString {
        &self.page_name_
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:585-607
    pub fn SetHasColumnSpanner(&mut self) {
        self.has_column_spanner_ = true;
        self.has_inflow_child_break_inside_ = true;
    }

    pub fn SetColumnSpannerPath(&mut self, path: &ColumnSpannerPath) {
        self.column_spanner_path_ = path;
        self.SetHasColumnSpanner();
    }

    pub fn FoundColumnSpanner(&self) -> bool {
        debug_assert!(self.has_column_spanner_ || self.column_spanner_path_.is_null());
        self.has_column_spanner_
    }

    pub fn SetIsEmptySpannerParent(&mut self, value: bool) {
        debug_assert!(self.FoundColumnSpanner());
        self.is_empty_spanner_parent_ = value;
    }

    pub fn IsEmptySpannerParent(&self) -> bool {
        self.is_empty_spanner_parent_
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:540-578
    pub fn SetLineCount(&mut self, count: i32) {
        self.line_count_ = count;
    }

    pub fn LineCount(&self) -> i32 {
        self.line_count_
    }

    pub fn SetHasSeenAllChildren(&mut self) {
        self.has_seen_all_children_ = true;
    }

    pub fn HasSeenAllChildren(&self) -> bool {
        self.has_seen_all_children_
    }

    pub fn SetHasSubsequentChildren(&mut self) {
        self.has_subsequent_children_ = true;
    }

    pub fn SetIsAtBlockEnd(&mut self) {
        self.is_at_block_end_ = true;
    }

    pub fn IsAtBlockEnd(&self) -> bool {
        self.is_at_block_end_
    }

    pub fn SetIsTruncatedByFragmentationLine(&mut self) {
        self.is_truncated_by_fragmentation_line = true;
    }

    pub fn SetInflowBounds(&mut self, bounds: LogicalRect) {
        debug_assert_ne!(self.GetBoxType(), BoxType::kInlineBox);
        debug_assert!(self.Node().IsScrollContainer());
        #[cfg(debug_assertions)]
        {
            self.is_inflow_bounds_explicitly_set_ = true;
        }
        self.inflow_bounds_ = Some(bounds);
    }

    pub fn InflowBounds(&self) -> &Option<LogicalRect> {
        &self.inflow_bounds_
    }

    pub fn SetEarlyBreak(&mut self, breakpoint: *const EarlyBreak) {
        self.early_break_ = breakpoint;
    }

    pub fn HasEarlyBreak(&self) -> bool {
        !self.early_break_.is_null()
    }

    pub fn GetEarlyBreak(&self) -> &EarlyBreak {
        debug_assert!(!self.early_break_.is_null());
        unsafe { &*self.early_break_ }
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:605-746
    pub fn SetShouldForceSameFragmentationFlow(&mut self) {
        self.should_force_same_fragmentation_flow_ = true;
    }

    pub fn ShouldForceSameFragmentationFlow(&self) -> bool {
        self.should_force_same_fragmentation_flow_
    }

    pub fn SetIsFieldsetContainer(&mut self) {
        self.is_fieldset_container_ = true;
    }

    pub fn SetIsTablePart(&mut self) {
        self.is_table_part_ = true;
    }

    pub fn SetIsInlineFormattingContext(&mut self, value: bool) {
        self.is_inline_formatting_context_ = value;
    }

    pub fn SetIsMathMLFraction(&mut self) {
        self.is_math_fraction_ = true;
    }

    pub fn SetIsMathMLOperator(&mut self) {
        self.is_math_operator_ = true;
    }

    pub fn SetMathMLPaintInfo(&mut self, info: *const MathMLPaintInfo) {
        self.mathml_paint_info_ = info;
    }

    pub fn SetSidesToInclude(&mut self, sides: LineLogicalBoxSides) {
        self.sides_to_include_ = sides;
    }

    pub fn SetLogicalSidesToInclude(&mut self, sides: LogicalBoxSides) {
        self.sides_to_include_ = LineLogicalBoxSides::from_logical(sides, self.Direction());
    }

    pub fn SetCustomLayoutData(&mut self, data: ScopedRefPtr<SerializedScriptValue>) {
        self.custom_layout_data_ = data;
    }

    pub fn SetFirstBaseline(&mut self, baseline: LayoutUnit) {
        self.first_baseline_ = Some(baseline);
    }

    pub fn FirstBaseline(&self) -> Option<LayoutUnit> {
        self.first_baseline_
    }

    pub fn SetLastBaseline(&mut self, baseline: LayoutUnit) {
        self.last_baseline_ = Some(baseline);
    }

    pub fn LastBaseline(&self) -> Option<LayoutUnit> {
        self.last_baseline_
    }

    pub fn SetBaselines(&mut self, baseline: LayoutUnit) {
        self.first_baseline_ = Some(baseline);
        self.last_baseline_ = Some(baseline);
    }

    pub fn ClearBaselines(&mut self) {
        self.first_baseline_ = None;
        self.last_baseline_ = None;
    }

    pub fn SetUseLastBaselineForInlineBaseline(&mut self) {
        self.use_last_baseline_for_inline_baseline_ = true;
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.cc:809-811
    pub fn SetGapGeometry(&mut self, geometry: *const GapGeometry) {
        self.gap_geometry_ = geometry;
    }

    pub fn GetGapGeometry(&self) -> *const GapGeometry {
        self.gap_geometry_
    }

    pub fn SetTableGridRect(&mut self, rect: LogicalRect) {
        self.table_grid_rect_ = Some(rect);
    }

    pub fn SetTableColumnGeometries(&mut self, geometries: TableColumnGeometries) {
        self.table_column_geometries_ = geometries;
    }

    pub fn SetTableCollapsedBorders(&mut self, borders: &TableBorders) {
        self.table_collapsed_borders_ = borders;
    }

    pub fn SetTableCollapsedBordersGeometry(
        &mut self,
        geometry: Box<CollapsedTableBordersGeometry>,
    ) {
        self.table_collapsed_borders_geometry_ = Some(geometry);
    }

    pub fn SetTableColumnCount(&mut self, count: u32) {
        self.table_column_count_ = Some(count);
    }

    pub fn SetTableCellColumnIndex(&mut self, index: u32) {
        self.table_cell_column_index_ = Some(index);
    }

    pub fn SetTableSectionCollapsedBordersGeometry(
        &mut self,
        start_row_index: u32,
        row_offsets: Vec<LayoutUnit>,
    ) {
        self.table_section_start_row_index_ = start_row_index;
        self.table_section_row_offsets_ = row_offsets;
    }

    pub fn SetGridLayoutData(&mut self, data: *const GridLayoutData) {
        self.grid_layout_data_ = data;
    }

    pub fn SetFlexLayoutData(&mut self, data: *const DevtoolsFlexInfo) {
        self.flex_layout_data_ = data;
    }

    pub fn TransferFrameSetLayoutData(&mut self, data: Box<FrameSetLayoutData>) {
        self.frame_set_layout_data_ = Some(data);
    }

    pub fn SetReadingFlowNodes(&mut self, nodes: HeapVector<Member<Node>>) {
        self.reading_flow_nodes_ = nodes;
    }

    pub fn GetGridLayoutData(&self) -> *const GridLayoutData {
        self.grid_layout_data_
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:753-755
    pub fn SetMathItalicCorrection(&mut self, italic_correction: LayoutUnit) {
        self.math_italic_correction_ = italic_correction;
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:615-629
    pub fn ToBoxFragment(&mut self) -> *const LayoutResult {
        debug_assert_ne!(self.GetBoxType(), BoxType::kInlineBox);
        self.ToBoxFragmentWithWritingMode(self.GetWritingMode())
    }

    pub fn ToInlineBoxFragment(&mut self) -> *const LayoutResult {
        debug_assert_eq!(self.GetBoxType(), BoxType::kInlineBox);
        self.ToBoxFragmentWithWritingMode(ToLineWritingMode(self.GetWritingMode()))
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:388-445
    pub fn SetIsKnownToFitInFragmentainer(&mut self, value: bool) {
        self.is_known_to_fit_in_fragmentainer_ = value;
    }

    pub fn IsKnownToFitInFragmentainer(&self) -> bool {
        self.is_known_to_fit_in_fragmentainer_
    }

    pub fn SetIsBlockSizeForFragmentationClamped(&mut self) {
        self.is_block_size_for_fragmentation_clamped_ = true;
    }

    pub fn MustStayInCurrentFragmentainer(&self) -> bool {
        self.is_known_to_fit_in_fragmentainer_ && self.is_first_for_node_
    }

    pub fn SetIsFirstForNode(&mut self, is_first: bool) {
        self.is_first_for_node_ = is_first;
    }

    pub fn ShouldCloneBoxEndDecorations(&self) -> bool {
        self.should_clone_box_end_decorations_
    }

    pub fn SetShouldCloneBoxEndDecorations(&mut self, value: bool) {
        self.should_clone_box_end_decorations_ = value;
    }

    pub fn SetShouldPreventBreakBeforeBlockEndDecorations(&mut self, value: bool) {
        self.should_prevent_break_before_block_end_decorations_ = value;
    }

    pub fn ShouldPreventBreakBeforeBlockEndDecorations(&self) -> bool {
        self.should_prevent_break_before_block_end_decorations_
    }

    pub fn SetIsMonolithic(&mut self, value: bool) {
        self.is_monolithic_ = value;
    }

    pub fn SetConsumedBlockSize(&mut self, size: LayoutUnit) {
        self.consumed_block_size_ = size;
    }

    pub fn ReserveSpaceForMonolithicOverflow(&mut self, overflow: LayoutUnit) {
        debug_assert!(self.space_.IsPaginated());
        self.monolithic_overflow_ = self.monolithic_overflow_.max(overflow);
    }

    pub fn SetSequenceNumber(&mut self, sequence_number: u32) {
        self.sequence_number_ = sequence_number;
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:448-506
    pub fn PresetNextBreakToken(&mut self, token: *const BreakToken) {
        debug_assert!(!self.did_break_self_);
        debug_assert!(self.child_break_tokens_.is_empty());
        self.break_token_ = token;
    }

    pub fn DidBreakSelf(&self) -> bool {
        self.did_break_self_
    }

    pub fn SetDidBreakSelf(&mut self) {
        self.did_break_self_ = true;
    }

    pub fn HasInsertedChildBreak(&self) -> bool {
        for child_token in self.child_break_tokens_.iter() {
            let block = DynamicTo::<BlockBreakToken>(child_token.Get());
            if block.is_null() || !unsafe { &*block }.IsRepeated() {
                return true;
            }
        }
        false
    }

    pub fn ShouldBreakInside(&self) -> bool {
        self.HasInsertedChildBreak()
            || !self.last_inline_break_token_.is_null()
            || self.monolithic_overflow_ != LayoutUnit::default()
            || self.has_subsequent_children_
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:533-539
    // cpp: layoutng_fragment_tree/box_fragment_builder.cc:103-122
    pub fn LayoutResultForPropagation<'a>(
        &'a self,
        layout_result: &'a LayoutResult,
    ) -> &'a LayoutResult {
        if layout_result.Status() != EStatus::kSuccess {
            return layout_result;
        }
        let fragment = layout_result.GetPhysicalFragment();
        if fragment.IsBox() {
            return layout_result;
        }
        let line = DynamicTo::<PhysicalLineBoxFragment>(fragment as *const _);
        if line.is_null() || !unsafe { &*line }.IsBlockInInline() || self.items_builder_.is_null() {
            return layout_result;
        }
        let line_items = unsafe { &*self.items_builder_ }.GetLogicalLineItems(unsafe { &*line });
        let block_result = line_items.BlockInInlineLayoutResult();
        debug_assert!(!block_result.is_null());
        unsafe { &*block_result }
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.cc:123-180
    pub fn AddBreakBeforeChild(
        &mut self,
        child: LayoutInputNode,
        appeal: Option<BreakAppeal>,
        is_forced_break: bool,
        oof_start_offset: LogicalOffset,
    ) {
        debug_assert!(self.break_token_.is_null());
        if is_forced_break {
            self.SetHasForcedBreak();
            debug_assert!(appeal.is_none() || appeal == Some(kBreakAppealPerfect));
        } else if let Some(appeal) = appeal {
            self.ClampBreakAppeal(appeal);
        }
        debug_assert!(self.GetConstraintSpace().HasBlockFragmentation());
        if !self.has_inflow_child_break_inside_ {
            self.has_inflow_child_break_inside_ = !child.IsFloatingOrOutOfFlowPositioned();
        }
        if child.IsInline() {
            if self.last_inline_break_token_.is_null() {
                let previous = self.PreviousBreakToken();
                if !previous.is_null() {
                    let child_tokens = unsafe { &*previous }.ChildBreakTokens();
                    if let Some(last) = child_tokens.last() {
                        self.last_inline_break_token_ = DynamicTo::<InlineBreakToken>(last.Get());
                        if !self.last_inline_break_token_.is_null() {
                            return;
                        }
                    }
                }
                self.last_inline_break_token_ = InlineBreakToken::CreateWithoutRareData(
                    InlineNode::from(child),
                    std::ptr::null(),
                    &InlineItemTextIndex::default(),
                    InlineBreakTokenFlag::kDefault as u32,
                );
            }
            return;
        }
        let token = BlockBreakToken::CreateBreakBefore(child, is_forced_break, oof_start_offset);
        self.child_break_tokens_
            .push(Member::from_ptr(token as *mut BreakToken));
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:348-378
    // cpp: layoutng_fragment_tree/box_fragment_builder.cc:182-242
    pub fn AddResult(
        &mut self,
        child_layout_result: &LayoutResult,
        offset: LogicalOffset,
        margins: Option<BoxStrut>,
        relative_offset: Option<LogicalOffset>,
        inline_container: *const OofInlineContainer<LogicalOffset>,
    ) {
        let fragment = child_layout_result.GetPhysicalFragment();
        let mut result_for_propagation = child_layout_result as *const LayoutResult;
        let items_builder = self.ItemsBuilder();
        if !fragment.IsBox() && !items_builder.is_null() {
            let line = DynamicTo::<PhysicalLineBoxFragment>(fragment as *const _);
            if !line.is_null() {
                let line = unsafe { &*line };
                if line.IsBlockInInline() && self.GetConstraintSpace().HasBlockFragmentation() {
                    let line_items = unsafe { &*items_builder }.GetLogicalLineItems(line);
                    result_for_propagation = line_items.BlockInInlineLayoutResult();
                    debug_assert!(!result_for_propagation.is_null());
                }
                unsafe { &mut *items_builder }.AddLine(line, offset);
            }
        }
        let end_margin_strut = child_layout_result.EndMarginStrut();
        debug_assert!(!fragment.IsFormattingContextRoot() || end_margin_strut.IsEmpty());
        self.AddChild(
            fragment,
            offset,
            Some(&end_margin_strut),
            child_layout_result.IsSelfCollapsing(),
            relative_offset,
            inline_container,
        );
        if let Some(margins) = margins {
            let box_fragment = unsafe { &*To::<PhysicalBoxFragment>(fragment as *const _) };
            if !margins.IsEmpty() || !box_fragment.Margins().IsZero() {
                box_fragment
                    .GetMutableForContainerLayout()
                    .SetMargins(margins.ConvertToPhysical(self.GetWritingDirection()));
            }
        }
        let result_for_propagation = unsafe { &*result_for_propagation };
        if self.GetConstraintSpace().HasBlockFragmentation() {
            self.PropagateBreakInfo(result_for_propagation, offset);
        }
        if self.GetConstraintSpace().ShouldPropagateChildBreakValues() {
            self.PropagateChildBreakValues(result_for_propagation);
        }
        self.PropagateFromLayoutResult(result_for_propagation);
    }

    pub fn AddResultAtOffset(&mut self, child_layout_result: &LayoutResult, offset: LogicalOffset) {
        self.AddResult(child_layout_result, offset, None, None, std::ptr::null());
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.cc:382-396
    pub fn AddBreakToken(&mut self, token: *const BreakToken, is_in_parallel_flow: bool) {
        debug_assert!(self.break_token_.is_null());
        debug_assert!(!token.is_null());
        self.child_break_tokens_
            .push(Member::from_ptr(token as *mut BreakToken));
        self.has_inflow_child_break_inside_ |= !is_in_parallel_flow;
    }

    pub fn JoinedBreakBetweenValue(&self, break_before: EBreakBetween) -> EBreakBetween {
        JoinFragmentainerBreakValues(self.previous_break_after_, break_before)
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.cc:244-381
    pub fn AddChild(
        &mut self,
        child: &PhysicalFragment,
        child_offset: LogicalOffset,
        margin_strut: Option<&MarginStrut>,
        is_self_collapsing: bool,
        mut relative_offset: Option<LogicalOffset>,
        inline_container: *const OofInlineContainer<LogicalOffset>,
    ) {
        #[cfg(debug_assertions)]
        {
            self.needs_inflow_bounds_explicitly_set_ = relative_offset.is_some();
            self.needs_may_have_descendant_above_block_start_explicitly_set_ =
                relative_offset.is_some();
        }
        if relative_offset.is_none() {
            relative_offset = Some(LogicalOffset::default());
            if self.GetBoxType() != BoxType::kInlineBox {
                if child.IsLineBox() {
                    if child.MayHaveDescendantAboveBlockStart() {
                        self.may_have_descendant_above_block_start_ = true;
                    }
                } else if child.IsCSSBox() {
                    let box_child = unsafe { &*To::<PhysicalBoxFragment>(child as *const _) };
                    if box_child.Style().GetPosition() == EPosition::kRelative {
                        relative_offset = Some(ComputeRelativeOffsetForBoxFragment(
                            box_child,
                            self.GetWritingDirection(),
                            &self.child_available_size_,
                        ));
                    }
                    if (child_offset.block_offset < LayoutUnit::default()
                        && !box_child.IsOutOfFlowPositioned())
                        || (!box_child.IsFormattingContextRoot()
                            && box_child.MayHaveDescendantAboveBlockStart())
                    {
                        self.may_have_descendant_above_block_start_ = true;
                    }
                }

                let node = BlockNode::from(self.node_.clone());
                if node.IsScrollContainer()
                    && !self.IsFragmentainerBoxType()
                    && !child.IsOutOfFlowPositioned()
                {
                    let mut margins = BoxStrut::default();
                    if child.IsCSSBox() {
                        margins = ComputeMarginsForInlineSize(
                            child.Style(),
                            self.child_available_size_.inline_size,
                            self.GetWritingDirection(),
                        );
                    }
                    if let Some(margin_strut) = margin_strut {
                        let mut end_margin_strut = margin_strut.clone();
                        end_margin_strut.Append(&margins.block_end, false);
                        margins.block_end = if is_self_collapsing {
                            end_margin_strut.Sum() - margin_strut.Sum()
                        } else {
                            end_margin_strut.Sum()
                        };
                    }
                    let fragment = LogicalFragment::new(self.GetWritingDirection(), child);
                    let mut bounds = LogicalRect::new(child_offset, fragment.Size());
                    if !margins.IsEmpty() {
                        let has_top_overflow = node.HasTopOverflow();
                        let has_left_overflow = node.HasLeftOverflow();
                        let converter = PhysicalToLogical::new(
                            self.GetWritingDirection(),
                            has_top_overflow,
                            !has_left_overflow,
                            !has_top_overflow,
                            has_left_overflow,
                        );
                        if converter.InlineStart() {
                            margins.inline_end = margins.inline_end.ClampNegativeToZero();
                            margins.inline_start = margins.inline_start.max(-fragment.InlineSize());
                        } else {
                            margins.inline_start = margins.inline_start.ClampNegativeToZero();
                            margins.inline_end = margins.inline_end.max(-fragment.InlineSize());
                        }
                        if converter.BlockStart() {
                            margins.block_end = margins.block_end.ClampNegativeToZero();
                            margins.block_start = margins.block_start.max(-fragment.BlockSize());
                        } else {
                            margins.block_start = margins.block_start.ClampNegativeToZero();
                            margins.block_end = margins.block_end.max(-fragment.BlockSize());
                        }
                        bounds.offset -=
                            LogicalOffset::new(margins.inline_start, margins.block_start);
                        bounds.size.inline_size += margins.InlineSum();
                        bounds.size.block_size += margins.BlockSum();
                        debug_assert!(bounds.size.inline_size >= LayoutUnit::default());
                        debug_assert!(bounds.size.block_size >= LayoutUnit::default());
                    }
                    if let Some(inflow_bounds) = &mut self.inflow_bounds_ {
                        inflow_bounds.UniteEvenIfEmpty(&bounds);
                    } else {
                        self.inflow_bounds_ = Some(bounds);
                    }
                }
            }
        }
        let relative_offset = relative_offset.expect("relative offset initialized");
        self.PropagateFromFragment(child, child_offset, relative_offset, inline_container);
        self.AddChildInternal(child, child_offset + relative_offset);
        self.SetRequiresContentBeforeBreaking(false);
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:735-741
    // cpp: layoutng_fragment_tree/box_fragment_builder.cc:398-479
    pub fn MoveChildrenInDirection(
        &mut self,
        offset: LayoutUnit,
        is_block_direction: bool,
        mut additional_offset_adjustment: Option<&mut dyn FnMut(&mut LogicalFragmentLink)>,
    ) {
        if offset == LayoutUnit::default() && additional_offset_adjustment.is_none() {
            return;
        }
        debug_assert!(self.is_new_fc_);
        if additional_offset_adjustment.is_none() {
            let size = if is_block_direction {
                self.FragmentBlockSize()
            } else {
                self.FragmentInlineSize()
            };
            debug_assert_ne!(size, kIndefiniteSize);
        }
        debug_assert!(self.oof_positioned_descendants_.is_empty());
        self.has_moved_children_ = true;

        if is_block_direction {
            if let Some(baseline) = &mut self.first_baseline_ {
                *baseline += offset;
            }
            if let Some(baseline) = &mut self.last_baseline_ {
                *baseline += offset;
            }
        }
        if let Some(bounds) = &mut self.inflow_bounds_ {
            if is_block_direction {
                bounds.offset.block_offset += offset;
            } else {
                bounds.offset.inline_offset += offset;
            }
        }

        let mut move_child = |child: &mut LogicalFragmentLink| {
            if let Some(adjustment) = &mut additional_offset_adjustment {
                adjustment(child);
            }
            if is_block_direction {
                child.offset.block_offset += offset;
            } else {
                child.offset.inline_offset += offset;
            }
        };
        for child in self.base_.children_.iter_mut() {
            move_child(child);
        }
        for child in self
            .base_
            .children_with_size_dependent_propagation_
            .iter_mut()
        {
            move_child(child);
        }
        for candidate in self.oof_positioned_candidates_.iter_mut() {
            let mut increase = LogicalOffset::default();
            if is_block_direction {
                increase.block_offset = offset;
            } else {
                increase.inline_offset = offset;
            }
            candidate.IncreaseStaticPositionOffset(increase);
        }
        for descendant in self.oof_positioned_fragmentainer_descendants_.iter_mut() {
            if is_block_direction {
                descendant.containing_block.IncreaseBlockOffset(offset);
                descendant
                    .fixedpos_containing_block
                    .IncreaseBlockOffset(offset);
            } else {
                descendant.containing_block.IncreaseInlineOffset(offset);
                descendant
                    .fixedpos_containing_block
                    .IncreaseInlineOffset(offset);
            }
        }
        let items_builder = self.ItemsBuilder();
        if !items_builder.is_null() {
            unsafe { &mut *items_builder }.MoveChildrenInDirection(offset, is_block_direction);
        }
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:815
    // cpp: layoutng_fragment_tree/box_fragment_builder.cc:481-618
    fn PropagateBreakInfo(&mut self, child_layout_result: &LayoutResult, offset: LogicalOffset) {
        debug_assert!(self.GetConstraintSpace().HasBlockFragmentation());
        let block_end_in_container = offset.block_offset
            - child_layout_result.AnnotationBlockOffsetAdjustment()
            + BlockSizeForFragmentation(child_layout_result, self.writing_direction_);
        self.block_size_for_fragmentation_ = self
            .block_size_for_fragmentation_
            .max(block_end_in_container);
        if self.GetConstraintSpace().RequiresContentBeforeBreaking()
            && child_layout_result.IsBlockSizeForFragmentationClamped()
        {
            self.is_block_size_for_fragmentation_clamped_ = true;
        }

        let child_fragment = child_layout_result.GetPhysicalFragment();
        let child_box_ptr = DynamicTo::<PhysicalBoxFragment>(child_fragment as *const _);
        let child_box = unsafe { child_box_ptr.as_ref() };
        let token = child_box.map_or(std::ptr::null(), |box_fragment| {
            box_fragment.GetBreakToken()
        });
        let child_is_in_same_flow = ((token.is_null() || !unsafe { &*token }.IsAtBlockEnd())
            && !child_fragment.IsFloatingOrOutOfFlowPositioned())
            || child_layout_result.ShouldForceSameFragmentationFlow();

        if self.GetConstraintSpace().IsPaginated()
            && ((child_is_in_same_flow && !self.IsFragmentainerBoxType())
                || self.node_.IsPaginatedRoot())
            && child_box
                .is_none_or(|box_fragment| !box_fragment.IsMonolithicOverflowPropagationDisabled())
        {
            debug_assert!(self.GetConstraintSpace().HasKnownFragmentainerBlockSize());
            let block_size = if self.node_.IsPaginatedRoot()
                && !child_fragment.HasNonVisibleBlockOverflow()
            {
                let box_fragment = child_box.expect("paginated root has a box child");
                LogicalBoxFragment::new(box_fragment.Style().GetWritingDirection(), box_fragment)
                    .BlockEndScrollableOverflow()
            } else {
                LogicalFragment::new(child_fragment.Style().GetWritingDirection(), child_fragment)
                    .BlockSize()
            };
            let fragment_block_end = offset.block_offset + block_size;
            let fragmentainer_overflow = fragment_block_end - FragmentainerSpaceLeft(self, false);
            if fragmentainer_overflow > LayoutUnit::default() {
                self.ReserveSpaceForMonolithicOverflow(fragmentainer_overflow);
            }
        }

        if IsBreakInside(token) {
            if child_is_in_same_flow {
                self.has_inflow_child_break_inside_ = true;
            }
            let appeal =
                CalculateBreakAppealInsideDefault(self.GetConstraintSpace(), child_layout_result);
            self.ClampBreakAppeal(appeal);
        }
        if self.GetConstraintSpace().IsInitialColumnBalancingPass() {
            self.PropagateTallestUnbreakableBlockSize(
                child_layout_result.TallestUnbreakableBlockSize(),
            );
        }
        if child_layout_result.HasForcedBreak() {
            self.SetHasForcedBreak();
        } else if !self.GetConstraintSpace().IsInitialColumnBalancingPass() {
            self.PropagateSpaceShortage(child_layout_result.MinimalSpaceShortage());
        }
        let Some(box_fragment) = child_box else {
            return;
        };

        if self.GetConstraintSpace().IsInColumnBfc() {
            let child_spanner_path = child_layout_result.GetColumnSpannerPath();
            if !child_spanner_path.is_null() {
                let node = BlockNode::from(self.node_.clone());
                let path =
                    MakeGarbageCollected(ColumnSpannerPath::with_child(node, child_spanner_path));
                self.SetColumnSpannerPath(unsafe { &*path });
                self.SetIsEmptySpannerParent(child_layout_result.IsEmptySpannerParent());
                debug_assert!(self.HasInflowChildBreakInside() || !child_fragment.IsBox());
            }
        } else {
            debug_assert!(child_layout_result.GetColumnSpannerPath().is_null());
        }
        if !RuntimeEnabledFeatures::FragmentedOofInCbEnabled()
            && !box_fragment.IsFragmentainerBox()
            && !self.HasOutOfFlowInFragmentainerSubtree()
        {
            self.SetHasOutOfFlowInFragmentainerSubtree(
                box_fragment.HasOutOfFlowInFragmentainerSubtree(),
            );
        }
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:781-783
    // cpp: layoutng_fragment_tree/box_fragment_builder.cc:620-660
    pub fn PropagateChildBreakValues(&mut self, child_layout_result: &LayoutResult) {
        if child_layout_result.Status() != EStatus::kSuccess {
            return;
        }
        let fragment = child_layout_result.GetPhysicalFragment();
        if fragment.IsInline()
            || !fragment.IsBox()
            || fragment.IsColumnBox()
            || fragment.IsFloatingOrOutOfFlowPositioned()
        {
            return;
        }
        let child_style = fragment.Style();
        let break_before = JoinFragmentainerBreakValues(
            child_layout_result.InitialBreakBefore(),
            child_style.BreakBefore(),
        );
        self.SetInitialBreakBeforeIfNeeded(break_before);
        let break_after = JoinFragmentainerBreakValues(
            child_layout_result.FinalBreakAfter(),
            child_style.BreakAfter(),
        );
        self.SetPreviousBreakAfter(break_after);
        let box_fragment = unsafe { &*To::<PhysicalBoxFragment>(fragment as *const _) };
        let page_name = PageNameForChildFragment(self, box_fragment);
        self.SetPageNameIfNeeded(page_name);
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:807-808
    // cpp: layoutng_fragment_tree/box_fragment_builder.cc:662-682
    pub fn HandleOofsAndSpecialDescendants(&mut self) {
        self.has_final_size_ = true;
        self.PropagateSizeDependentData();
        let algorithms = LayoutPassScope::Algorithms();
        let out_of_flow = if algorithms.is_null() {
            None
        } else {
            unsafe { &*algorithms }.out_of_flow_support.run
        };
        if let Some(run) = out_of_flow {
            run(self);
        } else if self.HasOutOfFlowPositionedCandidates()
            || self.HasOutOfFlowPositionedDescendants()
            || self.HasOutOfFlowFragmentainerDescendants()
            || self.HasMulticolsWithPendingOOFs()
        {
            std::panic::panic_any(layoutng::UnsupportedLayout::new(
                "out-of-flow descendants require the out-of-flow package",
            ));
        }
        if !self.Style().ScrollMarkerGroupNone() && !self.GetConstraintSpace().IsAnonymous() {
            self.Node().HandleScrollMarkerGroup();
        }
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.h:813-816
    // cpp: layoutng_fragment_tree/box_fragment_builder.cc:684-790
    fn ToBoxFragmentWithWritingMode(
        &mut self,
        block_or_line_writing_mode: WritingMode,
    ) -> *const LayoutResult {
        #[cfg(debug_assertions)]
        if !self.ItemsBuilder().is_null() {
            for child in self.Children().iter() {
                debug_assert!(!child.get().is_null());
                let fragment = unsafe { &*child.get() };
                debug_assert!(fragment.IsLineBox() || fragment.IsFloatingOrOutOfFlowPositioned());
            }
        }
        self.Finalize();
        if self.GetBoxType() == BoxType::kNormalBox
            && !self.node_.IsNull()
            && self.node_.IsBlockInInline()
        {
            self.SetIsBlockInInline();
        }
        let space = self.GetConstraintSpace().clone();
        if space.HasBlockFragmentation() && !self.node_.IsNull() {
            let previous = self.PreviousBreakToken();
            if !previous.is_null() && unsafe { &*previous }.IsAtBlockEnd() {
                self.end_margin_strut_ = MarginStrut::default();
            }
            if self.break_token_.is_null() {
                if !self.last_inline_break_token_.is_null() {
                    let inline_break_token = self.last_inline_break_token_ as *mut BreakToken;
                    self.child_break_tokens_
                        .push(Member::from_ptr(inline_break_token));
                    self.last_inline_break_token_ = std::ptr::null();
                }
                if self.DidBreakSelf() || self.ShouldBreakInside() {
                    self.break_token_ = BlockBreakToken::Create(self) as *const BreakToken;
                }
            }
            if !self.break_token_.is_null()
                && !self.is_at_block_end_
                && space.IsInsideBalancedColumns()
                && !self.IsFragmentainerBoxType()
                && IsAvoidBreakValue(&space, self.Style().BreakInside())
                && !space.IsInsideBreakAvoid()
                && space.HasKnownFragmentainerBlockSize()
            {
                self.minimal_space_shortage_ = kIndefiniteSize;
                if !IsBreakInside(previous) {
                    let _side_effects_disabled = DisableLayoutSideEffectsScope::new();
                    let measure_space = space.CloneWithoutFragmentation();
                    let measure_result = self.Node().Layout(
                        &measure_space,
                        std::ptr::null(),
                        std::ptr::null(),
                        std::ptr::null(),
                    );
                    if unsafe { &*measure_result }.Status() == EStatus::kSuccess {
                        let fragment = LogicalFragment::new(
                            measure_space.GetWritingDirection(),
                            unsafe { &*measure_result }.GetPhysicalFragment(),
                        );
                        let space_left = FragmentainerSpaceLeft(self, false);
                        self.minimal_space_shortage_ = fragment.BlockSize() - space_left;
                        debug_assert!(self.minimal_space_shortage_ > LayoutUnit::default());
                    }
                }
            }
            if !PhysicalFragment::IsFragmentainerBoxType(self.GetBoxType()) {
                let block_axis = if self.GetWritingDirection().IsHorizontal() {
                    kOverflowClipY
                } else {
                    kOverflowClipX
                };
                if (self.Node().GetOverflowClipAxes() & block_axis) != 0
                    || self.is_block_size_for_fragmentation_clamped_
                {
                    self.block_size_for_fragmentation_ = self.FragmentBlockSize();
                } else {
                    self.block_size_for_fragmentation_ = self
                        .block_size_for_fragmentation_
                        .max(self.FragmentBlockSize());
                }
                if self.IsKnownToFitInFragmentainer() {
                    self.early_break_ = std::ptr::null();
                }
            }
        }
        let fragment = PhysicalBoxFragment::Create(self, block_or_line_writing_mode);
        unsafe { &*fragment }.CheckType();
        MakeGarbageCollected(LayoutResult::from_box_fragment_builder(
            fragment as *const PhysicalFragment,
            self,
        ))
    }

    // cpp: layoutng_fragment_tree/box_fragment_builder.cc:792-807
    #[cfg(debug_assertions)]
    pub fn CheckNoBlockFragmentation(&self) {
        debug_assert!(!self.ShouldBreakInside());
        debug_assert!(!self.HasInflowChildBreakInside());
        debug_assert!(!self.DidBreakSelf());
        debug_assert!(!self.has_forced_break_);
        debug_assert!(self.break_token_data_.is_null());
        debug_assert_eq!(self.minimal_space_shortage_, kIndefiniteSize);
        if !self.GetConstraintSpace().ShouldPropagateChildBreakValues() {
            debug_assert!(self.initial_break_before_.is_none());
            debug_assert_eq!(self.previous_break_after_, EBreakBetween::kAuto);
        }
    }
}
