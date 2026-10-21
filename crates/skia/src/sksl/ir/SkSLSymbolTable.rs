use super::{SkSLIRNode::IRNodeBox, SkSLSymbol::Symbol, SkSLType::TypeData};
use crate::src::core::{
    SkChecksum::Hash32,
    SkTHash::{NativeHash, THashMap},
};
use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
use core::{mem::ManuallyDrop, ptr::NonNull};
use std::collections::LinkedList;
// Source: src/sksl/ir/SkSLSymbolTable.h:35-35
// Source: src/sksl/ir/SkSLSymbolTable.h:201-203
// Source: src/sksl/ir/SkSLSymbolTable.h:224-227
// Actual field order retained. Parent and map pointers borrow actual objects.
// Owning fields are explicitly destroyed in native reverse member order.
#[repr(C)]
pub struct SymbolTable {
    fParent: Option<NonNull<SymbolTable>>,
    fOwnedSymbols: ManuallyDrop<Vec<Option<IRNodeBox<dyn Symbol>>>>,
    fBuiltin: bool,
    fAtModuleBoundary: bool,
    fOwnedStrings: ManuallyDrop<LinkedList<Vec<u8>>>,
    fSymbols: ManuallyDrop<THashMap<SymbolKey, Option<NonNull<dyn Symbol>>, Hash>>,
}
impl SymbolTable {
    // Source: src/sksl/ir/SkSLSymbolTable.h:37-38
    pub fn new(builtin: bool) -> Self {
        Self {
            fParent: None,
            fOwnedSymbols: ManuallyDrop::new(Vec::new()),
            fBuiltin: builtin,
            fAtModuleBoundary: false,
            fOwnedStrings: ManuallyDrop::new(LinkedList::new()),
            fSymbols: ManuallyDrop::new(THashMap::default()),
        }
    }
    // Source: src/sksl/ir/SkSLSymbolTable.h:40-42
    /// Safety: parent remains live at its exact address throughout every access
    /// through this table or any descendant; original native parent-chain lease.
    pub unsafe fn with_parent(parent: Option<NonNull<SymbolTable>>, builtin: bool) -> Self {
        let mut value = Self::new(builtin);
        value.fParent = parent;
        value
    }
    // Source: src/sksl/ir/SkSLSymbolTable.h:54-56
    pub fn find(&self, name: &[u8]) -> Option<NonNull<dyn Symbol>> {
        self.lookup(&Self::MakeSymbolKey(name))
    }
    // Source: src/sksl/ir/SkSLSymbolTable.h:67-69
    pub fn findMutable(&self, name: &[u8]) -> Option<NonNull<dyn Symbol>> {
        self.lookup(&Self::MakeSymbolKey(name))
    }
    // Source: src/sksl/ir/SkSLSymbolTable.h:120-125
    pub unsafe fn add(
        &mut self,
        context: &crate::src::sksl::SkSLContext::Context,
        symbol: IRNodeBox<dyn Symbol>,
    ) -> NonNull<dyn Symbol> {
        let p = self.takeOwnershipOfSymbol(Some(symbol)).unwrap_unchecked();
        self.addWithoutOwnershipContext(context, p);
        p
    }
    // Source: src/sksl/ir/SkSLSymbolTable.h:131-136
    pub unsafe fn addOrDie(&mut self, symbol: IRNodeBox<dyn Symbol>) -> NonNull<dyn Symbol> {
        let p = self.takeOwnershipOfSymbol(Some(symbol)).unwrap_unchecked();
        self.addWithoutOwnershipOrDie(p);
        p
    }
    // Source: src/sksl/ir/SkSLSymbolTable.h:148-153
    /// Safety: symbol and name backing obey original lifetime/immutability leases;
    /// the same original pool must be attached when owned symbols are destroyed.
    pub unsafe fn inject(&mut self, symbol: IRNodeBox<dyn Symbol>) -> NonNull<dyn Symbol> {
        let ptr = NonNull::new_unchecked(symbol.as_ptr() as *mut dyn Symbol);
        let adopted = self.takeOwnershipOfSymbol(Some(symbol)).unwrap_unchecked();
        self.injectWithoutOwnership(adopted);
        ptr
    }
    // Source: src/sksl/ir/SkSLSymbolTable.h:158-163
    /// Safety: the borrowed names/types remain live for all future symbol accesses;
    /// concrete destructors and current-pool lease remain valid through table Drop.
    pub unsafe fn takeOwnershipOfSymbol(
        &mut self,
        symbol: Option<IRNodeBox<dyn Symbol>>,
    ) -> Option<NonNull<dyn Symbol>> {
        let ptr = symbol
            .as_ref()
            .map(|s| NonNull::new_unchecked(s.as_ptr() as *mut dyn Symbol));
        self.fOwnedSymbols.push(symbol);
        ptr
    }
    // Source: src/sksl/ir/SkSLSymbolTable.h:173-177
    pub fn foreach(&self, mut f: impl FnMut(&[u8], NonNull<dyn Symbol>)) {
        self.fSymbols.foreach(|key, symbol| {
            f(unsafe { key.fName.as_ref() }, unsafe {
                symbol.unwrap_unchecked()
            });
        });
    }
    // Source: src/sksl/ir/SkSLSymbolTable.h:183-185
    pub fn count(&self) -> usize {
        self.fSymbols.count() as usize
    }
    // Source: src/sksl/ir/SkSLSymbolTable.h:188-190
    pub fn isBuiltin(&self) -> bool {
        self.fBuiltin
    }
    // Source: src/sksl/ir/SkSLSymbolTable.h:197-199
    pub fn markModuleBoundary(&mut self) {
        self.fAtModuleBoundary = true;
    }
}
// Source: src/sksl/ir/SkSLSymbolTable.h:206-208
#[derive(Clone, Copy)]
struct SymbolKey {
    fName: NonNull<[u8]>,
    fHash: u32,
}
// Source: src/sksl/ir/SkSLSymbolTable.h:210-211
impl PartialEq for SymbolKey {
    fn eq(&self, that: &Self) -> bool {
        unsafe { self.fName.as_ref() == that.fName.as_ref() }
    }
}
// Source: src/sksl/ir/SkSLSymbolTable.h:212-214
#[derive(Default)]
struct Hash;
impl NativeHash<SymbolKey> for Hash {
    fn hash(&self, key: &SymbolKey) -> u32 {
        key.fHash
    }
}
impl SymbolTable {
    // Source: src/sksl/ir/SkSLSymbolTable.h:217-219
    fn MakeSymbolKey(name: &[u8]) -> SymbolKey {
        SymbolKey {
            fName: NonNull::from(name),
            fHash: Hash32(name, 0),
        }
    }
}
// Language-only destruction adapter for original implicit C++ destructor.
// THashMap dies first, then stable string nodes, then vector's unique owners.
// Selected native libc++ vector destroys elements from back to front; preserve
// that order explicitly instead of Rust Vec's forward element destruction.
impl Drop for SymbolTable {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fSymbols);
            while let Some(s) = self.fOwnedStrings.pop_front() {
                drop(s);
            }
            ManuallyDrop::drop(&mut self.fOwnedStrings);
            while let Some(s) = self.fOwnedSymbols.pop() {
                drop(s);
            }
            ManuallyDrop::drop(&mut self.fOwnedSymbols);
        }
    }
}
impl SymbolTable {
    // Source: src/sksl/ir/SkSLSymbolTable.cpp:20-24
    /// Safety: returned owner must outlive every subsequent access to this table's
    /// new parent; parent chain stays acyclic and all ancestor addresses stay live.
    pub unsafe fn insertNewParent(&mut self) -> Box<SymbolTable> {
        let mut newTable = Box::new(Self::with_parent(self.fParent, self.fBuiltin));
        self.fParent = Some(NonNull::from(&mut *newTable));
        newTable
    }
    // Source: src/sksl/ir/SkSLSymbolTable.cpp:26-29
    pub fn isType(&self, name: &[u8]) -> bool {
        let symbol = self.find(name);
        symbol.map_or(false, |p| unsafe { p.as_ref() }.node().is::<TypeData>())
    }
    // Source: src/sksl/ir/SkSLSymbolTable.cpp:31-36
    pub fn isBuiltinType(&self, name: &[u8]) -> bool {
        if !self.isBuiltin() {
            return self
                .fParent
                .is_some_and(|p| unsafe { p.as_ref() }.isBuiltinType(name));
        }
        self.isType(name)
    }
    // Source: src/sksl/ir/SkSLSymbolTable.cpp:38-43
    pub fn findBuiltinSymbol(&self, name: &[u8]) -> Option<NonNull<dyn Symbol>> {
        if !self.isBuiltin() {
            return self
                .fParent
                .and_then(|p| unsafe { p.as_ref() }.findBuiltinSymbol(name));
        }
        self.find(name)
    }
    // Source: src/sksl/ir/SkSLSymbolTable.cpp:45-66
    pub fn wouldShadowSymbolsFrom(&self, other: &SymbolTable) -> bool {
        let (mut this, mut other) = (self, other);
        if this.count() > other.count() {
            core::mem::swap(&mut this, &mut other);
        }
        let mut foundShadow = false;
        this.fSymbols.foreach(|key, _symbol| {
            if foundShadow {
                return;
            }
            if other.fSymbols.find(key).is_some() {
                foundShadow = true;
            }
        });
        foundShadow
    }
    // Source: src/sksl/ir/SkSLSymbolTable.cpp:68-76
    fn lookup(&self, key: &SymbolKey) -> Option<NonNull<dyn Symbol>> {
        let symbolPPtr = self.fSymbols.find(key);
        if let Some(symbol) = symbolPPtr {
            return *symbol;
        }
        self.fParent.and_then(|p| unsafe { p.as_ref() }.lookup(key))
    }
    // Source: src/sksl/ir/SkSLSymbolTable.cpp:78-91
    pub unsafe fn renameSymbol(
        &mut self,
        context: &crate::src::sksl::SkSLContext::Context,
        mut symbol: NonNull<dyn Symbol>,
        newName: &[u8],
    ) {
        use super::SkSLFunctionDeclaration::FunctionDeclaration;
        if symbol.as_ref().node().is::<FunctionDeclaration>() {
            let mut next = Some(NonNull::new_unchecked(
                symbol.as_ptr() as *mut () as *mut FunctionDeclaration
            ));
            while let Some(mut ptr) = next {
                ptr.as_mut().base.set_name_bytes(newName);
                next = ptr.as_ref().fNextOverload;
            }
        } else {
            core::pin::Pin::new_unchecked(symbol.as_mut()).setName(newName);
        }
        self.addWithoutOwnershipContext(context, symbol);
    }
    // Source: src/sksl/ir/SkSLSymbolTable.cpp:93-107
    /// Safety: symbol is a live actual object and its name backing is valid during
    /// removal; returned unique owner retains its original destructor/pool lease.
    pub unsafe fn removeSymbol(
        &mut self,
        symbol: NonNull<dyn Symbol>,
    ) -> Option<IRNodeBox<dyn Symbol>> {
        if self
            .fSymbols
            .removeIfExists(&Self::MakeSymbolKey(symbol.as_ref().name()))
        {
            for owned in self.fOwnedSymbols.iter_mut() {
                if owned
                    .as_ref()
                    .is_some_and(|s| core::ptr::addr_eq(s.as_ptr(), symbol.as_ptr()))
                {
                    return owned.take();
                }
            }
        }
        None
    }
    // Source: src/sksl/ir/SkSLSymbolTable.cpp:109-115
    pub unsafe fn moveSymbolTo(
        &mut self,
        otherTable: &mut SymbolTable,
        sym: NonNull<dyn Symbol>,
        context: &crate::src::sksl::SkSLContext::Context,
    ) {
        if let Some(owner) = self.removeSymbol(sym) {
            otherTable.add(context, owner);
        } else {
            otherTable.addWithoutOwnershipContext(context, sym);
        }
    }
    // Source: src/sksl/ir/SkSLSymbolTable.cpp:117-121
    pub fn takeOwnershipOfString(&mut self, str: Vec<u8>) -> NonNull<Vec<u8>> {
        self.fOwnedStrings.push_front(str);
        NonNull::from(self.fOwnedStrings.front_mut().unwrap())
    }
    // Source: src/sksl/ir/SkSLSymbolTable.cpp:123-128
    pub unsafe fn addWithoutOwnershipContext(
        &mut self,
        context: &crate::src::sksl::SkSLContext::Context,
        symbol: NonNull<dyn Symbol>,
    ) {
        if !self.addWithoutOwnership(symbol) {
            let mut message = b"symbol '".to_vec();
            message.extend_from_slice(symbol.as_ref().name());
            message.extend_from_slice(b"' was already defined");
            let mut errors = context.fErrors;
            errors
                .as_mut()
                .error(symbol.as_ref().node().fPosition, &message);
        }
    }
    // Source: src/sksl/ir/SkSLSymbolTable.cpp:130-135
    pub unsafe fn addWithoutOwnershipOrDie(&mut self, symbol: NonNull<dyn Symbol>) {
        if !self.addWithoutOwnership(symbol) {
            std::process::abort();
        }
    }
    // Source: src/sksl/ir/SkSLSymbolTable.cpp:137-166
    pub unsafe fn addWithoutOwnership(&mut self, symbol: NonNull<dyn Symbol>) -> bool {
        use super::SkSLFunctionDeclaration::FunctionDeclaration;
        if symbol.as_ref().name().is_empty() {
            return true;
        }
        let key = Self::MakeSymbolKey(symbol.as_ref().name());
        if symbol.as_ref().node().is::<FunctionDeclaration>() {
            if let Some(existing) = self.lookup(&key) {
                if existing.as_ref().node().is::<FunctionDeclaration>() {
                    let mut declaration = NonNull::new_unchecked(
                        symbol.as_ptr() as *mut () as *mut FunctionDeclaration
                    );
                    let old = NonNull::new_unchecked(
                        existing.as_ptr() as *mut () as *mut FunctionDeclaration
                    );
                    declaration.as_mut().setNextOverload(Some(old));
                    *self.fSymbols.index_or_default(&key) = Some(symbol);
                    return true;
                }
            }
        }
        if self.fAtModuleBoundary
            && self
                .fParent
                .is_some_and(|p| p.as_ref().lookup(&key).is_some())
        {
            return false;
        }
        self.fSymbols
            .index_or_default(&key)
            .replace(symbol)
            .is_none()
    }
    // Source: src/sksl/ir/SkSLSymbolTable.cpp:168-171
    /// Safety: symbol and its original name backing outlive every map lookup until
    /// removal/table destruction. Native operator[] preserves the existing key view.
    pub unsafe fn injectWithoutOwnership(&mut self, symbol: NonNull<dyn Symbol>) {
        let key = Self::MakeSymbolKey(symbol.as_ref().name());
        *self.fSymbols.index_or_default(&key) = Some(symbol);
    }
}
// Read-only native public-field projections; writes remain behind unsafe native
// lifetime contracts instead of exposing a safe dangling-parent constructor.
impl SymbolTable {
    pub fn parent(&self) -> Option<NonNull<SymbolTable>> {
        self.fParent
    }
    pub fn owned_symbols(&self) -> &[Option<IRNodeBox<dyn Symbol>>] {
        &self.fOwnedSymbols
    }
}

impl SymbolTable {
    // Source: src/sksl/ir/SkSLSymbolTable.cpp:173-198
    pub unsafe fn addArrayDimension(
        &mut self,
        context: &crate::src::sksl::SkSLContext::Context,
        ty: NonNull<dyn super::SkSLType::Type>,
        arraySize: i32,
    ) -> NonNull<dyn super::SkSLType::Type> {
        if arraySize == 0 {
            return ty;
        }
        if self.fParent.is_some()
            && !self.fAtModuleBoundary
            && !context.fConfig.unwrap_unchecked().as_ref().isBuiltinCode()
            && ty.as_ref().isBuiltin()
        {
            return self
                .fParent
                .unwrap_unchecked()
                .as_mut()
                .addArrayDimension(context, ty, arraySize);
        }
        let arrayName = ty.as_ref().getArrayName(arraySize);
        if let Some(existing) = self.find(&arrayName) {
            debug_assert!(existing.as_ref().node().is::<TypeData>());
            let existingType = existing.as_ref().as_type_symbol();
            if existingType.isArray() && ty.as_ref().matches(existingType.componentType()) {
                return NonNull::from(existingType);
            }
        }
        let name = self.takeOwnershipOfString(arrayName);
        let owner = <dyn super::SkSLType::Type>::MakeArrayType(
            context,
            name.as_ref(),
            ty.as_ref(),
            arraySize,
        );
        let result = NonNull::new_unchecked(owner.as_ptr() as *mut dyn super::SkSLType::Type);
        let owner = owner.cast(|p| p as *mut dyn Symbol);
        self.add(context, owner);
        result
    }
}

// UNVERIFIED by explicit instruction.
impl SymbolTable {
    // Source: src/sksl/ir/SkSLSymbolTable.cpp:200-208
    pub unsafe fn instantiateSymbolRef(
        &self,
        context: &crate::src::sksl::SkSLContext::Context,
        name: &[u8],
        pos: crate::src::sksl::SkSLPosition::Position,
    ) -> Option<super::SkSLIRNode::IRNodeBox<dyn super::SkSLExpression::Expression>> {
        use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
        if let Some(symbol) = self.find(name) {
            return super::SkSLSymbol::instantiate(symbol.as_ref(), context, pos);
        }
        let mut msg = b"unknown identifier '".to_vec();
        msg.extend_from_slice(name);
        msg.push(b'\'');
        let mut errors = context.fErrors;
        errors.as_mut().error(pos, &msg);
        None
    }
}
