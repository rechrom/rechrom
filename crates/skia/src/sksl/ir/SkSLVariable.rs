// v58 additions are UNVERIFIED; user prohibited all validation.
use super::{
    SkSLIRNode::IRNodeData,
    SkSLLayout::Layout,
    SkSLModifierFlags::ModifierFlags,
    SkSLSymbol::{Symbol, SymbolData},
    SkSLType::Type,
};
use crate::src::sksl::SkSLPosition::Position;
use core::{pin::Pin, ptr::NonNull};
// Source: src/sksl/ir/SkSLVariable.h:36-41
#[repr(i8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariableStorage {
    kGlobal,
    kInterfaceBlock,
    kLocal,
    kParameter,
}
// Source: src/sksl/ir/SkSLVariable.h:48-48
// Source: src/sksl/ir/SkSLVariable.h:140-144
#[repr(C)]
pub struct VariableData {
    pub(crate) base: SymbolData,
    pub(crate) fDeclaringElement: Option<NonNull<IRNodeData>>,
    pub(crate) fModifierFlags: ModifierFlags,
    pub(crate) fModifiersPosition: Position,
    pub(crate) fStorage: VariableStorage,
    pub(crate) fBuiltin: bool,
}
// Source: src/sksl/ir/SkSLVariable.h:50-50
pub type Storage = VariableStorage;
// Source: src/sksl/ir/SkSLVariable.h:52-52
impl super::SkSLIRNode::IRNodeType for VariableData {
    const kIRNodeKind: i32 = super::SkSLIRNode::SymbolKind::kVariable as i32;
}
impl VariableData {
    // Source: src/sksl/ir/SkSLVariable.h:54-60
    /// Safety: original Symbol leases hold. Concrete Variable must implement the native
    /// cpp destructor before it may be owned; this base data supplies no replacement destructor.
    pub unsafe fn new(
        pos: Position,
        modifiersPosition: Position,
        modifierFlags: ModifierFlags,
        name: &[u8],
        ty: Option<NonNull<dyn Type>>,
        builtin: bool,
        storage: VariableStorage,
    ) -> Self {
        Self {
            base: SymbolData::new(pos, super::SkSLIRNode::SymbolKind::kVariable, name, ty),
            fDeclaringElement: None,
            fModifierFlags: modifierFlags,
            fModifiersPosition: modifiersPosition,
            fStorage: storage,
            fBuiltin: builtin,
        }
    }
}
// Source: src/sksl/ir/SkSLVariable.h:79-81
// Adapted nested aggregate becomes a top-level Rust type; no uninitialized/default symbol owner is fabricated.
pub struct ScratchVariable {
    pub fVarSymbol: Option<NonNull<dyn Variable>>,
    pub fVarDecl: Option<super::SkSLIRNode::IRNodeBox<dyn super::SkSLStatement::Statement>>,
}
// Native virtual protocol; complete concrete base and extended producers follow below.
pub unsafe trait Variable: Symbol {
    fn variable_data(&self) -> &VariableData;
    fn variable_data_mut(self: Pin<&mut Self>) -> Pin<&mut VariableData>;
    // Source: src/sksl/ir/SkSLVariable.h:89-91
    fn modifierFlags(&self) -> ModifierFlags {
        self.variable_data().fModifierFlags
    }
    // Source: src/sksl/ir/SkSLVariable.h:93-93
    fn layout(&self) -> &Layout;
    // Source: src/sksl/ir/SkSLVariable.h:95-97
    fn modifiersPosition(&self) -> Position {
        self.variable_data().fModifiersPosition
    }
    // Source: src/sksl/ir/SkSLVariable.h:99-101
    fn isBuiltin(&self) -> bool {
        self.variable_data().fBuiltin
    }
    // Source: src/sksl/ir/SkSLVariable.h:103-105
    fn storage(&self) -> VariableStorage {
        self.variable_data().fStorage
    }
    // Native nonvirtual methods delegate to the single source-ordered implementations below.
    unsafe fn initialValue(&self) -> Option<&dyn super::SkSLExpression::Expression> {
        initial_value(self.variable_data())
    }
    unsafe fn varDeclaration(&self) -> Option<NonNull<super::SkSLVarDeclarations::VarDeclaration>> {
        variable_declaration(self.variable_data())
    }
    unsafe fn globalVarDeclaration(
        &self,
    ) -> Option<NonNull<super::SkSLVarDeclarations::GlobalVarDeclaration>> {
        global_declaration(self.variable_data())
    }
    unsafe fn setVarDeclaration(
        self: Pin<&mut Self>,
        declaration: NonNull<super::SkSLVarDeclarations::VarDeclaration>,
    ) {
        set_var_declaration(self.variable_data_mut(), declaration)
    }
    unsafe fn setGlobalVarDeclaration(
        self: Pin<&mut Self>,
        global: NonNull<super::SkSLVarDeclarations::GlobalVarDeclaration>,
    ) {
        set_global_declaration(self.variable_data_mut(), global)
    }
    // Source: src/sksl/ir/SkSLVariable.h:117-120
    unsafe fn detachDeadVarDeclaration(self: Pin<&mut Self>) {
        self.variable_data_mut()
            .get_unchecked_mut()
            .fDeclaringElement = None;
    }
    // Source: src/sksl/ir/SkSLVariable.h:125-125
    fn interfaceBlock(&self) -> Option<NonNull<super::SkSLInterfaceBlock::InterfaceBlock>> {
        None
    }
    // Source: src/sksl/ir/SkSLVariable.h:127-127
    unsafe fn setInterfaceBlock(
        self: Pin<&mut Self>,
        _elem: NonNull<super::SkSLInterfaceBlock::InterfaceBlock>,
    ) {
        std::process::abort();
    }
    // Source: src/sksl/ir/SkSLVariable.h:129-129
    unsafe fn detachDeadInterfaceBlock(self: Pin<&mut Self>) {}
    // Source: src/sksl/ir/SkSLVariable.h:132-132
    fn mangledName(&self) -> &[u8] {
        self.name()
    }
}
use super::{
    SkSLIRNode::{IRNode, IRNodeBox},
    SkSLInterfaceBlock::InterfaceBlock,
    SkSLVarDeclarations::{GlobalVarDeclaration, VarDeclaration},
};
use core::mem::ManuallyDrop;
// Source: src/sksl/ir/SkSLVariable.h:134-137
fn variable_description<T: Variable + ?Sized>(this: &T) -> Vec<u8> {
    let mut result = this.layout().paddedDescription();
    result.extend_from_slice(&this.modifierFlags().paddedDescription());
    result.extend_from_slice(&this.symbol_type().displayName());
    result.push(b' ');
    result.extend_from_slice(this.name());
    result
}
// Source: src/sksl/ir/SkSLVariable.h:158-158
// Source: src/sksl/ir/SkSLVariable.h:190-192
// Adapted explicit native destruction: extended name then base Variable teardown.
#[repr(C)]
pub struct ExtendedVariable {
    base: ManuallyDrop<VariableData>,
    fInterfaceBlockElement: Option<NonNull<InterfaceBlock>>,
    fLayout: Layout,
    fMangledName: ManuallyDrop<Vec<u8>>,
}
impl ExtendedVariable {
    // Source: src/sksl/ir/SkSLVariable.h:160-165
    pub unsafe fn new(
        pos: Position,
        modifiersPosition: Position,
        layout: Layout,
        flags: ModifierFlags,
        name: &[u8],
        ty: Option<NonNull<dyn Type>>,
        builtin: bool,
        storage: Storage,
        mangledName: Vec<u8>,
    ) -> Self {
        Self {
            base: ManuallyDrop::new(VariableData::new(
                pos,
                modifiersPosition,
                flags,
                name,
                ty,
                builtin,
                storage,
            )),
            fInterfaceBlockElement: None,
            fLayout: layout,
            fMangledName: ManuallyDrop::new(mangledName),
        }
    }
}
macro_rules! variable_projections {
    ($ty:ty,$base:expr) => {
        impl IRNode for $ty {
            fn node(&self) -> &IRNodeData {
                &self.variable_data().base.base
            }
            fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
                unsafe {
                    self.variable_data_mut()
                        .map_unchecked_mut(|v| &mut v.base.base)
                }
            }
            fn description(&self) -> Vec<u8> {
                variable_description(self)
            }
        }
        unsafe impl Symbol for $ty {
            unsafe fn as_variable_symbol(&self) -> &dyn Variable {
                self
            }

            unsafe fn as_type_symbol(&self) -> &dyn Type {
                panic!("invalid native Type cast")
            }
            fn symbol_data(&self) -> &SymbolData {
                &self.variable_data().base
            }
            fn symbol_data_mut(self: Pin<&mut Self>) -> Pin<&mut SymbolData> {
                unsafe { self.variable_data_mut().map_unchecked_mut(|v| &mut v.base) }
            }
        }
    };
}
variable_projections!(VariableData, ());
variable_projections!(ExtendedVariable, ());
unsafe impl Variable for VariableData {
    fn variable_data(&self) -> &VariableData {
        self
    }
    fn variable_data_mut(self: Pin<&mut Self>) -> Pin<&mut VariableData> {
        self
    }
    fn layout(&self) -> &Layout {
        default_layout()
    }
}
unsafe impl Variable for ExtendedVariable {
    fn variable_data(&self) -> &VariableData {
        &self.base
    }
    fn variable_data_mut(self: Pin<&mut Self>) -> Pin<&mut VariableData> {
        unsafe { self.map_unchecked_mut(|v| &mut *v.base) }
    }
    // Source: src/sksl/ir/SkSLVariable.h:169-171
    fn interfaceBlock(&self) -> Option<NonNull<InterfaceBlock>> {
        self.fInterfaceBlockElement
    }
    // Source: src/sksl/ir/SkSLVariable.h:173-175
    fn layout(&self) -> &Layout {
        &self.fLayout
    }
    // Source: src/sksl/ir/SkSLVariable.h:177-180
    unsafe fn setInterfaceBlock(self: Pin<&mut Self>, elem: NonNull<InterfaceBlock>) {
        let this = self.get_unchecked_mut();
        debug_assert!(this.fInterfaceBlockElement.is_none());
        this.fInterfaceBlockElement = Some(elem);
    }
    // Source: src/sksl/ir/SkSLVariable.h:182-185
    unsafe fn detachDeadInterfaceBlock(self: Pin<&mut Self>) {
        self.get_unchecked_mut().fInterfaceBlockElement = None;
    }
    fn mangledName(&self) -> &[u8] {
        extended_mangled_name(self)
    }
}
// Source: src/sksl/ir/SkSLVariable.cpp:29-29
static kDefaultLayout: Layout = Layout {
    fFlags: super::SkSLLayout::LayoutFlag(0),
    fLocation: -1,
    fOffset: -1,
    fBinding: -1,
    fTexture: -1,
    fSampler: -1,
    fIndex: -1,
    fSet: -1,
    fBuiltin: -1,
    fInputAttachmentIndex: -1,
    fLocalSizeX: -1,
    fLocalSizeY: -1,
    fLocalSizeZ: -1,
};
// Source: src/sksl/ir/SkSLVariable.cpp:31-36
impl Drop for VariableData {
    fn drop(&mut self) {
        unsafe {
            if let Some(mut declaration) = variable_declaration(self) {
                declaration.as_mut().detachDeadVariable();
            }
        }
    }
}
// Source: src/sksl/ir/SkSLVariable.cpp:38-43
impl Drop for ExtendedVariable {
    fn drop(&mut self) {
        unsafe {
            if let Some(mut block) = self.fInterfaceBlockElement {
                block.as_mut().detachDeadVariable();
            }
            ManuallyDrop::drop(&mut self.fMangledName);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}
// Source: src/sksl/ir/SkSLVariable.cpp:45-48
unsafe fn initial_value(data: &VariableData) -> Option<&dyn super::SkSLExpression::Expression> {
    variable_declaration(data).and_then(|p| p.as_ref().value().as_ref().map(|v| &**v))
}
// Source: src/sksl/ir/SkSLVariable.cpp:50-59
unsafe fn variable_declaration(data: &VariableData) -> Option<NonNull<VarDeclaration>> {
    let element = data.fDeclaringElement?;
    debug_assert!(
        element.as_ref().is::<VarDeclaration>() || element.as_ref().is::<GlobalVarDeclaration>()
    );
    if element.as_ref().is::<GlobalVarDeclaration>() {
        let mut global = element.cast::<GlobalVarDeclaration>();
        Some(NonNull::from(global.as_mut().varDeclaration_mut()))
    } else {
        Some(element.cast())
    }
}
// Source: src/sksl/ir/SkSLVariable.cpp:61-70
unsafe fn global_declaration(data: &VariableData) -> Option<NonNull<GlobalVarDeclaration>> {
    let element = data.fDeclaringElement?;
    debug_assert!(
        element.as_ref().is::<VarDeclaration>() || element.as_ref().is::<GlobalVarDeclaration>()
    );
    if element.as_ref().is::<GlobalVarDeclaration>() {
        Some(element.cast())
    } else {
        None
    }
}
// Source: src/sksl/ir/SkSLVariable.cpp:72-77
unsafe fn set_var_declaration(data: Pin<&mut VariableData>, declaration: NonNull<VarDeclaration>) {
    let this = data.get_unchecked_mut();
    debug_assert!(
        this.fDeclaringElement.is_none()
            || declaration
                .as_ref()
                .var()
                .is_some_and(|v| core::ptr::addr_eq(v.as_ref().variable_data(), this))
    );
    if this.fDeclaringElement.is_none() {
        this.fDeclaringElement = Some(declaration.cast());
    }
}
// Source: src/sksl/ir/SkSLVariable.cpp:79-82
unsafe fn set_global_declaration(
    data: Pin<&mut VariableData>,
    global: NonNull<GlobalVarDeclaration>,
) {
    let this = data.get_unchecked_mut();
    debug_assert!(
        this.fDeclaringElement.is_none()
            || global
                .as_ref()
                .varDeclaration()
                .var()
                .is_some_and(|v| core::ptr::addr_eq(v.as_ref().variable_data(), this))
    );
    this.fDeclaringElement = Some(global.cast());
}
// Source: src/sksl/ir/SkSLVariable.cpp:84-86
pub fn default_layout() -> &'static Layout {
    &kDefaultLayout
}
// Source: src/sksl/ir/SkSLVariable.cpp:88-90
fn extended_mangled_name(this: &ExtendedVariable) -> &[u8] {
    if this.fMangledName.is_empty() {
        this.name()
    } else {
        &this.fMangledName
    }
}
impl dyn Variable {
    // Source: src/sksl/ir/SkSLVariable.cpp:92-144
    pub unsafe fn Convert(
        context: &crate::src::sksl::SkSLContext::Context,
        pos: Position,
        modifiersPos: Position,
        layout: &Layout,
        mut flags: ModifierFlags,
        ty: NonNull<dyn Type>,
        _namePos: Position,
        name: &[u8],
        storage: Storage,
    ) -> IRNodeBox<dyn Variable> {
        use super::SkSLModifierFlags::ModifierFlag;
        use crate::src::sksl::{
            SkSLErrorReporter::ErrorReporterExt,
            SkSLIntrinsicList::{FindIntrinsicKind, IntrinsicKind},
            SkSLProgramSettings::ProgramConfig,
        };
        let config = context.fConfig.unwrap_unchecked().as_ref();
        let mut errors = context.fErrors;
        if layout.fLocation == 0
            && layout.fIndex == 0
            && flags.0 & ModifierFlag::kOut.0 != 0
            && ProgramConfig::IsFragment(config.fKind)
            && name != b"sk_FragColor"
        {
            errors.as_mut().error(
                modifiersPos,
                b"out location=0, index=0 is reserved for sk_FragColor",
            );
        }
        if ty.as_ref().isUnsizedArray()
            && storage != Storage::kInterfaceBlock
            && storage != Storage::kParameter
        {
            errors
                .as_mut()
                .error(pos, b"unsized arrays are not permitted here");
        }
        if ProgramConfig::IsCompute(config.fKind) && layout.fBuiltin == -1 {
            if storage == Storage::kGlobal {
                if flags.0 & ModifierFlag::kIn.0 != 0 {
                    errors
                        .as_mut()
                        .error(pos, b"pipeline inputs not permitted in compute shaders");
                } else if flags.0 & ModifierFlag::kOut.0 != 0 {
                    errors
                        .as_mut()
                        .error(pos, b"pipeline outputs not permitted in compute shaders");
                }
            }
        }
        if storage == Storage::kParameter {
            let mask = ModifierFlag::kOut.0 | ModifierFlag::kIn.0;
            if flags.0 & mask == ModifierFlag::kIn.0 {
                flags.0 &= !mask;
            }
        }
        let mut mangled = Vec::new();
        if name.first() == Some(&b'$') {
            mangled.extend_from_slice(b"sk_Priv");
            mangled.extend_from_slice(&name[1..]);
        } else if FindIntrinsicKind(name) != IntrinsicKind::kNotIntrinsic {
            mangled = crate::src::sksl::SkSLMangler::Mangler::default()
                .uniqueName(name, context.fSymbolTable.unwrap_unchecked().as_ref());
        }
        Self::Make(
            pos,
            modifiersPos,
            layout,
            flags,
            ty,
            name,
            mangled,
            config.isBuiltinCode(),
            storage,
        )
    }
    // Source: src/sksl/ir/SkSLVariable.cpp:146-179
    pub unsafe fn Make(
        pos: Position,
        modifiersPosition: Position,
        layout: &Layout,
        flags: ModifierFlags,
        ty: NonNull<dyn Type>,
        name: &[u8],
        mangledName: Vec<u8>,
        builtin: bool,
        storage: Storage,
    ) -> IRNodeBox<dyn Variable> {
        use super::SkSLModifierFlags::ModifierFlag;
        debug_assert!(
            !(storage == Storage::kParameter
                && flags.0 & (ModifierFlag::kOut.0 | ModifierFlag::kIn.0) == ModifierFlag::kIn.0)
        );
        if ty.as_ref().componentType().isInterfaceBlock()
            || !mangledName.is_empty()
            || *layout != kDefaultLayout
        {
            IRNodeBox::new_with(|p: *mut ExtendedVariable| {
                p.write(ExtendedVariable::new(
                    pos,
                    modifiersPosition,
                    *layout,
                    flags,
                    name,
                    Some(ty),
                    builtin,
                    storage,
                    mangledName,
                ))
            })
            .cast(|p| p as *mut dyn Variable)
        } else {
            IRNodeBox::new_with(|p: *mut VariableData| {
                p.write(VariableData::new(
                    pos,
                    modifiersPosition,
                    flags,
                    name,
                    Some(ty),
                    builtin,
                    storage,
                ))
            })
            .cast(|p| p as *mut dyn Variable)
        }
    }
}

impl dyn Variable {
    // Source: src/sksl/ir/SkSLVariable.cpp:181-220
    // UNVERIFIED actual scratch declaration producer. The stable name and Variable
    // owner transfer into the supplied real SymbolTable; no temporary registry/owner.
    pub unsafe fn MakeScratchVariable(
        context: &crate::src::sksl::SkSLContext::Context,
        mangler: &mut crate::src::sksl::SkSLMangler::Mangler,
        baseName: &[u8],
        mut ty: NonNull<dyn Type>,
        symbolTable: &mut super::SkSLSymbolTable::SymbolTable,
        initialValue: Option<IRNodeBox<dyn super::SkSLExpression::Expression>>,
    ) -> ScratchVariable {
        if ty.as_ref().isLiteral() {
            crate::SkDEBUGFAIL!(@source b"src/sksl/ir/SkSLVariable.cpp\0".as_ptr(),191; b"found a $literal type in MakeScratchVariable\0".as_ptr());
            ty = core::mem::transmute(NonNull::from(ty.as_ref().scalarTypeForLiteral()));
        }
        let name = mangler.uniqueName(baseName, symbolTable);
        let name = symbolTable.takeOwnershipOfString(name);
        // Native name->c_str() enters string_view through strlen, including embedded-NUL truncation.
        let bytes = name.as_ref().as_slice();
        let nameBytes = &bytes[..bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len())];
        let var = IRNodeBox::new_with(|p: *mut VariableData| {
            p.write(VariableData::new(
                initialValue
                    .as_ref()
                    .map_or(Position::new(), |value| value.node().fPosition),
                Position::new(),
                ModifierFlags(0),
                nameBytes,
                Some(ty),
                symbolTable.isBuiltin(),
                Storage::kLocal,
            ))
        })
        .cast(|p| p as *mut dyn Variable);
        let varPointer = NonNull::new_unchecked(var.as_ptr() as *mut dyn Variable);
        let mut arraySize = 0;
        if ty.as_ref().isArray() {
            arraySize = ty.as_ref().columns();
            ty = core::mem::transmute(NonNull::from(ty.as_ref().componentType()));
        }
        let declaration = super::SkSLVarDeclarations::VarDeclaration::Make(
            context,
            varPointer,
            ty,
            arraySize,
            initialValue,
        )
        .cast(|p| p as *mut dyn super::SkSLStatement::Statement);
        symbolTable.add(context, var.cast(|p| p as *mut dyn Symbol));
        ScratchVariable {
            fVarSymbol: Some(varPointer),
            fVarDecl: Some(declaration),
        }
    }
}
