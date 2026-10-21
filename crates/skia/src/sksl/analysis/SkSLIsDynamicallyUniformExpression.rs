use super::SkSLProgramVisitor::{self, ProgramVisitor};
use crate::src::sksl::ir::{
    SkSLExpression::{Expression, ExpressionExt, Kind},
    SkSLFunctionCall::FunctionCall,
    SkSLProgramElement::ProgramElement,
    SkSLStatement::Statement,
    SkSLVariableReference::VariableReference,
};

// Source: src/sksl/analysis/SkSLIsDynamicallyUniformExpression.cpp:21-84
pub unsafe fn IsDynamicallyUniformExpression(expr: &dyn Expression) -> bool {
    // Source: src/sksl/analysis/SkSLIsDynamicallyUniformExpression.cpp:22-23
    // Source: src/sksl/analysis/SkSLIsDynamicallyUniformExpression.cpp:77-79
    struct Visitor {
        fIsDynamicallyUniform: bool,
    }
    unsafe impl ProgramVisitor for Visitor {
        // Source: src/sksl/analysis/SkSLIsDynamicallyUniformExpression.cpp:24-75
        fn visitExpression(&mut self, expr: &dyn Expression) -> bool {
            unsafe {
                match expr.kind() {
                    Kind::kBinary
                    | Kind::kConstructorArray
                    | Kind::kConstructorArrayCast
                    | Kind::kConstructorCompound
                    | Kind::kConstructorCompoundCast
                    | Kind::kConstructorDiagonalMatrix
                    | Kind::kConstructorMatrixResize
                    | Kind::kConstructorScalarCast
                    | Kind::kConstructorSplat
                    | Kind::kConstructorStruct
                    | Kind::kFieldAccess
                    | Kind::kIndex
                    | Kind::kPostfix
                    | Kind::kPrefix
                    | Kind::kSwizzle
                    | Kind::kTernary => {}
                    Kind::kVariableReference => {
                        let var = expr.node().as_node::<VariableReference>().variable();
                        if !var.is_some_and(|var| {
                            let flags = var.as_ref().modifierFlags();
                            flags.isConst() || flags.isUniform()
                        }) {
                            self.fIsDynamicallyUniform = false;
                            return true;
                        }
                    }
                    Kind::kFunctionCall => {
                        if !expr
                            .node()
                            .as_node::<FunctionCall>()
                            .function()
                            .modifierFlags()
                            .isPure()
                        {
                            self.fIsDynamicallyUniform = false;
                            return true;
                        }
                    }
                    Kind::kLiteral => return false,
                    _ => {
                        self.fIsDynamicallyUniform = false;
                        return true;
                    }
                }
                SkSLProgramVisitor::inheritedExpression(self, expr)
            }
        }
        fn visitStatement(&mut self, stmt: &dyn Statement) -> bool {
            unsafe { SkSLProgramVisitor::inheritedStatement(self, stmt) }
        }
        fn visitProgramElement(&mut self, pe: &dyn ProgramElement) -> bool {
            unsafe { SkSLProgramVisitor::inheritedProgramElement(self, pe) }
        }
    }
    // Source: src/sksl/analysis/SkSLIsDynamicallyUniformExpression.cpp:81-83
    let mut visitor = Visitor {
        fIsDynamicallyUniform: true,
    };
    visitor.visitExpression(expr);
    visitor.fIsDynamicallyUniform
}
