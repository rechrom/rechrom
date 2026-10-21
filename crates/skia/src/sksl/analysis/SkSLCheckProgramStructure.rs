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
// Source: src/sksl/analysis/SkSLCheckProgramStructure.cpp:28-128
pub unsafe fn CheckProgramStructure(program: &Program) -> bool {
    struct Visitor<'a> {
        context: &'a Context,
        states: std::collections::HashMap<usize, bool>,
        stack: Vec<NonNull<FunctionDeclaration>>,
    }
    unsafe impl ProgramVisitor for Visitor<'_> {
        fn visitProgramElement(&mut self, pe: &dyn ProgramElement) -> bool {
            unsafe {
                if pe.node().is::<FunctionDefinition>() {
                    let decl = pe.node().as_node::<FunctionDefinition>().declaration();
                    let key = decl as *const _ as usize;
                    if let Some(visiting) = self.states.get(&key).copied() {
                        if visiting {
                            let mut msg = b"\n\t".to_vec();
                            msg.extend(decl.description());
                            for d in self.stack.iter().rev() {
                                let mut prefix = b"\n\t".to_vec();
                                prefix.extend(d.as_ref().description());
                                prefix.extend(msg);
                                msg = prefix;
                                if d.as_ptr() == decl as *const _ as *mut _ {
                                    break;
                                }
                            }
                            let mut prefix =
                                b"potential recursion (function call cycle) not allowed:".to_vec();
                            prefix.extend(msg);
                            error(self.context, pe.node().position(), &prefix);
                            self.states.insert(key, false);
                            return true;
                        }
                        return false;
                    }
                    if self.stack.len() >= 50 {
                        let mut msg = b"exceeded max function call depth:".to_vec();
                        for d in &self.stack {
                            msg.extend_from_slice(b"\n\t");
                            msg.extend(d.as_ref().description());
                        }
                        msg.extend_from_slice(b"\n\t");
                        msg.extend(decl.description());
                        error(self.context, pe.node().position(), &msg);
                        self.states.insert(key, false);
                        return true;
                    }
                    self.states.insert(key, true);
                    self.stack.push(NonNull::from(decl));
                    let result = SkSLProgramVisitor::inheritedProgramElement(self, pe);
                    self.states.insert(key, false);
                    self.stack.pop();
                    return result;
                }
                SkSLProgramVisitor::inheritedProgramElement(self, pe)
            }
        }
        fn visitExpression(&mut self, e: &dyn Expression) -> bool {
            unsafe {
                let mut early = false;
                if e.node().is::<FunctionCall>() {
                    let d = e.node().as_node::<FunctionCall>().function();
                    if !d.isIntrinsic() {
                        if let Some(def) = d.definition() {
                            early = self.visitProgramElement(def);
                        }
                    }
                }
                early || SkSLProgramVisitor::inheritedExpression(self, e)
            }
        }
        fn visitStatement(&mut self, s: &dyn Statement) -> bool {
            unsafe { SkSLProgramVisitor::inheritedStatement(self, s) }
        }
    }
    let mut v = Visitor {
        context: program.fContext.as_ref().unwrap_unchecked(),
        states: Default::default(),
        stack: Vec::new(),
    };
    for pe in program.fOwnedElements.iter() {
        let pe = &**pe.as_ref().unwrap_unchecked();
        if pe.node().is::<FunctionDefinition>() {
            v.visitProgramElement(pe);
        }
    }
    true
}
