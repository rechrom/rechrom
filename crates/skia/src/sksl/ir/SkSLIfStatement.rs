// UNVERIFIED: user explicitly prohibited all validation.
use super::{
    SkSLExpression::{Expression, ExpressionExt},
    SkSLIRNode::*,
    SkSLStatement::{Kind, Statement, StatementData},
    SkSLSymbolTable::SymbolTable,
    SkSLType::Type,
};
use crate::src::sksl::{
    SkSLAnalysis,
    SkSLContext::Context,
    SkSLDefines::{ExpressionArray, StatementArray, SKSL_INT},
    SkSLErrorReporter::ErrorReporterExt,
    SkSLPosition::{ForLoopPositions, Position},
    SkSLProgramSettings::ProgramConfig,
};
use core::{mem::ManuallyDrop, pin::Pin, ptr::NonNull};
type ExprSlot = Option<IRNodeBox<dyn Expression>>;
type StmtSlot = Option<IRNodeBox<dyn Statement>>;
// Source: src/sksl/ir/SkSLIfStatement.h:27-27
// Source: src/sksl/ir/SkSLIfStatement.h:54-54
// Source: src/sksl/ir/SkSLIfStatement.h:62-62
// Source: src/sksl/ir/SkSLIfStatement.h:70-70
#[repr(C)]
pub struct IfStatement {
    base: StatementData,
    fTest: ManuallyDrop<ExprSlot>,
    fIfTrue: ManuallyDrop<StmtSlot>,
    fIfFalse: ManuallyDrop<StmtSlot>,
}
// Source: src/sksl/ir/SkSLIfStatement.h:29-29
impl IRNodeType for IfStatement {
    const kIRNodeKind: i32 = Kind::kIf as i32;
}
impl IfStatement {
    // Source: src/sksl/ir/SkSLIfStatement.h:31-36
    pub unsafe fn new(
        pos: Position,
        fTest: ExprSlot,
        fIfTrue: StmtSlot,
        fIfFalse: StmtSlot,
    ) -> Self {
        Self {
            base: StatementData::new(pos, Kind::kIf),
            fTest: ManuallyDrop::new(fTest),
            fIfTrue: ManuallyDrop::new(fIfTrue),
            fIfFalse: ManuallyDrop::new(fIfFalse),
        }
    }
    // Source: src/sksl/ir/SkSLIfStatement.h:53-55
    pub unsafe fn test_mut(&mut self) -> &mut ExprSlot {
        &mut self.fTest
    }
    // Source: src/sksl/ir/SkSLIfStatement.h:57-59
    pub fn test(&self) -> &ExprSlot {
        &self.fTest
    }
    // Source: src/sksl/ir/SkSLIfStatement.h:61-63
    pub unsafe fn ifTrue_mut(&mut self) -> &mut StmtSlot {
        &mut self.fIfTrue
    }
    // Source: src/sksl/ir/SkSLIfStatement.h:65-67
    pub fn ifTrue(&self) -> &StmtSlot {
        &self.fIfTrue
    }
    // Source: src/sksl/ir/SkSLIfStatement.h:69-71
    pub unsafe fn ifFalse_mut(&mut self) -> &mut StmtSlot {
        &mut self.fIfFalse
    }
    // Source: src/sksl/ir/SkSLIfStatement.h:73-75
    pub fn ifFalse(&self) -> &StmtSlot {
        &self.fIfFalse
    }
}
// Native reverse-member destruction, including statements before their symbol owner.
impl Drop for IfStatement {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fIfFalse);
            ManuallyDrop::drop(&mut self.fIfTrue);
            ManuallyDrop::drop(&mut self.fTest);
        }
    }
}
unsafe impl Statement for IfStatement {}
impl IRNode for IfStatement {
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base.base) }
    }
    fn description(&self) -> Vec<u8> {
        description(self)
    }
}
use super::{SkSLExpressionStatement::ExpressionStatement, SkSLLiteral::Literal, SkSLNop::Nop};
// Source: src/sksl/ir/SkSLIfStatement.cpp:23-30
fn description(this: &IfStatement) -> Vec<u8> {
    unsafe {
        let mut out = b"if (".to_vec();
        out.extend_from_slice(&this.test().as_ref().unwrap_unchecked().description());
        out.extend_from_slice(b") ");
        out.extend_from_slice(&this.ifTrue().as_ref().unwrap_unchecked().description());
        if let Some(stmt) = this.ifFalse() {
            out.extend_from_slice(b" else ");
            out.extend_from_slice(&stmt.description());
        }
        out
    }
}
impl IfStatement {
    // Source: src/sksl/ir/SkSLIfStatement.cpp:32-49
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        test: ExprSlot,
        ifTrue: StmtSlot,
        ifFalse: StmtSlot,
    ) -> StmtSlot {
        let test =
            super::SkSLType::coerceExpression(&**context.fTypes.as_ref().fBool, test, context)?;
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLIfStatement.cpp\0".as_ptr(),41; @condition b"ifTrue\0".as_ptr(); ifTrue.is_some());
        if SkSLAnalysis::DetectVarDeclarationWithoutScope(
            &**ifTrue.as_ref().unwrap_unchecked(),
            Some(context.fErrors),
        ) {
            return None;
        }
        if ifFalse.as_ref().is_some_and(|s| {
            SkSLAnalysis::DetectVarDeclarationWithoutScope(&**s, Some(context.fErrors))
        }) {
            return None;
        }
        Some(Self::Make(
            context,
            pos,
            test,
            ifTrue.unwrap_unchecked(),
            ifFalse,
        ))
    }
}
// Source: src/sksl/ir/SkSLIfStatement.cpp:51-55
unsafe fn replace_empty_with_nop(stmt: StmtSlot, isEmpty: bool) -> IRNodeBox<dyn Statement> {
    if stmt
        .as_ref()
        .is_some_and(|s| !isEmpty || s.node().is::<Nop>())
    {
        stmt.unwrap_unchecked()
    } else {
        Nop::Make()
    }
}
impl IfStatement {
    // Source: src/sksl/ir/SkSLIfStatement.cpp:57-101
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        test: IRNodeBox<dyn Expression>,
        mut ifTrue: IRNodeBox<dyn Statement>,
        mut ifFalse: StmtSlot,
    ) -> IRNodeBox<dyn Statement> {
        debug_assert!(test
            .expression_type()
            .matches(&**context.fTypes.as_ref().fBool));
        debug_assert!(!SkSLAnalysis::DetectVarDeclarationWithoutScope(
            &*ifTrue, None
        ));
        debug_assert!(
            ifFalse.is_none()
                || !SkSLAnalysis::DetectVarDeclarationWithoutScope(
                    &**ifFalse.as_ref().unwrap_unchecked(),
                    None
                )
        );
        let optimize = context
            .fConfig
            .unwrap_unchecked()
            .as_ref()
            .fSettings
            .fOptimize;
        let mut trueIsEmpty = false;
        let mut falseIsEmpty = false;
        if optimize {
            trueIsEmpty = ifTrue.isEmpty();
            falseIsEmpty = ifFalse.as_ref().map_or(true, |s| s.isEmpty());
            if trueIsEmpty && falseIsEmpty {
                return ExpressionStatement::Make(context, test);
            }
        }
        if optimize {
            let testValue =
                crate::src::sksl::SkSLConstantFolder::ConstantFolder::GetConstantValueForVariable(
                    &*test,
                );
            if testValue.isBoolLiteral() {
                if testValue.node().as_node::<Literal>().boolValue() != 0 {
                    return replace_empty_with_nop(Some(ifTrue), trueIsEmpty);
                } else {
                    return replace_empty_with_nop(ifFalse, falseIsEmpty);
                }
            }
        }
        if optimize {
            ifTrue = replace_empty_with_nop(Some(ifTrue), trueIsEmpty);
            if falseIsEmpty {
                ifFalse = None;
            }
        }
        IRNodeBox::new_with(|p: *mut Self| {
            p.write(Self::new(pos, Some(test), Some(ifTrue), ifFalse))
        })
        .cast(|p| p as *mut dyn Statement)
    }
}
