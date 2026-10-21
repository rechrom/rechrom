use crate::src::sksl::{
    ir::SkSLIRNode::{IRNode, IRNodeData, ProgramElementKind},
    SkSLPosition::Position,
};
// Source: src/sksl/ir/SkSLProgramElement.h:20-20
// Adapted inheritance: one nested repr(C) base preserves actual address/layout;
// abstract virtual description remains mandatory in IRNode, no concrete element.
#[repr(C)]
pub struct ProgramElementData {
    pub(crate) base: IRNodeData,
}
// Source: src/sksl/ir/SkSLProgramElement.h:22-22
pub type Kind = ProgramElementKind;
impl ProgramElementData {
    // Source: src/sksl/ir/SkSLProgramElement.h:24-27
    pub fn new(pos: Position, kind: Kind) -> Self {
        let value = Self {
            base: IRNodeData::new(pos, kind as i32),
        };
        debug_assert!(kind as i32 >= Kind::kFirst as i32 && kind as i32 <= Kind::kLast as i32);
        value
    }
}
// Source: src/sksl/ir/SkSLProgramElement.h:29-31
/// Safety: native ProgramElement invariants require its base fKind to be a
/// valid ProgramElementKind. Concrete implementations must preserve this range.
pub unsafe trait ProgramElement: IRNode {
    fn kind(&self) -> Kind {
        unsafe { core::mem::transmute(self.node().fKind) }
    }
}
