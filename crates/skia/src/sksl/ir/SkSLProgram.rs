use super::{
    SkSLIRNode::IRNodeBox, SkSLProgramElement::ProgramElement, SkSLSymbolTable::SymbolTable,
    SkSLType::NumberKind,
};
use crate::src::sksl::{
    analysis::SkSLProgramUsage::ProgramUsage,
    SkSLContext::Context,
    SkSLPool::{AutoAttachPoolToThread, Pool},
    SkSLProgramSettings::ProgramConfig,
};
use core::{mem::ManuallyDrop, ptr::NonNull};
use std::sync::Arc;
// Source: src/sksl/ir/SkSLProgram.h:19-19
pub const SKSL_RTFLIP_NAME: &[u8] = b"u_skRTFlip";
// Source: src/sksl/ir/SkSLProgram.h:33-38
#[repr(C)]
pub struct UniformInfoUniform {
    pub fName: Vec<u8>,
    pub fKind: NumberKind,
    pub fColumns: i32,
    pub fRows: i32,
    pub fSlot: i32,
}
// Source: src/sksl/ir/SkSLProgram.h:40-41
#[repr(C)]
pub struct UniformInfo {
    pub fUniforms: Vec<UniformInfoUniform>,
    pub fUniformSlotCount: i32,
}
impl Default for UniformInfo {
    fn default() -> Self {
        Self {
            fUniforms: Vec::new(),
            fUniformSlotCount: 0,
        }
    }
}
// Source: src/sksl/ir/SkSLProgram.h:46-51
pub type RTFlip = u8;
pub const kRTFlip_None: u8 = 0;
pub const kRTFlip_FragCoord: u8 = 1;
pub const kRTFlip_Clockwise: u8 = 2;
pub const kRTFlip_Derivative: u8 = 4;
// Source: src/sksl/ir/SkSLProgram.h:52-54
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ProgramInterface {
    pub fRTFlipUniform: u8,
    pub fUseLastFragColor: bool,
    pub fOutputSecondaryColor: bool,
}
impl PartialEq for ProgramInterface {
    // Source: src/sksl/ir/SkSLProgram.h:56-60
    fn eq(&self, that: &Self) -> bool {
        self.fRTFlipUniform == that.fRTFlipUniform
            && self.fUseLastFragColor == that.fUseLastFragColor
            && self.fOutputSecondaryColor == that.fOutputSecondaryColor
    }
    // Source: src/sksl/ir/SkSLProgram.h:61-61
    fn ne(&self, that: &Self) -> bool {
        !self.eq(that)
    }
}
impl Eq for ProgramInterface {}
// Source: src/sksl/ir/SkSLProgram.h:65-65
// Source: src/sksl/ir/SkSLProgram.h:152-165
// Complete native field ownership; constructor below computes actual usage.
#[repr(C)]
pub struct Program {
    pub(crate) fSource: ManuallyDrop<Option<Box<Vec<u8>>>>,
    pub(crate) fConfig: ManuallyDrop<Option<Box<ProgramConfig>>>,
    pub(crate) fContext: ManuallyDrop<Option<Arc<Context>>>,
    pub(crate) fUsage: ManuallyDrop<Option<Box<ProgramUsage>>>,
    pub(crate) fSymbols: ManuallyDrop<Option<Box<SymbolTable>>>,
    pub(crate) fPool: ManuallyDrop<Option<Box<Pool>>>,
    pub(crate) fOwnedElements: ManuallyDrop<Vec<Option<IRNodeBox<dyn ProgramElement>>>>,
    pub(crate) fSharedElements: ManuallyDrop<Vec<Option<NonNull<dyn ProgramElement>>>>,
    pub(crate) fInterface: ProgramInterface,
}
// Source: src/sksl/ir/SkSLProgram.h:77-77
// Source: src/sksl/ir/SkSLProgram.h:112-115
// Two slice ranges retain native owned/shared current/end state; no copied IR nodes.
#[derive(Clone)]
pub struct ElementsIterator<'a> {
    fOwned: std::slice::Iter<'a, Option<IRNodeBox<dyn ProgramElement>>>,
    fShared: std::slice::Iter<'a, Option<NonNull<dyn ProgramElement>>>,
}
impl<'a> ElementsIterator<'a> {
    // Source: src/sksl/ir/SkSLProgram.h:79-85
    pub fn value(&self) -> Option<&'a dyn ProgramElement> {
        if !self.fShared.as_slice().is_empty() {
            self.fShared.as_slice()[0].map(|p| unsafe { p.as_ref() })
        } else {
            self.fOwned.as_slice()[0].as_ref().map(|p| &**p)
        }
    }
    // Source: src/sksl/ir/SkSLProgram.h:87-94
    pub fn advance(&mut self) -> &mut Self {
        if !self.fShared.as_slice().is_empty() {
            self.fShared.next();
        } else {
            assert!(
                self.fOwned.next().is_some(),
                "native iterator increment at end is undefined"
            );
        }
        self
    }
}
// Source: src/sksl/ir/SkSLProgram.h:96-102
impl PartialEq for ElementsIterator<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.fOwned.as_slice().as_ptr() == other.fOwned.as_slice().as_ptr()
            && self.fShared.as_slice().as_ptr() == other.fShared.as_slice().as_ptr()
    }
}
impl Eq for ElementsIterator<'_> {}
impl<'a> ElementsIterator<'a> {
    // Source: src/sksl/ir/SkSLProgram.h:109-110
    fn new(
        owned: &'a [Option<IRNodeBox<dyn ProgramElement>>],
        shared: &'a [Option<NonNull<dyn ProgramElement>>],
    ) -> Self {
        Self {
            fOwned: owned.iter(),
            fShared: shared.iter(),
        }
    }
}
// Uncredited Rust iteration protocol delegates the original dereference/increment operations.
impl<'a> Iterator for ElementsIterator<'a> {
    type Item = Option<&'a dyn ProgramElement>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.fShared.as_slice().is_empty() && self.fOwned.as_slice().is_empty() {
            return None;
        }
        let value = self.value();
        self.advance();
        Some(value)
    }
}
// Source: src/sksl/ir/SkSLProgram.h:132-132
// Source: src/sksl/ir/SkSLProgram.h:75-75
// Existing borrowed collection retains actual Program shared/owned element leases.
pub struct ElementsCollection<'a> {
    fProgram: &'a Program,
}
impl<'a> ElementsCollection<'a> {
    // Source: src/sksl/ir/SkSLProgram.h:118-121
    pub fn begin(&self) -> ElementsIterator<'a> {
        ElementsIterator::new(
            &self.fProgram.fOwnedElements,
            &self.fProgram.fSharedElements,
        )
    }
    // Source: src/sksl/ir/SkSLProgram.h:123-126
    pub fn end(&self) -> ElementsIterator<'a> {
        ElementsIterator::new(
            &self.fProgram.fOwnedElements[self.fProgram.fOwnedElements.len()..],
            &self.fProgram.fSharedElements[self.fProgram.fSharedElements.len()..],
        )
    }
    // Source: src/sksl/ir/SkSLProgram.h:131-131
    fn new(program: &'a Program) -> Self {
        Self { fProgram: program }
    }
}
impl<'a> IntoIterator for ElementsCollection<'a> {
    type Item = Option<&'a dyn ProgramElement>;
    type IntoIter = ElementsIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.begin()
    }
}
// Source: src/sksl/ir/SkSLProgram.h:140-140
impl Program {
    pub fn elements(&self) -> ElementsCollection<'_> {
        ElementsCollection::new(self)
    }
}
// Source: src/sksl/ir/SkSLProgram.h:150-150
impl Program {
    pub fn usage(&self) -> Option<&ProgramUsage> {
        self.fUsage.as_deref()
    }
}
// Source: src/sksl/ir/SkSLProgram.cpp:37-46
impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            {
                let _attach = AutoAttachPoolToThread::new(self.fPool.as_deref());
                // libc++ vector::clear destroys from the back; preserve that order.
                while let Some(element) = self.fOwnedElements.pop() {
                    drop(element);
                }
                drop(self.fContext.take());
                drop(self.fSymbols.take());
            }
            // The attachment guard has detached before implicit native member destruction.
            ManuallyDrop::drop(&mut self.fSharedElements);
            ManuallyDrop::drop(&mut self.fOwnedElements);
            ManuallyDrop::drop(&mut self.fPool);
            ManuallyDrop::drop(&mut self.fSymbols);
            ManuallyDrop::drop(&mut self.fUsage);
            ManuallyDrop::drop(&mut self.fContext);
            ManuallyDrop::drop(&mut self.fConfig);
            ManuallyDrop::drop(&mut self.fSource);
        }
    }
}
// Source: src/sksl/ir/SkSLProgram.cpp:48-54
impl Program {
    pub fn description(&self) -> Vec<u8> {
        let mut result = self
            .fConfig
            .as_ref()
            .expect("native Program config must exist")
            .versionDescription()
            .as_bytes()
            .to_vec();
        for element in self.elements() {
            result.extend(
                element
                    .expect("native Program element must exist")
                    .description(),
            );
        }
        result
    }
}
// Source: src/sksl/ir/SkSLProgram.cpp:56-61
impl Program {
    /// Safety: symbol kind tags describe the actual concrete symbol class, as in native IR.
    pub unsafe fn getFunction(
        &self,
        functionName: &core::ffi::CStr,
    ) -> Option<&super::SkSLFunctionDeclaration::FunctionDeclaration> {
        let symbol = self
            .fSymbols
            .as_ref()
            .expect("native Program symbol table must exist")
            .find(functionName.to_bytes());
        if let Some(symbol) = symbol {
            let symbol = symbol.as_ref();
            if symbol.kind() == super::SkSLIRNode::SymbolKind::kFunctionDeclaration {
                let function = &*(symbol.symbol_data() as *const _
                    as *const super::SkSLFunctionDeclaration::FunctionDeclaration);
                if function.definition().is_some() {
                    return Some(function);
                }
            }
        }
        None
    }
}

impl Program {
    // Source: src/sksl/ir/SkSLProgram.h:66-71
    // Source: src/sksl/ir/SkSLProgram.cpp:22-35
    /// Safety: the context's borrowed compiler/type/symbol leases and every IR
    /// node's allocation-pool lease remain valid through this program's lifetime.
    pub unsafe fn new(
        source: Option<Box<Vec<u8>>>,
        config: Option<Box<ProgramConfig>>,
        context: Arc<Context>,
        elements: Vec<Option<IRNodeBox<dyn ProgramElement>>>,
        symbols: Option<Box<SymbolTable>>,
        pool: Option<Box<Pool>>,
    ) -> Self {
        let mut program = Self {
            fSource: ManuallyDrop::new(source),
            fConfig: ManuallyDrop::new(config),
            fContext: ManuallyDrop::new(Some(context)),
            fUsage: ManuallyDrop::new(None),
            fSymbols: ManuallyDrop::new(symbols),
            fPool: ManuallyDrop::new(pool),
            fOwnedElements: ManuallyDrop::new(elements),
            fSharedElements: ManuallyDrop::new(Vec::new()),
            fInterface: ProgramInterface::default(),
        };
        *program.fUsage = Some(crate::src::sksl::analysis::SkSLProgramUsage::GetUsage(
            &program,
        ));
        program
    }
}
