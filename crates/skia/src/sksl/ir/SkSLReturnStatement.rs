// UNVERIFIED: user explicitly prohibited all validation.
use super::{
    SkSLExpression::{Expression, ExpressionExt},
    SkSLIRNode::*,
    SkSLStatement::{Kind, Statement, StatementData},
    SkSLSymbolTable::SymbolTable,
    SkSLType::Type,
};
use crate::src::sksl::{
    SkSLAnalysis,
    SkSLContext::Context,
    SkSLDefines::{ExpressionArray, StatementArray, SKSL_INT},
    SkSLErrorReporter::ErrorReporterExt,
    SkSLPosition::{ForLoopPositions, Position},
    SkSLProgramSettings::ProgramConfig,
};
use core::{mem::ManuallyDrop, pin::Pin, ptr::NonNull};
type ExprSlot = Option<IRNodeBox<dyn Expression>>;
type StmtSlot = Option<IRNodeBox<dyn Statement>>;
// Source: src/sksl/ir/SkSLReturnStatement.h:19-19
// Source: src/sksl/ir/SkSLReturnStatement.h:32-32
#[repr(C)]
pub struct ReturnStatement {
    base: StatementData,
    fExpression: ManuallyDrop<ExprSlot>,
}
// Source: src/sksl/ir/SkSLReturnStatement.h:21-21
impl IRNodeType for ReturnStatement {
    const kIRNodeKind: i32 = Kind::kReturn as i32;
}
impl ReturnStatement {
    // Source: src/sksl/ir/SkSLReturnStatement.h:23-25
    pub unsafe fn new(pos: Position, fExpression: ExprSlot) -> Self {
        Self {
            base: StatementData::new(pos, Kind::kReturn),
            fExpression: ManuallyDrop::new(fExpression),
        }
    }
    // Source: src/sksl/ir/SkSLReturnStatement.h:27-29
    pub unsafe fn Make(pos: Position, expression: ExprSlot) -> IRNodeBox<dyn Statement> {
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, expression)))
            .cast(|p| p as *mut dyn Statement)
    }
    // Source: src/sksl/ir/SkSLReturnStatement.h:31-33
    pub unsafe fn expression_mut(&mut self) -> &mut ExprSlot {
        &mut self.fExpression
    }
    // Source: src/sksl/ir/SkSLReturnStatement.h:35-37
    pub fn expression(&self) -> &ExprSlot {
        &self.fExpression
    }
    // Source: src/sksl/ir/SkSLReturnStatement.h:39-41
    pub unsafe fn setExpression(&mut self, expr: ExprSlot) {
        *self.fExpression = expr;
    }
}
// Native reverse-member destruction, including statements before their symbol owner.
impl Drop for ReturnStatement {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fExpression);
        }
    }
}
unsafe impl Statement for ReturnStatement {}
impl IRNode for ReturnStatement {
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base.base) }
    }
    fn description(&self) -> Vec<u8> {
        description(self)
    }
}
// Source: src/sksl/ir/SkSLReturnStatement.h:43-49
fn description(this: &ReturnStatement) -> Vec<u8> {
    let mut out = b"return".to_vec();
    if let Some(expression) = this.expression() {
        out.push(b' ');
        out.extend_from_slice(&expression.description());
    }
    out.push(b';');
    out
}
