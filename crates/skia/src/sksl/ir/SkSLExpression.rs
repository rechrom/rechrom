// Source: src/sksl/ir/SkSLExpression.h:23-23
pub use super::SkSLConstructor::AnyConstructor;
// Source: src/sksl/ir/SkSLExpression.h:24-24
pub use crate::src::sksl::SkSLContext::Context;
// Source: src/sksl/ir/SkSLExpression.h:25-25
use super::{
    SkSLIRNode::{ExpressionKind, IRNode, IRNodeBox, IRNodeData},
    SkSLType::{CoercionCost, Type},
};
pub use crate::src::sksl::SkSLOperator::OperatorPrecedence;
use crate::src::sksl::{SkSLDefines::ExpressionArray, SkSLPosition::Position};
use core::{pin::Pin, ptr::NonNull};
// Source: src/sksl/ir/SkSLExpression.h:30-30
// Source: src/sksl/ir/SkSLExpression.h:136-136
// Actual inherited state only; Type is borrowed, never a second owner.
#[repr(C)]
pub struct ExpressionData {
    pub(crate) base: IRNodeData,
    fType: Option<NonNull<dyn Type>>,
}
// Source: src/sksl/ir/SkSLExpression.h:32-32
pub type Kind = ExpressionKind;
impl ExpressionData {
    // Source: src/sksl/ir/SkSLExpression.h:34-38
    /// Safety: every published Expression has a live actual Type in ty, and its
    /// allocation remains alive for all expression access. No null Type is fabricated.
    pub unsafe fn new(pos: Position, kind: Kind, ty: Option<NonNull<dyn Type>>) -> Self {
        let value = Self {
            base: IRNodeData::new(pos, kind as i32),
            fType: ty,
        };
        debug_assert!(kind as i32 >= Kind::kFirst as i32 && kind as i32 <= Kind::kLast as i32);
        value
    }
}
// Rust extension protocol keeps the native pre-enum method group in source
// order. Rust traits cannot contain ComparisonResult's nested enum declaration.
// This is language dispatch glue, not a second Expression object or state.
pub trait ExpressionExt: Expression {
    // Source: src/sksl/ir/SkSLExpression.h:40-42
    fn kind(&self) -> Kind {
        unsafe { core::mem::transmute(self.node().fKind) }
    }
    // Source: src/sksl/ir/SkSLExpression.h:44-46
    fn expression_type(&self) -> &dyn Type {
        unsafe { self.expression_data().fType.unwrap_unchecked().as_ref() }
    }
    // Source: src/sksl/ir/SkSLExpression.h:48-52
    fn isAnyConstructor(&self) -> bool {
        const _: () = assert!(Kind::kConstructorArray as i32 - 1 == Kind::kChildCall as i32);
        const _: () = assert!(Kind::kConstructorStruct as i32 + 1 == Kind::kEmpty as i32);
        self.kind() as i32 >= Kind::kConstructorArray as i32
            && self.kind() as i32 <= Kind::kConstructorStruct as i32
    }
    // Source: src/sksl/ir/SkSLExpression.h:54-56
    fn isIntLiteral(&self) -> bool {
        self.kind() == Kind::kLiteral && self.expression_type().isInteger()
    }
    // Source: src/sksl/ir/SkSLExpression.h:58-60
    fn isFloatLiteral(&self) -> bool {
        self.kind() == Kind::kLiteral && self.expression_type().isFloat()
    }
    // Source: src/sksl/ir/SkSLExpression.h:62-64
    fn isBoolLiteral(&self) -> bool {
        self.kind() == Kind::kLiteral && self.expression_type().isBoolean()
    }
    // Source: src/sksl/ir/SkSLExpression.h:66-67
    // Native overloads use the actual AnyConstructor inheritance projections.
    unsafe fn asAnyConstructor(&self) -> &dyn super::SkSLConstructor::AnyConstructor {
        debug_assert!(self.isAnyConstructor());
        self.constructor_view()
    }
    unsafe fn asAnyConstructor_mut(
        self: Pin<&mut Self>,
    ) -> Pin<&mut dyn super::SkSLConstructor::AnyConstructor> {
        debug_assert!(self.as_ref().get_ref().isAnyConstructor());
        self.constructor_view_mut()
    }
    // Source: src/sksl/ir/SkSLExpression.h:74-74
    /// Safety: Context's reporter retains its native exclusive diagnostic callback lease.
    unsafe fn isIncomplete(&self, context: &crate::src::sksl::SkSLContext::Context) -> bool {
        isIncomplete(self.as_expression(), context)
    }
}
impl<T: Expression + ?Sized> ExpressionExt for T {}
// Source: src/sksl/ir/SkSLExpression.h:81-85
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComparisonResult {
    kUnknown = -1,
    kNotEqual = 0,
    kEqual = 1,
}
// Native nonvirtual methods remain unchanged. Actual Expression subclasses must
// implement both pure virtuals; no concrete default Expression is supplied.
/// Safety: base is this object's actual inherited subobject with original Kind
/// and live Type. Do not override native nonvirtual methods. IRNode::description
/// must delegate to expression_description, preserving the native final override.
pub unsafe trait Expression: IRNode {
    fn expression_data(&self) -> &ExpressionData;
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData>;
    fn as_expression(&self) -> &dyn Expression;
    /// Unverified v44 language-only inheritance projection. Safety: any native constructor Kind must
    /// override this with its real AnyConstructor subobject. Other Kinds cannot be downcast.
    unsafe fn constructor_view(&self) -> &dyn super::SkSLConstructor::AnyConstructor {
        debug_assert!(self.isAnyConstructor());
        core::hint::unreachable_unchecked()
    }
    unsafe fn constructor_view_mut(
        self: Pin<&mut Self>,
    ) -> Pin<&mut dyn super::SkSLConstructor::AnyConstructor> {
        debug_assert!(self.as_ref().get_ref().isAnyConstructor());
        core::hint::unreachable_unchecked()
    }

    // Source: src/sksl/ir/SkSLExpression.h:86-88
    fn compareConstant(&self, _other: &dyn Expression) -> ComparisonResult {
        ComparisonResult::kUnknown
    }
    // Source: src/sksl/ir/SkSLExpression.h:90-92
    fn coercionCost(&self, target: &dyn Type) -> CoercionCost {
        self.expression_type().coercionCost(target)
    }
    // Source: src/sksl/ir/SkSLExpression.h:103-105
    fn supportsConstantValues(&self) -> bool {
        false
    }
    // Source: src/sksl/ir/SkSLExpression.h:116-119
    fn getConstantValue(&self, _n: i32) -> Option<f64> {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLExpression.h\0".as_ptr(),117; @condition b"!this->supportsConstantValues()\0".as_ptr(); !self.supportsConstantValues());
        None
    }
    // Source: src/sksl/ir/SkSLExpression.h:121-121
    /// Safety: native dynamic clone allocation/borrowed dependencies retain the
    /// same current-pool destruction lease as the original concrete node.
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>>;
    // Source: src/sksl/ir/SkSLExpression.h:126-126
    unsafe fn clone_current(&self) -> Option<IRNodeBox<dyn Expression>> {
        self.clone_at(self.node().fPosition)
    }
    // Language-only forwarding bridge for the native final IRNode override below.
    fn expression_description(&self) -> Vec<u8> {
        description(self.as_expression())
    }
    // Source: src/sksl/ir/SkSLExpression.h:132-132
    fn description_with_precedence(&self, parentPrecedence: OperatorPrecedence) -> Vec<u8>;
}
// Source: src/sksl/ir/SkSLExpression.cpp:18-20
pub fn description(expr: &dyn Expression) -> Vec<u8> {
    expr.description_with_precedence(OperatorPrecedence::kExpression)
}
// Source: src/sksl/ir/SkSLExpression.cpp:22-47
pub unsafe fn isIncomplete(
    expr: &dyn Expression,
    context: &crate::src::sksl::SkSLContext::Context,
) -> bool {
    use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
    match expr.kind() {
        Kind::kFunctionReference => {
            (&mut *context.fErrors.as_ptr()).error(
                expr.node().fPosition.after(),
                b"expected '(' to begin function call",
            );
            true
        }
        Kind::kMethodReference => {
            (&mut *context.fErrors.as_ptr()).error(
                expr.node().fPosition.after(),
                b"expected '(' to begin method call",
            );
            true
        }
        Kind::kTypeReference => {
            (&mut *context.fErrors.as_ptr()).error(
                expr.node().fPosition.after(),
                b"expected '(' to begin constructor invocation",
            );
            true
        }
        Kind::kVariableReference => {
            if expr
                .expression_type()
                .matches(&**context.fTypes.as_ref().fSkCaps)
            {
                (&mut *context.fErrors.as_ptr())
                    .error(expr.node().fPosition, b"invalid expression");
                return true;
            }
            false
        }
        _ => false,
    }
}
impl ExpressionArray {
    // Source: src/sksl/ir/SkSLExpression.cpp:49-56
    /// Safety: all dynamically cloned concrete nodes retain their borrowed backing
    /// and same current pool through destruction. Box pinning adapts native STArray
    /// movable-return semantics without moving any initialized inline pointer storage.
    pub unsafe fn clone(&self) -> Pin<Box<ExpressionArray>> {
        let mut cloned = ExpressionArray::new();
        cloned
            .as_mut()
            .as_base_mut()
            .reserve_exact(self.as_base().size());
        for expr in self.as_base().as_slice() {
            cloned.as_mut().as_base_mut().push_back(match expr {
                Some(expr) => expr.clone_current(),
                None => None,
            });
        }
        cloned
    }
}
