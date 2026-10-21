use super::{
    SkSLExpression::{ComparisonResult, Expression, ExpressionData, ExpressionExt, Kind},
    SkSLIRNode::IRNodeBox,
    SkSLType::Type,
};
use crate::src::sksl::{
    SkSLDefines::ExpressionArray, SkSLOperator::OperatorPrecedence, SkSLPosition::Position,
};
use core::{pin::Pin, ptr::NonNull};
// Source: src/sksl/ir/SkSLConstructor.h:31-31
// Rust stores only the original abstract base subobject. This cannot be published
// as a concrete Expression; a real subclass must supply clone and virtual dispatch.
#[repr(C)]
pub struct AnyConstructorData {
    pub(crate) base: ExpressionData,
}
impl AnyConstructorData {
    // Source: src/sksl/ir/SkSLConstructor.h:33-34
    pub unsafe fn new(pos: Position, kind: Kind, ty: NonNull<dyn Type>) -> Self {
        Self {
            base: ExpressionData::new(pos, kind, Some(ty)),
        }
    }
}
// Source: src/sksl/ir/SkSLConstructor.h:36-37
// Adapted pure-virtual SkSpan protocol. No concrete constructor or empty
// argument-list default is supplied; actual derived storage must be projected.
pub unsafe trait AnyConstructor: Expression {
    fn argumentSpan_mut(self: Pin<&mut Self>) -> &mut [Option<IRNodeBox<dyn Expression>>];
    fn argumentSpan(&self) -> &[Option<IRNodeBox<dyn Expression>>];
    // Source: src/sksl/ir/SkSLConstructor.h:41-43
    fn componentType(&self) -> &dyn Type {
        self.expression_type().componentType()
    }
    // Source: src/sksl/ir/SkSLConstructor.h:45-45
    fn constructor_supportsConstantValues(&self) -> bool {
        true
    }
    // Dynamic calls to Expression's overrides are preserved by the algorithm helpers below.
    fn constructor_getConstantValue(&self, n: i32) -> Option<f64> {
        getConstantValue(self, n)
    }
    fn constructor_compareConstant(&self, other: &dyn Expression) -> ComparisonResult {
        compareConstant(self, other)
    }
    fn constructor_description(&self, p: OperatorPrecedence) -> Vec<u8> {
        description(self, p)
    }
}
// Source: src/sksl/ir/SkSLConstructor.h:57-57
// Source: src/sksl/ir/SkSLConstructor.h:81-81
// Adapted abstract inheritance only; nullable unique_ptr is the actual sole argument owner.
#[repr(C)]
pub struct SingleArgumentConstructorData {
    pub(crate) base: AnyConstructorData,
    fArgument: Option<IRNodeBox<dyn Expression>>,
}
impl SingleArgumentConstructorData {
    // Source: src/sksl/ir/SkSLConstructor.h:59-62
    pub unsafe fn new(
        pos: Position,
        kind: Kind,
        ty: NonNull<dyn Type>,
        argument: Option<IRNodeBox<dyn Expression>>,
    ) -> Self {
        Self {
            base: AnyConstructorData::new(pos, kind, ty),
            fArgument: argument,
        }
    }
    // Source: src/sksl/ir/SkSLConstructor.h:64-66
    pub fn argument_mut(&mut self) -> &mut Option<IRNodeBox<dyn Expression>> {
        &mut self.fArgument
    }
    // Source: src/sksl/ir/SkSLConstructor.h:68-70
    pub fn argument(&self) -> &Option<IRNodeBox<dyn Expression>> {
        &self.fArgument
    }
    // Source: src/sksl/ir/SkSLConstructor.h:72-74
    pub fn argumentSpan_mut(&mut self) -> &mut [Option<IRNodeBox<dyn Expression>>] {
        core::slice::from_mut(&mut self.fArgument)
    }
    // Source: src/sksl/ir/SkSLConstructor.h:76-78
    pub fn argumentSpan(&self) -> &[Option<IRNodeBox<dyn Expression>>] {
        core::slice::from_ref(&self.fArgument)
    }
}
// Source: src/sksl/ir/SkSLConstructor.h:89-89
// Source: src/sksl/ir/SkSLConstructor.h:113-113
// Adapted native inline STArray must remain pinned through every move/return.
#[repr(C)]
pub struct MultiArgumentConstructorData {
    pub(crate) base: AnyConstructorData,
    fArguments: Pin<Box<ExpressionArray>>,
}
impl MultiArgumentConstructorData {
    // Source: src/sksl/ir/SkSLConstructor.h:91-94
    pub unsafe fn new(
        pos: Position,
        kind: Kind,
        ty: NonNull<dyn Type>,
        arguments: Pin<Box<ExpressionArray>>,
    ) -> Self {
        Self {
            base: AnyConstructorData::new(pos, kind, ty),
            fArguments: arguments,
        }
    }
    // Source: src/sksl/ir/SkSLConstructor.h:96-98
    pub fn arguments_mut(&mut self) -> Pin<&mut ExpressionArray> {
        self.fArguments.as_mut()
    }
    // Source: src/sksl/ir/SkSLConstructor.h:100-102
    pub fn arguments(&self) -> &ExpressionArray {
        &self.fArguments
    }
    // Source: src/sksl/ir/SkSLConstructor.h:104-106
    pub fn argumentSpan_mut(&mut self) -> &mut [Option<IRNodeBox<dyn Expression>>] {
        debug_assert!(!self.fArguments.as_base().empty());
        self.fArguments.as_mut().as_base_mut().as_mut_slice()
    }
    // Source: src/sksl/ir/SkSLConstructor.h:108-110
    pub fn argumentSpan(&self) -> &[Option<IRNodeBox<dyn Expression>>] {
        debug_assert!(!self.fArguments.as_base().empty());
        self.fArguments.as_base().as_slice()
    }
}
use super::{
    SkSLConstructorArray::ConstructorArray, SkSLConstructorCompound::ConstructorCompound,
    SkSLConstructorCompoundCast::ConstructorCompoundCast,
    SkSLConstructorDiagonalMatrix::ConstructorDiagonalMatrix,
    SkSLConstructorMatrixResize::ConstructorMatrixResize,
    SkSLConstructorScalarCast::ConstructorScalarCast, SkSLConstructorSplat::ConstructorSplat,
    SkSLConstructorStruct::ConstructorStruct,
};
use crate::src::sksl::{SkSLContext::Context, SkSLErrorReporter::ErrorReporterExt};
// New translation is unverified. Native factory dependencies are retained directly.
// Source: src/sksl/ir/SkSLConstructor.cpp:28-148
unsafe fn convert_compound_constructor(
    context: &Context,
    pos: Position,
    ty: &dyn Type,
    mut args: Pin<Box<ExpressionArray>>,
) -> Option<IRNodeBox<dyn Expression>> {
    debug_assert!(ty.isVector() || ty.isMatrix());
    if args.as_base().size() == 1 {
        let argument = args.as_base().as_slice()[0].as_ref().unwrap_unchecked();
        let argumentType = argument.expression_type();
        if ty.isVector()
            && argumentType.isVector()
            && argumentType.componentType().matches(ty.componentType())
            && argumentType.slotCount() > ty.slotCount()
        {
            let hint = match ty.slotCount() {
                2 => b"; use '.xy' instead".as_slice(),
                3 => b"; use '.xyz' instead",
                _ => {
                    crate::SkDEBUGFAIL!(@source b"src/sksl/ir/SkSLConstructor.cpp\0".as_ptr(),49; b"unexpected slicing cast\0".as_ptr());
                    b""
                }
            };
            let mut message = b"'".to_vec();
            message.extend_from_slice(&argumentType.displayName());
            message.extend_from_slice(b"' is not a valid parameter to '");
            message.extend_from_slice(&ty.displayName());
            message.extend_from_slice(b"' constructor");
            message.extend_from_slice(hint);
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &message);
            return None;
        }
        if argumentType.isScalar() {
            let typecast = ConstructorScalarCast::Convert(context, pos, ty.componentType(), args)?;
            return Some(if ty.isMatrix() {
                ConstructorDiagonalMatrix::Make(context, pos, ty, typecast)
            } else {
                ConstructorSplat::Make(context, pos, ty, typecast)
            });
        } else if argumentType.isVector() {
            if ty.isVector() && argumentType.columns() == ty.columns() {
                return Some(ConstructorCompoundCast::Make(
                    context,
                    pos,
                    ty,
                    args.as_mut().as_base_mut().as_mut_slice()[0]
                        .take()
                        .unwrap_unchecked(),
                ));
            }
        } else if argumentType.isMatrix() {
            if ty.isMatrix() {
                let typecastType = ty.componentType().toCompound(
                    context,
                    argumentType.columns(),
                    argumentType.rows(),
                );
                let argument = &mut args.as_mut().as_base_mut().as_mut_slice()[0];
                *argument = Some(ConstructorCompoundCast::Make(
                    context,
                    pos,
                    typecastType,
                    argument.take().unwrap_unchecked(),
                ));
                return Some(ConstructorMatrixResize::Make(
                    context,
                    pos,
                    ty,
                    argument.take().unwrap_unchecked(),
                ));
            }
            if ty.isVector() && ty.columns() == 4 && argumentType.slotCount() == 4 {
                // Compound types are interned in Context; detach the overly narrow borrow
                // inferred through argumentType before moving the expression array.
                let vectorType: &'static dyn Type =
                    core::mem::transmute(argumentType.componentType().toCompound(context, 4, 1));
                let vecCtor = ConstructorCompound::Make(context, pos, vectorType, args);
                return Some(ConstructorCompoundCast::Make(context, pos, ty, vecCtor));
            }
        }
    }
    let expected = ty.rows() * ty.columns();
    let mut actual = 0;
    for arg in args.as_mut().as_base_mut().as_mut_slice() {
        let argType = arg.as_ref().unwrap_unchecked().expression_type();
        if !argType.isScalar() && !argType.isVector() {
            let mut message = b"'".to_vec();
            message.extend_from_slice(&argType.displayName());
            message.extend_from_slice(b"' is not a valid parameter to '");
            message.extend_from_slice(&ty.displayName());
            message.extend_from_slice(b"' constructor");
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &message);
            return None;
        }
        let ctorType = ty.componentType().toCompound(context, argType.columns(), 1);
        let mut ctorArg = ExpressionArray::new();
        ctorArg.as_mut().as_base_mut().push_back(arg.take());
        *arg = Convert(context, pos, ctorType, ctorArg);
        if arg.is_none() {
            return None;
        }
        actual += ctorType.columns();
    }
    if actual != expected {
        let mut message = b"invalid arguments to '".to_vec();
        message.extend_from_slice(&ty.displayName());
        message.extend_from_slice(b"' constructor (expected ");
        message.extend_from_slice(expected.to_string().as_bytes());
        message.extend_from_slice(b" scalars, but found ");
        message.extend_from_slice(actual.to_string().as_bytes());
        message.push(b')');
        let mut errors = context.fErrors;
        errors.as_mut().error(pos, &message);
        return None;
    }
    Some(ConstructorCompound::Make(context, pos, ty, args))
}
// Source: src/sksl/ir/SkSLConstructor.cpp:150-176
pub unsafe fn Convert(
    context: &Context,
    pos: Position,
    ty: &dyn Type,
    mut args: Pin<Box<ExpressionArray>>,
) -> Option<IRNodeBox<dyn Expression>> {
    if args.as_base().size() == 1
        && args.as_base().as_slice()[0]
            .as_ref()
            .unwrap_unchecked()
            .expression_type()
            .matches(ty)
        && !ty.componentType().isOpaque()
    {
        let mut argument = args.as_mut().as_base_mut().as_mut_slice()[0]
            .take()
            .unwrap_unchecked();
        argument.as_pin_mut().node_mut().setPosition(pos);
        return Some(argument);
    }
    if ty.isScalar() {
        return ConstructorScalarCast::Convert(context, pos, ty, args);
    }
    if ty.isVector() || ty.isMatrix() {
        return convert_compound_constructor(context, pos, ty, args);
    }
    if ty.isArray() && ty.columns() > 0 {
        return ConstructorArray::Convert(context, pos, ty, args);
    }
    if ty.isStruct() && !ty.fields().is_empty() {
        return ConstructorStruct::Convert(context, pos, ty, args);
    }
    let mut message = b"cannot construct '".to_vec();
    message.extend_from_slice(&ty.displayName());
    message.push(b'\'');
    let mut errors = context.fErrors;
    errors.as_mut().error(pos, &message);
    None
}
// Source: src/sksl/ir/SkSLConstructor.cpp:178-190
pub fn getConstantValue<T: AnyConstructor + ?Sized>(this: &T, mut n: i32) -> Option<f64> {
    debug_assert!(n >= 0 && (n as usize) < this.expression_type().slotCount());
    for arg in this.argumentSpan() {
        let arg = unsafe { arg.as_ref().unwrap_unchecked() };
        let argSlots = arg.expression_type().slotCount() as i32;
        if n < argSlots {
            return arg.getConstantValue(n);
        }
        n -= argSlots;
    }
    crate::SkDEBUGFAIL!(@source b"src/sksl/ir/SkSLConstructor.cpp\0".as_ptr(),188; b"argument-list slot count doesn't match constructor-type slot count\0".as_ptr());
    None
}
// Source: src/sksl/ir/SkSLConstructor.cpp:192-216
pub fn compareConstant<T: AnyConstructor + ?Sized>(
    this: &T,
    other: &dyn Expression,
) -> ComparisonResult {
    debug_assert_eq!(
        this.expression_type().slotCount(),
        other.expression_type().slotCount()
    );
    if !other.supportsConstantValues() {
        return ComparisonResult::kUnknown;
    }
    let exprs = this.expression_type().slotCount() as i32;
    for n in 0..exprs {
        let Some(left) = this.getConstantValue(n) else {
            return ComparisonResult::kUnknown;
        };
        let Some(right) = other.getConstantValue(n) else {
            return ComparisonResult::kUnknown;
        };
        if left != right {
            return ComparisonResult::kNotEqual;
        }
    }
    ComparisonResult::kEqual
}
// Source: src/sksl/ir/SkSLConstructor.cpp:218-221
pub unsafe fn asAnyConstructor_mut(expr: Pin<&mut dyn Expression>) -> Pin<&mut dyn AnyConstructor> {
    debug_assert!(expr.as_ref().get_ref().isAnyConstructor());
    expr.constructor_view_mut()
}
// Source: src/sksl/ir/SkSLConstructor.cpp:223-226
pub unsafe fn asAnyConstructor(expr: &dyn Expression) -> &dyn AnyConstructor {
    debug_assert!(expr.isAnyConstructor());
    expr.constructor_view()
}
// Source: src/sksl/ir/SkSLConstructor.cpp:228-237
pub fn description<T: AnyConstructor + ?Sized>(this: &T, _p: OperatorPrecedence) -> Vec<u8> {
    let mut result = this.expression_type().description();
    result.push(b'(');
    let mut separator = crate::src::sksl::SkSLString::Separator();
    for arg in this.argumentSpan() {
        result.extend_from_slice(separator());
        result.extend_from_slice(
            &unsafe { arg.as_ref().unwrap_unchecked() }
                .description_with_precedence(OperatorPrecedence::kSequence),
        );
    }
    result.push(b')');
    result
}
