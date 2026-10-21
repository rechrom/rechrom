// UNVERIFIED direct pinned-source translation; no validation performed.
use super::SkSLProgramVisitor::{self, ProgramVisitor};
use crate::src::sksl::ir::{
    SkSLDoStatement::DoStatement,
    SkSLExpression::Expression,
    SkSLForStatement::ForStatement,
    SkSLFunctionDeclaration::FunctionDeclaration,
    SkSLIfStatement::IfStatement,
    SkSLProgramElement::ProgramElement,
    SkSLStatement::{Kind, Statement},
    SkSLSwitchCase::SwitchCase,
    SkSLSwitchStatement::SwitchStatement,
};
// Source: src/sksl/analysis/SkSLCanExitWithoutReturningValue.cpp:27-28
// Source: src/sksl/analysis/SkSLCanExitWithoutReturningValue.cpp:157-159
struct ReturnsOnAllPathsVisitor {
    fFoundReturn: bool,
    fFoundBreak: bool,
    fFoundContinue: bool,
}
impl ReturnsOnAllPathsVisitor {
    fn new() -> Self {
        Self {
            fFoundReturn: false,
            fFoundBreak: false,
            fFoundContinue: false,
        }
    }
}
unsafe impl ProgramVisitor for ReturnsOnAllPathsVisitor {
    // Source: src/sksl/analysis/SkSLCanExitWithoutReturningValue.cpp:29-32
    fn visitExpression(&mut self, _expr: &dyn Expression) -> bool {
        false
    }
    // Source: src/sksl/analysis/SkSLCanExitWithoutReturningValue.cpp:34-155
    fn visitStatement(&mut self, stmt: &dyn Statement) -> bool {
        unsafe {
            match stmt.kind() {
                // Source: src/sksl/analysis/SkSLCanExitWithoutReturningValue.cpp:38-48
                Kind::kReturn => {
                    self.fFoundReturn = true;
                    true
                }
                Kind::kBreak => {
                    self.fFoundBreak = true;
                    true
                }
                Kind::kContinue => {
                    self.fFoundContinue = true;
                    true
                }
                // Source: src/sksl/analysis/SkSLCanExitWithoutReturningValue.cpp:50-66
                Kind::kIf => {
                    let i = stmt.node().as_node::<IfStatement>();
                    let mut yes = Self::new();
                    let mut no = Self::new();
                    yes.visitStatement(&**i.ifTrue().as_ref().unwrap_unchecked());
                    if let Some(other) = i.ifFalse() {
                        no.visitStatement(&**other);
                    }
                    self.fFoundBreak = yes.fFoundBreak || no.fFoundBreak;
                    self.fFoundContinue = yes.fFoundContinue || no.fFoundContinue;
                    self.fFoundReturn = yes.fFoundReturn && no.fFoundReturn;
                    self.fFoundBreak || self.fFoundContinue || self.fFoundReturn
                }
                // Source: src/sksl/analysis/SkSLCanExitWithoutReturningValue.cpp:67-78
                Kind::kFor => {
                    let f = stmt.node().as_node::<ForStatement>();
                    let mut scan = Self::new();
                    scan.visitStatement(&**f.statement().as_ref().unwrap_unchecked());
                    self.fFoundReturn = scan.fFoundReturn;
                    self.fFoundReturn
                }
                // Source: src/sksl/analysis/SkSLCanExitWithoutReturningValue.cpp:79-88
                Kind::kDo => {
                    let d = stmt.node().as_node::<DoStatement>();
                    let mut scan = Self::new();
                    scan.visitStatement(&**d.statement().as_ref().unwrap_unchecked());
                    self.fFoundReturn = scan.fFoundReturn;
                    self.fFoundReturn
                }
                // Source: src/sksl/analysis/SkSLCanExitWithoutReturningValue.cpp:89-92
                Kind::kBlock => SkSLProgramVisitor::inheritedStatement(self, stmt),
                // Source: src/sksl/analysis/SkSLCanExitWithoutReturningValue.cpp:94-140
                Kind::kSwitch => {
                    let switch = stmt.node().as_node::<SwitchStatement>();
                    let mut foundDefault = false;
                    let mut fellThrough = false;
                    for item in switch.cases().as_slice() {
                        let case = item
                            .as_ref()
                            .unwrap_unchecked()
                            .node()
                            .as_node::<SwitchCase>();
                        if case.isDefault() {
                            foundDefault = true;
                        }
                        let mut scan = Self::new();
                        scan.visitStatement(case);
                        if scan.fFoundContinue {
                            self.fFoundContinue = true;
                            return false;
                        }
                        if scan.fFoundBreak {
                            return false;
                        }
                        fellThrough = !scan.fFoundReturn;
                    }
                    if fellThrough || !foundDefault {
                        return false;
                    }
                    self.fFoundReturn = true;
                    true
                }
                // Source: src/sksl/analysis/SkSLCanExitWithoutReturningValue.cpp:142-144
                Kind::kSwitchCase => SkSLProgramVisitor::inheritedStatement(self, stmt),
                // Source: src/sksl/analysis/SkSLCanExitWithoutReturningValue.cpp:146-154
                Kind::kDiscard | Kind::kExpression | Kind::kNop | Kind::kVarDeclaration => false,
                _ => core::hint::unreachable_unchecked(),
            }
        }
    }
    // Native inherited virtual required by Rust's explicit visitor trait. This
    // delegates actual traversal; it is not a constant false missing-node body.
    fn visitProgramElement(&mut self, element: &dyn ProgramElement) -> bool {
        unsafe { SkSLProgramVisitor::inheritedProgramElement(self, element) }
    }
}
// Source: src/sksl/analysis/SkSLCanExitWithoutReturningValue.cpp:166-174
pub unsafe fn CanExitWithoutReturningValue(
    function: &FunctionDeclaration,
    body: &dyn Statement,
) -> bool {
    if function.returnType().isVoid() {
        return false;
    }
    let mut visitor = ReturnsOnAllPathsVisitor::new();
    visitor.visitStatement(body);
    !visitor.fFoundReturn
}
