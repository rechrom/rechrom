//! Lifetime-bound queries over the resident native table owners.
use super::*;
use foundation::table_focusgroup::TableFocusgroupFacts;

#[derive(Clone, Copy)]
pub struct TableFocusgroupCursor<'a> {
    output: LayoutOutput<'a>,
    object: &'a LayoutObject,
}
impl PartialEq for TableFocusgroupCursor<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.object, other.object)
    }
}
impl Eq for TableFocusgroupCursor<'_> {}
impl<'a> TableFocusgroupCursor<'a> {
    pub fn NodeId(self) -> Option<u64> {
        unsafe { self.object.GetNode().as_ref() }.map(|node| node.InputId())
    }
    pub fn Facts(self) -> Option<TableFocusgroupFacts<Self>> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let virtuals = unsafe { self.object.FactorySet().as_ref() }?.table_virtuals?;
        (virtuals.focusgroup_facts)(self.object).map(|facts| {
            facts.Map(|pointer| Self {
                output: self.output,
                // The table-package vtable returns only owners in this borrowed
                // resident tree. No mutable lifecycle can run during the borrow.
                object: unsafe { &*pointer },
            })
        })
    }
}
impl<'a> LayoutNode<'a> {
    pub fn TableFocusgroupCursor(&self) -> Option<TableFocusgroupCursor<'a>> {
        (self.object.IsTable()
            || self.object.IsTableSection()
            || self.object.IsTableRow()
            || self.object.IsTableCell())
        .then_some(TableFocusgroupCursor {
            output: self.output,
            object: self.object,
        })
    }
}
