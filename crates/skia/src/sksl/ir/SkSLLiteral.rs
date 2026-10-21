// Source: src/sksl/ir/SkSLLiteral.h:28-28
use super::{
    SkSLExpression::{ComparisonResult, Expression, ExpressionData, ExpressionExt},
    SkSLIRNode::{ExpressionKind, IRNode, IRNodeBox, IRNodeData, IRNodeType},
    SkSLType::Type,
};
pub use crate::src::sksl::SkSLOperator::OperatorPrecedence;
use crate::src::sksl::{SkSLDefines::SKSL_INT, SkSLPosition::Position, SkSLString};
use core::{pin::Pin, ptr::NonNull};
// Source: src/sksl/ir/SkSLLiteral.h:34-34
// Source: src/sksl/ir/SkSLLiteral.h:136-136
#[repr(C)]
pub struct Literal {
    base: ExpressionData,
    fValue: f64,
}
// Source: src/sksl/ir/SkSLLiteral.h:36-36
impl IRNodeType for Literal {
    const kIRNodeKind: i32 = ExpressionKind::kLiteral as i32;
}
impl Literal {
    // Source: src/sksl/ir/SkSLLiteral.h:38-40
    /// Safety: actual Type backing outlives every access to this Literal.
    pub unsafe fn new(pos: Position, value: f64, ty: &dyn Type) -> Self {
        Self {
            base: ExpressionData::new(pos, ExpressionKind::kLiteral, Some(borrow_type(ty))),
            fValue: value,
        }
    }
    // Source: src/sksl/ir/SkSLLiteral.h:43-45
    pub unsafe fn MakeFloatContext(
        context: &crate::src::sksl::SkSLContext::Context,
        pos: Position,
        value: f32,
    ) -> IRNodeBox<Literal> {
        let ty = &**context.fTypes.as_ref().fFloatLiteral;
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, value as f64, ty)))
    }
    // Source: src/sksl/ir/SkSLLiteral.h:48-51
    /// Safety: original borrowed Type and current-pool destruction lease apply.
    pub unsafe fn MakeFloat(pos: Position, value: f32, ty: &dyn Type) -> IRNodeBox<Literal> {
        debug_assert!(ty.isFloat());
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, value as f64, ty)))
    }
    // Source: src/sksl/ir/SkSLLiteral.h:54-56
    pub unsafe fn MakeIntContext(
        context: &crate::src::sksl::SkSLContext::Context,
        pos: Position,
        value: SKSL_INT,
    ) -> IRNodeBox<Literal> {
        let ty = &**context.fTypes.as_ref().fIntLiteral;
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, value as f64, ty)))
    }
    // Source: src/sksl/ir/SkSLLiteral.h:59-66
    /// Safety: original borrowed Type and current-pool destruction lease apply.
    pub unsafe fn MakeInt(pos: Position, value: SKSL_INT, ty: &dyn Type) -> IRNodeBox<Literal> {
        debug_assert!(ty.isInteger());
        debug_assert!(value as f64 >= ty.minimumValue());
        debug_assert!(value as f64 <= ty.maximumValue());
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, value as f64, ty)))
    }
    // Source: src/sksl/ir/SkSLLiteral.h:69-71
    pub unsafe fn MakeBoolContext(
        context: &crate::src::sksl::SkSLContext::Context,
        pos: Position,
        value: bool,
    ) -> IRNodeBox<Literal> {
        let ty = &**context.fTypes.as_ref().fBool;
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, u8::from(value) as f64, ty)))
    }
    // Source: src/sksl/ir/SkSLLiteral.h:75-78
    /// Safety: original borrowed Type and current-pool destruction lease apply.
    pub unsafe fn MakeBool(pos: Position, value: bool, ty: &dyn Type) -> IRNodeBox<Literal> {
        debug_assert!(ty.isBoolean());
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, u8::from(value) as f64, ty)))
    }
    // Source: src/sksl/ir/SkSLLiteral.h:81-90
    /// Safety: original borrowed Type/pool lease; native double→int64 conversions
    /// must be finite and representable whenever the integer branch is selected.
    pub unsafe fn Make(pos: Position, value: f64, ty: &dyn Type) -> IRNodeBox<Literal> {
        if ty.isFloat() {
            return Self::MakeFloat(pos, value as f32, ty);
        }
        if ty.isInteger() {
            return Self::MakeInt(pos, native_int(value), ty);
        }
        debug_assert!(ty.isBoolean());
        Self::MakeBool(pos, value != 0.0, ty)
    }
    // Source: src/sksl/ir/SkSLLiteral.h:92-95
    pub fn floatValue(&self) -> f32 {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLLiteral.h\0".as_ptr(),93; @condition b"this->type().isFloat()\0".as_ptr(); self.expression_type().isFloat());
        self.fValue as f32
    }
    // Source: src/sksl/ir/SkSLLiteral.h:97-100
    pub fn intValue(&self) -> SKSL_INT {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLLiteral.h\0".as_ptr(),98; @condition b"this->type().isInteger()\0".as_ptr(); self.expression_type().isInteger());
        native_int(self.fValue)
    }
    // Source: src/sksl/ir/SkSLLiteral.h:102-105
    pub fn boolValue(&self) -> SKSL_INT {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLLiteral.h\0".as_ptr(),103; @condition b"this->type().isBoolean()\0".as_ptr(); self.expression_type().isBoolean());
        (self.fValue != 0.0) as SKSL_INT
    }
    // Source: src/sksl/ir/SkSLLiteral.h:107-109
    pub fn value(&self) -> f64 {
        self.fValue
    }
}
// Inheritance projections only; every pointer refers to this same allocation.
impl IRNode for Literal {
    fn description(&self) -> Vec<u8> {
        self.expression_description()
    }
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|v| &mut v.base.base) }
    }
}
unsafe impl Expression for Literal {
    fn expression_data(&self) -> &ExpressionData {
        &self.base
    }
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData> {
        unsafe { self.map_unchecked_mut(|v| &mut v.base) }
    }
    fn as_expression(&self) -> &dyn Expression {
        self
    }
    // Source: src/sksl/ir/SkSLLiteral.h:113-120
    fn compareConstant(&self, other: &dyn Expression) -> ComparisonResult {
        if !other.node().is::<Literal>()
            || self.expression_type().numberKind() != other.expression_type().numberKind()
        {
            return ComparisonResult::kUnknown;
        }
        if self.value() == unsafe { other.node().as_node::<Literal>() }.value() {
            ComparisonResult::kEqual
        } else {
            ComparisonResult::kNotEqual
        }
    }
    // Source: src/sksl/ir/SkSLLiteral.h:122-124
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        Some(
            IRNodeBox::new_with(|p: *mut Self| {
                p.write(Self::new(pos, self.value(), self.expression_type()))
            })
            .cast(|p| p as *mut dyn Expression),
        )
    }
    // Source: src/sksl/ir/SkSLLiteral.h:126-128
    fn supportsConstantValues(&self) -> bool {
        true
    }
    // Source: src/sksl/ir/SkSLLiteral.h:130-133
    fn getConstantValue(&self, n: i32) -> Option<f64> {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLLiteral.h\0".as_ptr(),131; @condition b"n == 0\0".as_ptr(); n == 0);
        Some(self.fValue)
    }
    // Source: src/sksl/ir/SkSLLiteral.cpp:13-21
    fn description_with_precedence(&self, _p: OperatorPrecedence) -> Vec<u8> {
        if self.expression_type().isBoolean() {
            return if self.fValue != 0.0 {
                b"true".to_vec()
            } else {
                b"false".to_vec()
            };
        }
        if self.expression_type().isInteger() {
            return self.intValue().to_string().into_bytes();
        }
        SkSLString::to_string_float(self.floatValue())
    }
}
// Explicit native valid-domain adapter: Rust saturating casts must never invent
// results for native undefined nonfinite/out-of-int64 floating conversions.
fn native_int(v: f64) -> i64 {
    if !v.is_finite() || v < -9223372036854775808.0 || v >= 9223372036854775808.0 {
        std::process::abort();
    }
    v as i64
}
unsafe fn borrow_type(t: &dyn Type) -> NonNull<dyn Type> {
    core::mem::transmute::<NonNull<dyn Type + '_>, NonNull<dyn Type>>(NonNull::from(t))
}
