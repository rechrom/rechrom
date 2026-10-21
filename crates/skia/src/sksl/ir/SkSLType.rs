// v47 additions/API alignment are UNVERIFIED; user prohibited all validation commands.
use super::{
    SkSLIRNode::SymbolKind,
    SkSLLayout::Layout,
    SkSLModifierFlags::ModifierFlags,
    SkSLSymbol::{Symbol, SymbolData},
};
use crate::src::sksl::SkSLPosition::Position;
use core::{pin::Pin, ptr::NonNull};
// Source: src/sksl/ir/SkSLType.h:72-74
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoercionCost {
    pub fNormalCost: i32,
    pub fNarrowingCost: i32,
    pub fImpossible: bool,
}
impl CoercionCost {
    // Source: src/sksl/ir/SkSLType.h:45-45
    pub fn Free() -> Self {
        Self {
            fNormalCost: 0,
            fNarrowingCost: 0,
            fImpossible: false,
        }
    }
    // Source: src/sksl/ir/SkSLType.h:46-46
    pub fn Normal(cost: i32) -> Self {
        Self {
            fNormalCost: cost,
            fNarrowingCost: 0,
            fImpossible: false,
        }
    }
    // Source: src/sksl/ir/SkSLType.h:47-47
    pub fn Narrowing(cost: i32) -> Self {
        Self {
            fNormalCost: 0,
            fNarrowingCost: cost,
            fImpossible: false,
        }
    }
    // Source: src/sksl/ir/SkSLType.h:48-48
    pub fn Impossible() -> Self {
        Self {
            fNormalCost: 0,
            fNarrowingCost: 0,
            fImpossible: true,
        }
    }
    // Source: src/sksl/ir/SkSLType.h:50-52
    pub fn isPossible(self, allowNarrowing: bool) -> bool {
        !self.fImpossible && (self.fNarrowingCost == 0 || allowNarrowing)
    }
}
// Source: src/sksl/ir/SkSLType.h:55-60
impl core::ops::Add for CoercionCost {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        if self.fImpossible || rhs.fImpossible {
            return Self::Impossible();
        }
        Self {
            fNormalCost: self.fNormalCost + rhs.fNormalCost,
            fNarrowingCost: self.fNarrowingCost + rhs.fNarrowingCost,
            fImpossible: false,
        }
    }
}
// Source: src/sksl/ir/SkSLType.h:62-70
impl PartialOrd for CoercionCost {
    fn partial_cmp(&self, rhs: &Self) -> Option<core::cmp::Ordering> {
        Some(
            (self.fImpossible, self.fNarrowingCost, self.fNormalCost).cmp(&(
                rhs.fImpossible,
                rhs.fNarrowingCost,
                rhs.fNormalCost,
            )),
        )
    }
}
// Source: src/sksl/ir/SkSLType.h:90-94
// Adapted borrowed string_view and actual polymorphic Type pointer; no ownership.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Field {
    pub fPosition: Position,
    pub fLayout: Layout,
    pub fModifierFlags: ModifierFlags,
    pub fName: NonNull<[u8]>,
    pub fType: Option<NonNull<dyn Type>>,
}
impl Field {
    // Source: src/sksl/ir/SkSLType.h:81-86
    /// Safety: name/optional Type backing remains live for all accesses to Field.
    pub unsafe fn new(
        pos: Position,
        layout: Layout,
        flags: ModifierFlags,
        name: &[u8],
        ty: Option<NonNull<dyn Type>>,
    ) -> Self {
        Self {
            fPosition: pos,
            fLayout: layout,
            fModifierFlags: flags,
            fName: NonNull::from(name),
            fType: ty,
        }
    }
}
// Source: src/sksl/ir/SkSLType.h:102-105
pub const kIRNodeKind: SymbolKind = SymbolKind::kType;
pub const kMaxAbbrevLength: usize = 3;
pub const kUnsizedArray: i32 = -1;
// Source: src/sksl/ir/SkSLType.h:107-126
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeKind(pub i8);
impl TypeKind {
    pub const kArray: Self = Self(0);
    pub const kAtomic: Self = Self(1);
    pub const kGeneric: Self = Self(2);
    pub const kLiteral: Self = Self(3);
    pub const kMatrix: Self = Self(4);
    pub const kOther: Self = Self(5);
    pub const kSampler: Self = Self(6);
    pub const kSeparateSampler: Self = Self(7);
    pub const kScalar: Self = Self(8);
    pub const kStruct: Self = Self(9);
    pub const kTexture: Self = Self(10);
    pub const kVector: Self = Self(11);
    pub const kVoid: Self = Self(12);
    pub const kColorFilter: Self = Self(13);
    pub const kShader: Self = Self(14);
    pub const kBlender: Self = Self(15);
}
// Source: src/sksl/ir/SkSLType.h:128-134
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumberKind(pub i8);
impl NumberKind {
    pub const kFloat: Self = Self(0);
    pub const kSigned: Self = Self(1);
    pub const kUnsigned: Self = Self(2);
    pub const kBoolean: Self = Self(3);
    pub const kNonnumeric: Self = Self(4);
}
// Source: src/sksl/ir/SkSLType.h:136-141
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureAccess(pub i8);
impl TextureAccess {
    pub const kSample: Self = Self(0);
    pub const kRead: Self = Self(1);
    pub const kWrite: Self = Self(2);
    pub const kReadWrite: Self = Self(3);
}
// Source: src/sksl/ir/SkSLType.h:100-100
// Source: src/sksl/ir/SkSLType.h:677-678
// Adapted inheritance field grouping, no extra state. Native abbreviated char
// storage is copied including terminal NUL; original Symbol.fType stays null.
#[repr(C)]
pub struct TypeData {
    pub(crate) base: SymbolData,
    fAbbreviatedName: [u8; 4],
    fTypeKind: TypeKind,
}
// Inherited native Type::kIRNodeKind template tag, no additional object state.
impl super::SkSLIRNode::IRNodeType for TypeData {
    const kIRNodeKind: i32 = SymbolKind::kType as i32;
}
// Native template static kind tag for actual Type subclasses.
pub trait TypeTag {
    const kTypeKind: TypeKind;
}
// Nonvirtual native helpers, expressed as blanket extensions; no override state.
pub trait TypeExt: Type {
    // Source: src/sksl/ir/SkSLType.h:210-213
    fn is<T: TypeTag>(&self) -> bool {
        self.typeKind() == T::kTypeKind
    }
    // Source: src/sksl/ir/SkSLType.h:215-219
    /// Safety: actual TypeData begins the initialized concrete T object.
    unsafe fn as_type_node<T: TypeTag>(&self) -> &T {
        debug_assert!(self.is::<T>());
        &*(self.type_data() as *const TypeData as *const T)
    }
    // Source: src/sksl/ir/SkSLType.h:221-225
    /// Safety: actual TypeData begins T and exclusive pinned access covers T.
    unsafe fn as_type_node_mut<T: TypeTag>(self: Pin<&mut Self>) -> Pin<&mut T> {
        debug_assert!(self.as_ref().get_ref().is::<T>());
        Pin::new_unchecked(
            &mut *(self.type_data_mut().get_unchecked_mut() as *mut TypeData as *mut T),
        )
    }
}
impl<T: Type + ?Sized> TypeExt for T {}
// Native polymorphic Type protocol. Concrete types supply projections
// into their own inherited state. No concrete placeholder Type is constructed.
/// Safety: projections identify this actual concrete Type. Override only native
/// virtual methods; retain native nonvirtual methods unchanged. Borrowed state
/// obeys Symbol/Field lifetime leases and all native type-kind invariants.
pub unsafe trait Type: Symbol {
    fn type_data(&self) -> &TypeData;
    fn type_data_mut(self: Pin<&mut Self>) -> Pin<&mut TypeData>;
    fn as_type(&self) -> &dyn Type;
    // Source: src/sksl/ir/SkSLType.h:150-150
    fn getArrayName(&self, arraySize: i32) -> Vec<u8> {
        getArrayName(self.as_type(), arraySize)
    }
    // Source: src/sksl/ir/SkSLType.h:236-238
    fn isBuiltin(&self) -> bool {
        true
    }
    // Source: src/sksl/ir/SkSLType.h:240-242
    fn displayName(&self) -> Vec<u8> {
        self.scalarTypeForLiteral().name().to_vec()
    }
    // Source: src/sksl/ir/SkSLType.h:244-246
    // Concrete IRNode::description forwards here, preserving native Type override.
    fn type_description(&self) -> Vec<u8> {
        self.displayName()
    }
    // Source: src/sksl/ir/SkSLType.h:252-254
    fn isAllowedInES2(&self) -> bool {
        true
    }
    // Source: src/sksl/ir/SkSLType.h:263-267
    fn isAllowedInUniform(&self, _errorPosition: Option<&mut Position>) -> bool {
        !self.isOpaque()
    }
    // Source: src/sksl/ir/SkSLType.h:270-272
    fn resolve(&self) -> &dyn Type {
        self.as_type()
    }
    // Source: src/sksl/ir/SkSLType.h:275-277
    fn matches(&self, that: &dyn Type) -> bool {
        core::ptr::eq(self.resolve().type_data(), that.resolve().type_data())
    }
    // Source: src/sksl/ir/SkSLType.h:282-284
    fn abbreviatedName(&self) -> &core::ffi::CStr {
        unsafe { core::ffi::CStr::from_ptr(self.type_data().fAbbreviatedName.as_ptr().cast()) }
    }
    // Source: src/sksl/ir/SkSLType.h:289-291
    fn typeKind(&self) -> TypeKind {
        self.type_data().fTypeKind
    }
    // Source: src/sksl/ir/SkSLType.h:296-298
    fn numberKind(&self) -> NumberKind {
        NumberKind::kNonnumeric
    }
    // Source: src/sksl/ir/SkSLType.h:303-305
    fn isBoolean(&self) -> bool {
        self.numberKind() == NumberKind::kBoolean
    }
    // Source: src/sksl/ir/SkSLType.h:310-319
    fn isNumber(&self) -> bool {
        match self.numberKind() {
            NumberKind::kFloat | NumberKind::kSigned | NumberKind::kUnsigned => true,
            _ => false,
        }
    }
    // Source: src/sksl/ir/SkSLType.h:324-326
    fn isFloat(&self) -> bool {
        self.numberKind() == NumberKind::kFloat
    }
    // Source: src/sksl/ir/SkSLType.h:331-333
    fn isSigned(&self) -> bool {
        self.numberKind() == NumberKind::kSigned
    }
    // Source: src/sksl/ir/SkSLType.h:338-340
    fn isUnsigned(&self) -> bool {
        self.numberKind() == NumberKind::kUnsigned
    }
    // Source: src/sksl/ir/SkSLType.h:345-353
    fn isInteger(&self) -> bool {
        match self.numberKind() {
            NumberKind::kSigned | NumberKind::kUnsigned => true,
            _ => false,
        }
    }
    // Source: src/sksl/ir/SkSLType.h:359-372
    fn isOpaque(&self) -> bool {
        match self.type_data().fTypeKind {
            TypeKind::kAtomic
            | TypeKind::kBlender
            | TypeKind::kColorFilter
            | TypeKind::kSampler
            | TypeKind::kSeparateSampler
            | TypeKind::kShader
            | TypeKind::kTexture => true,
            _ => false,
        }
    }
    // Source: src/sksl/ir/SkSLType.h:377-379
    fn isStorageTexture(&self) -> bool {
        self.typeKind() == TypeKind::kTexture && self.dimensions() != 6
    }
    // Source: src/sksl/ir/SkSLType.h:385-388
    fn priority(&self) -> i32 {
        native_type_debugfail("not a number type");
        -1
    }
    // Source: src/sksl/ir/SkSLType.h:394-396
    fn canCoerceTo(&self, other: &dyn Type, allowNarrowing: bool) -> bool {
        self.coercionCost(other).isPossible(allowNarrowing)
    }
    // Language-only nonvirtual dispatch bridge; actual body mapped in cpp order below.
    fn coercionCost(&self, other: &dyn Type) -> CoercionCost {
        coercionCost(self.as_type(), other)
    }
    // Source: src/sksl/ir/SkSLType.h:410-412
    fn componentType(&self) -> &dyn Type {
        self.as_type()
    }
    // Source: src/sksl/ir/SkSLType.h:417-419
    unsafe fn columnType<'a>(
        &'a self,
        context: &'a crate::src::sksl::SkSLContext::Context,
    ) -> &'a dyn Type {
        self.componentType().toCompound(context, self.rows(), 1)
    }
    // Source: src/sksl/ir/SkSLType.h:425-428
    fn textureType(&self) -> &dyn Type {
        native_type_debugfail("not a sampler type");
        self.as_type()
    }
    // Source: src/sksl/ir/SkSLType.h:435-438
    fn columns(&self) -> i32 {
        native_type_debugfail("type does not have columns");
        -1
    }
    // Source: src/sksl/ir/SkSLType.h:444-447
    fn rows(&self) -> i32 {
        native_type_debugfail("type does not have rows");
        -1
    }
    // Source: src/sksl/ir/SkSLType.h:450-453
    fn minimumValue(&self) -> f64 {
        native_type_debugfail("type does not have a minimum value");
        f64::NEG_INFINITY
    }
    // Source: src/sksl/ir/SkSLType.h:455-458
    fn maximumValue(&self) -> f64 {
        native_type_debugfail("type does not have a maximum value");
        f64::INFINITY
    }
    // Source: src/sksl/ir/SkSLType.h:463-465
    fn slotCount(&self) -> usize {
        0
    }
    // Source: src/sksl/ir/SkSLType.h:471-473
    fn slotType(&self, _slot: usize) -> &dyn Type {
        self.as_type()
    }
    // Source: src/sksl/ir/SkSLType.h:475-477
    fn fields(&self) -> &[Field] {
        std::process::abort()
    }
    // Source: src/sksl/ir/SkSLType.h:482-485
    fn coercibleTypes(&self) -> &[NonNull<dyn Type>] {
        native_type_debugfail("Internal error: not a generic type");
        &[]
    }
    // Source: src/sksl/ir/SkSLType.h:487-490
    fn dimensions(&self) -> u32 {
        native_type_debugfail("Internal error: not a texture type");
        0
    }
    // Source: src/sksl/ir/SkSLType.h:492-495
    fn isDepth(&self) -> bool {
        native_type_debugfail("Internal error: not a texture type");
        false
    }
    // Source: src/sksl/ir/SkSLType.h:497-500
    fn isArrayedTexture(&self) -> bool {
        native_type_debugfail("Internal error: not a texture type");
        false
    }
    // Source: src/sksl/ir/SkSLType.h:502-504
    fn isVoid(&self) -> bool {
        self.type_data().fTypeKind == TypeKind::kVoid
    }
    // Source: src/sksl/ir/SkSLType.h:506-508
    fn isGeneric(&self) -> bool {
        self.type_data().fTypeKind == TypeKind::kGeneric
    }
    // Source: src/sksl/ir/SkSLType.h:510-512
    fn isSampler(&self) -> bool {
        self.type_data().fTypeKind == TypeKind::kSampler
    }
    // Source: src/sksl/ir/SkSLType.h:514-516
    fn isAtomic(&self) -> bool {
        self.typeKind() == TypeKind::kAtomic
    }
    // Source: src/sksl/ir/SkSLType.h:518-520
    fn isScalar(&self) -> bool {
        false
    }
    // Source: src/sksl/ir/SkSLType.h:522-524
    fn isLiteral(&self) -> bool {
        false
    }
    // Source: src/sksl/ir/SkSLType.h:526-528
    fn scalarTypeForLiteral(&self) -> &dyn Type {
        self.as_type()
    }
    // Source: src/sksl/ir/SkSLType.h:530-532
    fn isVector(&self) -> bool {
        false
    }
    // Source: src/sksl/ir/SkSLType.h:534-536
    fn isMatrix(&self) -> bool {
        false
    }
    // Source: src/sksl/ir/SkSLType.h:538-540
    fn isArray(&self) -> bool {
        false
    }
    // Source: src/sksl/ir/SkSLType.h:542-544
    fn isUnsizedArray(&self) -> bool {
        false
    }
    // Source: src/sksl/ir/SkSLType.h:546-548
    fn isStruct(&self) -> bool {
        false
    }
    // Source: src/sksl/ir/SkSLType.h:550-552
    fn isInterfaceBlock(&self) -> bool {
        false
    }
    // Source: src/sksl/ir/SkSLType.h:556-560
    fn isEffectChild(&self) -> bool {
        self.type_data().fTypeKind == TypeKind::kColorFilter
            || self.type_data().fTypeKind == TypeKind::kShader
            || self.type_data().fTypeKind == TypeKind::kBlender
    }
    // Source: src/sksl/ir/SkSLType.h:562-565
    fn isMultisampled(&self) -> bool {
        native_type_debugfail("not a texture type");
        false
    }
    // Source: src/sksl/ir/SkSLType.h:567-570
    fn textureAccess(&self) -> TextureAccess {
        native_type_debugfail("not a texture type");
        TextureAccess::kSample
    }
    // Source: src/sksl/ir/SkSLType.h:572-574
    fn hasPrecision(&self) -> bool {
        self.componentType().isNumber() || self.isSampler()
    }
    // Source: src/sksl/ir/SkSLType.h:576-578
    fn highPrecision(&self) -> bool {
        self.bitWidth() >= 32
    }
    // Source: src/sksl/ir/SkSLType.h:580-582
    fn bitWidth(&self) -> i32 {
        0
    }
    // Source: src/sksl/ir/SkSLType.h:584-586
    fn isOrContainsArray(&self) -> bool {
        false
    }
    // Source: src/sksl/ir/SkSLType.h:588-590
    fn isOrContainsUnsizedArray(&self) -> bool {
        false
    }
    // Source: src/sksl/ir/SkSLType.h:592-594
    fn isOrContainsAtomic(&self) -> bool {
        false
    }
    // Source: src/sksl/ir/SkSLType.h:596-598
    fn isOrContainsBool(&self) -> bool {
        false
    }
    // Source: src/sksl/ir/SkSLType.h:604-604
    unsafe fn toCompound<'a>(
        &'a self,
        context: &'a crate::src::sksl::SkSLContext::Context,
        columns: i32,
        rows: i32,
    ) -> &'a dyn Type {
        toCompound(self.as_type(), context, columns, rows)
    }

    // Source: src/sksl/ir/SkSLType.h:649-654
    /// Safety: borrowed name backing outlives this actual Type; caller constructs
    /// only a genuine concrete native-derived Type, preserving its kind invariants.
    unsafe fn new_data(
        name: &[u8],
        abbrev: &core::ffi::CStr,
        kind: TypeKind,
        pos: Position,
    ) -> TypeData
    where
        Self: Sized,
    {
        let mut value = TypeData {
            base: SymbolData::new(pos, kIRNodeKind, name, None),
            fAbbreviatedName: [0; 4],
            fTypeKind: kind,
        };
        let bytes = abbrev.to_bytes_with_nul();
        debug_assert!(bytes.len() - 1 <= kMaxAbbrevLength);
        // The native constructor requires at most three bytes; reject invalid input
        // before Rust indexing rather than publishing an invalid C-string.
        value.fAbbreviatedName[..bytes.len()].copy_from_slice(bytes);
        value
    }
    // Source: src/sksl/ir/SkSLType.h:665-667
    fn isInRootSymbolTable(&self) -> bool {
        !(self.isArray() || self.isStruct())
    }
    // Source: src/sksl/ir/SkSLType.h:670-672
    fn structNestingDepth(&self) -> i32 {
        0
    }
}

// Selected Rust replacement for native SkDEBUGFAIL; only debug builds terminate.
// Native SkLog decoration and trap signal differ and are counted adapted.
fn native_type_debugfail(message: &str) {
    #[cfg(debug_assertions)]
    {
        eprintln!("{message}");
        std::process::abort();
    }
    #[cfg(not(debug_assertions))]
    let _ = message;
}
use super::SkSLIRNode::{IRNode, IRNodeBox, IRNodeData};
use core::mem::MaybeUninit;
// Language-only inheritance projection: exactly the same object and base address.
// Dynamic destruction remains IRNodeBox's concrete destructor then Poolable delete.
macro_rules! type_projections {
    ($ty:ty) => {
        impl IRNode for $ty {
            fn description(&self) -> Vec<u8> {
                self.type_description()
            }
            fn node(&self) -> &IRNodeData {
                &self.base.base.base
            }
            fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
                unsafe { self.map_unchecked_mut(|v| &mut v.base.base.base) }
            }
        }
        unsafe impl Symbol for $ty {
            unsafe fn as_type_symbol(&self) -> &dyn Type {
                self
            }
            fn symbol_data(&self) -> &SymbolData {
                &self.base.base
            }
            fn symbol_data_mut(self: Pin<&mut Self>) -> Pin<&mut SymbolData> {
                unsafe { self.map_unchecked_mut(|v| &mut v.base.base) }
            }
        }
    };
}
macro_rules! type_base_projections {
    () => {
        fn type_data(&self) -> &TypeData {
            &self.base
        }
        fn type_data_mut(self: Pin<&mut Self>) -> Pin<&mut TypeData> {
            unsafe { self.map_unchecked_mut(|v| &mut v.base) }
        }
        fn as_type(&self) -> &dyn Type {
            self
        }
    };
}
// Native Type itself is concrete (MakeSpecialType constructs it directly).
// This is its actual complete in-scope state, not a stand-in for a subclass.
impl IRNode for TypeData {
    fn description(&self) -> Vec<u8> {
        self.type_description()
    }
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|v| &mut v.base.base) }
    }
}
unsafe impl Symbol for TypeData {
    unsafe fn as_type_symbol(&self) -> &dyn Type {
        self
    }
    fn symbol_data(&self) -> &SymbolData {
        &self.base
    }
    fn symbol_data_mut(self: Pin<&mut Self>) -> Pin<&mut SymbolData> {
        unsafe { self.map_unchecked_mut(|v| &mut v.base) }
    }
}
unsafe impl Type for TypeData {
    fn type_data(&self) -> &TypeData {
        self
    }
    fn type_data_mut(self: Pin<&mut Self>) -> Pin<&mut TypeData> {
        self
    }
    fn as_type(&self) -> &dyn Type {
        self
    }
}
// Source: src/sksl/ir/SkSLType.cpp:41-41
const kMaxStructDepth: i32 = 8;
// Source: src/sksl/ir/SkSLType.cpp:43-43
// Source: src/sksl/ir/SkSLType.cpp:156-156
#[repr(C)]
struct AliasType {
    base: TypeData,
    fTargetType: NonNull<dyn Type>,
}
type_projections!(AliasType);
impl AliasType {
    // Source: src/sksl/ir/SkSLType.cpp:45-47
    unsafe fn new(name: &[u8], target: &dyn Type) -> Self {
        Self {
            base: <TypeData as Type>::new_data(
                name,
                target.abbreviatedName(),
                target.typeKind(),
                Position::default(),
            ),
            fTargetType: borrow_type(target),
        }
    }
}
unsafe impl Type for AliasType {
    type_base_projections!();
    // Source: src/sksl/ir/SkSLType.cpp:49-51
    fn resolve(&self) -> &dyn Type {
        unsafe { self.fTargetType.as_ref() }
    }
    // Source: src/sksl/ir/SkSLType.cpp:53-55
    fn componentType(&self) -> &dyn Type {
        unsafe { self.fTargetType.as_ref() }.componentType()
    }
    // Source: src/sksl/ir/SkSLType.cpp:57-59
    fn numberKind(&self) -> NumberKind {
        unsafe { self.fTargetType.as_ref() }.numberKind()
    }
    // Source: src/sksl/ir/SkSLType.cpp:61-63
    fn priority(&self) -> i32 {
        unsafe { self.fTargetType.as_ref() }.priority()
    }
    // Source: src/sksl/ir/SkSLType.cpp:65-67
    fn columns(&self) -> i32 {
        unsafe { self.fTargetType.as_ref() }.columns()
    }
    // Source: src/sksl/ir/SkSLType.cpp:69-71
    fn rows(&self) -> i32 {
        unsafe { self.fTargetType.as_ref() }.rows()
    }
    // Source: src/sksl/ir/SkSLType.cpp:73-75
    fn bitWidth(&self) -> i32 {
        unsafe { self.fTargetType.as_ref() }.bitWidth()
    }
    // Source: src/sksl/ir/SkSLType.cpp:77-79
    fn isAllowedInES2(&self) -> bool {
        unsafe { self.fTargetType.as_ref() }.isAllowedInES2()
    }
    // Source: src/sksl/ir/SkSLType.cpp:81-83
    fn isOrContainsBool(&self) -> bool {
        unsafe { self.fTargetType.as_ref() }.isOrContainsBool()
    }
    // Source: src/sksl/ir/SkSLType.cpp:85-87
    fn isOrContainsAtomic(&self) -> bool {
        unsafe { self.fTargetType.as_ref() }.isOrContainsAtomic()
    }
    // Source: src/sksl/ir/SkSLType.cpp:89-91
    fn slotCount(&self) -> usize {
        unsafe { self.fTargetType.as_ref() }.slotCount()
    }
    // Source: src/sksl/ir/SkSLType.cpp:93-95
    fn slotType(&self, n: usize) -> &dyn Type {
        unsafe { self.fTargetType.as_ref() }.slotType(n)
    }
    // Source: src/sksl/ir/SkSLType.cpp:97-99
    fn dimensions(&self) -> u32 {
        unsafe { self.fTargetType.as_ref() }.dimensions()
    }
    // Source: src/sksl/ir/SkSLType.cpp:101-103
    fn isDepth(&self) -> bool {
        unsafe { self.fTargetType.as_ref() }.isDepth()
    }
    // Source: src/sksl/ir/SkSLType.cpp:105-107
    fn isArrayedTexture(&self) -> bool {
        unsafe { self.fTargetType.as_ref() }.isArrayedTexture()
    }
    // Source: src/sksl/ir/SkSLType.cpp:109-111
    fn isScalar(&self) -> bool {
        unsafe { self.fTargetType.as_ref() }.isScalar()
    }
    // Source: src/sksl/ir/SkSLType.cpp:113-115
    fn isLiteral(&self) -> bool {
        unsafe { self.fTargetType.as_ref() }.isLiteral()
    }
    // Source: src/sksl/ir/SkSLType.cpp:117-119
    fn isVector(&self) -> bool {
        unsafe { self.fTargetType.as_ref() }.isVector()
    }
    // Source: src/sksl/ir/SkSLType.cpp:121-123
    fn isMatrix(&self) -> bool {
        unsafe { self.fTargetType.as_ref() }.isMatrix()
    }
    // Source: src/sksl/ir/SkSLType.cpp:125-127
    fn isArray(&self) -> bool {
        unsafe { self.fTargetType.as_ref() }.isArray()
    }
    // Source: src/sksl/ir/SkSLType.cpp:129-131
    fn isUnsizedArray(&self) -> bool {
        unsafe { self.fTargetType.as_ref() }.isUnsizedArray()
    }
    // Source: src/sksl/ir/SkSLType.cpp:133-135
    fn isStruct(&self) -> bool {
        unsafe { self.fTargetType.as_ref() }.isStruct()
    }
    // Source: src/sksl/ir/SkSLType.cpp:137-139
    fn isInterfaceBlock(&self) -> bool {
        unsafe { self.fTargetType.as_ref() }.isInterfaceBlock()
    }
    // Source: src/sksl/ir/SkSLType.cpp:141-143
    fn isMultisampled(&self) -> bool {
        unsafe { self.fTargetType.as_ref() }.isMultisampled()
    }
    // Source: src/sksl/ir/SkSLType.cpp:145-147
    fn textureAccess(&self) -> TextureAccess {
        unsafe { self.fTargetType.as_ref() }.textureAccess()
    }
    // Source: src/sksl/ir/SkSLType.cpp:149-151
    fn coercibleTypes(&self) -> &[NonNull<dyn Type>] {
        unsafe { self.fTargetType.as_ref() }.coercibleTypes()
    }
}
// Source: src/sksl/ir/SkSLType.cpp:159-159
// Source: src/sksl/ir/SkSLType.cpp:242-244
#[repr(C)]
struct ArrayType {
    base: TypeData,
    fComponentType: NonNull<dyn Type>,
    fCount: i32,
    fIsBuiltin: bool,
}
type_projections!(ArrayType);
// Source: src/sksl/ir/SkSLType.cpp:161-161
impl TypeTag for ArrayType {
    const kTypeKind: TypeKind = TypeKind::kArray;
}
impl ArrayType {
    // Source: src/sksl/ir/SkSLType.cpp:163-172
    unsafe fn new(
        name: &[u8],
        abbrev: &core::ffi::CStr,
        component: &dyn Type,
        count: i32,
        builtin: bool,
    ) -> Self {
        let value = Self {
            base: <TypeData as Type>::new_data(name, abbrev, Self::kTypeKind, Position::default()),
            fComponentType: borrow_type(component),
            fCount: count,
            fIsBuiltin: builtin,
        };
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLType.cpp\0".as_ptr(),169; @condition b"count > 0 || count == kUnsizedArray\0".as_ptr(); count > 0 || count == kUnsizedArray);
        debug_assert!(component.typeKind() != TypeKind::kArray);
        value
    }
}
unsafe impl Type for ArrayType {
    type_base_projections!();
    // Source: src/sksl/ir/SkSLType.cpp:174-178
    fn matches(&self, other: &dyn Type) -> bool {
        let that = other.resolve();
        that.isArray()
            && self.columns() == that.columns()
            && self.componentType().matches(that.componentType())
    }
    // Source: src/sksl/ir/SkSLType.cpp:180-182
    fn isArray(&self) -> bool {
        true
    }
    // Source: src/sksl/ir/SkSLType.cpp:184-186
    fn isBuiltin(&self) -> bool {
        self.fIsBuiltin
    }
    // Source: src/sksl/ir/SkSLType.cpp:188-190
    fn isOrContainsArray(&self) -> bool {
        true
    }
    // Source: src/sksl/ir/SkSLType.cpp:192-194
    fn isOrContainsAtomic(&self) -> bool {
        unsafe { self.fComponentType.as_ref() }.isOrContainsAtomic()
    }
    // Source: src/sksl/ir/SkSLType.cpp:196-198
    fn isOrContainsBool(&self) -> bool {
        unsafe { self.fComponentType.as_ref() }.isOrContainsBool()
    }
    // Source: src/sksl/ir/SkSLType.cpp:200-202
    fn isUnsizedArray(&self) -> bool {
        self.fCount == kUnsizedArray
    }
    // Source: src/sksl/ir/SkSLType.cpp:204-206
    fn isOrContainsUnsizedArray(&self) -> bool {
        self.isUnsizedArray() || unsafe { self.fComponentType.as_ref() }.isOrContainsUnsizedArray()
    }
    // Source: src/sksl/ir/SkSLType.cpp:208-210
    fn componentType(&self) -> &dyn Type {
        unsafe { self.fComponentType.as_ref() }
    }
    // Source: src/sksl/ir/SkSLType.cpp:212-214
    fn columns(&self) -> i32 {
        self.fCount
    }
    // Source: src/sksl/ir/SkSLType.cpp:216-218
    fn bitWidth(&self) -> i32 {
        unsafe { self.fComponentType.as_ref() }.bitWidth()
    }
    // Source: src/sksl/ir/SkSLType.cpp:220-222
    fn isAllowedInES2(&self) -> bool {
        unsafe { self.fComponentType.as_ref() }.isAllowedInES2()
    }
    // Source: src/sksl/ir/SkSLType.cpp:224-226
    fn isAllowedInUniform(&self, pos: Option<&mut Position>) -> bool {
        unsafe { self.fComponentType.as_ref() }.isAllowedInUniform(pos)
    }
    // Source: src/sksl/ir/SkSLType.cpp:228-232
    fn slotCount(&self) -> usize {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLType.cpp\0".as_ptr(),229; @condition b"fCount != kUnsizedArray\0".as_ptr(); self.fCount != kUnsizedArray);
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLType.cpp\0".as_ptr(),230; @condition b"fCount > 0\0".as_ptr(); self.fCount > 0);
        (self.fCount as usize).wrapping_mul(unsafe { self.fComponentType.as_ref() }.slotCount())
    }
    // Source: src/sksl/ir/SkSLType.cpp:234-237
    fn slotType(&self, n: usize) -> &dyn Type {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLType.cpp\0".as_ptr(),235; @condition b"fCount == kUnsizedArray || n < this->slotCount()\0".as_ptr(); self.fCount == kUnsizedArray || n < self.slotCount());
        unsafe { self.fComponentType.as_ref() }
            .slotType(n % unsafe { self.fComponentType.as_ref() }.slotCount())
    }
}
// Source: src/sksl/ir/SkSLType.cpp:247-247
// Source: src/sksl/ir/SkSLType.cpp:270-272
#[repr(C)]
struct GenericType {
    base: TypeData,
    fCoercibleTypes: [MaybeUninit<NonNull<dyn Type>>; 9],
    fSlotType: Option<NonNull<dyn Type>>,
    fNumTypes: usize,
}
type_projections!(GenericType);
// Source: src/sksl/ir/SkSLType.cpp:249-249
impl TypeTag for GenericType {
    const kTypeKind: TypeKind = TypeKind::kGeneric;
}
impl GenericType {
    // Source: src/sksl/ir/SkSLType.cpp:251-257
    unsafe fn new(
        name: &core::ffi::CStr,
        coercibleTypes: &[NonNull<dyn Type>],
        slotType: Option<NonNull<dyn Type>>,
    ) -> Self {
        let mut value = Self {
            base: <Self as Type>::new_data(
                name.to_bytes(),
                c"G",
                Self::kTypeKind,
                Position::default(),
            ),
            fCoercibleTypes: [MaybeUninit::uninit(); 9],
            fSlotType: slotType,
            fNumTypes: coercibleTypes.len(),
        };
        debug_assert!(value.fNumTypes <= value.fCoercibleTypes.len());
        for (i, t) in coercibleTypes.iter().enumerate() {
            value.fCoercibleTypes[i].write(*t);
        }
        value
    }
}
unsafe impl Type for GenericType {
    type_base_projections!();
    // Source: src/sksl/ir/SkSLType.cpp:259-261
    fn coercibleTypes(&self) -> &[NonNull<dyn Type>] {
        unsafe { core::slice::from_raw_parts(self.fCoercibleTypes.as_ptr().cast(), self.fNumTypes) }
    }
    // Source: src/sksl/ir/SkSLType.cpp:263-265
    fn slotType(&self, _n: usize) -> &dyn Type {
        unsafe { self.fSlotType.unwrap_unchecked().as_ref() }
    }
}
// Source: src/sksl/ir/SkSLType.cpp:275-275
// Source: src/sksl/ir/SkSLType.cpp:340-340
// Source: src/sksl/ir/SkSLType.cpp:341-341
#[repr(C)]
struct LiteralType {
    base: TypeData,
    fScalarType: NonNull<dyn Type>,
    fPriority: i8,
}
type_projections!(LiteralType);
// Source: src/sksl/ir/SkSLType.cpp:277-277
impl TypeTag for LiteralType {
    const kTypeKind: TypeKind = TypeKind::kLiteral;
}
impl LiteralType {
    // Source: src/sksl/ir/SkSLType.cpp:279-282
    unsafe fn new(name: &core::ffi::CStr, scalarType: NonNull<dyn Type>, priority: i8) -> Self {
        Self {
            base: <Self as Type>::new_data(
                name.to_bytes(),
                c"L",
                Self::kTypeKind,
                Position::default(),
            ),
            fScalarType: scalarType,
            fPriority: priority,
        }
    }
}
unsafe impl Type for LiteralType {
    type_base_projections!();
    // Source: src/sksl/ir/SkSLType.cpp:284-286
    fn scalarTypeForLiteral(&self) -> &dyn Type {
        unsafe { self.fScalarType.as_ref() }
    }
    // Source: src/sksl/ir/SkSLType.cpp:288-290
    fn priority(&self) -> i32 {
        self.fPriority as i32
    }
    // Source: src/sksl/ir/SkSLType.cpp:292-294
    fn columns(&self) -> i32 {
        1
    }
    // Source: src/sksl/ir/SkSLType.cpp:296-298
    fn rows(&self) -> i32 {
        1
    }
    // Source: src/sksl/ir/SkSLType.cpp:300-302
    fn numberKind(&self) -> NumberKind {
        unsafe { self.fScalarType.as_ref() }.numberKind()
    }
    // Source: src/sksl/ir/SkSLType.cpp:304-306
    fn bitWidth(&self) -> i32 {
        unsafe { self.fScalarType.as_ref() }.bitWidth()
    }
    // Source: src/sksl/ir/SkSLType.cpp:308-310
    fn minimumValue(&self) -> f64 {
        unsafe { self.fScalarType.as_ref() }.minimumValue()
    }
    // Source: src/sksl/ir/SkSLType.cpp:312-314
    fn maximumValue(&self) -> f64 {
        unsafe { self.fScalarType.as_ref() }.maximumValue()
    }
    // Source: src/sksl/ir/SkSLType.cpp:316-318
    fn isScalar(&self) -> bool {
        true
    }
    // Source: src/sksl/ir/SkSLType.cpp:320-322
    fn isLiteral(&self) -> bool {
        true
    }
    // Source: src/sksl/ir/SkSLType.cpp:324-326
    fn isOrContainsBool(&self) -> bool {
        unsafe { self.fScalarType.as_ref() }.isBoolean()
    }
    // Source: src/sksl/ir/SkSLType.cpp:328-330
    fn slotCount(&self) -> usize {
        1
    }
    // Source: src/sksl/ir/SkSLType.cpp:332-335
    fn slotType(&self, n: usize) -> &dyn Type {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLType.cpp\0".as_ptr(),333; @condition b"n == 0\0".as_ptr(); n == 0);
        unsafe { self.fScalarType.as_ref() }
    }
}
// Source: src/sksl/ir/SkSLType.cpp:345-345
// Source: src/sksl/ir/SkSLType.cpp:445-445
// Source: src/sksl/ir/SkSLType.cpp:446-446
// Source: src/sksl/ir/SkSLType.cpp:447-447
#[repr(C)]
struct ScalarType {
    base: TypeData,
    fNumberKind: NumberKind,
    fPriority: i8,
    fBitWidth: i8,
}
type_projections!(ScalarType);
// Source: src/sksl/ir/SkSLType.cpp:347-347
impl TypeTag for ScalarType {
    const kTypeKind: TypeKind = TypeKind::kScalar;
}
impl ScalarType {
    // Source: src/sksl/ir/SkSLType.cpp:349-354
    unsafe fn new(
        name: &[u8],
        abbrev: &core::ffi::CStr,
        numberKind: NumberKind,
        priority: i8,
        bitWidth: i8,
    ) -> Self {
        Self {
            base: <Self as Type>::new_data(name, abbrev, Self::kTypeKind, Position::default()),
            fNumberKind: numberKind,
            fPriority: priority,
            fBitWidth: bitWidth,
        }
    }
}
unsafe impl Type for ScalarType {
    type_base_projections!();
    // Source: src/sksl/ir/SkSLType.cpp:356-358
    fn numberKind(&self) -> NumberKind {
        self.fNumberKind
    }
    // Source: src/sksl/ir/SkSLType.cpp:360-362
    fn priority(&self) -> i32 {
        self.fPriority as i32
    }
    // Source: src/sksl/ir/SkSLType.cpp:364-366
    fn bitWidth(&self) -> i32 {
        self.fBitWidth as i32
    }
    // Source: src/sksl/ir/SkSLType.cpp:368-370
    fn columns(&self) -> i32 {
        1
    }
    // Source: src/sksl/ir/SkSLType.cpp:372-374
    fn rows(&self) -> i32 {
        1
    }
    // Source: src/sksl/ir/SkSLType.cpp:376-378
    fn isScalar(&self) -> bool {
        true
    }
    // Source: src/sksl/ir/SkSLType.cpp:380-382
    fn isAllowedInES2(&self) -> bool {
        self.fNumberKind != NumberKind::kUnsigned
    }
    // Source: src/sksl/ir/SkSLType.cpp:384-386
    fn isAllowedInUniform(&self, _errorPosition: Option<&mut Position>) -> bool {
        self.fNumberKind != NumberKind::kBoolean
    }
    // Source: src/sksl/ir/SkSLType.cpp:388-390
    fn isOrContainsBool(&self) -> bool {
        self.fNumberKind == NumberKind::kBoolean
    }
    // Source: src/sksl/ir/SkSLType.cpp:392-394
    fn slotCount(&self) -> usize {
        1
    }
    // Source: src/sksl/ir/SkSLType.cpp:396-399
    fn slotType(&self, n: usize) -> &dyn Type {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLType.cpp\0".as_ptr(),397; @condition b"n == 0\0".as_ptr(); n == 0);
        self
    }
    // Source: src/sksl/ir/SkSLType.cpp:408-422
    fn minimumValue(&self) -> f64 {
        match self.numberKind() {
            NumberKind::kSigned => {
                if self.highPrecision() {
                    i32::MIN as f64
                } else {
                    i16::MIN as f64
                }
            }
            NumberKind::kUnsigned => 0.0,
            _ => {
                if self.highPrecision() {
                    f32::MIN as f64
                } else {
                    -(crate::src::core::SkHalf::SkHalfToFloat(crate::src::core::SkHalf::SK_HalfMax)
                        as f64)
                }
            }
        }
    }
    // Source: src/sksl/ir/SkSLType.cpp:425-440
    fn maximumValue(&self) -> f64 {
        match self.numberKind() {
            NumberKind::kSigned => {
                if self.highPrecision() {
                    i32::MAX as f64
                } else {
                    i16::MAX as f64
                }
            }
            NumberKind::kUnsigned => {
                if self.highPrecision() {
                    u32::MAX as f64
                } else {
                    u16::MAX as f64
                }
            }
            _ => {
                if self.highPrecision() {
                    f32::MAX as f64
                } else {
                    crate::src::core::SkHalf::SkHalfToFloat(crate::src::core::SkHalf::SK_HalfMax)
                        as f64
                }
            }
        }
    }
}
// Source: src/sksl/ir/SkSLType.cpp:450-450
#[repr(C)]
struct AtomicType {
    base: TypeData,
}
type_projections!(AtomicType);
// Source: src/sksl/ir/SkSLType.cpp:452-452
impl TypeTag for AtomicType {
    const kTypeKind: TypeKind = TypeKind::kAtomic;
}
impl AtomicType {
    // Source: src/sksl/ir/SkSLType.cpp:454-454
    unsafe fn new(name: &[u8], abbrev: &core::ffi::CStr) -> Self {
        Self {
            base: <TypeData as Type>::new_data(name, abbrev, Self::kTypeKind, Position::default()),
        }
    }
}
unsafe impl Type for AtomicType {
    type_base_projections!();
    // Source: src/sksl/ir/SkSLType.cpp:456-456
    fn isAllowedInES2(&self) -> bool {
        false
    }
    // Source: src/sksl/ir/SkSLType.cpp:458-458
    fn isAllowedInUniform(&self, _: Option<&mut Position>) -> bool {
        false
    }
    // Source: src/sksl/ir/SkSLType.cpp:460-460
    fn isOrContainsAtomic(&self) -> bool {
        true
    }
    // Source: src/sksl/ir/SkSLType.cpp:462-465
    fn slotType(&self, n: usize) -> &dyn Type {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLType.cpp\0".as_ptr(),463; @condition b"n == 0\0".as_ptr(); n == 0);
        self
    }
}
// Source: src/sksl/ir/SkSLType.cpp:471-471
// Source: src/sksl/ir/SkSLType.cpp:521-521
// Source: src/sksl/ir/SkSLType.cpp:522-522
// Source: src/sksl/ir/SkSLType.cpp:523-523
#[repr(C)]
struct MatrixType {
    base: TypeData,
    fComponentType: NonNull<ScalarType>,
    fColumns: i8,
    fRows: i8,
}
type_projections!(MatrixType);
// Source: src/sksl/ir/SkSLType.cpp:473-473
impl TypeTag for MatrixType {
    const kTypeKind: TypeKind = TypeKind::kMatrix;
}
impl MatrixType {
    // Source: src/sksl/ir/SkSLType.cpp:475-483
    unsafe fn new(
        name: &[u8],
        abbrev: &core::ffi::CStr,
        componentType: &dyn Type,
        columns: i8,
        rows: i8,
    ) -> Self {
        let value = Self {
            base: <Self as Type>::new_data(name, abbrev, Self::kTypeKind, Position::default()),
            fComponentType: NonNull::from(componentType.as_type_node::<ScalarType>()),
            fColumns: columns,
            fRows: rows,
        };
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLType.cpp\0".as_ptr(),481; @condition b"columns >= 2 && columns <= 4\0".as_ptr(); columns >= 2 && columns <= 4);
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLType.cpp\0".as_ptr(),482; @condition b"rows >= 2 && rows <= 4\0".as_ptr(); rows >= 2 && rows <= 4);
        value
    }
}
unsafe impl Type for MatrixType {
    type_base_projections!();
    // Source: src/sksl/ir/SkSLType.cpp:485-487
    fn componentType(&self) -> &dyn Type {
        unsafe { self.fComponentType.as_ref() }
    }
    // Source: src/sksl/ir/SkSLType.cpp:489-491
    fn columns(&self) -> i32 {
        self.fColumns as i32
    }
    // Source: src/sksl/ir/SkSLType.cpp:493-495
    fn rows(&self) -> i32 {
        self.fRows as i32
    }
    // Source: src/sksl/ir/SkSLType.cpp:497-499
    fn bitWidth(&self) -> i32 {
        self.componentType().bitWidth()
    }
    // Source: src/sksl/ir/SkSLType.cpp:501-503
    fn isMatrix(&self) -> bool {
        true
    }
    // Source: src/sksl/ir/SkSLType.cpp:505-507
    fn isAllowedInES2(&self) -> bool {
        self.fColumns == self.fRows
    }
    // Source: src/sksl/ir/SkSLType.cpp:509-511
    fn slotCount(&self) -> usize {
        (self.fColumns as i32 * self.fRows as i32) as usize
    }
    // Source: src/sksl/ir/SkSLType.cpp:513-516
    fn slotType(&self, n: usize) -> &dyn Type {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLType.cpp\0".as_ptr(),514; @condition b"n < this->slotCount()\0".as_ptr(); n < self.slotCount());
        unsafe { self.fComponentType.as_ref() }
    }
}
// Source: src/sksl/ir/SkSLType.cpp:526-526
// Source: src/sksl/ir/SkSLType.cpp:567-571
#[repr(C)]
struct TextureType {
    base: TypeData,
    fDimensions: u32,
    fIsDepth: bool,
    fIsArrayed: bool,
    fIsMultisampled: bool,
    fTextureAccess: TextureAccess,
}
type_projections!(TextureType);
// Source: src/sksl/ir/SkSLType.cpp:528-528
impl TypeTag for TextureType {
    const kTypeKind: TypeKind = TypeKind::kTexture;
}
impl TextureType {
    // Source: src/sksl/ir/SkSLType.cpp:530-537
    unsafe fn new(
        name: &core::ffi::CStr,
        dimensions: u32,
        isDepth: bool,
        isArrayed: bool,
        isMultisampled: bool,
        textureAccess: TextureAccess,
    ) -> Self {
        Self {
            base: <TypeData as Type>::new_data(
                name.to_bytes(),
                c"T",
                Self::kTypeKind,
                Position::default(),
            ),
            fDimensions: dimensions,
            fIsDepth: isDepth,
            fIsArrayed: isArrayed,
            fIsMultisampled: isMultisampled,
            fTextureAccess: textureAccess,
        }
    }
}
unsafe impl Type for TextureType {
    type_base_projections!();
    // Source: src/sksl/ir/SkSLType.cpp:539-541
    fn dimensions(&self) -> u32 {
        self.fDimensions
    }
    // Source: src/sksl/ir/SkSLType.cpp:543-545
    fn isDepth(&self) -> bool {
        self.fIsDepth
    }
    // Source: src/sksl/ir/SkSLType.cpp:547-549
    fn isArrayedTexture(&self) -> bool {
        self.fIsArrayed
    }
    // Source: src/sksl/ir/SkSLType.cpp:551-553
    fn isMultisampled(&self) -> bool {
        self.fIsMultisampled
    }
    // Source: src/sksl/ir/SkSLType.cpp:555-557
    fn textureAccess(&self) -> TextureAccess {
        self.fTextureAccess
    }
    // Source: src/sksl/ir/SkSLType.cpp:559-562
    fn slotType(&self, n: usize) -> &dyn Type {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLType.cpp\0".as_ptr(),560; @condition b"n == 0\0".as_ptr(); n == 0);
        self
    }
}
// Source: src/sksl/ir/SkSLType.cpp:574-574
// Source: src/sksl/ir/SkSLType.cpp:619-619
#[repr(C)]
struct SamplerType {
    base: TypeData,
    fTextureType: NonNull<TextureType>,
}
type_projections!(SamplerType);
// Source: src/sksl/ir/SkSLType.cpp:576-576
impl TypeTag for SamplerType {
    const kTypeKind: TypeKind = TypeKind::kSampler;
}
impl SamplerType {
    // Source: src/sksl/ir/SkSLType.cpp:578-585
    unsafe fn new(name: &core::ffi::CStr, textureType: &dyn Type) -> Self {
        debug_assert!(textureType.typeKind() == TextureType::kTypeKind);
        let value = Self {
            base: <TypeData as Type>::new_data(
                name.to_bytes(),
                c"Z",
                Self::kTypeKind,
                Position::default(),
            ),
            fTextureType: NonNull::new_unchecked(
                textureType as *const dyn Type as *const () as *mut TextureType,
            ),
        };
        debug_assert!(value.textureAccess() == TextureAccess::kSample);
        debug_assert!(value.dimensions() != 6);
        value
    }
}
unsafe impl Type for SamplerType {
    type_base_projections!();
    // Source: src/sksl/ir/SkSLType.cpp:587-589
    fn textureType(&self) -> &dyn Type {
        unsafe { self.fTextureType.as_ref() }
    }
    // Source: src/sksl/ir/SkSLType.cpp:591-593
    fn dimensions(&self) -> u32 {
        unsafe { self.fTextureType.as_ref() }.dimensions()
    }
    // Source: src/sksl/ir/SkSLType.cpp:595-597
    fn isDepth(&self) -> bool {
        unsafe { self.fTextureType.as_ref() }.isDepth()
    }
    // Source: src/sksl/ir/SkSLType.cpp:599-601
    fn isArrayedTexture(&self) -> bool {
        unsafe { self.fTextureType.as_ref() }.isArrayedTexture()
    }
    // Source: src/sksl/ir/SkSLType.cpp:603-605
    fn isMultisampled(&self) -> bool {
        unsafe { self.fTextureType.as_ref() }.isMultisampled()
    }
    // Source: src/sksl/ir/SkSLType.cpp:607-609
    fn textureAccess(&self) -> TextureAccess {
        unsafe { self.fTextureType.as_ref() }.textureAccess()
    }
    // Source: src/sksl/ir/SkSLType.cpp:611-614
    fn slotType(&self, n: usize) -> &dyn Type {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLType.cpp\0".as_ptr(),612; @condition b"n == 0\0".as_ptr(); n == 0);
        self
    }
}
// Source: src/sksl/ir/SkSLType.cpp:622-622
// Source: src/sksl/ir/SkSLType.cpp:722-732
#[repr(C)]
struct StructType {
    base: TypeData,
    fFields: crate::include::private::SkTArray::TArray<Field, true>,
    fSlotCount: usize,
    fNestingDepth: i32,
    fUniformErrorPosition: Position,
    fInterfaceBlock: bool,
    fContainsArray: bool,
    fContainsUnsizedArray: bool,
    fContainsAtomic: bool,
    fContainsBool: bool,
    fIsBuiltin: bool,
    fIsAllowedInES2: bool,
}
type_projections!(StructType);
// Source: src/sksl/ir/SkSLType.cpp:624-624
impl TypeTag for StructType {
    const kTypeKind: TypeKind = TypeKind::kStruct;
}
impl StructType {
    // Source: src/sksl/ir/SkSLType.cpp:626-652
    unsafe fn new(
        pos: Position,
        name: &[u8],
        fields: crate::include::private::SkTArray::TArray<Field, true>,
        depth: i32,
        interfaceBlock: bool,
        builtin: bool,
    ) -> Self {
        let mut value = Self {
            base: <TypeData as Type>::new_data(name, c"S", Self::kTypeKind, pos),
            fFields: fields,
            fSlotCount: 0,
            fNestingDepth: depth,
            fUniformErrorPosition: Position::default(),
            fInterfaceBlock: interfaceBlock,
            fContainsArray: false,
            fContainsUnsizedArray: false,
            fContainsAtomic: false,
            fContainsBool: false,
            fIsBuiltin: builtin,
            fIsAllowedInES2: true,
        };
        for f in value.fFields.as_slice() {
            let ty = f.fType.expect("native field must have type").as_ref();
            value.fContainsArray = value.fContainsArray || ty.isOrContainsArray();
            value.fContainsUnsizedArray =
                value.fContainsUnsizedArray || ty.isOrContainsUnsizedArray();
            value.fContainsAtomic = value.fContainsAtomic || ty.isOrContainsAtomic();
            value.fContainsBool = value.fContainsBool || ty.isOrContainsBool();
            value.fIsAllowedInES2 = value.fIsAllowedInES2 && ty.isAllowedInES2();
        }
        for f in value.fFields.as_slice() {
            let mut pos = f.fPosition;
            if !f
                .fType
                .expect("native field must have type")
                .as_ref()
                .isAllowedInUniform(Some(&mut pos))
            {
                value.fUniformErrorPosition = pos;
                break;
            }
        }
        if !value.fContainsUnsizedArray {
            for f in value.fFields.as_slice() {
                value.fSlotCount = value.fSlotCount.wrapping_add(
                    f.fType
                        .expect("native field must have type")
                        .as_ref()
                        .slotCount(),
                );
            }
        }
        value
    }
}
unsafe impl Type for StructType {
    type_base_projections!();
    // Source: src/sksl/ir/SkSLType.cpp:654-656
    fn fields(&self) -> &[Field] {
        self.fFields.as_slice()
    }
    // Source: src/sksl/ir/SkSLType.cpp:658-660
    fn isStruct(&self) -> bool {
        true
    }
    // Source: src/sksl/ir/SkSLType.cpp:662-664
    fn isInterfaceBlock(&self) -> bool {
        self.fInterfaceBlock
    }
    // Source: src/sksl/ir/SkSLType.cpp:666-668
    fn isBuiltin(&self) -> bool {
        self.fIsBuiltin
    }
    // Source: src/sksl/ir/SkSLType.cpp:670-672
    fn isAllowedInES2(&self) -> bool {
        self.fIsAllowedInES2
    }
    // Source: src/sksl/ir/SkSLType.cpp:674-679
    fn isAllowedInUniform(&self, pos: Option<&mut Position>) -> bool {
        if let Some(p) = pos {
            *p = self.fUniformErrorPosition;
        }
        !self.fUniformErrorPosition.valid()
    }
    // Source: src/sksl/ir/SkSLType.cpp:681-683
    fn isOrContainsArray(&self) -> bool {
        self.fContainsArray
    }
    // Source: src/sksl/ir/SkSLType.cpp:685-687
    fn isOrContainsUnsizedArray(&self) -> bool {
        self.fContainsUnsizedArray
    }
    // Source: src/sksl/ir/SkSLType.cpp:689-691
    fn isOrContainsAtomic(&self) -> bool {
        self.fContainsAtomic
    }
    // Source: src/sksl/ir/SkSLType.cpp:693-695
    fn isOrContainsBool(&self) -> bool {
        self.fContainsBool
    }
    // Source: src/sksl/ir/SkSLType.cpp:697-700
    fn slotCount(&self) -> usize {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLType.cpp\0".as_ptr(),698; @condition b"!fContainsUnsizedArray\0".as_ptr(); !self.fContainsUnsizedArray);
        self.fSlotCount
    }
    // Source: src/sksl/ir/SkSLType.cpp:702-704
    fn structNestingDepth(&self) -> i32 {
        self.fNestingDepth
    }
    // Source: src/sksl/ir/SkSLType.cpp:706-717
    fn slotType(&self, mut n: usize) -> &dyn Type {
        for f in self.fFields.as_slice() {
            let ty = unsafe { f.fType.expect("native field must have type").as_ref() };
            let slots = ty.slotCount();
            if n < slots {
                return ty.slotType(n);
            } else {
                n -= slots;
            }
        }
        native_type_debugfail("slot index out of range");
        self
    }
}
// Source: src/sksl/ir/SkSLType.cpp:735-735
// Source: src/sksl/ir/SkSLType.cpp:791-791
// Source: src/sksl/ir/SkSLType.cpp:792-792
#[repr(C)]
struct VectorType {
    base: TypeData,
    fComponentType: NonNull<ScalarType>,
    fColumns: i8,
}
type_projections!(VectorType);
// Source: src/sksl/ir/SkSLType.cpp:737-737
impl TypeTag for VectorType {
    const kTypeKind: TypeKind = TypeKind::kVector;
}
impl VectorType {
    // Source: src/sksl/ir/SkSLType.cpp:739-745
    unsafe fn new(
        name: &[u8],
        abbrev: &core::ffi::CStr,
        componentType: &dyn Type,
        columns: i8,
    ) -> Self {
        let value = Self {
            base: <Self as Type>::new_data(name, abbrev, Self::kTypeKind, Position::default()),
            fComponentType: NonNull::from(componentType.as_type_node::<ScalarType>()),
            fColumns: columns,
        };
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLType.cpp\0".as_ptr(),744; @condition b"columns >= 2 && columns <= 4\0".as_ptr(); columns >= 2 && columns <= 4);
        value
    }
}
unsafe impl Type for VectorType {
    type_base_projections!();
    // Source: src/sksl/ir/SkSLType.cpp:747-749
    fn componentType(&self) -> &dyn Type {
        unsafe { self.fComponentType.as_ref() }
    }
    // Source: src/sksl/ir/SkSLType.cpp:751-753
    fn columns(&self) -> i32 {
        self.fColumns as i32
    }
    // Source: src/sksl/ir/SkSLType.cpp:755-757
    fn rows(&self) -> i32 {
        1
    }
    // Source: src/sksl/ir/SkSLType.cpp:759-761
    fn bitWidth(&self) -> i32 {
        self.componentType().bitWidth()
    }
    // Source: src/sksl/ir/SkSLType.cpp:763-765
    fn isVector(&self) -> bool {
        true
    }
    // Source: src/sksl/ir/SkSLType.cpp:767-769
    fn isAllowedInES2(&self) -> bool {
        unsafe { self.fComponentType.as_ref() }.isAllowedInES2()
    }
    // Source: src/sksl/ir/SkSLType.cpp:771-773
    fn isAllowedInUniform(&self, errorPosition: Option<&mut Position>) -> bool {
        unsafe { self.fComponentType.as_ref() }.isAllowedInUniform(errorPosition)
    }
    // Source: src/sksl/ir/SkSLType.cpp:775-777
    fn isOrContainsBool(&self) -> bool {
        unsafe { self.fComponentType.as_ref() }.isOrContainsBool()
    }
    // Source: src/sksl/ir/SkSLType.cpp:779-781
    fn slotCount(&self) -> usize {
        self.fColumns as usize
    }
    // Source: src/sksl/ir/SkSLType.cpp:783-786
    fn slotType(&self, n: usize) -> &dyn Type {
        debug_assert!(n < self.fColumns as usize);
        unsafe { self.fComponentType.as_ref() }
    }
}
// Language-only borrowed string_view key representation for the native THashSet.
#[derive(Clone, Copy)]
struct FieldName(NonNull<[u8]>);
impl PartialEq for FieldName {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.0.as_ref() == other.0.as_ref() }
    }
}
#[derive(Default)]
struct FieldNameHash;
impl crate::src::core::SkTHash::NativeHash<FieldName> for FieldNameHash {
    fn hash(&self, value: &FieldName) -> u32 {
        crate::src::core::SkChecksum::Hash32(unsafe { value.0.as_ref() }, 0)
    }
}
// Source: src/sksl/ir/SkSLType.cpp:795-801
pub fn getArrayName(this: &dyn Type, arraySize: i32) -> Vec<u8> {
    let name = this.name();
    unsafe {
        if arraySize == kUnsizedArray {
            crate::src::sksl::SkSLString::printf(
                c"%.*s[]".as_ptr(),
                name.len() as i32,
                name.as_ptr(),
            )
        } else {
            crate::src::sksl::SkSLString::printf(
                c"%.*s[%d]".as_ptr(),
                name.len() as i32,
                name.as_ptr(),
                arraySize,
            )
        }
    }
}
impl dyn Type {
    // Source: src/sksl/ir/SkSLType.cpp:803-805
    pub unsafe fn MakeAliasType(name: &[u8], target: &dyn Type) -> IRNodeBox<dyn Type> {
        IRNodeBox::new_with(|p: *mut AliasType| p.write(AliasType::new(name, target)))
            .cast(|p| p as *mut dyn Type)
    }
    // Source: src/sksl/ir/SkSLType.cpp:807-813
    pub unsafe fn MakeArrayType(
        context: &crate::src::sksl::SkSLContext::Context,
        name: &[u8],
        component: &dyn Type,
        columns: i32,
    ) -> IRNodeBox<dyn Type> {
        IRNodeBox::new_with(|p: *mut ArrayType| {
            p.write(ArrayType::new(
                name,
                component.abbreviatedName(),
                component,
                columns,
                context
                    .fConfig
                    .expect("native context config")
                    .as_ref()
                    .isBuiltinCode(),
            ))
        })
        .cast(|p| p as *mut dyn Type)
    }
    // Source: src/sksl/ir/SkSLType.cpp:815-819
    /// Safety: names and all borrowed Types/slots outlive the returned owner;
    /// concrete Type invariants hold, and its pool stays valid and attached at drop.
    pub unsafe fn MakeGenericType(
        name: &core::ffi::CStr,
        types: &[NonNull<dyn Type>],
        slotType: Option<NonNull<dyn Type>>,
    ) -> IRNodeBox<dyn Type> {
        IRNodeBox::new_with(|p: *mut GenericType| p.write(GenericType::new(name, types, slotType)))
            .cast(|p| p as *mut dyn Type)
    }
    // Source: src/sksl/ir/SkSLType.cpp:821-824
    /// Safety: names and all borrowed Types/slots outlive the returned owner;
    /// concrete Type invariants hold, and its pool stays valid and attached at drop.
    pub unsafe fn MakeLiteralType(
        name: &core::ffi::CStr,
        scalarType: &dyn Type,
        priority: i8,
    ) -> IRNodeBox<dyn Type> {
        IRNodeBox::new_with(|p: *mut LiteralType| {
            p.write(LiteralType::new(name, borrow_type(scalarType), priority))
        })
        .cast(|p| p as *mut dyn Type)
    }
    // Source: src/sksl/ir/SkSLType.cpp:826-829
    /// Safety: names and all borrowed Types/slots outlive the returned owner;
    /// concrete Type invariants hold, and its pool stays valid and attached at drop.
    pub unsafe fn MakeMatrixType(
        name: &[u8],
        abbrev: &core::ffi::CStr,
        componentType: &dyn Type,
        columns: i32,
        rows: i8,
    ) -> IRNodeBox<dyn Type> {
        IRNodeBox::new_with(|p: *mut MatrixType| {
            p.write(MatrixType::new(
                name,
                abbrev,
                componentType,
                columns as i8,
                rows,
            ))
        })
        .cast(|p| p as *mut dyn Type)
    }
    // Source: src/sksl/ir/SkSLType.cpp:831-833
    pub unsafe fn MakeSamplerType(
        name: &core::ffi::CStr,
        texture: &dyn Type,
    ) -> IRNodeBox<dyn Type> {
        IRNodeBox::new_with(|p: *mut SamplerType| p.write(SamplerType::new(name, texture)))
            .cast(|p| p as *mut dyn Type)
    }
    // Source: src/sksl/ir/SkSLType.cpp:835-838
    /// Safety: names and all borrowed Types/slots outlive the returned owner;
    /// concrete Type invariants hold, and its pool stays valid and attached at drop.
    pub unsafe fn MakeSpecialType(
        name: &core::ffi::CStr,
        abbrev: &core::ffi::CStr,
        typeKind: TypeKind,
    ) -> IRNodeBox<dyn Type> {
        IRNodeBox::new_with(|p: *mut TypeData| {
            p.write(<TypeData as Type>::new_data(
                name.to_bytes(),
                abbrev,
                typeKind,
                Position::default(),
            ))
        })
        .cast(|p| p as *mut dyn Type)
    }
    // Source: src/sksl/ir/SkSLType.cpp:840-844
    /// Safety: names and all borrowed Types/slots outlive the returned owner;
    /// concrete Type invariants hold, and its pool stays valid and attached at drop.
    pub unsafe fn MakeScalarType(
        name: &[u8],
        abbrev: &core::ffi::CStr,
        numberKind: NumberKind,
        priority: i8,
        bitWidth: i8,
    ) -> IRNodeBox<dyn Type> {
        IRNodeBox::new_with(|p: *mut ScalarType| {
            p.write(ScalarType::new(
                name, abbrev, numberKind, priority, bitWidth,
            ))
        })
        .cast(|p| p as *mut dyn Type)
    }
    // Source: src/sksl/ir/SkSLType.cpp:846-848
    pub unsafe fn MakeAtomicType(name: &[u8], abbrev: &core::ffi::CStr) -> IRNodeBox<dyn Type> {
        IRNodeBox::new_with(|p: *mut AtomicType| p.write(AtomicType::new(name, abbrev)))
            .cast(|p| p as *mut dyn Type)
    }
    // Source: src/sksl/ir/SkSLType.cpp:850-929
    pub unsafe fn MakeStructType(
        context: &crate::src::sksl::SkSLContext::Context,
        pos: Position,
        name: &[u8],
        fields: crate::include::private::SkTArray::TArray<Field, true>,
        interfaceBlock: bool,
    ) -> IRNodeBox<dyn Type> {
        use super::SkSLLayout::LayoutFlag;
        use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
        let kind = if interfaceBlock {
            b"interface block".as_slice()
        } else {
            b"struct".as_slice()
        };
        let article = if interfaceBlock {
            b"an interface block".as_slice()
        } else {
            b"a struct".as_slice()
        };
        let mut errors = context.fErrors;
        if fields.empty() {
            let mut message = kind.to_vec();
            message.extend_from_slice(b" '");
            message.extend_from_slice(name);
            message.extend_from_slice(b"' must contain at least one field");
            errors.as_mut().error(pos, &message);
        }
        let mut slots = 0usize;
        let mut names = crate::src::core::SkTHash::THashSet::<FieldName, FieldNameHash>::default();
        for field in fields.as_slice() {
            let n = names.count();
            names.add(FieldName(field.fName));
            if names.count() == n {
                let mut message = b"field '".to_vec();
                message.extend_from_slice(field.fName.as_ref());
                message.extend_from_slice(b"' was already defined in the same ");
                message.extend_from_slice(kind);
                message.extend_from_slice(b" ('");
                message.extend_from_slice(name);
                message.extend_from_slice(b"')");
                errors.as_mut().error(field.fPosition, &message);
            }
            if field.fModifierFlags.0 != 0 {
                let desc = field.fModifierFlags.description();
                let mut message = b"modifier '".to_vec();
                message.extend_from_slice(&desc);
                message.extend_from_slice(b"' is not permitted on ");
                message.extend_from_slice(article);
                message.extend_from_slice(b" field");
                errors.as_mut().error(field.fPosition, &message);
            }
            if field.fLayout.fFlags.0 & LayoutFlag::kBinding.0 != 0 {
                let mut message = b"layout qualifier 'binding' is not permitted on ".to_vec();
                message.extend_from_slice(article);
                message.extend_from_slice(b" field");
                errors.as_mut().error(field.fPosition, &message);
            }
            if field.fLayout.fFlags.0 & LayoutFlag::kSet.0 != 0 {
                let mut message = b"layout qualifier 'set' is not permitted on ".to_vec();
                message.extend_from_slice(article);
                message.extend_from_slice(b" field");
                errors.as_mut().error(field.fPosition, &message);
            }
            let ty = field.fType.unwrap_unchecked().as_ref();
            if ty.isVoid() {
                let mut message = b"type 'void' is not permitted in ".to_vec();
                message.extend_from_slice(article);
                errors.as_mut().error(field.fPosition, &message);
            }
            if ty.isOpaque() && !ty.isAtomic() {
                let mut message = b"opaque type '".to_vec();
                message.extend_from_slice(&ty.displayName());
                message.extend_from_slice(b"' is not permitted in ");
                message.extend_from_slice(article);
                errors.as_mut().error(field.fPosition, &message);
            }
            if interfaceBlock && ty.isOrContainsBool() {
                errors.as_mut().error(
                    field.fPosition,
                    b"type 'bool' is not permitted in an interface block",
                );
            }
            if ty.isOrContainsUnsizedArray() {
                if !interfaceBlock {
                    errors
                        .as_mut()
                        .error(field.fPosition, b"unsized arrays are not permitted here");
                }
            } else {
                if slots < crate::src::sksl::SkSLDefines::kVariableSlotLimit as usize {
                    slots = slots.checked_add(ty.slotCount()).unwrap_or(usize::MAX);
                    if slots >= crate::src::sksl::SkSLDefines::kVariableSlotLimit as usize {
                        let mut message = kind.to_vec();
                        message.extend_from_slice(b" is too large");
                        errors.as_mut().error(pos, &message);
                    }
                }
            }
        }
        let mut depth = 0;
        for field in fields.as_slice() {
            depth = depth.max(field.fType.unwrap_unchecked().as_ref().structNestingDepth());
        }
        if depth >= kMaxStructDepth {
            let mut message = kind.to_vec();
            message.extend_from_slice(b" '");
            message.extend_from_slice(name);
            message.extend_from_slice(b"' is too deeply nested");
            errors.as_mut().error(pos, &message);
        }
        IRNodeBox::new_with(|p: *mut StructType| {
            p.write(StructType::new(
                pos,
                name,
                fields,
                depth + 1,
                interfaceBlock,
                context.fConfig.unwrap_unchecked().as_ref().isBuiltinCode(),
            ))
        })
        .cast(|p| p as *mut dyn Type)
    }
    // Source: src/sksl/ir/SkSLType.cpp:931-936
    pub unsafe fn MakeTextureType(
        name: &core::ffi::CStr,
        dimensions: u32,
        isDepth: bool,
        isArrayed: bool,
        isMultisampled: bool,
        access: TextureAccess,
    ) -> IRNodeBox<dyn Type> {
        IRNodeBox::new_with(|p: *mut TextureType| {
            p.write(TextureType::new(
                name,
                dimensions,
                isDepth,
                isArrayed,
                isMultisampled,
                access,
            ))
        })
        .cast(|p| p as *mut dyn Type)
    }
    // Source: src/sksl/ir/SkSLType.cpp:938-941
    /// Safety: names and all borrowed Types/slots outlive the returned owner;
    /// concrete Type invariants hold, and its pool stays valid and attached at drop.
    pub unsafe fn MakeVectorType(
        name: &[u8],
        abbrev: &core::ffi::CStr,
        componentType: &dyn Type,
        columns: i32,
    ) -> IRNodeBox<dyn Type> {
        IRNodeBox::new_with(|p: *mut VectorType| {
            p.write(VectorType::new(name, abbrev, componentType, columns as i8))
        })
        .cast(|p| p as *mut dyn Type)
    }
}
// Source: src/sksl/ir/SkSLType.cpp:943-986
pub fn coercionCost(this: &dyn Type, other: &dyn Type) -> CoercionCost {
    if this.matches(other) {
        return CoercionCost::Free();
    }
    if this.typeKind() == other.typeKind() && (this.isVector() || this.isMatrix() || this.isArray())
    {
        if this.isMatrix() && this.rows() != other.rows() {
            return CoercionCost::Impossible();
        }
        if this.columns() != other.columns() {
            return CoercionCost::Impossible();
        }
        return this.componentType().coercionCost(other.componentType());
    }
    if this.isNumber() && other.isNumber() {
        if this.isLiteral() && (this.isInteger() || this.numberKind() == other.numberKind()) {
            return CoercionCost::Free();
        } else if this.numberKind() != other.numberKind() {
            return CoercionCost::Impossible();
        } else if other.priority() >= this.priority() {
            return CoercionCost::Normal(other.priority() - this.priority());
        } else {
            return CoercionCost::Narrowing(this.priority() - other.priority());
        }
    }
    if this.type_data().fTypeKind == TypeKind::kGeneric {
        let types = this.coercibleTypes();
        for (i, t) in types.iter().enumerate() {
            if unsafe { t.as_ref() }.matches(other) {
                return CoercionCost::Normal(i as i32 + 1);
            }
        }
    }
    CoercionCost::Impossible()
}
// Rust lifetime erasure for native borrowed Type pointers; the unsafe factories
// require the concrete allocation to outlive every borrower, including Drop.
unsafe fn borrow_type(t: &dyn Type) -> NonNull<dyn Type> {
    core::mem::transmute::<NonNull<dyn Type + '_>, NonNull<dyn Type>>(NonNull::from(t))
}

// Source: src/sksl/ir/SkSLType.cpp:988-995
pub unsafe fn applyQualifiers<'a>(
    this: &'a dyn Type,
    context: &'a crate::src::sksl::SkSLContext::Context,
    flags: &mut ModifierFlags,
    pos: Position,
) -> &'a dyn Type {
    let ty = applyPrecisionQualifiers(this, context, flags, pos);
    applyAccessQualifiers(ty, context, flags, pos)
}
// Source: src/sksl/ir/SkSLType.cpp:997-1064
pub unsafe fn applyPrecisionQualifiers<'a>(
    this: &'a dyn Type,
    context: &'a crate::src::sksl::SkSLContext::Context,
    modifierFlags: &mut ModifierFlags,
    pos: Position,
) -> &'a dyn Type {
    use super::SkSLModifierFlags::ModifierFlag;
    use crate::src::sksl::{
        SkSLErrorReporter::ErrorReporterExt, SkSLProgramSettings::ProgramConfig,
    };
    let mask = ModifierFlag::kHighp.0 | ModifierFlag::kMediump.0 | ModifierFlag::kLowp.0;
    let precision = modifierFlags.0 & mask;
    if precision == 0 {
        return this;
    }
    let types = context.fTypes.as_ref();
    let mut errors = context.fErrors;
    if !ProgramConfig::IsRuntimeEffect(context.fConfig.unwrap_unchecked().as_ref().fKind) {
        errors
            .as_mut()
            .error(pos, b"precision qualifiers are not allowed");
        return &**types.fPoison;
    }
    if precision.count_ones() > 1 {
        errors
            .as_mut()
            .error(pos, b"only one precision qualifier can be used");
        return &**types.fPoison;
    }
    modifierFlags.0 &= !mask;
    let component = this.componentType();
    if component.highPrecision() {
        if precision & ModifierFlag::kHighp.0 != 0 {
            return this;
        }
        let mediump: &dyn Type = match component.numberKind() {
            NumberKind::kFloat => &**types.fHalf,
            NumberKind::kSigned => &**types.fShort,
            NumberKind::kUnsigned => &**types.fUShort,
            _ => &**types.fPoison,
        };
        return if this.isArray() {
            context
                .fSymbolTable
                .unwrap_unchecked()
                .as_mut()
                .addArrayDimension(context, borrow_type(mediump), this.columns())
                .as_ref()
        } else {
            mediump.toCompound(context, this.columns(), this.rows())
        };
    }
    let mut message = b"type '".to_vec();
    message.extend_from_slice(&this.displayName());
    message.extend_from_slice(b"' does not support precision qualifiers");
    errors.as_mut().error(pos, &message);
    &**types.fPoison
}
// Source: src/sksl/ir/SkSLType.cpp:1066-1099
pub unsafe fn applyAccessQualifiers<'a>(
    this: &'a dyn Type,
    context: &'a crate::src::sksl::SkSLContext::Context,
    flags: &mut ModifierFlags,
    pos: Position,
) -> &'a dyn Type {
    use super::SkSLModifierFlags::ModifierFlag as F;
    use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
    let access = flags.0 & (F::kReadOnly.0 | F::kWriteOnly.0);
    flags.0 &= !(F::kReadOnly.0 | F::kWriteOnly.0);
    let types = context.fTypes.as_ref();
    let mut errors = context.fErrors;
    if this.matches(&**types.fTexture2D) {
        if access == F::kReadOnly.0 {
            return &**types.fReadOnlyTexture2D;
        }
        if access == F::kWriteOnly.0 {
            return &**types.fWriteOnlyTexture2D;
        }
        errors.as_mut().error(
            pos,
            if access != 0 {
                b"'readonly' and 'writeonly' qualifiers cannot be combined"
            } else {
                b"'texture2D' requires a 'readonly' or 'writeonly' access qualifier"
            },
        );
        return this;
    }
    if access != 0 {
        let mut message = b"type '".to_vec();
        message.extend_from_slice(&this.displayName());
        message.extend_from_slice(b"' does not support qualifier '");
        message.extend_from_slice(&ModifierFlags(access).description());
        message.push(b'\'');
        errors.as_mut().error(pos, &message);
    }
    this
}
// Source: src/sksl/ir/SkSLType.cpp:1101-1235
pub unsafe fn toCompound<'a>(
    this: &'a dyn Type,
    context: &'a crate::src::sksl::SkSLContext::Context,
    columns: i32,
    rows: i32,
) -> &'a dyn Type {
    debug_assert!(this.isScalar());
    if columns == 1 && rows == 1 {
        return this;
    }
    let types = context.fTypes.as_ref();
    if this.matches(&**types.fFloat) || this.matches(&**types.fFloatLiteral) {
        match rows {
            1 => match columns {
                1 => &**types.fFloat,
                2 => &**types.fFloat2,
                3 => &**types.fFloat3,
                4 => &**types.fFloat4,
                _ => std::process::abort(),
            },
            2 => match columns {
                2 => &**types.fFloat2x2,
                3 => &**types.fFloat3x2,
                4 => &**types.fFloat4x2,
                _ => std::process::abort(),
            },
            3 => match columns {
                2 => &**types.fFloat2x3,
                3 => &**types.fFloat3x3,
                4 => &**types.fFloat4x3,
                _ => std::process::abort(),
            },
            4 => match columns {
                2 => &**types.fFloat2x4,
                3 => &**types.fFloat3x4,
                4 => &**types.fFloat4x4,
                _ => std::process::abort(),
            },
            _ => std::process::abort(),
        }
    } else if this.matches(&**types.fHalf) {
        match rows {
            1 => match columns {
                1 => &**types.fHalf,
                2 => &**types.fHalf2,
                3 => &**types.fHalf3,
                4 => &**types.fHalf4,
                _ => std::process::abort(),
            },
            2 => match columns {
                2 => &**types.fHalf2x2,
                3 => &**types.fHalf3x2,
                4 => &**types.fHalf4x2,
                _ => std::process::abort(),
            },
            3 => match columns {
                2 => &**types.fHalf2x3,
                3 => &**types.fHalf3x3,
                4 => &**types.fHalf4x3,
                _ => std::process::abort(),
            },
            4 => match columns {
                2 => &**types.fHalf2x4,
                3 => &**types.fHalf3x4,
                4 => &**types.fHalf4x4,
                _ => std::process::abort(),
            },
            _ => std::process::abort(),
        }
    } else if this.matches(&**types.fInt) || this.matches(&**types.fIntLiteral) {
        match rows {
            1 => match columns {
                1 => &**types.fInt,
                2 => &**types.fInt2,
                3 => &**types.fInt3,
                4 => &**types.fInt4,
                _ => std::process::abort(),
            },
            _ => std::process::abort(),
        }
    } else if this.matches(&**types.fShort) {
        match rows {
            1 => match columns {
                1 => &**types.fShort,
                2 => &**types.fShort2,
                3 => &**types.fShort3,
                4 => &**types.fShort4,
                _ => std::process::abort(),
            },
            _ => std::process::abort(),
        }
    } else if this.matches(&**types.fUInt) {
        match rows {
            1 => match columns {
                1 => &**types.fUInt,
                2 => &**types.fUInt2,
                3 => &**types.fUInt3,
                4 => &**types.fUInt4,
                _ => std::process::abort(),
            },
            _ => std::process::abort(),
        }
    } else if this.matches(&**types.fUShort) {
        match rows {
            1 => match columns {
                1 => &**types.fUShort,
                2 => &**types.fUShort2,
                3 => &**types.fUShort3,
                4 => &**types.fUShort4,
                _ => std::process::abort(),
            },
            _ => std::process::abort(),
        }
    } else if this.matches(&**types.fBool) {
        match rows {
            1 => match columns {
                1 => &**types.fBool,
                2 => &**types.fBool2,
                3 => &**types.fBool3,
                4 => &**types.fBool4,
                _ => std::process::abort(),
            },
            _ => std::process::abort(),
        }
    } else {
        native_type_debugfail("unsupported toCompound type");
        &**types.fVoid
    }
}
// Source: src/sksl/ir/SkSLType.cpp:1237-1278
pub unsafe fn cloneType(
    this: &dyn Type,
    context: &crate::src::sksl::SkSLContext::Context,
    symbolTable: &mut crate::src::sksl::ir::SkSLSymbolTable::SymbolTable,
) -> Option<NonNull<dyn Type>> {
    if this.isInRootSymbolTable() {
        return Some(borrow_type(this));
    }
    if !context.fConfig.unwrap_unchecked().as_ref().isBuiltinCode() && this.isBuiltin() {
        return Some(borrow_type(this));
    }
    if let Some(existing) = symbolTable.find(this.name()) {
        debug_assert!(existing.as_ref().node().is::<TypeData>());
        let ty = existing.as_ref().as_type_symbol();
        debug_assert_eq!(ty.typeKind(), this.typeKind());
        return Some(borrow_type(ty));
    }
    match this.typeKind() {
        TypeKind::kArray => Some(symbolTable.addArrayDimension(
            context,
            borrow_type(this.componentType()),
            this.columns(),
        )),
        TypeKind::kStruct => {
            let name = symbolTable.takeOwnershipOfString(this.name().to_vec());
            let fields = crate::include::private::SkTArray::TArray::from_slice(this.fields());
            let owner = IRNodeBox::new_with(|p: *mut StructType| {
                p.write(StructType::new(
                    this.node().fPosition,
                    name.as_ref(),
                    fields,
                    this.structNestingDepth(),
                    this.isInterfaceBlock(),
                    context.fConfig.unwrap_unchecked().as_ref().isBuiltinCode(),
                ))
            });
            let result = NonNull::new_unchecked(owner.as_ptr() as *mut StructType as *mut dyn Type);
            symbolTable.add(context, owner.cast(|p| p as *mut dyn Symbol));
            Some(result)
        }
        _ => {
            native_type_debugfail("don't know how to clone type");
            None
        }
    }
}
// Unverified by explicit user instruction. All three native cast branches below
// call their actual IR owners; no scalar-only replacement is used.
// Source: src/sksl/ir/SkSLType.cpp:1280-1308
pub unsafe fn coerceExpression(
    this: &dyn Type,
    expr: Option<IRNodeBox<dyn super::SkSLExpression::Expression>>,
    context: &crate::src::sksl::SkSLContext::Context,
) -> Option<IRNodeBox<dyn super::SkSLExpression::Expression>> {
    use super::SkSLExpression::ExpressionExt;
    use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
    let expr = expr?;
    if expr.isIncomplete(context) {
        return None;
    }
    if expr.expression_type().matches(this) {
        return Some(expr);
    }
    let pos = expr.node().fPosition;
    let settings = &context.fConfig.unwrap_unchecked().as_ref().fSettings;
    if !expr
        .coercionCost(this)
        .isPossible(settings.fAllowNarrowingConversions)
    {
        let mut message = b"expected '".to_vec();
        message.extend_from_slice(&this.displayName());
        message.extend_from_slice(b"', but found '");
        message.extend_from_slice(&expr.expression_type().displayName());
        message.push(b'\'');
        let mut errors = context.fErrors;
        errors.as_mut().error(pos, &message);
        return None;
    }
    if this.isScalar() {
        return Some(
            super::SkSLConstructorScalarCast::ConstructorScalarCast::Make(context, pos, this, expr),
        );
    }
    if this.isVector() || this.isMatrix() {
        return Some(
            super::SkSLConstructorCompoundCast::ConstructorCompoundCast::Make(
                context, pos, this, expr,
            ),
        );
    }
    if this.isArray() {
        return Some(super::SkSLConstructorArrayCast::ConstructorArrayCast::Make(
            context, pos, this, expr,
        ));
    }
    let mut message = b"cannot construct '".to_vec();
    message.extend_from_slice(&this.displayName());
    message.push(b'\'');
    let mut errors = context.fErrors;
    errors.as_mut().error(pos, &message);
    None
}
// Source: src/sksl/ir/SkSLType.cpp:1310-1312
pub unsafe fn isAllowedInES2Context(
    this: &dyn Type,
    context: &crate::src::sksl::SkSLContext::Context,
) -> bool {
    !context.fConfig.unwrap_unchecked().as_ref().strictES2Mode() || this.isAllowedInES2()
}
// Source: src/sksl/ir/SkSLType.cpp:1314-1337
pub unsafe fn checkForOutOfRangeLiteralExpr(
    this: &dyn Type,
    context: &crate::src::sksl::SkSLContext::Context,
    expr: &dyn super::SkSLExpression::Expression,
) -> bool {
    use super::SkSLExpression::ExpressionExt;
    let mut foundError = false;
    let baseType = this.componentType();
    if baseType.isNumber() {
        let valueExpr =
            crate::src::sksl::SkSLConstantFolder::ConstantFolder::GetConstantValueForVariable(expr);
        if valueExpr.supportsConstantValues() && !valueExpr.expression_type().isUnsizedArray() {
            let numSlots = valueExpr.expression_type().slotCount() as i32;
            for slot in 0..numSlots {
                let slotVal = valueExpr.getConstantValue(slot);
                if let Some(slotVal) = slotVal {
                    if checkForOutOfRangeLiteralValue(
                        baseType,
                        context,
                        slotVal,
                        valueExpr.node().fPosition,
                    ) {
                        foundError = true;
                    }
                }
            }
        }
    }
    foundError
}
// Source: src/sksl/ir/SkSLType.cpp:1339-1352
pub unsafe fn checkForOutOfRangeLiteralValue(
    this: &dyn Type,
    context: &crate::src::sksl::SkSLContext::Context,
    value: f64,
    pos: Position,
) -> bool {
    use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
    debug_assert!(this.isScalar());
    if !this.isNumber() {
        return false;
    }
    if value >= this.minimumValue() && value <= this.maximumValue() {
        return false;
    }
    let message = format_range_error(&this.displayName(), value);
    let mut errors = context.fErrors;
    errors.as_mut().error(pos, &message);
    true
}
// Source: src/sksl/ir/SkSLType.cpp:1348-1350
// Native temporary displayName().c_str() remains alive through the full call.
unsafe fn format_range_error(name: &[u8], value: f64) -> Vec<u8> {
    let mut terminated_name = name.to_vec();
    terminated_name.push(0);
    crate::src::sksl::SkSLString::printf(
        c"value is out of range for type '%s': %.0f".as_ptr(),
        terminated_name.as_ptr(),
        value,
    )
}
// Source: src/sksl/ir/SkSLType.cpp:1354-1369
pub unsafe fn checkIfUsableInArray(
    this: &dyn Type,
    context: &crate::src::sksl::SkSLContext::Context,
    arrayPos: Position,
) -> bool {
    use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
    let mut errors = context.fErrors;
    if this.isArray() {
        errors
            .as_mut()
            .error(arrayPos, b"multi-dimensional arrays are not supported");
        return false;
    }
    if this.isVoid() {
        errors
            .as_mut()
            .error(arrayPos, b"type 'void' may not be used in an array");
        return false;
    }
    if this.isOpaque() && !this.isAtomic() {
        let mut message = b"opaque type '".to_vec();
        message.extend_from_slice(this.name());
        message.extend_from_slice(b"' may not be used in an array");
        errors.as_mut().error(arrayPos, &message);
        return false;
    }
    true
}
// UNVERIFIED new translation.
// Source: src/sksl/ir/SkSLType.cpp:1371-1384
pub unsafe fn convertArraySizeExpression(
    this: &dyn Type,
    context: &crate::src::sksl::SkSLContext::Context,
    arrayPos: Position,
    size: Option<IRNodeBox<dyn super::SkSLExpression::Expression>>,
) -> i64 {
    use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
    let Some(size) = coerceExpression(&**context.fTypes.as_ref().fInt, size, context) else {
        return 0;
    };
    let mut count = core::mem::MaybeUninit::<i64>::uninit();
    if !crate::src::sksl::SkSLConstantFolder::ConstantFolder::GetConstantInt(
        &*size,
        count.as_mut_ptr(),
    ) {
        let mut errors = context.fErrors;
        errors
            .as_mut()
            .error(size.node().fPosition, b"array size must be an integer");
        return 0;
    }
    convertArraySizeValue(
        this,
        context,
        arrayPos,
        size.node().fPosition,
        count.assume_init(),
    )
}
// Source: src/sksl/ir/SkSLType.cpp:1386-1410
// Adapted SkSafeMath::Mul saturating library operation uses Rust checked multiplication.
pub unsafe fn convertArraySizeValue(
    this: &dyn Type,
    context: &crate::src::sksl::SkSLContext::Context,
    arrayPos: Position,
    sizePos: Position,
    size: i64,
) -> i64 {
    use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
    if !checkIfUsableInArray(this, context, arrayPos) {
        return 0;
    }
    let mut errors = context.fErrors;
    if size <= 0 {
        errors
            .as_mut()
            .error(sizePos, b"array size must be positive");
        return 0;
    }
    if !this.isOrContainsUnsizedArray() {
        if this
            .slotCount()
            .checked_mul(size as usize)
            .unwrap_or(usize::MAX)
            > crate::src::sksl::SkSLDefines::kVariableSlotLimit as usize
        {
            errors.as_mut().error(sizePos, b"array size is too large");
            return 0;
        }
    }
    size
}
impl Field {
    // Source: src/sksl/ir/SkSLType.cpp:1410-1413
    pub fn description(&self) -> Vec<u8> {
        let mut result = self.fLayout.paddedDescription();
        result.extend_from_slice(&self.fModifierFlags.paddedDescription());
        result.extend_from_slice(&unsafe { self.fType.unwrap_unchecked().as_ref() }.displayName());
        result.push(b' ');
        result.extend_from_slice(unsafe { self.fName.as_ref() });
        result.push(b';');
        result
    }
}
