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
// Source: src/sksl/ir/SkSLMethodReference.h:31-31
// Source: src/sksl/ir/SkSLMethodReference.h:66-67
#[repr(C)]
pub struct MethodReference {
    base: ExpressionData,
    fSelf: Option<IRNodeBox<dyn Expression>>,
    fOverloadChain: Option<NonNull<FunctionDeclaration>>,
}
// Source: src/sksl/ir/SkSLMethodReference.h:33-33
impl IRNodeType for MethodReference {
    const kIRNodeKind: i32 = ExpressionKind::kMethodReference as i32;
}
impl MethodReference {
    // Source: src/sksl/ir/SkSLMethodReference.h:35-41
    pub unsafe fn new(
        context: &Context,
        pos: Position,
        this: Option<IRNodeBox<dyn Expression>>,
        chain: Option<NonNull<FunctionDeclaration>>,
    ) -> Self {
        Self::new_typed(
            pos,
            this,
            chain,
            borrow_type(&**context.fTypes.as_ref().fInvalid),
        )
    }
    // Source: src/sksl/ir/SkSLMethodReference.h:43-44
    pub unsafe fn self_mut(&mut self) -> &mut Option<IRNodeBox<dyn Expression>> {
        &mut self.fSelf
    }
    pub fn self_expression(&self) -> &Option<IRNodeBox<dyn Expression>> {
        &self.fSelf
    }
    // Source: src/sksl/ir/SkSLMethodReference.h:46-46
    pub fn overloadChain(&self) -> Option<NonNull<FunctionDeclaration>> {
        self.fOverloadChain
    }
}
impl IRNode for MethodReference {
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
unsafe impl Expression for MethodReference {
    fn expression_data(&self) -> &ExpressionData {
        &self.base
    }
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base) }
    }
    fn as_expression(&self) -> &dyn Expression {
        self
    }
    // Source: src/sksl/ir/SkSLMethodReference.h:48-51
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        Some(
            IRNodeBox::new_with(|p: *mut Self| {
                p.write(Self::new_typed(
                    pos,
                    self.fSelf.as_ref().unwrap_unchecked().clone_current(),
                    self.overloadChain(),
                    borrow_type(self.expression_type()),
                ))
            })
            .cast(|p| p as *mut dyn Expression),
        )
    }
    // Source: src/sksl/ir/SkSLMethodReference.h:53-55
    fn description_with_precedence(&self, _: OperatorPrecedence) -> Vec<u8> {
        b"<method>".to_vec()
    }
}
impl MethodReference {
    // Source: src/sksl/ir/SkSLMethodReference.h:58-64
    unsafe fn new_typed(
        pos: Position,
        this: Option<IRNodeBox<dyn Expression>>,
        chain: Option<NonNull<FunctionDeclaration>>,
        ty: NonNull<dyn Type>,
    ) -> Self {
        Self {
            base: ExpressionData::new(pos, ExpressionKind::kMethodReference, Some(ty)),
            fSelf: this,
            fOverloadChain: chain,
        }
    }
}
