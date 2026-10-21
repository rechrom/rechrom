// UNVERIFIED: frozen approved source, no validation is permitted.
use super::SkSLProgramUsage::ObjectHash;
use crate::src::core::SkTHash::THashSet;
use crate::src::sksl::{
    ir::{
        SkSLBinaryExpression::BinaryExpression,
        SkSLExpression::{Expression, ExpressionExt, Kind},
        SkSLForStatement::ForStatement,
        SkSLIRNode::IRNode,
        SkSLIndexExpression::IndexExpression,
        SkSLStatement::Statement,
        SkSLVarDeclarations::VarDeclaration,
        SkSLVariable::{Storage, Variable, VariableData},
        SkSLVariableReference::VariableReference,
    },
    SkSLErrorReporter::{ErrorReporter, ErrorReporterExt},
    SkSLOperator::OperatorKind,
};
use core::ptr::NonNull;
type LoopIndices = THashSet<NonNull<VariableData>, ObjectHash>;
// Source: src/sksl/analysis/SkSLIsConstantExpression.cpp:37-37
// Source: src/sksl/analysis/SkSLIsConstantExpression.cpp:112-112
struct ConstantExpressionVisitor<'a> {
    fLoopIndices: Option<&'a LoopIndices>,
}
impl<'a> ConstantExpressionVisitor<'a> {
    // Source: src/sksl/analysis/SkSLIsConstantExpression.cpp:39-40
    fn new(indices: Option<&'a LoopIndices>) -> Self {
        Self {
            fLoopIndices: indices,
        }
    }
    // Source: src/sksl/analysis/SkSLIsConstantExpression.cpp:42-109
    unsafe fn visitExpression(&mut self, e: &dyn Expression) -> bool {
        match e.kind() {
            Kind::kLiteral => false,
            Kind::kSetting => false,
            Kind::kVariableReference => {
                let v = e
                    .node()
                    .as_node::<VariableReference>()
                    .variable()
                    .unwrap_unchecked();
                let v = v.as_ref();
                if v.modifierFlags().isConst()
                    && (v.storage() == Storage::kGlobal || v.storage() == Storage::kLocal)
                {
                    return false;
                }
                self.fLoopIndices.map_or(true, |indices| {
                    !indices.contains(&NonNull::from(v.variable_data()))
                })
            }
            Kind::kBinary => {
                if e.node().as_node::<BinaryExpression>().getOperator().kind()
                    == OperatorKind::COMMA
                {
                    return true;
                }
                super::SkSLProgramVisitor::visitExpression(
                    &mut |child| self.visitExpression(child),
                    e,
                )
            }
            Kind::kConstructorArray
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
            | Kind::kPrefix
            | Kind::kPostfix
            | Kind::kSwizzle
            | Kind::kTernary => super::SkSLProgramVisitor::visitExpression(
                &mut |child| self.visitExpression(child),
                e,
            ),
            Kind::kFunctionCall
            | Kind::kChildCall
            | Kind::kPoison
            | Kind::kFunctionReference
            | Kind::kMethodReference
            | Kind::kTypeReference
            | Kind::kEmpty => true,
            _ => {
                crate::SkDEBUGFAIL!(@source b"src/sksl/analysis/SkSLIsConstantExpression.cpp\0".as_ptr(),106; b"Unexpected expression type\0".as_ptr());
                true
            }
        }
    }
}
// Source: src/sksl/analysis/SkSLIsConstantExpression.cpp:118-118
// Source: src/sksl/analysis/SkSLIsConstantExpression.cpp:150-151
struct ES2IndexingVisitor<'a> {
    fErrors: &'a mut dyn ErrorReporter,
    fLoopIndices: LoopIndices,
}
impl<'a> ES2IndexingVisitor<'a> {
    // Source: src/sksl/analysis/SkSLIsConstantExpression.cpp:120-120
    fn new(errors: &'a mut dyn ErrorReporter) -> Self {
        Self {
            fErrors: errors,
            fLoopIndices: LoopIndices::default(),
        }
    }
    // Source: src/sksl/analysis/SkSLIsConstantExpression.cpp:122-134
    unsafe fn visitStatement(&mut self, s: &dyn Statement) -> bool {
        if s.node().is::<ForStatement>() {
            let f = s.node().as_node::<ForStatement>();
            debug_assert!(f
                .initializer()
                .as_ref()
                .is_some_and(|init| init.node().is::<VarDeclaration>()));
            let var = f
                .initializer()
                .as_ref()
                .unwrap_unchecked()
                .node()
                .as_node::<VarDeclaration>()
                .var()
                .unwrap_unchecked();
            let var = NonNull::from(var.as_ref().variable_data());
            debug_assert!(!self.fLoopIndices.contains(&var));
            self.fLoopIndices.add(var);
            let result = self.visitStatement(&**f.statement().as_ref().unwrap_unchecked());
            self.fLoopIndices.remove(&var);
            return result;
        }
        let this = self as *mut Self;
        super::SkSLProgramVisitor::visitStatement(
            &mut |statement| (*this).visitStatement(statement),
            &mut |expr| (*this).visitExpression(expr),
            s,
        )
    }
    // Source: src/sksl/analysis/SkSLIsConstantExpression.cpp:136-145
    unsafe fn visitExpression(&mut self, e: &dyn Expression) -> bool {
        if e.node().is::<IndexExpression>() {
            let index = e.node().as_node::<IndexExpression>();
            if ConstantExpressionVisitor::new(Some(&self.fLoopIndices))
                .visitExpression(&**index.index().as_ref().unwrap_unchecked())
            {
                self.fErrors
                    .error(index.node().fPosition, b"index expression must be constant");
                return true;
            }
        }
        super::SkSLProgramVisitor::visitExpression(&mut |expr| self.visitExpression(expr), e)
    }
}
// Actual native virtual-channel dispatch: inherited program-element recursion is
// now closed by the admitted FunctionDefinition header and GlobalVarDeclaration.
unsafe impl super::SkSLProgramVisitor::ProgramVisitor for ES2IndexingVisitor<'_> {
    fn visitExpression(&mut self, e: &dyn Expression) -> bool {
        unsafe { Self::visitExpression(self, e) }
    }
    fn visitStatement(&mut self, s: &dyn Statement) -> bool {
        unsafe { Self::visitStatement(self, s) }
    }
    fn visitProgramElement(
        &mut self,
        pe: &dyn crate::src::sksl::ir::SkSLProgramElement::ProgramElement,
    ) -> bool {
        unsafe { super::SkSLProgramVisitor::inheritedProgramElement(self, pe) }
    }
}
// Source: src/sksl/analysis/SkSLIsConstantExpression.cpp:157-159
pub unsafe fn IsConstantExpression(expr: &dyn Expression) -> bool {
    !ConstantExpressionVisitor::new(None).visitExpression(expr)
}

// Source: src/sksl/analysis/SkSLIsConstantExpression.cpp:161-164
pub fn ValidateIndexingForES2(
    pe: &dyn crate::src::sksl::ir::SkSLProgramElement::ProgramElement,
    errors: &mut dyn ErrorReporter,
) {
    let mut visitor = ES2IndexingVisitor::new(errors);
    super::SkSLProgramVisitor::ProgramVisitor::visitProgramElement(&mut visitor, pe);
}
