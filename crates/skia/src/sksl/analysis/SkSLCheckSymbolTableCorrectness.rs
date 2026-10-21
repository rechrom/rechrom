use crate::src::sksl::ir::SkSLIRNode::IRNode;
// UNVERIFIED: user prohibited validation.
use crate::src::sksl::{
    analysis::SkSLProgramVisitor::{self, ProgramVisitor},
    ir::{
        SkSLExpression::{Expression, ExpressionExt},
        SkSLFunctionCall::FunctionCall,
        SkSLFunctionDeclaration::FunctionDeclaration,
        SkSLFunctionDefinition::FunctionDefinition,
        SkSLProgram::Program,
        SkSLProgramElement::ProgramElement,
        SkSLStatement::Statement,
        SkSLSymbol::Symbol,
    },
    SkSLContext::Context,
    SkSLErrorReporter::ErrorReporterExt,
};
use core::ptr::NonNull;
unsafe fn error(c: &Context, p: crate::src::sksl::SkSLPosition::Position, m: &[u8]) {
    let mut e = c.fErrors;
    e.as_mut().error(p, m);
}
use crate::src::sksl::{
    ir::{SkSLSymbolTable::SymbolTable, SkSLVarDeclarations::VarDeclaration},
    SkSLAnalysis::SymbolTableStackBuilder,
};
// Source: src/sksl/analysis/SkSLCheckSymbolTableCorrectness.cpp:34-84
pub unsafe fn CheckSymbolTableCorrectness(program: &Program) {
    struct Visitor<'a> {
        context: &'a Context,
        stack: Vec<NonNull<SymbolTable>>,
    }
    unsafe impl ProgramVisitor for Visitor<'_> {
        fn visitStatement(&mut self, s: &dyn Statement) -> bool {
            unsafe {
                let _builder =
                    SymbolTableStackBuilder::new(Some(s), NonNull::from(&mut self.stack));
                if s.node().is::<VarDeclaration>() {
                    let d = s.node().as_node::<VarDeclaration>();
                    let var = d.var().unwrap_unchecked();
                    let mut found = false;
                    self.stack
                        .last()
                        .unwrap_unchecked()
                        .as_ref()
                        .foreach(|_, sym| {
                            if core::ptr::addr_eq(sym.as_ptr(), var.as_ptr()) {
                                found = true;
                            }
                        });
                    if !found {
                        let mut msg = b"internal error (variable '".to_vec();
                        msg.extend_from_slice(var.as_ref().name());
                        msg.extend_from_slice(b"' is incorrectly scoped)");
                        error(self.context, s.node().position(), &msg);
                    }
                }
                SkSLProgramVisitor::inheritedStatement(self, s)
            }
        }
        fn visitExpression(&mut self, _: &dyn Expression) -> bool {
            false
        }
        fn visitProgramElement(&mut self, pe: &dyn ProgramElement) -> bool {
            unsafe { SkSLProgramVisitor::inheritedProgramElement(self, pe) }
        }
    }
    let mut v = Visitor {
        context: program.fContext.as_ref().unwrap_unchecked(),
        stack: vec![NonNull::from(
            program.fSymbols.as_deref().unwrap_unchecked(),
        )],
    };
    for pe in program.fOwnedElements.iter() {
        v.visitProgramElement(&**pe.as_ref().unwrap_unchecked());
    }
}
