use crate::src::sksl::{
    ir::{
        SkSLIRNode::{IRNode, IRNodeBox, IRNodeData, IRNodeType},
        SkSLStatement::{Kind, Statement, StatementData},
    },
    SkSLPosition::Position,
};
use core::pin::Pin;
// Source: src/sksl/ir/SkSLBreakStatement.h:19-19
// Adapted actual inheritance storage; nested repr(C) base stays at offset0.
#[repr(C)]
pub struct BreakStatement {
    base: StatementData,
}
// Source: src/sksl/ir/SkSLBreakStatement.h:21-21
impl IRNodeType for BreakStatement {
    const kIRNodeKind: i32 = Kind::kBreak as i32;
}
impl BreakStatement {
    // Source: src/sksl/ir/SkSLBreakStatement.h:23-23
    pub fn new(pos: Position) -> Self {
        Self {
            base: StatementData::new(pos, Kind::kBreak),
        }
    }
    // Source: src/sksl/ir/SkSLBreakStatement.h:25-27
    /// Safety: native current-pool allocation/drop protocol must hold until the returned
    /// concrete destructor and inherited delete finish. No callbacks may unwind.
    pub unsafe fn Make(pos: Position) -> IRNodeBox<dyn Statement> {
        let value: IRNodeBox<Self> = IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos)));
        value.cast(|p| p as *mut dyn Statement)
    }
}
unsafe impl Statement for BreakStatement {}
impl IRNode for BreakStatement {
    // Source: src/sksl/ir/SkSLBreakStatement.h:29-31
    fn description(&self) -> Vec<u8> {
        b"break;".to_vec()
    }
    // Uncounted Rust projections of original inherited base, no copied state.
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|n| &mut n.base.base) }
    }
}
