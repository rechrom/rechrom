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
use super::{SkSLType::TypeKind, SkSLVariable::Variable};
use crate::src::sksl::SkSLDefines::ExpressionArray;
// Source: src/sksl/ir/SkSLChildCall.h:31-31
// Source: src/sksl/ir/SkSLChildCall.h:64-64
// Source: src/sksl/ir/SkSLChildCall.h:65-65
#[repr(C)]
pub struct ChildCall {
    base: ExpressionData,
    fChild: NonNull<dyn Variable>,
    fArguments: Pin<Box<ExpressionArray>>,
}
// Source: src/sksl/ir/SkSLChildCall.h:33-33
impl IRNodeType for ChildCall {
    const kIRNodeKind: i32 = ExpressionKind::kChildCall as i32;
}
impl ChildCall {
    // Source: src/sksl/ir/SkSLChildCall.h:35-38
    pub unsafe fn new(
        pos: Position,
        ty: NonNull<dyn Type>,
        child: NonNull<dyn Variable>,
        arguments: Pin<Box<ExpressionArray>>,
    ) -> Self {
        Self {
            base: ExpressionData::new(pos, ExpressionKind::kChildCall, Some(ty)),
            fChild: child,
            fArguments: arguments,
        }
    }
    // Source: src/sksl/ir/SkSLChildCall.h:47-49
    pub unsafe fn child(&self) -> &dyn Variable {
        self.fChild.as_ref()
    }
    // Source: src/sksl/ir/SkSLChildCall.h:51-53
    pub fn arguments_mut(&mut self) -> Pin<&mut ExpressionArray> {
        self.fArguments.as_mut()
    }
    // Source: src/sksl/ir/SkSLChildCall.h:55-57
    pub fn arguments(&self) -> &ExpressionArray {
        &self.fArguments
    }
}
impl IRNode for ChildCall {
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
unsafe impl Expression for ChildCall {
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
// Source: src/sksl/ir/SkSLChildCall.cpp:23-26
unsafe fn clone_at(this: &ChildCall, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
    Some(
        IRNodeBox::new_with(|p: *mut ChildCall| {
            p.write(ChildCall::new(
                pos,
                borrow_type(this.expression_type()),
                this.fChild,
                this.arguments().clone(),
            ))
        })
        .cast(|p| p as *mut dyn Expression),
    )
}
// Source: src/sksl/ir/SkSLChildCall.cpp:28-37
fn description(this: &ChildCall, _: OperatorPrecedence) -> Vec<u8> {
    unsafe {
        let mut result = this.child().name().to_vec();
        result.extend_from_slice(b".eval(");
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
// Source: src/sksl/ir/SkSLChildCall.cpp:39-64
unsafe fn call_signature_is_valid(
    context: &Context,
    child: &dyn Variable,
    arguments: &ExpressionArray,
) -> bool {
    let half4 = borrow_type(&**context.fTypes.as_ref().fHalf4);
    let float2 = borrow_type(&**context.fTypes.as_ref().fFloat2);
    let mut params =
        crate::include::private::SkTArray::STArray::<NonNull<dyn Type>, 2, true>::new_pinned();
    match child.symbol_type().typeKind() {
        TypeKind::kBlender => {
            params.as_mut().push_back(half4);
            params.as_mut().push_back(half4);
        }
        TypeKind::kColorFilter => {
            params.as_mut().push_back(half4);
        }
        TypeKind::kShader => {
            params.as_mut().push_back(float2);
        }
        _ => core::hint::unreachable_unchecked(),
    }
    if params.size() != arguments.as_base().size() {
        return false;
    }
    for i in 0..arguments.as_base().size() as usize {
        if !arguments.as_base().as_slice()[i]
            .as_ref()
            .unwrap_unchecked()
            .expression_type()
            .matches(params.as_slice()[i].as_ref())
        {
            return false;
        }
    }
    true
}
impl ChildCall {
    // Source: src/sksl/ir/SkSLChildCall.cpp:66-73
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        returnType: NonNull<dyn Type>,
        child: NonNull<dyn Variable>,
        arguments: Pin<Box<ExpressionArray>>,
    ) -> IRNodeBox<dyn Expression> {
        debug_assert!(call_signature_is_valid(context, child.as_ref(), &arguments));
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, returnType, child, arguments)))
            .cast(|p| p as *mut dyn Expression)
    }
}
