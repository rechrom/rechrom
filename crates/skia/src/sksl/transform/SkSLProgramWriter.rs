// Native mutable visitor instantiation. UNVERIFIED: no validation requested.
use crate::src::sksl::ir::{
    SkSLExpression::{Expression, ExpressionExt},
    SkSLIRNode::{IRNodeBox, IRNodeType},
    SkSLProgramElement::ProgramElement,
    SkSLStatement::Statement,
};
use core::pin::Pin;

// Source: src/sksl/transform/SkSLProgramWriter.h:15-24
// Rust type aliases select the mutable unique owners of the native template.
pub type UniquePtrExpression = Option<IRNodeBox<dyn Expression>>;
pub type UniquePtrStatement = Option<IRNodeBox<dyn Statement>>;

// Source: src/sksl/transform/SkSLProgramWriter.h:26-36
// Source: src/sksl/analysis/SkSLProgramVisitor.h:34-46
// Dynamic-this adaptation: generic inherited functions reborrow this same writer,
// preserving virtual dispatch and early exit. Pin protects the IR allocation;
// the unique_ptr slot may still be replaced, as in the native writer.
// Safety: implementations preserve node pinning and original pool/symbol leases;
// a callback cannot keep a mutable borrow of a node after returning.
pub unsafe trait ProgramWriter {
    unsafe fn visitExpression(&mut self, e: Pin<&mut dyn Expression>) -> bool {
        inheritedExpression(self, e)
    }
    unsafe fn visitStatement(&mut self, s: Pin<&mut dyn Statement>) -> bool {
        inheritedStatement(self, s)
    }
    unsafe fn visitProgramElement(&mut self, pe: Pin<&mut dyn ProgramElement>) -> bool {
        inheritedProgramElement(self, pe)
    }
    unsafe fn visitExpressionPtr(&mut self, e: &mut UniquePtrExpression) -> bool {
        self.visitExpression(e.as_mut().expect("native nonnull expression").as_pin_mut())
    }
    unsafe fn visitStatementPtr(&mut self, s: &mut UniquePtrStatement) -> bool {
        self.visitStatement(s.as_mut().expect("native nonnull statement").as_pin_mut())
    }
}

// Shared native template implementation lives at its original source path.
pub use crate::src::sksl::SkSLAnalysis::{
    writerVisitExpression as inheritedExpression,
    writerVisitProgramElement as inheritedProgramElement,
    writerVisitStatement as inheritedStatement,
};
