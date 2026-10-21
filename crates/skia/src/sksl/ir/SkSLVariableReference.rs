// New translation: unverified by explicit user instruction.
use super::{
    SkSLExpression::{Expression, ExpressionData, ExpressionExt},
    SkSLIRNode::*,
    SkSLVariable::Variable,
};
use crate::src::sksl::{SkSLOperator::OperatorPrecedence, SkSLPosition::Position};
use core::{pin::Pin, ptr::NonNull};
// Source: src/sksl/ir/SkSLVariableReference.h:25-32
#[repr(i8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariableRefKind {
    kRead,
    kWrite,
    kReadWrite,
    kPointer,
}
// Source: src/sksl/ir/SkSLVariableReference.h:41-41
// Source: src/sksl/ir/SkSLVariableReference.h:79-80
#[repr(C)]
pub struct VariableReference {
    base: ExpressionData,
    fVariable: Option<NonNull<dyn Variable>>,
    fRefKind: VariableRefKind,
}
// Source: src/sksl/ir/SkSLVariableReference.h:43-43
pub type RefKind = VariableRefKind;
// Source: src/sksl/ir/SkSLVariableReference.h:45-45
impl IRNodeType for VariableReference {
    const kIRNodeKind: i32 = ExpressionKind::kVariableReference as i32;
}
impl VariableReference {
    // Source: src/sksl/ir/SkSLVariableReference.h:51-56
    pub unsafe fn Make(
        pos: Position,
        variable: NonNull<dyn Variable>,
        refKind: RefKind,
    ) -> IRNodeBox<dyn Expression> {
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, variable, refKind)))
            .cast(|p| p as *mut dyn Expression)
    }
    // Source: src/sksl/ir/SkSLVariableReference.h:61-63
    pub fn variable(&self) -> Option<NonNull<dyn Variable>> {
        self.fVariable
    }
    // Source: src/sksl/ir/SkSLVariableReference.h:65-67
    pub fn refKind(&self) -> RefKind {
        self.fRefKind
    }
}
impl IRNode for VariableReference {
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
impl VariableReference {
    // Source: src/sksl/ir/SkSLVariableReference.cpp:14-19
    pub unsafe fn new(pos: Position, variable: NonNull<dyn Variable>, refKind: RefKind) -> Self {
        let ty = NonNull::from(variable.as_ref().symbol_type());
        Self {
            base: ExpressionData::new(pos, ExpressionKind::kVariableReference, Some(ty)),
            fVariable: Some(variable),
            fRefKind: refKind,
        }
    }
}
unsafe impl Expression for VariableReference {
    fn expression_data(&self) -> &ExpressionData {
        &self.base
    }
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base) }
    }
    fn as_expression(&self) -> &dyn Expression {
        self
    }
    // Source: src/sksl/ir/SkSLVariableReference.h:72-74
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        Some(Self::Make(
            pos,
            self.variable().unwrap_unchecked(),
            self.refKind(),
        ))
    }
    // Source: src/sksl/ir/SkSLVariableReference.cpp:21-23
    fn description_with_precedence(&self, _p: OperatorPrecedence) -> Vec<u8> {
        unsafe { self.variable().unwrap_unchecked().as_ref() }
            .name()
            .to_vec()
    }
}
impl VariableReference {
    // Source: src/sksl/ir/SkSLVariableReference.cpp:25-27
    pub fn setRefKind(&mut self, refKind: RefKind) {
        self.fRefKind = refKind;
    }
    // Source: src/sksl/ir/SkSLVariableReference.cpp:29-31
    /// Safety: nullable pointer is preserved; callers restore a live nonnull Variable before any dereferencing native operation.
    pub unsafe fn setVariable(&mut self, variable: Option<NonNull<dyn Variable>>) {
        self.fVariable = variable;
    }
}
