// UNVERIFIED: user prohibited all validation.
use super::{
    SkSLExpression::{Expression, ExpressionData, ExpressionExt},
    SkSLIRNode::*,
    SkSLType::Type,
};
use crate::src::sksl::{
    SkSLContext::Context, SkSLOperator::OperatorPrecedence, SkSLPosition::Position,
};
use core::{pin::Pin, ptr::NonNull};
unsafe fn borrow_type(t: &dyn Type) -> NonNull<dyn Type> {
    core::mem::transmute::<NonNull<dyn Type + '_>, NonNull<dyn Type>>(NonNull::from(t))
}
use super::{
    SkSLFieldAccess::FieldAccess,
    SkSLIndexExpression::IndexExpression,
    SkSLSwizzle::Swizzle,
    SkSLTernaryExpression::TernaryExpression,
    SkSLVariableReference::{VariableRefKind, VariableReference},
};
use crate::src::sksl::{
    SkSLAnalysis,
    SkSLConstantFolder::ConstantFolder,
    SkSLErrorReporter::ErrorReporterExt,
    SkSLOperator::{Operator, OperatorKind},
};
use core::mem::ManuallyDrop;
// Source: src/sksl/ir/SkSLBinaryExpression.h:31-31
// Source: src/sksl/ir/SkSLBinaryExpression.h:103-105
#[repr(C)]
pub struct BinaryExpression {
    base: ExpressionData,
    fLeft: ManuallyDrop<Option<IRNodeBox<dyn Expression>>>,
    fOperator: Operator,
    fRight: ManuallyDrop<Option<IRNodeBox<dyn Expression>>>,
}
// Source: src/sksl/ir/SkSLBinaryExpression.h:33-33
impl IRNodeType for BinaryExpression {
    const kIRNodeKind: i32 = ExpressionKind::kBinary as i32;
}
impl BinaryExpression {
    // Source: src/sksl/ir/SkSLBinaryExpression.h:35-43
    pub unsafe fn new(
        pos: Position,
        left: Option<IRNodeBox<dyn Expression>>,
        op: Operator,
        right: Option<IRNodeBox<dyn Expression>>,
        ty: NonNull<dyn Type>,
    ) -> Self {
        let value = Self {
            base: ExpressionData::new(pos, ExpressionKind::kBinary, Some(ty)),
            fLeft: ManuallyDrop::new(left),
            fOperator: op,
            fRight: ManuallyDrop::new(right),
        };
        debug_assert!(
            !op.isAssignment() || Self::CheckRef(&**value.left().as_ref().unwrap_unchecked())
        );
        value
    }
    // Source: src/sksl/ir/SkSLBinaryExpression.h:69-75
    pub unsafe fn left_mut(&mut self) -> &mut Option<IRNodeBox<dyn Expression>> {
        &mut self.fLeft
    }
    pub fn left(&self) -> &Option<IRNodeBox<dyn Expression>> {
        &self.fLeft
    }
    // Source: src/sksl/ir/SkSLBinaryExpression.h:77-83
    pub unsafe fn right_mut(&mut self) -> &mut Option<IRNodeBox<dyn Expression>> {
        &mut self.fRight
    }
    pub fn right(&self) -> &Option<IRNodeBox<dyn Expression>> {
        &self.fRight
    }
    // Source: src/sksl/ir/SkSLBinaryExpression.h:85-87
    pub fn getOperator(&self) -> Operator {
        self.fOperator
    }
}
// Adapted native reverse-member destruction: right child before left child.
impl Drop for BinaryExpression {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fRight);
            ManuallyDrop::drop(&mut self.fLeft);
        }
    }
}
impl IRNode for BinaryExpression {
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base.base) }
    }
    fn description(&self) -> Vec<u8> {
        self.expression_description()
    }
}
unsafe impl Expression for BinaryExpression {
    fn expression_data(&self) -> &ExpressionData {
        &self.base
    }
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base) }
    }
    fn as_expression(&self) -> &dyn Expression {
        self
    }
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        clone_at(self, pos)
    }
    fn description_with_precedence(&self, p: OperatorPrecedence) -> Vec<u8> {
        description(self, p)
    }
}
impl BinaryExpression {
    // Source: src/sksl/ir/SkSLBinaryExpression.cpp:24-95
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        left: Option<IRNodeBox<dyn Expression>>,
        op: Operator,
        right: Option<IRNodeBox<dyn Expression>>,
    ) -> Option<IRNodeBox<dyn Expression>> {
        let mut left = left?;
        let right = right?;
        let rawLeftType = borrow_type(
            if left.isIntLiteral() && right.expression_type().isInteger() {
                right.expression_type()
            } else {
                left.expression_type()
            },
        );
        let rawRightType = borrow_type(
            if right.isIntLiteral() && left.expression_type().isInteger() {
                left.expression_type()
            } else {
                right.expression_type()
            },
        );
        let isAssignment = op.isAssignment();
        if isAssignment
            && !SkSLAnalysis::UpdateVariableRefKind(
                left.as_ptr() as *mut dyn Expression,
                if op.kind() != OperatorKind::EQ {
                    VariableRefKind::kReadWrite
                } else {
                    VariableRefKind::kWrite
                },
                Some(context.fErrors),
            )
        {
            return None;
        }
        // Native out pointers are uninitialized; borrowed reference adapter seeds only output leases,
        // which are never consumed on failure, and records the original output write protocol.
        let mut leftType = rawLeftType.as_ref();
        let mut rightType = rawRightType.as_ref();
        let mut resultType = rawLeftType.as_ref();
        if !op.determineBinaryType(
            context,
            rawLeftType.as_ref(),
            rawRightType.as_ref(),
            &mut leftType,
            &mut rightType,
            &mut resultType,
        ) {
            let mut msg = b"type mismatch: '".to_vec();
            msg.extend_from_slice(op.tightOperatorName().as_bytes());
            msg.extend_from_slice(b"' cannot operate on '");
            msg.extend_from_slice(&left.expression_type().displayName());
            msg.extend_from_slice(b"', '");
            msg.extend_from_slice(&right.expression_type().displayName());
            msg.push(b'\'');
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &msg);
            return None;
        }
        if isAssignment && (leftType.componentType().isOpaque() || leftType.isOrContainsAtomic()) {
            let mut msg = b"assignments to opaque type '".to_vec();
            msg.extend_from_slice(&left.expression_type().displayName());
            msg.extend_from_slice(b"' are not permitted");
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &msg);
            return None;
        }
        let strict = context.fConfig.unwrap_unchecked().as_ref().strictES2Mode();
        if strict && !op.isAllowedInStrictES2Mode() {
            let mut msg = b"operator '".to_vec();
            msg.extend_from_slice(op.tightOperatorName().as_bytes());
            msg.extend_from_slice(b"' is not allowed");
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &msg);
            return None;
        }
        if strict || op.kind() == OperatorKind::COMMA {
            let arrayExpr = if leftType.isOrContainsArray() {
                Some(&*left)
            } else if rightType.isOrContainsArray() {
                Some(&*right)
            } else {
                None
            };
            if let Some(arrayExpr) = arrayExpr {
                let mut msg = b"operator '".to_vec();
                msg.extend_from_slice(op.tightOperatorName().as_bytes());
                msg.extend_from_slice(
                    b"' can not operate on arrays (or structs containing arrays)",
                );
                let mut errors = context.fErrors;
                errors.as_mut().error(arrayExpr.node().fPosition, &msg);
                return None;
            }
        }
        let l = borrow_type(leftType);
        let rr = borrow_type(rightType);
        let result = borrow_type(resultType);
        let left = super::SkSLType::coerceExpression(l.as_ref(), Some(left), context);
        let right = super::SkSLType::coerceExpression(rr.as_ref(), Some(right), context);
        let left = left?;
        let right = right?;
        Some(Self::MakeWithType(context, pos, left, op, right, result))
    }
    // Source: src/sksl/ir/SkSLBinaryExpression.cpp:97-110
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        left: IRNodeBox<dyn Expression>,
        op: Operator,
        right: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Expression> {
        let mut leftType = left.expression_type();
        let mut rightType = right.expression_type();
        let mut resultType = left.expression_type();
        let ok = op.determineBinaryType(
            context,
            left.expression_type(),
            right.expression_type(),
            &mut leftType,
            &mut rightType,
            &mut resultType,
        );
        debug_assert!(ok);
        let ty = borrow_type(resultType);
        Self::MakeWithType(context, pos, left, op, right, ty)
    }
    // Source: src/sksl/ir/SkSLBinaryExpression.cpp:112-139
    pub unsafe fn MakeWithType(
        context: &Context,
        pos: Position,
        left: IRNodeBox<dyn Expression>,
        op: Operator,
        right: IRNodeBox<dyn Expression>,
        resultType: NonNull<dyn Type>,
    ) -> IRNodeBox<dyn Expression> {
        debug_assert!(
            !context.fConfig.unwrap_unchecked().as_ref().strictES2Mode()
                || op.isAllowedInStrictES2Mode()
        );
        debug_assert!(
            !context.fConfig.unwrap_unchecked().as_ref().strictES2Mode()
                || !left.expression_type().isOrContainsArray()
        );
        debug_assert!(
            !op.isAssignment()
                || SkSLAnalysis::IsAssignable(left.as_ptr() as *mut dyn Expression, None, None)
        );
        debug_assert!(!op.isAssignment() || !left.expression_type().componentType().isOpaque());
        if op.kind() == OperatorKind::EQ {
            super::SkSLType::checkForOutOfRangeLiteralExpr(
                left.expression_type(),
                context,
                &*right,
            );
        }
        if let Some(result) =
            ConstantFolder::Simplify(context, pos, &*left, op, &*right, resultType.as_ref())
        {
            return result;
        }
        IRNodeBox::new_with(|p: *mut Self| {
            p.write(Self::new(pos, Some(left), op, Some(right), resultType))
        })
        .cast(|p| p as *mut dyn Expression)
    }
    // Source: src/sksl/ir/SkSLBinaryExpression.cpp:141-164
    unsafe fn CheckRef(expr: &dyn Expression) -> bool {
        match expr.kind() {
            super::SkSLExpression::Kind::kFieldAccess => Self::CheckRef(
                &**expr
                    .node()
                    .as_node::<FieldAccess>()
                    .base()
                    .as_ref()
                    .unwrap_unchecked(),
            ),
            super::SkSLExpression::Kind::kIndex => Self::CheckRef(
                &**expr
                    .node()
                    .as_node::<IndexExpression>()
                    .base()
                    .as_ref()
                    .unwrap_unchecked(),
            ),
            super::SkSLExpression::Kind::kSwizzle => Self::CheckRef(
                &**expr
                    .node()
                    .as_node::<Swizzle>()
                    .base()
                    .as_ref()
                    .unwrap_unchecked(),
            ),
            super::SkSLExpression::Kind::kTernary => {
                let t = expr.node().as_node::<TernaryExpression>();
                Self::CheckRef(&**t.ifTrue().as_ref().unwrap_unchecked())
                    && Self::CheckRef(&**t.ifFalse().as_ref().unwrap_unchecked())
            }
            super::SkSLExpression::Kind::kVariableReference => {
                let r = expr.node().as_node::<VariableReference>();
                r.refKind() == VariableRefKind::kWrite || r.refKind() == VariableRefKind::kReadWrite
            }
            _ => false,
        }
    }
}
// Source: src/sksl/ir/SkSLBinaryExpression.cpp:166-172
unsafe fn clone_at(this: &BinaryExpression, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
    Some(
        IRNodeBox::new_with(|p: *mut BinaryExpression| {
            p.write(BinaryExpression::new(
                pos,
                this.left().as_ref().unwrap_unchecked().clone_current(),
                this.getOperator(),
                this.right().as_ref().unwrap_unchecked().clone_current(),
                borrow_type(this.expression_type()),
            ))
        })
        .cast(|p| p as *mut dyn Expression),
    )
}
// Source: src/sksl/ir/SkSLBinaryExpression.cpp:174-182
fn description(this: &BinaryExpression, parent: OperatorPrecedence) -> Vec<u8> {
    unsafe {
        let prec = this.getOperator().getBinaryPrecedence();
        let parens = prec.0 >= parent.0;
        let mut out = Vec::new();
        if parens {
            out.push(b'(');
        }
        out.extend_from_slice(
            &this
                .left()
                .as_ref()
                .unwrap_unchecked()
                .description_with_precedence(prec),
        );
        out.extend_from_slice(this.getOperator().operatorName().as_bytes());
        out.extend_from_slice(
            &this
                .right()
                .as_ref()
                .unwrap_unchecked()
                .description_with_precedence(prec),
        );
        if parens {
            out.push(b')');
        }
        out
    }
}
impl BinaryExpression {
    // Source: src/sksl/ir/SkSLBinaryExpression.cpp:184-192
    pub unsafe fn isAssignmentIntoVariable(&self) -> Option<NonNull<VariableReference>> {
        if self.getOperator().isAssignment() {
            let mut info = SkSLAnalysis::AssignmentInfo::default();
            if SkSLAnalysis::IsAssignable(
                self.left().as_ref().unwrap_unchecked().as_ptr() as *mut dyn Expression,
                Some(&mut info),
                None,
            ) {
                return info.fAssignedVar;
            }
        }
        None
    }
}
