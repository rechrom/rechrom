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
use super::SkSLFunctionDeclaration::FunctionDeclaration;
// Source: src/sksl/ir/SkSLFunctionReference.h:22-22
// Source: src/sksl/ir/SkSLFunctionReference.h:49-49
#[repr(C)]
pub struct FunctionReference {
    base: ExpressionData,
    fOverloadChain: Option<NonNull<FunctionDeclaration>>,
}
// Source: src/sksl/ir/SkSLFunctionReference.h:24-24
impl IRNodeType for FunctionReference {
    const kIRNodeKind: i32 = ExpressionKind::kFunctionReference as i32;
}
impl FunctionReference {
    // Source: src/sksl/ir/SkSLFunctionReference.h:26-29
    pub unsafe fn new(
        context: &Context,
        pos: Position,
        overloadChain: Option<NonNull<FunctionDeclaration>>,
    ) -> Self {
        Self::new_typed(
            pos,
            overloadChain,
            borrow_type(&**context.fTypes.as_ref().fInvalid),
        )
    }
    // Source: src/sksl/ir/SkSLFunctionReference.h:31-33
    pub fn overloadChain(&self) -> Option<NonNull<FunctionDeclaration>> {
        self.fOverloadChain
    }
}
impl IRNode for FunctionReference {
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
unsafe impl Expression for FunctionReference {
    fn expression_data(&self) -> &ExpressionData {
        &self.base
    }
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base) }
    }
    fn as_expression(&self) -> &dyn Expression {
        self
    }
    // Source: src/sksl/ir/SkSLFunctionReference.h:35-38
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        Some(
            IRNodeBox::new_with(|p: *mut Self| {
                p.write(Self::new_typed(
                    pos,
                    self.overloadChain(),
                    borrow_type(self.expression_type()),
                ))
            })
            .cast(|p| p as *mut dyn Expression),
        )
    }
    // Source: src/sksl/ir/SkSLFunctionReference.h:40-42
    fn description_with_precedence(&self, _: OperatorPrecedence) -> Vec<u8> {
        b"<function>".to_vec()
    }
}
impl FunctionReference {
    // Source: src/sksl/ir/SkSLFunctionReference.h:45-47
    unsafe fn new_typed(
        pos: Position,
        overloadChain: Option<NonNull<FunctionDeclaration>>,
        ty: NonNull<dyn Type>,
    ) -> Self {
        Self {
            base: ExpressionData::new(pos, ExpressionKind::kFunctionReference, Some(ty)),
            fOverloadChain: overloadChain,
        }
    }
}
