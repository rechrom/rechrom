use super::SkSLProgramVisitor::{self, ProgramVisitor};
use crate::src::sksl::{
    ir::{
        SkSLExpression::Expression,
        SkSLProgramElement::ProgramElement,
        SkSLStatement::{Kind, Statement},
    },
    SkSLAnalysis::LoopControlFlowInfo,
};

// Source: src/sksl/analysis/SkSLGetLoopControlFlowInfo.cpp:21-23
// Source: src/sksl/analysis/SkSLGetLoopControlFlowInfo.cpp:66-68
#[derive(Default)]
struct LoopControlFlowVisitor {
    fResult: LoopControlFlowInfo,
    fDepth: i32,
}
unsafe impl ProgramVisitor for LoopControlFlowVisitor {
    // Source: src/sksl/analysis/SkSLGetLoopControlFlowInfo.cpp:25-28
    fn visitExpression(&mut self, _: &dyn Expression) -> bool {
        false
    }
    // Source: src/sksl/analysis/SkSLGetLoopControlFlowInfo.cpp:30-64
    fn visitStatement(&mut self, stmt: &dyn Statement) -> bool {
        unsafe {
            match stmt.kind() {
                Kind::kContinue => self.fResult.fHasContinue |= self.fDepth == 0,
                Kind::kBreak => self.fResult.fHasBreak |= self.fDepth == 0,
                Kind::kReturn => self.fResult.fHasReturn = true,
                Kind::kFor | Kind::kDo | Kind::kSwitch => {
                    self.fDepth += 1;
                    let done = SkSLProgramVisitor::inheritedStatement(self, stmt);
                    self.fDepth -= 1;
                    return done;
                }
                _ => return SkSLProgramVisitor::inheritedStatement(self, stmt),
            }
        }
        self.fResult.fHasContinue && self.fResult.fHasBreak && self.fResult.fHasReturn
    }
    fn visitProgramElement(&mut self, pe: &dyn ProgramElement) -> bool {
        unsafe { SkSLProgramVisitor::inheritedProgramElement(self, pe) }
    }
}
// Source: src/sksl/analysis/SkSLGetLoopControlFlowInfo.cpp:72-76
pub unsafe fn GetLoopControlFlowInfo(stmt: &dyn Statement) -> LoopControlFlowInfo {
    let mut visitor = LoopControlFlowVisitor::default();
    visitor.visitStatement(stmt);
    visitor.fResult
}
