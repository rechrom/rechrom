use crate::src::sksl::{
    analysis::SkSLProgramUsage::ProgramUsage,
    ir::{
        SkSLExpression::Expression,
        SkSLModifierFlags::{ModifierFlag, ModifierFlags},
        SkSLVariable::Variable,
    },
    SkSLAnalysis::IsCompileTimeConstant,
};
// Source: src/sksl/transform/SkSLAddConstToVarModifiers.cpp:19-38
pub unsafe fn AddConstToVarModifiers(
    var: &dyn Variable,
    initialValue: Option<&dyn Expression>,
    usage: &ProgramUsage,
) -> ModifierFlags {
    let flags = var.modifierFlags();
    if flags.isConst() {
        return flags;
    }
    if !initialValue.is_some_and(|value| IsCompileTimeConstant(value)) {
        return flags;
    }
    let counts = usage.getVariable(var);
    if counts.fWrite != 1 {
        return flags;
    }
    flags | ModifierFlag::kConst
}
