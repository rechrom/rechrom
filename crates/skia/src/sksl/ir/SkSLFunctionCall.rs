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
    SkSLConstructorCompound::ConstructorCompound,
    SkSLFieldAccess::FieldAccess,
    SkSLFunctionDeclaration::{FunctionDeclaration, ParamTypes},
    SkSLIndexExpression::IndexExpression,
    SkSLLayout::LayoutFlag,
    SkSLLiteral::Literal,
    SkSLModifierFlags::ModifierFlag,
    SkSLSymbol::Symbol,
    SkSLType::CoercionCost,
    SkSLVariable::Variable,
    SkSLVariableReference::{VariableRefKind, VariableReference},
};
use crate::src::sksl::{
    SkSLAnalysis,
    SkSLConstantFolder::ConstantFolder,
    SkSLDefines::ExpressionArray,
    SkSLErrorReporter::ErrorReporterExt,
    SkSLIntrinsicList::IntrinsicKind,
    SkSLOperator::{Operator, OperatorKind},
};
// Source: src/sksl/ir/SkSLFunctionCall.h:31-31
// Source: src/sksl/ir/SkSLFunctionCall.h:89-94
#[repr(C)]
pub struct FunctionCall {
    base: ExpressionData,
    fFunction: NonNull<FunctionDeclaration>,
    fArguments: Pin<Box<ExpressionArray>>,
    fStablePointer: Option<NonNull<FunctionCall>>,
}
// Source: src/sksl/ir/SkSLFunctionCall.h:33-33
impl IRNodeType for FunctionCall {
    const kIRNodeKind: i32 = ExpressionKind::kFunctionCall as i32;
}
impl FunctionCall {
    // Source: src/sksl/ir/SkSLFunctionCall.h:35-43
    // Placement is required: the native stable pointer is the final pool address, never a stack temporary.
    pub unsafe fn new(
        pos: Position,
        ty: NonNull<dyn Type>,
        function: NonNull<FunctionDeclaration>,
        arguments: Pin<Box<ExpressionArray>>,
        stablePointer: Option<NonNull<Self>>,
    ) -> IRNodeBox<Self> {
        IRNodeBox::new_with(|p: *mut Self| {
            p.write(Self {
                base: ExpressionData::new(pos, ExpressionKind::kFunctionCall, Some(ty)),
                fFunction: function,
                fArguments: arguments,
                fStablePointer: Some(stablePointer.unwrap_or_else(|| NonNull::new_unchecked(p))),
            })
        })
    }
    // Source: src/sksl/ir/SkSLFunctionCall.h:68-70
    pub unsafe fn function(&self) -> &FunctionDeclaration {
        self.fFunction.as_ref()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.h:72-78
    pub fn arguments_mut(&mut self) -> Pin<&mut ExpressionArray> {
        self.fArguments.as_mut()
    }
    pub fn arguments(&self) -> &ExpressionArray {
        &self.fArguments
    }
    // Source: src/sksl/ir/SkSLFunctionCall.h:80-82
    pub fn stablePointer(&self) -> Option<NonNull<Self>> {
        self.fStablePointer
    }
}
impl IRNode for FunctionCall {
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
unsafe impl Expression for FunctionCall {
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
// Source: src/sksl/ir/SkSLFunctionCall.cpp:53-53
type IntrinsicArguments<'a> = [Option<&'a dyn Expression>; 3];
// Source: src/sksl/ir/SkSLFunctionCall.cpp:55-63
unsafe fn has_compile_time_constant_arguments(arguments: &ExpressionArray) -> bool {
    for arg in arguments.as_base().as_slice() {
        let expr = ConstantFolder::GetConstantValueForVariable(&**arg.as_ref().unwrap_unchecked());
        if !SkSLAnalysis::IsCompileTimeConstant(expr) {
            return false;
        }
    }
    true
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:64-80
// Actual three native template specializations, no arbitrary generic type implementation.
trait IntrinsicScalar {
    fn type_check_expression(expr: &dyn Expression);
}
impl IntrinsicScalar for f32 {
    fn type_check_expression(expr: &dyn Expression) {
        debug_assert!(expr.expression_type().componentType().isFloat());
    }
}
impl IntrinsicScalar for i64 {
    fn type_check_expression(expr: &dyn Expression) {
        debug_assert!(expr.expression_type().componentType().isInteger());
    }
}
impl IntrinsicScalar for bool {
    fn type_check_expression(expr: &dyn Expression) {
        debug_assert!(expr.expression_type().componentType().isBoolean());
    }
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:83-84
type CoalesceFn = fn(f64, f64, f64) -> f64;
type FinalizeFn = fn(f64) -> f64;
// Source: src/sksl/ir/SkSLFunctionCall.cpp:86-144
unsafe fn coalesce_n_way_vector(
    arg0: &dyn Expression,
    arg1: Option<&dyn Expression>,
    startingState: f64,
    returnType: &dyn Type,
    coalesce: CoalesceFn,
    finalize: Option<FinalizeFn>,
) -> Option<IRNodeBox<dyn Expression>> {
    let pos = arg0.node().fPosition;
    let minimumValue = returnType.componentType().minimumValue();
    let maximumValue = returnType.componentType().maximumValue();
    let vecType = if arg0.expression_type().isVector() {
        arg0.expression_type()
    } else if let Some(arg1) = arg1.filter(|e| e.expression_type().isVector()) {
        arg1.expression_type()
    } else {
        arg0.expression_type()
    };
    crate::SkASSERT!(@source b"src/sksl/ir/SkSLFunctionCall.cpp\0".as_ptr(),111; @condition b"arg0->type().componentType().matches(vecType.componentType())\0".as_ptr(); arg0.expression_type().componentType().matches(vecType.componentType()));
    debug_assert!(
        arg1.is_none()
            || arg1
                .unwrap_unchecked()
                .expression_type()
                .componentType()
                .matches(vecType.componentType())
    );
    let mut value = startingState;
    let mut arg0Index = 0;
    let mut arg1Index = 0;
    for _index in 0..vecType.columns() {
        let arg0Value = arg0.getConstantValue(arg0Index);
        arg0Index += if arg0.expression_type().isVector() {
            1
        } else {
            0
        };
        debug_assert!(arg0Value.is_some());
        let mut arg1Value = Some(0.0);
        if let Some(arg1) = arg1 {
            arg1Value = arg1.getConstantValue(arg1Index);
            arg1Index += if arg1.expression_type().isVector() {
                1
            } else {
                0
            };
            debug_assert!(arg1Value.is_some());
        }
        value = coalesce(
            value,
            arg0Value.unwrap_unchecked(),
            arg1Value.unwrap_unchecked(),
        );
        if value >= minimumValue && value <= maximumValue {
        } else {
            return None;
        }
    }
    if let Some(finalize) = finalize {
        value = finalize(value);
    }
    Some(Literal::Make(pos, value, returnType).cast(|p| p as *mut dyn Expression))
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:147-158
unsafe fn coalesce_vector<T: IntrinsicScalar>(
    arguments: &IntrinsicArguments,
    startingState: f64,
    returnType: &dyn Type,
    coalesce: CoalesceFn,
    finalize: Option<FinalizeFn>,
) -> Option<IRNodeBox<dyn Expression>> {
    debug_assert!(arguments[0].is_some());
    debug_assert!(arguments[1].is_none());
    T::type_check_expression(arguments[0].unwrap_unchecked());
    coalesce_n_way_vector(
        arguments[0].unwrap_unchecked(),
        None,
        startingState,
        returnType,
        coalesce,
        finalize,
    )
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:161-174
unsafe fn coalesce_pairwise_vectors<T: IntrinsicScalar>(
    arguments: &IntrinsicArguments,
    startingState: f64,
    returnType: &dyn Type,
    coalesce: CoalesceFn,
    finalize: Option<FinalizeFn>,
) -> Option<IRNodeBox<dyn Expression>> {
    debug_assert!(arguments[0].is_some());
    debug_assert!(arguments[1].is_some());
    debug_assert!(arguments[2].is_none());
    T::type_check_expression(arguments[0].unwrap_unchecked());
    T::type_check_expression(arguments[1].unwrap_unchecked());
    coalesce_n_way_vector(
        arguments[0].unwrap_unchecked(),
        arguments[1],
        startingState,
        returnType,
        coalesce,
        finalize,
    )
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:176-176
type CompareFn = fn(f64, f64) -> bool;
// Source: src/sksl/ir/SkSLFunctionCall.cpp:178-204
unsafe fn optimize_comparison(
    context: &Context,
    arguments: &IntrinsicArguments,
    compare: CompareFn,
) -> Option<IRNodeBox<dyn Expression>> {
    let left = arguments[0].unwrap_unchecked();
    let right = arguments[1].unwrap_unchecked();
    debug_assert!(arguments[2].is_none());
    let ty = left.expression_type();
    debug_assert!(ty.isVector());
    debug_assert!(ty.componentType().isScalar());
    debug_assert!(ty.matches(right.expression_type()));
    let mut array = [core::mem::MaybeUninit::<f64>::uninit(); 4];
    for index in 0..ty.columns() {
        let leftValue = left.getConstantValue(index);
        let rightValue = right.getConstantValue(index);
        debug_assert!(leftValue.is_some());
        debug_assert!(rightValue.is_some());
        array[index as usize].write(
            if compare(leftValue.unwrap_unchecked(), rightValue.unwrap_unchecked()) {
                1.0
            } else {
                0.0
            },
        );
    }
    let bvecType = context
        .fTypes
        .as_ref()
        .fBool
        .toCompound(context, ty.columns(), 1);
    Some(ConstructorCompound::MakeFromConstants(
        context,
        left.node().fPosition,
        bvecType,
        core::slice::from_raw_parts(array.as_ptr().cast::<f64>(), ty.columns() as usize),
    ))
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:206-206
type EvaluateFn = fn(f64, f64, f64) -> f64;
// Source: src/sksl/ir/SkSLFunctionCall.cpp:208-263
unsafe fn evaluate_n_way_intrinsic(
    context: &Context,
    arg0: &dyn Expression,
    arg1: Option<&dyn Expression>,
    arg2: Option<&dyn Expression>,
    returnType: &dyn Type,
    eval: EvaluateFn,
) -> Option<IRNodeBox<dyn Expression>> {
    let minimumValue = returnType.componentType().minimumValue();
    let maximumValue = returnType.componentType().maximumValue();
    let slots = returnType.slotCount();
    let mut array = [core::mem::MaybeUninit::<f64>::uninit(); 16];
    let mut arg0Index = 0;
    let mut arg1Index = 0;
    let mut arg2Index = 0;
    for index in 0..slots {
        let arg0Value = arg0.getConstantValue(arg0Index);
        arg0Index += if arg0.expression_type().isScalar() {
            0
        } else {
            1
        };
        debug_assert!(arg0Value.is_some());
        let mut arg1Value = Some(0.0);
        if let Some(arg1) = arg1 {
            arg1Value = arg1.getConstantValue(arg1Index);
            arg1Index += if arg1.expression_type().isScalar() {
                0
            } else {
                1
            };
            debug_assert!(arg1Value.is_some());
        }
        let mut arg2Value = Some(0.0);
        if let Some(arg2) = arg2 {
            arg2Value = arg2.getConstantValue(arg2Index);
            arg2Index += if arg2.expression_type().isScalar() {
                0
            } else {
                1
            };
            debug_assert!(arg2Value.is_some());
        }
        let value = eval(
            arg0Value.unwrap_unchecked(),
            arg1Value.unwrap_unchecked(),
            arg2Value.unwrap_unchecked(),
        );
        array[index].write(value);
        if value >= minimumValue && value <= maximumValue {
        } else {
            return None;
        }
    }
    Some(ConstructorCompound::MakeFromConstants(
        context,
        arg0.node().fPosition,
        returnType,
        core::slice::from_raw_parts(array.as_ptr().cast::<f64>(), slots),
    ))
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:266-276
unsafe fn evaluate_intrinsic<T: IntrinsicScalar>(
    context: &Context,
    arguments: &IntrinsicArguments,
    returnType: &dyn Type,
    eval: EvaluateFn,
) -> Option<IRNodeBox<dyn Expression>> {
    debug_assert!(arguments[0].is_some());
    debug_assert!(arguments[1].is_none());
    T::type_check_expression(arguments[0].unwrap_unchecked());
    evaluate_n_way_intrinsic(
        context,
        arguments[0].unwrap_unchecked(),
        None,
        None,
        returnType,
        eval,
    )
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:278-295
unsafe fn evaluate_intrinsic_numeric(
    context: &Context,
    arguments: &IntrinsicArguments,
    returnType: &dyn Type,
    eval: EvaluateFn,
) -> Option<IRNodeBox<dyn Expression>> {
    debug_assert!(arguments[0].is_some());
    debug_assert!(arguments[1].is_none());
    let ty = arguments[0]
        .unwrap_unchecked()
        .expression_type()
        .componentType();
    if ty.isFloat() {
        return evaluate_intrinsic::<f32>(context, arguments, returnType, eval);
    }
    if ty.isInteger() {
        return evaluate_intrinsic::<i64>(context, arguments, returnType, eval);
    }
    #[cfg(sk_debug)]
    {
        let mut description = ty.description();
        description.push(0);
        crate::SkDEBUGFAILF!(@source b"src/sksl/ir/SkSLFunctionCall.cpp\0".as_ptr(),293; "unsupported type %s",description.as_ptr());
    }
    None
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:297-319
unsafe fn evaluate_pairwise_intrinsic(
    context: &Context,
    arguments: &IntrinsicArguments,
    returnType: &dyn Type,
    eval: EvaluateFn,
) -> Option<IRNodeBox<dyn Expression>> {
    debug_assert!(arguments[0].is_some());
    debug_assert!(arguments[1].is_some());
    debug_assert!(arguments[2].is_none());
    let ty = arguments[0]
        .unwrap_unchecked()
        .expression_type()
        .componentType();
    if ty.isFloat() {
        <f32 as IntrinsicScalar>::type_check_expression(arguments[0].unwrap_unchecked());
        <f32 as IntrinsicScalar>::type_check_expression(arguments[1].unwrap_unchecked());
    } else if ty.isInteger() {
        <i64 as IntrinsicScalar>::type_check_expression(arguments[0].unwrap_unchecked());
        <i64 as IntrinsicScalar>::type_check_expression(arguments[1].unwrap_unchecked());
    } else {
        #[cfg(sk_debug)]
        {
            let mut description = ty.description();
            description.push(0);
            crate::SkDEBUGFAILF!(@source b"src/sksl/ir/SkSLFunctionCall.cpp\0".as_ptr(),313; "unsupported type %s",description.as_ptr());
        }
        return None;
    }
    evaluate_n_way_intrinsic(
        context,
        arguments[0].unwrap_unchecked(),
        arguments[1],
        None,
        returnType,
        eval,
    )
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:321-345
unsafe fn evaluate_3_way_intrinsic(
    context: &Context,
    arguments: &IntrinsicArguments,
    returnType: &dyn Type,
    eval: EvaluateFn,
) -> Option<IRNodeBox<dyn Expression>> {
    debug_assert!(arguments[0].is_some());
    debug_assert!(arguments[1].is_some());
    debug_assert!(arguments[2].is_some());
    let ty = arguments[0]
        .unwrap_unchecked()
        .expression_type()
        .componentType();
    if ty.isFloat() {
        <f32 as IntrinsicScalar>::type_check_expression(arguments[0].unwrap_unchecked());
        <f32 as IntrinsicScalar>::type_check_expression(arguments[1].unwrap_unchecked());
        <f32 as IntrinsicScalar>::type_check_expression(arguments[2].unwrap_unchecked());
    } else if ty.isInteger() {
        <i64 as IntrinsicScalar>::type_check_expression(arguments[0].unwrap_unchecked());
        <i64 as IntrinsicScalar>::type_check_expression(arguments[1].unwrap_unchecked());
        <i64 as IntrinsicScalar>::type_check_expression(arguments[2].unwrap_unchecked());
    } else {
        #[cfg(sk_debug)]
        {
            let mut description = ty.description();
            description.push(0);
            crate::SkDEBUGFAILF!(@source b"src/sksl/ir/SkSLFunctionCall.cpp\0".as_ptr(),339; "unsupported type %s",description.as_ptr());
        }
        return None;
    }
    evaluate_n_way_intrinsic(
        context,
        arguments[0].unwrap_unchecked(),
        arguments[1],
        arguments[2],
        returnType,
        eval,
    )
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:348-357
// Native pun_value uses equal-width memcpy after numeric conversion. Helpers preserve the
// actual source conversion domain; Rust saturation may not fabricate an out-of-domain value.
unsafe fn native_i32(v: f64) -> i32 {
    if !v.is_finite() || v < -2147483648.0 || v >= 2147483648.0 {
        std::process::abort();
    }
    v as i32
}
unsafe fn native_u32(v: f64) -> u32 {
    if !v.is_finite() || v < 0.0 || v >= 4294967296.0 {
        std::process::abort();
    }
    v as u32
}
mod Intrinsics {
    use super::*;
    unsafe extern "C" {
        fn remainder(x: f64, y: f64) -> f64;
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:363-363
    pub(super) fn scalar_coalesce_length(a: f64, b: f64, c: f64) -> f64 {
        a + (b * b)
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:364-364
    pub(super) fn scalar_finalize_length(a: f64) -> f64 {
        a.sqrt()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:366-366
    pub(super) fn scalar_coalesce_distance(a: f64, mut b: f64, c: f64) -> f64 {
        {
            b -= c;
            a + (b * b)
        }
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:367-367
    pub(super) fn scalar_finalize_distance(a: f64) -> f64 {
        a.sqrt()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:369-369
    pub(super) fn scalar_coalesce_dot(a: f64, b: f64, c: f64) -> f64 {
        a + (b * c)
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:370-370
    pub(super) fn scalar_coalesce_any(a: f64, b: f64, c: f64) -> f64 {
        ((a != 0.0) || (b != 0.0)) as u8 as f64
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:371-371
    pub(super) fn scalar_coalesce_all(a: f64, b: f64, c: f64) -> f64 {
        ((a != 0.0) && (b != 0.0)) as u8 as f64
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:373-373
    pub(super) fn scalar_compare_lessThan(a: f64, b: f64) -> bool {
        a < b
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:374-374
    pub(super) fn scalar_compare_lessThanEqual(a: f64, b: f64) -> bool {
        a <= b
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:375-375
    pub(super) fn scalar_compare_greaterThan(a: f64, b: f64) -> bool {
        a > b
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:376-376
    pub(super) fn scalar_compare_greaterThanEqual(a: f64, b: f64) -> bool {
        a >= b
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:377-377
    pub(super) fn scalar_compare_equal(a: f64, b: f64) -> bool {
        a == b
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:378-378
    pub(super) fn scalar_compare_notEqual(a: f64, b: f64) -> bool {
        a != b
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:380-380
    pub(super) fn scalar_evaluate_radians(a: f64, b: f64, c: f64) -> f64 {
        a * 0.0174532925
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:381-381
    pub(super) fn scalar_evaluate_degrees(a: f64, b: f64, c: f64) -> f64 {
        a * 57.2957795
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:382-382
    pub(super) fn scalar_evaluate_sin(a: f64, b: f64, c: f64) -> f64 {
        a.sin()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:383-383
    pub(super) fn scalar_evaluate_cos(a: f64, b: f64, c: f64) -> f64 {
        a.cos()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:384-384
    pub(super) fn scalar_evaluate_tan(a: f64, b: f64, c: f64) -> f64 {
        a.tan()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:385-385
    pub(super) fn scalar_evaluate_asin(a: f64, b: f64, c: f64) -> f64 {
        a.asin()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:386-386
    pub(super) fn scalar_evaluate_acos(a: f64, b: f64, c: f64) -> f64 {
        a.acos()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:387-387
    pub(super) fn scalar_evaluate_atan(a: f64, b: f64, c: f64) -> f64 {
        a.atan()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:388-388
    pub(super) fn scalar_evaluate_atan2(a: f64, b: f64, c: f64) -> f64 {
        a.atan2(b)
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:389-389
    pub(super) fn scalar_evaluate_asinh(a: f64, b: f64, c: f64) -> f64 {
        a.asinh()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:390-390
    pub(super) fn scalar_evaluate_acosh(a: f64, b: f64, c: f64) -> f64 {
        a.acosh()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:391-391
    pub(super) fn scalar_evaluate_atanh(a: f64, b: f64, c: f64) -> f64 {
        a.atanh()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:393-393
    pub(super) fn scalar_evaluate_pow(a: f64, b: f64, c: f64) -> f64 {
        a.powf(b)
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:394-394
    pub(super) fn scalar_evaluate_exp(a: f64, b: f64, c: f64) -> f64 {
        a.exp()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:395-395
    pub(super) fn scalar_evaluate_log(a: f64, b: f64, c: f64) -> f64 {
        a.ln()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:396-396
    pub(super) fn scalar_evaluate_exp2(a: f64, b: f64, c: f64) -> f64 {
        a.exp2()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:397-397
    pub(super) fn scalar_evaluate_log2(a: f64, b: f64, c: f64) -> f64 {
        a.log2()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:398-398
    pub(super) fn scalar_evaluate_sqrt(a: f64, b: f64, c: f64) -> f64 {
        a.sqrt()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:399-401
    pub(super) fn scalar_evaluate_inversesqrt(a: f64, b: f64, c: f64) -> f64 {
        1.0 / a.sqrt()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:403-403
    pub(super) fn scalar_evaluate_add(a: f64, b: f64, c: f64) -> f64 {
        a + b
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:404-404
    pub(super) fn scalar_evaluate_sub(a: f64, b: f64, c: f64) -> f64 {
        a - b
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:405-405
    pub(super) fn scalar_evaluate_mul(a: f64, b: f64, c: f64) -> f64 {
        a * b
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:406-406
    pub(super) fn scalar_evaluate_div(a: f64, b: f64, c: f64) -> f64 {
        a / b
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:407-407
    pub(super) fn scalar_evaluate_abs(a: f64, b: f64, c: f64) -> f64 {
        a.abs()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:408-408
    pub(super) fn scalar_evaluate_sign(a: f64, b: f64, c: f64) -> f64 {
        ((a > 0.0) as i32 - (a < 0.0) as i32) as f64
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:409-409
    pub(super) fn scalar_evaluate_opposite_sign(a: f64, b: f64, c: f64) -> f64 {
        ((a < 0.0) as i32 - (a > 0.0) as i32) as f64
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:410-410
    pub(super) fn scalar_evaluate_floor(a: f64, b: f64, c: f64) -> f64 {
        a.floor()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:411-411
    pub(super) fn scalar_evaluate_ceil(a: f64, b: f64, c: f64) -> f64 {
        a.ceil()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:412-412
    pub(super) fn scalar_evaluate_fract(a: f64, b: f64, c: f64) -> f64 {
        a - a.floor()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:413-413
    pub(super) fn scalar_evaluate_min(a: f64, b: f64, c: f64) -> f64 {
        if a < b {
            a
        } else {
            b
        }
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:414-414
    pub(super) fn scalar_evaluate_max(a: f64, b: f64, c: f64) -> f64 {
        if a > b {
            a
        } else {
            b
        }
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:415-415
    pub(super) fn scalar_evaluate_clamp(a: f64, b: f64, c: f64) -> f64 {
        if a < b {
            b
        } else if a > c {
            c
        } else {
            a
        }
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:416-416
    pub(super) fn scalar_evaluate_fma(a: f64, b: f64, c: f64) -> f64 {
        a * b + c
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:417-417
    pub(super) fn scalar_evaluate_saturate(a: f64, b: f64, c: f64) -> f64 {
        if a < 0.0 {
            0.0
        } else if a > 1.0 {
            1.0
        } else {
            a
        }
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:418-418
    pub(super) fn scalar_evaluate_mix(a: f64, b: f64, c: f64) -> f64 {
        a * (1.0 - c) + b * c
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:419-419
    pub(super) fn scalar_evaluate_step(a: f64, b: f64, c: f64) -> f64 {
        if b < a {
            0.0
        } else {
            1.0
        }
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:420-422
    pub(super) fn scalar_evaluate_mod(a: f64, b: f64, c: f64) -> f64 {
        a - b * (a / b).floor()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:423-427
    pub(super) fn scalar_evaluate_smoothstep(a: f64, b: f64, c: f64) -> f64 {
        {
            let mut t = (c - a) / (b - a);
            t = if t < 0.0 {
                0.0
            } else if t > 1.0 {
                1.0
            } else {
                t
            };
            t * t * (3.0 - 2.0 * t)
        }
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:429-429
    pub(super) fn scalar_evaluate_matrixCompMult(a: f64, b: f64, c: f64) -> f64 {
        a * b
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:431-431
    pub(super) fn scalar_evaluate_not(a: f64, b: f64, c: f64) -> f64 {
        (a == 0.0) as u8 as f64
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:432-432
    pub(super) fn scalar_evaluate_sinh(a: f64, b: f64, c: f64) -> f64 {
        a.sinh()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:433-433
    pub(super) fn scalar_evaluate_cosh(a: f64, b: f64, c: f64) -> f64 {
        a.cosh()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:434-434
    pub(super) fn scalar_evaluate_tanh(a: f64, b: f64, c: f64) -> f64 {
        a.tanh()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:435-435
    pub(super) fn scalar_evaluate_trunc(a: f64, b: f64, c: f64) -> f64 {
        a.trunc()
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:436-440
    pub(super) fn scalar_evaluate_round(a: f64, b: f64, c: f64) -> f64 {
        a - unsafe { remainder(a, 1.0) }
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:441-441
    pub(super) fn scalar_evaluate_floatBitsToInt(a: f64, b: f64, c: f64) -> f64 {
        (a as f32).to_bits() as i32 as f64
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:442-442
    pub(super) fn scalar_evaluate_floatBitsToUint(a: f64, b: f64, c: f64) -> f64 {
        (a as f32).to_bits() as f64
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:443-443
    pub(super) fn scalar_evaluate_intBitsToFloat(a: f64, b: f64, c: f64) -> f64 {
        f32::from_bits(unsafe { native_i32(a) } as u32) as f64
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:444-444
    pub(super) fn scalar_evaluate_uintBitsToFloat(a: f64, b: f64, c: f64) -> f64 {
        f32::from_bits(unsafe { native_u32(a) }) as f64
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:446-451
    pub(super) unsafe fn evaluate_length(
        arguments: &IntrinsicArguments,
    ) -> Option<IRNodeBox<dyn Expression>> {
        coalesce_vector::<f32>(
            arguments,
            0.0,
            arguments[0]
                .unwrap_unchecked()
                .expression_type()
                .componentType(),
            scalar_coalesce_length,
            Some(scalar_finalize_length),
        )
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:453-458
    pub(super) unsafe fn evaluate_distance(
        arguments: &IntrinsicArguments,
    ) -> Option<IRNodeBox<dyn Expression>> {
        coalesce_pairwise_vectors::<f32>(
            arguments,
            0.0,
            arguments[0]
                .unwrap_unchecked()
                .expression_type()
                .componentType(),
            scalar_coalesce_distance,
            Some(scalar_finalize_distance),
        )
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:459-464
    pub(super) unsafe fn evaluate_dot(
        arguments: &IntrinsicArguments,
    ) -> Option<IRNodeBox<dyn Expression>> {
        coalesce_pairwise_vectors::<f32>(
            arguments,
            0.0,
            arguments[0]
                .unwrap_unchecked()
                .expression_type()
                .componentType(),
            scalar_coalesce_dot,
            None,
        )
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:466-470
    pub(super) unsafe fn evaluate_sign(
        context: &Context,
        arguments: &IntrinsicArguments,
    ) -> Option<IRNodeBox<dyn Expression>> {
        evaluate_intrinsic_numeric(
            context,
            arguments,
            arguments[0].unwrap_unchecked().expression_type(),
            scalar_evaluate_sign,
        )
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:472-476
    pub(super) unsafe fn evaluate_opposite_sign(
        context: &Context,
        arguments: &IntrinsicArguments,
    ) -> Option<IRNodeBox<dyn Expression>> {
        evaluate_intrinsic_numeric(
            context,
            arguments,
            arguments[0].unwrap_unchecked().expression_type(),
            scalar_evaluate_opposite_sign,
        )
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:478-482
    pub(super) unsafe fn evaluate_add(
        context: &Context,
        arguments: &IntrinsicArguments,
    ) -> Option<IRNodeBox<dyn Expression>> {
        evaluate_pairwise_intrinsic(
            context,
            arguments,
            arguments[0].unwrap_unchecked().expression_type(),
            scalar_evaluate_add,
        )
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:484-488
    pub(super) unsafe fn evaluate_sub(
        context: &Context,
        arguments: &IntrinsicArguments,
    ) -> Option<IRNodeBox<dyn Expression>> {
        evaluate_pairwise_intrinsic(
            context,
            arguments,
            arguments[0].unwrap_unchecked().expression_type(),
            scalar_evaluate_sub,
        )
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:490-494
    pub(super) unsafe fn evaluate_mul(
        context: &Context,
        arguments: &IntrinsicArguments,
    ) -> Option<IRNodeBox<dyn Expression>> {
        evaluate_pairwise_intrinsic(
            context,
            arguments,
            arguments[0].unwrap_unchecked().expression_type(),
            scalar_evaluate_mul,
        )
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:496-500
    pub(super) unsafe fn evaluate_div(
        context: &Context,
        arguments: &IntrinsicArguments,
    ) -> Option<IRNodeBox<dyn Expression>> {
        evaluate_pairwise_intrinsic(
            context,
            arguments,
            arguments[0].unwrap_unchecked().expression_type(),
            scalar_evaluate_div,
        )
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:502-510
    pub(super) unsafe fn evaluate_normalize(
        context: &Context,
        arguments: &IntrinsicArguments,
    ) -> Option<IRNodeBox<dyn Expression>> {
        let length = evaluate_length(arguments)?;
        let divArgs: IntrinsicArguments = [arguments[0], Some(&*length), None];
        evaluate_div(context, &divArgs)
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:512-529
    pub(super) unsafe fn evaluate_faceforward(
        context: &Context,
        arguments: &IntrinsicArguments,
    ) -> Option<IRNodeBox<dyn Expression>> {
        let n = arguments[0];
        let i = arguments[1];
        let nRef = arguments[2];
        let dotArgs = [i, nRef, None];
        let dotExpr = evaluate_dot(&dotArgs)?;
        let signArgs: IntrinsicArguments = [Some(&*dotExpr), None, None];
        let signExpr = evaluate_opposite_sign(context, &signArgs)?;
        let mulArgs: IntrinsicArguments = [n, Some(&*signExpr), None];
        evaluate_mul(context, &mulArgs)
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:531-551
    pub(super) unsafe fn evaluate_reflect(
        context: &Context,
        arguments: &IntrinsicArguments,
    ) -> Option<IRNodeBox<dyn Expression>> {
        let i = arguments[0];
        let n = arguments[1];
        let dotArgs = [n, i, None];
        let dotExpr = evaluate_dot(&dotArgs)?;
        let mulArgs: IntrinsicArguments = [n, Some(&*dotExpr), None];
        let mulExpr = evaluate_mul(context, &mulArgs)?;
        let addArgs: IntrinsicArguments = [Some(&*mulExpr), Some(&*mulExpr), None];
        let addExpr = evaluate_add(context, &addArgs)?;
        let subArgs: IntrinsicArguments = [i, Some(&*addExpr), None];
        evaluate_sub(context, &subArgs)
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:553-625
    pub(super) unsafe fn evaluate_refract(
        context: &Context,
        arguments: &IntrinsicArguments,
    ) -> Option<IRNodeBox<dyn Expression>> {
        let i = arguments[0].unwrap_unchecked();
        let n = arguments[1].unwrap_unchecked();
        let eta = arguments[2].unwrap_unchecked();
        let dotNIArgs: IntrinsicArguments = [Some(n), Some(i), None];
        let dotNIExpr = evaluate_dot(&dotNIArgs)?;
        let dotNI2Args: IntrinsicArguments = [Some(&*dotNIExpr), Some(&*dotNIExpr), None];
        let dotNI2Expr = evaluate_mul(context, &dotNI2Args)?;
        let oneLiteral = Literal::new(Position::new(), 1.0, dotNI2Expr.expression_type());
        let oneMinusDotArgs: IntrinsicArguments = [Some(&oneLiteral), Some(&*dotNI2Expr), None];
        let oneMinusDotExpr = evaluate_sub(context, &oneMinusDotArgs)?;
        let eta2Args: IntrinsicArguments = [Some(eta), Some(eta), None];
        let eta2Expr = evaluate_mul(context, &eta2Args)?;
        let eta2xDotArgs: IntrinsicArguments = [Some(&*eta2Expr), Some(&*oneMinusDotExpr), None];
        let eta2xDotExpr = evaluate_mul(context, &eta2xDotArgs)?;
        let kArgs: IntrinsicArguments = [Some(&oneLiteral), Some(&*eta2xDotExpr), None];
        let kExpr = evaluate_sub(context, &kArgs)?;
        if !kExpr.node().is::<Literal>() {
            return None;
        }
        let kValue = kExpr.node().as_node::<Literal>().value();
        if kValue < 0.0 {
            let zero = [0.0; 4];
            return Some(ConstructorCompound::MakeFromConstants(
                context,
                i.node().fPosition,
                i.expression_type(),
                &zero,
            ));
        }
        let etaDotArgs: IntrinsicArguments = [Some(eta), Some(&*dotNIExpr), None];
        let etaDotExpr = evaluate_mul(context, &etaDotArgs)?;
        let sqrtKLiteral = Literal::new(Position::new(), kValue.sqrt(), eta.expression_type());
        let etaDotSqrtArgs: IntrinsicArguments = [Some(&*etaDotExpr), Some(&sqrtKLiteral), None];
        let etaDotSqrtExpr = evaluate_add(context, &etaDotSqrtArgs)?;
        let nxEDSArgs: IntrinsicArguments = [Some(n), Some(&*etaDotSqrtExpr), None];
        let nxEDSExpr = evaluate_mul(context, &nxEDSArgs)?;
        let iEtaArgs: IntrinsicArguments = [Some(i), Some(eta), None];
        let iEtaExpr = evaluate_mul(context, &iEtaArgs)?;
        let refractArgs: IntrinsicArguments = [Some(&*iEtaExpr), Some(&*nxEDSExpr), None];
        evaluate_sub(context, &refractArgs)
    }
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:630-635
unsafe fn extract_matrix(expr: &dyn Expression, mat: *mut f32) {
    let numSlots = expr.expression_type().slotCount();
    for index in 0..numSlots {
        mat.add(index)
            .write(expr.getConstantValue(index as i32).unwrap_unchecked() as f32);
    }
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:637-1005
unsafe fn optimize_intrinsic_call(
    context: &Context,
    _pos: Position,
    intrinsic: IntrinsicKind,
    argArray: &ExpressionArray,
    returnType: &dyn Type,
) -> Option<IRNodeBox<dyn Expression>> {
    let mut arguments: IntrinsicArguments = [None; 3];
    debug_assert!(argArray.as_base().size() as usize <= arguments.len());
    for index in 0..argArray.as_base().size() as usize {
        arguments[index] = Some(ConstantFolder::GetConstantValueForVariable(
            &**argArray.as_base().as_slice()[index]
                .as_ref()
                .unwrap_unchecked(),
        ));
    }
    let get = |idx: usize, col: i32| {
        arguments[idx]
            .unwrap_unchecked()
            .getConstantValue(col)
            .unwrap_unchecked() as f32
    };
    match intrinsic {
        IntrinsicKind::k_radians_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_radians,
        ),
        IntrinsicKind::k_degrees_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_degrees,
        ),
        IntrinsicKind::k_sin_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_sin,
        ),
        IntrinsicKind::k_cos_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_cos,
        ),
        IntrinsicKind::k_tan_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_tan,
        ),
        IntrinsicKind::k_sinh_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_sinh,
        ),
        IntrinsicKind::k_cosh_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_cosh,
        ),
        IntrinsicKind::k_tanh_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_tanh,
        ),
        IntrinsicKind::k_asin_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_asin,
        ),
        IntrinsicKind::k_acos_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_acos,
        ),
        IntrinsicKind::k_atan_IntrinsicKind => {
            if argArray.as_base().size() == 1 {
                evaluate_intrinsic::<f32>(
                    context,
                    &arguments,
                    returnType,
                    Intrinsics::scalar_evaluate_atan,
                )
            } else {
                evaluate_pairwise_intrinsic(
                    context,
                    &arguments,
                    returnType,
                    Intrinsics::scalar_evaluate_atan2,
                )
            }
        }
        IntrinsicKind::k_asinh_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_asinh,
        ),
        IntrinsicKind::k_acosh_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_acosh,
        ),
        IntrinsicKind::k_atanh_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_atanh,
        ),
        IntrinsicKind::k_pow_IntrinsicKind => evaluate_pairwise_intrinsic(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_pow,
        ),
        IntrinsicKind::k_exp_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_exp,
        ),
        IntrinsicKind::k_log_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_log,
        ),
        IntrinsicKind::k_exp2_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_exp2,
        ),
        IntrinsicKind::k_log2_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_log2,
        ),
        IntrinsicKind::k_sqrt_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_sqrt,
        ),
        IntrinsicKind::k_inversesqrt_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_inversesqrt,
        ),
        IntrinsicKind::k_abs_IntrinsicKind => evaluate_intrinsic_numeric(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_abs,
        ),
        IntrinsicKind::k_sign_IntrinsicKind => Intrinsics::evaluate_sign(context, &arguments),
        IntrinsicKind::k_floor_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_floor,
        ),
        IntrinsicKind::k_ceil_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_ceil,
        ),
        IntrinsicKind::k_fract_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_fract,
        ),
        IntrinsicKind::k_mod_IntrinsicKind => evaluate_pairwise_intrinsic(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_mod,
        ),
        IntrinsicKind::k_min_IntrinsicKind => evaluate_pairwise_intrinsic(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_min,
        ),
        IntrinsicKind::k_max_IntrinsicKind => evaluate_pairwise_intrinsic(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_max,
        ),
        IntrinsicKind::k_clamp_IntrinsicKind => evaluate_3_way_intrinsic(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_clamp,
        ),
        IntrinsicKind::k_fma_IntrinsicKind => evaluate_3_way_intrinsic(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_fma,
        ),
        IntrinsicKind::k_saturate_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_saturate,
        ),
        IntrinsicKind::k_mix_IntrinsicKind => {
            if arguments[2]
                .unwrap_unchecked()
                .expression_type()
                .componentType()
                .isBoolean()
            {
                let numericType = arguments[0]
                    .unwrap_unchecked()
                    .expression_type()
                    .componentType();
                if numericType.isFloat() {
                    <f32 as IntrinsicScalar>::type_check_expression(
                        arguments[0].unwrap_unchecked(),
                    );
                    <f32 as IntrinsicScalar>::type_check_expression(
                        arguments[1].unwrap_unchecked(),
                    );
                } else if numericType.isInteger() {
                    <i64 as IntrinsicScalar>::type_check_expression(
                        arguments[0].unwrap_unchecked(),
                    );
                    <i64 as IntrinsicScalar>::type_check_expression(
                        arguments[1].unwrap_unchecked(),
                    );
                } else if numericType.isBoolean() {
                    <bool as IntrinsicScalar>::type_check_expression(
                        arguments[0].unwrap_unchecked(),
                    );
                    <bool as IntrinsicScalar>::type_check_expression(
                        arguments[1].unwrap_unchecked(),
                    );
                } else {
                    #[cfg(sk_debug)]
                    {
                        let mut description = numericType.description();
                        description.push(0);
                        crate::SkDEBUGFAILF!(@source b"src/sksl/ir/SkSLFunctionCall.cpp\0".as_ptr(),773; "unsupported type %s",description.as_ptr());
                    }
                    return None;
                }
                evaluate_n_way_intrinsic(
                    context,
                    arguments[0].unwrap_unchecked(),
                    arguments[1],
                    arguments[2],
                    returnType,
                    Intrinsics::scalar_evaluate_mix,
                )
            } else {
                evaluate_3_way_intrinsic(
                    context,
                    &arguments,
                    returnType,
                    Intrinsics::scalar_evaluate_mix,
                )
            }
        }
        IntrinsicKind::k_step_IntrinsicKind => evaluate_pairwise_intrinsic(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_step,
        ),
        IntrinsicKind::k_smoothstep_IntrinsicKind => evaluate_3_way_intrinsic(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_smoothstep,
        ),
        IntrinsicKind::k_trunc_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_trunc,
        ),
        IntrinsicKind::k_round_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_round,
        ),
        IntrinsicKind::k_roundEven_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_round,
        ),
        IntrinsicKind::k_floatBitsToInt_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_floatBitsToInt,
        ),
        IntrinsicKind::k_floatBitsToUint_IntrinsicKind => evaluate_intrinsic::<f32>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_floatBitsToUint,
        ),
        IntrinsicKind::k_intBitsToFloat_IntrinsicKind => evaluate_intrinsic::<i64>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_intBitsToFloat,
        ),
        IntrinsicKind::k_uintBitsToFloat_IntrinsicKind => evaluate_intrinsic::<i64>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_uintBitsToFloat,
        ),
        IntrinsicKind::k_packUnorm2x16_IntrinsicKind => {
            let pack = |n: i32| -> u32 {
                {
                    let x = get(0, n);
                    let value =
                        (Intrinsics::scalar_evaluate_clamp(x as f64, 0.0, 1.0) * 65535.0).round();
                    native_i32(value) as u32
                }
            };
            let packed = ((pack(0) << 0) & 0x0000ffff) | ((pack(1) << 16) & 0xffff0000);
            Some(ConstructorCompound::MakeFromConstants(
                context,
                arguments[0].unwrap_unchecked().node().fPosition,
                &**context.fTypes.as_ref().fUInt,
                &[packed as f64],
            ))
        }
        IntrinsicKind::k_packSnorm2x16_IntrinsicKind => {
            let pack = |n: i32| -> u32 {
                {
                    let x = get(0, n);
                    let value =
                        (Intrinsics::scalar_evaluate_clamp(x as f64, -1.0, 1.0) * 32767.0).round();
                    native_i32(value) as u32
                }
            };
            let packed = ((pack(0) << 0) & 0x0000ffff) | ((pack(1) << 16) & 0xffff0000);
            Some(ConstructorCompound::MakeFromConstants(
                context,
                arguments[0].unwrap_unchecked().node().fPosition,
                &**context.fTypes.as_ref().fUInt,
                &[packed as f64],
            ))
        }
        IntrinsicKind::k_packHalf2x16_IntrinsicKind => {
            let pack =
                |n: i32| -> u32 { crate::src::core::SkHalf::SkFloatToHalf(get(0, n)) as u32 };
            let packed = ((pack(0) << 0) & 0x0000ffff) | ((pack(1) << 16) & 0xffff0000);
            Some(ConstructorCompound::MakeFromConstants(
                context,
                arguments[0].unwrap_unchecked().node().fPosition,
                &**context.fTypes.as_ref().fUInt,
                &[packed as f64],
            ))
        }
        IntrinsicKind::k_unpackUnorm2x16_IntrinsicKind => {
            let value = arguments[0]
                .unwrap_unchecked()
                .getConstantValue(0)
                .unwrap_unchecked();
            if !value.is_finite()
                || value < -9223372036854775808.0
                || value >= 9223372036854775808.0
            {
                std::process::abort();
            }
            let x = value as i64;
            let a = ((x >> 0) & 0xffff) as u16;
            let b = ((x >> 16) & 0xffff) as u16;
            let unpacked = [a as f64 / 65535.0, b as f64 / 65535.0];
            Some(ConstructorCompound::MakeFromConstants(
                context,
                arguments[0].unwrap_unchecked().node().fPosition,
                &**context.fTypes.as_ref().fFloat2,
                &unpacked,
            ))
        }
        IntrinsicKind::k_unpackSnorm2x16_IntrinsicKind => {
            let value = arguments[0]
                .unwrap_unchecked()
                .getConstantValue(0)
                .unwrap_unchecked();
            if !value.is_finite()
                || value < -9223372036854775808.0
                || value >= 9223372036854775808.0
            {
                std::process::abort();
            }
            let x = value as i64;
            let a = ((x >> 0) & 0xffff) as i16;
            let b = ((x >> 16) & 0xffff) as i16;
            let unpacked = [
                Intrinsics::scalar_evaluate_clamp(a as f64 / 32767.0, -1.0, 1.0),
                Intrinsics::scalar_evaluate_clamp(b as f64 / 32767.0, -1.0, 1.0),
            ];
            Some(ConstructorCompound::MakeFromConstants(
                context,
                arguments[0].unwrap_unchecked().node().fPosition,
                &**context.fTypes.as_ref().fFloat2,
                &unpacked,
            ))
        }
        IntrinsicKind::k_unpackHalf2x16_IntrinsicKind => {
            let value = arguments[0]
                .unwrap_unchecked()
                .getConstantValue(0)
                .unwrap_unchecked();
            if !value.is_finite()
                || value < -9223372036854775808.0
                || value >= 9223372036854775808.0
            {
                std::process::abort();
            }
            let x = value as i64;
            let a = ((x >> 0) & 0xffff) as u16;
            let b = ((x >> 16) & 0xffff) as u16;
            let unpacked = [
                crate::src::core::SkHalf::SkHalfToFloat(a) as f64,
                crate::src::core::SkHalf::SkHalfToFloat(b) as f64,
            ];
            Some(ConstructorCompound::MakeFromConstants(
                context,
                arguments[0].unwrap_unchecked().node().fPosition,
                &**context.fTypes.as_ref().fFloat2,
                &unpacked,
            ))
        }
        IntrinsicKind::k_length_IntrinsicKind => Intrinsics::evaluate_length(&arguments),
        IntrinsicKind::k_distance_IntrinsicKind => Intrinsics::evaluate_distance(&arguments),
        IntrinsicKind::k_dot_IntrinsicKind => Intrinsics::evaluate_dot(&arguments),
        IntrinsicKind::k_cross_IntrinsicKind => {
            let x = |n: i32| get(0, n);
            let y = |n: i32| get(1, n);
            debug_assert!(arguments[0].unwrap_unchecked().expression_type().columns() == 3);
            let vec = [
                (x(1) * y(2) - y(1) * x(2)) as f64,
                (x(2) * y(0) - y(2) * x(0)) as f64,
                (x(0) * y(1) - y(0) * x(1)) as f64,
            ];
            Some(ConstructorCompound::MakeFromConstants(
                context,
                arguments[0].unwrap_unchecked().node().fPosition,
                returnType,
                &vec,
            ))
        }
        IntrinsicKind::k_normalize_IntrinsicKind => {
            Intrinsics::evaluate_normalize(context, &arguments)
        }
        IntrinsicKind::k_faceforward_IntrinsicKind => {
            Intrinsics::evaluate_faceforward(context, &arguments)
        }
        IntrinsicKind::k_reflect_IntrinsicKind => Intrinsics::evaluate_reflect(context, &arguments),
        IntrinsicKind::k_refract_IntrinsicKind => Intrinsics::evaluate_refract(context, &arguments),
        IntrinsicKind::k_matrixCompMult_IntrinsicKind => evaluate_pairwise_intrinsic(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_matrixCompMult,
        ),
        IntrinsicKind::k_transpose_IntrinsicKind => {
            let mut mat = [core::mem::MaybeUninit::<f64>::uninit(); 16];
            let mut index = 0;
            for c in 0..returnType.columns() {
                for rr in 0..returnType.rows() {
                    mat[index].write(get(0, returnType.columns() * rr + c) as f64);
                    index += 1;
                }
            }
            Some(ConstructorCompound::MakeFromConstants(
                context,
                arguments[0].unwrap_unchecked().node().fPosition,
                returnType,
                core::slice::from_raw_parts(mat.as_ptr().cast::<f64>(), index),
            ))
        }
        IntrinsicKind::k_outerProduct_IntrinsicKind => {
            let mut mat = [core::mem::MaybeUninit::<f64>::uninit(); 16];
            let mut index = 0;
            for c in 0..returnType.columns() {
                for rr in 0..returnType.rows() {
                    mat[index].write((get(0, rr) * get(1, c)) as f64);
                    index += 1;
                }
            }
            Some(ConstructorCompound::MakeFromConstants(
                context,
                arguments[0].unwrap_unchecked().node().fPosition,
                returnType,
                core::slice::from_raw_parts(mat.as_ptr().cast::<f64>(), index),
            ))
        }
        IntrinsicKind::k_determinant_IntrinsicKind => {
            let mut mat = [core::mem::MaybeUninit::<f32>::uninit(); 16];
            extract_matrix(arguments[0].unwrap_unchecked(), mat.as_mut_ptr().cast());
            let determinant = match arguments[0]
                .unwrap_unchecked()
                .expression_type()
                .slotCount()
            {
                4 => crate::src::core::SkMatrixInvert::SkInvert2x2Matrix(
                    &*mat.as_ptr().cast::<[f32; 4]>(),
                    None,
                ),
                9 => crate::src::core::SkMatrixInvert::SkInvert3x3Matrix(
                    &*mat.as_ptr().cast::<[f32; 9]>(),
                    None,
                ),
                16 => crate::src::core::SkMatrixInvert::SkInvert4x4Matrix(
                    &*mat.as_ptr().cast::<[f32; 16]>(),
                    None,
                ),
                _ => {
                    #[cfg(sk_debug)]
                    {
                        let mut description = arguments[0]
                            .unwrap_unchecked()
                            .expression_type()
                            .description();
                        description.push(0);
                        crate::SkDEBUGFAILF!(@source b"src/sksl/ir/SkSLFunctionCall.cpp\0".as_ptr(),938; "unsupported type %s",description.as_ptr());
                    }
                    return None;
                }
            };
            Some(
                Literal::MakeFloat(
                    arguments[0].unwrap_unchecked().node().fPosition,
                    determinant,
                    returnType,
                )
                .cast(|p| p as *mut dyn Expression),
            )
        }
        IntrinsicKind::k_inverse_IntrinsicKind => {
            let mut mat = [0.0f32; 16];
            extract_matrix(arguments[0].unwrap_unchecked(), mat.as_mut_ptr());
            let input = mat;
            match arguments[0]
                .unwrap_unchecked()
                .expression_type()
                .slotCount()
            {
                4 => {
                    if crate::src::core::SkMatrixInvert::SkInvert2x2Matrix(
                        &*input.as_ptr().cast::<[f32; 4]>(),
                        Some(&mut *mat.as_mut_ptr().cast::<[f32; 4]>()),
                    ) == 0.0
                    {
                        return None;
                    }
                }
                9 => {
                    if crate::src::core::SkMatrixInvert::SkInvert3x3Matrix(
                        &*input.as_ptr().cast::<[f32; 9]>(),
                        Some(&mut *mat.as_mut_ptr().cast::<[f32; 9]>()),
                    ) == 0.0
                    {
                        return None;
                    }
                }
                16 => {
                    if crate::src::core::SkMatrixInvert::SkInvert4x4Matrix(&input, Some(&mut mat))
                        == 0.0
                    {
                        return None;
                    }
                }
                _ => {
                    #[cfg(sk_debug)]
                    {
                        let mut description = arguments[0]
                            .unwrap_unchecked()
                            .expression_type()
                            .description();
                        description.push(0);
                        crate::SkDEBUGFAILF!(@source b"src/sksl/ir/SkSLFunctionCall.cpp\0".as_ptr(),963; "unsupported type %s",description.as_ptr());
                    }
                    return None;
                }
            }
            let dmat = mat.map(|v| v as f64);
            Some(ConstructorCompound::MakeFromConstants(
                context,
                arguments[0].unwrap_unchecked().node().fPosition,
                returnType,
                &dmat,
            ))
        }
        IntrinsicKind::k_lessThan_IntrinsicKind => {
            optimize_comparison(context, &arguments, Intrinsics::scalar_compare_lessThan)
        }
        IntrinsicKind::k_lessThanEqual_IntrinsicKind => optimize_comparison(
            context,
            &arguments,
            Intrinsics::scalar_compare_lessThanEqual,
        ),
        IntrinsicKind::k_greaterThan_IntrinsicKind => {
            optimize_comparison(context, &arguments, Intrinsics::scalar_compare_greaterThan)
        }
        IntrinsicKind::k_greaterThanEqual_IntrinsicKind => optimize_comparison(
            context,
            &arguments,
            Intrinsics::scalar_compare_greaterThanEqual,
        ),
        IntrinsicKind::k_equal_IntrinsicKind => {
            optimize_comparison(context, &arguments, Intrinsics::scalar_compare_equal)
        }
        IntrinsicKind::k_notEqual_IntrinsicKind => {
            optimize_comparison(context, &arguments, Intrinsics::scalar_compare_notEqual)
        }
        IntrinsicKind::k_any_IntrinsicKind => coalesce_vector::<bool>(
            &arguments,
            0.0,
            returnType,
            Intrinsics::scalar_coalesce_any,
            None,
        ),
        IntrinsicKind::k_all_IntrinsicKind => coalesce_vector::<bool>(
            &arguments,
            1.0,
            returnType,
            Intrinsics::scalar_coalesce_all,
            None,
        ),
        IntrinsicKind::k_not_IntrinsicKind => evaluate_intrinsic::<bool>(
            context,
            &arguments,
            returnType,
            Intrinsics::scalar_evaluate_not,
        ),
        _ => None,
    }
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:1007-1010
unsafe fn clone_at(this: &FunctionCall, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
    Some(
        FunctionCall::new(
            pos,
            borrow_type(this.expression_type()),
            this.fFunction,
            this.arguments().clone(),
            this.stablePointer(),
        )
        .cast(|p| p as *mut dyn Expression),
    )
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:1012-1021
fn description(this: &FunctionCall, _: OperatorPrecedence) -> Vec<u8> {
    unsafe {
        let mut result = this.function().name().to_vec();
        result.push(b'(');
        let mut separator = crate::src::sksl::SkSLString::Separator();
        for arg in this.arguments().as_base().as_slice() {
            result.extend_from_slice(separator());
            result.extend_from_slice(
                &arg.as_ref()
                    .unwrap_unchecked()
                    .description_with_precedence(OperatorPrecedence::kSequence),
            );
        }
        result.push(b')');
        result
    }
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:1023-1048
unsafe fn argument_and_parameter_flags_match(
    argument: &dyn Expression,
    parameter: &dyn Variable,
) -> bool {
    let paramPixelFormat = parameter.layout().fFlags.0 & LayoutFlag::kAllPixelFormats.0;
    if paramPixelFormat != LayoutFlag::kNone.0 {
        if parameter.symbol_type().isStorageTexture() {
            if !argument.node().is::<VariableReference>() {
                return false;
            }
            let var = argument
                .node()
                .as_node::<VariableReference>()
                .variable()
                .unwrap_unchecked();
            if var.as_ref().layout().fFlags.0 & LayoutFlag::kAllPixelFormats.0 != paramPixelFormat {
                return false;
            }
        }
    }
    true
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:1056-1088
unsafe fn call_cost(
    context: &Context,
    function: &FunctionDeclaration,
    arguments: &ExpressionArray,
) -> CoercionCost {
    if context.fConfig.unwrap_unchecked().as_ref().strictES2Mode()
        && function.modifierFlags().isES3()
    {
        return CoercionCost::Impossible();
    }
    if function.parameters().len() != arguments.as_base().size() as usize {
        return CoercionCost::Impossible();
    }
    let mut types = ParamTypes::new_pinned();
    let mut ignored = None;
    if !function.determineFinalTypes(arguments, types.as_mut(), &mut ignored) {
        return CoercionCost::Impossible();
    }
    for i in 0..arguments.as_base().size() as usize {
        let arg = arguments.as_base().as_slice()[i]
            .as_ref()
            .unwrap_unchecked();
        let param = function.parameters()[i].unwrap_unchecked();
        if !argument_and_parameter_flags_match(&**arg, param.as_ref()) {
            return CoercionCost::Impossible();
        }
    }
    let mut total = CoercionCost::Free();
    for i in 0..arguments.as_base().size() as usize {
        total = total
            + arguments.as_base().as_slice()[i]
                .as_ref()
                .unwrap_unchecked()
                .coercionCost(types.as_slice()[i].unwrap_unchecked().as_ref());
    }
    total
}
impl FunctionCall {
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:1090-1107
    pub unsafe fn FindBestFunctionForCall<'a>(
        context: &Context,
        overloadChain: &'a FunctionDeclaration,
        arguments: &ExpressionArray,
    ) -> Option<&'a FunctionDeclaration> {
        if overloadChain.nextOverload().is_none() {
            return Some(overloadChain);
        }
        let mut bestCost = CoercionCost::Impossible();
        let mut best = None;
        let mut f = Some(overloadChain);
        while let Some(function) = f {
            let cost = call_cost(context, function, arguments);
            if cost <= bestCost {
                bestCost = cost;
                best = Some(function);
            }
            f = function.nextOverload();
        }
        if bestCost.fImpossible {
            None
        } else {
            best
        }
    }
}
// Source: src/sksl/ir/SkSLFunctionCall.cpp:1109-1117
unsafe fn build_argument_type_list(arguments: &[Option<IRNodeBox<dyn Expression>>]) -> Vec<u8> {
    let mut result = b"(".to_vec();
    let mut separator = crate::src::sksl::SkSLString::Separator();
    for arg in arguments {
        result.extend_from_slice(separator());
        result.extend_from_slice(
            &arg.as_ref()
                .unwrap_unchecked()
                .expression_type()
                .displayName(),
        );
    }
    result.push(b')');
    result
}
// True native dependencies remain pending; no local FunctionReference or ChildCall replacement.
use super::{
    SkSLChildCall::ChildCall, SkSLFunctionReference::FunctionReference,
    SkSLMethodReference::MethodReference, SkSLTypeReference::TypeReference,
};
impl FunctionCall {
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:1119-1164
    pub unsafe fn ConvertValue(
        context: &Context,
        pos: Position,
        mut functionValue: IRNodeBox<dyn Expression>,
        mut arguments: Pin<Box<ExpressionArray>>,
    ) -> Option<IRNodeBox<dyn Expression>> {
        match functionValue.kind() {
            super::SkSLExpression::Kind::kTypeReference => super::SkSLConstructor::Convert(
                context,
                pos,
                functionValue.node().as_node::<TypeReference>().value(),
                arguments,
            ),
            super::SkSLExpression::Kind::kFunctionReference => {
                let reference = functionValue.node().as_node::<FunctionReference>();
                let chain = reference.overloadChain().unwrap_unchecked().as_ref();
                if let Some(best) = Self::FindBestFunctionForCall(context, chain, &arguments) {
                    return Self::Convert(context, pos, best, arguments);
                }
                let mut msg = b"no match for ".to_vec();
                msg.extend_from_slice(chain.name());
                msg.extend_from_slice(&build_argument_type_list(arguments.as_base().as_slice()));
                let mut errors = context.fErrors;
                errors.as_mut().error(pos, &msg);
                None
            }
            super::SkSLExpression::Kind::kMethodReference => {
                let reference = &mut *(functionValue.as_ptr() as *mut () as *mut MethodReference);
                arguments
                    .as_mut()
                    .as_base_mut()
                    .push_back(reference.self_mut().take());
                let chain = reference.overloadChain().unwrap_unchecked().as_ref();
                if let Some(best) = Self::FindBestFunctionForCall(context, chain, &arguments) {
                    return Self::Convert(context, pos, best, arguments);
                }
                let mut msg = b"no match for ".to_vec();
                let args = arguments.as_base().as_slice();
                msg.extend_from_slice(
                    &args
                        .last()
                        .unwrap_unchecked()
                        .as_ref()
                        .unwrap_unchecked()
                        .expression_type()
                        .displayName(),
                );
                msg.extend_from_slice(b"::");
                msg.extend_from_slice(&chain.name()[1..]);
                msg.extend_from_slice(&build_argument_type_list(&args[..args.len() - 1]));
                let mut errors = context.fErrors;
                errors.as_mut().error(pos, &msg);
                None
            }
            super::SkSLExpression::Kind::kPoison => {
                functionValue.as_pin_mut().node_mut().setPosition(pos);
                Some(functionValue)
            }
            _ => {
                let mut errors = context.fErrors;
                errors.as_mut().error(pos, b"not a function");
                None
            }
        }
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:1166-1275
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        function: &FunctionDeclaration,
        mut arguments: Pin<Box<ExpressionArray>>,
    ) -> Option<IRNodeBox<dyn Expression>> {
        if context.fConfig.unwrap_unchecked().as_ref().strictES2Mode()
            && function.modifierFlags().isES3()
        {
            let mut msg = b"call to '".to_vec();
            msg.extend_from_slice(&function.description());
            msg.extend_from_slice(b"' is not supported");
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &msg);
            return None;
        }
        if function.parameters().len() != arguments.as_base().size() as usize {
            let mut msg = b"call to '".to_vec();
            msg.extend_from_slice(function.name());
            msg.extend_from_slice(b"' expected ");
            msg.extend_from_slice(function.parameters().len().to_string().as_bytes());
            msg.extend_from_slice(b" argument");
            if function.parameters().len() != 1 {
                msg.push(b's');
            }
            msg.extend_from_slice(b", but found ");
            msg.extend_from_slice(arguments.as_base().size().to_string().as_bytes());
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &msg);
            return None;
        }
        for i in 0..arguments.as_base().size() as usize {
            let arg = arguments.as_base().as_slice()[i]
                .as_ref()
                .unwrap_unchecked();
            let param = function.parameters()[i].unwrap_unchecked();
            let param = param.as_ref();
            if !argument_and_parameter_flags_match(&**arg, param) {
                let mut msg = b"expected argument of type '".to_vec();
                msg.extend_from_slice(&param.layout().paddedDescription());
                msg.extend_from_slice(&param.modifierFlags().paddedDescription());
                msg.extend_from_slice(&param.symbol_type().description());
                msg.push(b'\'');
                let mut errors = context.fErrors;
                errors.as_mut().error(arg.node().fPosition, &msg);
                return None;
            }
        }
        let mut types = ParamTypes::new_pinned();
        let mut returnType = None;
        if !function.determineFinalTypes(&arguments, types.as_mut(), &mut returnType) {
            let mut msg = b"no match for ".to_vec();
            msg.extend_from_slice(function.name());
            msg.extend_from_slice(&build_argument_type_list(arguments.as_base().as_slice()));
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &msg);
            return None;
        }
        for i in 0..arguments.as_base().size() as usize {
            let arg = arguments.as_mut().as_base_mut().as_mut_slice()[i].take();
            let arg = super::SkSLType::coerceExpression(
                types.as_slice()[i].unwrap_unchecked().as_ref(),
                arg,
                context,
            );
            arguments.as_mut().as_base_mut().as_mut_slice()[i] = arg;
            if arguments.as_base().as_slice()[i].is_none() {
                return None;
            }
            let paramFlags = function.parameters()[i]
                .unwrap_unchecked()
                .as_ref()
                .modifierFlags();
            if paramFlags.0 & ModifierFlag::kOut.0 != 0 {
                let refKind = if paramFlags.0 & ModifierFlag::kIn.0 != 0 {
                    VariableRefKind::kReadWrite
                } else {
                    VariableRefKind::kPointer
                };
                if !SkSLAnalysis::UpdateVariableRefKind(
                    arguments.as_base().as_slice()[i]
                        .as_ref()
                        .unwrap_unchecked()
                        .as_ptr() as *mut dyn Expression,
                    refKind,
                    Some(context.fErrors),
                ) {
                    return None;
                }
            }
        }
        if function.isMain() {
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, b"call to 'main' is not allowed");
            return None;
        }
        if function.intrinsicKind() == IntrinsicKind::k_workgroupUniformLoad_IntrinsicKind {
            debug_assert!(arguments.as_base().size() == 1);
            let mut expr = arguments.as_base().as_slice()[0].as_ref().map(|v| &**v);
            let mut rootVar = None;
            while let Some(value) = expr {
                if value.node().is::<VariableReference>() {
                    rootVar = value.node().as_node::<VariableReference>().variable();
                    break;
                }
                if value.node().is::<FieldAccess>() {
                    expr = value
                        .node()
                        .as_node::<FieldAccess>()
                        .base()
                        .as_ref()
                        .map(|v| &**v);
                    continue;
                }
                if value.node().is::<IndexExpression>() {
                    let indexExpr = value.node().as_node::<IndexExpression>();
                    if indexExpr
                        .base()
                        .as_ref()
                        .unwrap_unchecked()
                        .expression_type()
                        .isVector()
                    {
                        break;
                    }
                    expr = indexExpr.base().as_ref().map(|v| &**v);
                    continue;
                }
                break;
            }
            if rootVar.is_none()
                || !rootVar
                    .unwrap_unchecked()
                    .as_ref()
                    .modifierFlags()
                    .isWorkgroup()
            {
                let mut errors = context.fErrors;
                errors.as_mut().error(pos,b"workgroupUniformLoad must be called with a variable expression in workgroup address space");
                return None;
            }
        }
        if function.intrinsicKind() == IntrinsicKind::k_eval_IntrinsicKind {
            let args = arguments.as_base().as_slice();
            let child = args
                .last()
                .unwrap_unchecked()
                .as_ref()
                .unwrap_unchecked()
                .node()
                .as_node::<VariableReference>()
                .variable()
                .unwrap_unchecked();
            arguments.as_mut().as_base_mut().pop_back();
            return Some(ChildCall::Make(
                context,
                pos,
                returnType.unwrap_unchecked(),
                child,
                arguments,
            ));
        }
        Some(Self::Make(
            context,
            pos,
            returnType.unwrap_unchecked(),
            function,
            arguments,
        ))
    }
    // Source: src/sksl/ir/SkSLFunctionCall.cpp:1277-1299
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        returnType: NonNull<dyn Type>,
        function: &FunctionDeclaration,
        arguments: Pin<Box<ExpressionArray>>,
    ) -> IRNodeBox<dyn Expression> {
        debug_assert!(function.parameters().len() == arguments.as_base().size() as usize);
        if function.isIntrinsic() && has_compile_time_constant_arguments(&arguments) {
            if let Some(mut expr) = optimize_intrinsic_call(
                context,
                pos,
                function.intrinsicKind(),
                &arguments,
                returnType.as_ref(),
            ) {
                expr.as_pin_mut().node_mut().setPosition(pos);
                return expr;
            }
        }
        Self::new(pos, returnType, NonNull::from(function), arguments, None)
            .cast(|p| p as *mut dyn Expression)
    }
}
