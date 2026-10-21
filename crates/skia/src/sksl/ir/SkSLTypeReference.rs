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
use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
// Source: src/sksl/ir/SkSLTypeReference.h:31-31
// Source: src/sksl/ir/SkSLTypeReference.h:69-69
#[repr(C)]
pub struct TypeReference {
    base: ExpressionData,
    fValue: NonNull<dyn Type>,
}
// Source: src/sksl/ir/SkSLTypeReference.h:33-33
impl IRNodeType for TypeReference {
    const kIRNodeKind: i32 = ExpressionKind::kTypeReference as i32;
}
impl TypeReference {
    // Source: src/sksl/ir/SkSLTypeReference.h:35-36
    pub unsafe fn new(context: &Context, pos: Position, value: NonNull<dyn Type>) -> Self {
        Self::new_typed(pos, value, borrow_type(&**context.fTypes.as_ref().fInvalid))
    }
    // Source: src/sksl/ir/SkSLTypeReference.h:52-54
    pub unsafe fn value(&self) -> &dyn Type {
        self.fValue.as_ref()
    }
}
impl IRNode for TypeReference {
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
unsafe impl Expression for TypeReference {
    fn expression_data(&self) -> &ExpressionData {
        &self.base
    }
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base) }
    }
    fn as_expression(&self) -> &dyn Expression {
        self
    }
    // Source: src/sksl/ir/SkSLTypeReference.h:56-58
    fn description_with_precedence(&self, _: OperatorPrecedence) -> Vec<u8> {
        unsafe { self.value().name().to_vec() }
    }
    // Source: src/sksl/ir/SkSLTypeReference.h:60-62
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        Some(
            IRNodeBox::new_with(|p: *mut Self| {
                p.write(Self::new_typed(
                    pos,
                    self.fValue,
                    borrow_type(self.expression_type()),
                ))
            })
            .cast(|p| p as *mut dyn Expression),
        )
    }
}
impl TypeReference {
    // Source: src/sksl/ir/SkSLTypeReference.h:65-67
    unsafe fn new_typed(pos: Position, value: NonNull<dyn Type>, ty: NonNull<dyn Type>) -> Self {
        Self {
            base: ExpressionData::new(pos, ExpressionKind::kTypeReference, Some(ty)),
            fValue: value,
        }
    }
    // Source: src/sksl/ir/SkSLTypeReference.cpp:16-29
    pub unsafe fn VerifyType(
        context: &Context,
        ty: Option<NonNull<dyn Type>>,
        pos: Position,
    ) -> bool {
        if !context.fConfig.unwrap_unchecked().as_ref().isBuiltinCode() {
            if let Some(ty) = ty {
                let ty = ty.as_ref();
                if ty.isGeneric() || ty.isLiteral() {
                    let mut msg = b"type '".to_vec();
                    msg.extend_from_slice(ty.name());
                    msg.extend_from_slice(b"' is generic");
                    let mut errors = context.fErrors;
                    errors.as_mut().error(pos, &msg);
                    return false;
                }
                if !super::SkSLType::isAllowedInES2Context(ty, context) {
                    let mut msg = b"type '".to_vec();
                    msg.extend_from_slice(ty.name());
                    msg.extend_from_slice(b"' is not supported");
                    let mut errors = context.fErrors;
                    errors.as_mut().error(pos, &msg);
                    return false;
                }
            }
        }
        true
    }
    // Source: src/sksl/ir/SkSLTypeReference.cpp:31-36
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        ty: NonNull<dyn Type>,
    ) -> Option<IRNodeBox<Self>> {
        if Self::VerifyType(context, Some(ty), pos) {
            Some(Self::Make(context, pos, ty))
        } else {
            None
        }
    }
    // Source: src/sksl/ir/SkSLTypeReference.cpp:37-42
    pub unsafe fn Make(context: &Context, pos: Position, ty: NonNull<dyn Type>) -> IRNodeBox<Self> {
        debug_assert!(super::SkSLType::isAllowedInES2Context(ty.as_ref(), context));
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(context, pos, ty)))
    }
}
