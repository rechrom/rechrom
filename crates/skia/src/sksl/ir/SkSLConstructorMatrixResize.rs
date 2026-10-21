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
// Source: src/sksl/ir/SkSLConstructorMatrixResize.h:31-31
// Adapted native inherited subobject only; no extra constructor state.
#[repr(C)]
pub struct ConstructorMatrixResize {
    pub(crate) base: SingleArgumentConstructorData,
}
// Source: src/sksl/ir/SkSLConstructorMatrixResize.h:33-33
impl IRNodeType for ConstructorMatrixResize {
    const kIRNodeKind: i32 = ExpressionKind::kConstructorMatrixResize as i32;
}
impl ConstructorMatrixResize {
    // Source: src/sksl/ir/SkSLConstructorMatrixResize.h:35-36
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
                ExpressionKind::kConstructorMatrixResize,
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
impl IRNode for ConstructorMatrixResize {
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
unsafe impl Expression for ConstructorMatrixResize {
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
    // Source: src/sksl/ir/SkSLConstructorMatrixResize.h:43-45
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        let arg = self.argument().as_ref().unwrap_unchecked().clone_current();
        Some(
            IRNodeBox::new_with(|p: *mut Self| {
                p.write(Self::new(pos, self.expression_type(), arg))
            })
            .cast(|p| p as *mut dyn Expression),
        )
    }
    // Source: src/sksl/ir/SkSLConstructorMatrixResize.h:47-47
    fn supportsConstantValues(&self) -> bool {
        true
    }
    fn getConstantValue(&self, n: i32) -> Option<f64> {
        getConstantValue(self, n)
    }
    fn compareConstant(&self, other: &dyn Expression) -> ComparisonResult {
        self.constructor_compareConstant(other)
    }
    fn description_with_precedence(&self, p: OperatorPrecedence) -> Vec<u8> {
        self.constructor_description(p)
    }
}
unsafe impl AnyConstructor for ConstructorMatrixResize {
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
impl ConstructorMatrixResize {
    // Source: src/sksl/ir/SkSLConstructorMatrixResize.cpp:15-29
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        ty: &dyn Type,
        arg: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Expression> {
        debug_assert!(ty.isMatrix());
        debug_assert!(isAllowedInES2Context(ty, context));
        debug_assert!(arg
            .expression_type()
            .componentType()
            .matches(ty.componentType()));
        if ty.rows() == arg.expression_type().rows()
            && ty.columns() == arg.expression_type().columns()
        {
            return arg;
        }
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, ty, Some(arg))))
            .cast(|p| p as *mut dyn Expression)
    }
}
// Source: src/sksl/ir/SkSLConstructorMatrixResize.cpp:31-56
fn getConstantValue(this: &ConstructorMatrixResize, mut n: i32) -> Option<f64> {
    let rows = this.expression_type().rows();
    let row = n % rows;
    let col = n / rows;
    crate::SkASSERT!(@source b"src/sksl/ir/SkSLConstructorMatrixResize.cpp\0".as_ptr(),36; @condition b"col >= 0\0".as_ptr(); col>=0);
    crate::SkASSERT!(@source b"src/sksl/ir/SkSLConstructorMatrixResize.cpp\0".as_ptr(),37; @condition b"row >= 0\0".as_ptr(); row>=0);
    debug_assert!(col < this.expression_type().columns());
    debug_assert!(row < this.expression_type().rows());
    let argument = unsafe { this.argument().as_ref().unwrap_unchecked() };
    if col < argument.expression_type().columns() && row < argument.expression_type().rows() {
        n = row + col * argument.expression_type().rows();
        return argument.getConstantValue(n);
    }
    Some(if col == row { 1.0 } else { 0.0 })
}
