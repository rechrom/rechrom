#![allow(non_snake_case)]
use crate::{
    layout_table::LayoutTable, layout_table_row::LayoutTableRow,
    layout_table_section::LayoutTableSection, table_constants::*,
};
use foundation::{DynamicTo, EDisplay, MakeGarbageCollected, To, Traceable, Visitor};
use layoutng_assembly::internal::{
    layout_block::LayoutBlock,
    layout_block_flow::LayoutBlockFlow,
    layout_box::LayoutBox,
    layout_invalidation_reason,
    layout_node_metadata::Element,
    layout_object::{LayoutObject, LayoutObjectClass, StyleChangeContext},
};
use layoutng_geometry::geometry::box_strut::{BoxStrut, PhysicalBoxStrut};
use layoutng_style::style::{
    anonymous_style::CreateAnonymousStyleBuilderWithDisplay, computed_style::ComputedStyle,
    style_difference::StyleDifference,
};
use std::ops::{Deref, DerefMut};
// cpp: layoutng_table/layout_table_cell.cc:22-30
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInputNodeTableCellColspanFromTable(
    node: &layoutng_assembly::internal::layout_input_node::LayoutInputNode,
) -> u32 {
    debug_assert!(unsafe { &*node.GetLayoutBox() }.IsTableCell());
    unsafe { &*To::<LayoutTableCell>(node.GetLayoutBox()) }.ColSpan()
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInputNodeTableCellRowspanFromTable(
    node: &layoutng_assembly::internal::layout_input_node::LayoutInputNode,
) -> u32 {
    debug_assert!(unsafe { &*node.GetLayoutBox() }.IsTableCell());
    unsafe { &*To::<LayoutTableCell>(node.GetLayoutBox()) }.ComputedRowSpan()
}
// cpp: layoutng_table/layout_table_cell.h:18-143
#[repr(C)]
pub struct LayoutTableCell {
    base: LayoutBlockFlow,
    intrinsic_borders: BoxStrut,
    has_col_span: bool,
    has_rowspan: bool,
}
impl LayoutTableCell {
    // cpp: layoutng_table/layout_table_cell.cc:32-47
    pub fn new(element: *mut Element) -> Self {
        let base = LayoutBlockFlow::new(element.cast());
        base.SetRuntimeClass(LayoutObjectClass::TableCell);
        let mut cell = Self {
            base,
            intrinsic_borders: BoxStrut::default(),
            has_col_span: false,
            has_rowspan: false,
        };
        cell.UpdateColAndRowSpanFlags();
        cell
    }
    pub fn CreateAnonymousWithParent(parent: &LayoutObject) -> *mut Self {
        let mut builder = CreateAnonymousStyleBuilderWithDisplay(
            parent.StyleRef(),
            EDisplay::kTableCell,
            parent.StyleRef().AppliedTextDecorationData(),
        );
        parent.UpdateAnonymousChildStyle(std::ptr::null(), &mut builder);
        let object = MakeGarbageCollected(Self::new(std::ptr::null_mut()));
        unsafe { &mut *object }.SetInputOwnerForAnonymous(parent);
        unsafe { &mut *object }.SetStyle(builder.TakeStyle());
        object
    }
    // cpp: layoutng_table/layout_table_cell.h:25-33
    pub fn ComputedRowSpan(&self) -> u32 {
        self.CheckIsNotDestroyed();
        if !self.has_rowspan {
            return 1;
        }
        let span = self.ParseRowSpanFromDOM();
        if span == 0 {
            kMaxRowSpan
        } else {
            span
        }
    }
    // cpp: layoutng_table/layout_table_cell.h:35-44
    pub fn IntrinsicLogicalWidthsBorderSizes(&self) -> &BoxStrut {
        self.CheckIsNotDestroyed();
        &self.intrinsic_borders
    }
    pub fn SetIntrinsicLogicalWidthsBorderSizes(&mut self, borders: &BoxStrut) {
        self.CheckIsNotDestroyed();
        self.intrinsic_borders = *borders;
    }
    // cpp: layoutng_table/layout_table_cell.cc:49-70
    pub fn InvalidateLayoutResultCacheAfterMeasure(&self) {
        self.CheckIsNotDestroyed();
        if let Some(row) = unsafe { self.Row().as_mut() } {
            row.SetShouldSkipLayoutCache(true);
            if let Some(section) = unsafe { row.Section().as_mut() } {
                section.SetShouldSkipLayoutCache(true);
            }
        }
    }
    pub fn BorderOutsets(&self) -> PhysicalBoxStrut {
        self.CheckIsNotDestroyed();
        if unsafe { &*self.Table() }.HasCollapsedBorders() && self.PhysicalFragmentCount() > 0 {
            unsafe { &*self.GetPhysicalFragment(0) }.Borders()
        } else {
            self.base.BorderOutsetsBase()
        }
    }
    // cpp: layoutng_table/layout_table_cell.cc:72-100
    pub fn NextCell(&self) -> *mut Self {
        self.CheckIsNotDestroyed();
        To::<Self>(self.NextSibling())
    }
    pub fn PreviousCell(&self) -> *mut Self {
        self.CheckIsNotDestroyed();
        To::<Self>(self.PreviousSibling())
    }
    pub fn Row(&self) -> *mut LayoutTableRow {
        self.CheckIsNotDestroyed();
        To::<LayoutTableRow>(self.Parent())
    }
    pub fn Section(&self) -> *mut LayoutTableSection {
        self.CheckIsNotDestroyed();
        To::<LayoutTableSection>(unsafe { &*self.Parent() }.Parent())
    }
    pub fn Table(&self) -> *mut LayoutTable {
        self.CheckIsNotDestroyed();
        let parent = self.Parent();
        if parent.is_null() {
            return std::ptr::null_mut();
        }
        let grand = unsafe { &*parent }.Parent();
        if grand.is_null() {
            std::ptr::null_mut()
        } else {
            To::<LayoutTable>(unsafe { &*grand }.Parent())
        }
    }
    // cpp: layoutng_table/layout_table_cell.cc:102-160
    pub fn StyleDidChange(
        &mut self,
        diff: StyleDifference,
        old: *const ComputedStyle,
        new: &ComputedStyle,
        context: &StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
        if let Some(table) = unsafe { self.Table().as_mut() } {
            if let Some(old) = unsafe { old.as_ref() } {
                if !old.BorderVisuallyEqual(new)
                    || old.GetWritingDirection() != new.GetWritingDirection()
                {
                    table.GridBordersChanged();
                }
            }
        }
        self.base.StyleDidChange(diff, old, new, context);
    }
    pub fn WillBeRemovedFromTree(&mut self) {
        self.CheckIsNotDestroyed();
        if let Some(table) = unsafe { self.Table().as_mut() } {
            table.TableGridStructureChanged();
        }
        self.base.WillBeRemovedFromTreeBase();
    }
    pub fn ColSpanOrRowSpanChanged(&mut self) {
        self.CheckIsNotDestroyed();
        self.UpdateColAndRowSpanFlags();
        if let Some(table) = unsafe { self.Table().as_mut() } {
            table.SetNeedsLayoutAndIntrinsicWidthsRecalc(std::ptr::addr_of!(
                layout_invalidation_reason::kTableChanged
            ));
            table.TableGridStructureChanged();
        }
    }
    pub fn CreateAnonymousBoxWithSameTypeAs(&self, parent: *const LayoutObject) -> *mut LayoutBox {
        self.CheckIsNotDestroyed();
        Self::CreateAnonymousWithParent(unsafe { &*parent }).cast()
    }
    pub fn StickyContainer(&self) -> *mut LayoutBlock {
        self.CheckIsNotDestroyed();
        self.Table().cast()
    }
    pub fn RowIndex(&self) -> u32 {
        self.CheckIsNotDestroyed();
        unsafe { &*To::<LayoutTableRow>(self.Parent()) }.RowIndex()
    }
    pub fn ResolvedRowSpan(&self) -> u32 {
        self.CheckIsNotDestroyed();
        if self.has_rowspan {
            self.ParseRowSpanFromDOM()
        } else {
            1
        }
    }
    // cpp: layoutng_table/layout_table_cell.cc:162-168
    pub fn AbsoluteColumnIndex(&self) -> u32 {
        self.CheckIsNotDestroyed();
        if self.PhysicalFragmentCount() > 0 {
            unsafe { &*self.GetPhysicalFragment(0) }.TableCellColumnIndex()
        } else {
            panic!("AbsoluteColumnIndex did not find cell");
        }
    }
    // cpp: layoutng_table/layout_table_cell.h:131-136
    #[allow(dead_code)]
    fn ParsedRowSpan(&self) -> u32 {
        self.CheckIsNotDestroyed();
        if !self.has_rowspan {
            return 1;
        }
        self.ParseRowSpanFromDOM()
    }
    // cpp: layoutng_table/layout_table_cell.cc:170-201
    pub fn ColSpan(&self) -> u32 {
        self.CheckIsNotDestroyed();
        if self.has_col_span {
            self.ParseColSpanFromDOM()
        } else {
            1
        }
    }
    fn ParseColSpanFromDOM(&self) -> u32 {
        self.CheckIsNotDestroyed();
        let element = DynamicTo::<Element>(self.GetNode());
        let span = unsafe { element.as_ref() }
            .and_then(|e| e.InputElementData().as_ref())
            .map_or(kDefaultColSpan, |d| d.column_span);
        assert!(span >= kMinColSpan && span <= kMaxColSpan);
        span
    }
    fn ParseRowSpanFromDOM(&self) -> u32 {
        self.CheckIsNotDestroyed();
        let element = DynamicTo::<Element>(self.GetNode());
        let span = unsafe { element.as_ref() }
            .and_then(|e| e.InputElementData().as_ref())
            .map_or(kDefaultRowSpan, |d| d.row_span);
        assert!(span <= kMaxRowSpan);
        span
    }
    fn UpdateColAndRowSpanFlags(&mut self) {
        self.CheckIsNotDestroyed();
        self.has_col_span = self.ParseColSpanFromDOM() != kDefaultColSpan;
        self.has_rowspan = self.ParseRowSpanFromDOM() != kDefaultRowSpan;
    }
}

impl Deref for LayoutTableCell {
    type Target = LayoutBlockFlow;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for LayoutTableCell {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutTableCell, base) == 0);
impl foundation::DowncastFrom<LayoutObject> for LayoutTableCell {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsTableCell()
    }
}
impl foundation::DowncastFrom<LayoutBox> for LayoutTableCell {
    fn AllowFrom(object: &LayoutBox) -> bool {
        object.IsTableCell()
    }
}
impl Traceable for LayoutTableCell {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.base.Trace(visitor);
    }
}
