#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::sync::Arc;

use foundation::{
    kIndefiniteSize, AtomicString, LayoutUnit, MakeGarbageCollected, MarginStrut, Member, StrCat,
    String as BlinkString, TextDirection, Visitor, WritingDirectionMode, WritingMode,
};
use layoutng_geometry::geometry::bfc_offset::BfcOffset;
use layoutng_geometry::geometry::box_sides::LogicalBoxSides;
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_geometry::geometry::logical_size::LogicalSize;

use super::break_appeal::BreakAppeal;
use super::constraint_space_custom_data::CustomData;
use super::custom_layout_payload::SerializedScriptValue;
use super::exclusions::exclusion_space::ExclusionSpace;
use super::grid_layout_data::GridLayoutSubtree;
use super::line_clamp_data::{LineClampAncestorChain, LineClampData, State as LineClampState};
use super::table_constraint_space_data::TableConstraintSpaceData;

// base::ValuesEquivalent compares identity first, then non-null pointee values.
fn values_equivalent<T: PartialEq>(a: *const T, b: *const T) -> bool {
    if a == b {
        true
    } else if a.is_null() || b.is_null() {
        false
    } else {
        unsafe { &*a == &*b }
    }
}

// cpp: layoutng/internal/constraint_space.h:35-40
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FragmentationType {
    kFragmentNone,
    kFragmentPage,
    kFragmentColumn,
    kFragmentRegion,
}

// cpp: layoutng/internal/constraint_space.h:46-54
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdjoiningObjectTypeValue {
    kAdjoiningNone = 0b000,
    kAdjoiningFloatLeft = 0b001,
    kAdjoiningFloatRight = 0b010,
    kAdjoiningFloatBoth = 0b011,
    kAdjoiningInlineOutOfFlow = 0b100,
}
pub type AdjoiningObjectTypes = i32;

// cpp: layoutng/internal/constraint_space.h:59-67
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaselineAlgorithmType {
    kDefault,
    kInlineBlock,
}

// cpp: layoutng/internal/constraint_space.h:70-81
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoSizeBehavior {
    kFitContent,
    kStretchImplicit,
    kStretchExplicit,
}

// cpp: layoutng/internal/constraint_space.h:89-89
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutResultCacheSlot {
    kLayout,
    kMeasure,
}

// cpp: layoutng/internal/constraint_space.h:92-101
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecorationPercentageResolutionType {
    kContainingBlockInlineSize,
    kContainingBlockSize,
}

// cpp: layoutng/internal/constraint_space.h:1626-1663
// The source stores all common constraints in one unsigned bitfield word.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Bitfields(pub(crate) u32);

impl Bitfields {
    const WRITING_MODE_SHIFT: u32 = 3;
    const DIRECTION_SHIFT: u32 = 6;
    const ORTHOGONAL_ROOT_BIT: u32 = 9;
    const HIDDEN_FOR_PAINT_BIT: u32 = 11;

    // cpp: layoutng/internal/constraint_space.h:1587-1624
    pub(crate) fn MaySkipLayout(self, other: Self) -> bool {
        // The source compares fields through baseline_algorithm_type (bits
        // 0..=19), excluding cache slot and all size constraints.
        const MASK: u32 = (1 << 20) - 1;
        (self.0 & MASK) == (other.0 & MASK)
    }

    pub(crate) fn AreInlineSizeConstraintsEqual(self, other: Self) -> bool {
        const MASK: u32 = (0b11 << 21) | (1 << 25);
        (self.0 & MASK) == (other.0 & MASK)
    }

    pub(crate) fn AreBlockSizeConstraintsEqual(self, other: Self) -> bool {
        const MASK: u32 = (0b11 << 23) | (0b1111 << 26);
        (self.0 & MASK) == (other.0 & MASK)
    }

    // cpp: layoutng/internal/constraint_space.h:1581-1585
    pub(crate) fn new(writing_direction: WritingDirectionMode) -> Self {
        Self(
            ((writing_direction.GetWritingMode() as u32) << Self::WRITING_MODE_SHIFT)
                | ((writing_direction.Direction() as u32) << Self::DIRECTION_SHIFT),
        )
    }

    pub(crate) fn writing_mode(self) -> WritingMode {
        match (self.0 >> Self::WRITING_MODE_SHIFT) & 0b111 {
            0 => WritingMode::kHorizontalTb,
            1 => WritingMode::kVerticalRl,
            2 => WritingMode::kVerticalLr,
            3 => WritingMode::kSidewaysRl,
            4 => WritingMode::kSidewaysLr,
            _ => unreachable!("invalid writing mode bitfield"),
        }
    }

    pub(crate) fn direction(self) -> TextDirection {
        if (self.0 >> Self::DIRECTION_SHIFT) & 1 == 0 {
            TextDirection::kLtr
        } else {
            TextDirection::kRtl
        }
    }

    pub(crate) fn is_orthogonal_writing_mode_root(self) -> bool {
        self.0 & (1 << Self::ORTHOGONAL_ROOT_BIT) != 0
    }

    pub(crate) fn is_hidden_for_paint(self) -> bool {
        self.0 & (1 << Self::HIDDEN_FOR_PAINT_BIT) != 0
    }

    pub(crate) fn bit(self, index: u32) -> bool {
        self.0 & (1 << index) != 0
    }

    pub(crate) fn set_bit(&mut self, index: u32, value: bool) {
        let mask = 1 << index;
        self.0 = (self.0 & !mask) | ((value as u32) << index);
    }

    pub(crate) fn set_bits(&mut self, shift: u32, width: u32, value: u32) {
        let mask = ((1 << width) - 1) << shift;
        debug_assert_eq!(value << shift & !mask, 0);
        self.0 = (self.0 & !mask) | ((value << shift) & mask);
    }

    fn auto_behavior(self, shift: u32) -> AutoSizeBehavior {
        match (self.0 >> shift) & 0b11 {
            0 => AutoSizeBehavior::kFitContent,
            1 => AutoSizeBehavior::kStretchImplicit,
            2 => AutoSizeBehavior::kStretchExplicit,
            _ => unreachable!("invalid auto size behavior bitfield"),
        }
    }
}

// cpp: layoutng/internal/constraint_space.h:1408-1413
#[derive(Clone)]
pub struct BlockData {
    pub margin_strut: MarginStrut,
    pub optimistic_bfc_block_offset: Option<LayoutUnit>,
    pub forced_bfc_block_offset: Option<LayoutUnit>,
    pub clearance_offset: LayoutUnit,
    pub previous_sibling_block_end_annotation_space: LayoutUnit,
    pub line_clamp_data: LineClampData,
}

impl Default for BlockData {
    fn default() -> Self {
        Self {
            margin_strut: MarginStrut::default(),
            optimistic_bfc_block_offset: None,
            forced_bfc_block_offset: None,
            clearance_offset: LayoutUnit::Min(),
            previous_sibling_block_end_annotation_space: LayoutUnit::default(),
            line_clamp_data: LineClampData::default(),
        }
    }
}

impl BlockData {
    // cpp: layoutng/internal/constraint_space.h:1396-1406
    pub fn MaySkipLayout(&self, other: &Self) -> bool {
        self.line_clamp_data == other.line_clamp_data
            && self.previous_sibling_block_end_annotation_space
                == other.previous_sibling_block_end_annotation_space
    }

    pub fn IsInitialForMaySkipLayout(&self) -> bool {
        self.line_clamp_data.state == LineClampState::kDisabled
            && self.previous_sibling_block_end_annotation_space == LayoutUnit::default()
    }
}

// cpp: layoutng/internal/constraint_space.h:1430-1433
#[derive(Clone)]
pub struct TableCellData {
    pub table_cell_borders: BoxStrut,
    pub table_cell_column_index: u32,
    pub table_cell_alignment_baseline: Option<LayoutUnit>,
    pub has_collapsed_borders: bool,
}

impl Default for TableCellData {
    fn default() -> Self {
        Self {
            table_cell_borders: BoxStrut::default(),
            table_cell_column_index: u32::MAX,
            table_cell_alignment_baseline: None,
            has_collapsed_borders: false,
        }
    }
}

impl TableCellData {
    // cpp: layoutng/internal/constraint_space.h:1417-1429
    pub fn MaySkipLayout(&self, other: &Self) -> bool {
        self.table_cell_borders == other.table_cell_borders
            && self.table_cell_column_index == other.table_cell_column_index
            && self.has_collapsed_borders == other.has_collapsed_borders
    }

    pub fn IsInitialForMaySkipLayout(&self) -> bool {
        self.table_cell_borders == BoxStrut::default()
            && self.table_cell_column_index == u32::MAX
            && !self.has_collapsed_borders
    }
}

// cpp: layoutng/internal/constraint_space.h:1436-1451
#[derive(Clone)]
pub struct TableRowData {
    pub table_data: Option<Arc<TableConstraintSpaceData>>,
    pub row_index: u32,
}

impl Default for TableRowData {
    fn default() -> Self {
        Self {
            table_data: None,
            row_index: u32::MAX,
        }
    }
}

impl TableRowData {
    pub fn MaySkipLayout(&self, other: &Self) -> bool {
        let (Some(table), Some(other_table)) = (&self.table_data, &other.table_data) else {
            return false;
        };
        table.IsTableSpecificDataEqual(other_table)
            && table.MaySkipRowLayout(other_table, self.row_index, other.row_index)
    }

    pub fn IsInitialForMaySkipLayout(&self) -> bool {
        self.table_data.is_none() && self.row_index == u32::MAX
    }
}

// cpp: layoutng/internal/constraint_space.h:1452-1471
#[derive(Clone)]
pub struct TableSectionData {
    pub table_data: Option<Arc<TableConstraintSpaceData>>,
    pub section_index: u32,
}

impl Default for TableSectionData {
    fn default() -> Self {
        Self {
            table_data: None,
            section_index: u32::MAX,
        }
    }
}

impl TableSectionData {
    pub fn MaySkipLayout(&self, other: &Self) -> bool {
        let (Some(table), Some(other_table)) = (&self.table_data, &other.table_data) else {
            return false;
        };
        table.IsTableSpecificDataEqual(other_table)
            && table.MaySkipSectionLayout(other_table, self.section_index, other.section_index)
    }

    pub fn IsInitialForMaySkipLayout(&self) -> bool {
        self.table_data.is_none() && self.section_index == u32::MAX
    }
}

// cpp: layoutng/internal/constraint_space.h:258-261
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MathTargetStretchBlockSizes {
    pub ascent: LayoutUnit,
    pub descent: LayoutUnit,
}

// cpp: layoutng/internal/constraint_space.h:1477-1496
#[derive(Clone)]
pub struct StretchData {
    pub target_stretch_inline_size: LayoutUnit,
    pub target_stretch_block_sizes: Option<MathTargetStretchBlockSizes>,
}

impl Default for StretchData {
    fn default() -> Self {
        Self {
            target_stretch_inline_size: kIndefiniteSize,
            target_stretch_block_sizes: None,
        }
    }
}

impl StretchData {
    pub fn MaySkipLayout(&self, other: &Self) -> bool {
        self.target_stretch_inline_size == other.target_stretch_inline_size
            && self.target_stretch_block_sizes == other.target_stretch_block_sizes
    }

    pub fn IsInitialForMaySkipLayout(&self) -> bool {
        self.target_stretch_inline_size == kIndefiniteSize
            && self.target_stretch_block_sizes.is_none()
    }
}

// cpp: layoutng/internal/constraint_space.h:880-890
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataUnionType {
    kNone,
    kBlockData,
    kTableCellData,
    kTableRowData,
    kTableSectionData,
    kCustomData,
    kStretchData,
}

// cpp: layoutng/internal/constraint_space.h:1559-1566
#[derive(Clone, Default)]
enum RareDataPayload {
    #[default]
    None,
    Block(BlockData),
    TableCell(TableCellData),
    TableRow(TableRowData),
    TableSection(TableSectionData),
    Custom(CustomData),
    Stretch(StretchData),
}

// cpp: layoutng/internal/constraint_space.h:1344-1393
// The uncommon values are GC-owned separately from the one-word common flags.
// The source's mutually exclusive data union and its accessors follow in this
// in-progress translation.
#[derive(Clone)]
pub struct RareData {
    pub percentage_resolution_size: LogicalSize,
    pub block_start_annotation_space: LayoutUnit,
    pub replaced_child_percentage_resolution_block_size: LayoutUnit,
    pub page_name: AtomicString,
    pub fragmentainer_block_size: LayoutUnit,
    pub fragmentainer_offset: LayoutUnit,
    pub safe_printable_inset: LayoutUnit,
    pub is_pushed_by_floats: bool,
    pub is_restricted_block_size_table_cell: bool,
    pub hide_table_cell_if_empty: bool,
    pub block_direction_fragmentation_type: FragmentationType,
    pub is_block_fragmentation_forced_off: bool,
    pub is_monolithic_overflow_propagation_disabled: bool,
    pub requires_content_before_breaking: bool,
    pub is_inside_balanced_columns: bool,
    pub should_ignore_forced_breaks: bool,
    pub is_in_column_bfc: bool,
    pub is_past_break: bool,
    pub min_block_size_should_encompass_intrinsic_size: bool,
    pub min_break_appeal: BreakAppeal,
    pub propagate_child_break_values: bool,
    pub is_at_fragmentainer_start: bool,
    pub should_repeat: bool,
    pub is_inside_repeatable_content: bool,
    pub is_inside_break_avoid: bool,
    pub uses_orthogonal_fallback_inline_size: bool,
    pub should_text_box_trim_node_start: bool,
    pub should_text_box_trim_node_end: bool,
    pub should_text_box_trim_fragmentainer_start: bool,
    pub should_text_box_trim_fragmentainer_end: bool,
    pub should_force_text_box_trim_end: bool,
    pub should_text_box_trim_inside_when_line_clamp: bool,
    pub should_force_margin_trim_end: bool,
    pub decoration_percentage_resolution_type: DecorationPercentageResolutionType,
    pub is_adjacent_to_paper_edge_inline_start: bool,
    pub is_adjacent_to_paper_edge_inline_end: bool,
    pub is_adjacent_to_paper_edge_block_start: bool,
    pub is_adjacent_to_paper_edge_block_end: bool,
    pub line_clamp_ancestor_chain_: Member<LineClampAncestorChain>,
    pub grid_layout_subtree_: Member<GridLayoutSubtree>,
    payload: RareDataPayload,
}

impl Default for RareData {
    // cpp: layoutng/internal/constraint_space.h:892-892
    // cpp: layoutng/internal/constraint_space.h:1344-1393
    fn default() -> Self {
        Self {
            percentage_resolution_size: LogicalSize::default(),
            block_start_annotation_space: LayoutUnit::default(),
            replaced_child_percentage_resolution_block_size: kIndefiniteSize,
            page_name: AtomicString::default(),
            fragmentainer_block_size: kIndefiniteSize,
            fragmentainer_offset: LayoutUnit::default(),
            safe_printable_inset: LayoutUnit::default(),
            is_pushed_by_floats: false,
            is_restricted_block_size_table_cell: false,
            hide_table_cell_if_empty: false,
            block_direction_fragmentation_type: FragmentationType::kFragmentNone,
            is_block_fragmentation_forced_off: false,
            is_monolithic_overflow_propagation_disabled: false,
            requires_content_before_breaking: false,
            is_inside_balanced_columns: false,
            should_ignore_forced_breaks: false,
            is_in_column_bfc: false,
            is_past_break: false,
            min_block_size_should_encompass_intrinsic_size: false,
            min_break_appeal: BreakAppeal::kBreakAppealLastResort,
            propagate_child_break_values: false,
            is_at_fragmentainer_start: false,
            should_repeat: false,
            is_inside_repeatable_content: false,
            is_inside_break_avoid: false,
            uses_orthogonal_fallback_inline_size: false,
            should_text_box_trim_node_start: false,
            should_text_box_trim_node_end: false,
            should_text_box_trim_fragmentainer_start: false,
            should_text_box_trim_fragmentainer_end: false,
            should_force_text_box_trim_end: false,
            should_text_box_trim_inside_when_line_clamp: false,
            should_force_margin_trim_end: false,
            decoration_percentage_resolution_type:
                DecorationPercentageResolutionType::kContainingBlockInlineSize,
            is_adjacent_to_paper_edge_inline_start: false,
            is_adjacent_to_paper_edge_inline_end: false,
            is_adjacent_to_paper_edge_block_start: false,
            is_adjacent_to_paper_edge_block_end: false,
            line_clamp_ancestor_chain_: Member::default(),
            grid_layout_subtree_: Member::default(),
            payload: RareDataPayload::None,
        }
    }
}

impl RareData {
    // cpp: layoutng/internal/constraint_space.h:1007-1010
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.line_clamp_ancestor_chain_);
        visitor.Trace(&self.grid_layout_subtree_);
    }

    // cpp: layoutng/internal/constraint_space.h:1012-1084
    pub fn MaySkipLayout(&self, other: &Self) -> bool {
        if self.replaced_child_percentage_resolution_block_size
            != other.replaced_child_percentage_resolution_block_size
            || self.GetDataUnionType() != other.GetDataUnionType()
            || self.is_pushed_by_floats != other.is_pushed_by_floats
            || self.is_restricted_block_size_table_cell != other.is_restricted_block_size_table_cell
            || self.hide_table_cell_if_empty != other.hide_table_cell_if_empty
            || self.block_direction_fragmentation_type != other.block_direction_fragmentation_type
            || self.is_block_fragmentation_forced_off != other.is_block_fragmentation_forced_off
            || self.is_monolithic_overflow_propagation_disabled
                != other.is_monolithic_overflow_propagation_disabled
            || self.requires_content_before_breaking != other.requires_content_before_breaking
            || self.is_inside_balanced_columns != other.is_inside_balanced_columns
            || self.should_ignore_forced_breaks != other.should_ignore_forced_breaks
            || self.is_in_column_bfc != other.is_in_column_bfc
            || self.is_past_break != other.is_past_break
            || self.min_break_appeal != other.min_break_appeal
            || self.propagate_child_break_values != other.propagate_child_break_values
            || self.should_repeat != other.should_repeat
            || self.is_inside_repeatable_content != other.is_inside_repeatable_content
            || self.is_inside_break_avoid != other.is_inside_break_avoid
            || self.should_text_box_trim_node_start != other.should_text_box_trim_node_start
            || self.should_text_box_trim_node_end != other.should_text_box_trim_node_end
            || self.should_text_box_trim_fragmentainer_start
                != other.should_text_box_trim_fragmentainer_start
            || self.should_text_box_trim_fragmentainer_end
                != other.should_text_box_trim_fragmentainer_end
            || self.should_force_text_box_trim_end != other.should_force_text_box_trim_end
            || self.should_text_box_trim_inside_when_line_clamp
                != other.should_text_box_trim_inside_when_line_clamp
            || self.should_force_margin_trim_end != other.should_force_margin_trim_end
            || self.decoration_percentage_resolution_type
                != other.decoration_percentage_resolution_type
            || self.safe_printable_inset != other.safe_printable_inset
            || self.is_adjacent_to_paper_edge_inline_start
                != other.is_adjacent_to_paper_edge_inline_start
            || self.is_adjacent_to_paper_edge_inline_end
                != other.is_adjacent_to_paper_edge_inline_end
            || self.is_adjacent_to_paper_edge_block_start
                != other.is_adjacent_to_paper_edge_block_start
            || self.is_adjacent_to_paper_edge_block_end != other.is_adjacent_to_paper_edge_block_end
            || !values_equivalent(
                self.line_clamp_ancestor_chain_.Get(),
                other.line_clamp_ancestor_chain_.Get(),
            )
            || !values_equivalent(
                self.grid_layout_subtree_.Get(),
                other.grid_layout_subtree_.Get(),
            )
        {
            return false;
        }
        match (&self.payload, &other.payload) {
            (RareDataPayload::None, RareDataPayload::None) => true,
            (RareDataPayload::Block(a), RareDataPayload::Block(b)) => a.MaySkipLayout(b),
            (RareDataPayload::TableCell(a), RareDataPayload::TableCell(b)) => a.MaySkipLayout(b),
            (RareDataPayload::TableRow(a), RareDataPayload::TableRow(b)) => a.MaySkipLayout(b),
            (RareDataPayload::TableSection(a), RareDataPayload::TableSection(b)) => {
                a.MaySkipLayout(b)
            }
            (RareDataPayload::Custom(a), RareDataPayload::Custom(b)) => a.MaySkipLayout(b),
            (RareDataPayload::Stretch(a), RareDataPayload::Stretch(b)) => a.MaySkipLayout(b),
            _ => unreachable!("incompatible RareData union arms"),
        }
    }

    // cpp: layoutng/internal/constraint_space.h:1086-1133
    pub fn IsInitialForMaySkipLayout(&self) -> bool {
        if self.replaced_child_percentage_resolution_block_size != kIndefiniteSize
            || !self.page_name.IsNull()
            || self.fragmentainer_block_size != kIndefiniteSize
            || self.fragmentainer_offset != LayoutUnit::default()
            || self.safe_printable_inset != LayoutUnit::default()
            || self.is_pushed_by_floats
            || self.is_restricted_block_size_table_cell
            || self.hide_table_cell_if_empty
            || self.block_direction_fragmentation_type != FragmentationType::kFragmentNone
            || self.is_block_fragmentation_forced_off
            || self.is_monolithic_overflow_propagation_disabled
            || self.requires_content_before_breaking
            || self.is_inside_balanced_columns
            || self.should_ignore_forced_breaks
            || self.is_in_column_bfc
            || self.is_past_break
            || self.min_break_appeal != BreakAppeal::kBreakAppealLastResort
            || self.propagate_child_break_values
            || self.is_at_fragmentainer_start
            || self.should_repeat
            || self.is_inside_repeatable_content
            || self.is_inside_break_avoid
            || self.should_text_box_trim_node_start
            || self.should_text_box_trim_node_end
            || self.should_text_box_trim_fragmentainer_start
            || self.should_text_box_trim_fragmentainer_end
            || self.should_force_text_box_trim_end
            || self.should_text_box_trim_inside_when_line_clamp
            || self.should_force_margin_trim_end
            || self.decoration_percentage_resolution_type
                != DecorationPercentageResolutionType::kContainingBlockInlineSize
            || self.is_adjacent_to_paper_edge_inline_start
            || self.is_adjacent_to_paper_edge_inline_end
            || self.is_adjacent_to_paper_edge_block_start
            || self.is_adjacent_to_paper_edge_block_end
            || !self.line_clamp_ancestor_chain_.Get().is_null()
            || !self.grid_layout_subtree_.Get().is_null()
        {
            return false;
        }
        match &self.payload {
            RareDataPayload::None => true,
            RareDataPayload::Block(data) => data.IsInitialForMaySkipLayout(),
            RareDataPayload::TableCell(data) => data.IsInitialForMaySkipLayout(),
            RareDataPayload::TableRow(data) => data.IsInitialForMaySkipLayout(),
            RareDataPayload::TableSection(data) => data.IsInitialForMaySkipLayout(),
            RareDataPayload::Custom(data) => data.IsInitialForMaySkipLayout(),
            RareDataPayload::Stretch(data) => data.IsInitialForMaySkipLayout(),
        }
    }

    // cpp: layoutng/internal/constraint_space.h:1340-1342
    pub fn GetDataUnionType(&self) -> DataUnionType {
        match &self.payload {
            RareDataPayload::None => DataUnionType::kNone,
            RareDataPayload::Block(_) => DataUnionType::kBlockData,
            RareDataPayload::TableCell(_) => DataUnionType::kTableCellData,
            RareDataPayload::TableRow(_) => DataUnionType::kTableRowData,
            RareDataPayload::TableSection(_) => DataUnionType::kTableSectionData,
            RareDataPayload::Custom(_) => DataUnionType::kCustomData,
            RareDataPayload::Stretch(_) => DataUnionType::kStretchData,
        }
    }

    // cpp: layoutng/internal/constraint_space.h:1498-1506
    pub fn EnsureBlockData(&mut self) -> &mut BlockData {
        debug_assert!(matches!(
            &self.payload,
            RareDataPayload::None | RareDataPayload::Block(_)
        ));
        if matches!(&self.payload, RareDataPayload::None) {
            self.payload = RareDataPayload::Block(BlockData::default());
        }
        match &mut self.payload {
            RareDataPayload::Block(data) => data,
            _ => panic!("incompatible RareData union arm"),
        }
    }

    // cpp: layoutng/internal/constraint_space.h:1508-1516
    pub fn EnsureTableCellData(&mut self) -> &mut TableCellData {
        debug_assert!(matches!(
            &self.payload,
            RareDataPayload::None | RareDataPayload::TableCell(_)
        ));
        if matches!(&self.payload, RareDataPayload::None) {
            self.payload = RareDataPayload::TableCell(TableCellData::default());
        }
        match &mut self.payload {
            RareDataPayload::TableCell(data) => data,
            _ => panic!("incompatible RareData union arm"),
        }
    }

    // cpp: layoutng/internal/constraint_space.h:1518-1526
    pub fn EnsureTableRowData(&mut self) -> &mut TableRowData {
        debug_assert!(matches!(
            &self.payload,
            RareDataPayload::None | RareDataPayload::TableRow(_)
        ));
        if matches!(&self.payload, RareDataPayload::None) {
            self.payload = RareDataPayload::TableRow(TableRowData::default());
        }
        match &mut self.payload {
            RareDataPayload::TableRow(data) => data,
            _ => panic!("incompatible RareData union arm"),
        }
    }

    // cpp: layoutng/internal/constraint_space.h:1528-1537
    pub fn EnsureTableSectionData(&mut self) -> &mut TableSectionData {
        debug_assert!(matches!(
            &self.payload,
            RareDataPayload::None | RareDataPayload::TableSection(_)
        ));
        if matches!(&self.payload, RareDataPayload::None) {
            self.payload = RareDataPayload::TableSection(TableSectionData::default());
        }
        match &mut self.payload {
            RareDataPayload::TableSection(data) => data,
            _ => panic!("incompatible RareData union arm"),
        }
    }

    // cpp: layoutng/internal/constraint_space.h:1539-1547
    pub fn EnsureCustomData(&mut self) -> *mut CustomData {
        debug_assert!(matches!(
            &self.payload,
            RareDataPayload::None | RareDataPayload::Custom(_)
        ));
        if matches!(&self.payload, RareDataPayload::None) {
            self.payload = RareDataPayload::Custom(CustomData::default());
        }
        match &mut self.payload {
            RareDataPayload::Custom(data) => data,
            _ => panic!("incompatible RareData union arm"),
        }
    }

    // cpp: layoutng/internal/constraint_space.h:1549-1557
    pub fn EnsureStretchData(&mut self) -> &mut StretchData {
        debug_assert!(matches!(
            &self.payload,
            RareDataPayload::None | RareDataPayload::Stretch(_)
        ));
        if matches!(&self.payload, RareDataPayload::None) {
            self.payload = RareDataPayload::Stretch(StretchData::default());
        }
        match &mut self.payload {
            RareDataPayload::Stretch(data) => data,
            _ => panic!("incompatible RareData union arm"),
        }
    }

    // cpp: layoutng/internal/constraint_space.h:1135-1141
    pub fn BlockStartAnnotationSpace(&self) -> LayoutUnit {
        self.block_start_annotation_space
    }

    pub fn SetBlockStartAnnotationSpace(&mut self, space: LayoutUnit) {
        self.block_start_annotation_space = space;
    }

    // cpp: layoutng/internal/constraint_space.h:1143-1151
    pub fn PreviousSiblingBlockEndAnnotationSpace(&self) -> LayoutUnit {
        match &self.payload {
            RareDataPayload::Block(data) => data.previous_sibling_block_end_annotation_space,
            _ => LayoutUnit::default(),
        }
    }

    pub fn SetPreviousSiblingBlockEndAnnotationSpace(&mut self, space: LayoutUnit) {
        self.EnsureBlockData()
            .previous_sibling_block_end_annotation_space = space;
    }

    // cpp: layoutng/internal/constraint_space.h:1153-1161
    pub fn GetMarginStrut(&self) -> MarginStrut {
        match &self.payload {
            RareDataPayload::Block(data) => data.margin_strut.clone(),
            _ => MarginStrut::default(),
        }
    }

    pub fn SetMarginStrut(&mut self, margin_strut: &MarginStrut) {
        self.EnsureBlockData().margin_strut = margin_strut.clone();
    }

    // cpp: layoutng/internal/constraint_space.h:1163-1172
    pub fn OptimisticBfcBlockOffset(&self) -> Option<LayoutUnit> {
        match &self.payload {
            RareDataPayload::Block(data) => data.optimistic_bfc_block_offset,
            _ => None,
        }
    }

    pub fn SetOptimisticBfcBlockOffset(&mut self, offset: LayoutUnit) {
        self.EnsureBlockData().optimistic_bfc_block_offset = Some(offset);
    }

    // cpp: layoutng/internal/constraint_space.h:1174-1182
    pub fn ForcedBfcBlockOffset(&self) -> Option<LayoutUnit> {
        match &self.payload {
            RareDataPayload::Block(data) => data.forced_bfc_block_offset,
            _ => None,
        }
    }

    pub fn SetForcedBfcBlockOffset(&mut self, offset: LayoutUnit) {
        self.EnsureBlockData().forced_bfc_block_offset = Some(offset);
    }

    // cpp: layoutng/internal/constraint_space.h:1184-1192
    pub fn ClearanceOffset(&self) -> LayoutUnit {
        match &self.payload {
            RareDataPayload::Block(data) => data.clearance_offset,
            _ => LayoutUnit::Min(),
        }
    }

    pub fn SetClearanceOffset(&mut self, offset: LayoutUnit) {
        self.EnsureBlockData().clearance_offset = offset;
    }

    // cpp: layoutng/internal/constraint_space.h:1194-1202
    pub fn GetLineClampData(&self) -> LineClampData {
        match &self.payload {
            RareDataPayload::Block(data) => data.line_clamp_data.clone(),
            _ => LineClampData::default(),
        }
    }

    pub fn SetLineClampData(&mut self, value: LineClampData) {
        self.EnsureBlockData().line_clamp_data = value;
    }

    // cpp: layoutng/internal/constraint_space.h:1204-1210
    pub fn GetLineClampAncestorChain(&self) -> *const LineClampAncestorChain {
        self.line_clamp_ancestor_chain_.Get()
    }

    pub fn SetLineClampAncestorChain(&mut self, chain: *const LineClampAncestorChain) {
        self.line_clamp_ancestor_chain_ = Member::from_ptr(chain.cast_mut());
    }

    // cpp: layoutng/internal/constraint_space.h:1212-1222
    pub fn SetIsTableCell(&mut self) {
        self.EnsureTableCellData();
    }

    pub fn TableCellBorders(&self) -> BoxStrut {
        match &self.payload {
            RareDataPayload::TableCell(data) => data.table_cell_borders,
            _ => BoxStrut::default(),
        }
    }

    pub fn SetTableCellBorders(&mut self, borders: &BoxStrut) {
        self.EnsureTableCellData().table_cell_borders = *borders;
    }

    // cpp: layoutng/internal/constraint_space.h:1224-1232
    pub fn TableCellColumnIndex(&self) -> u32 {
        match &self.payload {
            RareDataPayload::TableCell(data) => data.table_cell_column_index,
            _ => 0,
        }
    }

    pub fn SetTableCellColumnIndex(&mut self, index: u32) {
        self.EnsureTableCellData().table_cell_column_index = index;
    }

    // cpp: layoutng/internal/constraint_space.h:1234-1244
    pub fn TableCellAlignmentBaseline(&self) -> Option<LayoutUnit> {
        match &self.payload {
            RareDataPayload::TableCell(data) => data.table_cell_alignment_baseline,
            _ => None,
        }
    }

    pub fn SetTableCellAlignmentBaseline(&mut self, baseline: LayoutUnit) {
        self.EnsureTableCellData().table_cell_alignment_baseline = Some(baseline);
    }

    // cpp: layoutng/internal/constraint_space.h:1246-1253
    pub fn IsTableCellWithCollapsedBorders(&self) -> bool {
        match &self.payload {
            RareDataPayload::TableCell(data) => data.has_collapsed_borders,
            _ => false,
        }
    }

    pub fn SetIsTableCellWithCollapsedBorders(&mut self, collapsed: bool) {
        self.EnsureTableCellData().has_collapsed_borders = collapsed;
    }

    // cpp: layoutng/internal/constraint_space.h:1255-1267
    pub fn SetTableRowData(&mut self, table_data: Arc<TableConstraintSpaceData>, row_index: u32) {
        let data = self.EnsureTableRowData();
        data.table_data = Some(table_data);
        data.row_index = row_index;
    }

    pub fn SetTableSectionData(
        &mut self,
        table_data: Arc<TableConstraintSpaceData>,
        section_index: u32,
    ) {
        let data = self.EnsureTableSectionData();
        data.table_data = Some(table_data);
        data.section_index = section_index;
    }

    // cpp: layoutng/internal/constraint_space.h:1269-1278
    // Arc retains the source scoped_refptr when a table is replaced.
    pub fn ReplaceTableRowData(
        &mut self,
        table_data: Arc<TableConstraintSpaceData>,
        row_index: u32,
    ) {
        let RareDataPayload::TableRow(data) = &mut self.payload else {
            panic!("ReplaceTableRowData requires table-row data");
        };
        let previous = data.table_data.as_ref().expect("table-row data must exist");
        debug_assert!(table_data.IsTableSpecificDataEqual(previous));
        debug_assert!(table_data.MaySkipRowLayout(previous, row_index, data.row_index));
        data.table_data = Some(table_data);
        data.row_index = row_index;
    }

    // cpp: layoutng/internal/constraint_space.h:1280-1298
    pub fn TableData(&self) -> *const TableConstraintSpaceData {
        match &self.payload {
            RareDataPayload::TableRow(data) => data
                .table_data
                .as_deref()
                .map_or(std::ptr::null(), |value| value),
            RareDataPayload::TableSection(data) => data
                .table_data
                .as_deref()
                .map_or(std::ptr::null(), |value| value),
            _ => std::ptr::null(),
        }
    }

    // C++ copies a scoped_refptr from TableData() when replacing row data.
    // Rust must clone the existing owning Arc rather than copy the pointee.
    // cpp: layoutng/internal/constraint_space.h:1269-1278,1436-1460
    fn CloneTableDataHandle(&self) -> Arc<TableConstraintSpaceData> {
        match &self.payload {
            RareDataPayload::TableRow(data) => data.table_data.as_ref(),
            RareDataPayload::TableSection(data) => data.table_data.as_ref(),
            _ => None,
        }
        .expect("table data must exist")
        .clone()
    }

    pub fn TableRowIndex(&self) -> u32 {
        match &self.payload {
            RareDataPayload::TableRow(data) => data.row_index,
            _ => u32::MAX,
        }
    }

    pub fn TableSectionIndex(&self) -> u32 {
        match &self.payload {
            RareDataPayload::TableSection(data) => data.section_index,
            _ => u32::MAX,
        }
    }

    // cpp: layoutng/internal/constraint_space.h:1300-1304
    pub fn CustomLayoutData(&self) -> *mut SerializedScriptValue {
        match &self.payload {
            RareDataPayload::Custom(data) => {
                data.data.as_ref().map_or(std::ptr::null_mut(), |value| {
                    Arc::as_ptr(value) as *mut SerializedScriptValue
                })
            }
            _ => std::ptr::null_mut(),
        }
    }

    // cpp: layoutng/internal/constraint_space.h:1309-1330
    pub fn TargetStretchInlineSize(&self) -> LayoutUnit {
        match &self.payload {
            RareDataPayload::Stretch(data) => data.target_stretch_inline_size,
            _ => kIndefiniteSize,
        }
    }

    pub fn SetTargetStretchInlineSize(&mut self, size: LayoutUnit) {
        self.EnsureStretchData().target_stretch_inline_size = size;
    }

    pub fn TargetStretchBlockSizes(&self) -> Option<MathTargetStretchBlockSizes> {
        match &self.payload {
            RareDataPayload::Stretch(data) => data.target_stretch_block_sizes,
            _ => None,
        }
    }

    pub fn SetTargetStretchBlockSizes(&mut self, sizes: MathTargetStretchBlockSizes) {
        self.EnsureStretchData().target_stretch_block_sizes = Some(sizes);
    }

    // cpp: layoutng/internal/constraint_space.h:1332-1338
    pub fn GetGridLayoutSubtree(&self) -> *const GridLayoutSubtree {
        self.grid_layout_subtree_.Get()
    }

    pub fn SetGridLayoutSubtree(&mut self, subtree: *const GridLayoutSubtree) {
        self.grid_layout_subtree_ = Member::from_ptr(subtree.cast_mut());
    }
}

// cpp: layoutng/internal/constraint_space.h:105-126
// cpp: layoutng/internal/constraint_space.h:1670-1676
#[repr(C)]
pub struct ConstraintSpace {
    pub(crate) available_size_: LogicalSize,
    pub(crate) percentage_size_: LogicalSize,
    pub(crate) bfc_offset_: BfcOffset,
    pub(crate) exclusion_space_: ExclusionSpace,
    pub(crate) rare_data_: Member<RareData>,
    pub(crate) bitfields_: Bitfields,
}

impl Clone for ConstraintSpace {
    // cpp: layoutng/internal/constraint_space.h:115-121
    fn clone(&self) -> Self {
        Self {
            available_size_: self.available_size_,
            percentage_size_: self.percentage_size_,
            bfc_offset_: self.bfc_offset_,
            exclusion_space_: self.exclusion_space_.clone(),
            rare_data_: self.rare_data_,
            bitfields_: self.bitfields_,
        }
    }
}

impl ConstraintSpace {
    // cpp: layoutng/internal/constraint_space.h:806-808
    pub fn GetGridLayoutSubtree(&self) -> *const GridLayoutSubtree {
        let data = self.rare_data_.Get();
        if data.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*data }.GetGridLayoutSubtree()
        }
    }

    // cpp: layoutng/internal/constraint_space.h:1665-1668
    pub fn new(writing_direction: WritingDirectionMode) -> Self {
        Self {
            available_size_: LogicalSize::new(kIndefiniteSize, kIndefiniteSize),
            percentage_size_: LogicalSize::new(kIndefiniteSize, kIndefiniteSize),
            bfc_offset_: BfcOffset::default(),
            exclusion_space_: ExclusionSpace::default(),
            rare_data_: Member::default(),
            bitfields_: Bitfields::new(writing_direction),
        }
    }

    // cpp: layoutng/internal/constraint_space.h:132-135
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.exclusion_space_);
        visitor.Trace(&self.rare_data_);
    }

    // cpp: layoutng/internal/constraint_space.h:142-152
    pub fn CloneWithoutFragmentation(&self) -> Self {
        debug_assert!(self.HasBlockFragmentation());
        let mut copy = self.clone();
        let rare_data = copy.rare_data_.Get();
        debug_assert!(!rare_data.is_null());
        let mut cloned_rare_data = unsafe { &*rare_data }.clone();
        cloned_rare_data.block_direction_fragmentation_type = FragmentationType::kFragmentNone;
        cloned_rare_data.is_block_fragmentation_forced_off = true;
        copy.rare_data_ = Member::from_ptr(MakeGarbageCollected(cloned_rare_data));
        copy
    }

    // cpp: layoutng/internal/constraint_space.h:154
    pub fn GetExclusionSpace(&self) -> &ExclusionSpace {
        &self.exclusion_space_
    }

    // cpp: layoutng/internal/constraint_space.h:156-169
    pub fn Direction(&self) -> TextDirection {
        self.bitfields_.direction()
    }

    pub fn GetWritingMode(&self) -> WritingMode {
        self.bitfields_.writing_mode()
    }

    pub fn GetWritingDirection(&self) -> WritingDirectionMode {
        WritingDirectionMode::new(self.GetWritingMode(), self.Direction())
    }

    // cpp: layoutng/internal/constraint_space.h:171-175
    pub fn IsOrthogonalWritingModeRoot(&self) -> bool {
        self.bitfields_.is_orthogonal_writing_mode_root()
    }

    pub fn IsHiddenForPaint(&self) -> bool {
        self.bitfields_.is_hidden_for_paint()
    }

    // cpp: layoutng/internal/constraint_space.h:380-382
    pub fn IsNewFormattingContext(&self) -> bool {
        self.bitfields_.bit(8)
    }

    // cpp: layoutng/internal/constraint_space.h:400-400
    pub fn IsAnonymous(&self) -> bool {
        self.bitfields_.bit(7)
    }

    // cpp: layoutng/internal/constraint_space.h:407-407
    pub fn UseFirstLineStyle(&self) -> bool {
        self.bitfields_.bit(12)
    }

    // cpp: layoutng/internal/constraint_space.h:415-417
    pub fn AncestorHasClearancePastAdjoiningFloats(&self) -> bool {
        self.bitfields_.bit(13)
    }

    // cpp: layoutng/internal/constraint_space.h:421-424
    pub fn GetBaselineAlgorithmType(&self) -> BaselineAlgorithmType {
        if self.bitfields_.bit(19) {
            BaselineAlgorithmType::kInlineBlock
        } else {
            BaselineAlgorithmType::kDefault
        }
    }

    // cpp: layoutng/internal/constraint_space.h:427-429
    pub fn CacheSlot(&self) -> LayoutResultCacheSlot {
        if self.bitfields_.bit(20) {
            LayoutResultCacheSlot::kMeasure
        } else {
            LayoutResultCacheSlot::kLayout
        }
    }

    // cpp: layoutng/internal/constraint_space.h:438-440
    pub fn IsFixedInlineSize(&self) -> bool {
        self.bitfields_.bit(25)
    }

    pub fn IsFixedBlockSize(&self) -> bool {
        self.bitfields_.bit(26)
    }

    // cpp: layoutng/internal/constraint_space.h:457-459
    pub fn IsInitialBlockSizeIndefinite(&self) -> bool {
        self.bitfields_.bit(27)
    }

    // cpp: layoutng/internal/constraint_space.h:462-473
    pub fn InlineAutoBehavior(&self) -> AutoSizeBehavior {
        self.bitfields_.auto_behavior(21)
    }

    pub fn BlockAutoBehavior(&self) -> AutoSizeBehavior {
        self.bitfields_.auto_behavior(23)
    }

    pub fn IsInlineAutoBehaviorStretch(&self) -> bool {
        self.InlineAutoBehavior() != AutoSizeBehavior::kFitContent
    }

    pub fn IsBlockAutoBehaviorStretch(&self) -> bool {
        self.BlockAutoBehavior() != AutoSizeBehavior::kFitContent
    }

    // cpp: layoutng/internal/constraint_space.h:476-484
    pub fn IsTableCellChild(&self) -> bool {
        self.bitfields_.bit(28)
    }

    pub fn IsRestrictedBlockSizeTableCellChild(&self) -> bool {
        self.bitfields_.bit(29)
    }

    pub fn IsPaintedAtomically(&self) -> bool {
        self.bitfields_.bit(10)
    }

    // cpp: layoutng/internal/constraint_space.h:176-186
    pub fn AvailableSize(&self) -> LogicalSize {
        self.available_size_
    }

    pub fn PercentageResolutionInlineSize(&self) -> LayoutUnit {
        self.percentage_size_.inline_size
    }

    pub fn PercentageResolutionBlockSize(&self) -> LayoutUnit {
        self.percentage_size_.block_size
    }

    pub fn PercentageResolutionSize(&self) -> LogicalSize {
        self.percentage_size_
    }

    // cpp: layoutng/internal/constraint_space.h:190-206
    pub fn ReplacedChildPercentageResolutionBlockSize(&self) -> LayoutUnit {
        let rare_data = self.rare_data_.Get();
        if !rare_data.is_null() {
            let value = unsafe { &*rare_data }.replaced_child_percentage_resolution_block_size;
            if value != kIndefiniteSize {
                return value;
            }
        }
        self.PercentageResolutionBlockSize()
    }

    pub fn ReplacedChildPercentageResolutionSize(&self) -> LogicalSize {
        LogicalSize::new(
            self.PercentageResolutionInlineSize(),
            self.ReplacedChildPercentageResolutionBlockSize(),
        )
    }

    // cpp: layoutng/internal/constraint_space.h:209-243
    pub fn MarginPaddingPercentageResolutionSize(&self) -> LogicalSize {
        if self.GetDecorationPercentageResolutionType()
            == DecorationPercentageResolutionType::kContainingBlockSize
        {
            return self.PercentageResolutionSize();
        }
        let containing_block_inline_size = if !self.IsOrthogonalWritingModeRoot() {
            self.PercentageResolutionInlineSize()
        } else if self.PercentageResolutionBlockSize() != kIndefiniteSize {
            self.PercentageResolutionBlockSize()
        } else {
            LayoutUnit::default()
        };
        LogicalSize::new(containing_block_inline_size, containing_block_inline_size)
    }

    // cpp: layoutng/internal/constraint_space.h:244-246
    pub fn UsesOrthogonalFallbackInlineSize(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.uses_orthogonal_fallback_inline_size
    }

    // cpp: layoutng/internal/constraint_space.h:250-266
    pub fn TargetStretchInlineSize(&self) -> LayoutUnit {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            kIndefiniteSize
        } else {
            unsafe { &*rare_data }.TargetStretchInlineSize()
        }
    }

    pub fn HasTargetStretchInlineSize(&self) -> bool {
        self.TargetStretchInlineSize() != kIndefiniteSize
    }

    pub fn TargetStretchBlockSizes(&self) -> Option<MathTargetStretchBlockSizes> {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            None
        } else {
            unsafe { &*rare_data }.TargetStretchBlockSizes()
        }
    }

    // cpp: layoutng/internal/constraint_space.h:269-298
    pub fn TableCellBorders(&self) -> BoxStrut {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            BoxStrut::default()
        } else {
            unsafe { &*rare_data }.TableCellBorders()
        }
    }

    pub fn TableCellColumnIndex(&self) -> u32 {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            0
        } else {
            unsafe { &*rare_data }.TableCellColumnIndex()
        }
    }

    pub fn TableCellAlignmentBaseline(&self) -> Option<LayoutUnit> {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            None
        } else {
            unsafe { &*rare_data }.TableCellAlignmentBaseline()
        }
    }

    pub fn IsTableCellWithCollapsedBorders(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.IsTableCellWithCollapsedBorders()
    }

    pub fn TableData(&self) -> *const TableConstraintSpaceData {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*rare_data }.TableData()
        }
    }

    // cpp: layoutng/internal/constraint_space.h:855-860,1269-1278
    pub fn CloneTableDataHandle(&self) -> Arc<TableConstraintSpaceData> {
        let rare_data = self.rare_data_.Get();
        assert!(!rare_data.is_null(), "table data requires rare data");
        unsafe { &*rare_data }.CloneTableDataHandle()
    }

    pub fn TableRowIndex(&self) -> u32 {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            u32::MAX
        } else {
            unsafe { &*rare_data }.TableRowIndex()
        }
    }

    pub fn TableSectionIndex(&self) -> u32 {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            u32::MAX
        } else {
            unsafe { &*rare_data }.TableSectionIndex()
        }
    }

    // cpp: layoutng/internal/constraint_space.h:385-395
    pub fn IsTableCell(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null()
            && unsafe { &*rare_data }.GetDataUnionType() == DataUnionType::kTableCellData
    }

    pub fn HideTableCellIfEmpty(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.hide_table_cell_if_empty
    }

    // cpp: layoutng/internal/constraint_space.h:632-646
    pub fn BlockStartAnnotationSpace(&self) -> LayoutUnit {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            LayoutUnit::default()
        } else {
            unsafe { &*rare_data }.BlockStartAnnotationSpace()
        }
    }

    pub fn ContainsAnnotations(&self) -> bool {
        self.bitfields_.bit(14)
    }

    pub fn PreviousSiblingBlockEndAnnotationSpace(&self) -> LayoutUnit {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            LayoutUnit::default()
        } else {
            unsafe { &*rare_data }.PreviousSiblingBlockEndAnnotationSpace()
        }
    }

    pub fn GetMarginStrut(&self) -> MarginStrut {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            MarginStrut::default()
        } else {
            unsafe { &*rare_data }.GetMarginStrut()
        }
    }

    // cpp: layoutng/internal/constraint_space.h:664-664
    pub fn GetBfcOffset(&self) -> BfcOffset {
        self.bfc_offset_
    }

    // cpp: layoutng/internal/constraint_space.h:679-687
    pub fn ForcedBfcBlockOffset(&self) -> Option<LayoutUnit> {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            None
        } else {
            unsafe { &*rare_data }.ForcedBfcBlockOffset()
        }
    }

    pub fn OptimisticBfcBlockOffset(&self) -> Option<LayoutUnit> {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            None
        } else {
            unsafe { &*rare_data }.OptimisticBfcBlockOffset()
        }
    }

    // cpp: layoutng/internal/constraint_space.h:697-707
    pub fn ExpectedBfcBlockOffset(&self) -> LayoutUnit {
        if self.rare_data_.Get().is_null() {
            debug_assert!(self.ForcedBfcBlockOffset().is_none());
            debug_assert!(self.OptimisticBfcBlockOffset().is_none());
            return self.bfc_offset_.block_offset;
        }
        self.ForcedBfcBlockOffset().unwrap_or(
            self.OptimisticBfcBlockOffset()
                .unwrap_or(self.GetBfcOffset().block_offset),
        )
    }

    // cpp: layoutng/internal/constraint_space.h:709-711
    pub fn CustomLayoutData(&self) -> *mut SerializedScriptValue {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &*rare_data }.CustomLayoutData()
        }
    }

    // cpp: layoutng/internal/constraint_space.h:724-726
    pub fn GetAdjoiningObjectTypes(&self) -> AdjoiningObjectTypes {
        (self.bitfields_.0 & 0b111) as AdjoiningObjectTypes
    }

    // cpp: layoutng/internal/constraint_space.h:730-730
    pub fn HasFloats(&self) -> bool {
        !self.GetExclusionSpace().IsEmpty()
    }

    // cpp: layoutng/internal/constraint_space.h:732-737
    pub fn HasClearanceOffset(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.ClearanceOffset() != LayoutUnit::Min()
    }

    pub fn ClearanceOffset(&self) -> LayoutUnit {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            LayoutUnit::Min()
        } else {
            unsafe { &*rare_data }.ClearanceOffset()
        }
    }

    // cpp: layoutng/internal/constraint_space.h:741-743
    pub fn IsPushedByFloats(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.is_pushed_by_floats
    }

    // cpp: layoutng/internal/constraint_space.h:745-751
    pub fn GetLineClampData(&self) -> LineClampData {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            LineClampData::default()
        } else {
            unsafe { &*rare_data }.GetLineClampData()
        }
    }

    pub fn GetLineClampAncestorChain(&self) -> *const LineClampAncestorChain {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*rare_data }.GetLineClampAncestorChain()
        }
    }

    // cpp: layoutng/internal/constraint_space.h:300-302
    pub fn PageName(&self) -> AtomicString {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            AtomicString::default()
        } else {
            unsafe { &*rare_data }.page_name.clone()
        }
    }

    // cpp: layoutng/internal/constraint_space.h:313-315
    pub fn FragmentainerBlockSize(&self) -> LayoutUnit {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            kIndefiniteSize
        } else {
            unsafe { &*rare_data }.fragmentainer_block_size
        }
    }

    // cpp: layoutng/internal/constraint_space.h:319-322
    pub fn IsInitialColumnBalancingPass(&self) -> bool {
        self.BlockFragmentationType() == FragmentationType::kFragmentColumn
            && self.FragmentainerBlockSize() == kIndefiniteSize
    }

    // cpp: layoutng/internal/constraint_space.h:326-333
    pub fn HasKnownFragmentainerBlockSize(&self) -> bool {
        if !self.HasBlockFragmentation() || self.IsInitialColumnBalancingPass() {
            return false;
        }
        debug_assert_ne!(self.FragmentainerBlockSize(), kIndefiniteSize);
        true
    }

    // cpp: layoutng/internal/constraint_space.h:340-345
    pub fn FragmentainerOffset(&self) -> LayoutUnit {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() || !self.HasBlockFragmentation() {
            LayoutUnit::default()
        } else {
            unsafe { &*rare_data }.fragmentainer_offset
        }
    }

    // cpp: layoutng/internal/constraint_space.h:354-356
    pub fn IsAtFragmentainerStart(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.is_at_fragmentainer_start
    }

    // cpp: layoutng/internal/constraint_space.h:363-363
    pub fn ShouldRepeat(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.should_repeat
    }

    // cpp: layoutng/internal/constraint_space.h:368-370
    pub fn IsInsideRepeatableContent(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.is_inside_repeatable_content
    }

    // cpp: layoutng/internal/constraint_space.h:374-376
    pub fn IsInsideBreakAvoid(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.is_inside_break_avoid
    }

    // cpp: layoutng/internal/constraint_space.h:791-798
    pub fn GetDecorationPercentageResolutionType(&self) -> DecorationPercentageResolutionType {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            DecorationPercentageResolutionType::kContainingBlockInlineSize
        } else {
            unsafe { &*rare_data }.decoration_percentage_resolution_type
        }
    }

    // cpp: layoutng/internal/constraint_space.h:496-498
    pub fn HasBlockFragmentation(&self) -> bool {
        self.BlockFragmentationType() != FragmentationType::kFragmentNone
    }

    // cpp: layoutng/internal/constraint_space.h:502-504
    pub fn IsBlockFragmentationForcedOff(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.is_block_fragmentation_forced_off
    }

    // cpp: layoutng/internal/constraint_space.h:512-515
    pub fn IsMonolithicOverflowPropagationDisabled(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.is_monolithic_overflow_propagation_disabled
    }

    // cpp: layoutng/internal/constraint_space.h:518-522
    pub fn IsPaginated(&self) -> bool {
        self.BlockFragmentationType() == FragmentationType::kFragmentPage
    }

    // cpp: layoutng/internal/constraint_space.h:532-534
    pub fn SafePrintableInset(&self) -> LayoutUnit {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            LayoutUnit::default()
        } else {
            unsafe { &*rare_data }.safe_printable_inset
        }
    }

    // cpp: layoutng/internal/constraint_space.h:542-550
    pub fn PaperEdgeAdjacentSides(&self) -> LogicalBoxSides {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            return LogicalBoxSides::with_value(false);
        }
        let data = unsafe { &*rare_data };
        LogicalBoxSides::new(
            data.is_adjacent_to_paper_edge_inline_start,
            data.is_adjacent_to_paper_edge_inline_end,
            data.is_adjacent_to_paper_edge_block_start,
            data.is_adjacent_to_paper_edge_block_end,
        )
    }

    // cpp: layoutng/internal/constraint_space.h:555-557
    pub fn RequiresContentBeforeBreaking(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.requires_content_before_breaking
    }

    // cpp: layoutng/internal/constraint_space.h:561-563
    pub fn IsInsideBalancedColumns(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.is_inside_balanced_columns
    }

    // cpp: layoutng/internal/constraint_space.h:567-569
    pub fn ShouldIgnoreForcedBreaks(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.should_ignore_forced_breaks
    }

    // cpp: layoutng/internal/constraint_space.h:573-575
    pub fn IsInColumnBfc(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.is_in_column_bfc
    }

    // cpp: layoutng/internal/constraint_space.h:579
    pub fn IsPastBreak(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.is_past_break
    }

    // cpp: layoutng/internal/constraint_space.h:589-592
    pub fn MinBlockSizeShouldEncompassIntrinsicSize(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null()
            && unsafe { &*rare_data }.min_block_size_should_encompass_intrinsic_size
    }

    // cpp: layoutng/internal/constraint_space.h:606-609
    pub fn MinBreakAppeal(&self) -> BreakAppeal {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            BreakAppeal::kBreakAppealLastResort
        } else {
            unsafe { &*rare_data }.min_break_appeal
        }
    }

    // cpp: layoutng/internal/constraint_space.h:618-619
    pub fn ShouldPropagateChildBreakValues(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.propagate_child_break_values
    }

    // cpp: layoutng/internal/constraint_space.h:623-625
    pub fn IsRestrictedBlockSizeTableCell(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.is_restricted_block_size_table_cell
    }

    // cpp: layoutng/internal/constraint_space.h:488-492
    pub fn BlockFragmentationType(&self) -> FragmentationType {
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            FragmentationType::kFragmentNone
        } else {
            unsafe { &*rare_data }.block_direction_fragmentation_type
        }
    }

    // cpp: layoutng/internal/constraint_space.h:759-761
    pub fn ShouldTextBoxTrimNodeEnd(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.should_text_box_trim_node_end
    }

    // cpp: layoutng/internal/constraint_space.h:755-757
    pub fn ShouldTextBoxTrimNodeStart(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.should_text_box_trim_node_start
    }

    // cpp: layoutng/internal/constraint_space.h:764-766
    pub fn ShouldTextBoxTrimFragmentainerStart(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.should_text_box_trim_fragmentainer_start
    }

    // cpp: layoutng/internal/constraint_space.h:769-771
    pub fn ShouldTextBoxTrimFragmentainerEnd(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.should_text_box_trim_fragmentainer_end
    }

    // cpp: layoutng/internal/constraint_space.h:774-777
    pub fn ShouldTextBoxTrimInsideWhenLineClamp(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.should_text_box_trim_inside_when_line_clamp
    }

    // cpp: layoutng/internal/constraint_space.h:780-782
    pub fn ShouldForceTextBoxTrimEnd(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.should_force_text_box_trim_end
    }

    // cpp: layoutng/internal/constraint_space.h:786-788
    pub fn ShouldForceMarginTrimEnd(&self) -> bool {
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null() && unsafe { &*rare_data }.should_force_margin_trim_end
    }

    // cpp: layoutng/internal/constraint_space.h:799-804
    pub fn IgnoreMarginsForStretch(&self) -> LogicalBoxSides {
        LogicalBoxSides::new(
            self.bitfields_.bit(15),
            self.bitfields_.bit(16),
            self.bitfields_.bit(17),
            self.bitfields_.bit(18),
        )
    }

    // cpp: layoutng/internal/constraint_space.h:814-833
    pub fn MaySkipLayout(&self, other: &Self) -> bool {
        if !self.bitfields_.MaySkipLayout(other.bitfields_) {
            return false;
        }
        let a = self.rare_data_.Get();
        let b = other.rare_data_.Get();
        match (a.is_null(), b.is_null()) {
            (true, true) => true,
            (false, false) => unsafe { &*a }.MaySkipLayout(unsafe { &*b }),
            (false, true) => unsafe { &*a }.IsInitialForMaySkipLayout(),
            (true, false) => unsafe { &*b }.IsInitialForMaySkipLayout(),
        }
    }

    // cpp: layoutng/internal/constraint_space.h:836-838
    pub fn AreInlineSizeConstraintsEqual(&self, other: &Self) -> bool {
        self.bitfields_
            .AreInlineSizeConstraintsEqual(other.bitfields_)
    }

    // cpp: layoutng/internal/constraint_space.h:839-848
    pub fn AreBlockSizeConstraintsEqual(&self, other: &Self) -> bool {
        if !self
            .bitfields_
            .AreBlockSizeConstraintsEqual(other.bitfields_)
        {
            return false;
        }
        if self.rare_data_.Get().is_null() && other.rare_data_.Get().is_null() {
            return true;
        }
        self.TableCellAlignmentBaseline() == other.TableCellAlignmentBaseline()
            && self.MinBlockSizeShouldEncompassIntrinsicSize()
                == other.MinBlockSizeShouldEncompassIntrinsicSize()
    }

    // cpp: layoutng/internal/constraint_space.h:850-853
    pub fn AreSizesEqual(&self, other: &Self) -> bool {
        self.available_size_ == other.available_size_
            && self.percentage_size_ == other.percentage_size_
    }

    // cpp: layoutng/internal/constraint_space.h:855-860
    pub fn ReplaceTableRowData(
        &mut self,
        table_data: Arc<TableConstraintSpaceData>,
        row_index: u32,
    ) {
        let rare_data = self.rare_data_.Get();
        debug_assert!(!rare_data.is_null());
        unsafe { &mut *rare_data }.ReplaceTableRowData(table_data, row_index);
    }
}

// cpp: layoutng/internal/constraint_space.cc:15-25
#[repr(C)]
struct SameSizeAsConstraintSpace {
    available_size: LogicalSize,
    percentage_size: LogicalSize,
    bfc_offset: BfcOffset,
    exclusion_space: ExclusionSpace,
    rare_data: Member<std::ffi::c_void>,
    bitfields: [u32; 1],
}

const _: () = assert!(
    std::mem::size_of::<ConstraintSpace>() == std::mem::size_of::<SameSizeAsConstraintSpace>()
);

impl ConstraintSpace {
    // cpp: layoutng/internal/constraint_space.cc:29-55
    pub fn CloneForBlockInInlineIfNeeded<'a>(
        &'a self,
        space: &'a mut Option<ConstraintSpace>,
    ) -> &'a ConstraintSpace {
        if self.ShouldTextBoxTrimNodeEnd() {
            *space = Some(self.clone());
            let copy = space.as_mut().unwrap();
            let original_rare_data = copy.rare_data_.Get();
            debug_assert!(!original_rare_data.is_null());
            let mut rare_data = unsafe { &*original_rare_data }.clone();
            if self.ShouldForceTextBoxTrimEnd() {
                rare_data.should_force_text_box_trim_end = false;
            } else {
                rare_data.should_text_box_trim_node_end = false;
                rare_data.should_text_box_trim_fragmentainer_end = false;
            }
            copy.rare_data_ = Member::from_ptr(MakeGarbageCollected(rare_data));
            copy
        } else {
            debug_assert!(!self.ShouldForceTextBoxTrimEnd());
            self
        }
    }

    // cpp: layoutng/internal/constraint_space.cc:55-61
    pub fn ToString(&self) -> BlinkString {
        let bfc_offset = self.GetBfcOffset();
        let available_size = self.AvailableSize();
        StrCat(&[
            BlinkString::from("Offset: "),
            bfc_offset.line_offset.ToString().into(),
            BlinkString::from(","),
            bfc_offset.block_offset.ToString().into(),
            BlinkString::from(" Size: "),
            available_size.inline_size.ToString().into(),
            BlinkString::from("x"),
            available_size.block_size.ToString().into(),
            BlinkString::from(" Clearance: "),
            if self.HasClearanceOffset() {
                self.ClearanceOffset().ToString().into()
            } else {
                BlinkString::from("none")
            },
        ])
    }
}

// cpp: layoutng/internal/constraint_space.h:1679-1682
impl std::fmt::Display for ConstraintSpace {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str(&self.ToString().Utf8())
    }
}
