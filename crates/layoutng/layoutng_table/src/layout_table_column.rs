#![allow(non_snake_case)]
use crate::{layout_table::LayoutTable, table_break_token_data::TableBreakTokenData};
use foundation::paint_invalidation_reason::PaintInvalidationReason;
use foundation::{
    DynamicTo, EDisplay, LayoutUnit, PhysicalOffset, PhysicalRect, PhysicalSize, To, Traceable,
    Vector, Visitor,
};
use layoutng_assembly::{
    internal::{
        block_node::BlockNode,
        layout_box::LayoutBox,
        layout_box_model_object::LayoutBoxModelObject,
        layout_invalidation_reason,
        layout_node_metadata::Element,
        layout_object::{LayoutObject, LayoutObjectClass, StyleChangeContext},
        layout_object_child_list::LayoutObjectChildList,
        map_coordinates_flags::MapCoordinatesFlags,
        table_layout_algorithm_types::{TableGroupedChildren, TableTypes},
    },
    physical_box_fragment::PhysicalBoxFragment,
};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_geometry::geometry::{
    logical_offset::LogicalOffset,
    logical_rect::LogicalRect,
    logical_size::{LogicalSize, ToPhysicalSize},
    writing_mode_converter::WritingModeConverter,
};
use layoutng_style::style::{computed_style::ComputedStyle, style_difference::StyleDifference};
use std::ops::{Deref, DerefMut};

// cpp: layoutng_table/layout_table_column.cc:21-24
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInputNodeTableColumnSpanFromTable(
    node: &layoutng_assembly::internal::layout_input_node::LayoutInputNode,
) -> u32 {
    debug_assert!(node.IsTableCol() || node.IsTableColgroup());
    unsafe { &*To::<LayoutTableColumn>(node.GetLayoutBox()) }.Span()
}
// cpp: layoutng_table/layout_table_column.cc:32-52
fn TableHasColumnsWithBackground(table: *mut LayoutTable) -> bool {
    let grouped = TableGroupedChildren::new(&BlockNode::new(table.cast()));
    for column in &grouped.columns {
        if column.Style().HasBackground() {
            return true;
        }
        if column.IsTableColgroup() {
            let mut child = column.FirstChild();
            while child.is_non_null() {
                debug_assert!(child.IsTableCol());
                if child.Style().HasBackground() {
                    return true;
                }
                child = child.NextSibling();
            }
        }
    }
    false
}
// cpp: layoutng_table/layout_table_column.h:17-172
#[repr(C)]
pub struct LayoutTableColumn {
    base: LayoutBox,
    span: u32,
    children: LayoutObjectChildList,
    column_index: Option<usize>,
}
// cpp: layoutng_table/layout_table_column.h:129-157
pub struct SynthesizedFragment<'a> {
    pub rect: PhysicalRect,
    pub additional_offset_from_table_fragment: PhysicalOffset,
    pub table_fragment: &'a PhysicalBoxFragment,
}
impl LayoutTableColumn {
    // cpp: layoutng_table/layout_table_column.cc:57-59
    pub fn new(element: *mut Element) -> Self {
        let base = LayoutBox::new(element.cast());
        base.SetRuntimeClass(LayoutObjectClass::TableColumn);
        let mut column = Self {
            base,
            span: 1,
            children: LayoutObjectChildList::default(),
            column_index: None,
        };
        column.UpdateFromElement();
        column
    }
    // cpp: layoutng_table/layout_table_column.h:24-37,77-80
    pub fn IsColumn(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().Display() == EDisplay::kTableColumn
    }
    pub fn IsColumnGroup(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().Display() == EDisplay::kTableColumnGroup
    }
    pub fn Span(&self) -> u32 {
        self.CheckIsNotDestroyed();
        self.span
    }
    pub fn SetColumnIndex(&mut self, index: usize) {
        self.CheckIsNotDestroyed();
        self.column_index = Some(index);
    }
    fn ColumnIndex(&self) -> usize {
        self.column_index
            .expect("table column index has not been assigned by layout")
    }
    // cpp: layoutng_table/layout_table_column.h:117-126
    pub fn VirtualChildren(&self) -> *mut LayoutObjectChildList {
        self.CheckIsNotDestroyed();
        (&self.children as *const LayoutObjectChildList).cast_mut()
    }
    // cpp: layoutng_table/layout_table_column.cc:66-107
    pub fn StyleDidChange(
        &mut self,
        diff: StyleDifference,
        old: *const ComputedStyle,
        new: &ComputedStyle,
        context: &StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
        if diff.HasDifference() {
            if let Some(table) = unsafe { self.Table().as_mut() } {
                if let Some(old) = unsafe { old.as_ref() } {
                    if diff.NeedsNormalPaintInvalidation() {
                        if !old.BorderVisuallyEqual(new) {
                            table.GridBordersChanged();
                        }
                        if new.HasBackground() || old.HasBackground() {
                            table.SetBackgroundNeedsFullPaintInvalidation();
                        }
                    }
                }
                if diff.NeedsFullLayout() {
                    table.SetIntrinsicLogicalWidthsDirty(layoutng_assembly::internal::layout_object::MarkingBehavior::kMarkContainerChain);
                    if let Some(old) = unsafe { old.as_ref() } {
                        if TableTypes::CreateColumn(
                            old,
                            None,
                            table.StyleRef().IsFixedTableLayout(),
                        ) != TableTypes::CreateColumn(
                            new,
                            None,
                            table.StyleRef().IsFixedTableLayout(),
                        ) {
                            table.GridBordersChanged();
                        }
                    }
                }
            }
        }
        self.base.StyleDidChange(diff, old, new, context);
    }
    pub fn ImageChanged(
        &mut self,
        _image: layoutng_assembly::internal::loader::resource::image_resource_observer::WrappedImagePtr,
        _can_defer: layoutng_assembly::internal::loader::resource::image_resource_observer::CanDeferInvalidation,
    ) {
        self.CheckIsNotDestroyed();
        if let Some(table) = unsafe { self.Table().as_mut() } {
            table.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
                PaintInvalidationReason::kImage,
            );
        }
    }
    // cpp: layoutng_table/layout_table_column.cc:109-127
    pub fn InsertedIntoTree(&mut self) {
        self.CheckIsNotDestroyed();
        self.base.InsertedIntoTreeBase();
        let table = unsafe { &mut *self.Table() };
        if self.StyleRef().HasBackground() {
            table.SetBackgroundNeedsFullPaintInvalidation();
        }
        table.TableGridStructureChanged();
    }
    pub fn WillBeRemovedFromTree(&mut self) {
        self.CheckIsNotDestroyed();
        self.base.WillBeRemovedFromTreeBase();
        let table = unsafe { &mut *self.Table() };
        if self.StyleRef().HasBackground() {
            table.SetBackgroundNeedsFullPaintInvalidation();
        }
        table.TableGridStructureChanged();
    }
    // cpp: layoutng_table/layout_table_column.cc:129-148
    pub fn IsChildAllowed(&self, child: *mut LayoutObject, style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { &*child }.IsLayoutTableCol() && style.Display() == EDisplay::kTableColumn
    }
    pub fn CanHaveChildren(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsColumnGroup()
    }
    pub fn ClearNeedsLayoutForChildren(&self) {
        self.CheckIsNotDestroyed();
        let mut child = self.children.FirstChild();
        while !child.is_null() {
            unsafe { &mut *child }.ClearNeedsLayout();
            child = unsafe { &*child }.NextSibling();
        }
    }
    // cpp: layoutng_table/layout_table_column.cc:150-180
    pub fn Table(&self) -> *mut LayoutTable {
        self.CheckIsNotDestroyed();
        let mut table = self.Parent();
        if !table.is_null() && !unsafe { &*table }.IsTable() {
            table = unsafe { &*table }.Parent();
        }
        if !table.is_null() {
            debug_assert!(unsafe { &*table }.IsTable());
        }
        To::<LayoutTable>(table)
    }
    pub fn UpdateFromElement(&mut self) {
        self.CheckIsNotDestroyed();
        let element = DynamicTo::<Element>(self.GetNode());
        let old = self.span;
        self.span = unsafe { element.as_ref() }
            .and_then(|e| e.InputElementData().as_ref())
            .map_or(1, |d| d.column_span);
        if self.span == old {
            return;
        }
        let table = self.Table();
        if !table.is_null() {
            self.SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(
                std::ptr::addr_of!(layout_invalidation_reason::kAttributeChanged),
            );
            unsafe { &mut *table }.GridBordersChanged();
            if self.StyleRef().HasBackground() || TableHasColumnsWithBackground(table) {
                unsafe { &mut *table }.SetBackgroundNeedsFullPaintInvalidation();
            }
        }
    }
    // cpp: layoutng_table/layout_table_column.cc:182-222
    pub fn StitchedSize(&self) -> PhysicalSize {
        self.CheckIsNotDestroyed();
        let table = unsafe { &*self.Table() };
        if table.PhysicalFragmentCount() == 0 {
            return PhysicalSize::default();
        }
        let direction = self.StyleRef().GetWritingDirection();
        let mut size = LogicalSize::default();
        let mut found = false;
        for fragment in table.PhysicalFragments() {
            let geometry = fragment.TableColumnGeometries();
            if !found && !geometry.is_null() {
                let geometries = unsafe { &*geometry };
                let index = self.ColumnIndex();
                if index >= geometries.len() {
                    return PhysicalSize::default();
                }
                let g = &geometries[index];
                if g.node.GetLayoutBox() != (self as *const Self).cast_mut().cast() {
                    return PhysicalSize::default();
                }
                found = true;
                size.inline_size = g.inline_size;
                size.block_size -= table.StyleRef().TableBorderSpacing().block_size * 2;
            }
            size.block_size += fragment.TableGridRect().size.block_size
                - (fragment.Padding().ConvertToLogical(direction).BlockSum()
                    + fragment.Borders().ConvertToLogical(direction).BlockSum());
        }
        ToPhysicalSize(size, table.StyleRef().GetWritingMode())
    }
    // cpp: layoutng_table/layout_table_column.cc:224-256
    pub fn PhysicalLocation(&self) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        let table = unsafe { &*self.Table() };
        if table.PhysicalFragmentCount() == 0 {
            debug_assert!(self.NeedsLayout());
            return PhysicalOffset::default();
        }
        let group = if self.IsColumn() {
            DynamicTo::<Self>(self.Parent())
        } else {
            std::ptr::null_mut()
        };
        debug_assert!(group.is_null() || unsafe { &*group }.IsColumnGroup());
        let first = unsafe { &*table.GetPhysicalFragment(0) };
        let mut offset = PhysicalOffset::default();
        self.ForAllSynthesizedFragments(|f| {
            offset = f.rect.offset;
            if group.is_null() {
                offset += f.table_fragment.OffsetFromRootFragmentationContext()
                    - first.OffsetFromRootFragmentationContext();
            }
            false
        });
        offset
    }
    // cpp: layoutng_table/layout_table_column.cc:258-278
    pub fn BoundingBoxRelativeToFirstFragment(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let mut first = None;
        let mut bounds = PhysicalRect::default();
        self.ForAllSynthesizedFragments(|f| {
            let offset = f.additional_offset_from_table_fragment
                + f.rect.offset
                + f.table_fragment.OffsetFromRootFragmentationContext();
            let first = *first.get_or_insert(offset);
            bounds.UniteEvenIfEmpty(&PhysicalRect::new(offset - first, f.rect.size));
            true
        });
        bounds
    }
    // cpp: layoutng_table/layout_table_column.cc:280-320
    pub fn QuadsInAncestorInternal(
        &self,
        quads: &mut Vector<foundation::gfx::QuadF>,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
        _quad_type: layoutng_assembly::internal::layout_object::BoxQuadType,
    ) {
        self.CheckIsNotDestroyed();
        let mut first = None;
        self.ForAllSynthesizedFragments(|f| {
            let absolute = f.additional_offset_from_table_fragment
                + f.rect.offset
                + f.table_fragment.OffsetFromRootFragmentationContext();
            let offset = if let Some(first) = first {
                absolute - first
            } else {
                first = Some(absolute);
                PhysicalOffset::default()
            };
            quads.push(self.LocalRectToAncestorQuad(
                &PhysicalRect::new(offset, f.rect.size),
                ancestor,
                mode,
            ));
            true
        });
        if quads.is_empty() {
            quads.push(self.LocalRectToAncestorQuad(&PhysicalRect::default(), ancestor, mode));
        }
    }
    // cpp: layoutng_table/layout_table_column.cc:322-448
    pub fn ForAllSynthesizedFragments(
        &self,
        mut callback: impl FnMut(SynthesizedFragment<'_>) -> bool,
    ) {
        self.CheckIsNotDestroyed();
        let table = unsafe { &*self.Table() };
        debug_assert!(table.PhysicalFragmentCount() > 0);
        let direction = self.StyleRef().GetWritingDirection();
        let group = if self.IsColumn() {
            DynamicTo::<Self>(self.Parent())
        } else {
            std::ptr::null_mut()
        };
        debug_assert!(group.is_null() || unsafe { &*group }.IsColumnGroup());
        for fragment in table.PhysicalFragments() {
            let geometry = fragment.TableColumnGeometries();
            if geometry.is_null() {
                return;
            }
            let geometry = unsafe { &*geometry };
            let index = self.ColumnIndex();
            if index >= geometry.len() {
                return;
            }
            let g = &geometry[index];
            if g.node.GetLayoutBox() != (self as *const Self).cast_mut().cast() {
                return;
            }
            let token = fragment.GetBreakToken();
            let data = if token.is_null() {
                std::ptr::null()
            } else {
                DynamicTo::<TableBreakTokenData>(unsafe { &*token }.TokenData())
            };
            if !data.is_null() && !unsafe { &*data }.has_entered_table_box {
                continue;
            }
            let mut rect = LogicalRect::default();
            rect.offset.inline_offset = g.inline_offset;
            rect.size.inline_size = g.inline_size;
            let converter = WritingModeConverter::new(direction, fragment.Size());
            let mut sections = LogicalRect::default();
            for child in fragment.Children() {
                if child.IsLayoutObjectDestroyedOrMoved() {
                    continue;
                }
                if child.IsTableSection() {
                    sections.Unite(
                        &converter.ToLogicalRect(PhysicalRect::new(child.offset, child.Size())),
                    );
                }
            }
            sections.offset.inline_offset = rect.offset.inline_offset;
            sections.size.inline_size = rect.size.inline_size;
            rect.Unite(&sections);
            let mut container_size = fragment.Size();
            let mut additional = PhysicalOffset::default();
            if !group.is_null() {
                let parent = &geometry[unsafe { &*group }.ColumnIndex()];
                let extra = LogicalOffset::new(parent.inline_offset, rect.offset.block_offset);
                rect.offset.inline_offset -= parent.inline_offset;
                rect.offset.block_offset = LayoutUnit::default();
                container_size = ToPhysicalSize(
                    LogicalSize::new(parent.inline_size, sections.size.block_size),
                    direction.GetWritingMode(),
                );
                additional = converter.ToPhysicalOffset(extra, container_size);
            } else {
                let decorations =
                    (fragment.Padding() + fragment.Borders()).ConvertToLogical(direction);
                rect.offset.inline_offset +=
                    decorations.inline_start + table.StyleRef().TableBorderSpacing().inline_size;
            }
            let rect = WritingModeConverter::new(direction, container_size).ToPhysicalRect(rect);
            if !callback(SynthesizedFragment {
                rect,
                additional_offset_from_table_fragment: additional,
                table_fragment: fragment,
            }) {
                return;
            }
            if !data.is_null() && unsafe { &*data }.is_past_table_box {
                return;
            }
        }
    }
}

impl Deref for LayoutTableColumn {
    type Target = LayoutBox;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for LayoutTableColumn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutTableColumn, base) == 0);
impl foundation::DowncastFrom<LayoutObject> for LayoutTableColumn {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsLayoutTableCol()
    }
}
impl foundation::DowncastFrom<LayoutBox> for LayoutTableColumn {
    fn AllowFrom(object: &LayoutBox) -> bool {
        object.IsLayoutTableCol()
    }
}
impl Traceable for LayoutTableColumn {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.children.Trace(visitor);
        self.base.Trace(visitor);
    }
}
