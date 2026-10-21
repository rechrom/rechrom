// New translation is UNVERIFIED by explicit user instruction.
use super::{
    SkSLExpression::{Expression, ExpressionData, ExpressionExt},
    SkSLIRNode::*,
    SkSLType::Type,
};
use crate::src::sksl::{
    SkSLContext::Context,
    SkSLOperator::{Operator, OperatorKind, OperatorPrecedence},
    SkSLPosition::Position,
};
use core::{pin::Pin, ptr::NonNull};
unsafe fn borrow_type(ty: &dyn Type) -> NonNull<dyn Type> {
    core::mem::transmute::<NonNull<dyn Type + '_>, NonNull<dyn Type>>(NonNull::from(ty))
}
// Source: src/sksl/ir/SkSLPrefixExpression.h:27-27
// Source: src/sksl/ir/SkSLPrefixExpression.h:65-66
// Adapted inherited state and contiguous field group preserve original field order.
#[repr(C)]
pub struct PrefixExpression {
    base: ExpressionData,
    fOperator: Operator,
    fOperand: Option<IRNodeBox<dyn Expression>>,
}
// Source: src/sksl/ir/SkSLPrefixExpression.h:29-29
impl IRNodeType for PrefixExpression {
    const kIRNodeKind: i32 = ExpressionKind::kPrefix as i32;
}
impl PrefixExpression {
    // Source: src/sksl/ir/SkSLPrefixExpression.h:32-35
    pub unsafe fn new(
        pos: Position,
        op: Operator,
        operand: Option<IRNodeBox<dyn Expression>>,
    ) -> Self {
        let ty = borrow_type(operand.as_ref().unwrap_unchecked().expression_type());
        Self {
            base: ExpressionData::new(pos, ExpressionKind::kPrefix, Some(ty)),
            fOperator: op,
            fOperand: operand,
        }
    }
    // Source: src/sksl/ir/SkSLPrefixExpression.h:45-47
    pub fn getOperator(&self) -> Operator {
        self.fOperator
    }
    // Source: src/sksl/ir/SkSLPrefixExpression.h:49-51
    pub unsafe fn operand_mut(&mut self) -> &mut Option<IRNodeBox<dyn Expression>> {
        &mut self.fOperand
    }
    // Source: src/sksl/ir/SkSLPrefixExpression.h:53-55
    pub fn operand(&self) -> &Option<IRNodeBox<dyn Expression>> {
        &self.fOperand
    }
}
impl IRNode for PrefixExpression {
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
unsafe impl Expression for PrefixExpression {
    fn expression_data(&self) -> &ExpressionData {
        &self.base
    }
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base) }
    }
    fn as_expression(&self) -> &dyn Expression {
        self
    }
    // Source: src/sksl/ir/SkSLPrefixExpression.h:57-60
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        Some(
            IRNodeBox::new_with(|p: *mut Self| {
                p.write(Self::new(
                    pos,
                    self.getOperator(),
                    self.operand().as_ref().unwrap_unchecked().clone_current(),
                ))
            })
            .cast(|p| p as *mut dyn Expression),
        )
    }
    fn description_with_precedence(&self, p: OperatorPrecedence) -> Vec<u8> {
        description(self, p)
    }
}
use super::{
    SkSLBinaryExpression::BinaryExpression, SkSLConstructorArray::ConstructorArray,
    SkSLConstructorCompound::ConstructorCompound,
    SkSLConstructorDiagonalMatrix::ConstructorDiagonalMatrix, SkSLLiteral::Literal,
    SkSLVariableReference::VariableRefKind,
};
use crate::src::sksl::{
    SkSLConstantFolder::ConstantFolder, SkSLDefines::ExpressionArray,
    SkSLErrorReporter::ErrorReporterExt,
};
// Source: src/sksl/ir/SkSLPrefixExpression.cpp:35-37
fn negate_value(value: f64) -> f64 {
    -value
}
// Source: src/sksl/ir/SkSLPrefixExpression.cpp:39-41
fn bitwise_not_value(value: f64) -> f64 {
    if !value.is_finite() || value < -9223372036854775808.0 || value >= 9223372036854775808.0 {
        std::process::abort();
    }
    (!(value as i64)) as f64
}
// Source: src/sksl/ir/SkSLPrefixExpression.cpp:43-69
unsafe fn apply_to_elements(
    context: &Context,
    pos: Position,
    expr: &dyn Expression,
    f: fn(f64) -> f64,
) -> Option<IRNodeBox<dyn Expression>> {
    let elementType = expr.expression_type().componentType();
    let mut values = [0.0; 16];
    let numSlots = expr.expression_type().slotCount();
    if numSlots > values.len() {
        return None;
    }
    for index in 0..numSlots {
        if let Some(value) = expr.getConstantValue(index as i32) {
            values[index] = f(value);
            if super::SkSLType::checkForOutOfRangeLiteralValue(
                elementType,
                context,
                values[index],
                pos,
            ) {
                return None;
            }
        } else {
            return None;
        }
    }
    Some(ConstructorCompound::MakeFromConstants(
        context,
        pos,
        expr.expression_type(),
        &values,
    ))
}
// Source: src/sksl/ir/SkSLPrefixExpression.cpp:71-120
unsafe fn simplify_negation(
    context: &Context,
    pos: Position,
    originalExpr: &dyn Expression,
) -> Option<IRNodeBox<dyn Expression>> {
    let value = ConstantFolder::GetConstantValueForVariable(originalExpr);
    match value.kind() {
        super::SkSLExpression::Kind::kLiteral
        | super::SkSLExpression::Kind::kConstructorSplat
        | super::SkSLExpression::Kind::kConstructorCompound => {
            if let Some(expr) = apply_to_elements(context, pos, value, negate_value) {
                return Some(expr);
            }
        }
        super::SkSLExpression::Kind::kPrefix => {
            let prefix = value.node().as_node::<PrefixExpression>();
            if prefix.getOperator().kind() == OperatorKind::MINUS {
                return prefix.operand().as_ref().unwrap_unchecked().clone_at(pos);
            }
        }
        super::SkSLExpression::Kind::kConstructorArray => {
            if crate::src::sksl::SkSLAnalysis::IsCompileTimeConstant(value) {
                let ctor = value.node().as_node::<ConstructorArray>();
                return Some(ConstructorArray::Make(
                    context,
                    pos,
                    ctor.expression_type(),
                    negate_operands(context, pos, ctor.arguments()),
                ));
            }
        }
        super::SkSLExpression::Kind::kConstructorDiagonalMatrix => {
            if crate::src::sksl::SkSLAnalysis::IsCompileTimeConstant(value) {
                let ctor = value.node().as_node::<ConstructorDiagonalMatrix>();
                if let Some(simplified) =
                    simplify_negation(context, pos, &**ctor.argument().as_ref().unwrap_unchecked())
                {
                    return Some(ConstructorDiagonalMatrix::Make(
                        context,
                        pos,
                        ctor.expression_type(),
                        simplified,
                    ));
                }
            }
        }
        _ => {}
    }
    None
}
// Source: src/sksl/ir/SkSLPrefixExpression.cpp:122-137
unsafe fn negate_operands(
    context: &Context,
    pos: Position,
    array: &ExpressionArray,
) -> Pin<Box<ExpressionArray>> {
    let mut replacement = ExpressionArray::new();
    replacement
        .as_mut()
        .as_base_mut()
        .reserve_exact(array.as_base().size());
    for expr in array.as_base().as_slice() {
        let expr = expr.as_ref().unwrap_unchecked();
        if let Some(simplified) = simplify_negation(context, pos, &**expr) {
            replacement
                .as_mut()
                .as_base_mut()
                .push_back(Some(simplified));
        } else {
            replacement.as_mut().as_base_mut().push_back(Some(
                IRNodeBox::new_with(|p: *mut PrefixExpression| {
                    p.write(PrefixExpression::new(
                        pos,
                        Operator::new(OperatorKind::MINUS),
                        expr.clone_current(),
                    ))
                })
                .cast(|p| p as *mut dyn Expression),
            ));
        }
    }
    replacement
}
// Source: src/sksl/ir/SkSLPrefixExpression.cpp:139-149
unsafe fn negate_operand(
    context: &Context,
    pos: Position,
    value: IRNodeBox<dyn Expression>,
) -> IRNodeBox<dyn Expression> {
    if let Some(simplified) = simplify_negation(context, pos, &*value) {
        return simplified;
    }
    IRNodeBox::new_with(|p: *mut PrefixExpression| {
        p.write(PrefixExpression::new(
            pos,
            Operator::new(OperatorKind::MINUS),
            Some(value),
        ))
    })
    .cast(|p| p as *mut dyn Expression)
}
// Source: src/sksl/ir/SkSLPrefixExpression.cpp:151-196
unsafe fn logical_not_operand(
    context: &Context,
    pos: Position,
    operand: IRNodeBox<dyn Expression>,
) -> IRNodeBox<dyn Expression> {
    let value = ConstantFolder::GetConstantValueForVariable(&*operand);
    match value.kind() {
        super::SkSLExpression::Kind::kLiteral => {
            crate::SkASSERT!(@source b"src/sksl/ir/SkSLPrefixExpression.cpp\0".as_ptr(),158; @condition b"value->type().isBoolean()\0".as_ptr(); value.expression_type().isBoolean());
            let b = value.node().as_node::<Literal>();
            return Literal::MakeBool(pos, b.boolValue() == 0, operand.expression_type())
                .cast(|p| p as *mut dyn Expression);
        }
        super::SkSLExpression::Kind::kPrefix => {
            debug_assert!(operand.node().is::<PrefixExpression>());
            let prefix = &mut *(operand.as_ptr() as *mut () as *mut PrefixExpression);
            if prefix.getOperator().kind() == OperatorKind::LOGICALNOT {
                let mut result = prefix.operand_mut().take().unwrap_unchecked();
                result.as_pin_mut().node_mut().setPosition(pos);
                return result;
            }
        }
        super::SkSLExpression::Kind::kBinary => {
            debug_assert!(operand.node().is::<BinaryExpression>());
            let binary = &mut *(operand.as_ptr() as *mut () as *mut BinaryExpression);
            let replacement = match binary.getOperator().kind() {
                OperatorKind::EQEQ => Some(OperatorKind::NEQ),
                OperatorKind::NEQ => Some(OperatorKind::EQEQ),
                OperatorKind::LT => Some(OperatorKind::GTEQ),
                OperatorKind::LTEQ => Some(OperatorKind::GT),
                OperatorKind::GT => Some(OperatorKind::LTEQ),
                OperatorKind::GTEQ => Some(OperatorKind::LT),
                _ => None,
            };
            if let Some(replacement) = replacement {
                let ty = borrow_type(binary.expression_type());
                let left = binary.left_mut().take().unwrap_unchecked();
                let right = binary.right_mut().take().unwrap_unchecked();
                return BinaryExpression::MakeWithType(
                    context,
                    pos,
                    left,
                    Operator::new(replacement),
                    right,
                    ty,
                );
            }
        }
        _ => {}
    }
    IRNodeBox::new_with(|p: *mut PrefixExpression| {
        p.write(PrefixExpression::new(
            pos,
            Operator::new(OperatorKind::LOGICALNOT),
            Some(operand),
        ))
    })
    .cast(|p| p as *mut dyn Expression)
}
// Source: src/sksl/ir/SkSLPrefixExpression.cpp:198-231
unsafe fn bitwise_not_operand(
    context: &Context,
    pos: Position,
    operand: IRNodeBox<dyn Expression>,
) -> IRNodeBox<dyn Expression> {
    crate::SkASSERT!(@source b"src/sksl/ir/SkSLPrefixExpression.cpp\0".as_ptr(),201; @condition b"operand->type().componentType().isInteger()\0".as_ptr(); operand.expression_type().componentType().isInteger());
    let value = ConstantFolder::GetConstantValueForVariable(&*operand);
    match value.kind() {
        super::SkSLExpression::Kind::kLiteral
        | super::SkSLExpression::Kind::kConstructorSplat
        | super::SkSLExpression::Kind::kConstructorCompound => {
            if let Some(expr) = apply_to_elements(context, pos, value, bitwise_not_value) {
                return expr;
            }
        }
        super::SkSLExpression::Kind::kPrefix => {
            debug_assert!(operand.node().is::<PrefixExpression>());
            let prefix = &mut *(operand.as_ptr() as *mut () as *mut PrefixExpression);
            if prefix.getOperator().kind() == OperatorKind::BITWISENOT {
                let mut result = prefix.operand_mut().take().unwrap_unchecked();
                result.as_pin_mut().node_mut().setPosition(pos);
                return result;
            }
        }
        _ => {}
    }
    IRNodeBox::new_with(|p: *mut PrefixExpression| {
        p.write(PrefixExpression::new(
            pos,
            Operator::new(OperatorKind::BITWISENOT),
            Some(operand),
        ))
    })
    .cast(|p| p as *mut dyn Expression)
}
impl PrefixExpression {
    // Source: src/sksl/ir/SkSLPrefixExpression.cpp:233-301
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        op: Operator,
        base: IRNodeBox<dyn Expression>,
    ) -> Option<IRNodeBox<dyn Expression>> {
        let baseType = base.expression_type();
        match op.kind() {
            OperatorKind::PLUS => {
                if baseType.isArray() || !baseType.componentType().isNumber() {
                    let mut message = b"'+' cannot operate on '".to_vec();
                    message.extend_from_slice(&baseType.displayName());
                    message.push(b'\'');
                    let mut errors = context.fErrors;
                    errors.as_mut().error(pos, &message);
                    return None;
                }
            }
            OperatorKind::MINUS => {
                if baseType.isArray() || !baseType.componentType().isNumber() {
                    let mut message = b"'-' cannot operate on '".to_vec();
                    message.extend_from_slice(&baseType.displayName());
                    message.push(b'\'');
                    let mut errors = context.fErrors;
                    errors.as_mut().error(pos, &message);
                    return None;
                }
            }
            OperatorKind::PLUSPLUS | OperatorKind::MINUSMINUS => {
                if baseType.isArray() || !baseType.componentType().isNumber() {
                    let mut message = b"'".to_vec();
                    message.extend_from_slice(op.tightOperatorName().as_bytes());
                    message.extend_from_slice(b"' cannot operate on '");
                    message.extend_from_slice(&baseType.displayName());
                    message.push(b'\'');
                    let mut errors = context.fErrors;
                    errors.as_mut().error(pos, &message);
                    return None;
                }
                if !crate::src::sksl::SkSLAnalysis::UpdateVariableRefKind(
                    base.as_ptr() as *mut dyn Expression,
                    VariableRefKind::kReadWrite,
                    Some(context.fErrors),
                ) {
                    return None;
                }
            }
            OperatorKind::LOGICALNOT => {
                if !baseType.isBoolean() {
                    let mut message = b"'".to_vec();
                    message.extend_from_slice(op.tightOperatorName().as_bytes());
                    message.extend_from_slice(b"' cannot operate on '");
                    message.extend_from_slice(&baseType.displayName());
                    message.push(b'\'');
                    let mut errors = context.fErrors;
                    errors.as_mut().error(pos, &message);
                    return None;
                }
            }
            OperatorKind::BITWISENOT => {
                if context.fConfig.unwrap_unchecked().as_ref().strictES2Mode() {
                    let mut message = b"operator '".to_vec();
                    message.extend_from_slice(op.tightOperatorName().as_bytes());
                    message.extend_from_slice(b"' is not allowed");
                    let mut errors = context.fErrors;
                    errors.as_mut().error(pos, &message);
                    return None;
                }
                if baseType.isArray() || !baseType.componentType().isInteger() {
                    let mut message = b"'".to_vec();
                    message.extend_from_slice(op.tightOperatorName().as_bytes());
                    message.extend_from_slice(b"' cannot operate on '");
                    message.extend_from_slice(&baseType.displayName());
                    message.push(b'\'');
                    let mut errors = context.fErrors;
                    errors.as_mut().error(pos, &message);
                    return None;
                }
            }
            _ => std::process::abort(),
        }
        let result = Self::Make(context, pos, op, base);
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLPrefixExpression.cpp\0".as_ptr(),299; @condition b"result->fPosition == pos\0".as_ptr(); result.node().fPosition==pos);
        Some(result)
    }
    // Source: src/sksl/ir/SkSLPrefixExpression.cpp:303-347
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        op: Operator,
        mut base: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Expression> {
        let baseType = base.expression_type();
        match op.kind() {
            OperatorKind::PLUS => {
                debug_assert!(!baseType.isArray());
                debug_assert!(baseType.componentType().isNumber());
                base.as_pin_mut().node_mut().setPosition(pos);
                return base;
            }
            OperatorKind::MINUS => {
                debug_assert!(!baseType.isArray());
                debug_assert!(baseType.componentType().isNumber());
                return negate_operand(context, pos, base);
            }
            OperatorKind::LOGICALNOT => {
                crate::SkASSERT!(@source b"src/sksl/ir/SkSLPrefixExpression.cpp\0".as_ptr(),321; @condition b"baseType.isBoolean()\0".as_ptr(); baseType.isBoolean());
                return logical_not_operand(context, pos, base);
            }
            OperatorKind::PLUSPLUS | OperatorKind::MINUSMINUS => {
                debug_assert!(!baseType.isArray());
                debug_assert!(baseType.componentType().isNumber());
                debug_assert!(crate::src::sksl::SkSLAnalysis::IsAssignable(
                    base.as_ptr() as *mut dyn Expression,
                    None,
                    None
                ));
            }
            OperatorKind::BITWISENOT => {
                debug_assert!(!context.fConfig.unwrap_unchecked().as_ref().strictES2Mode());
                debug_assert!(!baseType.isArray());
                crate::SkASSERT!(@source b"src/sksl/ir/SkSLPrefixExpression.cpp\0".as_ptr(),334; @condition b"baseType.componentType().isInteger()\0".as_ptr(); baseType.componentType().isInteger());
                if baseType.isLiteral() {
                    let ty = borrow_type(baseType.scalarTypeForLiteral());
                    base = super::SkSLType::coerceExpression(ty.as_ref(), Some(base), context)
                        .unwrap_unchecked();
                }
                return bitwise_not_operand(context, pos, base);
            }
            _ => debug_assert!(false, "unsupported prefix operator: {}", op.operatorName()),
        }
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, op, Some(base))))
            .cast(|p| p as *mut dyn Expression)
    }
}
// Source: src/sksl/ir/SkSLPrefixExpression.cpp:349-355
fn description(this: &PrefixExpression, parentPrecedence: OperatorPrecedence) -> Vec<u8> {
    unsafe {
        let needsParens = OperatorPrecedence::kPrefix >= parentPrecedence;
        let mut result = if needsParens {
            b"(".to_vec()
        } else {
            Vec::new()
        };
        result.extend_from_slice(this.getOperator().tightOperatorName().as_bytes());
        result.extend_from_slice(
            &this
                .operand()
                .as_ref()
                .unwrap_unchecked()
                .description_with_precedence(OperatorPrecedence::kPrefix),
        );
        if needsParens {
            result.push(b')');
        }
        result
    }
}
