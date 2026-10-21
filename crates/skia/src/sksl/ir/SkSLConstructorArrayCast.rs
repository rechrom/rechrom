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
// Source: src/sksl/ir/SkSLConstructorArrayCast.h:32-32
// Adapted native inherited subobject only; no extra constructor state.
#[repr(C)]
pub struct ConstructorArrayCast {
    pub(crate) base: SingleArgumentConstructorData,
}
// Source: src/sksl/ir/SkSLConstructorArrayCast.h:34-34
impl IRNodeType for ConstructorArrayCast {
    const kIRNodeKind: i32 = ExpressionKind::kConstructorArrayCast as i32;
}
impl ConstructorArrayCast {
    // Source: src/sksl/ir/SkSLConstructorArrayCast.h:36-37
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
                ExpressionKind::kConstructorArrayCast,
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
impl IRNode for ConstructorArrayCast {
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
unsafe impl Expression for ConstructorArrayCast {
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
    // Source: src/sksl/ir/SkSLConstructorArrayCast.h:44-46
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
unsafe impl AnyConstructor for ConstructorArrayCast {
    fn argumentSpan(&self) -> &[Option<IRNodeBox<dyn Expression>>] {
        self.base.argumentSpan()
    }
    fn argumentSpan_mut(self: Pin<&mut Self>) -> &mut [Option<IRNodeBox<dyn Expression>>] {
        unsafe { self.get_unchecked_mut().base.argumentSpan_mut() }
    }
}
use super::{
    SkSLConstructorArray::ConstructorArray, SkSLConstructorCompoundCast::ConstructorCompoundCast,
    SkSLConstructorScalarCast::ConstructorScalarCast, SkSLType::isAllowedInES2Context,
};
use crate::src::sksl::SkSLConstantFolder::ConstantFolder;
// Source: src/sksl/ir/SkSLConstructorArrayCast.cpp:22-44
unsafe fn cast_constant_array(
    context: &Context,
    pos: Position,
    destType: &dyn Type,
    constCtor: IRNodeBox<dyn Expression>,
) -> IRNodeBox<dyn Expression> {
    let scalarType = destType.componentType();
    debug_assert!(constCtor.node().is::<ConstructorArray>());
    let array = &mut *(constCtor.as_ptr() as *mut () as *mut ConstructorArray);
    let inputArgs = Pin::new_unchecked(array).argumentSpan_mut();
    let mut typecastArgs = ExpressionArray::new();
    typecastArgs
        .as_mut()
        .as_base_mut()
        .reserve_exact(inputArgs.len() as i32);
    for arg in inputArgs {
        let argPos = arg.as_ref().unwrap_unchecked().node().fPosition;
        if arg.as_ref().unwrap_unchecked().expression_type().isScalar() {
            typecastArgs
                .as_mut()
                .as_base_mut()
                .push_back(Some(ConstructorScalarCast::Make(
                    context,
                    argPos,
                    scalarType,
                    arg.take().unwrap_unchecked(),
                )));
        } else {
            typecastArgs
                .as_mut()
                .as_base_mut()
                .push_back(Some(ConstructorCompoundCast::Make(
                    context,
                    argPos,
                    scalarType,
                    arg.take().unwrap_unchecked(),
                )));
        }
    }
    ConstructorArray::Make(context, pos, destType, typecastArgs)
}
impl ConstructorArrayCast {
    // Source: src/sksl/ir/SkSLConstructorArrayCast.cpp:46-71
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        ty: &dyn Type,
        mut arg: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Expression> {
        debug_assert!(ty.isArray());
        debug_assert!(isAllowedInES2Context(ty, context));
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLConstructorArrayCast.cpp\0".as_ptr(),53; @condition b"arg->type().isArray()\0".as_ptr(); arg.expression_type().isArray());
        debug_assert_eq!(ty.columns(), arg.expression_type().columns());
        if ty.matches(arg.expression_type()) {
            arg.as_pin_mut().node_mut().setPosition(pos);
            return arg;
        }
        arg = ConstantFolder::MakeConstantValueForVariable(pos, arg);
        if crate::src::sksl::SkSLAnalysis::IsCompileTimeConstant(&*arg) {
            return cast_constant_array(context, pos, ty, arg);
        }
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, ty, Some(arg))))
            .cast(|p| p as *mut dyn Expression)
    }
}
