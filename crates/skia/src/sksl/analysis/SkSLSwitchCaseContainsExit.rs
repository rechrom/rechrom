// UNVERIFIED: no compilation, tests, syntax or source checks were run.
use super::SkSLProgramVisitor;
use crate::src::sksl::ir::SkSLStatement::{Kind, Statement};
// Source: src/sksl/analysis/SkSLSwitchCaseContainsExit.cpp:20-85
struct SwitchCaseContainsExit {
    fConditionalExits: bool,
    fInConditional: i32,
    fInLoop: i32,
    fInSwitch: i32,
}
impl SwitchCaseContainsExit {
    fn new(conditionalExits: bool) -> Self {
        Self {
            fConditionalExits: conditionalExits,
            fInConditional: 0,
            fInLoop: 0,
            fInSwitch: 0,
        }
    }
    unsafe fn inherited(&mut self, stmt: &dyn Statement) -> bool {
        SkSLProgramVisitor::visitStatement(
            &mut |s| self.visitStatement(s),
            &mut |_expr| false,
            stmt,
        )
    }
    unsafe fn visitStatement(&mut self, stmt: &dyn Statement) -> bool {
        match stmt.kind() {
            Kind::kBlock | Kind::kSwitchCase => self.inherited(stmt),
            Kind::kReturn => {
                if self.fConditionalExits {
                    self.fInConditional != 0
                } else {
                    self.fInConditional == 0
                }
            }
            Kind::kContinue => {
                self.fInLoop == 0
                    && (if self.fConditionalExits {
                        self.fInConditional != 0
                    } else {
                        self.fInConditional == 0
                    })
            }
            Kind::kBreak => {
                self.fInLoop == 0
                    && self.fInSwitch == 0
                    && (if self.fConditionalExits {
                        self.fInConditional != 0
                    } else {
                        self.fInConditional == 0
                    })
            }
            Kind::kIf => {
                self.fInConditional += 1;
                let result = self.inherited(stmt);
                self.fInConditional -= 1;
                result
            }
            Kind::kFor | Kind::kDo => {
                self.fInConditional += 1;
                self.fInLoop += 1;
                let result = self.inherited(stmt);
                self.fInLoop -= 1;
                self.fInConditional -= 1;
                result
            }
            Kind::kSwitch => {
                self.fInSwitch += 1;
                let result = self.inherited(stmt);
                self.fInSwitch -= 1;
                result
            }
            _ => false,
        }
    }
}
// Source: src/sksl/analysis/SkSLSwitchCaseContainsExit.cpp:90-92
pub fn SwitchCaseContainsUnconditionalExit(stmt: &dyn Statement) -> bool {
    unsafe { SwitchCaseContainsExit::new(false).visitStatement(stmt) }
}
// Source: src/sksl/analysis/SkSLSwitchCaseContainsExit.cpp:94-96
pub fn SwitchCaseContainsConditionalExit(stmt: &dyn Statement) -> bool {
    unsafe { SwitchCaseContainsExit::new(true).visitStatement(stmt) }
}
