#![allow(non_snake_case)]
use crate::layout_table::LayoutTable;
use foundation::{To, Traceable, Visitor};
use layoutng_assembly::internal::{
    layout_block_flow::LayoutBlockFlow,
    layout_box::LayoutBox,
    layout_node_metadata::Element,
    layout_object::{LayoutObject, LayoutObjectClass, StyleChangeContext},
};
use layoutng_style::style::{computed_style::ComputedStyle, style_difference::StyleDifference};
use std::ops::{Deref, DerefMut};
// cpp: layoutng_table/layout_table_caption.h:15-41
#[repr(C)]
pub struct LayoutTableCaption {
    base: LayoutBlockFlow,
}
impl LayoutTableCaption {
    // cpp: layoutng_table/layout_table_caption.cc:12-39
    pub fn new(element: *mut Element) -> Self {
        let base = LayoutBlockFlow::new(element.cast());
        base.SetRuntimeClass(LayoutObjectClass::TableCaption);
        Self { base }
    }
    fn Table(&self) -> *mut LayoutTable {
        self.CheckIsNotDestroyed();
        To::<LayoutTable>(self.Parent())
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
                if old.CaptionSide() != new.CaptionSide() {
                    table.SetShouldDoFullPaintInvalidation();
                }
            }
        }
        self.base.StyleDidChange(diff, old, new, context);
    }
}

impl Deref for LayoutTableCaption {
    type Target = LayoutBlockFlow;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for LayoutTableCaption {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutTableCaption, base) == 0);
impl foundation::DowncastFrom<LayoutObject> for LayoutTableCaption {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsTableCaption()
    }
}
impl foundation::DowncastFrom<LayoutBox> for LayoutTableCaption {
    fn AllowFrom(object: &LayoutBox) -> bool {
        object.IsTableCaption()
    }
}
impl Traceable for LayoutTableCaption {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.base.Trace(visitor);
    }
}
