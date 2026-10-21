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
use super::SkSLVariableReference::VariableRefKind;
use crate::src::sksl::{
    SkSLAnalysis,
    SkSLErrorReporter::ErrorReporterExt,
    SkSLOperator::{Operator, OperatorKind},
};
// Source: src/sksl/ir/SkSLPostfixExpression.h:27-27
// Source: src/sksl/ir/SkSLPostfixExpression.h:68-68
// Source: src/sksl/ir/SkSLPostfixExpression.h:69-69
#[repr(C)]
pub struct PostfixExpression {
    base: ExpressionData,
    fOperand: Option<IRNodeBox<dyn Expression>>,
    fOperator: Operator,
}
// Source: src/sksl/ir/SkSLPostfixExpression.h:29-29
impl IRNodeType for PostfixExpression {
    const kIRNodeKind: i32 = ExpressionKind::kPostfix as i32;
}
impl PostfixExpression {
    // Source: src/sksl/ir/SkSLPostfixExpression.h:31-34
    pub unsafe fn new(
        pos: Position,
        operand: Option<IRNodeBox<dyn Expression>>,
        op: Operator,
    ) -> Self {
        let ty = borrow_type(operand.as_ref().unwrap_unchecked().expression_type());
        Self {
            base: ExpressionData::new(pos, ExpressionKind::kPostfix, Some(ty)),
            fOperand: operand,
            fOperator: op,
        }
    }
    // Source: src/sksl/ir/SkSLPostfixExpression.h:48-50
    pub fn getOperator(&self) -> Operator {
        self.fOperator
    }
    // Source: src/sksl/ir/SkSLPostfixExpression.h:52-54
    pub unsafe fn operand_mut(&mut self) -> &mut Option<IRNodeBox<dyn Expression>> {
        &mut self.fOperand
    }
    // Source: src/sksl/ir/SkSLPostfixExpression.h:56-58
    pub fn operand(&self) -> &Option<IRNodeBox<dyn Expression>> {
        &self.fOperand
    }
}
impl IRNode for PostfixExpression {
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
unsafe impl Expression for PostfixExpression {
    fn expression_data(&self) -> &ExpressionData {
        &self.base
    }
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base) }
    }
    fn as_expression(&self) -> &dyn Expression {
        self
    }
    // Source: src/sksl/ir/SkSLPostfixExpression.h:60-63
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        Some(
            IRNodeBox::new_with(|p: *mut Self| {
                p.write(Self::new(
                    pos,
                    self.operand().as_ref().unwrap_unchecked().clone_current(),
                    self.getOperator(),
                ))
            })
            .cast(|p| p as *mut dyn Expression),
        )
    }
    fn description_with_precedence(&self, p: OperatorPrecedence) -> Vec<u8> {
        description(self, p)
    }
}
impl PostfixExpression {
    // Source: src/sksl/ir/SkSLPostfixExpression.cpp:20-34
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        base: IRNodeBox<dyn Expression>,
        op: Operator,
    ) -> Option<IRNodeBox<dyn Expression>> {
        if base.expression_type().isArray() || !base.expression_type().componentType().isNumber() {
            let mut msg = b"'".to_vec();
            msg.extend_from_slice(op.tightOperatorName().as_bytes());
            msg.extend_from_slice(b"' cannot operate on '");
            msg.extend_from_slice(&base.expression_type().displayName());
            msg.push(b'\'');
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &msg);
            return None;
        }
        if !SkSLAnalysis::UpdateVariableRefKind(
            base.as_ptr() as *mut dyn Expression,
            VariableRefKind::kReadWrite,
            Some(context.fErrors),
        ) {
            return None;
        }
        Some(Self::Make(context, pos, base, op))
    }
    // Source: src/sksl/ir/SkSLPostfixExpression.cpp:36-43
    pub unsafe fn Make(
        _context: &Context,
        pos: Position,
        base: IRNodeBox<dyn Expression>,
        op: Operator,
    ) -> IRNodeBox<dyn Expression> {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLPostfixExpression.cpp\0".as_ptr(),40; @condition b"!base->type().isArray() && base->type().componentType().isNumber()\0".as_ptr(); !base.expression_type().isArray()&&base.expression_type().componentType().isNumber());
        debug_assert!(SkSLAnalysis::IsAssignable(
            base.as_ptr() as *mut dyn Expression,
            None,
            None
        ));
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, Some(base), op)))
            .cast(|p| p as *mut dyn Expression)
    }
}
// Source: src/sksl/ir/SkSLPostfixExpression.cpp:45-51
fn description(this: &PostfixExpression, parent: OperatorPrecedence) -> Vec<u8> {
    unsafe {
        let precedence = OperatorPrecedence::kPostfix;
        let parens = precedence.0 >= parent.0;
        let mut out = Vec::new();
        if parens {
            out.push(b'(');
        }
        out.extend_from_slice(
            &this
                .operand()
                .as_ref()
                .unwrap_unchecked()
                .description_with_precedence(precedence),
        );
        out.extend_from_slice(this.getOperator().tightOperatorName().as_bytes());
        if parens {
            out.push(b')');
        }
        out
    }
}
