#![allow(non_snake_case)]
use crate::{
    layout_table::LayoutTable, layout_table_cell::LayoutTableCell,
    layout_table_section::LayoutTableSection,
};
use foundation::{EDisplay, MakeGarbageCollected, To, Traceable, Visitor};
use layoutng_assembly::internal::{
    layout_block::LayoutBlock,
    layout_box::LayoutBox,
    layout_node_metadata::Element,
    layout_object::{LayoutObject, LayoutObjectClass, StyleChangeContext},
};
use layoutng_style::style::{
    anonymous_style::CreateAnonymousStyleBuilderWithDisplay, computed_style::ComputedStyle,
    style_difference::StyleDifference,
};
use std::ops::{Deref, DerefMut};
// cpp: layoutng_table/layout_table_row.h:19-121
#[repr(C)]
pub struct LayoutTableRow {
    base: LayoutBlock,
}
impl LayoutTableRow {
    // cpp: layoutng_table/layout_table_row.cc:16-29
    pub fn new(element: *mut Element) -> Self {
        let base = LayoutBlock::new(element.cast());
        base.SetRuntimeClass(LayoutObjectClass::TableRow);
        Self { base }
    }
    pub fn CreateAnonymousWithParent(parent: &LayoutObject) -> *mut Self {
        let mut builder = CreateAnonymousStyleBuilderWithDisplay(
            parent.StyleRef(),
            EDisplay::kTableRow,
            parent.StyleRef().AppliedTextDecorationData(),
        );
        parent.UpdateAnonymousChildStyle(std::ptr::null(), &mut builder);
        let object = MakeGarbageCollected(Self::new(std::ptr::null_mut()));
        unsafe { &mut *object }.SetInputOwnerForAnonymous(parent);
        unsafe { &mut *object }.SetStyle(builder.TakeStyle());
        object
    }
    // cpp: layoutng_table/layout_table_row.cc:31-63
    pub fn FirstCell(&self) -> *mut LayoutTableCell {
        self.CheckIsNotDestroyed();
        To::<LayoutTableCell>(self.FirstChild())
    }
    pub fn LastCell(&self) -> *mut LayoutTableCell {
        self.CheckIsNotDestroyed();
        To::<LayoutTableCell>(self.LastChild())
    }
    pub fn NextRow(&self) -> *mut Self {
        self.CheckIsNotDestroyed();
        To::<Self>(self.NextSibling())
    }
    pub fn PreviousRow(&self) -> *mut Self {
        self.CheckIsNotDestroyed();
        To::<Self>(self.PreviousSibling())
    }
    pub fn Section(&self) -> *mut LayoutTableSection {
        self.CheckIsNotDestroyed();
        To::<LayoutTableSection>(self.Parent())
    }
    pub fn Table(&self) -> *mut LayoutTable {
        self.CheckIsNotDestroyed();
        let section = self.Parent();
        if section.is_null() {
            std::ptr::null_mut()
        } else {
            To::<LayoutTable>(unsafe { &*section }.Parent())
        }
    }
    // cpp: layoutng_table/layout_table_row.cc:65-118
    pub fn AddChild(&mut self, child: *mut LayoutObject, before: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        if let Some(table) = unsafe { self.Table().as_mut() } {
            table.TableGridStructureChanged();
        }
        let this = (self as *mut Self).cast::<LayoutObject>();
        if !before.is_null() && unsafe { &*before }.Parent() != this {
            if unsafe { &*child }.IsTableCell() {
                let direct = self.SplitAnonymousBoxesAroundChild(before);
                debug_assert_eq!(unsafe { &*direct }.Parent(), this);
                self.AddChild(child, direct);
            } else {
                let mut container = unsafe { &*before }.Parent();
                while unsafe { &*container }.Parent() != this {
                    container = unsafe { &*container }.Parent();
                }
                assert!(
                    unsafe { &*container }.IsAnonymous() && unsafe { &*container }.IsTableCell()
                );
                unsafe { &mut *container }.AddChild(child, before);
            }
            return;
        }
        if !unsafe { &*child }.IsTableCell() {
            let after = if before.is_null() {
                self.LastChild()
            } else {
                unsafe { &*before }.PreviousSibling()
            };
            if !after.is_null() && unsafe { &*after }.IsAnonymous() {
                unsafe { &mut *after }.AddChild(child, std::ptr::null_mut());
                return;
            }
            let cell = LayoutTableCell::CreateAnonymousWithParent(unsafe { &*this });
            unsafe { &mut *this }.AddChildBase(cell.cast(), before);
            unsafe { &mut *cell }.AddChild(child, std::ptr::null_mut());
            return;
        }
        unsafe { &mut *this }.AddChildBase(child, before);
    }
    // cpp: layoutng_table/layout_table_row.cc:120-156
    pub fn RemoveChild(&mut self, child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        if let Some(table) = unsafe { self.Table().as_mut() } {
            table.TableGridStructureChanged();
        }
        if self.StyleRef().HasBackground() {
            self.SetBackgroundNeedsFullPaintInvalidation();
        }
        self.base.RemoveChildBase(child);
    }
    pub fn WillBeRemovedFromTree(&mut self) {
        self.CheckIsNotDestroyed();
        if let Some(table) = unsafe { self.Table().as_mut() } {
            table.TableGridStructureChanged();
        }
        self.base.WillBeRemovedFromTreeBase();
    }
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
    // cpp: layoutng_table/layout_table_row.cc:158-179
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
        let this = (self as *const Self).cast_mut().cast::<LayoutObject>();
        let mut child = unsafe { &*self.Parent() }.SlowFirstChild();
        let mut index = 0;
        while !child.is_null() {
            if child == this {
                return index;
            }
            index += 1;
            child = unsafe { &*child }.NextSibling();
        }
        panic!("table row was not found");
    }
}

impl Deref for LayoutTableRow {
    type Target = LayoutBlock;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for LayoutTableRow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutTableRow, base) == 0);
impl foundation::DowncastFrom<LayoutObject> for LayoutTableRow {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsTableRow()
    }
}
impl foundation::DowncastFrom<LayoutBox> for LayoutTableRow {
    fn AllowFrom(object: &LayoutBox) -> bool {
        object.IsTableRow()
    }
}
impl Traceable for LayoutTableRow {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.base.Trace(visitor);
    }
}
