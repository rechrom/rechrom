use crate::src::sksl::{
    ir::{
        SkSLBlock::{Block, Kind as BlockKind},
        SkSLDoStatement::DoStatement,
        SkSLForStatement::ForStatement,
        SkSLFunctionDefinition::FunctionDefinition,
        SkSLIRNode::*,
        SkSLIfStatement::IfStatement,
        SkSLNop::Nop,
        SkSLStatement::Statement,
    },
    transform::SkSLProgramWriter::{self, ProgramWriter, UniquePtrExpression, UniquePtrStatement},
    SkSLContext::Context,
    SkSLDefines::StatementArray,
    SkSLModule::Module,
};
use core::pin::Pin;
// Unique owner projections never move initialized IR allocations.
unsafe fn statement_mut<T: IRNodeType>(stmt: &mut UniquePtrStatement) -> &mut T {
    stmt.as_mut()
        .unwrap_unchecked()
        .as_pin_mut()
        .node_mut()
        .as_node_mut::<T>()
        .get_unchecked_mut()
}
// Source: src/sksl/transform/SkSLEliminateUnnecessaryBraces.cpp:32-33
pub unsafe fn EliminateUnnecessaryBraces(context: &Context, module: &mut Module) {
    struct UnnecessaryBraceEliminator;
    unsafe impl ProgramWriter for UnnecessaryBraceEliminator {
        // Source: src/sksl/transform/SkSLEliminateUnnecessaryBraces.cpp:35-38
        unsafe fn visitExpressionPtr(&mut self, _expr: &mut UniquePtrExpression) -> bool {
            false
        }
        // Source: src/sksl/transform/SkSLEliminateUnnecessaryBraces.cpp:40-67
        unsafe fn visitStatementPtr(&mut self, stmt: &mut UniquePtrStatement) -> bool {
            SkSLProgramWriter::inheritedStatement(
                self,
                stmt.as_mut().unwrap_unchecked().as_pin_mut(),
            );
            match stmt.as_ref().unwrap_unchecked().kind() {
                StatementKind::kIf => {
                    let s = statement_mut::<IfStatement>(stmt);
                    Self::EliminateBracesFrom(s.ifTrue_mut());
                    Self::EliminateBracesFrom(s.ifFalse_mut());
                }
                StatementKind::kFor => {
                    Self::EliminateBracesFrom(statement_mut::<ForStatement>(stmt).statement_mut())
                }
                StatementKind::kDo => {
                    Self::EliminateBracesFrom(statement_mut::<DoStatement>(stmt).statement_mut())
                }
                _ => {}
            }
            false
        }
    }
    impl UnnecessaryBraceEliminator {
        // Source: src/sksl/transform/SkSLEliminateUnnecessaryBraces.cpp:69-95
        unsafe fn EliminateBracesFrom(stmt: &mut UniquePtrStatement) {
            if stmt.as_ref().is_none_or(|s| !s.node().is::<Block>()) {
                return;
            }
            let block = statement_mut::<Block>(stmt);
            let mut useful = None;
            for (i, child) in block.children().as_slice().iter().enumerate() {
                if child.as_ref().unwrap_unchecked().isEmpty() {
                    continue;
                }
                if useful.is_some() {
                    return;
                }
                useful = Some(i);
            }
            let replacement = if let Some(i) = useful {
                Pin::new_unchecked(block)
                    .children_mut()
                    .at_mut(i as i32)
                    .take()
            } else {
                Some(Nop::Make())
            };
            *stmt = replacement;
        }
    }
    // Source: src/sksl/transform/SkSLEliminateUnnecessaryBraces.cpp:100-100
    // Source: src/sksl/transform/SkSLEliminateUnnecessaryBraces.cpp:169-169
    struct RequiredBraceWriter<'a> {
        fContext: &'a Context,
    }
    impl<'a> RequiredBraceWriter<'a> {
        // Source: src/sksl/transform/SkSLEliminateUnnecessaryBraces.cpp:102-102
        fn new(context: &'a Context) -> Self {
            Self { fContext: context }
        }
    }
    unsafe impl ProgramWriter for RequiredBraceWriter<'_> {
        // Source: src/sksl/transform/SkSLEliminateUnnecessaryBraces.cpp:104-107
        unsafe fn visitExpressionPtr(&mut self, _expr: &mut UniquePtrExpression) -> bool {
            false
        }
        // Source: src/sksl/transform/SkSLEliminateUnnecessaryBraces.cpp:109-167
        unsafe fn visitStatementPtr(&mut self, stmt: &mut UniquePtrStatement) -> bool {
            SkSLProgramWriter::inheritedStatement(
                self,
                stmt.as_mut().unwrap_unchecked().as_pin_mut(),
            );
            if stmt.as_ref().unwrap_unchecked().node().is::<IfStatement>() {
                let outer = statement_mut::<IfStatement>(stmt);
                if outer.ifFalse().is_some()
                    && outer
                        .ifTrue()
                        .as_ref()
                        .unwrap_unchecked()
                        .node()
                        .is::<IfStatement>()
                {
                    let inner = outer
                        .ifTrue()
                        .as_ref()
                        .unwrap_unchecked()
                        .node()
                        .as_node::<IfStatement>();
                    if inner.ifFalse().is_none() {
                        let mut blockStmts = StatementArray::new_pinned();
                        blockStmts.as_mut().push_back(outer.ifTrue_mut().take());
                        let stmtPosition = blockStmts.as_slice()[0]
                            .as_ref()
                            .unwrap_unchecked()
                            .node()
                            .fPosition;
                        let bracedIfTrue = Block::MakeBlock(
                            stmtPosition,
                            blockStmts,
                            BlockKind::kBracedScope,
                            None,
                        )
                        .cast(|p| p as *mut dyn Statement);
                        let pos = outer.node().fPosition;
                        let test = outer.test_mut().take().unwrap_unchecked();
                        let ifFalse = outer.ifFalse_mut().take();
                        *stmt = Some(IfStatement::Make(
                            self.fContext,
                            pos,
                            test,
                            bracedIfTrue,
                            ifFalse,
                        ));
                    }
                }
            }
            false
        }
    }
    // Source: src/sksl/transform/SkSLEliminateUnnecessaryBraces.cpp:173-185
    for pe in module.fElements.iter_mut() {
        let pe = pe.as_mut().unwrap_unchecked();
        if pe.node().is::<FunctionDefinition>() {
            let def = pe
                .as_pin_mut()
                .node_mut()
                .as_node_mut::<FunctionDefinition>()
                .get_unchecked_mut();
            let mut eliminator = UnnecessaryBraceEliminator;
            eliminator.visitStatementPtr(def.body_mut());
            let mut writer = RequiredBraceWriter::new(context);
            writer.visitStatementPtr(def.body_mut());
        }
    }
}
