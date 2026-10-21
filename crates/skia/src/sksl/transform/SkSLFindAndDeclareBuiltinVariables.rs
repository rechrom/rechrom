use crate::src::sksl::{
    ir::{
        SkSLFunctionDefinition::FunctionDefinition,
        SkSLIRNode::IRNodeBox,
        SkSLInterfaceBlock::InterfaceBlock,
        SkSLProgram::{kRTFlip_Clockwise, kRTFlip_FragCoord, Program},
        SkSLProgramElement::{Kind, ProgramElement},
        SkSLSymbol::{Kind as SymbolKind, Symbol},
        SkSLSymbolTable::SymbolTable,
        SkSLVarDeclarations::GlobalVarDeclaration,
        SkSLVariable::{Storage, Variable},
    },
    SkSLCompiler::{
        FRAGCOLOR_NAME, SK_CLOCKWISE_BUILTIN, SK_FRAGCOORD_BUILTIN, SK_LASTFRAGCOLOR_BUILTIN,
        SK_SECONDARYFRAGCOLOR_BUILTIN,
    },
    SkSLContext::Context,
    SkSLProgramSettings::ProgramConfig,
};
use core::ptr::NonNull;

// Source: src/sksl/transform/SkSLFindAndDeclareBuiltinVariables.cpp:39-43
// Source: src/sksl/transform/SkSLFindAndDeclareBuiltinVariables.cpp:120-123
// Native reference members borrow the same program context and symbol owner.
struct BuiltinVariableScanner<'a> {
    fContext: &'a Context,
    fSymbols: &'a SymbolTable,
    fNewElements: Vec<NonNull<dyn ProgramElement>>,
}
impl<'a> BuiltinVariableScanner<'a> {
    fn new(context: &'a Context, symbols: &'a SymbolTable) -> Self {
        Self {
            fContext: context,
            fSymbols: symbols,
            fNewElements: Vec::new(),
        }
    }
    // Source: src/sksl/transform/SkSLFindAndDeclareBuiltinVariables.cpp:45-51
    fn addDeclaringElement(&mut self, declaration: NonNull<dyn ProgramElement>) {
        if !self
            .fNewElements
            .iter()
            .any(|old| core::ptr::addr_eq(old.as_ptr(), declaration.as_ptr()))
        {
            self.fNewElements.push(declaration);
        }
    }
    // Source: src/sksl/transform/SkSLFindAndDeclareBuiltinVariables.cpp:53-69
    // Rust spells the native Symbol downcast and Variable body separately, so
    // the usage-map's actual Variable trait pointer needs no fabricated vtable.
    unsafe fn addSymbol(&mut self, symbol: Option<NonNull<dyn Symbol>>) {
        let Some(symbol) = symbol else {
            return;
        };
        let symbol = symbol.as_ref();
        if symbol.kind() != SymbolKind::kVariable {
            return;
        }
        self.addVariable(symbol.as_variable_symbol());
    }
    unsafe fn addVariable(&mut self, var: &dyn Variable) {
        if let Some(declaration) = var.globalVarDeclaration() {
            self.addDeclaringElement(NonNull::new_unchecked(
                declaration.as_ptr() as *mut dyn ProgramElement
            ));
        } else if let Some(block) = var.interfaceBlock() {
            self.addDeclaringElement(NonNull::new_unchecked(
                block.as_ptr() as *mut dyn ProgramElement
            ));
        } else {
            debug_assert!(
                var.storage() != Storage::kGlobal && var.storage() != Storage::kInterfaceBlock
            );
        }
    }
    // Source: src/sksl/transform/SkSLFindAndDeclareBuiltinVariables.cpp:71-87
    unsafe fn addImplicitFragColorWrite(
        &mut self,
        elements: &[Option<IRNodeBox<dyn ProgramElement>>],
    ) {
        for element in elements {
            let element = element.as_ref().expect("native nonnull program element");
            if !element.node().is::<FunctionDefinition>() {
                continue;
            }
            let definition = element.node().as_node::<FunctionDefinition>();
            if definition.declaration().isMain() {
                if definition
                    .declaration()
                    .returnType()
                    .matches(&**self.fContext.fTypes.as_ref().fHalf4)
                {
                    self.addSymbol(
                        self.fSymbols
                            .findBuiltinSymbol(&FRAGCOLOR_NAME[..FRAGCOLOR_NAME.len() - 1]),
                    );
                }
                break;
            }
        }
    }
    // Source: src/sksl/transform/SkSLFindAndDeclareBuiltinVariables.cpp:89-91
    unsafe fn GlobalVarBuiltinName(element: &dyn ProgramElement) -> &[u8] {
        element
            .node()
            .as_node::<GlobalVarDeclaration>()
            .varDeclaration()
            .var()
            .unwrap_unchecked()
            .as_ref()
            .name()
    }
    // Source: src/sksl/transform/SkSLFindAndDeclareBuiltinVariables.cpp:93-95
    unsafe fn InterfaceBlockName(element: &dyn ProgramElement) -> &[u8] {
        element.node().as_node::<InterfaceBlock>().instanceName()
    }
    // Source: src/sksl/transform/SkSLFindAndDeclareBuiltinVariables.cpp:97-118
    unsafe fn sortNewElements(&mut self) {
        self.fNewElements.sort_unstable_by(|a, b| {
            let (aRef, bRef) = (a.as_ref(), b.as_ref());
            if aRef.kind() != bRef.kind() {
                return (aRef.kind() as i32).cmp(&(bRef.kind() as i32));
            }
            match aRef.kind() {
                Kind::kGlobalVar => {
                    let (aName, bName) = (
                        Self::GlobalVarBuiltinName(aRef),
                        Self::GlobalVarBuiltinName(bRef),
                    );
                    debug_assert!(core::ptr::addr_eq(a.as_ptr(), b.as_ptr()) || aName != bName);
                    aName.cmp(bName)
                }
                Kind::kInterfaceBlock => {
                    let (aName, bName) = (
                        Self::InterfaceBlockName(aRef),
                        Self::InterfaceBlockName(bRef),
                    );
                    debug_assert!(core::ptr::addr_eq(a.as_ptr(), b.as_ptr()) || aName != bName);
                    aName.cmp(bName)
                }
                _ => core::hint::unreachable_unchecked(),
            }
        });
    }
}

// Source: src/sksl/transform/SkSLFindAndDeclareBuiltinVariables.cpp:127-184
pub unsafe fn FindAndDeclareBuiltinVariables(program: &mut Program) {
    let context = program.fContext.as_ref().expect("native program context");
    let symbols = program.fSymbols.as_ref().expect("native program symbols");
    let mut scanner = BuiltinVariableScanner::new(context, symbols);
    if ProgramConfig::IsFragment(program.fConfig.as_ref().unwrap_unchecked().fKind) {
        scanner.addImplicitFragColorWrite(&program.fOwnedElements);
    }
    let interface = &mut program.fInterface;
    program
        .fUsage
        .as_ref()
        .unwrap_unchecked()
        .fVariableCounts
        .foreach(|var, _counts| {
            let var = var.as_ref();
            if var.isBuiltin() {
                scanner.addVariable(var);
                match var.layout().fBuiltin {
                    SK_FRAGCOORD_BUILTIN => {
                        if !context
                            .fConfig
                            .unwrap_unchecked()
                            .as_ref()
                            .fSettings
                            .fForceNoRTFlip
                        {
                            interface.fRTFlipUniform |= kRTFlip_FragCoord;
                        }
                    }
                    SK_CLOCKWISE_BUILTIN => {
                        if !context
                            .fConfig
                            .unwrap_unchecked()
                            .as_ref()
                            .fSettings
                            .fForceNoRTFlip
                        {
                            interface.fRTFlipUniform |= kRTFlip_Clockwise;
                        }
                    }
                    SK_LASTFRAGCOLOR_BUILTIN => interface.fUseLastFragColor = true,
                    SK_SECONDARYFRAGCOLOR_BUILTIN => interface.fOutputSecondaryColor = true,
                    _ => {}
                }
            }
        });
    scanner.sortNewElements();
    program
        .fSharedElements
        .splice(0..0, scanner.fNewElements.iter().copied().map(Some));
    for element in scanner.fNewElements {
        program
            .fUsage
            .as_mut()
            .unwrap_unchecked()
            .addProgramElement(element.as_ref());
    }
}
