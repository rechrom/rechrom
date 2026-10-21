// UNVERIFIED: validation prohibited by user.
use crate::src::sksl::{
    analysis::SkSLProgramUsage::ProgramUsage,
    ir::{
        SkSLFunctionDefinition::FunctionDefinition, SkSLProgram::Program,
        SkSLProgramElement::ProgramElement, SkSLSymbol::Symbol,
        SkSLVarDeclarations::GlobalVarDeclaration,
    },
    SkSLContext::Context,
    SkSLModule::Module,
};
// Source: src/sksl/transform/SkSLEliminateDeadGlobalVariables.cpp:25-44
unsafe fn is_dead_variable(
    element: &dyn ProgramElement,
    usage: &mut ProgramUsage,
    onlyPrivateGlobals: bool,
) -> bool {
    if !element.node().is::<GlobalVarDeclaration>() {
        return false;
    }
    let decl = element
        .node()
        .as_node::<GlobalVarDeclaration>()
        .varDeclaration();
    let var = decl.var().unwrap_unchecked().as_ref();
    if onlyPrivateGlobals && !var.name().starts_with(b"$") {
        return false;
    }
    if !usage.isDead(var) {
        return false;
    }
    usage.removeStatement(decl);
    true
}
// Source: src/sksl/transform/SkSLEliminateDeadGlobalVariables.cpp:46-65
pub unsafe fn EliminateDeadGlobalVariablesForModule(
    context: &Context,
    module: &mut Module,
    usage: &mut ProgramUsage,
    onlyPrivateGlobals: bool,
) -> bool {
    let count = module.fElements.len();
    if context
        .fConfig
        .unwrap_unchecked()
        .as_ref()
        .fSettings
        .fRemoveDeadVariables
    {
        module.fElements.retain(|pe| {
            !is_dead_variable(&**pe.as_ref().unwrap_unchecked(), usage, onlyPrivateGlobals)
        });
    }
    module.fElements.len() < count
}
// Source: src/sksl/transform/SkSLEliminateDeadGlobalVariables.cpp:67-88
pub unsafe fn EliminateDeadGlobalVariables(program: &mut Program) -> bool {
    let owned = program.fOwnedElements.len();
    let shared = program.fSharedElements.len();
    if program
        .fConfig
        .as_ref()
        .unwrap_unchecked()
        .fSettings
        .fRemoveDeadVariables
    {
        let usage = program.fUsage.as_deref_mut().unwrap_unchecked();
        program
            .fOwnedElements
            .retain(|pe| !is_dead_variable(&**pe.as_ref().unwrap_unchecked(), usage, false));
        program
            .fSharedElements
            .retain(|pe| !is_dead_variable(pe.unwrap_unchecked().as_ref(), usage, false));
    }
    program.fOwnedElements.len() < owned || program.fSharedElements.len() < shared
}
