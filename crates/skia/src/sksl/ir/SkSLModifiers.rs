use super::{SkSLLayout::Layout, SkSLModifierFlags::ModifierFlags};
use crate::src::sksl::SkSLPosition::Position;
// Source: src/sksl/ir/SkSLModifiers.h:17-21
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Modifiers {
    pub fPosition: Position,
    pub fLayout: Layout,
    pub fFlags: ModifierFlags,
}
impl Modifiers {
    pub fn new(position: Position, layout: Layout) -> Self {
        Self {
            fPosition: position,
            fLayout: layout,
            fFlags: ModifierFlags::default(),
        }
    }
}

// Native aggregate default initialization uses the actual three member defaults.
impl Default for Modifiers {
    fn default() -> Self {
        Self::new(Position::default(), Layout::default())
    }
}
