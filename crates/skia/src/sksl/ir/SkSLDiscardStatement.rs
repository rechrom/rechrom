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
// Source: src/sksl/ir/SkSLDiscardStatement.h:25-25
#[repr(C)]
pub struct DiscardStatement {
    base: StatementData,
}
// Source: src/sksl/ir/SkSLDiscardStatement.h:27-27
impl IRNodeType for DiscardStatement {
    const kIRNodeKind: i32 = Kind::kDiscard as i32;
}
impl DiscardStatement {
    // Source: src/sksl/ir/SkSLDiscardStatement.h:29-29
    pub unsafe fn new(pos: Position) -> Self {
        Self {
            base: StatementData::new(pos, Kind::kDiscard),
        }
    }
}
unsafe impl Statement for DiscardStatement {}
impl IRNode for DiscardStatement {
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
// Source: src/sksl/ir/SkSLDiscardStatement.h:37-39
fn description(_: &DiscardStatement) -> Vec<u8> {
    b"discard;".to_vec()
}
impl DiscardStatement {
    // Source: src/sksl/ir/SkSLDiscardStatement.cpp:16-22
    pub unsafe fn Convert(context: &Context, pos: Position) -> StmtSlot {
        if !ProgramConfig::IsFragment(context.fConfig.unwrap_unchecked().as_ref().fKind) {
            let mut errors = context.fErrors;
            errors.as_mut().error(
                pos,
                b"discard statement is only permitted in fragment shaders",
            );
            return None;
        }
        Some(Self::Make(context, pos))
    }
    // Source: src/sksl/ir/SkSLDiscardStatement.cpp:24-27
    pub unsafe fn Make(context: &Context, pos: Position) -> IRNodeBox<dyn Statement> {
        debug_assert!(ProgramConfig::IsFragment(
            context.fConfig.unwrap_unchecked().as_ref().fKind
        ));
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos)))
            .cast(|p| p as *mut dyn Statement)
    }
}
