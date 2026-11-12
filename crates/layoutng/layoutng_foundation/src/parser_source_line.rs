//! Read-only identity for the parser input's current source line. The input
//! keeps its own scalar; consumers cannot mutate it or borrow the parser tree.
use std::{cell::Cell, rc::Rc};
#[derive(Clone, Debug)]
pub struct ParserSourceLine(Rc<Cell<i32>>);
impl ParserSourceLine {
    pub fn Borrow(line: &Rc<Cell<i32>>) -> Self {
        Self(line.clone())
    }
    pub fn ZeroBasedLine(&self) -> i32 {
        self.0.get()
    }
    pub fn SameOwner(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
