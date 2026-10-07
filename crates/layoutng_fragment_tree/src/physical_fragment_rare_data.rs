use std::mem::ManuallyDrop;

use foundation::{
    AtomicString, GCedHeapVector, LayoutUnit, MakeGarbageCollected, Member,
    PhysicalOffset, PhysicalRect, ScopedRefPtr, Vector, Visitor,
};
use layoutng::internal::frame_set_layout_data::FrameSetLayoutData;
use layoutng::internal::mathml_paint_info::MathMLPaintInfo;
use layoutng::internal::layout_node_metadata::Node;
use layoutng::internal::gap::gap_geometry::GapGeometry;
use layoutng::internal::table_borders::TableBorders;
use layoutng::internal::table_fragment_data::{
    CollapsedTableBordersGeometry, GCedTableColumnGeometries,
};
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_geometry::geometry::logical_rect::LogicalRect;

use crate::box_fragment_builder::BoxFragmentBuilder;

// Field identifiers match their bit positions. The last value is 13, below
// the 32-bit mask width even on ARM.
// cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:62-86
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FieldId {
    kScrollableOverflow = 0,
    kBorders = 1,
    kScrollbar = 2,
    kPadding = 3,
    kInflowBounds = 4,
    kFrameSetLayoutData = 5,
    kTableGridRect = 6,
    kTableCollapsedBordersGeometry = 7,
    kTableCellColumnIndex = 8,
    kTableSectionStartRowIndex = 9,
    kTableSectionRowOffsets = 10,
    kPageName = 11,
    kMargins = 12,
    kOffsetFromRootFragmentationContext = 13,
}

impl FieldId {
    pub(crate) const kMaxValue: Self = Self::kOffsetFromRootFragmentationContext;
}

const _: () = assert!((FieldId::kMaxValue as usize) < u32::BITS as usize);

// C++ stores exactly one active value in a union; ManuallyDrop prevents Rust
// from dropping inactive fields. The tag remains a separate trailing word.
// cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:88-107
#[repr(C)]
pub(crate) union RareFieldValue {
    pub(crate) scrollable_overflow: ManuallyDrop<PhysicalRect>,
    pub(crate) borders: ManuallyDrop<PhysicalBoxStrut>,
    pub(crate) scrollbar: ManuallyDrop<PhysicalBoxStrut>,
    pub(crate) padding: ManuallyDrop<PhysicalBoxStrut>,
    pub(crate) inflow_bounds: ManuallyDrop<PhysicalRect>,
    pub(crate) frame_set_layout_data: ManuallyDrop<Option<Box<FrameSetLayoutData>>>,
    pub(crate) table_grid_rect: ManuallyDrop<LogicalRect>,
    pub(crate) table_collapsed_borders: ManuallyDrop<Option<ScopedRefPtr<TableBorders>>>,
    pub(crate) table_collapsed_borders_geometry:
        ManuallyDrop<Option<Box<CollapsedTableBordersGeometry>>>,
    pub(crate) table_cell_column_index: ManuallyDrop<u32>,
    pub(crate) table_section_start_row_index: ManuallyDrop<u32>,
    pub(crate) table_section_row_offsets: ManuallyDrop<Vector<LayoutUnit>>,
    pub(crate) page_name: ManuallyDrop<AtomicString>,
    pub(crate) margins: ManuallyDrop<PhysicalBoxStrut>,
    pub(crate) offset_from_root_fragmentation_context: ManuallyDrop<PhysicalOffset>,
}

// cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:88-112
#[repr(C)]
pub(crate) struct RareField {
    pub(crate) value_: RareFieldValue,
    pub(crate) type_: FieldId,
}

#[allow(non_snake_case)]
impl RareField {
    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.cc:139-178
    pub(crate) fn new(field_id: FieldId) -> Self {
        let value = match field_id {
            FieldId::kScrollableOverflow => RareFieldValue {
                scrollable_overflow: ManuallyDrop::new(PhysicalRect::default()),
            },
            FieldId::kBorders => RareFieldValue {
                borders: ManuallyDrop::new(PhysicalBoxStrut::default()),
            },
            FieldId::kScrollbar => RareFieldValue {
                scrollbar: ManuallyDrop::new(PhysicalBoxStrut::default()),
            },
            FieldId::kPadding => RareFieldValue {
                padding: ManuallyDrop::new(PhysicalBoxStrut::default()),
            },
            FieldId::kInflowBounds => RareFieldValue {
                inflow_bounds: ManuallyDrop::new(PhysicalRect::default()),
            },
            FieldId::kFrameSetLayoutData => RareFieldValue {
                frame_set_layout_data: ManuallyDrop::new(None),
            },
            FieldId::kTableGridRect => RareFieldValue {
                table_grid_rect: ManuallyDrop::new(LogicalRect::default()),
            },
            FieldId::kTableCollapsedBordersGeometry => RareFieldValue {
                table_collapsed_borders_geometry: ManuallyDrop::new(None),
            },
            FieldId::kTableCellColumnIndex => RareFieldValue {
                table_cell_column_index: ManuallyDrop::new(0),
            },
            FieldId::kTableSectionStartRowIndex => RareFieldValue {
                table_section_start_row_index: ManuallyDrop::new(0),
            },
            FieldId::kTableSectionRowOffsets => RareFieldValue {
                table_section_row_offsets: ManuallyDrop::new(Vector::default()),
            },
            FieldId::kPageName => RareFieldValue {
                page_name: ManuallyDrop::new(AtomicString::default()),
            },
            FieldId::kMargins => RareFieldValue {
                margins: ManuallyDrop::new(PhysicalBoxStrut::default()),
            },
            FieldId::kOffsetFromRootFragmentationContext => RareFieldValue {
                offset_from_root_fragmentation_context: ManuallyDrop::new(PhysicalOffset::default()),
            },
        };
        Self {
            value_: value,
            type_: field_id,
        }
    }

    // C++'s move constructor dispatches on the active member. A Rust move
    // relocates the tagged value, leaving the source uninitialized; Drop then
    // runs once on the destination's active member.
    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.cc:180-191
    pub(crate) fn move_from(other: Self) -> Self {
        other
    }

    // C++ copy construction of the owner assigns each active field to a new
    // default member. The two unique_ptr values require deep copies.
    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.cc:91-99
    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.cc:111-127
    pub(crate) fn copy_from(other: &Self) -> Self {
        let mut result = Self::new(other.type_);
        unsafe {
            match other.type_ {
                FieldId::kScrollableOverflow => {
                    *result.value_.scrollable_overflow = *other.value_.scrollable_overflow
                }
                FieldId::kBorders => *result.value_.borders = *other.value_.borders,
                FieldId::kScrollbar => *result.value_.scrollbar = *other.value_.scrollbar,
                FieldId::kPadding => *result.value_.padding = *other.value_.padding,
                FieldId::kInflowBounds => {
                    *result.value_.inflow_bounds = *other.value_.inflow_bounds
                }
                FieldId::kFrameSetLayoutData => {
                    *result.value_.frame_set_layout_data = other
                        .value_
                        .frame_set_layout_data
                        .as_ref()
                        .map(|value| Box::new((**value).clone()));
                }
                FieldId::kTableGridRect => {
                    *result.value_.table_grid_rect = *other.value_.table_grid_rect
                }
                FieldId::kTableCollapsedBordersGeometry => {
                    *result.value_.table_collapsed_borders_geometry = other
                        .value_
                        .table_collapsed_borders_geometry
                        .as_ref()
                        .map(|value| Box::new((**value).clone()));
                }
                FieldId::kTableCellColumnIndex => {
                    *result.value_.table_cell_column_index = *other.value_.table_cell_column_index
                }
                FieldId::kTableSectionStartRowIndex => {
                    *result.value_.table_section_start_row_index =
                        *other.value_.table_section_start_row_index
                }
                FieldId::kTableSectionRowOffsets => {
                    *result.value_.table_section_row_offsets =
                        (*other.value_.table_section_row_offsets).clone()
                }
                FieldId::kPageName => *result.value_.page_name = (*other.value_.page_name).clone(),
                FieldId::kMargins => *result.value_.margins = *other.value_.margins,
                FieldId::kOffsetFromRootFragmentationContext => {
                    *result.value_.offset_from_root_fragmentation_context =
                        *other.value_.offset_from_root_fragmentation_context
                }
            }
        }
        result
    }
}

// cpp: layoutng_fragment_tree/physical_fragment_rare_data.cc:193-202
impl Drop for RareField {
    fn drop(&mut self) {
        unsafe {
            match self.type_ {
                FieldId::kScrollableOverflow => {
                    ManuallyDrop::drop(&mut self.value_.scrollable_overflow)
                }
                FieldId::kBorders => ManuallyDrop::drop(&mut self.value_.borders),
                FieldId::kScrollbar => ManuallyDrop::drop(&mut self.value_.scrollbar),
                FieldId::kPadding => ManuallyDrop::drop(&mut self.value_.padding),
                FieldId::kInflowBounds => ManuallyDrop::drop(&mut self.value_.inflow_bounds),
                FieldId::kFrameSetLayoutData => {
                    ManuallyDrop::drop(&mut self.value_.frame_set_layout_data)
                }
                FieldId::kTableGridRect => ManuallyDrop::drop(&mut self.value_.table_grid_rect),
                FieldId::kTableCollapsedBordersGeometry => {
                    ManuallyDrop::drop(&mut self.value_.table_collapsed_borders_geometry)
                }
                FieldId::kTableCellColumnIndex => {
                    ManuallyDrop::drop(&mut self.value_.table_cell_column_index)
                }
                FieldId::kTableSectionStartRowIndex => {
                    ManuallyDrop::drop(&mut self.value_.table_section_start_row_index)
                }
                FieldId::kTableSectionRowOffsets => {
                    ManuallyDrop::drop(&mut self.value_.table_section_row_offsets)
                }
                FieldId::kPageName => ManuallyDrop::drop(&mut self.value_.page_name),
                FieldId::kMargins => ManuallyDrop::drop(&mut self.value_.margins),
                FieldId::kOffsetFromRootFragmentationContext => {
                    ManuallyDrop::drop(&mut self.value_.offset_from_root_fragmentation_context)
                }
            }
        }
    }
}

// cpp: layoutng_fragment_tree/physical_fragment_rare_data.cc:166-174
#[repr(C)]
union SameSizeAsRareFieldValue {
    pointer: ManuallyDrop<Option<Box<i32>>>,
    units: ManuallyDrop<[LayoutUnit; 4]>,
    vector: ManuallyDrop<Vector<i32>>,
}

#[repr(C)]
struct SameSizeAsRareField {
    value: SameSizeAsRareFieldValue,
    type_: u8,
}

const _: [(); std::mem::size_of::<SameSizeAsRareField>()] = [(); std::mem::size_of::<RareField>()];

// C++ keeps GC-owned fields outside its conditional vector, so its visitor
// need not inspect the active union member.
// cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:37-49
// cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:165-174
#[repr(C)]
pub struct PhysicalFragmentRareData {
    // The C++ Vector keeps one element inline. Rust Vec preserves its element
    // order and mutation behavior; inline-capacity allocation is pending.
    pub(crate) field_list_: Vec<RareField>,
    pub(crate) bit_field_: u32,
    pub(crate) table_collapsed_borders_: Member<TableBorders>,
    pub(crate) table_column_geometries_: Member<GCedTableColumnGeometries>,
    pub(crate) mathml_paint_info_: Member<MathMLPaintInfo>,
    pub(crate) reading_flow_nodes_: Member<GCedHeapVector<Member<Node>>>,
    pub(crate) gap_geometry_: Member<GapGeometry>,
}

#[allow(non_snake_case)]
impl PhysicalFragmentRareData {
    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:40
    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.cc:13-15
    pub fn new(num_fields: u32) -> Self {
        let mut result = Self {
            field_list_: Vector::default(),
            bit_field_: 0,
            table_collapsed_borders_: Member::default(),
            table_column_geometries_: Member::default(),
            mathml_paint_info_: Member::default(),
            reading_flow_nodes_: Member::default(),
            gap_geometry_: Member::default(),
        };
        result.field_list_.reserve(num_fields as usize);
        result
    }

    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:41-47
    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.cc:17-89
    pub fn from_builder(
        scrollable_overflow: Option<&PhysicalRect>,
        borders: Option<&PhysicalBoxStrut>,
        scrollbar: Option<&PhysicalBoxStrut>,
        padding: Option<&PhysicalBoxStrut>,
        inflow_bounds: Option<PhysicalRect>,
        builder: &mut BoxFragmentBuilder,
        num_fields: u32,
    ) -> Self {
        let reading_flow_nodes = if builder.reading_flow_nodes_.is_empty() {
            Member::default()
        } else {
            let nodes: GCedHeapVector<Member<Node>> =
                builder.reading_flow_nodes_.iter().cloned().collect();
            Member::from_ptr(MakeGarbageCollected(nodes))
        };
        let mut result = Self {
            field_list_: Vector::default(),
            bit_field_: 0,
            table_collapsed_borders_: Member::from_ptr(
                builder.table_collapsed_borders_ as *mut TableBorders,
            ),
            table_column_geometries_: Member::default(),
            mathml_paint_info_: Member::from_ptr(
                builder.mathml_paint_info_ as *mut MathMLPaintInfo,
            ),
            reading_flow_nodes_: reading_flow_nodes,
            gap_geometry_: Member::from_ptr(builder.gap_geometry_ as *mut GapGeometry),
        };
        result.field_list_.reserve(num_fields as usize);

        // The source inserts in FieldId order, avoiding middle insertions.
        if let Some(value) = scrollable_overflow {
            unsafe {
                *result
                    .SetField(FieldId::kScrollableOverflow)
                    .value_
                    .scrollable_overflow = *value;
            }
        }
        if let Some(value) = borders {
            unsafe {
                *result.SetField(FieldId::kBorders).value_.borders = *value;
            }
        }
        if let Some(value) = scrollbar {
            unsafe {
                *result.SetField(FieldId::kScrollbar).value_.scrollbar = *value;
            }
        }
        if let Some(value) = padding {
            unsafe {
                *result.SetField(FieldId::kPadding).value_.padding = *value;
            }
        }
        if let Some(value) = inflow_bounds {
            unsafe {
                *result.SetField(FieldId::kInflowBounds).value_.inflow_bounds = value;
            }
        }
        if builder.frame_set_layout_data_.is_some() {
            unsafe {
                *result
                    .SetField(FieldId::kFrameSetLayoutData)
                    .value_
                    .frame_set_layout_data = builder.frame_set_layout_data_.take();
            }
        }
        if let Some(value) = builder.table_grid_rect_ {
            unsafe {
                *result
                    .SetField(FieldId::kTableGridRect)
                    .value_
                    .table_grid_rect = value;
            }
        }
        if builder.table_collapsed_borders_geometry_.is_some() {
            unsafe {
                *result
                    .SetField(FieldId::kTableCollapsedBordersGeometry)
                    .value_
                    .table_collapsed_borders_geometry =
                    builder.table_collapsed_borders_geometry_.take();
            }
        }
        if let Some(value) = builder.table_cell_column_index_ {
            unsafe {
                *result
                    .SetField(FieldId::kTableCellColumnIndex)
                    .value_
                    .table_cell_column_index = value;
            }
        }
        if !builder.table_section_row_offsets_.is_empty() {
            unsafe {
                *result
                    .SetField(FieldId::kTableSectionStartRowIndex)
                    .value_
                    .table_section_start_row_index = builder.table_section_start_row_index_;
                *result
                    .SetField(FieldId::kTableSectionRowOffsets)
                    .value_
                    .table_section_row_offsets =
                    std::mem::take(&mut builder.table_section_row_offsets_);
            }
        }
        if !builder.page_name_.IsNull() {
            unsafe {
                *result.SetField(FieldId::kPageName).value_.page_name = builder.page_name_.clone();
            }
        }

        if !builder.table_column_geometries_.is_empty() {
            let columns: GCedTableColumnGeometries =
                builder.table_column_geometries_.clone().into();
            result.table_column_geometries_ = Member::from_ptr(MakeGarbageCollected(columns));
        }
        debug_assert!(result.field_list_.len() <= num_fields as usize);
        result
    }

    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:48
    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.cc:101-130
    pub fn copy_from(other: &Self) -> Self {
        // The source initializes only these three GC fields in its copy
        // constructor; mathml_paint_info_ and reading_flow_nodes_ stay null.
        let mut result = Self {
            field_list_: Vector::default(),
            bit_field_: 0,
            table_collapsed_borders_: other.table_collapsed_borders_.clone(),
            table_column_geometries_: other.table_column_geometries_.clone(),
            mathml_paint_info_: Member::default(),
            reading_flow_nodes_: Member::default(),
            gap_geometry_: other.gap_geometry_.clone(),
        };
        result.field_list_.reserve(other.field_list_.capacity());
        for field in other.field_list_.iter() {
            *result.SetField(field.type_) = RareField::copy_from(field);
        }
        debug_assert_eq!(result.field_list_.len(), other.field_list_.len());
        result
    }

    // C++ has a default destructor. Rust automatically drops field_list_
    // (and each active union member) and its GC handles.
    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:49
    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.cc:135

    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:51-57
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.table_collapsed_borders_);
        visitor.Trace(&self.table_column_geometries_);
        visitor.Trace(&self.mathml_paint_info_);
        visitor.Trace(&self.reading_flow_nodes_);
        visitor.Trace(&self.gap_geometry_);
    }

    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:114-116
    pub(crate) const fn FieldIdBit(field_id: FieldId) -> u32 {
        1u32 << field_id as u32
    }

    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:118-121
    pub(crate) const fn FieldIdLowerMask(field_id: FieldId) -> u32 {
        !(!0u32 << field_id as u32)
    }

    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:123-126
    pub(crate) fn GetFieldIndex(&self, field_id: FieldId) -> u32 {
        debug_assert_ne!(self.bit_field_ & Self::FieldIdBit(field_id), 0);
        (self.bit_field_ & Self::FieldIdLowerMask(field_id)).count_ones()
    }

    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:128-133
    pub(crate) fn GetField(&self, field_id: FieldId) -> Option<&RareField> {
        if self.bit_field_ & Self::FieldIdBit(field_id) != 0 {
            Some(&self.field_list_[self.GetFieldIndex(field_id) as usize])
        } else {
            None
        }
    }

    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:135-149
    pub(crate) fn EnsureFieldWithPolicy<const ALLOW_OVERWRITE: bool>(
        &mut self,
        field_id: FieldId,
    ) -> &mut RareField {
        let field_id_bit = Self::FieldIdBit(field_id);
        if ALLOW_OVERWRITE {
            if self.bit_field_ & field_id_bit != 0 {
                let index = self.GetFieldIndex(field_id) as usize;
                return &mut self.field_list_[index];
            }
        } else {
            debug_assert_eq!(self.bit_field_ & field_id_bit, 0);
        }
        self.bit_field_ |= field_id_bit;
        let index = self.GetFieldIndex(field_id) as usize;
        self.field_list_.insert(index, RareField::new(field_id));
        &mut self.field_list_[index]
    }

    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:151-152
    pub(crate) fn SetField(&mut self, field_id: FieldId) -> &mut RareField {
        self.EnsureFieldWithPolicy::<false>(field_id)
    }

    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:154-157
    pub(crate) fn EnsureField(&mut self, field_id: FieldId) -> &mut RareField {
        self.EnsureFieldWithPolicy::<true>(field_id)
    }

    // cpp: layoutng_fragment_tree/physical_fragment_rare_data.h:159-163
    pub(crate) fn RemoveField(&mut self, field_id: FieldId) {
        self.field_list_.remove(self.GetFieldIndex(field_id) as usize);
        self.bit_field_ &= !Self::FieldIdBit(field_id);
    }
}
