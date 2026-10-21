// New translation is UNVERIFIED by explicit user instruction.
// The native FieldAccess/IndexExpression/PrefixExpression/Swizzle dependencies
// below are real unresolved modules; no literal-only or unknown-Kind fallback.
use crate::src::sksl::ir::{
    SkSLConstructor::asAnyConstructor,
    SkSLExpression::{Expression, ExpressionExt, Kind},
    SkSLFieldAccess::FieldAccess,
    SkSLIndexExpression::IndexExpression,
    SkSLLiteral::Literal,
    SkSLPrefixExpression::PrefixExpression,
    SkSLSwizzle::Swizzle,
    SkSLVariableReference::VariableReference,
};
// Source: src/sksl/analysis/SkSLIsSameExpressionTree.cpp:28-95
pub unsafe fn IsSameExpressionTree(left: &dyn Expression, right: &dyn Expression) -> bool {
    if left.kind() != right.kind() || !left.expression_type().matches(right.expression_type()) {
        return false;
    }
    match left.kind() {
        Kind::kLiteral => {
            left.node().as_node::<Literal>().value() == right.node().as_node::<Literal>().value()
        }
        Kind::kConstructorArray
        | Kind::kConstructorArrayCast
        | Kind::kConstructorCompound
        | Kind::kConstructorCompoundCast
        | Kind::kConstructorDiagonalMatrix
        | Kind::kConstructorMatrixResize
        | Kind::kConstructorScalarCast
        | Kind::kConstructorStruct
        | Kind::kConstructorSplat => {
            if left.kind() != right.kind() {
                return false;
            }
            let leftCtor = asAnyConstructor(left);
            let rightCtor = asAnyConstructor(right);
            let leftSpan = leftCtor.argumentSpan();
            let rightSpan = rightCtor.argumentSpan();
            if leftSpan.len() != rightSpan.len() {
                return false;
            }
            for index in 0..leftSpan.len() {
                if !IsSameExpressionTree(
                    &**leftSpan[index].as_ref().unwrap_unchecked(),
                    &**rightSpan[index].as_ref().unwrap_unchecked(),
                ) {
                    return false;
                }
            }
            true
        }
        Kind::kFieldAccess => {
            let l = left.node().as_node::<FieldAccess>();
            let r = right.node().as_node::<FieldAccess>();
            l.fieldIndex() == r.fieldIndex()
                && IsSameExpressionTree(
                    &**l.base().as_ref().unwrap_unchecked(),
                    &**r.base().as_ref().unwrap_unchecked(),
                )
        }
        Kind::kIndex => {
            let l = left.node().as_node::<IndexExpression>();
            let r = right.node().as_node::<IndexExpression>();
            IsSameExpressionTree(
                &**l.index().as_ref().unwrap_unchecked(),
                &**r.index().as_ref().unwrap_unchecked(),
            ) && IsSameExpressionTree(
                &**l.base().as_ref().unwrap_unchecked(),
                &**r.base().as_ref().unwrap_unchecked(),
            )
        }
        Kind::kPrefix => {
            let l = left.node().as_node::<PrefixExpression>();
            let r = right.node().as_node::<PrefixExpression>();
            l.getOperator().kind() == r.getOperator().kind()
                && IsSameExpressionTree(
                    &**l.operand().as_ref().unwrap_unchecked(),
                    &**r.operand().as_ref().unwrap_unchecked(),
                )
        }
        Kind::kSwizzle => {
            let l = left.node().as_node::<Swizzle>();
            let r = right.node().as_node::<Swizzle>();
            l.components() == r.components()
                && IsSameExpressionTree(
                    &**l.base().as_ref().unwrap_unchecked(),
                    &**r.base().as_ref().unwrap_unchecked(),
                )
        }
        Kind::kVariableReference => {
            left.node()
                .as_node::<VariableReference>()
                .variable()
                .map(|p| p.as_ptr() as *const ())
                == right
                    .node()
                    .as_node::<VariableReference>()
                    .variable()
                    .map(|p| p.as_ptr() as *const ())
        }
        _ => false,
    }
}
