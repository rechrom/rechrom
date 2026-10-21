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
// Source: src/sksl/ir/SkSLExpressionStatement.h:26-26
// Source: src/sksl/ir/SkSLExpressionStatement.h:43-43
#[repr(C)]
pub struct ExpressionStatement {
    base: StatementData,
    fExpression: ManuallyDrop<ExprSlot>,
}
// Source: src/sksl/ir/SkSLExpressionStatement.h:28-28
impl IRNodeType for ExpressionStatement {
    const kIRNodeKind: i32 = Kind::kExpression as i32;
}
impl ExpressionStatement {
    // Source: src/sksl/ir/SkSLExpressionStatement.h:30-32
    pub unsafe fn new(fExpression: ExprSlot) -> Self {
        Self {
            base: StatementData::new(
                fExpression.as_ref().unwrap_unchecked().node().fPosition,
                Kind::kExpression,
            ),
            fExpression: ManuallyDrop::new(fExpression),
        }
    }
    // Source: src/sksl/ir/SkSLExpressionStatement.h:42-44
    pub fn expression(&self) -> &ExprSlot {
        &self.fExpression
    }
    // Source: src/sksl/ir/SkSLExpressionStatement.h:46-48
    pub unsafe fn expression_mut(&mut self) -> &mut ExprSlot {
        &mut self.fExpression
    }
}
// Native reverse-member destruction, including statements before their symbol owner.
impl Drop for ExpressionStatement {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fExpression);
        }
    }
}
unsafe impl Statement for ExpressionStatement {}
impl IRNode for ExpressionStatement {
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
use super::{
    SkSLBinaryExpression::BinaryExpression, SkSLNop::Nop, SkSLVariableReference::VariableRefKind,
};
impl ExpressionStatement {
    // Source: src/sksl/ir/SkSLExpressionStatement.cpp:21-29
    pub unsafe fn Convert(context: &Context, expr: IRNodeBox<dyn Expression>) -> StmtSlot {
        if expr.isIncomplete(context) {
            return None;
        }
        Some(Self::Make(context, expr))
    }
    // Source: src/sksl/ir/SkSLExpressionStatement.cpp:31-56
    pub unsafe fn Make(
        context: &Context,
        expr: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Statement> {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLExpressionStatement.cpp\0".as_ptr(),33; @condition b"!expr->isIncomplete(context)\0".as_ptr(); !expr.isIncomplete(context));
        if context
            .fConfig
            .unwrap_unchecked()
            .as_ref()
            .fSettings
            .fOptimize
        {
            if !SkSLAnalysis::HasSideEffects(&*expr) {
                return Nop::Make();
            }
            if expr.node().is::<BinaryExpression>() {
                let binary = &mut *(expr.as_ptr() as *mut () as *mut BinaryExpression);
                if let Some(mut assignedVar) = binary.isAssignmentIntoVariable() {
                    if assignedVar.as_ref().refKind() == VariableRefKind::kReadWrite {
                        assignedVar.as_mut().setRefKind(VariableRefKind::kWrite);
                    }
                }
            }
        }
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(Some(expr))))
            .cast(|p| p as *mut dyn Statement)
    }
}
// Source: src/sksl/ir/SkSLExpressionStatement.cpp:58-60
fn description(this: &ExpressionStatement) -> Vec<u8> {
    unsafe {
        let mut out = this
            .expression()
            .as_ref()
            .unwrap_unchecked()
            .description_with_precedence(
                crate::src::sksl::SkSLOperator::OperatorPrecedence::kStatement,
            );
        out.push(b';');
        out
    }
}
