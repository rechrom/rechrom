#![allow(non_snake_case)]
use crate::{layout_table_cell::LayoutTableCell, layout_table_section::LayoutTableSection};
use foundation::{
    infinite_int_rect::InfiniteIntRect, paint_invalidation_reason::PaintInvalidationReason,
};
use foundation::{
    EBorderCollapse, EDisplay, LayoutUnit, MakeGarbageCollected, Member,
    OverlayScrollbarClipBehavior, PhysicalOffset, PhysicalRect, To, Traceable, Visitor,
};
use layoutng_assembly::internal::{
    block_node::BlockNode,
    layout_block::LayoutBlock,
    layout_box::LayoutBox,
    layout_invalidation_reason,
    layout_node_metadata::Element,
    layout_object::{LayoutObject, LayoutObjectClass, StyleChangeContext},
    table_borders::TableBorders,
    table_layout_algorithm_types::{Columns, TableGroupedChildren},
};
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_style::style::{
    anonymous_style::CreateAnonymousStyleBuilderWithDisplay, computed_style::ComputedStyle,
    style_difference::StyleDifference,
};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

// cpp: layoutng_table/layout_table.h:96-230
#[repr(C)]
pub struct LayoutTable {
    base: LayoutBlock,
    cached_table_borders: Member<TableBorders>,
    cached_table_columns: Option<Arc<Columns>>,
}
impl LayoutTable {
    // cpp: layoutng_table/layout_table.cc:26-28
    pub fn new(element: *mut Element) -> Self {
        let base = LayoutBlock::new(element.cast());
        base.SetRuntimeClass(LayoutObjectClass::Table);
        Self {
            base,
            cached_table_borders: Member::default(),
            cached_table_columns: None,
        }
    }
    // cpp: layoutng_table/layout_table.cc:35-53
    pub fn CreateAnonymousWithParent(parent: &LayoutObject) -> *mut Self {
        let display = if parent.IsLayoutInline() {
            EDisplay::kInlineTable
        } else {
            EDisplay::kTable
        };
        let mut builder = CreateAnonymousStyleBuilderWithDisplay(
            parent.StyleRef(),
            display,
            parent.StyleRef().AppliedTextDecorationData(),
        );
        parent.UpdateAnonymousChildStyle(std::ptr::null(), &mut builder);
        let object = MakeGarbageCollected(Self::new(std::ptr::null_mut()));
        unsafe { &mut *object }.SetInputOwnerForAnonymous(parent);
        unsafe { &mut *object }.SetStyle(builder.TakeStyle());
        object
    }
    // cpp: layoutng_table/layout_table.cc:54-69
    pub fn IsFirstCell(&self, cell: &LayoutTableCell) -> bool {
        self.CheckIsNotDestroyed();
        let row = unsafe { &*cell.Row() };
        if row.FirstCell() != cell as *const _ as *mut _ {
            return false;
        }
        let section = unsafe { &*row.Section() };
        if section.FirstRow() != row as *const _ as *mut _ {
            return false;
        }
        let grouped = self.Grouped();
        let it = grouped.begin();
        !it.Equals(&grouped.end())
            && it.Dereference().GetLayoutBox() == section as *const _ as *mut LayoutBox
    }
    fn Grouped(&self) -> TableGroupedChildren {
        TableGroupedChildren::new(&BlockNode::new((self as *const Self).cast_mut().cast()))
    }
    // cpp: layoutng_table/layout_table.cc:71-101
    pub fn FirstSection(&self) -> *mut LayoutTableSection {
        self.CheckIsNotDestroyed();
        let grouped = self.Grouped();
        let it = grouped.begin();
        if it.Equals(&grouped.end()) {
            return std::ptr::null_mut();
        }
        let node = it.Dereference();
        let section = To::<LayoutTableSection>(node.GetLayoutBox());
        if node.IsEmptyTableSection() {
            self.NextSection(section)
        } else {
            section
        }
    }
    pub fn LastSection(&self) -> *mut LayoutTableSection {
        self.CheckIsNotDestroyed();
        let grouped = self.Grouped();
        let mut it = grouped.end();
        it.Decrement();
        if it.Equals(&grouped.end()) {
            return std::ptr::null_mut();
        }
        let node = it.Dereference();
        let section = To::<LayoutTableSection>(node.GetLayoutBox());
        if node.IsEmptyTableSection() {
            self.PreviousSection(section)
        } else {
            section
        }
    }
    // cpp: layoutng_table/layout_table.cc:103-137
    pub fn NextSection(&self, current: *const LayoutTableSection) -> *mut LayoutTableSection {
        self.CheckIsNotDestroyed();
        let grouped = self.Grouped();
        let mut it = grouped.begin();
        let mut found = false;
        while !it.Equals(&grouped.end()) {
            let node = it.Dereference();
            let section = To::<LayoutTableSection>(node.GetLayoutBox());
            if found && !node.IsEmptyTableSection() {
                return section;
            }
            if section.cast_const() == current {
                found = true;
            }
            it.Increment();
        }
        std::ptr::null_mut()
    }
    pub fn PreviousSection(&self, current: *const LayoutTableSection) -> *mut LayoutTableSection {
        self.CheckIsNotDestroyed();
        let grouped = self.Grouped();
        let mut stop = grouped.begin();
        stop.Decrement();
        let mut it = grouped.end();
        it.Decrement();
        let mut found = false;
        while !it.Equals(&stop) {
            let node = it.Dereference();
            let section = To::<LayoutTableSection>(node.GetLayoutBox());
            if found && !node.IsEmptyTableSection() {
                return section;
            }
            if section.cast_const() == current {
                found = true;
            }
            it.Decrement();
        }
        std::ptr::null_mut()
    }
    // cpp: layoutng_table/layout_table.cc:139-145
    pub fn ColumnCount(&self) -> u32 {
        self.CheckIsNotDestroyed();
        let result = self.GetCachedLayoutResult(std::ptr::null());
        if result.is_null() {
            0
        } else {
            unsafe { &*result }.TableColumnCount()
        }
    }
    // cpp: layoutng_table/layout_table.h:115-118
    pub fn GetCachedTableBorders(&self) -> *const TableBorders {
        self.CheckIsNotDestroyed();
        self.cached_table_borders.Get()
    }
    // cpp: layoutng_table/layout_table.cc:147-173
    pub fn SetCachedTableBorders(&mut self, borders: *const TableBorders) {
        self.CheckIsNotDestroyed();
        self.cached_table_borders = Member::from_ptr(borders.cast_mut());
    }
    pub fn InvalidateCachedTableBorders(&mut self) {
        self.CheckIsNotDestroyed();
        self.cached_table_borders = Member::default();
    }
    pub fn GetCachedTableColumnConstraints(&mut self) -> *const Columns {
        self.CheckIsNotDestroyed();
        if self.IsTableColumnConstraintsDirty() {
            self.cached_table_columns = None;
        }
        self.cached_table_columns
            .as_ref()
            .map_or(std::ptr::null(), Arc::as_ptr)
    }
    pub fn CloneCachedTableColumnConstraints(&mut self) -> Option<Arc<Columns>> {
        self.GetCachedTableColumnConstraints();
        self.cached_table_columns.clone()
    }
    pub fn SetCachedTableColumnConstraints(&mut self, columns: Option<Arc<Columns>>) {
        self.CheckIsNotDestroyed();
        self.cached_table_columns = columns;
        self.SetTableColumnConstraintsDirty(false);
    }
    // cpp: layoutng_table/layout_table.cc:175-193
    pub fn GridBordersChanged(&mut self) {
        self.CheckIsNotDestroyed();
        self.InvalidateCachedTableBorders();
        if self.HasCollapsedBorders() {
            self.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
                PaintInvalidationReason::kStyle,
            );
            self.SetNeedsLayoutAndIntrinsicWidthsRecalc(std::ptr::addr_of!(
                layout_invalidation_reason::kTableChanged
            ));
        }
    }
    pub fn TableGridStructureChanged(&mut self) {
        self.CheckIsNotDestroyed();
        self.InvalidateCachedTableBorders();
        if self.HasCollapsedBorders() {
            self.SetShouldDoFullPaintInvalidation();
        }
    }
    // cpp: layoutng_table/layout_table.cc:195-209
    pub fn HasBackgroundForPaint(&self) -> bool {
        self.CheckIsNotDestroyed();
        if self.StyleRef().HasBackground() {
            return true;
        }
        debug_assert!(self.PhysicalFragmentCount() > 0);
        let geometry = unsafe { &*self.GetPhysicalFragment(0) }.TableColumnGeometries();
        if !geometry.is_null() {
            for column in unsafe { &*geometry } {
                if column.node.Style().HasBackground() {
                    return true;
                }
            }
        }
        false
    }
    // cpp: layoutng_table/layout_table.cc:211-268
    pub fn AddChild(&mut self, child: *mut LayoutObject, before: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        self.TableGridStructureChanged();
        let this = (self as *mut Self).cast::<LayoutObject>();
        let allowed = unsafe { &*child }.IsTableCaption()
            || unsafe { &*child }.IsLayoutTableCol()
            || unsafe { &*child }.IsTableSection();
        if !before.is_null() && unsafe { &*before }.Parent() != this {
            if allowed {
                let direct = self.SplitAnonymousBoxesAroundChild(before);
                debug_assert_eq!(unsafe { &*direct }.Parent(), this);
                self.AddChild(child, direct);
            } else {
                let mut container = unsafe { &*before }.Parent();
                while unsafe { &*container }.Parent() != this {
                    container = unsafe { &*container }.Parent();
                }
                assert!(
                    unsafe { &*container }.IsAnonymous() && unsafe { &*container }.IsTableSection()
                );
                unsafe { &mut *container }.AddChild(child, before);
            }
            return;
        }
        if !allowed {
            let after = if before.is_null() {
                self.LastChild()
            } else {
                unsafe { &*before }.PreviousSibling()
            };
            if !after.is_null() && unsafe { &*after }.IsAnonymous() {
                unsafe { &mut *after }.AddChild(child, std::ptr::null_mut());
                return;
            }
            let section = LayoutTableSection::CreateAnonymousWithParent(unsafe { &*this });
            unsafe { &mut *this }.AddChildBase(section.cast(), before);
            unsafe { &mut *section }.AddChild(child, std::ptr::null_mut());
            return;
        }
        unsafe { &mut *this }.AddChildBase(child, before);
    }
    // cpp: layoutng_table/layout_table.cc:270-274
    pub fn RemoveChild(&mut self, child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        self.TableGridStructureChanged();
        self.base.RemoveChildBase(child);
    }
    // cpp: layoutng_table/layout_table.cc:276-295
    pub fn StyleDidChange(
        &mut self,
        diff: StyleDifference,
        old: *const ComputedStyle,
        new: &ComputedStyle,
        context: &StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
        if let Some(old) = unsafe { old.as_ref() } {
            if !old.BorderVisuallyEqual(new)
                || old.GetWritingDirection() != new.GetWritingDirection()
                || old.IsFixedTableLayout() != new.IsFixedTableLayout()
                || old.EmptyCells() != new.EmptyCells()
                || old.BorderCollapse() != new.BorderCollapse()
            {
                self.GridBordersChanged();
            }
        }
        self.base.StyleDidChange(diff, old, new, context);
    }
    // cpp: layoutng_table/layout_table.cc:297-301
    pub fn CreateAnonymousBoxWithSameTypeAs(&self, parent: *const LayoutObject) -> *mut LayoutBox {
        self.CheckIsNotDestroyed();
        Self::CreateAnonymousWithParent(unsafe { &*parent }).cast()
    }
    // cpp: layoutng_table/layout_table.cc:303-340
    pub fn OverflowClipRect(&self, behavior: OverlayScrollbarClipBehavior) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let mut rect = if self.HasCollapsedBorders() {
            let mut r = PhysicalRect::new(PhysicalOffset::default(), self.StitchedSize());
            let axes = self.GetOverflowClipAxes();
            let infinite = InfiniteIntRect();
            if axes & layoutng_geometry::geometry::overflow_clip_axes::kOverflowClipX
                == layoutng_geometry::geometry::overflow_clip_axes::kNoOverflowClip
            {
                r.offset.left = LayoutUnit::from_signed(infinite.x());
                r.size.width = LayoutUnit::from_signed(infinite.width());
            }
            if axes & layoutng_geometry::geometry::overflow_clip_axes::kOverflowClipY
                == layoutng_geometry::geometry::overflow_clip_axes::kNoOverflowClip
            {
                r.offset.top = LayoutUnit::from_signed(infinite.y());
                r.size.height = LayoutUnit::from_signed(infinite.height());
            }
            r
        } else {
            self.base.OverflowClipRectWithBehaviorBase(behavior)
        };
        let mut child = self.FirstChild();
        while !child.is_null() {
            if unsafe { &*child }.IsTableCaption() {
                rect.Unite(&PhysicalRect::new(
                    PhysicalOffset::default(),
                    self.StitchedSize(),
                ));
                break;
            }
            child = unsafe { &*child }.NextSibling();
        }
        rect
    }
    // cpp: layoutng_table/layout_table.cc:342-359
    pub fn BorderOutsets(&self) -> PhysicalBoxStrut {
        self.CheckIsNotDestroyed();
        let borders = self.cached_table_borders.Get();
        if self.HasCollapsedBorders() && !borders.is_null() {
            unsafe { &*borders }
                .TableBorder()
                .ConvertToPhysical(self.StyleRef().GetWritingDirection())
        } else {
            self.base.BorderOutsetsBase()
        }
    }
    pub fn PaddingOutsets(&self) -> PhysicalBoxStrut {
        self.CheckIsNotDestroyed();
        if self.HasCollapsedBorders() {
            PhysicalBoxStrut::default()
        } else {
            self.base.PaddingOutsetsBase()
        }
    }
    // cpp: layoutng_table/layout_table.h:187-190
    pub fn HasCollapsedBorders(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().BorderCollapse() == EBorderCollapse::kCollapse
    }
    // cpp: layoutng_table/layout_table.cc:361-387
    pub fn AbsoluteColumnToEffectiveColumn(&self, index: u32) -> u32 {
        self.CheckIsNotDestroyed();
        let columns = self
            .cached_table_columns
            .as_ref()
            .expect("table column constraints are required");
        let mut effective = 0;
        for (i, c) in columns.data.iter().enumerate() {
            if i != 0 && !c.is_mergeable {
                effective += 1;
            }
            if i == index as usize {
                return effective;
            }
        }
        effective
    }
    pub fn EffectiveColumnCount(&self) -> u32 {
        self.CheckIsNotDestroyed();
        let count = self.ColumnCount();
        if count == 0 {
            0
        } else {
            self.AbsoluteColumnToEffectiveColumn(count - 1) + 1
        }
    }
}

impl Deref for LayoutTable {
    type Target = LayoutBlock;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for LayoutTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutTable, base) == 0);
impl foundation::DowncastFrom<LayoutObject> for LayoutTable {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsTable()
    }
}
impl foundation::DowncastFrom<LayoutBox> for LayoutTable {
    fn AllowFrom(object: &LayoutBox) -> bool {
        object.IsTable()
    }
}
impl Traceable for LayoutTable {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.cached_table_borders);
        self.base.Trace(visitor);
    }
}
