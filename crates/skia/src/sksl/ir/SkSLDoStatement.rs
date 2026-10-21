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
// Source: src/sksl/ir/SkSLDoStatement.h:27-27
// Source: src/sksl/ir/SkSLDoStatement.h:50-50
// Source: src/sksl/ir/SkSLDoStatement.h:58-58
#[repr(C)]
pub struct DoStatement {
    base: StatementData,
    fStatement: ManuallyDrop<StmtSlot>,
    fTest: ManuallyDrop<ExprSlot>,
}
// Source: src/sksl/ir/SkSLDoStatement.h:29-29
impl IRNodeType for DoStatement {
    const kIRNodeKind: i32 = Kind::kDo as i32;
}
impl DoStatement {
    // Source: src/sksl/ir/SkSLDoStatement.h:31-35
    pub unsafe fn new(pos: Position, fStatement: StmtSlot, fTest: ExprSlot) -> Self {
        Self {
            base: StatementData::new(pos, Kind::kDo),
            fStatement: ManuallyDrop::new(fStatement),
            fTest: ManuallyDrop::new(fTest),
        }
    }
    // Source: src/sksl/ir/SkSLDoStatement.h:49-51
    pub unsafe fn statement_mut(&mut self) -> &mut StmtSlot {
        &mut self.fStatement
    }
    // Source: src/sksl/ir/SkSLDoStatement.h:53-55
    pub fn statement(&self) -> &StmtSlot {
        &self.fStatement
    }
    // Source: src/sksl/ir/SkSLDoStatement.h:57-59
    pub unsafe fn test_mut(&mut self) -> &mut ExprSlot {
        &mut self.fTest
    }
    // Source: src/sksl/ir/SkSLDoStatement.h:61-63
    pub fn test(&self) -> &ExprSlot {
        &self.fTest
    }
}
// Native reverse-member destruction, including statements before their symbol owner.
impl Drop for DoStatement {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fTest);
            ManuallyDrop::drop(&mut self.fStatement);
        }
    }
}
unsafe impl Statement for DoStatement {}
impl IRNode for DoStatement {
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
impl DoStatement {
    // Source: src/sksl/ir/SkSLDoStatement.cpp:20-36
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        stmt: IRNodeBox<dyn Statement>,
        test: ExprSlot,
    ) -> StmtSlot {
        if context.fConfig.unwrap_unchecked().as_ref().strictES2Mode() {
            let mut errors = context.fErrors;
            errors
                .as_mut()
                .error(pos, b"do-while loops are not supported");
            return None;
        }
        let test =
            super::SkSLType::coerceExpression(&**context.fTypes.as_ref().fBool, test, context)?;
        if SkSLAnalysis::DetectVarDeclarationWithoutScope(&*stmt, Some(context.fErrors)) {
            return None;
        }
        Some(Self::Make(context, pos, stmt, test))
    }
    // Source: src/sksl/ir/SkSLDoStatement.cpp:38-46
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        stmt: IRNodeBox<dyn Statement>,
        test: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Statement> {
        debug_assert!(!context.fConfig.unwrap_unchecked().as_ref().strictES2Mode());
        debug_assert!(test
            .expression_type()
            .matches(&**context.fTypes.as_ref().fBool));
        debug_assert!(!SkSLAnalysis::DetectVarDeclarationWithoutScope(
            &*stmt, None
        ));
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, Some(stmt), Some(test))))
            .cast(|p| p as *mut dyn Statement)
    }
}
// Source: src/sksl/ir/SkSLDoStatement.cpp:48-51
fn description(this: &DoStatement) -> Vec<u8> {
    unsafe {
        let mut out = b"do ".to_vec();
        out.extend_from_slice(&this.statement().as_ref().unwrap_unchecked().description());
        out.extend_from_slice(b" while (");
        out.extend_from_slice(&this.test().as_ref().unwrap_unchecked().description());
        out.extend_from_slice(b");");
        out
    }
}
