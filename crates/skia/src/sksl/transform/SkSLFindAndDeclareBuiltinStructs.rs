use crate::src::sksl::{
    analysis::SkSLProgramUsage::{ProgramUsage, TypeKey},
    ir::{
        SkSLProgram::Program, SkSLProgramElement::ProgramElement,
        SkSLStructDefinition::StructDefinition,
    },
    SkSLModule::Module,
};
use core::ptr::NonNull;

// Source: src/sksl/transform/SkSLFindAndDeclareBuiltinStructs.cpp:28-36
unsafe fn contains_builtin_struct(usage: &ProgramUsage) -> bool {
    // THash's callback spelling retains the native first-match behavior.
    let mut found = false;
    usage.fStructCounts.foreach(|symbol, _count| {
        if !found && symbol.as_ref().isBuiltin() {
            found = true;
        }
    });
    found
}

// Source: src/sksl/transform/SkSLFindAndDeclareBuiltinStructs.cpp:38-59
unsafe fn get_struct_definitions_from_module(
    program: &Program,
    module: &Module,
    addedStructDefs: &mut Vec<NonNull<dyn ProgramElement>>,
) {
    if let Some(parent) = module.fParent {
        get_struct_definitions_from_module(program, parent.as_ref(), addedStructDefs);
    }
    for element in module.fElements.iter() {
        let element = element.as_ref().expect("native nonnull module element");
        if element.node().is::<StructDefinition>() {
            let structDef = element.node().as_node::<StructDefinition>();
            let key = TypeKey::from_type(structDef.r#type());
            let count = program
                .fUsage
                .as_ref()
                .unwrap_unchecked()
                .fStructCounts
                .find(&key);
            if count.is_some_and(|count| *count > 0) {
                let ptr = NonNull::from(structDef);
                addedStructDefs.push(NonNull::new_unchecked(
                    ptr.as_ptr() as *mut dyn ProgramElement
                ));
            }
        }
    }
}

// Source: src/sksl/transform/SkSLFindAndDeclareBuiltinStructs.cpp:61-76
pub unsafe fn FindAndDeclareBuiltinStructs(program: &mut Program) {
    if contains_builtin_struct(program.fUsage.as_ref().unwrap_unchecked()) {
        let mut addedStructDefs = Vec::new();
        let module = program
            .fContext
            .as_ref()
            .unwrap_unchecked()
            .fModule
            .unwrap_unchecked();
        get_struct_definitions_from_module(program, module.as_ref(), &mut addedStructDefs);
        program
            .fSharedElements
            .splice(0..0, addedStructDefs.iter().copied().map(Some));
        for element in addedStructDefs {
            program
                .fUsage
                .as_mut()
                .unwrap_unchecked()
                .addProgramElement(element.as_ref());
        }
    }
}
