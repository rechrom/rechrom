// LayoutResult is still being translated. This module starts with the exact
// status and packed-bit representation used by the source.
use std::cell::{Cell, UnsafeCell};
use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

use foundation::{
    kIndefiniteSize, DynamicTo, EBreakBetween, GCedHeapHashSet, GCedHeapVector, HeapVector,
    LayoutUnit, MakeGarbageCollected, MarginStrut, Member, ScopedRefPtr, To, Traceable, Visitor,
    WeakMember,
};
use layoutng::internal::break_appeal::BreakAppeal;
use layoutng::internal::column_spanner_path::ColumnSpannerPath;
use layoutng::internal::constraint_space::ConstraintSpace;
use layoutng::internal::custom_layout_payload::SerializedScriptValue;
use layoutng::internal::devtools_flex_info::DevtoolsFlexInfo;
use layoutng::internal::early_break::EarlyBreak;
use layoutng::internal::exclusions::exclusion_space::ExclusionSpace;
use layoutng::internal::grid_layout_data::GridLayoutData;
use layoutng::internal::layout_node_metadata::Element;
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::non_overflowing_scroll_range::NonOverflowingScrollRange;
use layoutng::internal::table_constraint_space_data::TableConstraintSpaceData;
use layoutng_geometry::geometry::bfc_offset::{BfcDelta, BfcOffset};
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;

use crate::box_fragment_builder::BoxFragmentBuilder;
use crate::fragment_builder::FragmentBuilder;
use crate::logical_fragment::LogicalFragment;
use crate::physical_box_fragment::PhysicalBoxFragment;
use crate::physical_fragment::PhysicalFragment;

// C++ permits three semantic aliases for value 8; Rust enum discriminants
// cannot repeat, so a newtype preserves both the aliases and unknown values.
// cpp: layoutng_fragment_tree/layout_result.h:46-62
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EStatus(pub u8);

#[allow(non_upper_case_globals)]
impl EStatus {
    pub const kSuccess: Self = Self(0);
    pub const kBfcBlockOffsetResolved: Self = Self(1);
    pub const kNeedsEarlierBreak: Self = Self(2);
    pub const kOutOfFragmentainerSpace: Self = Self(3);
    pub const kNeedsLineClampRelayout: Self = Self(4);
    pub const kDisableFragmentation: Self = Self(5);
    pub const kNeedsRelayoutWithNoChildScrollbarChanges: Self = Self(6);
    pub const kTextBoxTrimEndDidNotApply: Self = Self(7);
    pub const kAlgorithmSpecific1: Self = Self(8);
    pub const kNeedsRelayoutWithRowCrossSizeChanges: Self = Self::kAlgorithmSpecific1;
    pub const kNeedsRelayoutAsLastTableBox: Self = Self::kAlgorithmSpecific1;
    pub const kMarginTrimEndDidNotApply: Self = Self::kAlgorithmSpecific1;
}

// The source fields use 31 of 32 bits. Explicit positions preserve the
// original four-byte storage and make all conditional reads auditable.
// cpp: layoutng_fragment_tree/layout_result.h:770-833
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct LayoutResultBitfields(u32);

#[allow(non_snake_case)]
impl LayoutResultBitfields {
    pub const HAS_RARE_DATA_EXCLUSION_SPACE: u32 = 0;
    pub const HAS_OOF_INSETS_FOR_GET_COMPUTED_STYLE: u32 = 1;
    pub const IS_BFC_BLOCK_OFFSET_NULLOPT: u32 = 2;
    pub const HAS_FORCED_BREAK: u32 = 3;
    pub const BREAK_APPEAL: u32 = 4;
    pub const IS_EMPTY_SPANNER_PARENT: u32 = 6;
    pub const IS_BLOCK_SIZE_FOR_FRAGMENTATION_CLAMPED: u32 = 7;
    pub const SHOULD_FORCE_SAME_FRAGMENTATION_FLOW: u32 = 8;
    pub const IS_SELF_COLLAPSING: u32 = 9;
    pub const IS_PUSHED_BY_FLOATS: u32 = 10;
    pub const ADJOINING_OBJECT_TYPES: u32 = 11;
    pub const IS_INITIAL_BLOCK_SIZE_INDEFINITE: u32 = 14;
    pub const HAS_DESCENDANT_THAT_DEPENDS_ON_PERCENTAGE_BLOCK_SIZE: u32 = 15;
    pub const SUBTREE_MODIFIED_MARGIN_STRUT: u32 = 16;
    pub const INITIAL_BREAK_BEFORE: u32 = 17;
    pub const FINAL_BREAK_AFTER: u32 = 21;
    pub const STATUS: u32 = 25;
    pub const IS_TRUNCATED_BY_FRAGMENTATION_LINE: u32 = 29;
    pub const HAS_ORTHOGONAL_FALLBACK_SIZE_DESCENDANT: u32 = 30;

    pub fn get_flag(&self, offset: u32) -> bool {
        self.0 & (1 << offset) != 0
    }

    pub fn set_flag(&mut self, offset: u32, value: bool) {
        self.0 = (self.0 & !(1 << offset)) | ((value as u32) << offset);
    }

    pub fn get_value(&self, offset: u32, width: u32) -> u32 {
        (self.0 >> offset) & ((1 << width) - 1)
    }

    pub fn set_value(&mut self, offset: u32, width: u32, value: u32) {
        let mask = ((1 << width) - 1) << offset;
        self.0 = (self.0 & !mask) | ((value << offset) & mask);
    }

    // cpp: layoutng_fragment_tree/layout_result.h:774-806
    pub fn new(
        is_self_collapsing: bool,
        is_pushed_by_floats: bool,
        adjoining_object_types: u8,
        has_descendant_that_depends_on_percentage_block_size: bool,
        subtree_modified_margin_strut: bool,
    ) -> Self {
        let mut result = Self(0);
        result.set_value(Self::BREAK_APPEAL, 2, 3); // kBreakAppealPerfect
        result.set_flag(Self::IS_SELF_COLLAPSING, is_self_collapsing);
        result.set_flag(Self::IS_PUSHED_BY_FLOATS, is_pushed_by_floats);
        result.set_value(
            Self::ADJOINING_OBJECT_TYPES,
            3,
            adjoining_object_types as u32,
        );
        result.set_flag(
            Self::HAS_DESCENDANT_THAT_DEPENDS_ON_PERCENTAGE_BLOCK_SIZE,
            has_descendant_that_depends_on_percentage_block_size,
        );
        result.set_flag(
            Self::SUBTREE_MODIFIED_MARGIN_STRUT,
            subtree_modified_margin_strut,
        );
        result.set_value(Self::INITIAL_BREAK_BEFORE, 4, 2); // EBreakBetween::kAuto
        result.set_value(Self::FINAL_BREAK_AFTER, 4, 2); // EBreakBetween::kAuto
        result.set_value(Self::STATUS, 4, EStatus::kSuccess.0 as u32);
        result
    }
}

impl Default for LayoutResultBitfields {
    fn default() -> Self {
        Self::new(false, false, 0, false, false)
    }
}

const _: () = assert!(std::mem::size_of::<LayoutResultBitfields>() == 4);

// C++'s ConcurrentlyReadBitField<uint16_t> permits one writer and multiple
// readers. AtomicU16 with relaxed ordering retains that access contract.
// cpp: layoutng_fragment_tree/layout_result.h:610-667
pub struct LayoutResultRareFlags(AtomicU16);

impl Clone for LayoutResultRareFlags {
    fn clone(&self) -> Self {
        Self(AtomicU16::new(self.0.load(Ordering::Relaxed)))
    }
}

impl Default for LayoutResultRareFlags {
    fn default() -> Self {
        Self(AtomicU16::new(0))
    }
}

#[allow(non_snake_case)]
impl LayoutResultRareFlags {
    const LINE_BOX_BFC_BLOCK_OFFSET_IS_SET: u16 = 1 << 0;
    const OOF_POSITIONED_OFFSET_IS_SET: u16 = 1 << 1;
    const NEEDS_ANCHOR_POSITION_SCROLL_ADJUSTMENT_IN_X: u16 = 1 << 2;
    const NEEDS_ANCHOR_POSITION_SCROLL_ADJUSTMENT_IN_Y: u16 = 1 << 3;
    const IS_BLOCK_END_TRIMMABLE_LINE: u16 = 1 << 4;
    const WOULD_BE_LAST_LINE_IF_NOT_FOR_ELLIPSIS: u16 = 1 << 5;

    fn get(&self, mask: u16) -> bool {
        self.0.load(Ordering::Relaxed) & mask != 0
    }

    fn set(&self, mask: u16, flag: bool) {
        if flag {
            self.0.fetch_or(mask, Ordering::Relaxed);
        } else {
            self.0.fetch_and(!mask, Ordering::Relaxed);
        }
    }

    pub fn line_box_bfc_block_offset_is_set(&self) -> bool {
        self.get(Self::LINE_BOX_BFC_BLOCK_OFFSET_IS_SET)
    }

    pub fn set_line_box_bfc_block_offset_is_set(&self, flag: bool) {
        self.set(Self::LINE_BOX_BFC_BLOCK_OFFSET_IS_SET, flag);
    }

    pub fn oof_positioned_offset_is_set(&self) -> bool {
        self.get(Self::OOF_POSITIONED_OFFSET_IS_SET)
    }

    pub fn set_oof_positioned_offset_is_set(&self, flag: bool) {
        self.set(Self::OOF_POSITIONED_OFFSET_IS_SET, flag);
    }

    pub fn needs_anchor_position_scroll_adjustment_in_x(&self) -> bool {
        self.get(Self::NEEDS_ANCHOR_POSITION_SCROLL_ADJUSTMENT_IN_X)
    }

    pub fn set_needs_anchor_position_scroll_adjustment_in_x(&self, flag: bool) {
        self.set(Self::NEEDS_ANCHOR_POSITION_SCROLL_ADJUSTMENT_IN_X, flag);
    }

    pub fn needs_anchor_position_scroll_adjustment_in_y(&self) -> bool {
        self.get(Self::NEEDS_ANCHOR_POSITION_SCROLL_ADJUSTMENT_IN_Y)
    }

    pub fn set_needs_anchor_position_scroll_adjustment_in_y(&self, flag: bool) {
        self.set(Self::NEEDS_ANCHOR_POSITION_SCROLL_ADJUSTMENT_IN_Y, flag);
    }

    pub fn is_block_end_trimmable_line(&self) -> bool {
        self.get(Self::IS_BLOCK_END_TRIMMABLE_LINE)
    }

    pub fn set_is_block_end_trimmable_line(&self) {
        self.set(Self::IS_BLOCK_END_TRIMMABLE_LINE, true);
    }

    pub fn would_be_last_line_if_not_for_ellipsis(&self) -> bool {
        self.get(Self::WOULD_BE_LAST_LINE_IF_NOT_FOR_ELLIPSIS)
    }

    pub fn set_would_be_last_line_if_not_for_ellipsis(&self) {
        self.set(Self::WOULD_BE_LAST_LINE_IF_NOT_FOR_ELLIPSIS, true);
    }
}

// The C++ rare-data object is GC-allocated only when a field needs it.
// cpp: layoutng_fragment_tree/layout_result.h:608-609
// cpp: layoutng_fragment_tree/layout_result.h:717-763
#[allow(non_snake_case)]
pub struct LayoutResultRareData {
    pub early_break: Member<EarlyBreak>,
    pub column_spanner_path: Member<ColumnSpannerPath>,
    pub grid_layout_data: Member<GridLayoutData>,
    pub end_margin_strut: MarginStrut,
    pub tallest_unbreakable_block_size: LayoutUnit,
    pub minimal_space_shortage: LayoutUnit,
    pub block_size_for_fragmentation: LayoutUnit,
    pub exclusion_space: ExclusionSpace,
    pub custom_layout_data: ScopedRefPtr<SerializedScriptValue>,
    pub annotation_overflow: LayoutUnit,
    pub block_end_annotation_space: LayoutUnit,
    pub lines_until_clamp: i32,
    pub line_clamp_after_layout_object: WeakMember<LayoutObject>,
    pub display_locks_affected_by_anchors: Member<GCedHeapHashSet<Member<Element>>>,
    pub flex_layout_data_: Member<DevtoolsFlexInfo>,
    pub clearance_after_line_: LayoutUnit,
    pub trim_block_end_by_: LayoutUnit,
    pub annotation_block_offset_adjustment_: LayoutUnit,
    pub math_italic_correction_: LayoutUnit,
    pub table_column_count_: u32,
    line_box_bfc_block_offset: LayoutUnit,
    non_overflowing_scroll_ranges: Member<GCedHeapVector<NonOverflowingScrollRange>>,
    oof_positioned_offset: LogicalOffset,
    bit_field: LayoutResultRareFlags,
}

impl Default for LayoutResultRareData {
    // cpp: layoutng_fragment_tree/layout_result.h:669-671
    // cpp: layoutng_fragment_tree/layout_result.cc:289-291
    fn default() -> Self {
        Self {
            early_break: Member::default(),
            column_spanner_path: Member::default(),
            grid_layout_data: Member::default(),
            end_margin_strut: MarginStrut::default(),
            tallest_unbreakable_block_size: LayoutUnit::default(),
            minimal_space_shortage: kIndefiniteSize,
            block_size_for_fragmentation: kIndefiniteSize,
            exclusion_space: ExclusionSpace::default(),
            custom_layout_data: ScopedRefPtr::default(),
            annotation_overflow: LayoutUnit::default(),
            block_end_annotation_space: LayoutUnit::default(),
            lines_until_clamp: 0,
            line_clamp_after_layout_object: WeakMember::default(),
            display_locks_affected_by_anchors: Member::default(),
            flex_layout_data_: Member::default(),
            clearance_after_line_: LayoutUnit::Min(),
            trim_block_end_by_: LayoutUnit::Min(),
            annotation_block_offset_adjustment_: LayoutUnit::default(),
            math_italic_correction_: LayoutUnit::default(),
            table_column_count_: 0,
            line_box_bfc_block_offset: LayoutUnit::default(),
            non_overflowing_scroll_ranges: Member::default(),
            oof_positioned_offset: LogicalOffset::default(),
            bit_field: LayoutResultRareFlags::default(),
        }
    }
}

impl Clone for LayoutResultRareData {
    // C++ explicitly default-copies RareData, including its GC handles and
    // concurrently readable flag word.
    // cpp: layoutng_fragment_tree/layout_result.cc:290
    fn clone(&self) -> Self {
        Self {
            early_break: self.early_break.clone(),
            column_spanner_path: self.column_spanner_path.clone(),
            grid_layout_data: self.grid_layout_data.clone(),
            end_margin_strut: self.end_margin_strut.clone(),
            tallest_unbreakable_block_size: self.tallest_unbreakable_block_size,
            minimal_space_shortage: self.minimal_space_shortage,
            block_size_for_fragmentation: self.block_size_for_fragmentation,
            exclusion_space: self.exclusion_space.clone(),
            custom_layout_data: self.custom_layout_data.clone(),
            annotation_overflow: self.annotation_overflow,
            block_end_annotation_space: self.block_end_annotation_space,
            lines_until_clamp: self.lines_until_clamp,
            line_clamp_after_layout_object: self.line_clamp_after_layout_object.clone(),
            display_locks_affected_by_anchors: self.display_locks_affected_by_anchors.clone(),
            flex_layout_data_: self.flex_layout_data_.clone(),
            clearance_after_line_: self.clearance_after_line_,
            trim_block_end_by_: self.trim_block_end_by_,
            annotation_block_offset_adjustment_: self.annotation_block_offset_adjustment_,
            math_italic_correction_: self.math_italic_correction_,
            table_column_count_: self.table_column_count_,
            line_box_bfc_block_offset: self.line_box_bfc_block_offset,
            non_overflowing_scroll_ranges: self.non_overflowing_scroll_ranges.clone(),
            oof_positioned_offset: self.oof_positioned_offset,
            bit_field: self.bit_field.clone(),
        }
    }
}

#[allow(non_snake_case)]
impl LayoutResultRareData {
    // cpp: layoutng_fragment_tree/layout_result.h:673-681
    pub fn SetLineBoxBfcBlockOffset(&mut self, offset: LayoutUnit) {
        self.line_box_bfc_block_offset = offset;
        self.bit_field.set_line_box_bfc_block_offset_is_set(true);
    }

    pub fn LineBoxBfcBlockOffset(&self) -> Option<LayoutUnit> {
        if !self.bit_field.line_box_bfc_block_offset_is_set() {
            return None;
        }
        Some(self.line_box_bfc_block_offset)
    }

    // cpp: layoutng_fragment_tree/layout_result.h:683-689
    pub fn ClearanceAfterLine(&self) -> Option<LayoutUnit> {
        self.clearance_after_line_.NullOptIfMin()
    }

    pub fn TrimBlockEndBy(&self) -> Option<LayoutUnit> {
        self.trim_block_end_by_.NullOptIfMin()
    }

    // cpp: layoutng_fragment_tree/layout_result.h:691-704
    pub fn SetNonOverflowingScrollRanges(
        &mut self,
        non_overflowing_ranges: &HeapVector<NonOverflowingScrollRange>,
    ) {
        if non_overflowing_ranges.is_empty() {
            self.non_overflowing_scroll_ranges = Member::default();
        } else {
            self.non_overflowing_scroll_ranges = Member::from_ptr(MakeGarbageCollected(
                GCedHeapVector::from(non_overflowing_ranges.iter().cloned().collect::<Vec<_>>()),
            ));
        }
    }

    pub fn NonOverflowingScrollRanges(&self) -> *const GCedHeapVector<NonOverflowingScrollRange> {
        self.non_overflowing_scroll_ranges.Get()
    }

    // cpp: layoutng_fragment_tree/layout_result.h:706-713
    pub fn SetOutOfFlowPositionedOffset(&mut self, offset: &LogicalOffset) {
        self.oof_positioned_offset = *offset;
        self.bit_field.set_oof_positioned_offset_is_set(true);
    }

    pub fn OutOfFlowPositionedOffset(&self) -> LogicalOffset {
        assert!(self.bit_field.oof_positioned_offset_is_set());
        self.oof_positioned_offset
    }

    // cpp: layoutng_fragment_tree/layout_result.h:715
    // cpp: layoutng_fragment_tree/layout_result.cc:372-381
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.early_break);
        visitor.Trace(&self.non_overflowing_scroll_ranges);
        visitor.Trace(&self.column_spanner_path);
        visitor.Trace(&self.grid_layout_data);
        visitor.Trace(&self.exclusion_space);
        visitor.Trace(&self.line_clamp_after_layout_object);
        visitor.Trace(&self.display_locks_affected_by_anchors);
        visitor.Trace(&self.flex_layout_data_);
    }
}

// The C++ const out-of-flow facade mutates RareData. Keeping the GC referent
// behind UnsafeCell makes those controlled writes explicit in Rust.
#[repr(transparent)]
struct LayoutResultRareDataCell(UnsafeCell<LayoutResultRareData>);

#[allow(non_snake_case)]
impl LayoutResultRareDataCell {
    fn Trace(&self, visitor: &mut Visitor) {
        unsafe { &*self.0.get() }.Trace(visitor);
    }
}

impl Traceable for LayoutResultRareDataCell {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        LayoutResultRareDataCell::Trace(self, visitor);
    }
}

// The source union's active arm is selected by
// has_oof_insets_for_get_computed_style. ManuallyDrop prevents Rust from
// reading or dropping the inactive arm.
// cpp: layoutng_fragment_tree/layout_result.h:840-852
#[repr(C)]
union LayoutResultOffsetData {
    bfc_offset_: ManuallyDrop<BfcOffset>,
    oof_insets_for_get_computed_style_: ManuallyDrop<BoxStrut>,
}

// The deleted C++ copy and move operations remain absent. UnsafeCell permits
// only the narrowly exposed const mutation used by out-of-flow layout.
// cpp: layoutng_fragment_tree/layout_result.h:44-45
// cpp: layoutng_fragment_tree/layout_result.h:92-102
// cpp: layoutng_fragment_tree/layout_result.h:835-855
#[repr(C)]
pub struct LayoutResult {
    space_: UnsafeCell<ConstraintSpace>,
    physical_fragment_: Member<PhysicalFragment>,
    rare_data_: UnsafeCell<Member<LayoutResultRareDataCell>>,
    offset_data_: UnsafeCell<LayoutResultOffsetData>,
    intrinsic_block_size_: LayoutUnit,
    bitfields_: Cell<LayoutResultBitfields>,
}

// C++ ASSERT_SIZE compares this shape with LayoutResult. The check will be
// type-checked once the external ConstraintSpace and GC handle are connected.
// cpp: layoutng_fragment_tree/layout_result.cc:30-43
#[repr(C)]
struct SameSizeAsLayoutResult {
    space: ConstraintSpace,
    physical_fragment: Member<PhysicalFragment>,
    rare_data: Member<LayoutResultRareDataCell>,
    offset_data: LayoutResultOffsetData,
    intrinsic_block_size: LayoutUnit,
    bitfields: [u32; 1],
}

const _: () =
    assert!(std::mem::size_of::<LayoutResult>() == std::mem::size_of::<SameSizeAsLayoutResult>());

#[allow(non_snake_case)]
impl LayoutResult {
    fn bits(&self) -> LayoutResultBitfields {
        self.bitfields_.get()
    }

    // cpp: layoutng_fragment_tree/layout_result.h:88-90
    // cpp: layoutng_fragment_tree/layout_result.cc:185-266
    pub fn from_fragment_builder(
        physical_fragment: *const PhysicalFragment,
        builder: &mut FragmentBuilder,
    ) -> Self {
        let result = Self {
            space_: UnsafeCell::new(builder.space_.clone()),
            physical_fragment_: Member::from_ptr(physical_fragment as *mut _),
            rare_data_: UnsafeCell::new(Member::default()),
            offset_data_: UnsafeCell::new(LayoutResultOffsetData {
                bfc_offset_: ManuallyDrop::new(BfcOffset::default()),
            }),
            intrinsic_block_size_: LayoutUnit::default(),
            bitfields_: Cell::new(LayoutResultBitfields::new(
                builder.is_self_collapsing_,
                builder.is_pushed_by_floats_,
                builder.adjoining_object_types_ as u8,
                builder.has_descendant_that_depends_on_percentage_block_size_,
                builder.subtree_modified_margin_strut_,
            )),
        };

        // cpp: layoutng_fragment_tree/layout_result.cc:195-205
        #[cfg(debug_assertions)]
        if result.IsSelfCollapsing() && !physical_fragment.is_null() {
            let physical = unsafe { &*physical_fragment };
            debug_assert!(!physical.IsFormattingContextRoot());
            let logical = LogicalFragment::new(physical.Style().GetWritingDirection(), physical);
            debug_assert_eq!(LayoutUnit::default(), logical.BlockSize());
        }

        // cpp: layoutng_fragment_tree/layout_result.cc:207-234
        if builder.end_margin_strut_ != MarginStrut::default() {
            result.EnsureRareData().end_margin_strut = builder.end_margin_strut_.clone();
        }
        if builder.annotation_overflow_ > LayoutUnit::default() {
            result.EnsureRareData().annotation_overflow = builder.annotation_overflow_;
        }
        if builder.block_end_annotation_space_ != LayoutUnit::default() {
            result.EnsureRareData().block_end_annotation_space =
                builder.block_end_annotation_space_;
        }
        if builder.exclusion_space_ != *result.GetConstraintSpaceForCaching().GetExclusionSpace() {
            let mut bits = result.bits();
            bits.set_flag(LayoutResultBitfields::HAS_RARE_DATA_EXCLUSION_SPACE, true);
            result.bitfields_.set(bits);
            result.EnsureRareData().exclusion_space = std::mem::take(&mut builder.exclusion_space_);
        } else {
            result
                .GetConstraintSpaceForCaching()
                .GetExclusionSpace()
                .MoveDerivedGeometry(&mut builder.exclusion_space_);
        }
        if let Some(lines) = builder.lines_until_clamp_ {
            result.EnsureRareData().lines_until_clamp = lines;
        }
        if builder.is_block_end_trimmable_line_ {
            result
                .EnsureRareData()
                .bit_field
                .set_is_block_end_trimmable_line();
        }
        if builder.would_be_last_line_if_not_for_ellipsis_ {
            result
                .EnsureRareData()
                .bit_field
                .set_would_be_last_line_if_not_for_ellipsis();
        }
        if !builder.line_clamp_after_layout_object_.is_null() {
            result.EnsureRareData().line_clamp_after_layout_object =
                WeakMember::from_ptr(builder.line_clamp_after_layout_object_ as *mut _);
        }

        // cpp: layoutng_fragment_tree/layout_result.cc:236-255
        if builder.tallest_unbreakable_block_size_ >= LayoutUnit::default() {
            result.EnsureRareData().tallest_unbreakable_block_size =
                builder.tallest_unbreakable_block_size_;
        } else if builder.minimal_space_shortage_ != kIndefiniteSize {
            result.EnsureRareData().minimal_space_shortage = builder.minimal_space_shortage_;
        }
        if !builder.early_break_.is_null()
            && (physical_fragment.is_null()
                || unsafe { &*physical_fragment }.GetBreakToken().is_null())
        {
            result.EnsureRareData().early_break = Member::from_ptr(builder.early_break_ as *mut _);
        }

        // cpp: layoutng_fragment_tree/layout_result.cc:257-265
        let mut bits = result.bits();
        bits.set_value(
            LayoutResultBitfields::BREAK_APPEAL,
            2,
            builder.break_appeal_ as u32,
        );
        bits.set_flag(
            LayoutResultBitfields::HAS_ORTHOGONAL_FALLBACK_SIZE_DESCENDANT,
            builder.has_orthogonal_fallback_size_descendant_,
        );
        result.bitfields_.set(bits);
        unsafe {
            (*result.offset_data_.get()).bfc_offset_ = ManuallyDrop::new(BfcOffset::new(
                builder.bfc_line_offset_,
                builder.bfc_block_offset_.unwrap_or_default(),
            ));
        }
        bits.set_flag(
            LayoutResultBitfields::IS_BFC_BLOCK_OFFSET_NULLOPT,
            builder.bfc_block_offset_.is_none(),
        );
        result.bitfields_.set(bits);
        result
    }

    // cpp: layoutng_fragment_tree/layout_result.h:585-589
    // cpp: layoutng_fragment_tree/layout_result.cc:55-115
    pub fn from_box_fragment_builder(
        physical_fragment: *const PhysicalFragment,
        builder: &mut BoxFragmentBuilder,
    ) -> Self {
        let mut result = Self::from_fragment_builder(physical_fragment, builder.base_mut());
        if !builder.column_spanner_path_.is_null() {
            result.EnsureRareData().column_spanner_path =
                Member::from_ptr(builder.column_spanner_path_ as *mut _);
            let mut bits = result.bits();
            bits.set_flag(
                LayoutResultBitfields::IS_EMPTY_SPANNER_PARENT,
                builder.is_empty_spanner_parent_,
            );
            result.bitfields_.set(bits);
        }
        let mut bits = result.bits();
        bits.set_flag(
            LayoutResultBitfields::SHOULD_FORCE_SAME_FRAGMENTATION_FLOW,
            builder.should_force_same_fragmentation_flow_,
        );
        bits.set_flag(
            LayoutResultBitfields::IS_INITIAL_BLOCK_SIZE_INDEFINITE,
            builder.is_initial_block_size_indefinite_,
        );
        result.bitfields_.set(bits);
        result.intrinsic_block_size_ = builder.intrinsic_block_size_;
        if !builder.custom_layout_data_.get().is_null() {
            result.EnsureRareData().custom_layout_data =
                std::mem::take(&mut builder.custom_layout_data_);
        }
        if builder.annotation_overflow_ != LayoutUnit::default() {
            result.EnsureRareData().annotation_overflow = builder.annotation_overflow_;
        }
        if builder.block_end_annotation_space_ != LayoutUnit::default() {
            result.EnsureRareData().block_end_annotation_space =
                builder.block_end_annotation_space_;
        }
        if builder.GetConstraintSpace().HasBlockFragmentation() {
            result.EnsureRareData().block_size_for_fragmentation =
                builder.block_size_for_fragmentation_;
            let mut bits = result.bits();
            bits.set_flag(
                LayoutResultBitfields::IS_BLOCK_SIZE_FOR_FRAGMENTATION_CLAMPED,
                builder.is_block_size_for_fragmentation_clamped_,
            );
            bits.set_flag(
                LayoutResultBitfields::HAS_FORCED_BREAK,
                builder.has_forced_break_,
            );
            result.bitfields_.set(bits);
        }
        let mut bits = result.bits();
        bits.set_flag(
            LayoutResultBitfields::IS_TRUNCATED_BY_FRAGMENTATION_LINE,
            builder.is_truncated_by_fragmentation_line,
        );
        result.bitfields_.set(bits);
        if builder
            .GetConstraintSpace()
            .ShouldPropagateChildBreakValues()
            && !unsafe { &*builder.layout_object_ }.ShouldApplyLayoutContainment()
        {
            let mut bits = result.bits();
            bits.set_value(
                LayoutResultBitfields::INITIAL_BREAK_BEFORE,
                4,
                builder
                    .initial_break_before_
                    .unwrap_or(EBreakBetween::kAuto) as u32,
            );
            bits.set_value(
                LayoutResultBitfields::FINAL_BREAK_AFTER,
                4,
                builder.previous_break_after_ as u32,
            );
            result.bitfields_.set(bits);
        }
        if let Some(count) = builder.table_column_count_ {
            result.EnsureRareData().table_column_count_ = count;
        }
        if builder.math_italic_correction_ != LayoutUnit::default() {
            result.EnsureRareData().math_italic_correction_ = builder.math_italic_correction_;
        }
        if !builder.grid_layout_data_.is_null() {
            result.EnsureRareData().grid_layout_data =
                Member::from_ptr(builder.grid_layout_data_ as *mut _);
        }
        if !builder.flex_layout_data_.is_null() {
            result.EnsureRareData().flex_layout_data_ =
                Member::from_ptr(builder.flex_layout_data_ as *mut _);
        }
        result
    }

    // cpp: layoutng_fragment_tree/layout_result.h:581-583
    // cpp: layoutng_fragment_tree/layout_result.cc:117-124
    pub fn from_failed_fragment_builder(status: EStatus, builder: &mut FragmentBuilder) -> Self {
        let result = Self::from_fragment_builder(std::ptr::null(), builder);
        let mut bits = result.bits();
        bits.set_value(LayoutResultBitfields::STATUS, 4, status.0 as u32);
        result.bitfields_.set(bits);
        debug_assert_ne!(status, EStatus::kSuccess);
        result
    }

    fn copied_rare_data(other: &Self) -> Member<LayoutResultRareDataCell> {
        other.rare().map_or(Member::default(), |rare| {
            Member::from_ptr(MakeGarbageCollected(LayoutResultRareDataCell(
                UnsafeCell::new(rare.clone()),
            )))
        })
    }

    // cpp: layoutng_fragment_tree/layout_result.h:74-82
    // cpp: layoutng_fragment_tree/layout_result.cc:126-163
    pub fn copy_with_new_space(
        other: &Self,
        new_space: &ConstraintSpace,
        new_end_margin_strut: &MarginStrut,
        bfc_line_offset: LayoutUnit,
        bfc_block_offset: Option<LayoutUnit>,
        block_offset_delta: LayoutUnit,
    ) -> Self {
        let result = Self {
            space_: UnsafeCell::new(new_space.clone()),
            physical_fragment_: other.physical_fragment_.clone(),
            rare_data_: UnsafeCell::new(Self::copied_rare_data(other)),
            offset_data_: UnsafeCell::new(LayoutResultOffsetData {
                bfc_offset_: ManuallyDrop::new(BfcOffset::default()),
            }),
            intrinsic_block_size_: other.intrinsic_block_size_,
            bitfields_: Cell::new(other.bits()),
        };
        let mut bits = result.bits();
        if !bits.get_flag(LayoutResultBitfields::HAS_OOF_INSETS_FOR_GET_COMPUTED_STYLE) {
            unsafe {
                (*result.offset_data_.get()).bfc_offset_ = ManuallyDrop::new(BfcOffset::new(
                    bfc_line_offset,
                    bfc_block_offset.unwrap_or_default(),
                ));
            }
            bits.set_flag(
                LayoutResultBitfields::IS_BFC_BLOCK_OFFSET_NULLOPT,
                bfc_block_offset.is_none(),
            );
            result.bitfields_.set(bits);
        } else {
            debug_assert!(unsafe { &*result.physical_fragment_.Get() }.IsOutOfFlowPositioned());
            debug_assert_eq!(bfc_line_offset, LayoutUnit::default());
            debug_assert_eq!(bfc_block_offset, Some(LayoutUnit::default()));
            unsafe {
                (*result.offset_data_.get()).oof_insets_for_get_computed_style_ =
                    ManuallyDrop::new(BoxStrut::default());
            }
        }
        let mut new_exclusion_space = Self::MergeExclusionSpaces(
            other,
            result.GetConstraintSpaceForCaching().GetExclusionSpace(),
            bfc_line_offset,
            block_offset_delta,
        );
        if new_exclusion_space != *result.GetConstraintSpaceForCaching().GetExclusionSpace() {
            bits.set_flag(LayoutResultBitfields::HAS_RARE_DATA_EXCLUSION_SPACE, true);
            result.bitfields_.set(bits);
            result.EnsureRareData().exclusion_space = new_exclusion_space;
        } else {
            result
                .GetConstraintSpaceForCaching()
                .GetExclusionSpace()
                .MoveDerivedGeometry(&mut new_exclusion_space);
        }
        if *new_end_margin_strut != MarginStrut::default() || result.rare().is_some() {
            result.EnsureRareData().end_margin_strut = new_end_margin_strut.clone();
        }
        result
    }

    // cpp: layoutng_fragment_tree/layout_result.h:84-86
    // cpp: layoutng_fragment_tree/layout_result.cc:165-183
    pub fn copy_with_fragment(other: &Self, physical_fragment: *const PhysicalFragment) -> Self {
        let result = Self {
            space_: UnsafeCell::new(other.GetConstraintSpaceForCaching().clone()),
            physical_fragment_: Member::from_ptr(physical_fragment as *mut _),
            rare_data_: UnsafeCell::new(Self::copied_rare_data(other)),
            offset_data_: UnsafeCell::new(LayoutResultOffsetData {
                bfc_offset_: ManuallyDrop::new(BfcOffset::default()),
            }),
            intrinsic_block_size_: other.intrinsic_block_size_,
            bitfields_: Cell::new(other.bits()),
        };
        if !result
            .bits()
            .get_flag(LayoutResultBitfields::HAS_OOF_INSETS_FOR_GET_COMPUTED_STYLE)
        {
            unsafe {
                (*result.offset_data_.get()).bfc_offset_ =
                    ManuallyDrop::new(*other.offset_data_.get().cast::<BfcOffset>());
            }
        } else {
            debug_assert!(unsafe { &*physical_fragment }.IsOutOfFlowPositioned());
            unsafe {
                (*result.offset_data_.get()).oof_insets_for_get_computed_style_ =
                    ManuallyDrop::new(*other.OutOfFlowInsetsForGetComputedStyle());
            }
        }
        debug_assert_eq!(
            unsafe { &*physical_fragment }.Size(),
            unsafe { &*other.physical_fragment_.Get() }.Size(),
        );
        result
    }

    // cpp: layoutng_fragment_tree/layout_result.h:64-67
    // cpp: layoutng_fragment_tree/layout_result.cc:22-26
    pub fn Clone(other: &Self) -> *const Self {
        let box_fragment = unsafe {
            &*To::<PhysicalBoxFragment>(other.GetPhysicalFragment() as *const PhysicalFragment)
        };
        let cloned = PhysicalBoxFragment::Clone(box_fragment);
        MakeGarbageCollected(Self::copy_with_fragment(
            other,
            cloned as *const PhysicalFragment,
        ))
    }

    // cpp: layoutng_fragment_tree/layout_result.h:69-72
    // cpp: layoutng_fragment_tree/layout_result.cc:47-53
    pub fn CloneWithPostLayoutFragments(other: &Self) -> *const Self {
        let box_fragment = unsafe {
            &*To::<PhysicalBoxFragment>(other.GetPhysicalFragment() as *const PhysicalFragment)
        };
        let cloned = PhysicalBoxFragment::CloneWithPostLayoutFragments(box_fragment);
        MakeGarbageCollected(Self::copy_with_fragment(
            other,
            cloned as *const PhysicalFragment,
        ))
    }

    // cpp: layoutng_fragment_tree/layout_result.h:766-768
    // cpp: layoutng_fragment_tree/layout_result.cc:358-364
    #[cfg(debug_assertions)]
    fn AssertSoleBoxFragment(&self) {
        let physical = self.GetPhysicalFragment();
        debug_assert!(physical.IsBox());
        let box_fragment = unsafe { &*To::<PhysicalBoxFragment>(physical as *const _) };
        debug_assert!(box_fragment.IsFirstForNode());
        debug_assert!(physical.GetBreakToken().is_null());
    }

    // cpp: layoutng_fragment_tree/layout_result.h:576-579
    // cpp: layoutng_fragment_tree/layout_result.cc:313-356
    #[cfg(debug_assertions)]
    pub fn CheckSameForSimplifiedLayout(&self, other: &Self, check_no_fragmentation: bool) {
        let box_fragment = unsafe { &*To::<PhysicalBoxFragment>(self.physical_fragment_.Get()) };
        let other_box_fragment =
            unsafe { &*To::<PhysicalBoxFragment>(other.physical_fragment_.Get()) };
        box_fragment.CheckSameForSimplifiedLayout(other_box_fragment, check_no_fragmentation);
        debug_assert_eq!(self.LinesUntilClamp(), other.LinesUntilClamp());
        self.GetExclusionSpace()
            .CheckSameForSimplifiedLayout(other.GetExclusionSpace());
        debug_assert_eq!(self.EndMarginStrut(), other.EndMarginStrut());
        debug_assert_eq!(self.MinimalSpaceShortage(), other.MinimalSpaceShortage());
        debug_assert_eq!(self.TableColumnCount(), other.TableColumnCount());
        debug_assert_eq!(self.HasForcedBreak(), other.HasForcedBreak());
        debug_assert_eq!(self.IsSelfCollapsing(), other.IsSelfCollapsing());
        debug_assert_eq!(self.IsPushedByFloats(), other.IsPushedByFloats());
        debug_assert_eq!(
            self.GetAdjoiningObjectTypes(),
            other.GetAdjoiningObjectTypes()
        );
        debug_assert_eq!(
            self.SubtreeModifiedMarginStrut(),
            other.SubtreeModifiedMarginStrut()
        );
        debug_assert_eq!(self.CustomLayoutData(), other.CustomLayoutData());
        debug_assert_eq!(self.InitialBreakBefore(), other.InitialBreakBefore());
        debug_assert_eq!(self.FinalBreakAfter(), other.FinalBreakAfter());
        debug_assert_eq!(
            self.HasDescendantThatDependsOnPercentageBlockSize(),
            other.HasDescendantThatDependsOnPercentageBlockSize()
        );
        debug_assert_eq!(self.Status(), other.Status());
    }

    fn rare(&self) -> Option<&LayoutResultRareData> {
        let ptr = unsafe { &*self.rare_data_.get() }.Get();
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { &*(*ptr).0.get() })
        }
    }

    // cpp: layoutng_fragment_tree/layout_result.h:764
    // cpp: layoutng_fragment_tree/layout_result.cc:281-287
    pub(crate) fn EnsureRareData(&self) -> &mut LayoutResultRareData {
        let member = unsafe { &mut *self.rare_data_.get() };
        if member.Get().is_null() {
            *member = Member::from_ptr(MakeGarbageCollected(LayoutResultRareDataCell(
                UnsafeCell::new(LayoutResultRareData::default()),
            )));
        }
        unsafe { &mut *(*member.Get()).0.get() }
    }

    // cpp: layoutng_fragment_tree/layout_result.h:602-606
    // cpp: layoutng_fragment_tree/layout_result.cc:268-280
    fn MergeExclusionSpaces(
        other: &Self,
        new_input_exclusion_space: &ExclusionSpace,
        bfc_line_offset: LayoutUnit,
        block_offset_delta: LayoutUnit,
    ) -> ExclusionSpace {
        let offset_delta =
            BfcDelta::new(bfc_line_offset - other.BfcLineOffset(), block_offset_delta);
        ExclusionSpace::MergeExclusionSpaces(
            other.GetExclusionSpace(),
            other.GetConstraintSpaceForCaching().GetExclusionSpace(),
            new_input_exclusion_space,
            &offset_delta,
        )
    }

    // cpp: layoutng_fragment_tree/layout_result.h:104-108
    pub fn GetPhysicalFragment(&self) -> &PhysicalFragment {
        let ptr = self.physical_fragment_.Get();
        debug_assert!(!ptr.is_null());
        debug_assert_eq!(self.Status(), EStatus::kSuccess);
        unsafe { &*ptr }
    }

    // cpp: layoutng_fragment_tree/layout_result.h:110-116
    pub fn LinesUntilClamp(&self) -> i32 {
        self.rare().map_or(0, |rare| rare.lines_until_clamp)
    }

    pub fn LineClampAfterLayoutObject(&self) -> *const LayoutObject {
        self.rare().map_or(std::ptr::null(), |rare| {
            rare.line_clamp_after_layout_object.Get()
        })
    }

    // cpp: layoutng_fragment_tree/layout_result.h:118-133
    pub fn IsBlockEndTrimmableLine(&self) -> bool {
        self.rare()
            .is_some_and(|rare| rare.bit_field.is_block_end_trimmable_line())
    }

    pub fn WouldBeLastLineIfNotForEllipsis(&self) -> bool {
        self.rare()
            .is_some_and(|rare| rare.bit_field.would_be_last_line_if_not_for_ellipsis())
    }

    // cpp: layoutng_fragment_tree/layout_result.h:135-143
    pub fn HasOrthogonalFallbackInlineSize(&self) -> bool {
        self.GetConstraintSpaceForCaching()
            .UsesOrthogonalFallbackInlineSize()
    }

    pub fn HasOrthogonalFallbackSizeDescendant(&self) -> bool {
        self.bits()
            .get_flag(LayoutResultBitfields::HAS_ORTHOGONAL_FALLBACK_SIZE_DESCENDANT)
    }

    // cpp: layoutng_fragment_tree/layout_result.h:145-167
    pub fn AnnotationBlockOffsetAdjustment(&self) -> LayoutUnit {
        self.rare().map_or(LayoutUnit::default(), |rare| {
            rare.annotation_block_offset_adjustment_
        })
    }

    pub fn AnnotationOverflow(&self) -> LayoutUnit {
        self.rare()
            .map_or(LayoutUnit::default(), |rare| rare.annotation_overflow)
    }

    pub fn BlockEndAnnotationSpace(&self) -> LayoutUnit {
        self.rare().map_or(LayoutUnit::default(), |rare| {
            rare.block_end_annotation_space
        })
    }

    // cpp: layoutng_fragment_tree/layout_result.h:169-176
    pub fn OutOfFlowPositionedOffset(&self) -> LogicalOffset {
        assert!(self
            .bits()
            .get_flag(LayoutResultBitfields::HAS_OOF_INSETS_FOR_GET_COMPUTED_STYLE,));
        if let Some(rare) = self.rare() {
            if rare.bit_field.oof_positioned_offset_is_set() {
                return rare.OutOfFlowPositionedOffset();
            }
        }
        self.OutOfFlowInsetsForGetComputedStyle().StartOffset()
    }

    // cpp: layoutng_fragment_tree/layout_result.h:178-184
    pub fn OutOfFlowInsetsForGetComputedStyle(&self) -> &BoxStrut {
        assert!(self
            .bits()
            .get_flag(LayoutResultBitfields::HAS_OOF_INSETS_FOR_GET_COMPUTED_STYLE,));
        unsafe { &(*self.offset_data_.get()).oof_insets_for_get_computed_style_ }
    }

    // cpp: layoutng_fragment_tree/layout_result.h:186-188
    // cpp: layoutng_fragment_tree/layout_result.cc:293-301
    pub fn CopyMutableOutOfFlowData(&self, other: &LayoutResult) {
        if self
            .bits()
            .get_flag(LayoutResultBitfields::HAS_OOF_INSETS_FOR_GET_COMPUTED_STYLE)
        {
            return;
        }
        self.GetMutableForOutOfFlow()
            .SetOutOfFlowInsetsForGetComputedStyle(other.OutOfFlowInsetsForGetComputedStyle());
        self.GetMutableForOutOfFlow()
            .SetOutOfFlowPositionedOffset(&other.OutOfFlowPositionedOffset());
    }

    // cpp: layoutng_fragment_tree/layout_result.h:190-193
    pub fn NonOverflowingScrollRanges(&self) -> *const GCedHeapVector<NonOverflowingScrollRange> {
        self.rare()
            .map_or(std::ptr::null(), |rare| rare.NonOverflowingScrollRanges())
    }

    // cpp: layoutng_fragment_tree/layout_result.h:195-202
    pub fn NeedsAnchorPositionScrollAdjustmentInX(&self) -> bool {
        self.rare().is_some_and(|rare| {
            rare.bit_field
                .needs_anchor_position_scroll_adjustment_in_x()
        })
    }

    pub fn NeedsAnchorPositionScrollAdjustmentInY(&self) -> bool {
        self.rare().is_some_and(|rare| {
            rare.bit_field
                .needs_anchor_position_scroll_adjustment_in_y()
        })
    }

    // cpp: layoutng_fragment_tree/layout_result.h:204-207
    pub fn GetColumnSpannerPath(&self) -> *const ColumnSpannerPath {
        self.rare()
            .map_or(std::ptr::null(), |rare| rare.column_spanner_path.Get())
    }

    // cpp: layoutng_fragment_tree/layout_result.h:209-219
    pub fn IsEmptySpannerParent(&self) -> bool {
        self.bits()
            .get_flag(LayoutResultBitfields::IS_EMPTY_SPANNER_PARENT)
    }

    // cpp: layoutng_fragment_tree/layout_result.h:221-226
    pub fn GetEarlyBreak(&self) -> *const EarlyBreak {
        self.rare()
            .map_or(std::ptr::null(), |rare| rare.early_break.Get())
    }

    // cpp: layoutng_fragment_tree/layout_result.h:228-235
    pub fn GetExclusionSpace(&self) -> &ExclusionSpace {
        if self
            .bits()
            .get_flag(LayoutResultBitfields::HAS_RARE_DATA_EXCLUSION_SPACE)
        {
            let rare = self
                .rare()
                .expect("rare exclusion space requires rare data");
            return &rare.exclusion_space;
        }
        self.GetConstraintSpaceForCaching().GetExclusionSpace()
    }

    // cpp: layoutng_fragment_tree/layout_result.h:237
    pub fn Status(&self) -> EStatus {
        EStatus(self.bits().get_value(LayoutResultBitfields::STATUS, 4) as u8)
    }

    // cpp: layoutng_fragment_tree/layout_result.h:239-246
    pub fn BfcLineOffset(&self) -> LayoutUnit {
        if self
            .bits()
            .get_flag(LayoutResultBitfields::HAS_OOF_INSETS_FOR_GET_COMPUTED_STYLE)
        {
            debug_assert!(unsafe { &*self.physical_fragment_.Get() }.IsOutOfFlowPositioned());
            return LayoutUnit::default();
        }
        unsafe { (&*self.offset_data_.get()).bfc_offset_.line_offset }
    }

    // cpp: layoutng_fragment_tree/layout_result.h:248-258
    pub fn BfcBlockOffset(&self) -> Option<LayoutUnit> {
        if self
            .bits()
            .get_flag(LayoutResultBitfields::HAS_OOF_INSETS_FOR_GET_COMPUTED_STYLE)
        {
            debug_assert!(unsafe { &*self.physical_fragment_.Get() }.IsOutOfFlowPositioned());
            return Some(LayoutUnit::default());
        }
        if self
            .bits()
            .get_flag(LayoutResultBitfields::IS_BFC_BLOCK_OFFSET_NULLOPT)
        {
            return None;
        }
        Some(unsafe { (&*self.offset_data_.get()).bfc_offset_.block_offset })
    }

    // cpp: layoutng_fragment_tree/layout_result.h:260-286
    pub fn LineBoxBfcBlockOffset(&self) -> Option<LayoutUnit> {
        if self.Status() != EStatus::kSuccess || !self.GetPhysicalFragment().IsLineBox() {
            return None;
        }
        if let Some(rare) = self.rare() {
            if let Some(offset) = rare.LineBoxBfcBlockOffset() {
                return Some(offset);
            }
        }
        self.BfcBlockOffset()
    }

    // cpp: layoutng_fragment_tree/layout_result.h:288-290
    pub fn EndMarginStrut(&self) -> MarginStrut {
        self.rare()
            .map_or(MarginStrut::default(), |rare| rare.end_margin_strut.clone())
    }

    // cpp: layoutng_fragment_tree/layout_result.h:292-309
    pub fn IntrinsicBlockSize(&self) -> LayoutUnit {
        #[cfg(debug_assertions)]
        self.AssertSoleBoxFragment();
        self.intrinsic_block_size_
    }

    // cpp: layoutng_fragment_tree/layout_result.h:311-318
    pub fn ClearanceAfterLine(&self) -> Option<LayoutUnit> {
        self.rare().and_then(|rare| rare.ClearanceAfterLine())
    }

    // cpp: layoutng_fragment_tree/layout_result.h:320-326
    pub fn TrimBlockEndBy(&self) -> Option<LayoutUnit> {
        self.rare().and_then(|rare| rare.TrimBlockEndBy())
    }

    // cpp: layoutng_fragment_tree/layout_result.h:328-333
    pub fn MinimalSpaceShortage(&self) -> Option<LayoutUnit> {
        let rare = self.rare()?;
        if rare.minimal_space_shortage == kIndefiniteSize {
            return None;
        }
        Some(rare.minimal_space_shortage)
    }

    // cpp: layoutng_fragment_tree/layout_result.h:335-340
    pub fn TallestUnbreakableBlockSize(&self) -> LayoutUnit {
        self.rare().map_or(LayoutUnit::default(), |rare| {
            rare.tallest_unbreakable_block_size
        })
    }

    // cpp: layoutng_fragment_tree/layout_result.h:342-354
    pub fn BlockSizeForFragmentation(&self) -> LayoutUnit {
        self.rare()
            .map_or(kIndefiniteSize, |rare| rare.block_size_for_fragmentation)
    }

    // cpp: layoutng_fragment_tree/layout_result.h:356-361
    pub fn IsBlockSizeForFragmentationClamped(&self) -> bool {
        self.bits()
            .get_flag(LayoutResultBitfields::IS_BLOCK_SIZE_FOR_FRAGMENTATION_CLAMPED)
    }

    // cpp: layoutng_fragment_tree/layout_result.h:363-368
    pub fn ShouldForceSameFragmentationFlow(&self) -> bool {
        self.bits()
            .get_flag(LayoutResultBitfields::SHOULD_FORCE_SAME_FRAGMENTATION_FLOW)
    }

    // The numeric values come from the later layoutng::break_appeal enum.
    // cpp: layoutng_fragment_tree/layout_result.h:370-388
    pub fn GetBreakAppeal(&self) -> BreakAppeal {
        BreakAppeal::try_from(
            self.bits()
                .get_value(LayoutResultBitfields::BREAK_APPEAL, 2) as u8,
        )
        .expect("stored break appeal must be valid")
    }

    // cpp: layoutng_fragment_tree/layout_result.h:390-408
    pub fn CustomLayoutData(&self) -> *mut SerializedScriptValue {
        self.rare().map_or(std::ptr::null_mut(), |rare| {
            rare.custom_layout_data.get() as *mut SerializedScriptValue
        })
    }

    pub fn TableColumnCount(&self) -> u32 {
        self.rare().map_or(0, |rare| rare.table_column_count_)
    }

    pub fn GetGridLayoutData(&self) -> *const GridLayoutData {
        self.rare()
            .map_or(std::ptr::null(), |rare| rare.grid_layout_data.Get())
    }

    pub fn FlexLayoutData(&self) -> *const DevtoolsFlexInfo {
        self.rare()
            .map_or(std::ptr::null(), |rare| rare.flex_layout_data_.Get())
    }

    pub fn MathItalicCorrection(&self) -> LayoutUnit {
        self.rare()
            .map_or(LayoutUnit::default(), |rare| rare.math_italic_correction_)
    }

    // cpp: layoutng_fragment_tree/layout_result.h:410-420
    pub fn InitialBreakBefore(&self) -> EBreakBetween {
        EBreakBetween::try_from(
            self.bits()
                .get_value(LayoutResultBitfields::INITIAL_BREAK_BEFORE, 4) as u8,
        )
        .expect("stored break-before value must be valid")
    }

    pub fn FinalBreakAfter(&self) -> EBreakBetween {
        EBreakBetween::try_from(
            self.bits()
                .get_value(LayoutResultBitfields::FINAL_BREAK_AFTER, 4) as u8,
        )
        .expect("stored break-after value must be valid")
    }

    // cpp: layoutng_fragment_tree/layout_result.h:422-431
    pub fn HasForcedBreak(&self) -> bool {
        self.bits()
            .get_flag(LayoutResultBitfields::HAS_FORCED_BREAK)
    }

    pub fn IsSelfCollapsing(&self) -> bool {
        self.bits()
            .get_flag(LayoutResultBitfields::IS_SELF_COLLAPSING)
    }

    pub fn IsPushedByFloats(&self) -> bool {
        self.bits()
            .get_flag(LayoutResultBitfields::IS_PUSHED_BY_FLOATS)
    }

    // cpp: layoutng_fragment_tree/layout_result.h:433-443
    pub fn GetAdjoiningObjectTypes(&self) -> i32 {
        self.bits()
            .get_value(LayoutResultBitfields::ADJOINING_OBJECT_TYPES, 3) as i32
    }

    // cpp: layoutng_fragment_tree/layout_result.h:445-449
    pub fn IsInitialBlockSizeIndefinite(&self) -> bool {
        self.bits()
            .get_flag(LayoutResultBitfields::IS_INITIAL_BLOCK_SIZE_INDEFINITE)
    }

    // cpp: layoutng_fragment_tree/layout_result.h:451-458
    pub fn HasDescendantThatDependsOnPercentageBlockSize(&self) -> bool {
        self.bits()
            .get_flag(LayoutResultBitfields::HAS_DESCENDANT_THAT_DEPENDS_ON_PERCENTAGE_BLOCK_SIZE)
    }

    // cpp: layoutng_fragment_tree/layout_result.h:460-464
    pub fn SubtreeModifiedMarginStrut(&self) -> bool {
        self.bits()
            .get_flag(LayoutResultBitfields::SUBTREE_MODIFIED_MARGIN_STRUT)
    }

    // cpp: layoutng_fragment_tree/layout_result.h:466-471
    pub fn IsTruncatedByFragmentationLine(&self) -> bool {
        self.bits()
            .get_flag(LayoutResultBitfields::IS_TRUNCATED_BY_FRAGMENTATION_LINE)
    }

    // cpp: layoutng_fragment_tree/layout_result.h:473-474
    pub fn GetConstraintSpaceForCaching(&self) -> &ConstraintSpace {
        unsafe { &*self.space_.get() }
    }

    // The source layout_box.cc const_cast mutates this cached constraint
    // space. The result's storage is explicitly interior mutable.
    pub fn ReplaceTableRowDataForCaching(
        &self,
        table_data: Arc<TableConstraintSpaceData>,
        row_index: u32,
    ) {
        unsafe { &mut *self.space_.get() }.ReplaceTableRowData(table_data, row_index);
    }

    // cpp: layoutng_fragment_tree/layout_result.h:476-482
    pub fn DisplayLocksAffectedByAnchors(&self) -> *const GCedHeapHashSet<Member<Element>> {
        self.rare().map_or(std::ptr::null(), |rare| {
            rare.display_locks_affected_by_anchors.Get()
        })
    }

    // cpp: layoutng_fragment_tree/layout_result.h:551-553
    pub fn GetMutableForOutOfFlow(&self) -> MutableForOutOfFlow<'_> {
        MutableForOutOfFlow {
            layout_result_: self,
        }
    }

    // cpp: layoutng_fragment_tree/layout_result.h:572-574
    pub fn GetMutableForLayoutBoxCachedResults(&self) -> MutableForLayoutBoxCachedResults<'_> {
        MutableForLayoutBoxCachedResults {
            layout_result_: self,
        }
    }

    // cpp: layoutng_fragment_tree/layout_result.h:597
    // cpp: layoutng_fragment_tree/layout_result.cc:366-370
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(self.GetConstraintSpaceForCaching());
        visitor.Trace(&self.physical_fragment_);
        visitor.Trace(unsafe { &*self.rare_data_.get() });
    }
}

// A borrowed wrapper mirrors the C++ stack-allocated friend facade. The
// unselected out-of-flow package owns the display-lock setter definition.
// cpp: layoutng_fragment_tree/layout_result.h:484-489
// cpp: layoutng_fragment_tree/layout_result.h:543-549
pub struct MutableForOutOfFlow<'a> {
    layout_result_: &'a LayoutResult,
}

#[allow(non_snake_case)]
impl MutableForOutOfFlow<'_> {
    // cpp: layoutng_out_of_flow/out_of_flow_element_data.cc:49-55
    pub fn SetDisplayLocksAffectedByAnchors(
        &self,
        display_locks: *mut GCedHeapHashSet<Member<Element>>,
    ) {
        let result = self.layout_result_;
        if !unsafe { &*result.rare_data_.get() }.Get().is_null() || !display_locks.is_null() {
            result.EnsureRareData().display_locks_affected_by_anchors =
                Member::from_ptr(display_locks);
        }
    }

    // cpp: layoutng_fragment_tree/layout_result.h:492-501
    pub fn SetOutOfFlowInsetsForGetComputedStyle(&self, insets: &BoxStrut) {
        let result = self.layout_result_;
        debug_assert!(unsafe { &*result.physical_fragment_.Get() }.IsOutOfFlowPositioned());
        debug_assert_eq!(result.BfcLineOffset(), LayoutUnit::default());
        debug_assert_eq!(
            result.BfcBlockOffset().unwrap_or_default(),
            LayoutUnit::default()
        );
        let mut bits = result.bits();
        bits.set_flag(
            LayoutResultBitfields::HAS_OOF_INSETS_FOR_GET_COMPUTED_STYLE,
            true,
        );
        result.bitfields_.set(bits);
        unsafe {
            (*result.offset_data_.get()).oof_insets_for_get_computed_style_ =
                ManuallyDrop::new(*insets);
        }
    }

    // cpp: layoutng_fragment_tree/layout_result.h:503-516
    pub fn SetOutOfFlowPositionedOffset(&self, offset: &LogicalOffset) {
        let result = self.layout_result_;
        assert!(result
            .bits()
            .get_flag(LayoutResultBitfields::HAS_OOF_INSETS_FOR_GET_COMPUTED_STYLE));
        let already_set = result
            .rare()
            .is_some_and(|rare| rare.bit_field.oof_positioned_offset_is_set());
        if already_set || *offset != result.OutOfFlowInsetsForGetComputedStyle().StartOffset() {
            result.EnsureRareData().SetOutOfFlowPositionedOffset(offset);
        }
    }

    // cpp: layoutng_fragment_tree/layout_result.h:518-529
    pub fn SetNeedsScrollAdjustment(&self, needs_x: bool, needs_y: bool) {
        if !needs_x && !needs_y {
            return;
        }
        self.layout_result_
            .EnsureRareData()
            .bit_field
            .set_needs_anchor_position_scroll_adjustment_in_x(needs_x);
        self.layout_result_
            .EnsureRareData()
            .bit_field
            .set_needs_anchor_position_scroll_adjustment_in_y(needs_y);
    }

    // cpp: layoutng_fragment_tree/layout_result.h:531-537
    pub fn SetNonOverflowingScrollRanges(&self, ranges: &HeapVector<NonOverflowingScrollRange>) {
        if self.layout_result_.rare().is_some() || !ranges.is_empty() {
            self.layout_result_
                .EnsureRareData()
                .SetNonOverflowingScrollRanges(ranges);
        }
    }
}

// cpp: layoutng_fragment_tree/layout_result.h:555-570
pub struct MutableForLayoutBoxCachedResults<'a> {
    layout_result_: &'a LayoutResult,
}

#[allow(non_snake_case)]
impl MutableForLayoutBoxCachedResults<'_> {
    // cpp: layoutng_fragment_tree/layout_result.h:562
    // cpp: layoutng_fragment_tree/layout_result.cc:305-311
    pub fn SetFragmentChildrenInvalid(&self) {
        let box_fragment =
            DynamicTo::<PhysicalBoxFragment>(self.layout_result_.physical_fragment_.Get());
        if !box_fragment.is_null() {
            unsafe { &*box_fragment }.SetChildrenInvalid();
        }
    }
}
