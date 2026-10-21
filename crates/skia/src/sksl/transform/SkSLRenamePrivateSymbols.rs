// Unverified direct translation; native symbol/IR allocation leases are required.
use crate::src::sksl::{
    analysis::SkSLProgramUsage::ProgramUsage,
    ir::{
        SkSLFunctionDeclaration::FunctionDeclaration, SkSLFunctionDefinition::FunctionDefinition,
        SkSLFunctionPrototype::FunctionPrototype, SkSLIRNode::*, SkSLModifierFlags::ModifierFlag,
        SkSLProgramElement::ProgramElement, SkSLSymbol::Symbol, SkSLSymbolTable::SymbolTable,
        SkSLVarDeclarations::VarDeclaration, SkSLVariable::Variable,
    },
    transform::SkSLProgramWriter::{self, ProgramWriter, UniquePtrStatement},
    SkSLAnalysis::SymbolTableStackBuilder,
    SkSLContext::Context,
    SkSLModule::Module,
    SkSLProgramKind::ProgramKind,
    SkSLProgramSettings::ProgramConfig,
};
use core::{pin::Pin, ptr::NonNull};
// Source: src/sksl/transform/SkSLRenamePrivateSymbols.cpp:42-56
unsafe fn strip_export_flag(
    _context: &Context,
    funcDecl: &FunctionDeclaration,
    symbols: &mut SymbolTable,
) {
    let mut sym = symbols
        .findMutable(funcDecl.name())
        .map(|s| NonNull::new_unchecked(s.as_ptr() as *mut () as *mut FunctionDeclaration));
    while let Some(mut p) = sym {
        let decl = p.as_mut();
        let mut flags = decl.modifierFlags();
        flags.0 &= !ModifierFlag::kExport.0;
        decl.setModifierFlags(flags);
        sym = decl.mutableNextOverload().map(NonNull::from);
    }
}
// Source: src/sksl/transform/SkSLRenamePrivateSymbols.cpp:58-62
pub unsafe fn RenamePrivateSymbols(
    context: &Context,
    module: &mut Module,
    usage: &mut ProgramUsage,
    kind: ProgramKind,
) {
    // Source: src/sksl/transform/SkSLRenamePrivateSymbols.cpp:219-222
    // Rust contiguous field group. Stack guard holds a raw borrowed stack address;
    // nested guards end before the writer moves or its stack is destroyed.
    struct SymbolRenamer<'a> {
        fContext: &'a Context,
        fUsage: *mut ProgramUsage,
        fSymbolTableStack: Vec<NonNull<SymbolTable>>,
        fKind: ProgramKind,
    }
    impl<'a> SymbolRenamer<'a> {
        // Source: src/sksl/transform/SkSLRenamePrivateSymbols.cpp:64-71
        fn new(
            context: &'a Context,
            usage: *mut ProgramUsage,
            symbolBase: NonNull<SymbolTable>,
            kind: ProgramKind,
        ) -> Self {
            Self {
                fContext: context,
                fUsage: usage,
                fSymbolTableStack: vec![symbolBase],
                fKind: kind,
            }
        }
        // Source: src/sksl/transform/SkSLRenamePrivateSymbols.cpp:73-103
        unsafe fn FindShortNameForSymbol(
            sym: &dyn Symbol,
            symbolTable: &SymbolTable,
            namePrefix: &[u8],
        ) -> Vec<u8> {
            const LETTERS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
            for &letter in LETTERS {
                let mut name = namePrefix.to_vec();
                name.push(letter);
                if symbolTable.find(&name).is_none() {
                    return name;
                }
            }
            for &a in LETTERS {
                for &b in LETTERS {
                    let mut name = namePrefix.to_vec();
                    name.push(a);
                    name.push(b);
                    if symbolTable.find(&name).is_none() {
                        return name;
                    }
                }
            }
            debug_assert!(
                false,
                "Unable to find unique name for '{}'",
                String::from_utf8_lossy(sym.name())
            );
            sym.name().to_vec()
        }
        // Source: src/sksl/transform/SkSLRenamePrivateSymbols.cpp:105-132
        unsafe fn minifyVariableName(&mut self, var: &dyn Variable) {
            if var.name().is_empty() {
                return;
            }
            let mut symbols = *self.fSymbolTableStack.last().unwrap_unchecked();
            let mutableSym = symbols.as_ref().findMutable(var.name());
            debug_assert!(mutableSym.is_some());
            debug_assert!(core::ptr::addr_eq(
                mutableSym.unwrap_unchecked().as_ptr(),
                var as *const dyn Variable
            ));
            let shortName = Self::FindShortNameForSymbol(var, symbols.as_ref(), b"");
            debug_assert!(symbols.as_ref().findMutable(&shortName).is_none());
            let owned = symbols.as_mut().takeOwnershipOfString(shortName);
            symbols.as_mut().renameSymbol(
                self.fContext,
                mutableSym.unwrap_unchecked(),
                owned.as_ref(),
            );
        }
        // Source: src/sksl/transform/SkSLRenamePrivateSymbols.cpp:134-149
        unsafe fn minifyFunctionName(&mut self, decl: &FunctionDeclaration) {
            let prefix: &[u8] = if ProgramConfig::IsRuntimeEffect(self.fKind) {
                b""
            } else {
                b"$"
            };
            let mut symbols = *self.fSymbolTableStack.last().unwrap_unchecked();
            let shortName = Self::FindShortNameForSymbol(decl, symbols.as_ref(), prefix);
            debug_assert!(symbols.as_ref().findMutable(&shortName).is_none());
            if shortName.len() < decl.name().len() {
                let sym = symbols.as_ref().findMutable(decl.name()).unwrap_unchecked();
                let owned = symbols.as_mut().takeOwnershipOfString(shortName);
                symbols
                    .as_mut()
                    .renameSymbol(self.fContext, sym, owned.as_ref());
            }
        }
        // Source: src/sksl/transform/SkSLRenamePrivateSymbols.cpp:151-160
        fn functionNameCanBeMinifiedSafely(&self, decl: &FunctionDeclaration) -> bool {
            if ProgramConfig::IsRuntimeEffect(self.fKind) {
                !decl.isMain()
            } else {
                decl.name().starts_with(b"$") && !decl.modifierFlags().isExport()
            }
        }
        // Source: src/sksl/transform/SkSLRenamePrivateSymbols.cpp:162-175
        unsafe fn minifyFunction(&mut self, def: &FunctionDefinition) {
            let decl = def.declaration();
            if self.functionNameCanBeMinifiedSafely(decl) {
                self.minifyFunctionName(decl);
            }
            let _guard = SymbolTableStackBuilder::new(
                def.body().as_deref(),
                NonNull::from(&mut self.fSymbolTableStack),
            );
            for p in decl.parameters() {
                self.minifyVariableName(p.unwrap_unchecked().as_ref());
            }
        }
        // Source: src/sksl/transform/SkSLRenamePrivateSymbols.cpp:177-190
        unsafe fn minifyPrototype(&mut self, proto: &FunctionPrototype) {
            let decl = proto.declaration();
            if decl.definition().is_some() {
                return;
            }
            for p in decl.parameters() {
                let mut p = p.unwrap_unchecked();
                Pin::new_unchecked(p.as_mut()).setName(b"");
            }
        }
    }
    unsafe impl ProgramWriter for SymbolRenamer<'_> {
        // Source: src/sksl/transform/SkSLRenamePrivateSymbols.cpp:192-205
        unsafe fn visitProgramElement(&mut self, elem: Pin<&mut dyn ProgramElement>) -> bool {
            match elem.kind() {
                ProgramElementKind::kFunction => {
                    self.minifyFunction(elem.node().as_node::<FunctionDefinition>());
                    SkSLProgramWriter::inheritedProgramElement(self, elem)
                }
                ProgramElementKind::kFunctionPrototype => {
                    self.minifyPrototype(elem.node().as_node::<FunctionPrototype>());
                    SkSLProgramWriter::inheritedProgramElement(self, elem)
                }
                _ => false,
            }
        }
        // Source: src/sksl/transform/SkSLRenamePrivateSymbols.cpp:207-217
        unsafe fn visitStatementPtr(&mut self, stmt: &mut UniquePtrStatement) -> bool {
            let _guard = SymbolTableStackBuilder::new(
                stmt.as_deref(),
                NonNull::from(&mut self.fSymbolTableStack),
            );
            if stmt
                .as_ref()
                .unwrap_unchecked()
                .node()
                .is::<VarDeclaration>()
            {
                let var = stmt
                    .as_ref()
                    .unwrap_unchecked()
                    .node()
                    .as_node::<VarDeclaration>()
                    .var()
                    .unwrap_unchecked();
                self.minifyVariableName(var.as_ref());
            }
            SkSLProgramWriter::inheritedStatement(
                self,
                stmt.as_mut().unwrap_unchecked().as_pin_mut(),
            )
        }
    }
    // Source: src/sksl/transform/SkSLRenamePrivateSymbols.cpp:227-230
    let mut renamer = SymbolRenamer::new(
        context,
        usage,
        NonNull::from(module.fSymbols.as_deref_mut().unwrap_unchecked()),
        kind,
    );
    for pe in module.fElements.iter_mut() {
        renamer.visitProgramElement(pe.as_mut().unwrap_unchecked().as_pin_mut());
    }
    // Source: src/sksl/transform/SkSLRenamePrivateSymbols.cpp:234-241
    for pe in module.fElements.iter() {
        let pe = pe.as_ref().unwrap_unchecked();
        if pe.node().is::<FunctionDefinition>() {
            let decl = pe.node().as_node::<FunctionDefinition>().declaration();
            if decl.modifierFlags().isExport() {
                strip_export_flag(
                    context,
                    decl,
                    module.fSymbols.as_deref_mut().unwrap_unchecked(),
                );
            }
        }
    }
}
