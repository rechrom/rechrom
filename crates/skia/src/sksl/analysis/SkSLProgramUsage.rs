use crate::src::{
    core::{
        SkChecksum::Hash32,
        SkTHash::{NativeHash, THashMap},
    },
    sksl::ir::{SkSLSymbol::SymbolData, SkSLVariable::VariableData},
};
use core::{mem::ManuallyDrop, ptr::NonNull};
// Source: src/sksl/analysis/SkSLProgramUsage.h:28-32
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct VariableCounts {
    pub fVarExists: i32,
    pub fRead: i32,
    pub fWrite: i32,
}
// Native pointer hash: only thin object identity bytes, never Rust trait metadata.
#[derive(Default)]
pub struct ObjectHash;
impl<T> NativeHash<NonNull<T>> for ObjectHash {
    fn hash(&self, p: &NonNull<T>) -> u32 {
        if size_of::<usize>() == 4 {
            crate::src::core::SkChecksum::Mix(p.as_ptr() as usize as u32)
        } else {
            Hash32(&(p.as_ptr() as usize).to_ne_bytes(), 0)
        }
    }
}
// Native const Variable*/Type* keys must retain the actual polymorphic object.
// Erasing these to a base-data pointer discarded the Rust vtable needed by
// layout/isBuiltin consumers. This is pointer metadata, not a second owner.
// Hashing and equality retain native data-address identity, ignoring vtables.
pub struct ObjectKey<T: ?Sized> {
    pointer: NonNull<T>,
}
impl<T: ?Sized> Copy for ObjectKey<T> {}
impl<T: ?Sized> Clone for ObjectKey<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: ?Sized> PartialEq for ObjectKey<T> {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::addr_eq(self.pointer.as_ptr(), other.pointer.as_ptr())
    }
}
impl<T: ?Sized> Eq for ObjectKey<T> {}
impl<T: ?Sized> ObjectKey<T> {
    /// Safety: the original native symbol/type owner still lives; no mutable
    /// reference to this same object may overlap the returned shared borrow.
    pub unsafe fn as_ref(&self) -> &T {
        self.pointer.as_ref()
    }
    pub fn as_ptr(&self) -> *mut T {
        self.pointer.as_ptr()
    }
}
pub type VariableKey = ObjectKey<dyn crate::src::sksl::ir::SkSLVariable::Variable>;
pub type TypeKey = ObjectKey<dyn crate::src::sksl::ir::SkSLType::Type>;
impl VariableKey {
    /// Safety: usage records only borrow the actual IR symbol; its native
    /// lifetime must cover every lookup/dereference of this key.
    pub unsafe fn from_variable(v: &dyn crate::src::sksl::ir::SkSLVariable::Variable) -> Self {
        Self {
            pointer: core::mem::transmute::<
                NonNull<dyn crate::src::sksl::ir::SkSLVariable::Variable + '_>,
                NonNull<dyn crate::src::sksl::ir::SkSLVariable::Variable>,
            >(NonNull::from(v)),
        }
    }
}
impl TypeKey {
    /// Safety: the original type owner outlives every use of this borrowed key.
    pub unsafe fn from_type(t: &dyn crate::src::sksl::ir::SkSLType::Type) -> Self {
        Self {
            pointer: core::mem::transmute::<
                NonNull<dyn crate::src::sksl::ir::SkSLType::Type + '_>,
                NonNull<dyn crate::src::sksl::ir::SkSLType::Type>,
            >(NonNull::from(t)),
        }
    }
}
impl<T: ?Sized> NativeHash<ObjectKey<T>> for ObjectHash {
    fn hash(&self, key: &ObjectKey<T>) -> u32 {
        let address = key.as_ptr() as *mut () as usize;
        if size_of::<usize>() == 4 {
            crate::src::core::SkChecksum::Mix(address as u32)
        } else {
            Hash32(&address.to_ne_bytes(), 0)
        }
    }
}
// Source: src/sksl/analysis/SkSLProgramUsage.h:48-52
#[repr(C)]
pub struct ProgramUsage {
    pub fStructCounts: ManuallyDrop<THashMap<TypeKey, i32, ObjectHash>>,
    pub fCallCounts: ManuallyDrop<THashMap<NonNull<SymbolData>, i32, ObjectHash>>,
    pub fVariableCounts: ManuallyDrop<THashMap<VariableKey, VariableCounts, ObjectHash>>,
}
// Actual native default member construction. This is not a replacement for the
// mandatory Analysis::GetUsage pass in Program's source constructor.
impl Default for ProgramUsage {
    fn default() -> Self {
        Self {
            fStructCounts: ManuallyDrop::new(THashMap::default()),
            fCallCounts: ManuallyDrop::new(THashMap::default()),
            fVariableCounts: ManuallyDrop::new(THashMap::default()),
        }
    }
}
impl Drop for ProgramUsage {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fVariableCounts);
            ManuallyDrop::drop(&mut self.fCallCounts);
            ManuallyDrop::drop(&mut self.fStructCounts);
        }
    }
}
// UNVERIFIED frozen-scope actual usage visitor, expression/statement channels.
use crate::src::sksl::ir::{
    SkSLExpression::{Expression, ExpressionExt},
    SkSLFunctionCall::FunctionCall,
    SkSLStatement::Statement,
    SkSLType::Type,
    SkSLVarDeclarations::VarDeclaration,
    SkSLVariableReference::{VariableRefKind, VariableReference},
};
// Source: src/sksl/analysis/SkSLProgramUsage.cpp:45-45
// Source: src/sksl/analysis/SkSLProgramUsage.cpp:137-139
struct ProgramUsageVisitor {
    fUsage: NonNull<ProgramUsage>,
    fDelta: i32,
}
impl ProgramUsageVisitor {
    // Source: src/sksl/analysis/SkSLProgramUsage.cpp:45-45
    fn new(usage: &mut ProgramUsage, delta: i32) -> Self {
        Self {
            fUsage: NonNull::from(usage),
            fDelta: delta,
        }
    }
    // Source: src/sksl/analysis/SkSLProgramUsage.cpp:71-86
    unsafe fn visitStatement(&mut self, s: &dyn Statement) -> bool {
        if s.node().is::<VarDeclaration>() {
            let vd = s.node().as_node::<VarDeclaration>();
            let var = vd.var().unwrap_unchecked();
            let var = var.as_ref();
            let key = VariableKey::from_variable(var);
            {
                let counts = self.fUsage.as_mut().fVariableCounts.index_or_default(&key);
                counts.fVarExists += self.fDelta;
                crate::SkASSERT!(@source b"src/sksl/analysis/SkSLProgramUsage.cpp\0".as_ptr(),78; @condition b"counts.fVarExists >= 0 && counts.fVarExists <= 1\0".as_ptr(); counts.fVarExists>=0&&counts.fVarExists<=1);
                if vd.value().is_some() {
                    counts.fWrite += self.fDelta;
                }
            }
            self.visitType(var.symbol_type());
        }
        let this = self as *mut Self;
        crate::src::sksl::analysis::SkSLProgramVisitor::visitStatement(
            &mut |stmt| (*this).visitStatement(stmt),
            &mut |expr| (*this).visitExpression(expr),
            s,
        )
    }
    // Source: src/sksl/analysis/SkSLProgramUsage.cpp:88-113
    unsafe fn visitExpression(&mut self, e: &dyn Expression) -> bool {
        self.visitType(e.expression_type());
        if e.node().is::<FunctionCall>() {
            let f = e.node().as_node::<FunctionCall>().function();
            let counts = self
                .fUsage
                .as_mut()
                .fCallCounts
                .index_or_default(&NonNull::from(&f.base));
            *counts += self.fDelta;
            debug_assert!(*counts >= 0);
        } else if e.node().is::<VariableReference>() {
            let reference = e.node().as_node::<VariableReference>();
            let var = reference.variable().unwrap_unchecked();
            let key = VariableKey::from_variable(var.as_ref());
            let counts = self.fUsage.as_mut().fVariableCounts.index_or_default(&key);
            match reference.refKind() {
                VariableRefKind::kRead => counts.fRead += self.fDelta,
                VariableRefKind::kWrite => counts.fWrite += self.fDelta,
                VariableRefKind::kReadWrite | VariableRefKind::kPointer => {
                    counts.fRead += self.fDelta;
                    counts.fWrite += self.fDelta;
                }
            }
            crate::SkASSERT!(@source b"src/sksl/analysis/SkSLProgramUsage.cpp\0".as_ptr(),110; @condition b"counts.fRead >= 0 && counts.fWrite >= 0\0".as_ptr(); counts.fRead>=0&&counts.fWrite>=0);
        }
        crate::src::sksl::analysis::SkSLProgramVisitor::visitExpression(
            &mut |expr| self.visitExpression(expr),
            e,
        )
    }
    // Source: src/sksl/analysis/SkSLProgramUsage.cpp:115-127
    unsafe fn visitType(&mut self, t: &dyn Type) {
        if t.isArray() {
            self.visitType(t.componentType());
            return;
        }
        if t.isStruct() {
            let counts = self
                .fUsage
                .as_mut()
                .fStructCounts
                .index_or_default(&TypeKey::from_type(t));
            *counts += self.fDelta;
            debug_assert!(*counts >= 0);
            self.visitStructFields(t);
        }
    }
    // Source: src/sksl/analysis/SkSLProgramUsage.cpp:129-133
    unsafe fn visitStructFields(&mut self, t: &dyn Type) {
        for field in t.fields() {
            self.visitType(field.fType.unwrap_unchecked().as_ref());
        }
    }
}
impl ProgramUsage {
    // Source: src/sksl/analysis/SkSLProgramUsage.cpp:164-168
    pub fn getVariable(
        &self,
        v: &dyn crate::src::sksl::ir::SkSLVariable::Variable,
    ) -> VariableCounts {
        let key = unsafe { VariableKey::from_variable(v) };
        let counts = self.fVariableCounts.find(&key);
        crate::SkASSERT!(@source b"src/sksl/analysis/SkSLProgramUsage.cpp\0".as_ptr(),166; @condition b"counts\0".as_ptr(); counts.is_some());
        unsafe { *counts.unwrap_unchecked() }
    }
    // Source: src/sksl/analysis/SkSLProgramUsage.cpp:170-183
    pub unsafe fn isDead(&self, v: &dyn crate::src::sksl::ir::SkSLVariable::Variable) -> bool {
        use crate::src::sksl::ir::SkSLModifierFlags::ModifierFlag as F;
        let flags = v.modifierFlags();
        let counts = self.getVariable(v);
        if flags.0 & (F::kIn.0 | F::kOut.0 | F::kUniform.0) != 0 {
            return false;
        }
        if v.symbol_type().componentType().isOpaque() {
            return false;
        }
        counts.fRead == 0 && counts.fWrite <= if v.initialValue().is_some() { 1 } else { 0 }
    }
    // Source: src/sksl/analysis/SkSLProgramUsage.cpp:185-188
    pub fn getFunction(
        &self,
        f: &crate::src::sksl::ir::SkSLFunctionDeclaration::FunctionDeclaration,
    ) -> i32 {
        self.fCallCounts
            .find(&NonNull::from(&f.base))
            .copied()
            .unwrap_or(0)
    }
}

impl ProgramUsage {
    // Source: src/sksl/analysis/SkSLProgramUsage.cpp:190-193
    pub unsafe fn addExpression(&mut self, value: &dyn Expression) {
        ProgramUsageVisitor::new(self, 1).visitExpression(value);
    }
    // Source: src/sksl/analysis/SkSLProgramUsage.cpp:195-198
    pub unsafe fn addStatement(&mut self, value: &dyn Statement) {
        ProgramUsageVisitor::new(self, 1).visitStatement(value);
    }
    // Source: src/sksl/analysis/SkSLProgramUsage.cpp:205-208
    pub unsafe fn removeExpression(&mut self, value: &dyn Expression) {
        ProgramUsageVisitor::new(self, -1).visitExpression(value);
    }
    // Source: src/sksl/analysis/SkSLProgramUsage.cpp:210-213
    pub unsafe fn removeStatement(&mut self, value: &dyn Statement) {
        ProgramUsageVisitor::new(self, -1).visitStatement(value);
    }
}
// Source: src/sksl/analysis/SkSLProgramUsage.cpp:220-283
// Adapted THash foreach traversal uses a guarded result to retain native first-mismatch
// early-return semantics; disabled kReportMismatch diagnostics remain unexecuted.
fn contains_matching_data(a: &ProgramUsage, b: &ProgramUsage) -> bool {
    let mut matches = true;
    a.fVariableCounts.foreach(|key, count| {
        if !matches {
            return;
        }
        if count.fVarExists == 0 && count.fRead == 0 && count.fWrite == 0 {
            return;
        }
        if b.fVariableCounts.find(key) != Some(count) {
            matches = false;
        }
    });
    if !matches {
        return false;
    }
    a.fCallCounts.foreach(|key, count| {
        if !matches || *count == 0 {
            return;
        }
        if b.fCallCounts.find(key) != Some(count) {
            matches = false;
        }
    });
    if !matches {
        return false;
    }
    a.fStructCounts.foreach(|key, count| {
        if !matches || *count == 0 {
            return;
        }
        if b.fStructCounts.find(key) != Some(count) {
            matches = false;
        }
    });
    if !matches {
        return false;
    }
    true
}
// Source: src/sksl/analysis/SkSLProgramUsage.cpp:285-298
impl PartialEq for ProgramUsage {
    fn eq(&self, that: &Self) -> bool {
        contains_matching_data(self, that) && contains_matching_data(that, self)
    }
    // Source: src/sksl/analysis/SkSLProgramUsage.h:46-46
    fn ne(&self, that: &Self) -> bool {
        !self.eq(that)
    }
}

impl ProgramUsageVisitor {
    // Source: src/sksl/analysis/SkSLProgramUsage.cpp:47-69
    unsafe fn visitProgramElement(
        &mut self,
        pe: &dyn crate::src::sksl::ir::SkSLProgramElement::ProgramElement,
    ) -> bool {
        use crate::src::sksl::ir::{
            SkSLFunctionDefinition::FunctionDefinition, SkSLInterfaceBlock::InterfaceBlock,
            SkSLStructDefinition::StructDefinition,
        };
        if pe.node().is::<FunctionDefinition>() {
            for param in pe
                .node()
                .as_node::<FunctionDefinition>()
                .declaration()
                .parameters()
            {
                let param = param.unwrap_unchecked().as_ref();
                self.fUsage
                    .as_mut()
                    .fVariableCounts
                    .index_or_default(&VariableKey::from_variable(param))
                    .fVarExists += self.fDelta;
                self.visitType(param.symbol_type());
            }
        } else if pe.node().is::<InterfaceBlock>() {
            let var = pe
                .node()
                .as_node::<InterfaceBlock>()
                .var()
                .unwrap_unchecked()
                .as_ref();
            self.fUsage
                .as_mut()
                .fVariableCounts
                .index_or_default(&VariableKey::from_variable(var));
            self.visitType(var.symbol_type());
        } else if pe.node().is::<StructDefinition>() {
            self.visitStructFields(pe.node().as_node::<StructDefinition>().r#type());
        }
        crate::src::sksl::analysis::SkSLProgramVisitor::visitProgramElement(
            &mut |stmt| self.visitStatement(stmt),
            pe,
        )
    }
}
// Source: src/sksl/analysis/SkSLProgramUsage.cpp:145-150
pub unsafe fn GetUsage(program: &crate::src::sksl::ir::SkSLProgram::Program) -> Box<ProgramUsage> {
    let mut usage = Box::new(ProgramUsage::default());
    let mut addRefs = ProgramUsageVisitor::new(&mut usage, 1);
    for element in program.elements() {
        addRefs.visitProgramElement(element.unwrap_unchecked());
    }
    usage
}
// Source: src/sksl/analysis/SkSLProgramUsage.cpp:152-162
pub unsafe fn GetUsageForModule(
    module: &crate::src::sksl::SkSLModule::Module,
) -> Box<ProgramUsage> {
    let mut usage = Box::new(ProgramUsage::default());
    let mut addRefs = ProgramUsageVisitor::new(&mut usage, 1);
    let mut current = Some(NonNull::from(module));
    while let Some(m) = current {
        let m = m.as_ref();
        for element in m.fElements.iter() {
            addRefs
                .visitProgramElement(&**element.as_ref().expect("native nonnull module element"));
        }
        current = m.fParent;
    }
    usage
}
impl ProgramUsage {
    // Source: src/sksl/analysis/SkSLProgramUsage.cpp:200-203
    pub unsafe fn addProgramElement(
        &mut self,
        element: &dyn crate::src::sksl::ir::SkSLProgramElement::ProgramElement,
    ) {
        ProgramUsageVisitor::new(self, 1).visitProgramElement(element);
    }
    // Source: src/sksl/analysis/SkSLProgramUsage.cpp:215-218
    pub unsafe fn removeProgramElement(
        &mut self,
        element: &dyn crate::src::sksl::ir::SkSLProgramElement::ProgramElement,
    ) {
        ProgramUsageVisitor::new(self, -1).visitProgramElement(element);
    }
}
