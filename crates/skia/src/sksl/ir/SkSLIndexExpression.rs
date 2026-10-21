// New translation is UNVERIFIED by explicit user instruction.
use super::{
    SkSLExpression::{Expression, ExpressionData, ExpressionExt},
    SkSLIRNode::*,
    SkSLType::Type,
};
use crate::src::sksl::{
    SkSLContext::Context,
    SkSLOperator::{Operator, OperatorKind, OperatorPrecedence},
    SkSLPosition::Position,
};
use core::{pin::Pin, ptr::NonNull};
unsafe fn borrow_type(ty: &dyn Type) -> NonNull<dyn Type> {
    core::mem::transmute::<NonNull<dyn Type + '_>, NonNull<dyn Type>>(NonNull::from(ty))
}
// Source: src/sksl/ir/SkSLIndexExpression.h:29-29
// Source: src/sksl/ir/SkSLIndexExpression.h:89-90
// Adapted inherited state and contiguous field group preserve original field order.
#[repr(C)]
pub struct IndexExpression {
    base: ExpressionData,
    fBase: core::mem::ManuallyDrop<Option<IRNodeBox<dyn Expression>>>,
    fIndex: core::mem::ManuallyDrop<Option<IRNodeBox<dyn Expression>>>,
}
// Source: src/sksl/ir/SkSLIndexExpression.h:31-31
impl IRNodeType for IndexExpression {
    const kIRNodeKind: i32 = ExpressionKind::kIndex as i32;
}
impl IndexExpression {
    // Source: src/sksl/ir/SkSLIndexExpression.h:33-37
    pub unsafe fn new(
        context: &Context,
        pos: Position,
        base: Option<IRNodeBox<dyn Expression>>,
        index: Option<IRNodeBox<dyn Expression>>,
    ) -> Self {
        let ty = borrow_type(Self::IndexType(
            context,
            base.as_ref().unwrap_unchecked().expression_type(),
        ));
        Self {
            base: ExpressionData::new(pos, ExpressionKind::kIndex, Some(ty)),
            fBase: core::mem::ManuallyDrop::new(base),
            fIndex: core::mem::ManuallyDrop::new(index),
        }
    }
    // Source: src/sksl/ir/SkSLIndexExpression.h:56-58
    pub unsafe fn base_mut(&mut self) -> &mut Option<IRNodeBox<dyn Expression>> {
        &mut self.fBase
    }
    // Source: src/sksl/ir/SkSLIndexExpression.h:60-62
    pub fn base(&self) -> &Option<IRNodeBox<dyn Expression>> {
        &self.fBase
    }
    // Source: src/sksl/ir/SkSLIndexExpression.h:64-66
    pub unsafe fn index_mut(&mut self) -> &mut Option<IRNodeBox<dyn Expression>> {
        &mut self.fIndex
    }
    // Source: src/sksl/ir/SkSLIndexExpression.h:68-70
    pub fn index(&self) -> &Option<IRNodeBox<dyn Expression>> {
        &self.fIndex
    }
}
impl IRNode for IndexExpression {
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
unsafe impl Expression for IndexExpression {
    fn expression_data(&self) -> &ExpressionData {
        &self.base
    }
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base) }
    }
    fn as_expression(&self) -> &dyn Expression {
        self
    }
    // Source: src/sksl/ir/SkSLIndexExpression.h:72-76
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        Some(
            IRNodeBox::new_with(|p: *mut Self| {
                p.write(Self::new_typed(
                    pos,
                    self.base().as_ref().unwrap_unchecked().clone_current(),
                    self.index().as_ref().unwrap_unchecked().clone_current(),
                    borrow_type(self.expression_type()),
                ))
            })
            .cast(|p| p as *mut dyn Expression),
        )
    }
    fn description_with_precedence(&self, p: OperatorPrecedence) -> Vec<u8> {
        description(self, p)
    }
}
impl IndexExpression {
    // Source: src/sksl/ir/SkSLIndexExpression.h:83-87
    unsafe fn new_typed(
        pos: Position,
        base: Option<IRNodeBox<dyn Expression>>,
        index: Option<IRNodeBox<dyn Expression>>,
        ty: NonNull<dyn Type>,
    ) -> Self {
        Self {
            base: ExpressionData::new(pos, ExpressionKind::kIndex, Some(ty)),
            fBase: core::mem::ManuallyDrop::new(base),
            fIndex: core::mem::ManuallyDrop::new(index),
        }
    }
}
// Rust destruction adapter for native reverse member order: index dies before base.
impl Drop for IndexExpression {
    fn drop(&mut self) {
        unsafe {
            core::mem::ManuallyDrop::drop(&mut self.fIndex);
            core::mem::ManuallyDrop::drop(&mut self.fBase);
        }
    }
}
use super::{
    SkSLConstructorArray::ConstructorArray,
    SkSLConstructorCompound::ConstructorCompound,
    SkSLLiteral::Literal,
    SkSLSwizzle::{ComponentArray, Swizzle},
    SkSLTypeReference::TypeReference,
};
use crate::src::sksl::{SkSLConstantFolder::ConstantFolder, SkSLErrorReporter::ErrorReporterExt};
// Source: src/sksl/ir/SkSLIndexExpression.cpp:32-44
unsafe fn index_out_of_range(
    context: &Context,
    pos: Position,
    index: i64,
    base: &dyn Expression,
) -> bool {
    if index >= 0 {
        if base.expression_type().columns() == super::SkSLType::kUnsizedArray {
            return false;
        } else if index < base.expression_type().columns() as i64 {
            return false;
        }
    }
    let mut message = b"index ".to_vec();
    message.extend_from_slice(index.to_string().as_bytes());
    message.extend_from_slice(b" out of range for '");
    message.extend_from_slice(&base.expression_type().displayName());
    message.push(b'\'');
    let mut errors = context.fErrors;
    errors.as_mut().error(pos, &message);
    true
}
impl IndexExpression {
    // Source: src/sksl/ir/SkSLIndexExpression.cpp:46-65
    pub unsafe fn IndexType<'a>(context: &'a Context, ty: &'a dyn Type) -> &'a dyn Type {
        if ty.isMatrix() {
            let b = context.fTypes.as_ref();
            if ty.componentType().matches(&**b.fFloat) {
                match ty.rows() {
                    2 => return &**b.fFloat2,
                    3 => return &**b.fFloat3,
                    4 => return &**b.fFloat4,
                    _ => debug_assert!(false),
                }
            } else if ty.componentType().matches(&**b.fHalf) {
                match ty.rows() {
                    2 => return &**b.fHalf2,
                    3 => return &**b.fHalf3,
                    4 => return &**b.fHalf4,
                    _ => debug_assert!(false),
                }
            }
        }
        ty.componentType()
    }
    // Source: src/sksl/ir/SkSLIndexExpression.cpp:67-105
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        base: IRNodeBox<dyn Expression>,
        mut index: IRNodeBox<dyn Expression>,
    ) -> Option<IRNodeBox<dyn Expression>> {
        if base.node().is::<TypeReference>() {
            let baseType = base.node().as_node::<TypeReference>().value();
            let arraySize =
                super::SkSLType::convertArraySizeExpression(baseType, context, pos, Some(index));
            if arraySize == 0 {
                return None;
            }
            let mut table = context.fSymbolTable.unwrap_unchecked();
            let ty =
                table
                    .as_mut()
                    .addArrayDimension(context, borrow_type(baseType), arraySize as i32);
            return TypeReference::Convert(context, pos, ty)
                .map(|p| p.cast(|p| p as *mut dyn Expression));
        }
        let baseType = base.expression_type();
        if !baseType.isArray() && !baseType.isMatrix() && !baseType.isVector() {
            let mut message = b"expected array, but found '".to_vec();
            message.extend_from_slice(&baseType.displayName());
            message.push(b'\'');
            let mut errors = context.fErrors;
            errors.as_mut().error(base.node().fPosition, &message);
            return None;
        }
        if !index.expression_type().isInteger() {
            index = super::SkSLType::coerceExpression(
                &**context.fTypes.as_ref().fInt,
                Some(index),
                context,
            )?;
        }
        let indexExpr = ConstantFolder::GetConstantValueForVariable(&*index);
        if indexExpr.isIntLiteral() {
            let indexValue = indexExpr.node().as_node::<Literal>().intValue();
            if index_out_of_range(context, index.node().fPosition, indexValue, &*base) {
                return None;
            }
        }
        Some(Self::Make(context, pos, base, index))
    }
    // Source: src/sksl/ir/SkSLIndexExpression.cpp:107-170
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        base: IRNodeBox<dyn Expression>,
        index: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Expression> {
        let baseType = base.expression_type();
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLIndexExpression.cpp\0".as_ptr(),112; @condition b"baseType.isArray() || baseType.isMatrix() || baseType.isVector()\0".as_ptr(); baseType.isArray()||baseType.isMatrix()||baseType.isVector());
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLIndexExpression.cpp\0".as_ptr(),113; @condition b"index->type().isInteger()\0".as_ptr(); index.expression_type().isInteger());
        let indexExpr = ConstantFolder::GetConstantValueForVariable(&*index);
        if indexExpr.isIntLiteral() {
            let mut indexValue = indexExpr.node().as_node::<Literal>().intValue();
            if !index_out_of_range(context, index.node().fPosition, indexValue, &*base) {
                if baseType.isVector() {
                    return Swizzle::Make(
                        context,
                        pos,
                        base,
                        ComponentArray::from_values(&[indexValue as i8]),
                    );
                }
                if baseType.isArray() && !crate::src::sksl::SkSLAnalysis::HasSideEffects(&*base) {
                    let baseExpr = ConstantFolder::GetConstantValueForVariable(&*base);
                    if baseExpr.node().is::<ConstructorArray>() {
                        let arrayCtor = baseExpr.node().as_node::<ConstructorArray>();
                        let arguments = arrayCtor.arguments();
                        debug_assert_eq!(arguments.as_base().size(), baseType.columns());
                        return arguments.as_base().as_slice()[indexValue as usize]
                            .as_ref()
                            .unwrap_unchecked()
                            .clone_at(pos)
                            .unwrap_unchecked();
                    }
                }
                if baseType.isMatrix() && !crate::src::sksl::SkSLAnalysis::HasSideEffects(&*base) {
                    let baseExpr = ConstantFolder::GetConstantValueForVariable(&*base);
                    let vecWidth = baseType.rows();
                    let vecType = baseType.columnType(context);
                    indexValue *= vecWidth as i64;
                    let mut ctorArgs = [0.0; 4];
                    let mut allConstant = true;
                    for slot in 0..vecWidth {
                        let slotVal = baseExpr.getConstantValue((indexValue + slot as i64) as i32);
                        if let Some(slotVal) = slotVal {
                            ctorArgs[slot as usize] = slotVal;
                        } else {
                            allConstant = false;
                            break;
                        }
                    }
                    if allConstant {
                        return ConstructorCompound::MakeFromConstants(
                            context, pos, vecType, &ctorArgs,
                        );
                    }
                }
            }
        }
        IRNodeBox::new_with(|p: *mut Self| {
            p.write(Self::new(context, pos, Some(base), Some(index)))
        })
        .cast(|p| p as *mut dyn Expression)
    }
}
// Source: src/sksl/ir/SkSLIndexExpression.cpp:172-175
fn description(this: &IndexExpression, _p: OperatorPrecedence) -> Vec<u8> {
    unsafe {
        let mut result = this
            .base()
            .as_ref()
            .unwrap_unchecked()
            .description_with_precedence(OperatorPrecedence::kPostfix);
        result.push(b'[');
        result.extend_from_slice(
            &this
                .index()
                .as_ref()
                .unwrap_unchecked()
                .description_with_precedence(OperatorPrecedence::kExpression),
        );
        result.push(b']');
        result
    }
}
