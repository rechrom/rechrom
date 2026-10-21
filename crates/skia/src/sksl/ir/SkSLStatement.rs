use crate::src::sksl::{
    ir::SkSLIRNode::{IRNode, IRNodeData, StatementKind},
    SkSLPosition::Position,
};
// Source: src/sksl/ir/SkSLStatement.h:19-19
// Adapted actual inheritance base; no extra fields or concrete default statement.
#[repr(C)]
pub struct StatementData {
    pub(crate) base: IRNodeData,
}
// Source: src/sksl/ir/SkSLStatement.h:21-21
pub type Kind = StatementKind;
impl StatementData {
    // Source: src/sksl/ir/SkSLStatement.h:23-26
    pub fn new(pos: Position, kind: Kind) -> Self {
        let value = Self {
            base: IRNodeData::new(pos, kind as i32),
        };
        debug_assert!(kind as i32 >= Kind::kFirst as i32 && kind as i32 <= Kind::kLast as i32);
        value
    }
}
// Source: src/sksl/ir/SkSLStatement.h:28-30
/// Safety: original native base fKind must stay in StatementKind's valid range.
/// The actual concrete object supplies IRNode description and destructor.
pub unsafe trait Statement: IRNode {
    fn kind(&self) -> Kind {
        unsafe { core::mem::transmute(self.node().fKind) }
    }
    // Source: src/sksl/ir/SkSLStatement.h:32-34
    fn isEmpty(&self) -> bool {
        false
    }
}
