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
// Source: src/sksl/ir/SkSLConstructorScalarCast.h:30-30
// Adapted native inherited subobject only; no extra constructor state.
#[repr(C)]
pub struct ConstructorScalarCast {
    pub(crate) base: SingleArgumentConstructorData,
}
// Source: src/sksl/ir/SkSLConstructorScalarCast.h:32-32
impl IRNodeType for ConstructorScalarCast {
    const kIRNodeKind: i32 = ExpressionKind::kConstructorScalarCast as i32;
}
impl ConstructorScalarCast {
    // Source: src/sksl/ir/SkSLConstructorScalarCast.h:34-35
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
                ExpressionKind::kConstructorScalarCast,
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
impl IRNode for ConstructorScalarCast {
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
unsafe impl Expression for ConstructorScalarCast {
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
    // Source: src/sksl/ir/SkSLConstructorScalarCast.h:51-53
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
unsafe impl AnyConstructor for ConstructorScalarCast {
    fn argumentSpan(&self) -> &[Option<IRNodeBox<dyn Expression>>] {
        self.base.argumentSpan()
    }
    fn argumentSpan_mut(self: Pin<&mut Self>) -> &mut [Option<IRNodeBox<dyn Expression>>] {
        unsafe { self.get_unchecked_mut().base.argumentSpan_mut() }
    }
}
use super::SkSLLiteral::Literal;
use super::SkSLType::{
    checkForOutOfRangeLiteralExpr, checkForOutOfRangeLiteralValue, isAllowedInES2Context,
};
use crate::src::sksl::SkSLConstantFolder::ConstantFolder;
use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
impl ConstructorScalarCast {
    // Source: src/sksl/ir/SkSLConstructorScalarCast.cpp:23-61
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        rawType: &dyn Type,
        mut args: Pin<Box<ExpressionArray>>,
    ) -> Option<IRNodeBox<dyn Expression>> {
        let ty = rawType.scalarTypeForLiteral();
        debug_assert!(ty.isScalar());
        if args.as_base().size() != 1 {
            let mut message = b"invalid arguments to '".to_vec();
            message.extend_from_slice(&ty.displayName());
            message.extend_from_slice(b"' constructor, (expected exactly 1 argument, but found ");
            message.extend_from_slice(args.as_base().size().to_string().as_bytes());
            message.push(b')');
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &message);
            return None;
        }
        let arg = args.as_base().as_slice()[0].as_ref().unwrap_unchecked();
        let argType = arg.expression_type();
        if !argType.isScalar() {
            let mut hint = b"".as_slice();
            if argType.componentType().matches(ty) {
                if argType.isVector() {
                    hint = b"; use '.x' instead";
                } else if argType.isMatrix() {
                    hint = b"; use '[0][0]' instead";
                }
            }
            let mut message = b"'".to_vec();
            message.extend_from_slice(&argType.displayName());
            message.extend_from_slice(b"' is not a valid parameter to '");
            message.extend_from_slice(&ty.displayName());
            message.extend_from_slice(b"' constructor");
            message.extend_from_slice(hint);
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &message);
            return None;
        }
        if checkForOutOfRangeLiteralExpr(ty, context, &**arg) {
            return None;
        }
        Some(Self::Make(
            context,
            pos,
            ty,
            args.as_mut().as_base_mut().as_mut_slice()[0]
                .take()
                .unwrap_unchecked(),
        ))
    }
    // Source: src/sksl/ir/SkSLConstructorScalarCast.cpp:63-106
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        ty: &dyn Type,
        mut arg: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Expression> {
        debug_assert!(ty.isScalar());
        debug_assert!(isAllowedInES2Context(ty, context));
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLConstructorScalarCast.cpp\0".as_ptr(),69; @condition b"arg->type().isScalar()\0".as_ptr(); arg.expression_type().isScalar());
        if arg.expression_type().matches(ty) {
            arg.as_pin_mut().node_mut().setPosition(pos);
            return arg;
        }
        arg = ConstantFolder::MakeConstantValueForVariable(pos, arg);
        if arg.node().is::<Literal>() {
            let mut value = arg.node().as_node::<Literal>().value();
            if checkForOutOfRangeLiteralValue(ty, context, value, arg.node().fPosition) {
                value = 0.0;
            }
            return Literal::Make(pos, value, ty).cast(|p| p as *mut dyn Expression);
        }
        if arg.node().is::<Self>() && arg.expression_type().isLiteral() {
            let cast = &mut *(arg.as_ptr() as *mut () as *mut Self);
            let inner = cast.argument_mut().take().unwrap_unchecked();
            return Self::Make(context, pos, ty, inner);
        }
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, ty, Some(arg))))
            .cast(|p| p as *mut dyn Expression)
    }
}
