use crate::src::sksl::{
    ir::{
        SkSLFunctionDeclaration::FunctionDeclaration,
        SkSLFunctionDefinition::FunctionDefinition,
        SkSLIRNode::IRNode,
        SkSLProgram::{kRTFlip_Derivative, Program},
        SkSLProgramElement::ProgramElement,
        SkSLSymbol::Symbol,
    },
    SkSLIntrinsicList::IntrinsicKind,
};
use core::ptr::NonNull;

// Source: src/sksl/transform/SkSLFindAndDeclareBuiltinFunctions.cpp:31-97
// All appended pointers borrow the original immutable module definitions. Their
// owners and the active Context/config must outlive the resulting Program.
pub unsafe fn FindAndDeclareBuiltinFunctions(program: &mut Program) {
    let usage = program.fUsage.as_mut().expect("native program usage");
    let context = program.fContext.as_ref().expect("native program context");
    let interface = &mut program.fInterface;
    let mut addedBuiltins: Vec<NonNull<FunctionDefinition>> = Vec::new();
    loop {
        let numBuiltinsAtStart = addedBuiltins.len();
        usage.fCallCounts.foreach(|symbol, count| {
            // fCallCounts only contains actual final FunctionDeclaration objects;
            // SymbolData is their first native inheritance field.
            let function = &*symbol.as_ptr().cast::<FunctionDeclaration>();
            if !function.isBuiltin() || *count == 0 {
                return;
            }
            if function.intrinsicKind() == IntrinsicKind::k_dFdy_IntrinsicKind {
                if !context
                    .fConfig
                    .unwrap_unchecked()
                    .as_ref()
                    .fSettings
                    .fForceNoRTFlip
                {
                    interface.fRTFlipUniform |= kRTFlip_Derivative;
                }
            }
            if let Some(definition) = function.definition() {
                let definition = NonNull::from(definition);
                if !addedBuiltins.contains(&definition) {
                    addedBuiltins.push(definition);
                }
            }
        });
        if addedBuiltins.len() == numBuiltinsAtStart {
            break;
        }
        addedBuiltins[numBuiltinsAtStart..].sort_unstable_by(|a, b| {
            let a = a.as_ref().declaration();
            let b = b.as_ref().declaration();
            if a.name() != b.name() {
                b.name().cmp(a.name())
            } else {
                b.description().cmp(&a.description())
            }
        });
        let usageCallCounts = usage.fCallCounts.count();
        for definition in &addedBuiltins[numBuiltinsAtStart..] {
            usage.addProgramElement(definition.as_ref());
        }
        if usage.fCallCounts.count() == usageCallCounts {
            break;
        }
    }
    program.fSharedElements.splice(
        0..0,
        addedBuiltins.into_iter().rev().map(|definition| {
            Some(NonNull::new_unchecked(
                definition.as_ptr() as *mut dyn ProgramElement
            ))
        }),
    );
}
