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
// Source: src/sksl/ir/SkSLFieldAccess.h:29-34
#[repr(i8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldAccessOwnerKind {
    kDefault,
    kAnonymousInterfaceBlock,
}
// Source: src/sksl/ir/SkSLFieldAccess.h:39-39
// Source: src/sksl/ir/SkSLFieldAccess.h:93-95
// Adapted inherited state and contiguous field group preserve original field order.
#[repr(C)]
pub struct FieldAccess {
    base: ExpressionData,
    fFieldIndex: i32,
    fOwnerKind: FieldAccessOwnerKind,
    fBase: Option<IRNodeBox<dyn Expression>>,
}
// Source: src/sksl/ir/SkSLFieldAccess.h:43-43
impl IRNodeType for FieldAccess {
    const kIRNodeKind: i32 = ExpressionKind::kFieldAccess as i32;
}
// Source: src/sksl/ir/SkSLFieldAccess.h:41-41
pub type OwnerKind = FieldAccessOwnerKind;
impl FieldAccess {
    // Source: src/sksl/ir/SkSLFieldAccess.h:45-50
    /// Safety: native nonnull-child, field-index, Type/name lifetime and pool lease hold.
    pub unsafe fn new(
        pos: Position,
        base: Option<IRNodeBox<dyn Expression>>,
        fieldIndex: i32,
        ownerKind: OwnerKind,
    ) -> Self {
        let ty =
            base.as_ref().unwrap_unchecked().expression_type().fields()[fieldIndex as usize].fType;
        Self {
            base: ExpressionData::new(pos, ExpressionKind::kFieldAccess, ty),
            fFieldIndex: fieldIndex,
            fOwnerKind: ownerKind,
            fBase: base,
        }
    }
    // Source: src/sksl/ir/SkSLFieldAccess.h:65-67
    pub unsafe fn base_mut(&mut self) -> &mut Option<IRNodeBox<dyn Expression>> {
        &mut self.fBase
    }
    // Source: src/sksl/ir/SkSLFieldAccess.h:69-71
    pub fn base(&self) -> &Option<IRNodeBox<dyn Expression>> {
        &self.fBase
    }
    // Source: src/sksl/ir/SkSLFieldAccess.h:73-75
    pub fn fieldIndex(&self) -> i32 {
        self.fFieldIndex
    }
    // Source: src/sksl/ir/SkSLFieldAccess.h:77-79
    pub fn ownerKind(&self) -> OwnerKind {
        self.fOwnerKind
    }
}
impl IRNode for FieldAccess {
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
unsafe impl Expression for FieldAccess {
    fn expression_data(&self) -> &ExpressionData {
        &self.base
    }
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base) }
    }
    fn as_expression(&self) -> &dyn Expression {
        self
    }
    // Source: src/sksl/ir/SkSLFieldAccess.h:81-86
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        Some(
            IRNodeBox::new_with(|p: *mut Self| {
                p.write(Self::new(
                    pos,
                    self.base().as_ref().unwrap_unchecked().clone_current(),
                    self.fieldIndex(),
                    self.ownerKind(),
                ))
            })
            .cast(|p| p as *mut dyn Expression),
        )
    }
    fn description_with_precedence(&self, p: OperatorPrecedence) -> Vec<u8> {
        description(self, p)
    }
}
use super::{
    SkSLConstructorStruct::ConstructorStruct, SkSLFunctionDeclaration::FunctionDeclaration,
    SkSLMethodReference::MethodReference, SkSLSetting::Setting,
};
use crate::src::sksl::{SkSLConstantFolder::ConstantFolder, SkSLErrorReporter::ErrorReporterExt};
impl FieldAccess {
    // Source: src/sksl/ir/SkSLFieldAccess.cpp:31-63
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        base: IRNodeBox<dyn Expression>,
        field: &[u8],
    ) -> Option<IRNodeBox<dyn Expression>> {
        let baseType = base.expression_type();
        if baseType.isEffectChild() {
            let mut methodName = b"$".to_vec();
            methodName.extend_from_slice(field);
            let result = context
                .fSymbolTable
                .unwrap_unchecked()
                .as_ref()
                .find(&methodName);
            if let Some(result) = result {
                if result.as_ref().node().is::<FunctionDeclaration>() {
                    let chain = NonNull::new_unchecked(
                        result.as_ptr() as *mut () as *mut FunctionDeclaration
                    );
                    return Some(
                        IRNodeBox::new_with(|p: *mut MethodReference| {
                            p.write(MethodReference::new(context, pos, Some(base), Some(chain)))
                        })
                        .cast(|p| p as *mut dyn Expression),
                    );
                }
            }
            let mut message = b"type '".to_vec();
            message.extend_from_slice(&baseType.displayName());
            message.extend_from_slice(b"' has no method named '");
            message.extend_from_slice(field);
            message.push(b'\'');
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &message);
            return None;
        }
        if baseType.isStruct() {
            for (i, f) in baseType.fields().iter().enumerate() {
                if f.fName.as_ref() == field {
                    return Some(Self::Make(
                        context,
                        pos,
                        base,
                        i as i32,
                        OwnerKind::kDefault,
                    ));
                }
            }
        }
        if baseType.matches(&**context.fTypes.as_ref().fSkCaps) {
            return Setting::Convert(context, pos, field);
        }
        let mut message = b"type '".to_vec();
        message.extend_from_slice(&baseType.displayName());
        message.extend_from_slice(b"' does not have a field named '");
        message.extend_from_slice(field);
        message.push(b'\'');
        let mut errors = context.fErrors;
        errors.as_mut().error(pos, &message);
        None
    }
}
// Source: src/sksl/ir/SkSLFieldAccess.cpp:65-82
unsafe fn extract_field(
    pos: Position,
    ctor: &ConstructorStruct,
    fieldIndex: i32,
) -> Option<IRNodeBox<dyn Expression>> {
    let args = ctor.arguments();
    let numFields = args.as_base().size();
    for index in 0..numFields {
        if fieldIndex == index {
            continue;
        }
        if crate::src::sksl::SkSLAnalysis::HasSideEffects(
            &**args.as_base().as_slice()[index as usize]
                .as_ref()
                .unwrap_unchecked(),
        ) {
            return None;
        }
    }
    args.as_base().as_slice()[fieldIndex as usize]
        .as_ref()
        .unwrap_unchecked()
        .clone_at(pos)
}
impl FieldAccess {
    // Source: src/sksl/ir/SkSLFieldAccess.cpp:84-103
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        base: IRNodeBox<dyn Expression>,
        fieldIndex: i32,
        ownerKind: OwnerKind,
    ) -> IRNodeBox<dyn Expression> {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLFieldAccess.cpp\0".as_ptr(),89; @condition b"base->type().isStruct()\0".as_ptr(); base.expression_type().isStruct());
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLFieldAccess.cpp\0".as_ptr(),90; @condition b"fieldIndex >= 0\0".as_ptr(); fieldIndex>=0);
        debug_assert!(fieldIndex < base.expression_type().fields().len() as i32);
        let expr = ConstantFolder::GetConstantValueForVariable(&*base);
        if expr.node().is::<ConstructorStruct>() {
            if let Some(field) =
                extract_field(pos, expr.node().as_node::<ConstructorStruct>(), fieldIndex)
            {
                return field;
            }
        }
        IRNodeBox::new_with(|p: *mut Self| {
            p.write(Self::new(pos, Some(base), fieldIndex, ownerKind))
        })
        .cast(|p| p as *mut dyn Expression)
    }
    // Source: src/sksl/ir/SkSLFieldAccess.cpp:105-114
    pub unsafe fn initialSlot(&self) -> usize {
        let fields = self
            .base()
            .as_ref()
            .unwrap_unchecked()
            .expression_type()
            .fields();
        let fieldIndex = self.fieldIndex();
        let mut slot = 0usize;
        for index in 0..fieldIndex {
            slot = slot.wrapping_add(
                fields[index as usize]
                    .fType
                    .unwrap_unchecked()
                    .as_ref()
                    .slotCount(),
            );
        }
        slot
    }
}
// Source: src/sksl/ir/SkSLFieldAccess.cpp:116-122
fn description(this: &FieldAccess, _p: OperatorPrecedence) -> Vec<u8> {
    unsafe {
        let base = this.base().as_ref().unwrap_unchecked();
        let mut f = base.description_with_precedence(OperatorPrecedence::kPostfix);
        if !f.is_empty() {
            f.push(b'.');
        }
        f.extend_from_slice(
            base.expression_type().fields()[this.fieldIndex() as usize]
                .fName
                .as_ref(),
        );
        f
    }
}
