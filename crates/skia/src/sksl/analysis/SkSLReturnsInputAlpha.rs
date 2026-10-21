use super::{
    SkSLProgramUsage::ProgramUsage,
    SkSLProgramVisitor::{self, ProgramVisitor},
};
use crate::src::sksl::ir::{
    SkSLConstructor::asAnyConstructor,
    SkSLConstructorCompoundCast::ConstructorCompoundCast,
    SkSLExpression::{Expression, ExpressionExt, Kind},
    SkSLFunctionDefinition::FunctionDefinition,
    SkSLProgramElement::ProgramElement,
    SkSLReturnStatement::ReturnStatement,
    SkSLStatement::Statement,
    SkSLSwizzle::Swizzle,
    SkSLTernaryExpression::TernaryExpression,
    SkSLVariable::{Variable, VariableData},
    SkSLVariableReference::VariableReference,
};
use std::ptr::NonNull;

// Source: src/sksl/analysis/SkSLReturnsInputAlpha.cpp:37-39
// Source: src/sksl/analysis/SkSLReturnsInputAlpha.cpp:117-122
struct ReturnsInputAlphaVisitor<'a> {
    fUsage: &'a ProgramUsage,
    fInputVar: Option<NonNull<VariableData>>,
}
impl<'a> ReturnsInputAlphaVisitor<'a> {
    fn new(usage: &'a ProgramUsage) -> Self {
        Self {
            fUsage: usage,
            fInputVar: None,
        }
    }
    // Source: src/sksl/analysis/SkSLReturnsInputAlpha.cpp:41-62
    unsafe fn visitProgramElement(&mut self, pe: &dyn ProgramElement) -> bool {
        let decl = pe.node().as_node::<FunctionDefinition>().declaration();
        let parameters = decl.parameters();
        if parameters.len() != 1 {
            return true;
        }
        let input = parameters[0].unwrap_unchecked().as_ref();
        if input.symbol_type().columns() != 4 || !input.symbol_type().componentType().isFloat() {
            return true;
        }
        self.fInputVar = Some(NonNull::from(input.variable_data()));
        if self.fUsage.getVariable(input).fWrite != 0 {
            return true;
        }
        SkSLProgramVisitor::inheritedProgramElement(self, pe)
    }
    // Source: src/sksl/analysis/SkSLReturnsInputAlpha.cpp:64-66
    unsafe fn isInputVar(&self, expr: &dyn Expression) -> bool {
        expr.node().is::<VariableReference>()
            && expr
                .node()
                .as_node::<VariableReference>()
                .variable()
                .map(|var| NonNull::from(var.as_ref().variable_data()))
                == self.fInputVar
    }
    // Source: src/sksl/analysis/SkSLReturnsInputAlpha.cpp:68-74
    unsafe fn isInputSwizzleEndingWithAlpha(&self, expr: &dyn Expression) -> bool {
        if !expr.node().is::<Swizzle>() {
            return false;
        }
        let swizzle = expr.node().as_node::<Swizzle>();
        self.isInputVar(&**swizzle.base().as_ref().unwrap_unchecked())
            && *swizzle.components().as_slice().last().unwrap_unchecked() == 3
    }
    // Source: src/sksl/analysis/SkSLReturnsInputAlpha.cpp:76-103
    unsafe fn returnsInputAlpha(&self, expr: &dyn Expression) -> bool {
        if self.isInputVar(expr) {
            return true;
        }
        if expr.node().is::<Swizzle>() {
            return self.isInputSwizzleEndingWithAlpha(expr);
        }
        if matches!(
            expr.kind(),
            Kind::kConstructorSplat | Kind::kConstructorCompound
        ) {
            let ctor = asAnyConstructor(expr);
            return self.returnsInputAlpha(
                &**ctor
                    .argumentSpan()
                    .last()
                    .unwrap_unchecked()
                    .as_ref()
                    .unwrap_unchecked(),
            );
        }
        if expr.node().is::<ConstructorCompoundCast>() {
            let arg = expr
                .node()
                .as_node::<ConstructorCompoundCast>()
                .argument()
                .as_ref()
                .unwrap_unchecked();
            return arg.expression_type().componentType().isFloat()
                && self.returnsInputAlpha(&**arg);
        }
        if expr.node().is::<TernaryExpression>() {
            let ternary = expr.node().as_node::<TernaryExpression>();
            return self.returnsInputAlpha(&**ternary.ifTrue().as_ref().unwrap_unchecked())
                && self.returnsInputAlpha(&**ternary.ifFalse().as_ref().unwrap_unchecked());
        }
        false
    }
    // Source: src/sksl/analysis/SkSLReturnsInputAlpha.cpp:105-110
    unsafe fn visitStatement(&mut self, stmt: &dyn Statement) -> bool {
        if stmt.node().is::<ReturnStatement>() {
            return !self.returnsInputAlpha(
                &**stmt
                    .node()
                    .as_node::<ReturnStatement>()
                    .expression()
                    .as_ref()
                    .unwrap_unchecked(),
            );
        }
        SkSLProgramVisitor::inheritedStatement(self, stmt)
    }
    // Source: src/sksl/analysis/SkSLReturnsInputAlpha.cpp:112-115
    fn visitExpression(&mut self, _: &dyn Expression) -> bool {
        false
    }
}
unsafe impl ProgramVisitor for ReturnsInputAlphaVisitor<'_> {
    fn visitProgramElement(&mut self, pe: &dyn ProgramElement) -> bool {
        unsafe { Self::visitProgramElement(self, pe) }
    }
    fn visitStatement(&mut self, stmt: &dyn Statement) -> bool {
        unsafe { Self::visitStatement(self, stmt) }
    }
    fn visitExpression(&mut self, expr: &dyn Expression) -> bool {
        Self::visitExpression(self, expr)
    }
}
// Source: src/sksl/analysis/SkSLReturnsInputAlpha.cpp:126-129
pub unsafe fn ReturnsInputAlpha(function: &FunctionDefinition, usage: &ProgramUsage) -> bool {
    let mut visitor = ReturnsInputAlphaVisitor::new(usage);
    !ProgramVisitor::visitProgramElement(&mut visitor, function)
}
