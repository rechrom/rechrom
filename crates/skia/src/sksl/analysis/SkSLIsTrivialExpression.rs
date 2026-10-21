// UNVERIFIED: user prohibited all validation.
use crate::src::sksl::{
    ir::{
        SkSLExpression::{Expression, ExpressionExt, Kind},
        SkSLFieldAccess::FieldAccess,
        SkSLIndexExpression::IndexExpression,
        SkSLPrefixExpression::PrefixExpression,
        SkSLSwizzle::Swizzle,
    },
    SkSLAnalysis::IsCompileTimeConstant,
    SkSLOperator::OperatorKind,
};
// Source: src/sksl/analysis/SkSLIsTrivialExpression.cpp:26-83
pub unsafe fn IsTrivialExpression(expr: &dyn Expression) -> bool {
    match expr.kind() {
        Kind::kLiteral | Kind::kVariableReference => true,
        Kind::kSwizzle => IsTrivialExpression(
            &**expr
                .node()
                .as_node::<Swizzle>()
                .base()
                .as_ref()
                .unwrap_unchecked(),
        ),
        Kind::kPrefix => {
            let prefix = expr.node().as_node::<PrefixExpression>();
            match prefix.getOperator().kind() {
                OperatorKind::PLUS
                | OperatorKind::MINUS
                | OperatorKind::LOGICALNOT
                | OperatorKind::BITWISENOT => {
                    IsTrivialExpression(&**prefix.operand().as_ref().unwrap_unchecked())
                }
                _ => false,
            }
        }
        Kind::kFieldAccess => IsTrivialExpression(
            &**expr
                .node()
                .as_node::<FieldAccess>()
                .base()
                .as_ref()
                .unwrap_unchecked(),
        ),
        Kind::kIndex => {
            let inner = expr.node().as_node::<IndexExpression>();
            inner.index().as_ref().unwrap_unchecked().isIntLiteral()
                && IsTrivialExpression(&**inner.base().as_ref().unwrap_unchecked())
        }
        Kind::kConstructorArray | Kind::kConstructorStruct => {
            expr.expression_type().slotCount() <= 4 && IsCompileTimeConstant(expr)
        }
        Kind::kConstructorArrayCast | Kind::kConstructorMatrixResize => false,
        Kind::kConstructorCompound => IsCompileTimeConstant(expr),
        Kind::kConstructorCompoundCast
        | Kind::kConstructorScalarCast
        | Kind::kConstructorSplat
        | Kind::kConstructorDiagonalMatrix => {
            let args = expr.constructor_view().argumentSpan();
            debug_assert!(args.len() == 1);
            IsTrivialExpression(&**args[0].as_ref().unwrap_unchecked())
        }
        _ => false,
    }
}
