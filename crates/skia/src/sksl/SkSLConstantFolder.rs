// ConstantFolder completion batch UNVERIFIED; all validation prohibited.
// v47 additions/API alignment are UNVERIFIED; user prohibited all validation commands.
use super::ir::{
    SkSLBinaryExpression::BinaryExpression, SkSLConstructorCompound::ConstructorCompound,
    SkSLConstructorDiagonalMatrix::ConstructorDiagonalMatrix,
    SkSLConstructorSplat::ConstructorSplat, SkSLPrefixExpression::PrefixExpression,
    SkSLVariableReference::VariableRefKind,
};
use super::SkSLErrorReporter::ErrorReporterExt;
use super::{
    ir::{
        SkSLExpression::{ComparisonResult, Expression, ExpressionExt},
        SkSLIRNode::IRNodeBox,
        SkSLLiteral::Literal,
        SkSLType::{Type, TypeKind},
    },
    SkSLContext::Context,
    SkSLOperator::{Operator, OperatorKind},
    SkSLPosition::Position,
};
// Source: src/sksl/SkSLConstantFolder.h:27-27
// Native class is a static-method namespace; no synthetic expression owner.
pub struct ConstantFolder;
// Source: src/sksl/SkSLConstantFolder.cpp:41-50
pub(crate) fn is_vec_or_mat(ty: &dyn Type) -> bool {
    match ty.typeKind() {
        TypeKind::kMatrix | TypeKind::kVector => true,
        _ => false,
    }
}
// Source: src/sksl/SkSLConstantFolder.cpp:52-69
pub(crate) unsafe fn eliminate_no_op_boolean(
    pos: Position,
    left: &dyn Expression,
    op: Operator,
    right: &dyn Expression,
) -> Option<IRNodeBox<dyn Expression>> {
    let rightVal = right.node().as_node::<Literal>().boolValue() != 0;
    if (op.kind() == OperatorKind::LOGICALAND && rightVal)
        || (op.kind() == OperatorKind::LOGICALOR && !rightVal)
        || (op.kind() == OperatorKind::LOGICALXOR && !rightVal)
        || (op.kind() == OperatorKind::EQEQ && rightVal)
        || (op.kind() == OperatorKind::NEQ && !rightVal)
    {
        return left.clone_at(pos);
    }
    None
}
// Source: src/sksl/SkSLConstantFolder.cpp:71-87
pub(crate) unsafe fn short_circuit_boolean(
    pos: Position,
    left: &dyn Expression,
    op: Operator,
    right: &dyn Expression,
) -> Option<IRNodeBox<dyn Expression>> {
    let leftVal = left.node().as_node::<Literal>().boolValue() != 0;
    if (op.kind() == OperatorKind::LOGICALAND && !leftVal)
        || (op.kind() == OperatorKind::LOGICALOR && leftVal)
    {
        return left.clone_at(pos);
    }
    eliminate_no_op_boolean(pos, right, op, left)
}
// Source: src/sksl/SkSLConstantFolder.cpp:89-110
pub(crate) unsafe fn simplify_constant_equality(
    context: &Context,
    pos: Position,
    left: &dyn Expression,
    op: Operator,
    right: &dyn Expression,
) -> Option<IRNodeBox<dyn Expression>> {
    if op.kind() == OperatorKind::EQEQ || op.kind() == OperatorKind::NEQ {
        let mut equality = op.kind() == OperatorKind::EQEQ;
        match left.compareConstant(right) {
            ComparisonResult::kNotEqual => {
                equality = !equality;
                return Some(
                    Literal::MakeBoolContext(context, pos, equality)
                        .cast(|p| p as *mut dyn Expression),
                );
            }
            ComparisonResult::kEqual => {
                return Some(
                    Literal::MakeBoolContext(context, pos, equality)
                        .cast(|p| p as *mut dyn Expression),
                )
            }
            ComparisonResult::kUnknown => {}
        }
    }
    None
}
// Source: src/sksl/SkSLConstantFolder.cpp:112-168
unsafe fn simplify_matrix_multiplication(
    context: &Context,
    pos: Position,
    left: &dyn Expression,
    right: &dyn Expression,
    leftColumns: i32,
    leftRows: i32,
    rightColumns: i32,
    rightRows: i32,
) -> Option<IRNodeBox<dyn Expression>> {
    let componentType = left.expression_type().componentType();
    crate::SkASSERT!(@source b"src/sksl/SkSLConstantFolder.cpp\0".as_ptr(),121; @condition b"componentType.matches(right.type().componentType())\0".as_ptr(); componentType.matches(right.expression_type().componentType()));
    let mut leftVals = [[core::mem::MaybeUninit::<f64>::uninit(); 4]; 4];
    for c in 0..leftColumns {
        for r in 0..leftRows {
            leftVals[c as usize][r as usize]
                .write(left.getConstantValue(c * leftRows + r).unwrap_unchecked());
        }
    }
    let mut rightVals = [[core::mem::MaybeUninit::<f64>::uninit(); 4]; 4];
    for c in 0..rightColumns {
        for r in 0..rightRows {
            rightVals[c as usize][r as usize]
                .write(right.getConstantValue(c * rightRows + r).unwrap_unchecked());
        }
    }
    crate::SkASSERT!(@source b"src/sksl/SkSLConstantFolder.cpp\0".as_ptr(),138; @condition b"leftColumns == rightRows\0".as_ptr(); leftColumns==rightRows);
    let mut outColumns = rightColumns;
    let mut outRows = leftRows;
    let mut args = [core::mem::MaybeUninit::<f64>::uninit(); 16];
    let mut argIndex = 0usize;
    for c in 0..outColumns {
        for r in 0..outRows {
            let mut val = 0.0;
            for dotIdx in 0..leftColumns {
                val += leftVals[dotIdx as usize][r as usize].assume_init()
                    * rightVals[c as usize][dotIdx as usize].assume_init();
            }
            if val >= -(f32::MAX as f64) && val <= f32::MAX as f64 {
                args[argIndex].write(val);
                argIndex += 1;
            } else {
                return None;
            }
        }
    }
    if outColumns == 1 {
        core::mem::swap(&mut outColumns, &mut outRows);
    }
    let resultType = componentType.toCompound(context, outColumns, outRows);
    Some(ConstructorCompound::MakeFromConstants(
        context,
        pos,
        resultType,
        core::slice::from_raw_parts(args.as_ptr().cast::<f64>(), argIndex),
    ))
}
// Source: src/sksl/SkSLConstantFolder.cpp:170-183
unsafe fn simplify_matrix_times_matrix(
    context: &Context,
    pos: Position,
    left: &dyn Expression,
    right: &dyn Expression,
) -> Option<IRNodeBox<dyn Expression>> {
    let lt = left.expression_type();
    let rt = right.expression_type();
    debug_assert!(lt.isMatrix());
    debug_assert!(rt.isMatrix());
    simplify_matrix_multiplication(
        context,
        pos,
        left,
        right,
        lt.columns(),
        lt.rows(),
        rt.columns(),
        rt.rows(),
    )
}
// Source: src/sksl/SkSLConstantFolder.cpp:185-198
unsafe fn simplify_vector_times_matrix(
    context: &Context,
    pos: Position,
    left: &dyn Expression,
    right: &dyn Expression,
) -> Option<IRNodeBox<dyn Expression>> {
    let lt = left.expression_type();
    let rt = right.expression_type();
    debug_assert!(lt.isVector());
    debug_assert!(rt.isMatrix());
    simplify_matrix_multiplication(
        context,
        pos,
        left,
        right,
        lt.columns(),
        1,
        rt.columns(),
        rt.rows(),
    )
}
// Source: src/sksl/SkSLConstantFolder.cpp:200-213
unsafe fn simplify_matrix_times_vector(
    context: &Context,
    pos: Position,
    left: &dyn Expression,
    right: &dyn Expression,
) -> Option<IRNodeBox<dyn Expression>> {
    let lt = left.expression_type();
    let rt = right.expression_type();
    debug_assert!(lt.isMatrix());
    debug_assert!(rt.isVector());
    simplify_matrix_multiplication(
        context,
        pos,
        left,
        right,
        lt.columns(),
        lt.rows(),
        1,
        rt.columns(),
    )
}
// Source: src/sksl/SkSLConstantFolder.cpp:215-258
unsafe fn simplify_componentwise(
    context: &Context,
    pos: Position,
    left: &dyn Expression,
    op: Operator,
    right: &dyn Expression,
) -> Option<IRNodeBox<dyn Expression>> {
    crate::SkASSERT!(@source b"src/sksl/SkSLConstantFolder.cpp\0".as_ptr(),220; @condition b"is_vec_or_mat(left.type())\0".as_ptr(); is_vec_or_mat(left.expression_type()));
    crate::SkASSERT!(@source b"src/sksl/SkSLConstantFolder.cpp\0".as_ptr(),221; @condition b"left.type().matches(right.type())\0".as_ptr(); left.expression_type().matches(right.expression_type()));
    let ty = left.expression_type();
    if let Some(result) = simplify_constant_equality(context, pos, left, op, right) {
        return Some(result);
    }
    let foldFn: fn(f64, f64) -> f64 = match op.kind() {
        OperatorKind::PLUS => |a, b| a + b,
        OperatorKind::MINUS => |a, b| a - b,
        OperatorKind::STAR => |a, b| a * b,
        OperatorKind::SLASH => |a, b| a / b,
        _ => return None,
    };
    let componentType = ty.componentType();
    crate::SkASSERT!(@source b"src/sksl/SkSLConstantFolder.cpp\0".as_ptr(),243; @condition b"componentType.isNumber()\0".as_ptr(); componentType.isNumber());
    let minimumValue = componentType.minimumValue();
    let maximumValue = componentType.maximumValue();
    let mut args = [core::mem::MaybeUninit::<f64>::uninit(); 16];
    let numSlots = ty.slotCount();
    for i in 0..numSlots {
        let value = foldFn(
            left.getConstantValue(i as i32).unwrap_unchecked(),
            right.getConstantValue(i as i32).unwrap_unchecked(),
        );
        if value < minimumValue || value > maximumValue {
            return None;
        }
        args[i].write(value);
    }
    Some(ConstructorCompound::MakeFromConstants(
        context,
        pos,
        ty,
        core::slice::from_raw_parts(args.as_ptr().cast::<f64>(), numSlots),
    ))
}
// Source: src/sksl/SkSLConstantFolder.cpp:260-277
unsafe fn splat_scalar(
    context: &Context,
    scalar: &dyn Expression,
    ty: &dyn Type,
) -> Option<IRNodeBox<dyn Expression>> {
    if ty.isVector() {
        return Some(ConstructorSplat::Make(
            context,
            scalar.node().fPosition,
            ty,
            scalar.clone_current().unwrap_unchecked(),
        ));
    }
    if ty.isMatrix() {
        let numSlots = ty.slotCount();
        let mut splatMatrix = super::SkSLDefines::ExpressionArray::new();
        splatMatrix
            .as_mut()
            .as_base_mut()
            .reserve_exact(numSlots as i32);
        for _index in 0..numSlots {
            splatMatrix
                .as_mut()
                .as_base_mut()
                .push_back(scalar.clone_current());
        }
        return Some(ConstructorCompound::Make(
            context,
            scalar.node().fPosition,
            ty,
            splatMatrix,
        ));
    }
    debug_assert!(false, "unsupported type {:?}", ty.description());
    None
}
// Source: src/sksl/SkSLConstantFolder.cpp:279-297
unsafe fn cast_expression(
    context: &Context,
    pos: Position,
    expr: &dyn Expression,
    ty: &dyn Type,
) -> Option<IRNodeBox<dyn Expression>> {
    debug_assert!(ty
        .componentType()
        .matches(expr.expression_type().componentType()));
    if expr.expression_type().isScalar() {
        if ty.isMatrix() {
            return Some(ConstructorDiagonalMatrix::Make(
                context,
                pos,
                ty,
                expr.clone_current().unwrap_unchecked(),
            ));
        }
        if ty.isVector() {
            return Some(ConstructorSplat::Make(
                context,
                pos,
                ty,
                expr.clone_current().unwrap_unchecked(),
            ));
        }
    }
    if ty.matches(expr.expression_type()) {
        return expr.clone_at(pos);
    }
    None
}
// Source: src/sksl/SkSLConstantFolder.cpp:299-314
unsafe fn zero_expression(
    context: &Context,
    pos: Position,
    ty: &dyn Type,
) -> Option<IRNodeBox<dyn Expression>> {
    let zero = Literal::Make(pos, 0.0, ty.componentType()).cast(|p| p as *mut dyn Expression);
    if ty.isScalar() {
        return Some(zero);
    }
    if ty.isVector() {
        return Some(ConstructorSplat::Make(context, pos, ty, zero));
    }
    if ty.isMatrix() {
        return Some(ConstructorDiagonalMatrix::Make(context, pos, ty, zero));
    }
    debug_assert!(false, "unsupported type {:?}", ty.description());
    None
}
// Source: src/sksl/SkSLConstantFolder.cpp:316-323
unsafe fn negate_expression(
    context: &Context,
    pos: Position,
    expr: &dyn Expression,
    ty: &dyn Type,
) -> Option<IRNodeBox<dyn Expression>> {
    let ctor = cast_expression(context, pos, expr, ty);
    ctor.map(|ctor| PrefixExpression::Make(context, pos, Operator::new(OperatorKind::MINUS), ctor))
}
impl ConstantFolder {
    // v44 new code is unverified by explicit user instruction.
    // Source: src/sksl/SkSLConstantFolder.cpp:325-332
    pub unsafe fn GetConstantInt(
        value: &dyn Expression,
        out: *mut super::SkSLDefines::SKSL_INT,
    ) -> bool {
        let expr = Self::GetConstantValueForVariable(value);
        if !expr.isIntLiteral() {
            return false;
        }
        *out = expr.node().as_node::<Literal>().intValue();
        true
    }
    // Source: src/sksl/SkSLConstantFolder.cpp:334-341
    pub unsafe fn GetConstantValue(value: &dyn Expression, out: *mut f64) -> bool {
        let expr = Self::GetConstantValueForVariable(value);
        if !expr.node().is::<Literal>() {
            return false;
        }
        *out = expr.node().as_node::<Literal>().value();
        true
    }
}
// Source: src/sksl/SkSLConstantFolder.cpp:343-352
pub(crate) fn contains_constant_zero(expr: &dyn Expression) -> bool {
    let numSlots = expr.expression_type().slotCount() as i32;
    for index in 0..numSlots {
        let slotVal = expr.getConstantValue(index);
        if slotVal.is_some() && slotVal.unwrap() == 0.0 {
            return true;
        }
    }
    false
}
impl ConstantFolder {
    // Source: src/sksl/SkSLConstantFolder.cpp:354-363
    pub fn IsConstantSplat(expr: &dyn Expression, value: f64) -> bool {
        let numSlots = expr.expression_type().slotCount() as i32;
        for index in 0..numSlots {
            let slotVal = expr.getConstantValue(index);
            if slotVal.is_none() || slotVal.unwrap() != value {
                return false;
            }
        }
        true
    }
}
// Source: src/sksl/SkSLConstantFolder.cpp:366-384
pub(crate) fn is_constant_diagonal(expr: &dyn Expression, value: f64) -> bool {
    crate::SkASSERT!(@source b"src/sksl/SkSLConstantFolder.cpp\0".as_ptr(),367; @condition b"expr.type().isMatrix()\0".as_ptr(); expr.expression_type().isMatrix());
    let columns = expr.expression_type().columns();
    let rows = expr.expression_type().rows();
    if columns != rows {
        return false;
    }
    let mut slotIdx = 0;
    for c in 0..columns {
        for r in 0..rows {
            let expectation = if c == r { value } else { 0.0 };
            let slotVal = expr.getConstantValue(slotIdx);
            slotIdx += 1;
            if slotVal.is_none() || slotVal.unwrap() != expectation {
                return false;
            }
        }
    }
    true
}
// Source: src/sksl/SkSLConstantFolder.cpp:387-390
pub(crate) fn is_constant_value(expr: &dyn Expression, value: f64) -> bool {
    if expr.expression_type().isMatrix() {
        is_constant_diagonal(expr, value)
    } else {
        ConstantFolder::IsConstantSplat(expr, value)
    }
}
// Source: src/sksl/SkSLConstantFolder.cpp:396-421
unsafe fn make_reciprocal_expression(
    context: &Context,
    right: &dyn Expression,
) -> Option<IRNodeBox<dyn Expression>> {
    if right.expression_type().isMatrix() || !right.expression_type().componentType().isFloat() {
        return None;
    }
    let mut values = [core::mem::MaybeUninit::<f64>::uninit(); 4];
    let nslots = right.expression_type().slotCount();
    for index in 0..nslots {
        let mut value = right.getConstantValue(index as i32)?;
        value = 1.0 / value;
        if value >= -(f32::MAX as f64) && value <= f32::MAX as f64 && value != 0.0 {
            values[index].write(value);
        } else {
            return None;
        }
    }
    Some(ConstructorCompound::MakeFromConstants(
        context,
        right.node().fPosition,
        right.expression_type(),
        core::slice::from_raw_parts(values.as_ptr().cast::<f64>(), nslots),
    ))
}
// Source: src/sksl/SkSLConstantFolder.cpp:423-438
pub(crate) unsafe fn error_on_divide_by_zero(
    context: &Context,
    pos: Position,
    op: Operator,
    right: &dyn Expression,
) -> bool {
    use super::SkSLErrorReporter::ErrorReporterExt;
    match op.kind() {
        OperatorKind::SLASH
        | OperatorKind::SLASHEQ
        | OperatorKind::PERCENT
        | OperatorKind::PERCENTEQ => {
            if contains_constant_zero(right) {
                let mut errors = context.fErrors;
                errors.as_mut().error(pos, b"division by zero");
                return true;
            }
            false
        }
        _ => false,
    }
}
impl ConstantFolder {
    // v44 new code is unverified by explicit user instruction.
    // Source: src/sksl/SkSLConstantFolder.cpp:440-459
    pub unsafe fn GetConstantValueOrNull(inExpr: &dyn Expression) -> Option<&dyn Expression> {
        use super::ir::SkSLVariableReference::{VariableRefKind, VariableReference};
        let mut expr = inExpr;
        while expr.node().is::<VariableReference>() {
            let varRef = expr.node().as_node::<VariableReference>();
            if varRef.refKind() != VariableRefKind::kRead {
                return None;
            }
            let var = varRef.variable().unwrap_unchecked();
            let var = var.as_ref();
            if !var.modifierFlags().isConst() {
                return None;
            }
            let Some(value) = var.initialValue() else {
                return None;
            };
            expr = value;
        }
        if super::SkSLAnalysis::IsCompileTimeConstant(expr) {
            Some(expr)
        } else {
            None
        }
    }
    // Source: src/sksl/SkSLConstantFolder.cpp:461-464
    pub unsafe fn GetConstantValueForVariable(inExpr: &dyn Expression) -> &dyn Expression {
        match Self::GetConstantValueOrNull(inExpr) {
            Some(expr) => expr,
            None => inExpr,
        }
    }
    // Source: src/sksl/SkSLConstantFolder.cpp:466-470
    pub unsafe fn MakeConstantValueForVariable(
        pos: Position,
        inExpr: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Expression> {
        let constant = Self::GetConstantValueOrNull(&*inExpr);
        match constant {
            Some(expr) => expr.clone_at(pos).unwrap_unchecked(),
            None => inExpr,
        }
    }
}
// Source: src/sksl/SkSLConstantFolder.cpp:472-474
pub(crate) fn is_scalar_op_matrix(left: &dyn Expression, right: &dyn Expression) -> bool {
    left.expression_type().isScalar() && right.expression_type().isMatrix()
}
// Source: src/sksl/SkSLConstantFolder.cpp:476-478
pub(crate) fn is_matrix_op_scalar(left: &dyn Expression, right: &dyn Expression) -> bool {
    is_scalar_op_matrix(right, left)
}
// Source: src/sksl/SkSLConstantFolder.cpp:480-610
unsafe fn simplify_arithmetic(
    context: &Context,
    pos: Position,
    left: &dyn Expression,
    op: Operator,
    right: &dyn Expression,
    resultType: &dyn Type,
) -> Option<IRNodeBox<dyn Expression>> {
    match op.kind() {
        OperatorKind::PLUS => {
            if !is_scalar_op_matrix(left, right) && ConstantFolder::IsConstantSplat(right, 0.0) {
                if let Some(expr) = cast_expression(context, pos, left, resultType) {
                    return Some(expr);
                }
            }
            if !is_matrix_op_scalar(left, right) && ConstantFolder::IsConstantSplat(left, 0.0) {
                if let Some(expr) = cast_expression(context, pos, right, resultType) {
                    return Some(expr);
                }
            }
        }
        OperatorKind::STAR => {
            if is_constant_value(right, 1.0) {
                if let Some(expr) = cast_expression(context, pos, left, resultType) {
                    return Some(expr);
                }
            }
            if is_constant_value(left, 1.0) {
                if let Some(expr) = cast_expression(context, pos, right, resultType) {
                    return Some(expr);
                }
            }
            if is_constant_value(right, 0.0) && !super::SkSLAnalysis::HasSideEffects(left) {
                return zero_expression(context, pos, resultType);
            }
            if is_constant_value(left, 0.0) && !super::SkSLAnalysis::HasSideEffects(right) {
                return zero_expression(context, pos, resultType);
            }
            if is_constant_value(right, -1.0) {
                if let Some(expr) = negate_expression(context, pos, left, resultType) {
                    return Some(expr);
                }
            }
            if is_constant_value(left, -1.0) {
                if let Some(expr) = negate_expression(context, pos, right, resultType) {
                    return Some(expr);
                }
            }
        }
        OperatorKind::MINUS => {
            if !is_scalar_op_matrix(left, right) && ConstantFolder::IsConstantSplat(right, 0.0) {
                if let Some(expr) = cast_expression(context, pos, left, resultType) {
                    return Some(expr);
                }
            }
            if !is_matrix_op_scalar(left, right) && ConstantFolder::IsConstantSplat(left, 0.0) {
                if let Some(expr) = negate_expression(context, pos, right, resultType) {
                    return Some(expr);
                }
            }
        }
        OperatorKind::SLASH => {
            if !is_scalar_op_matrix(left, right) && ConstantFolder::IsConstantSplat(right, 1.0) {
                if let Some(expr) = cast_expression(context, pos, left, resultType) {
                    return Some(expr);
                }
            }
            if !left.expression_type().isMatrix() {
                if let Some(expr) = make_reciprocal_expression(context, right) {
                    return Some(BinaryExpression::Make(
                        context,
                        pos,
                        left.clone_current().unwrap_unchecked(),
                        Operator::new(OperatorKind::STAR),
                        expr,
                    ));
                }
            }
        }
        OperatorKind::PLUSEQ | OperatorKind::MINUSEQ => {
            if ConstantFolder::IsConstantSplat(right, 0.0) {
                if let Some(var) = cast_expression(context, pos, left, resultType) {
                    super::SkSLAnalysis::UpdateVariableRefKind(
                        var.as_ptr() as *mut dyn Expression,
                        VariableRefKind::kRead,
                        None,
                    );
                    return Some(var);
                }
            }
        }
        OperatorKind::STAREQ => {
            if is_constant_value(right, 1.0) {
                if let Some(var) = cast_expression(context, pos, left, resultType) {
                    super::SkSLAnalysis::UpdateVariableRefKind(
                        var.as_ptr() as *mut dyn Expression,
                        VariableRefKind::kRead,
                        None,
                    );
                    return Some(var);
                }
            }
        }
        OperatorKind::SLASHEQ => {
            if ConstantFolder::IsConstantSplat(right, 1.0) {
                if let Some(var) = cast_expression(context, pos, left, resultType) {
                    super::SkSLAnalysis::UpdateVariableRefKind(
                        var.as_ptr() as *mut dyn Expression,
                        VariableRefKind::kRead,
                        None,
                    );
                    return Some(var);
                }
            }
            if let Some(expr) = make_reciprocal_expression(context, right) {
                return Some(BinaryExpression::Make(
                    context,
                    pos,
                    left.clone_current().unwrap_unchecked(),
                    Operator::new(OperatorKind::STAREQ),
                    expr,
                ));
            }
        }
        _ => {}
    }
    None
}
// Source: src/sksl/SkSLConstantFolder.cpp:615-623
unsafe fn one_over_scalar(context: &Context, right: &dyn Expression) -> IRNodeBox<dyn Expression> {
    crate::SkASSERT!(@source b"src/sksl/SkSLConstantFolder.cpp\0".as_ptr(),617; @condition b"right.type().isScalar()\0".as_ptr(); right.expression_type().isScalar());
    let pos = right.node().fPosition;
    BinaryExpression::Make(
        context,
        pos,
        Literal::Make(pos, 1.0, right.expression_type()).cast(|p| p as *mut dyn Expression),
        Operator::new(OperatorKind::SLASH),
        right.clone_current().unwrap_unchecked(),
    )
}
// Source: src/sksl/SkSLConstantFolder.cpp:625-651
unsafe fn simplify_matrix_division(
    context: &Context,
    pos: Position,
    left: &dyn Expression,
    op: Operator,
    right: &dyn Expression,
    _resultType: &dyn Type,
) -> Option<IRNodeBox<dyn Expression>> {
    match op.kind() {
        OperatorKind::SLASH | OperatorKind::SLASHEQ => {
            if left.expression_type().isMatrix() && right.expression_type().isScalar() {
                let multiplyOp = if op.isAssignment() {
                    OperatorKind::STAREQ
                } else {
                    OperatorKind::STAR
                };
                return Some(BinaryExpression::Make(
                    context,
                    pos,
                    left.clone_current().unwrap_unchecked(),
                    Operator::new(multiplyOp),
                    one_over_scalar(context, right),
                ));
            }
        }
        _ => {}
    }
    None
}
// Source: src/sksl/SkSLConstantFolder.cpp:653-666
pub(crate) unsafe fn fold_expression(
    pos: Position,
    result: f64,
    resultType: &dyn Type,
) -> Option<IRNodeBox<dyn Expression>> {
    if resultType.isNumber() {
        if result >= resultType.minimumValue() && result <= resultType.maximumValue() {
        } else {
            return None;
        }
    }
    Some(Literal::Make(pos, result, resultType).cast(|p| p as *mut dyn Expression))
}
// Source: src/sksl/SkSLConstantFolder.cpp:668-808
unsafe fn fold_two_constants(
    context: &Context,
    pos: Position,
    left: &dyn Expression,
    op: Operator,
    right: &dyn Expression,
    resultType: &dyn Type,
) -> Option<IRNodeBox<dyn Expression>> {
    debug_assert!(super::SkSLAnalysis::IsCompileTimeConstant(left));
    debug_assert!(super::SkSLAnalysis::IsCompileTimeConstant(right));
    let leftType = left.expression_type();
    let rightType = right.expression_type();
    if left.isIntLiteral() && right.isIntLiteral() {
        let l = left.node().as_node::<Literal>().intValue();
        let rr = right.node().as_node::<Literal>().intValue();
        // Native URESULT arithmetic executes in uint64 then reinterprets int64 before double conversion.
        let value = match op.kind() {
            OperatorKind::PLUS => (l as u64).wrapping_add(rr as u64) as i64 as f64,
            OperatorKind::MINUS => (l as u64).wrapping_sub(rr as u64) as i64 as f64,
            OperatorKind::STAR => (l as u64).wrapping_mul(rr as u64) as i64 as f64,
            OperatorKind::SLASH => {
                if l == i64::MIN && rr == -1 {
                    let mut errors = context.fErrors;
                    errors.as_mut().error(pos, b"arithmetic overflow");
                    return None;
                }
                (l / rr) as f64
            }
            OperatorKind::PERCENT => {
                if l == i64::MIN && rr == -1 {
                    let mut errors = context.fErrors;
                    errors.as_mut().error(pos, b"arithmetic overflow");
                    return None;
                }
                (l % rr) as f64
            }
            OperatorKind::BITWISEAND => (l & rr) as f64,
            OperatorKind::BITWISEOR => (l | rr) as f64,
            OperatorKind::BITWISEXOR => (l ^ rr) as f64,
            OperatorKind::EQEQ => (l == rr) as u8 as f64,
            OperatorKind::NEQ => (l != rr) as u8 as f64,
            OperatorKind::GT => (l > rr) as u8 as f64,
            OperatorKind::GTEQ => (l >= rr) as u8 as f64,
            OperatorKind::LT => (l < rr) as u8 as f64,
            OperatorKind::LTEQ => (l <= rr) as u8 as f64,
            OperatorKind::SHL => {
                if rr >= 0 && rr <= 31 {
                    ((l as u64) << (rr as u32)) as i64 as f64
                } else {
                    let mut errors = context.fErrors;
                    errors.as_mut().error(pos, b"shift value out of range");
                    return None;
                }
            }
            OperatorKind::SHR => {
                if rr >= 0 && rr <= 31 {
                    (l >> (rr as u32)) as f64
                } else {
                    let mut errors = context.fErrors;
                    errors.as_mut().error(pos, b"shift value out of range");
                    return None;
                }
            }
            _ => return None,
        };
        return fold_expression(pos, value, resultType);
    }
    if left.isFloatLiteral() && right.isFloatLiteral() {
        let l = left.node().as_node::<Literal>().floatValue();
        let rr = right.node().as_node::<Literal>().floatValue();
        let value = match op.kind() {
            OperatorKind::PLUS => ((l + rr) as f64),
            OperatorKind::MINUS => ((l - rr) as f64),
            OperatorKind::STAR => ((l * rr) as f64),
            OperatorKind::SLASH => ((l / rr) as f64),
            OperatorKind::EQEQ => (l == rr) as u8 as f64,
            OperatorKind::NEQ => (l != rr) as u8 as f64,
            OperatorKind::GT => (l > rr) as u8 as f64,
            OperatorKind::GTEQ => (l >= rr) as u8 as f64,
            OperatorKind::LT => (l < rr) as u8 as f64,
            OperatorKind::LTEQ => (l <= rr) as u8 as f64,
            _ => return None,
        };
        return fold_expression(pos, value, resultType);
    }
    if op.kind() == OperatorKind::STAR {
        if leftType.isMatrix() && rightType.isMatrix() {
            return simplify_matrix_times_matrix(context, pos, left, right);
        }
        if leftType.isVector() && rightType.isMatrix() {
            return simplify_vector_times_matrix(context, pos, left, right);
        }
        if leftType.isMatrix() && rightType.isVector() {
            return simplify_matrix_times_vector(context, pos, left, right);
        }
    }
    if is_vec_or_mat(leftType) && leftType.matches(rightType) {
        return simplify_componentwise(context, pos, left, op, right);
    }
    if rightType.isScalar()
        && is_vec_or_mat(leftType)
        && leftType.componentType().matches(rightType)
    {
        let splat = splat_scalar(context, right, leftType).unwrap_unchecked();
        return simplify_componentwise(context, pos, left, op, &*splat);
    }
    if leftType.isScalar()
        && is_vec_or_mat(rightType)
        && rightType.componentType().matches(leftType)
    {
        let splat = splat_scalar(context, left, rightType).unwrap_unchecked();
        return simplify_componentwise(context, pos, &*splat, op, right);
    }
    if (leftType.isMatrix() && rightType.isMatrix())
        || (leftType.isArray() && rightType.isArray())
        || (leftType.isStruct() && rightType.isStruct())
    {
        return simplify_constant_equality(context, pos, left, op, right);
    }
    None
}
// Source: src/sksl/SkSLConstantFolder.cpp:810-902
impl ConstantFolder {
    pub unsafe fn Simplify(
        context: &Context,
        pos: Position,
        leftExpr: &dyn Expression,
        op: Operator,
        rightExpr: &dyn Expression,
        resultType: &dyn Type,
    ) -> Option<IRNodeBox<dyn Expression>> {
        let left = Self::GetConstantValueForVariable(leftExpr);
        let right = Self::GetConstantValueForVariable(rightExpr);
        if op.kind() == OperatorKind::EQ && super::SkSLAnalysis::IsSameExpressionTree(left, right) {
            return right.clone_at(pos);
        }
        if left.isBoolLiteral() && right.isBoolLiteral() {
            let l = left.node().as_node::<Literal>().boolValue() != 0;
            let r = right.node().as_node::<Literal>().boolValue() != 0;
            let result = match op.kind() {
                OperatorKind::LOGICALAND => l && r,
                OperatorKind::LOGICALOR => l || r,
                OperatorKind::LOGICALXOR => l ^ r,
                OperatorKind::EQEQ => l == r,
                OperatorKind::NEQ => l != r,
                _ => return None,
            };
            return Some(
                Literal::MakeBoolContext(context, pos, result).cast(|p| p as *mut dyn Expression),
            );
        }
        if left.isBoolLiteral() {
            return short_circuit_boolean(pos, left, op, right);
        }
        if right.isBoolLiteral() {
            if !super::SkSLAnalysis::HasSideEffects(left) {
                return short_circuit_boolean(pos, right, op, left);
            }
            return eliminate_no_op_boolean(pos, left, op, right);
        }
        if op.kind() == OperatorKind::EQEQ && super::SkSLAnalysis::IsSameExpressionTree(left, right)
        {
            return Some(
                Literal::MakeBoolContext(context, pos, true).cast(|p| p as *mut dyn Expression),
            );
        }
        if op.kind() == OperatorKind::NEQ && super::SkSLAnalysis::IsSameExpressionTree(left, right)
        {
            return Some(
                Literal::MakeBoolContext(context, pos, false).cast(|p| p as *mut dyn Expression),
            );
        }
        if error_on_divide_by_zero(context, pos, op, right) {
            return None;
        }
        let leftSideIsConstant = super::SkSLAnalysis::IsCompileTimeConstant(left);
        let rightSideIsConstant = super::SkSLAnalysis::IsCompileTimeConstant(right);
        if leftSideIsConstant && rightSideIsConstant {
            return fold_two_constants(context, pos, left, op, right, resultType);
        }
        if context
            .fConfig
            .unwrap_unchecked()
            .as_ref()
            .fSettings
            .fOptimize
        {
            if leftSideIsConstant || rightSideIsConstant {
                if let Some(expr) = simplify_arithmetic(context, pos, left, op, right, resultType) {
                    return Some(expr);
                }
            }
            if let Some(expr) = simplify_matrix_division(context, pos, left, op, right, resultType)
            {
                return Some(expr);
            }
        }
        None
    }
}
