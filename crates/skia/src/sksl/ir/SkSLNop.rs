use crate::src::sksl::{
    ir::{
        SkSLIRNode::{IRNode, IRNodeBox, IRNodeData, IRNodeType},
        SkSLStatement::{Kind, Statement, StatementData},
    },
    SkSLPosition::Position,
};
use core::pin::Pin;
// Source: src/sksl/ir/SkSLNop.h:19-19
// Adapted actual inheritance storage; nested repr(C) base stays at offset0.
#[repr(C)]
pub struct Nop {
    base: StatementData,
}
// Source: src/sksl/ir/SkSLNop.h:21-21
impl IRNodeType for Nop {
    const kIRNodeKind: i32 = Kind::kNop as i32;
}
impl Nop {
    // Source: src/sksl/ir/SkSLNop.h:23-24
    pub fn new() -> Self {
        Self {
            base: StatementData::new(Position::new(), Kind::kNop),
        }
    }
    // Source: src/sksl/ir/SkSLNop.h:26-28
    /// Safety: native current-pool allocation/drop protocol must hold until the returned
    /// concrete destructor and inherited delete finish. No callbacks may unwind.
    pub unsafe fn Make() -> IRNodeBox<dyn Statement> {
        let value: IRNodeBox<Self> = IRNodeBox::new_with(|p: *mut Self| p.write(Self::new()));
        value.cast(|p| p as *mut dyn Statement)
    }
}
unsafe impl Statement for Nop {
    // Source: src/sksl/ir/SkSLNop.h:30-32
    fn isEmpty(&self) -> bool {
        true
    }
}
impl IRNode for Nop {
    // Source: src/sksl/ir/SkSLNop.h:34-36
    fn description(&self) -> Vec<u8> {
        b";".to_vec()
    }
    // Uncounted Rust projections of original inherited base, no copied state.
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|n| &mut n.base.base) }
    }
}
