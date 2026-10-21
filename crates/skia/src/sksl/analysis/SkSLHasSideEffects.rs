// UNVERIFIED: user prohibited all validation.
// Actual FunctionCall and inherited expression dispatcher are now present.
use crate::src::sksl::{
    ir::{
        SkSLBinaryExpression::BinaryExpression,
        SkSLExpression::{Expression, ExpressionExt, Kind},
        SkSLFunctionCall::FunctionCall,
        SkSLPrefixExpression::PrefixExpression,
    },
    SkSLOperator::OperatorKind,
};
// Source: src/sksl/analysis/SkSLHasSideEffects.cpp:23-62
pub unsafe fn HasSideEffects(expr: &dyn Expression) -> bool {
    struct HasSideEffectsVisitor;
    impl HasSideEffectsVisitor {
        unsafe fn visitExpression(&mut self, expr: &dyn Expression) -> bool {
            match expr.kind() {
                Kind::kFunctionCall => {
                    let call = expr.node().as_node::<FunctionCall>();
                    if !call.function().modifierFlags().isPure() {
                        return true;
                    }
                }
                Kind::kPrefix => {
                    let prefix = expr.node().as_node::<PrefixExpression>();
                    if prefix.getOperator().kind() == OperatorKind::PLUSPLUS
                        || prefix.getOperator().kind() == OperatorKind::MINUSMINUS
                    {
                        return true;
                    }
                }
                Kind::kBinary => {
                    let binary = expr.node().as_node::<BinaryExpression>();
                    if binary.getOperator().isAssignment() {
                        return true;
                    }
                }
                Kind::kPostfix => return true,
                _ => {}
            }
            // Native inherited recursion must call the genuine ProgramVisitor dispatcher.
            // Actual callback dispatch preserves the derived visitor override.
            super::SkSLProgramVisitor::visitExpression(
                &mut |child| self.visitExpression(child),
                expr,
            )
        }
    }
    let mut visitor = HasSideEffectsVisitor;
    visitor.visitExpression(expr)
}
