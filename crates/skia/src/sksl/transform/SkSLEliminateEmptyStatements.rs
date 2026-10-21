use crate::src::sksl::{
    ir::{
        SkSLBlock::Block, SkSLFunctionDefinition::FunctionDefinition, SkSLIRNode::*,
        SkSLProgramElement::ProgramElement,
    },
    transform::SkSLProgramWriter::{self, ProgramWriter, UniquePtrExpression, UniquePtrStatement},
    SkSLModule::Module,
};
use core::pin::Pin;
// Source: src/sksl/transform/SkSLEliminateEmptyStatements.cpp:27-28
unsafe fn eliminate_empty_statements(elements: &mut [Option<IRNodeBox<dyn ProgramElement>>]) {
    struct EmptyStatementEliminator;
    unsafe impl ProgramWriter for EmptyStatementEliminator {
        // Source: src/sksl/transform/SkSLEliminateEmptyStatements.cpp:30-33
        unsafe fn visitExpressionPtr(&mut self, _expr: &mut UniquePtrExpression) -> bool {
            false
        }
        // Source: src/sksl/transform/SkSLEliminateEmptyStatements.cpp:35-50
        unsafe fn visitStatementPtr(&mut self, stmt: &mut UniquePtrStatement) -> bool {
            SkSLProgramWriter::inheritedStatement(
                self,
                stmt.as_mut().unwrap_unchecked().as_pin_mut(),
            );
            if stmt.as_ref().unwrap_unchecked().node().is::<Block>() {
                let block = stmt
                    .as_mut()
                    .unwrap_unchecked()
                    .as_pin_mut()
                    .node_mut()
                    .as_node_mut::<Block>();
                let mut children = block.children_mut();
                // std::remove_if's stable compaction moves the same unique owners, then
                // resize destroys the removed tail. Taking source leaves the native moved-from null slot.
                let mut out = 0;
                let n = children.size();
                for i in 0..n {
                    if !children.as_slice()[i as usize]
                        .as_ref()
                        .unwrap_unchecked()
                        .isEmpty()
                    {
                        if out != i {
                            let moved = children.as_mut().at_mut(i).take();
                            *children.as_mut().at_mut(out) = moved;
                        }
                        out += 1;
                    }
                }
                children.as_mut().resize_back_with(out, || None);
            }
            false
        }
    }
    // Source: src/sksl/transform/SkSLEliminateEmptyStatements.cpp:55-60
    for pe in elements {
        let pe = pe.as_mut().unwrap_unchecked();
        if pe.node().is::<FunctionDefinition>() {
            let mut visitor = EmptyStatementEliminator;
            let def = pe
                .as_pin_mut()
                .node_mut()
                .as_node_mut::<FunctionDefinition>()
                .get_unchecked_mut();
            visitor.visitStatementPtr(def.body_mut());
        }
    }
}
// Source: src/sksl/transform/SkSLEliminateEmptyStatements.cpp:63-65
pub unsafe fn EliminateEmptyStatements(module: &mut Module) {
    eliminate_empty_statements(&mut module.fElements);
}
