#![allow(non_snake_case)]
use crate::{layout_table::LayoutTable, layout_table_row::LayoutTableRow};
use foundation::{EDisplay, MakeGarbageCollected, To, Traceable, Visitor};
use layoutng_assembly::internal::{
    layout_block::LayoutBlock,
    layout_box::LayoutBox,
    layout_input_node::LayoutInputNode,
    layout_node_metadata::Element,
    layout_object::{LayoutObject, LayoutObjectClass, StyleChangeContext},
};
use layoutng_style::style::{
    anonymous_style::CreateAnonymousStyleBuilderWithDisplay, computed_style::ComputedStyle,
    style_difference::StyleDifference,
};
use std::ops::{Deref, DerefMut};
// cpp: layoutng_table/layout_table_section.cc:17-20
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInputNodeIsEmptyTableSectionFromTable(node: &LayoutInputNode) -> bool {
    let box_ = unsafe { &*node.GetLayoutBox() };
    box_.IsTableSection() && unsafe { &*To::<LayoutTableSection>(node.GetLayoutBox()) }.IsEmpty()
}
// cpp: layoutng_table/layout_table_section.h:19-109
#[repr(C)]
pub struct LayoutTableSection {
    base: LayoutBlock,
}
impl LayoutTableSection {
    // cpp: layoutng_table/layout_table_section.cc:21-35
    pub fn new(element: *mut Element) -> Self {
        let base = LayoutBlock::new(element.cast());
        base.SetRuntimeClass(LayoutObjectClass::TableSection);
        Self { base }
    }
    pub fn CreateAnonymousWithParent(parent: &LayoutObject) -> *mut Self {
        let mut builder = CreateAnonymousStyleBuilderWithDisplay(
            parent.StyleRef(),
            EDisplay::kTableRowGroup,
            parent.StyleRef().AppliedTextDecorationData(),
        );
        parent.UpdateAnonymousChildStyle(std::ptr::null(), &mut builder);
        let object = MakeGarbageCollected(Self::new(std::ptr::null_mut()));
        unsafe { &mut *object }.SetInputOwnerForAnonymous(parent);
        unsafe { &mut *object }.SetStyle(builder.TakeStyle());
        object
    }
    // cpp: layoutng_table/layout_table_section.cc:37-55
    pub fn IsEmpty(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.FirstChild().is_null()
    }
    pub fn FirstRow(&self) -> *mut LayoutTableRow {
        self.CheckIsNotDestroyed();
        To::<LayoutTableRow>(self.FirstChild())
    }
    pub fn LastRow(&self) -> *mut LayoutTableRow {
        self.CheckIsNotDestroyed();
        To::<LayoutTableRow>(self.LastChild())
    }
    pub fn Table(&self) -> *mut LayoutTable {
        self.CheckIsNotDestroyed();
        To::<LayoutTable>(self.Parent())
    }
    // cpp: layoutng_table/layout_table_section.cc:57-111
    pub fn AddChild(&mut self, child: *mut LayoutObject, before: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        if let Some(table) = unsafe { self.Table().as_mut() } {
            table.TableGridStructureChanged();
        }
        let this = (self as *mut Self).cast::<LayoutObject>();
        if !before.is_null() && unsafe { &*before }.Parent() != this {
            if unsafe { &*child }.IsTableRow() {
                let direct = self.SplitAnonymousBoxesAroundChild(before);
                debug_assert_eq!(unsafe { &*direct }.Parent(), this);
                self.AddChild(child, direct);
            } else {
                let mut container = unsafe { &*before }.Parent();
                while unsafe { &*container }.Parent() != this {
                    container = unsafe { &*container }.Parent();
                }
                assert!(
                    unsafe { &*container }.IsAnonymous() && unsafe { &*container }.IsTableRow()
                );
                unsafe { &mut *container }.AddChild(child, before);
            }
            return;
        }
        if !unsafe { &*child }.IsTableRow() {
            let after = if before.is_null() {
                self.LastChild()
            } else {
                unsafe { &*before }.PreviousSibling()
            };
            if !after.is_null() && unsafe { &*after }.IsAnonymous() {
                unsafe { &mut *after }.AddChild(child, std::ptr::null_mut());
                return;
            }
            let row = LayoutTableRow::CreateAnonymousWithParent(unsafe { &*this });
            unsafe { &mut *this }.AddChildBase(row.cast(), before);
            unsafe { &mut *row }.AddChild(child, std::ptr::null_mut());
            return;
        }
        unsafe { &mut *this }.AddChildBase(child, before);
    }
    // cpp: layoutng_table/layout_table_section.cc:113-143
    pub fn RemoveChild(&mut self, child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        if let Some(table) = unsafe { self.Table().as_mut() } {
            table.TableGridStructureChanged();
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
    // cpp: layoutng_table/layout_table_section.cc:145-162
    pub fn CreateAnonymousBoxWithSameTypeAs(&self, parent: *const LayoutObject) -> *mut LayoutBox {
        self.CheckIsNotDestroyed();
        Self::CreateAnonymousWithParent(unsafe { &*parent }).cast()
    }
    pub fn NumRows(&self) -> u32 {
        self.CheckIsNotDestroyed();
        let mut count = 0;
        let mut row = self.FirstChild();
        while !row.is_null() {
            count += 1;
            row = unsafe { &*row }.NextSibling();
        }
        count
    }
}

impl Deref for LayoutTableSection {
    type Target = LayoutBlock;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for LayoutTableSection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutTableSection, base) == 0);
impl foundation::DowncastFrom<LayoutObject> for LayoutTableSection {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsTableSection()
    }
}
impl foundation::DowncastFrom<LayoutBox> for LayoutTableSection {
    fn AllowFrom(object: &LayoutBox) -> bool {
        object.IsTableSection()
    }
}
impl Traceable for LayoutTableSection {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.base.Trace(visitor);
    }
}
