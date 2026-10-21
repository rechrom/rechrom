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
// Source: src/sksl/ir/SkSLConstructorSplat.h:31-31
// Adapted native inherited subobject only; no extra constructor state.
#[repr(C)]
pub struct ConstructorSplat {
    pub(crate) base: SingleArgumentConstructorData,
}
// Source: src/sksl/ir/SkSLConstructorSplat.h:33-33
impl IRNodeType for ConstructorSplat {
    const kIRNodeKind: i32 = ExpressionKind::kConstructorSplat as i32;
}
impl ConstructorSplat {
    // Source: src/sksl/ir/SkSLConstructorSplat.h:35-36
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
                ExpressionKind::kConstructorSplat,
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
impl IRNode for ConstructorSplat {
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
unsafe impl Expression for ConstructorSplat {
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
    // Source: src/sksl/ir/SkSLConstructorSplat.h:44-46
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        let arg = self.argument().as_ref().unwrap_unchecked().clone_current();
        Some(
            IRNodeBox::new_with(|p: *mut Self| {
                p.write(Self::new(pos, self.expression_type(), arg))
            })
            .cast(|p| p as *mut dyn Expression),
        )
    }
    // Source: src/sksl/ir/SkSLConstructorSplat.h:48-50
    fn supportsConstantValues(&self) -> bool {
        true
    }
    // Source: src/sksl/ir/SkSLConstructorSplat.h:52-55
    fn getConstantValue(&self, n: i32) -> Option<f64> {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLConstructorSplat.h\0".as_ptr(),53; @condition b"n >= 0 && n < this->type().columns()\0".as_ptr(); n>=0&&n<self.expression_type().columns());
        unsafe { self.argument().as_ref().unwrap_unchecked() }.getConstantValue(0)
    }
    fn compareConstant(&self, other: &dyn Expression) -> ComparisonResult {
        self.constructor_compareConstant(other)
    }
    fn description_with_precedence(&self, p: OperatorPrecedence) -> Vec<u8> {
        self.constructor_description(p)
    }
}
unsafe impl AnyConstructor for ConstructorSplat {
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
use crate::src::sksl::SkSLConstantFolder::ConstantFolder;
impl ConstructorSplat {
    // Source: src/sksl/ir/SkSLConstructorSplat.cpp:14-36
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        ty: &dyn Type,
        mut arg: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Expression> {
        debug_assert!(isAllowedInES2Context(ty, context));
        debug_assert!(ty.isScalar() || ty.isVector());
        debug_assert!(arg
            .expression_type()
            .scalarTypeForLiteral()
            .matches(ty.componentType().scalarTypeForLiteral()));
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLConstructorSplat.cpp\0".as_ptr(),22; @condition b"arg->type().isScalar()\0".as_ptr(); arg.expression_type().isScalar());
        if ty.isScalar() {
            arg.as_pin_mut().node_mut().setPosition(pos);
            return arg;
        }
        arg = ConstantFolder::MakeConstantValueForVariable(pos, arg);
        debug_assert!(ty.isVector());
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, ty, Some(arg))))
            .cast(|p| p as *mut dyn Expression)
    }
}
