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
    SkSLBinaryExpression::BinaryExpression, SkSLConstructorScalarCast::ConstructorScalarCast,
    SkSLLiteral::Literal, SkSLPrefixExpression::PrefixExpression,
};
use crate::src::sksl::{
    SkSLAnalysis,
    SkSLConstantFolder::ConstantFolder,
    SkSLErrorReporter::ErrorReporterExt,
    SkSLOperator::{Operator, OperatorKind},
};
use core::mem::ManuallyDrop;
// Source: src/sksl/ir/SkSLTernaryExpression.h:30-30
// Source: src/sksl/ir/SkSLTernaryExpression.h:91-93
#[repr(C)]
pub struct TernaryExpression {
    base: ExpressionData,
    fTest: ManuallyDrop<Option<IRNodeBox<dyn Expression>>>,
    fIfTrue: ManuallyDrop<Option<IRNodeBox<dyn Expression>>>,
    fIfFalse: ManuallyDrop<Option<IRNodeBox<dyn Expression>>>,
}
// Source: src/sksl/ir/SkSLTernaryExpression.h:32-32
impl IRNodeType for TernaryExpression {
    const kIRNodeKind: i32 = ExpressionKind::kTernary as i32;
}
impl TernaryExpression {
    // Source: src/sksl/ir/SkSLTernaryExpression.h:34-41
    pub unsafe fn new(
        pos: Position,
        test: Option<IRNodeBox<dyn Expression>>,
        ifTrue: Option<IRNodeBox<dyn Expression>>,
        ifFalse: Option<IRNodeBox<dyn Expression>>,
    ) -> Self {
        let ty = borrow_type(ifTrue.as_ref().unwrap_unchecked().expression_type());
        let value = Self {
            base: ExpressionData::new(pos, ExpressionKind::kTernary, Some(ty)),
            fTest: ManuallyDrop::new(test),
            fIfTrue: ManuallyDrop::new(ifTrue),
            fIfFalse: ManuallyDrop::new(ifFalse),
        };
        debug_assert!(value
            .ifTrue()
            .as_ref()
            .unwrap_unchecked()
            .expression_type()
            .matches(
                value
                    .ifFalse()
                    .as_ref()
                    .unwrap_unchecked()
                    .expression_type()
            ));
        value
    }
    // Source: src/sksl/ir/SkSLTernaryExpression.h:58-64
    pub unsafe fn test_mut(&mut self) -> &mut Option<IRNodeBox<dyn Expression>> {
        &mut self.fTest
    }
    pub fn test(&self) -> &Option<IRNodeBox<dyn Expression>> {
        &self.fTest
    }
    // Source: src/sksl/ir/SkSLTernaryExpression.h:66-72
    pub unsafe fn ifTrue_mut(&mut self) -> &mut Option<IRNodeBox<dyn Expression>> {
        &mut self.fIfTrue
    }
    pub fn ifTrue(&self) -> &Option<IRNodeBox<dyn Expression>> {
        &self.fIfTrue
    }
    // Source: src/sksl/ir/SkSLTernaryExpression.h:74-80
    pub unsafe fn ifFalse_mut(&mut self) -> &mut Option<IRNodeBox<dyn Expression>> {
        &mut self.fIfFalse
    }
    pub fn ifFalse(&self) -> &Option<IRNodeBox<dyn Expression>> {
        &self.fIfFalse
    }
}
// Native reverse member destruction: false, true, test.
impl Drop for TernaryExpression {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fIfFalse);
            ManuallyDrop::drop(&mut self.fIfTrue);
            ManuallyDrop::drop(&mut self.fTest);
        }
    }
}
impl IRNode for TernaryExpression {
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
unsafe impl Expression for TernaryExpression {
    fn expression_data(&self) -> &ExpressionData {
        &self.base
    }
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base) }
    }
    fn as_expression(&self) -> &dyn Expression {
        self
    }
    // Source: src/sksl/ir/SkSLTernaryExpression.h:82-86
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        Some(
            IRNodeBox::new_with(|p: *mut Self| {
                p.write(Self::new(
                    pos,
                    self.test().as_ref().unwrap_unchecked().clone_current(),
                    self.ifTrue().as_ref().unwrap_unchecked().clone_current(),
                    self.ifFalse().as_ref().unwrap_unchecked().clone_current(),
                ))
            })
            .cast(|p| p as *mut dyn Expression),
        )
    }
    fn description_with_precedence(&self, p: OperatorPrecedence) -> Vec<u8> {
        description(self, p)
    }
}
impl TernaryExpression {
    // Source: src/sksl/ir/SkSLTernaryExpression.cpp:24-70
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        test: Option<IRNodeBox<dyn Expression>>,
        ifTrue: Option<IRNodeBox<dyn Expression>>,
        ifFalse: Option<IRNodeBox<dyn Expression>>,
    ) -> Option<IRNodeBox<dyn Expression>> {
        let test =
            super::SkSLType::coerceExpression(&**context.fTypes.as_ref().fBool, test, context);
        if test.is_none() || ifTrue.is_none() || ifFalse.is_none() {
            return None;
        }
        let test = test.unwrap_unchecked();
        let ifTrue = ifTrue.unwrap_unchecked();
        let ifFalse = ifFalse.unwrap_unchecked();
        if ifTrue.expression_type().componentType().isOpaque() {
            let mut msg = b"ternary expression of opaque type '".to_vec();
            msg.extend_from_slice(&ifTrue.expression_type().displayName());
            msg.extend_from_slice(b"' is not allowed");
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &msg);
            return None;
        }
        let mut trueType = ifTrue.expression_type();
        let mut falseType = ifFalse.expression_type();
        let mut resultType = trueType;
        let equalityOp = Operator::new(OperatorKind::EQEQ);
        if !equalityOp.determineBinaryType(
            context,
            ifTrue.expression_type(),
            ifFalse.expression_type(),
            &mut trueType,
            &mut falseType,
            &mut resultType,
        ) || !trueType.matches(falseType)
        {
            let errorPos = ifTrue
                .node()
                .fPosition
                .rangeThrough(ifFalse.node().fPosition);
            let mut errors = context.fErrors;
            if ifTrue.expression_type().isVoid() {
                errors.as_mut().error(
                    errorPos,
                    b"ternary expression of type 'void' is not allowed",
                );
            } else {
                let mut msg = b"ternary operator result mismatch: '".to_vec();
                msg.extend_from_slice(&ifTrue.expression_type().displayName());
                msg.extend_from_slice(b"', '");
                msg.extend_from_slice(&ifFalse.expression_type().displayName());
                msg.push(b'\'');
                errors.as_mut().error(errorPos, &msg);
            }
            return None;
        }
        if trueType.isOrContainsArray() {
            let mut errors = context.fErrors;
            errors.as_mut().error(
                pos,
                b"ternary operator result may not be an array (or struct containing an array)",
            );
            return None;
        }
        let tt = borrow_type(trueType);
        let ft = borrow_type(falseType);
        let ifTrue = super::SkSLType::coerceExpression(tt.as_ref(), Some(ifTrue), context)?;
        let ifFalse = super::SkSLType::coerceExpression(ft.as_ref(), Some(ifFalse), context)?;
        Some(Self::Make(context, pos, test, ifTrue, ifFalse))
    }
    // Source: src/sksl/ir/SkSLTernaryExpression.cpp:72-137
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        test: IRNodeBox<dyn Expression>,
        mut ifTrue: IRNodeBox<dyn Expression>,
        mut ifFalse: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Expression> {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLTernaryExpression.cpp\0".as_ptr(),77; @condition b"ifTrue->type().matches(ifFalse->type())\0".as_ptr(); ifTrue.expression_type().matches(ifFalse.expression_type()));
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLTernaryExpression.cpp\0".as_ptr(),78; @condition b"!ifTrue->type().componentType().isOpaque()\0".as_ptr(); !ifTrue.expression_type().componentType().isOpaque());
        debug_assert!(
            !context.fConfig.unwrap_unchecked().as_ref().strictES2Mode()
                || !ifTrue.expression_type().isOrContainsArray()
        );
        let testExpr = ConstantFolder::GetConstantValueForVariable(&*test);
        if testExpr.isBoolLiteral() {
            if testExpr.node().as_node::<Literal>().boolValue() != 0 {
                ifTrue.as_pin_mut().node_mut().setPosition(pos);
                return ifTrue;
            } else {
                ifFalse.as_pin_mut().node_mut().setPosition(pos);
                return ifFalse;
            }
        }
        if context
            .fConfig
            .unwrap_unchecked()
            .as_ref()
            .fSettings
            .fOptimize
        {
            let t = ConstantFolder::GetConstantValueForVariable(&*ifTrue);
            let f = ConstantFolder::GetConstantValueForVariable(&*ifFalse);
            if SkSLAnalysis::IsSameExpressionTree(t, f) {
                if !SkSLAnalysis::HasSideEffects(&*test) {
                    ifTrue.as_pin_mut().node_mut().setPosition(pos);
                    return ifTrue;
                }
                return BinaryExpression::Make(
                    context,
                    pos,
                    test,
                    Operator::new(OperatorKind::COMMA),
                    ifTrue,
                );
            }
            if f.isBoolLiteral() && f.node().as_node::<Literal>().boolValue() == 0 {
                return BinaryExpression::Make(
                    context,
                    pos,
                    test,
                    Operator::new(OperatorKind::LOGICALAND),
                    ifTrue,
                );
            }
            if t.isBoolLiteral() && t.node().as_node::<Literal>().boolValue() != 0 {
                return BinaryExpression::Make(
                    context,
                    pos,
                    test,
                    Operator::new(OperatorKind::LOGICALOR),
                    ifFalse,
                );
            }
            if t.isBoolLiteral()
                && t.node().as_node::<Literal>().boolValue() == 0
                && f.isBoolLiteral()
                && f.node().as_node::<Literal>().boolValue() != 0
            {
                return PrefixExpression::Make(
                    context,
                    pos,
                    Operator::new(OperatorKind::LOGICALNOT),
                    test,
                );
            }
            if t.node().is::<Literal>()
                && t.node().as_node::<Literal>().value() == 1.0
                && f.node().is::<Literal>()
                && f.node().as_node::<Literal>().value() == 0.0
            {
                return ConstructorScalarCast::Make(context, pos, ifTrue.expression_type(), test);
            }
        }
        IRNodeBox::new_with(|p: *mut Self| {
            p.write(Self::new(pos, Some(test), Some(ifTrue), Some(ifFalse)))
        })
        .cast(|p| p as *mut dyn Expression)
    }
}
// Source: src/sksl/ir/SkSLTernaryExpression.cpp:139-146
fn description(this: &TernaryExpression, parent: OperatorPrecedence) -> Vec<u8> {
    unsafe {
        let p = OperatorPrecedence::kTernary;
        let parens = p.0 >= parent.0;
        let mut out = Vec::new();
        if parens {
            out.push(b'(');
        }
        out.extend_from_slice(
            &this
                .test()
                .as_ref()
                .unwrap_unchecked()
                .description_with_precedence(p),
        );
        out.extend_from_slice(b" ? ");
        out.extend_from_slice(
            &this
                .ifTrue()
                .as_ref()
                .unwrap_unchecked()
                .description_with_precedence(p),
        );
        out.extend_from_slice(b" : ");
        out.extend_from_slice(
            &this
                .ifFalse()
                .as_ref()
                .unwrap_unchecked()
                .description_with_precedence(p),
        );
        if parens {
            out.push(b')');
        }
        out
    }
}
