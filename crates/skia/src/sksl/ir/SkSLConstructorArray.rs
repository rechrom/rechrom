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
// Source: src/sksl/ir/SkSLConstructorArray.h:28-28
// Adapted native inherited subobject only; no extra constructor state.
#[repr(C)]
pub struct ConstructorArray {
    pub(crate) base: MultiArgumentConstructorData,
}
// Source: src/sksl/ir/SkSLConstructorArray.h:30-30
impl IRNodeType for ConstructorArray {
    const kIRNodeKind: i32 = ExpressionKind::kConstructorArray as i32;
}
impl ConstructorArray {
    // Source: src/sksl/ir/SkSLConstructorArray.h:32-33
    /// Safety: Type and child-expression borrowed dependencies retain their native lifetime/pool lease.
    pub unsafe fn new(pos: Position, ty: &dyn Type, arg: Pin<Box<ExpressionArray>>) -> Self {
        let ty =
            core::mem::transmute::<NonNull<dyn Type + '_>, NonNull<dyn Type>>(NonNull::from(ty));
        Self {
            base: MultiArgumentConstructorData::new(
                pos,
                ExpressionKind::kConstructorArray,
                ty,
                arg,
            ),
        }
    }
    pub fn arguments(&self) -> &ExpressionArray {
        self.base.arguments()
    }
    pub fn arguments_mut(&mut self) -> Pin<&mut ExpressionArray> {
        self.base.arguments_mut()
    }
}
impl IRNode for ConstructorArray {
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
unsafe impl Expression for ConstructorArray {
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
    // Source: src/sksl/ir/SkSLConstructorArray.h:48-50
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        let arg = self.arguments().clone();
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
unsafe impl AnyConstructor for ConstructorArray {
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
use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
impl ConstructorArray {
    // Source: src/sksl/ir/SkSLConstructorArray.cpp:23-77
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        ty: &dyn Type,
        mut args: Pin<Box<ExpressionArray>>,
    ) -> Option<IRNodeBox<dyn Expression>> {
        debug_assert!(ty.isArray() && ty.columns() > 0);
        if context.fConfig.unwrap_unchecked().as_ref().strictES2Mode() {
            let mut message = b"construction of array type '".to_vec();
            message.extend_from_slice(&ty.displayName());
            message.extend_from_slice(b"' is not supported");
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &message);
            return None;
        }
        if ty.isOrContainsAtomic() {
            let display = ty.displayName();
            let display = display.split(|b| *b == 0).next().unwrap_unchecked();
            let mut message = b"construction of array type '".to_vec();
            message.extend_from_slice(display);
            message.extend_from_slice(b"' with atomic member is not allowed");
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &message);
            return None;
        }
        if args.as_base().size() == 1 {
            let expr = args.as_base().as_slice()[0].as_ref().unwrap_unchecked();
            let exprType = expr.expression_type();
            if exprType.isArray() && exprType.canCoerceTo(ty, true) {
                return Some(super::SkSLConstructorArrayCast::ConstructorArrayCast::Make(
                    context,
                    pos,
                    ty,
                    args.as_mut().as_base_mut().as_mut_slice()[0]
                        .take()
                        .unwrap_unchecked(),
                ));
            }
        }
        if ty.columns() != args.as_base().size() {
            let display = ty.displayName();
            let display = display.split(|b| *b == 0).next().unwrap_unchecked();
            let mut message = b"invalid arguments to '".to_vec();
            message.extend_from_slice(display);
            message.extend_from_slice(b"' constructor (expected ");
            message.extend_from_slice(ty.columns().to_string().as_bytes());
            message.extend_from_slice(b" elements, but found ");
            message.extend_from_slice(args.as_base().size().to_string().as_bytes());
            message.push(b')');
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &message);
            return None;
        }
        let baseType = ty.componentType();
        for argument in args.as_mut().as_base_mut().as_mut_slice() {
            *argument = super::SkSLType::coerceExpression(baseType, argument.take(), context);
            if argument.is_none() {
                return None;
            }
        }
        Some(Self::Make(context, pos, ty, args))
    }
    // Source: src/sksl/ir/SkSLConstructorArray.cpp:79-92
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        ty: &dyn Type,
        args: Pin<Box<ExpressionArray>>,
    ) -> IRNodeBox<dyn Expression> {
        debug_assert!(!context.fConfig.unwrap_unchecked().as_ref().strictES2Mode());
        debug_assert!(isAllowedInES2Context(ty, context));
        debug_assert_eq!(ty.columns(), args.as_base().size());
        debug_assert!(!ty.isOrContainsAtomic());
        debug_assert!(args.as_base().as_slice().iter().all(|arg| ty
            .componentType()
            .matches(arg.as_ref().unwrap_unchecked().expression_type())));
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, ty, args)))
            .cast(|p| p as *mut dyn Expression)
    }
}
