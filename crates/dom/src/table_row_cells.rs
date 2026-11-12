//! HTMLTableRowElement's retained cells collection.
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: html/html_table_row_element.cc:135-137;
//! html/html_collection.cc:75-80,226-228,295-320.
#![allow(non_snake_case)]
use super::{DOMNodeClass, PersistentDocument};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Debug, Default)]
pub(super) struct HTMLTableRowElement {
    cells: RefCell<Option<Rc<Vec<usize>>>>,
}
impl PersistentDocument {
    // kTRCells has no attribute invalidation. Structural mutations invalidate
    // only this row's canonical collection, never an unrelated table/row.
    pub(super) fn InvalidateTableRowCells(&self, row: usize) {
        if let DOMNodeClass::HTMLTableRowElement(owner) = &self.arena[row].class {
            *owner.cells.borrow_mut() = None;
        }
    }
    pub fn IsHTMLTableRowElementForFocusgroup(&self, node: usize) -> bool {
        matches!(self.arena[node].class, DOMNodeClass::HTMLTableRowElement(_))
    }
    pub fn TableRowCellAtIndex(&self, row: usize, index: u32) -> Option<usize> {
        let DOMNodeClass::HTMLTableRowElement(owner) = &self.arena[row].class else {
            return None;
        };
        if let Some(cells) = owner.cells.borrow().as_ref() {
            return cells.get(index as usize).copied();
        }
        let cells: Rc<Vec<_>> = Rc::new(
            self.arena[row]
                .children
                .iter()
                .copied()
                .filter(|&child| {
                    matches!(self.arena[child].class, DOMNodeClass::HTMLTableCellElement)
                })
                .collect(),
        );
        let result = cells.get(index as usize).copied();
        *owner.cells.borrow_mut() = Some(cells);
        result
    }
}
