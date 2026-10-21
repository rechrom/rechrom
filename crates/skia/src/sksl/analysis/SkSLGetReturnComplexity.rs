use crate::src::sksl::{
    analysis::SkSLProgramVisitor::{self, ProgramVisitor},
    ir::{
        SkSLBlock::Block,
        SkSLExpression::Expression,
        SkSLFunctionDefinition::FunctionDefinition,
        SkSLProgramElement::ProgramElement,
        SkSLStatement::{Kind, Statement},
    },
};

use crate::src::sksl::SkSLAnalysis::ReturnComplexity;

// Source: src/sksl/analysis/SkSLGetReturnComplexity.cpp:23-63
unsafe fn count_returns_at_end_of_control_flow(funcDef: &FunctionDefinition) -> i32 {
    struct CountReturnsAtEndOfControlFlow {
        fNumReturns: i32,
    }
    unsafe impl ProgramVisitor for CountReturnsAtEndOfControlFlow {
        fn visitExpression(&mut self, _: &dyn Expression) -> bool {
            false
        }
        fn visitStatement(&mut self, stmt: &dyn Statement) -> bool {
            unsafe {
                match stmt.kind() {
                    Kind::kBlock => {
                        let block = stmt.node().as_node::<Block>();
                        let children = block.children().as_slice();
                        !children.is_empty()
                            && self.visitStatement(
                                &**children
                                    .last()
                                    .unwrap_unchecked()
                                    .as_ref()
                                    .unwrap_unchecked(),
                            )
                    }
                    Kind::kSwitch | Kind::kDo | Kind::kFor => false,
                    Kind::kReturn => {
                        self.fNumReturns += 1;
                        SkSLProgramVisitor::inheritedStatement(self, stmt)
                    }
                    _ => SkSLProgramVisitor::inheritedStatement(self, stmt),
                }
            }
        }
        fn visitProgramElement(&mut self, pe: &dyn ProgramElement) -> bool {
            unsafe { SkSLProgramVisitor::inheritedProgramElement(self, pe) }
        }
    }
    let mut counter = CountReturnsAtEndOfControlFlow { fNumReturns: 0 };
    counter.visitProgramElement(funcDef);
    counter.fNumReturns
}

// Source: src/sksl/analysis/SkSLGetReturnComplexity.cpp:65-114
// Native state order is retained; the recursive methods reborrow the same visitor.
struct CountReturnsWithLimit {
    fNumReturns: i32,
    fDeepestReturn: i32,
    fLimit: i32,
    fScopedBlockDepth: i32,
    fVariablesInBlocks: bool,
}
unsafe impl ProgramVisitor for CountReturnsWithLimit {
    fn visitExpression(&mut self, _: &dyn Expression) -> bool {
        false
    }
    fn visitStatement(&mut self, stmt: &dyn Statement) -> bool {
        unsafe {
            match stmt.kind() {
                Kind::kReturn => {
                    self.fNumReturns += 1;
                    self.fDeepestReturn = self.fDeepestReturn.max(self.fScopedBlockDepth);
                    self.fNumReturns >= self.fLimit
                        || SkSLProgramVisitor::inheritedStatement(self, stmt)
                }
                Kind::kVarDeclaration => {
                    if self.fScopedBlockDepth > 1 {
                        self.fVariablesInBlocks = true;
                    }
                    SkSLProgramVisitor::inheritedStatement(self, stmt)
                }
                Kind::kBlock => {
                    let depthIncrement = i32::from(stmt.node().as_node::<Block>().isScope());
                    self.fScopedBlockDepth += depthIncrement;
                    let result = SkSLProgramVisitor::inheritedStatement(self, stmt);
                    self.fScopedBlockDepth -= depthIncrement;
                    if self.fNumReturns == 0 && self.fScopedBlockDepth <= 1 {
                        self.fVariablesInBlocks = false;
                    }
                    result
                }
                _ => SkSLProgramVisitor::inheritedStatement(self, stmt),
            }
        }
    }
    fn visitProgramElement(&mut self, pe: &dyn ProgramElement) -> bool {
        unsafe { SkSLProgramVisitor::inheritedProgramElement(self, pe) }
    }
}

// Source: src/sksl/analysis/SkSLGetReturnComplexity.cpp:116-129
pub unsafe fn GetReturnComplexity(funcDef: &FunctionDefinition) -> ReturnComplexity {
    let returnsAtEndOfControlFlow = count_returns_at_end_of_control_flow(funcDef);
    let mut counter = CountReturnsWithLimit {
        fNumReturns: 0,
        fDeepestReturn: 0,
        fLimit: returnsAtEndOfControlFlow + 1,
        fScopedBlockDepth: 0,
        fVariablesInBlocks: false,
    };
    counter.visitProgramElement(funcDef);
    if counter.fNumReturns > returnsAtEndOfControlFlow {
        return ReturnComplexity::kEarlyReturns;
    }
    if counter.fNumReturns > 1 {
        return ReturnComplexity::kScopedReturns;
    }
    if counter.fVariablesInBlocks && counter.fDeepestReturn > 1 {
        return ReturnComplexity::kScopedReturns;
    }
    ReturnComplexity::kSingleSafeReturn
}
