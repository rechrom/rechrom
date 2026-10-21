// New translation: unverified by explicit user instruction; no compile/test/oracle was run.
use super::{
    SkSLConstructor::*,
    SkSLExpression::{ComparisonResult, Expression, ExpressionData, ExpressionExt},
    SkSLIRNode::*,
    SkSLType::Type,
};
use crate::src::sksl::{
    SkSLContext::Context, SkSLDefines::ExpressionArray, SkSLOperator::OperatorPrecedence,
    SkSLPosition::Position,
};
use core::{pin::Pin, ptr::NonNull};
// Source: src/sksl/ir/SkSLConstructorCompoundCast.h:30-30
// Adapted native inherited subobject only; no extra constructor state.
#[repr(C)]
pub struct ConstructorCompoundCast {
    pub(crate) base: SingleArgumentConstructorData,
}
// Source: src/sksl/ir/SkSLConstructorCompoundCast.h:32-32
impl IRNodeType for ConstructorCompoundCast {
    const kIRNodeKind: i32 = ExpressionKind::kConstructorCompoundCast as i32;
}
impl ConstructorCompoundCast {
    // Source: src/sksl/ir/SkSLConstructorCompoundCast.h:34-35
    /// Safety: Type and child-expression borrowed dependencies retain their native lifetime/pool lease.
    pub unsafe fn new(
        pos: Position,
        ty: &dyn Type,
        arg: Option<IRNodeBox<dyn Expression>>,
    ) -> Self {
        let ty =
            core::mem::transmute::<NonNull<dyn Type + '_>, NonNull<dyn Type>>(NonNull::from(ty));
        Self {
            base: SingleArgumentConstructorData::new(
                pos,
                ExpressionKind::kConstructorCompoundCast,
                ty,
                arg,
            ),
        }
    }
    pub fn argument(&self) -> &Option<IRNodeBox<dyn Expression>> {
        self.base.argument()
    }
    pub fn argument_mut(&mut self) -> &mut Option<IRNodeBox<dyn Expression>> {
        self.base.argument_mut()
    }
}
impl IRNode for ConstructorCompoundCast {
    fn node(&self) -> &IRNodeData {
        &self.base.base.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base.base.base.base) }
    }
    fn description(&self) -> Vec<u8> {
        self.expression_description()
    }
}
unsafe impl Expression for ConstructorCompoundCast {
    fn expression_data(&self) -> &ExpressionData {
        &self.base.base.base
    }
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base.base.base) }
    }
    fn as_expression(&self) -> &dyn Expression {
        self
    }
    unsafe fn constructor_view(&self) -> &dyn AnyConstructor {
        self
    }
    unsafe fn constructor_view_mut(self: Pin<&mut Self>) -> Pin<&mut dyn AnyConstructor> {
        self
    }
    // Source: src/sksl/ir/SkSLConstructorCompoundCast.h:42-44
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        let arg = self.argument().as_ref().unwrap_unchecked().clone_current();
        Some(
            IRNodeBox::new_with(|p: *mut Self| {
                p.write(Self::new(pos, self.expression_type(), arg))
            })
            .cast(|p| p as *mut dyn Expression),
        )
    }
    fn supportsConstantValues(&self) -> bool {
        self.constructor_supportsConstantValues()
    }
    fn getConstantValue(&self, n: i32) -> Option<f64> {
        self.constructor_getConstantValue(n)
    }
    fn compareConstant(&self, other: &dyn Expression) -> ComparisonResult {
        self.constructor_compareConstant(other)
    }
    fn description_with_precedence(&self, p: OperatorPrecedence) -> Vec<u8> {
        self.constructor_description(p)
    }
}
unsafe impl AnyConstructor for ConstructorCompoundCast {
    fn argumentSpan(&self) -> &[Option<IRNodeBox<dyn Expression>>] {
        self.base.argumentSpan()
    }
    fn argumentSpan_mut(self: Pin<&mut Self>) -> &mut [Option<IRNodeBox<dyn Expression>>] {
        unsafe { self.get_unchecked_mut().base.argumentSpan_mut() }
    }
}
use super::SkSLType::{
    checkForOutOfRangeLiteralExpr, checkForOutOfRangeLiteralValue, isAllowedInES2Context,
};
use super::{
    SkSLConstructorCompound::ConstructorCompound,
    SkSLConstructorDiagonalMatrix::ConstructorDiagonalMatrix,
    SkSLConstructorScalarCast::ConstructorScalarCast, SkSLConstructorSplat::ConstructorSplat,
};
use crate::src::sksl::SkSLConstantFolder::ConstantFolder;
// Source: src/sksl/ir/SkSLConstructorCompoundCast.cpp:25-70
unsafe fn cast_constant_composite(
    context: &Context,
    pos: Position,
    destType: &dyn Type,
    constCtor: IRNodeBox<dyn Expression>,
) -> IRNodeBox<dyn Expression> {
    let scalarType = destType.componentType();
    if constCtor.node().is::<ConstructorSplat>() {
        let splat = &mut *(constCtor.as_ptr() as *mut () as *mut ConstructorSplat);
        let arg = splat.argument_mut().take().unwrap_unchecked();
        return ConstructorSplat::Make(
            context,
            pos,
            destType,
            ConstructorScalarCast::Make(context, pos, scalarType, arg),
        );
    }
    if constCtor.node().is::<ConstructorDiagonalMatrix>() && destType.isMatrix() {
        let matrix = &mut *(constCtor.as_ptr() as *mut () as *mut ConstructorDiagonalMatrix);
        let arg = matrix.argument_mut().take().unwrap_unchecked();
        return ConstructorDiagonalMatrix::Make(
            context,
            pos,
            destType,
            ConstructorScalarCast::Make(context, pos, scalarType, arg),
        );
    }
    let numSlots = destType.slotCount();
    debug_assert_eq!(numSlots, constCtor.expression_type().slotCount());
    let mut typecastArgs = [0.0f64; 16];
    debug_assert!(numSlots <= typecastArgs.len());
    for index in 0..numSlots {
        let mut slotVal = constCtor.getConstantValue(index as i32).unwrap_unchecked();
        if checkForOutOfRangeLiteralValue(scalarType, context, slotVal, constCtor.node().fPosition)
        {
            slotVal = 0.0;
        }
        typecastArgs[index] = slotVal;
    }
    ConstructorCompound::MakeFromConstants(context, pos, destType, &typecastArgs)
}
impl ConstructorCompoundCast {
    // Source: src/sksl/ir/SkSLConstructorCompoundCast.cpp:72-98
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        ty: &dyn Type,
        mut arg: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Expression> {
        debug_assert!(ty.isVector() || ty.isMatrix());
        debug_assert!(isAllowedInES2Context(ty, context));
        debug_assert_eq!(arg.expression_type().isVector(), ty.isVector());
        debug_assert_eq!(arg.expression_type().isMatrix(), ty.isMatrix());
        debug_assert_eq!(ty.columns(), arg.expression_type().columns());
        debug_assert_eq!(ty.rows(), arg.expression_type().rows());
        if ty.matches(arg.expression_type()) {
            arg.as_pin_mut().node_mut().setPosition(pos);
            return arg;
        }
        arg = ConstantFolder::MakeConstantValueForVariable(pos, arg);
        if crate::src::sksl::SkSLAnalysis::IsCompileTimeConstant(&*arg) {
            return cast_constant_composite(context, pos, ty, arg);
        }
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, ty, Some(arg))))
            .cast(|p| p as *mut dyn Expression)
    }
}
