// Source: src/sksl/SkSLDefines.h:21-22
// Native forward declarations project the existing abstract interfaces.
pub use crate::src::sksl::ir::{SkSLExpression::Expression, SkSLStatement::Statement};
// Source: src/sksl/SkSLDefines.h:16-17
pub type SKSL_INT = i64;
pub type SKSL_FLOAT = f32;
// Source: src/sksl/SkSLDefines.h:24-26
// Actual inherited inline2 unique-expression storage; no separate list/counter.
// Rust pinning prevents native inline pointers from dangling after a plain move.
#[repr(transparent)]
pub struct ExpressionArray {
    base: crate::include::private::SkTArray::STArray<
        Option<
            crate::src::sksl::ir::SkSLIRNode::IRNodeBox<
                dyn crate::src::sksl::ir::SkSLExpression::Expression,
            >,
        >,
        2,
        true,
    >,
}
impl ExpressionArray {
    // Selected Rust owner/constructor/inheritance projection, no new node state.
    pub fn new() -> core::pin::Pin<Box<Self>> {
        Box::pin(Self {
            base: crate::include::private::SkTArray::STArray::new(),
        })
    }
    pub fn as_base(
        &self,
    ) -> &crate::include::private::SkTArray::STArray<
        Option<
            crate::src::sksl::ir::SkSLIRNode::IRNodeBox<
                dyn crate::src::sksl::ir::SkSLExpression::Expression,
            >,
        >,
        2,
        true,
    > {
        &self.base
    }
    pub fn as_base_mut(
        self: core::pin::Pin<&mut Self>,
    ) -> core::pin::Pin<
        &mut crate::include::private::SkTArray::STArray<
            Option<
                crate::src::sksl::ir::SkSLIRNode::IRNodeBox<
                    dyn crate::src::sksl::ir::SkSLExpression::Expression,
                >,
            >,
            2,
            true,
        >,
    > {
        unsafe { self.map_unchecked_mut(|v| &mut v.base) }
    }
}
// Source: src/sksl/SkSLDefines.h:32-32
// Adapted actual unique_ptr<Statement> storage is the same canonical pool owner;
// actual native inline2 STArray owns/moves/drops those exact objects.
// MEM_MOVE=true matches pinned SkTypeTraits unique_ptr trivial-relocation specialization.
pub type StatementArray = crate::include::private::SkTArray::STArray<
    Option<
        crate::src::sksl::ir::SkSLIRNode::IRNodeBox<
            dyn crate::src::sksl::ir::SkSLStatement::Statement,
        >,
    >,
    2,
    true,
>;
// Source: src/sksl/SkSLDefines.h:38-38
pub const kDefaultInlineThreshold: i32 = 50;
// Source: src/sksl/SkSLDefines.h:43-43
pub const kVariableSlotLimit: i32 = 100000;
