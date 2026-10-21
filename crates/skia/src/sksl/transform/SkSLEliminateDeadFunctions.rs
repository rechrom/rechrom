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
// Source: src/sksl/transform/SkSLEliminateDeadFunctions.cpp:24-35
unsafe fn dead_function_predicate(element: &dyn ProgramElement, usage: &mut ProgramUsage) -> bool {
    if !element.node().is::<FunctionDefinition>() {
        return false;
    }
    let f = element.node().as_node::<FunctionDefinition>();
    if f.declaration().isMain() || usage.getFunction(f.declaration()) > 0 {
        return false;
    }
    usage.removeProgramElement(element);
    true
}
// Source: src/sksl/transform/SkSLEliminateDeadFunctions.cpp:37-61
pub unsafe fn EliminateDeadFunctions(program: &mut Program) -> bool {
    let owned = program.fOwnedElements.len();
    let shared = program.fSharedElements.len();
    if program
        .fConfig
        .as_ref()
        .unwrap_unchecked()
        .fSettings
        .fRemoveDeadFunctions
    {
        let usage = program.fUsage.as_deref_mut().unwrap_unchecked();
        program
            .fOwnedElements
            .retain(|pe| !dead_function_predicate(&**pe.as_ref().unwrap_unchecked(), usage));
        program
            .fSharedElements
            .retain(|pe| !dead_function_predicate(pe.unwrap_unchecked().as_ref(), usage));
    }
    program.fOwnedElements.len() < owned || program.fSharedElements.len() < shared
}
// Source: src/sksl/transform/SkSLEliminateDeadFunctions.cpp:63-77
pub unsafe fn EliminateDeadFunctionsForModule(
    context: &Context,
    module: &mut Module,
    usage: &mut ProgramUsage,
) -> bool {
    let count = module.fElements.len();
    if context
        .fConfig
        .unwrap_unchecked()
        .as_ref()
        .fSettings
        .fRemoveDeadFunctions
    {
        module
            .fElements
            .retain(|pe| !dead_function_predicate(&**pe.as_ref().unwrap_unchecked(), usage));
    }
    module.fElements.len() < count
}
