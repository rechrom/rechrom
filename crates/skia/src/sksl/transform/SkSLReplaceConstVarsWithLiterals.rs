use crate::src::{
    core::SkTHash::THashSetStorage,
    sksl::{
        analysis::SkSLProgramUsage::{ObjectHash, ProgramUsage, VariableKey},
        ir::{
            SkSLExpression::Expression, SkSLFunctionDefinition::FunctionDefinition, SkSLIRNode::*,
            SkSLSymbol::Symbol, SkSLVariableReference::VariableReference,
        },
        transform::SkSLProgramWriter::{self, ProgramWriter, UniquePtrExpression},
        SkSLConstantFolder::ConstantFolder,
        SkSLModule::Module,
    },
};
// Source: src/sksl/transform/SkSLReplaceConstVarsWithLiterals.cpp:32-33
pub unsafe fn ReplaceConstVarsWithLiterals(module: &mut Module, usage: &mut ProgramUsage) {
    // Source: src/sksl/transform/SkSLReplaceConstVarsWithLiterals.cpp:58-59
    struct ConstVarReplacer {
        fUsage: *mut ProgramUsage,
        fCandidates: THashSetStorage<VariableKey, ObjectHash>,
    }
    impl ConstVarReplacer {
        // Source: src/sksl/transform/SkSLReplaceConstVarsWithLiterals.cpp:35-35
        fn new(usage: *mut ProgramUsage) -> Self {
            Self {
                fUsage: usage,
                fCandidates: THashSetStorage::default(),
            }
        }
    }
    unsafe impl ProgramWriter for ConstVarReplacer {
        // Source: src/sksl/transform/SkSLReplaceConstVarsWithLiterals.cpp:39-56
        unsafe fn visitExpressionPtr(&mut self, expr: &mut UniquePtrExpression) -> bool {
            if expr
                .as_ref()
                .unwrap_unchecked()
                .node()
                .is::<VariableReference>()
            {
                let var = expr
                    .as_ref()
                    .unwrap_unchecked()
                    .node()
                    .as_node::<VariableReference>();
                if self.fCandidates.contains(&VariableKey::from_variable(
                    var.variable().unwrap_unchecked().as_ref(),
                )) {
                    if let Some(value) = ConstantFolder::GetConstantValueOrNull(var) {
                        // Keep the constant-value native borrow across usage removal, then clone
                        // before replacing its referencing unique_ptr. Only counts are changed.
                        let value = core::ptr::NonNull::from(value);
                        (*self.fUsage).removeExpression(&**expr.as_ref().unwrap_unchecked());
                        *expr = value.as_ref().clone_current();
                        (*self.fUsage).addExpression(&**expr.as_ref().unwrap_unchecked());
                        return false;
                    }
                }
            }
            SkSLProgramWriter::inheritedExpression(
                self,
                expr.as_mut().unwrap_unchecked().as_pin_mut(),
            )
        }
    }
    // Source: src/sksl/transform/SkSLReplaceConstVarsWithLiterals.cpp:64-94
    let mut visitor = ConstVarReplacer::new(usage);
    usage.fVariableCounts.foreach(|var, count| {
        let var = var.as_ref();
        if count.fVarExists == 0 || count.fWrite != 1 {
            return;
        }
        if !var.modifierFlags().isConst() {
            return;
        }
        if var.initialValue().is_none() {
            return;
        }
        let initialvalueSize =
            ConstantFolder::GetConstantValueForVariable(var.initialValue().unwrap_unchecked())
                .description()
                .len();
        let totalOldSize = var.description().len()
            + 1
            + initialvalueSize
            + 1
            + (count.fRead as usize) * var.name().len();
        let totalNewSize = (count.fRead as usize) * initialvalueSize;
        if totalNewSize <= totalOldSize {
            visitor.fCandidates.add(VariableKey::from_variable(var));
        }
    });
    // Source: src/sksl/transform/SkSLReplaceConstVarsWithLiterals.cpp:96-102
    if !visitor.fCandidates.empty() {
        for pe in module.fElements.iter_mut() {
            let pe = pe.as_mut().unwrap_unchecked();
            if pe.node().is::<FunctionDefinition>() {
                visitor.visitProgramElement(pe.as_pin_mut());
            }
        }
    }
}
