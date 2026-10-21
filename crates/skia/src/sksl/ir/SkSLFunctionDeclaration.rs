use super::{
    SkSLFunctionDefinition::FunctionDefinition, SkSLModifierFlags::ModifierFlags,
    SkSLSymbol::SymbolData, SkSLType::Type, SkSLVariable::Variable,
};
use crate::{
    include::private::SkTArray::TArray,
    src::sksl::{SkSLIntrinsicList::IntrinsicKind, SkSLModule::ModuleType},
};
use core::{cell::Cell, ptr::NonNull};
// Source: src/sksl/ir/SkSLFunctionDeclaration.h:37-37
// Source: src/sksl/ir/SkSLFunctionDeclaration.h:158-168
// Actual final-class native fields. No constructor/IRNode vtable is fabricated in
// place of native FunctionDeclaration.cpp definitions not in this ownership batch.
#[repr(C)]
pub struct FunctionDeclaration {
    pub(crate) base: SymbolData,
    pub(crate) fDefinition: Option<NonNull<FunctionDefinition>>,
    pub(crate) fNextOverload: Option<NonNull<FunctionDeclaration>>,
    pub(crate) fParameters: TArray<Option<NonNull<dyn Variable>>, true>,
    pub(crate) fReturnType: Option<NonNull<dyn Type>>,
    pub(crate) fModifierFlags: ModifierFlags,
    pub(crate) fIntrinsicKind: Cell<IntrinsicKind>,
    pub(crate) fModuleType: ModuleType,
    pub(crate) fIsMain: bool,
    pub(crate) fHasMainCoordsParameter: bool,
    pub(crate) fHasMainInputColorParameter: bool,
    pub(crate) fHasMainDestColorParameter: bool,
}
// Source: src/sksl/ir/SkSLFunctionDeclaration.h:39-39
impl super::SkSLIRNode::IRNodeType for FunctionDeclaration {
    const kIRNodeKind: i32 = super::SkSLIRNode::SymbolKind::kFunctionDeclaration as i32;
}
impl FunctionDeclaration {
    // Source: src/sksl/ir/SkSLFunctionDeclaration.h:57-59
    pub fn modifierFlags(&self) -> ModifierFlags {
        self.fModifierFlags
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.h:61-63
    pub fn setModifierFlags(&mut self, m: ModifierFlags) {
        self.fModifierFlags = m;
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.h:65-67
    pub fn definition(&self) -> Option<&FunctionDefinition> {
        self.fDefinition.map(|p| unsafe { p.as_ref() })
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.h:69-72
    pub unsafe fn setDefinition(&mut self, definition: Option<NonNull<FunctionDefinition>>) {
        self.fDefinition = definition;
        self.fIntrinsicKind.set(IntrinsicKind::kNotIntrinsic);
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.h:74-77
    pub unsafe fn setNextOverload(&mut self, overload: Option<NonNull<FunctionDeclaration>>) {
        debug_assert!(
            overload.is_none_or(|p| p.as_ref().base.name_bytes() == self.base.name_bytes())
        );
        self.fNextOverload = overload;
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.h:79-81
    pub fn parameters(&self) -> &[Option<NonNull<dyn Variable>>] {
        self.fParameters.as_slice()
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.h:83-85
    pub fn returnType(&self) -> &dyn Type {
        debug_assert!(self.fReturnType.is_some());
        unsafe { self.fReturnType.unwrap_unchecked().as_ref() }
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.h:87-89
    pub fn moduleType(&self) -> ModuleType {
        self.fModuleType
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.h:91-93
    pub fn isBuiltin(&self) -> bool {
        self.moduleType() != ModuleType::program
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.h:95-97
    pub fn isMain(&self) -> bool {
        self.fIsMain
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.h:99-101
    pub fn intrinsicKind(&self) -> IntrinsicKind {
        self.fIntrinsicKind.get()
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.h:103-105
    pub fn isIntrinsic(&self) -> bool {
        self.intrinsicKind() != IntrinsicKind::kNotIntrinsic
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.h:107-109
    pub fn nextOverload(&self) -> Option<&FunctionDeclaration> {
        self.fNextOverload.map(|p| unsafe { p.as_ref() })
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.h:111-113
    pub unsafe fn mutableNextOverload(&mut self) -> Option<&mut FunctionDeclaration> {
        self.fNextOverload.map(|mut p| p.as_mut())
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.h:127-129
    pub fn getMainCoordsParameter(&self) -> Option<&dyn Variable> {
        if self.fHasMainCoordsParameter {
            self.fParameters.as_slice()[0].map(|p| unsafe { p.as_ref() })
        } else {
            None
        }
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.h:130-132
    pub fn getMainInputColorParameter(&self) -> Option<&dyn Variable> {
        if self.fHasMainInputColorParameter {
            self.fParameters.as_slice()[0].map(|p| unsafe { p.as_ref() })
        } else {
            None
        }
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.h:133-135
    pub fn getMainDestColorParameter(&self) -> Option<&dyn Variable> {
        if self.fHasMainDestColorParameter {
            self.fParameters.as_slice()[1].map(|p| unsafe { p.as_ref() })
        } else {
            None
        }
    }
}
// Source: src/sksl/ir/SkSLFunctionDeclaration.h:152-152
// Adapted pinned native inline8 nullable Type-pointer storage.
pub type ParamTypes =
    crate::include::private::SkTArray::STArray<Option<NonNull<dyn Type>>, 8, true>;
// Language-only typed span projections retain borrowed and unique_ptr element views without temporary owners.
pub trait ParameterSpan {
    fn len(&self) -> usize;
    unsafe fn variable(&self, i: usize) -> &dyn Variable;
}
impl ParameterSpan for [Option<NonNull<dyn Variable>>] {
    fn len(&self) -> usize {
        <[Option<NonNull<dyn Variable>>]>::len(self)
    }
    unsafe fn variable(&self, i: usize) -> &dyn Variable {
        self[i].unwrap_unchecked().as_ref()
    }
}
impl ParameterSpan for [Option<super::SkSLIRNode::IRNodeBox<dyn Variable>>] {
    fn len(&self) -> usize {
        <[Option<super::SkSLIRNode::IRNodeBox<dyn Variable>>]>::len(self)
    }
    unsafe fn variable(&self, i: usize) -> &dyn Variable {
        &**self[i].as_ref().unwrap_unchecked()
    }
}
// Source: src/sksl/ir/SkSLFunctionDeclaration.cpp:38-51
unsafe fn check_modifiers(
    context: &crate::src::sksl::SkSLContext::Context,
    pos: crate::src::sksl::SkSLPosition::Position,
    flags: ModifierFlags,
) -> bool {
    use super::SkSLModifierFlags::ModifierFlag as F;
    use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
    let permitted = F::kInline.0
        | F::kNoInline.0
        | if context.fConfig.unwrap_unchecked().as_ref().isBuiltinCode() {
            F::kES3.0 | F::kPure.0 | F::kExport.0
        } else {
            0
        };
    flags.checkPermittedFlags(context, pos, ModifierFlags(permitted));
    if flags.isInline() && flags.isNoInline() {
        let mut errors = context.fErrors;
        errors
            .as_mut()
            .error(pos, b"functions cannot be both 'inline' and 'noinline'");
        return false;
    }
    true
}
// Source: src/sksl/ir/SkSLFunctionDeclaration.cpp:53-69
pub unsafe fn check_return_type(
    context: &crate::src::sksl::SkSLContext::Context,
    pos: crate::src::sksl::SkSLPosition::Position,
    returnType: &dyn Type,
) -> bool {
    use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
    let mut errors = context.fErrors;
    if returnType.isArray() {
        let mut message = b"functions may not return type '".to_vec();
        message.extend_from_slice(&returnType.displayName());
        message.push(b'\'');
        errors.as_mut().error(pos, &message);
        return false;
    }
    let config = context.fConfig.unwrap_unchecked().as_ref();
    if config.strictES2Mode() && returnType.isOrContainsArray() {
        errors
            .as_mut()
            .error(pos, b"functions may not return structs containing arrays");
        return false;
    }
    if !config.isBuiltinCode() && returnType.componentType().isOpaque() {
        let mut message = b"functions may not return opaque type '".to_vec();
        message.extend_from_slice(&returnType.displayName());
        message.push(b'\'');
        errors.as_mut().error(pos, &message);
        return false;
    }
    true
}
// Source: src/sksl/ir/SkSLFunctionDeclaration.cpp:71-123
unsafe fn check_parameters(
    context: &crate::src::sksl::SkSLContext::Context,
    parameters: &[Option<super::SkSLIRNode::IRNodeBox<dyn Variable>>],
    flags: ModifierFlags,
    intrinsic: IntrinsicKind,
) -> bool {
    use super::{SkSLLayout::LayoutFlag as L, SkSLModifierFlags::ModifierFlag as F};
    use crate::src::sksl::{
        SkSLErrorReporter::ErrorReporterExt, SkSLProgramSettings::ProgramConfig,
    };
    let mut errors = context.fErrors;
    for param in parameters {
        let param = param.as_ref().unwrap_unchecked();
        let ty = param.symbol_type();
        let mut permitted = F::kConst.0 | F::kIn.0;
        let mut layouts = 0;
        if !ty.isOpaque() {
            permitted |= F::kOut.0;
        }
        if ty.isStorageTexture() {
            permitted |= F::kReadOnly.0 | F::kWriteOnly.0;
            layouts |= L::kAllPixelFormats.0;
            if intrinsic == IntrinsicKind::kNotIntrinsic
                && param.layout().fFlags.0 & L::kAllPixelFormats.0 == 0
            {
                errors.as_mut().error(
                    param.node().fPosition,
                    b"storage texture parameters must specify a pixel format layout-qualifier",
                );
                return false;
            }
        }
        param.modifierFlags().checkPermittedFlags(
            context,
            param.modifiersPosition(),
            ModifierFlags(permitted),
        );
        param.layout().checkPermittedLayout(
            context,
            param.modifiersPosition(),
            super::SkSLLayout::LayoutFlag(layouts),
        );
        if !ProgramConfig::AllowsPrivateIdentifiers(
            context.fConfig.unwrap_unchecked().as_ref().fKind,
        ) && ty.isEffectChild()
        {
            let mut message = b"parameters of type '".to_vec();
            message.extend_from_slice(&ty.displayName());
            message.extend_from_slice(b"' not allowed");
            errors.as_mut().error(param.node().fPosition, &message);
            return false;
        }
        if flags.isPure() && param.modifierFlags().0 & F::kOut.0 != 0 {
            errors.as_mut().error(
                param.modifiersPosition(),
                b"pure functions cannot have out parameters",
            );
            return false;
        }
    }
    true
}
// Source: src/sksl/ir/SkSLFunctionDeclaration.cpp:125-127
fn type_is_valid_for_color(ty: &dyn Type) -> bool {
    ty.isVector() && ty.columns() == 4 && ty.componentType().isFloat()
}
// Source: src/sksl/ir/SkSLFunctionDeclaration.cpp:129-132
fn type_is_valid_for_coords(ty: &dyn Type) -> bool {
    ty.isVector() && ty.highPrecision() && ty.columns() == 2 && ty.componentType().isFloat()
}
// Source: src/sksl/ir/SkSLFunctionDeclaration.cpp:134-264
// Adapted borrowed parameter view retains native nonnull unique_ptr element preconditions.
pub unsafe fn check_main_signature<P: ParameterSpan + ?Sized>(
    context: &crate::src::sksl::SkSLContext::Context,
    pos: crate::src::sksl::SkSLPosition::Position,
    returnType: &dyn Type,
    parameters: &P,
) -> bool {
    use super::SkSLModifierFlags::ModifierFlag;
    use crate::src::sksl::{SkSLErrorReporter::ErrorReporterExt, SkSLProgramKind::ProgramKind};
    let mut errors = context.fErrors;
    let kind = context.fConfig.unwrap_unchecked().as_ref().fKind;
    let attributes = |ty: &dyn Type| ty.isStruct() && ty.name() == b"Attributes";
    let varyings = |ty: &dyn Type| ty.isStruct() && ty.name() == b"Varyings";
    let coords = |i: usize| {
        let p = parameters.variable(i);
        type_is_valid_for_coords(p.symbol_type()) && p.modifierFlags().0 == 0
    };
    let color = |i: usize| {
        let p = parameters.variable(i);
        type_is_valid_for_color(p.symbol_type()) && p.modifierFlags().0 == 0
    };
    let const_attributes = |i: usize| {
        let p = parameters.variable(i);
        attributes(p.symbol_type()) && p.modifierFlags().0 == ModifierFlag::kConst.0
    };
    let const_varyings = |i: usize| {
        let p = parameters.variable(i);
        varyings(p.symbol_type()) && p.modifierFlags().0 == ModifierFlag::kConst.0
    };
    let out_color = |i: usize| {
        let p = parameters.variable(i);
        type_is_valid_for_color(p.symbol_type()) && p.modifierFlags().0 == ModifierFlag::kOut.0
    };
    match kind {
        ProgramKind::kRuntimeColorFilter | ProgramKind::kPrivateRuntimeColorFilter => {
            if !type_is_valid_for_color(returnType) {
                errors
                    .as_mut()
                    .error(pos, b"'main' must return: 'vec4', 'float4', or 'half4'");
                return false;
            }
            let valid = parameters.len() == 1 && color(0);
            if !valid {
                errors.as_mut().error(
                    pos,
                    b"'main' parameter must be 'vec4', 'float4', or 'half4'",
                );
                return false;
            }
        }
        ProgramKind::kRuntimeShader | ProgramKind::kPrivateRuntimeShader => {
            if !type_is_valid_for_color(returnType) {
                errors
                    .as_mut()
                    .error(pos, b"'main' must return: 'vec4', 'float4', or 'half4'");
                return false;
            }
            if !(parameters.len() == 1 && coords(0)) {
                errors
                    .as_mut()
                    .error(pos, b"'main' parameter must be 'float2' or 'vec2'");
                return false;
            }
        }
        ProgramKind::kRuntimeBlender | ProgramKind::kPrivateRuntimeBlender => {
            if !type_is_valid_for_color(returnType) {
                errors
                    .as_mut()
                    .error(pos, b"'main' must return: 'vec4', 'float4', or 'half4'");
                return false;
            }
            if !(parameters.len() == 2 && color(0) && color(1)) {
                errors.as_mut().error(
                    pos,
                    b"'main' parameters must be (vec4|float4|half4, vec4|float4|half4)",
                );
                return false;
            }
        }
        ProgramKind::kMeshVertex => {
            if !varyings(returnType) {
                errors
                    .as_mut()
                    .error(pos, b"'main' must return 'Varyings'.");
                return false;
            }
            if !(parameters.len() == 1 && const_attributes(0)) {
                errors
                    .as_mut()
                    .error(pos, b"'main' parameter must be 'const Attributes'.");
                return false;
            }
        }
        ProgramKind::kMeshFragment => {
            if !type_is_valid_for_coords(returnType) {
                errors
                    .as_mut()
                    .error(pos, b"'main' must return: 'vec2' or 'float2'");
                return false;
            }
            if !((parameters.len() == 1 && const_varyings(0))
                || (parameters.len() == 2 && const_varyings(0) && out_color(1)))
            {
                errors.as_mut().error(
                    pos,
                    b"'main' parameters must be (const Varyings, (out (half4|float4))?)",
                );
                return false;
            }
        }
        ProgramKind::kFragment | ProgramKind::kGraphiteFragment => {
            let valid = (parameters.len() == 0) || (parameters.len() == 1 && coords(0));
            if !valid {
                errors
                    .as_mut()
                    .error(pos, b"shader 'main' must be main() or main(float2)");
                return false;
            }
        }
        ProgramKind::kVertex | ProgramKind::kGraphiteVertex | ProgramKind::kCompute => {
            if !returnType.matches(&**context.fTypes.as_ref().fVoid) {
                errors.as_mut().error(pos, b"'main' must return 'void'");
                return false;
            }
            if !(parameters.len() == 0) {
                errors
                    .as_mut()
                    .error(pos, b"shader 'main' must have zero parameters");
                return false;
            }
        }
    }
    true
}
// Source: src/sksl/ir/SkSLFunctionDeclaration.cpp:270-280
fn find_generic_index(concrete: &dyn Type, generic: &dyn Type, allowNarrowing: bool) -> i32 {
    let types = generic.coercibleTypes();
    for (index, ty) in types.iter().enumerate() {
        if concrete.canCoerceTo(unsafe { ty.as_ref() }, allowNarrowing) {
            return index as i32;
        }
    }
    -1
}
// Source: src/sksl/ir/SkSLFunctionDeclaration.cpp:283-287
fn type_generically_matches(concrete: &dyn Type, generic: &dyn Type) -> bool {
    if generic.isGeneric() {
        find_generic_index(concrete, generic, false) != -1
    } else {
        concrete.matches(generic)
    }
}
// Source: src/sksl/ir/SkSLFunctionDeclaration.cpp:294-339
pub unsafe fn parameters_match<P: ParameterSpan + ?Sized>(
    params: &P,
    other: &[Option<NonNull<dyn Variable>>],
) -> bool {
    if params.len() != other.len() {
        return false;
    }
    let mut genericIndex = -1;
    for i in 0..params.len() {
        let param = params.variable(i).symbol_type();
        let otherParam = other[i].unwrap_unchecked().as_ref().symbol_type();
        if otherParam.isGeneric() {
            let next = find_generic_index(param, otherParam, false);
            if next == -1 {
                return false;
            }
            if genericIndex != -1 && genericIndex != next {
                return false;
            }
            genericIndex = next;
        }
    }
    for i in 0..params.len() {
        let param = params.variable(i).symbol_type();
        let mut otherParam = other[i].unwrap_unchecked().as_ref().symbol_type();
        if otherParam.isGeneric() {
            crate::SkASSERT!(@source b"src/sksl/ir/SkSLFunctionDeclaration.cpp\0".as_ptr(),329; @condition b"genericIndex != -1\0".as_ptr(); genericIndex != -1);
            debug_assert!((genericIndex as usize) < otherParam.coercibleTypes().len());
            otherParam = otherParam.coercibleTypes()[genericIndex as usize].as_ref();
        }
        if !param.matches(otherParam) {
            return false;
        }
    }
    true
}
// Source: src/sksl/ir/SkSLFunctionDeclaration.cpp:346-418
unsafe fn find_existing_declaration(
    context: &crate::src::sksl::SkSLContext::Context,
    pos: crate::src::sksl::SkSLPosition::Position,
    flags: ModifierFlags,
    intrinsic: IntrinsicKind,
    name: &[u8],
    parameters: &[Option<super::SkSLIRNode::IRNodeBox<dyn Variable>>],
    returnTypePos: crate::src::sksl::SkSLPosition::Position,
    returnType: NonNull<dyn Type>,
    out: &mut Option<NonNull<FunctionDeclaration>>,
) -> bool {
    use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
    let invalid_description = || {
        let mut pointers = TArray::new();
        pointers.reserve_exact(parameters.len() as i32);
        for param in parameters {
            pointers.push_back(Some(NonNull::new_unchecked(
                param.as_ref().unwrap_unchecked().as_ptr() as *mut dyn Variable,
            )));
        }
        FunctionDeclaration::new(
            context,
            pos,
            flags,
            name,
            pointers,
            Some(returnType),
            intrinsic,
        )
        .description()
    };
    let mut errors = context.fErrors;
    let entry = context
        .fSymbolTable
        .unwrap_unchecked()
        .as_ref()
        .findMutable(name);
    *out = None;
    if let Some(entry) = entry {
        if !entry.as_ref().node().is::<FunctionDeclaration>() {
            let mut message = b"symbol '".to_vec();
            message.extend_from_slice(name);
            message.extend_from_slice(b"' was already defined");
            errors.as_mut().error(pos, &message);
            return false;
        }
        let first = NonNull::new_unchecked(entry.as_ptr() as *mut () as *mut FunctionDeclaration);
        let mut next = Some(first);
        while let Some(other) = next {
            let otherRef = other.as_ref();
            debug_assert_eq!(name, otherRef.base.name_bytes());
            if !parameters_match(parameters, otherRef.parameters()) {
                next = otherRef.fNextOverload;
                continue;
            }
            if !type_generically_matches(returnType.as_ref(), otherRef.returnType()) {
                let mut message = b"functions '".to_vec();
                message.extend_from_slice(&invalid_description());
                message.extend_from_slice(b"' and '");
                message.extend_from_slice(&otherRef.description());
                message.extend_from_slice(b"' differ only in return type");
                errors.as_mut().error(returnTypePos, &message);
                return false;
            }
            for i in 0..parameters.len() {
                let p = parameters[i].as_ref().unwrap_unchecked();
                let q = otherRef.parameters()[i].unwrap_unchecked().as_ref();
                if p.modifierFlags() != q.modifierFlags() || p.layout() != q.layout() {
                    let mut message = b"modifiers on parameter ".to_vec();
                    message.extend_from_slice((i + 1).to_string().as_bytes());
                    message.extend_from_slice(b" differ between declaration and definition");
                    errors.as_mut().error(p.node().fPosition, &message);
                    return false;
                }
            }
            if otherRef.isIntrinsic() {
                let mut message = b"duplicate definition of intrinsic function '".to_vec();
                message.extend_from_slice(name);
                message.push(b'\'');
                errors.as_mut().error(pos, &message);
                return false;
            }
            if flags != otherRef.modifierFlags() {
                let mut message = b"functions '".to_vec();
                message.extend_from_slice(&invalid_description());
                message.extend_from_slice(b"' and '");
                message.extend_from_slice(&otherRef.description());
                message.extend_from_slice(b"' differ only in modifiers");
                errors.as_mut().error(pos, &message);
                return false;
            }
            *out = Some(other);
            break;
        }
        if out.is_none() && first.as_ref().isMain() {
            errors
                .as_mut()
                .error(pos, b"duplicate definition of 'main'");
            return false;
        }
    }
    true
}
impl FunctionDeclaration {
    // Source: src/sksl/ir/SkSLFunctionDeclaration.cpp:420-465
    // Adapted owning parameter array plus borrowed polymorphic inputs retain the native object leases.
    pub unsafe fn new(
        context: &crate::src::sksl::SkSLContext::Context,
        pos: crate::src::sksl::SkSLPosition::Position,
        modifierFlags: ModifierFlags,
        name: &[u8],
        parameters: TArray<Option<NonNull<dyn Variable>>, true>,
        returnType: Option<NonNull<dyn Type>>,
        intrinsicKind: IntrinsicKind,
    ) -> Self {
        use crate::src::sksl::SkSLProgramSettings::ProgramConfig;
        let config = context.fConfig.unwrap_unchecked().as_ref();
        let mut value = Self {
            base: SymbolData::new(
                pos,
                super::SkSLIRNode::SymbolKind::kFunctionDeclaration,
                name,
                None,
            ),
            fDefinition: None,
            fNextOverload: None,
            fParameters: parameters,
            fReturnType: returnType,
            fModifierFlags: modifierFlags,
            fIntrinsicKind: Cell::new(intrinsicKind),
            fModuleType: config.fModuleType,
            fIsMain: name == b"main",
            fHasMainCoordsParameter: false,
            fHasMainInputColorParameter: false,
            fHasMainDestColorParameter: false,
        };
        let mut index = 0;
        for param in value.fParameters.as_slice() {
            crate::SkASSERT!(@source b"src/sksl/ir/SkSLFunctionDeclaration.cpp\0".as_ptr(),438; @condition b"param\0".as_ptr(); param.is_some());
            let param = param.unwrap_unchecked().as_ref();
            if value.fIsMain {
                if ProgramConfig::IsRuntimeShader(config.fKind)
                    || ProgramConfig::IsFragment(config.fKind)
                {
                    if type_is_valid_for_coords(param.symbol_type()) {
                        value.fHasMainCoordsParameter = true;
                    }
                } else if ProgramConfig::IsRuntimeColorFilter(config.fKind)
                    || ProgramConfig::IsRuntimeBlender(config.fKind)
                {
                    if type_is_valid_for_color(param.symbol_type()) {
                        match index {
                            0 => value.fHasMainInputColorParameter = true,
                            1 => value.fHasMainDestColorParameter = true,
                            _ => {}
                        }
                        index += 1;
                    }
                }
            }
        }
        value
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.cpp:467-515
    pub unsafe fn Convert(
        context: &crate::src::sksl::SkSLContext::Context,
        pos: crate::src::sksl::SkSLPosition::Position,
        modifiers: &super::SkSLModifiers::Modifiers,
        name: &[u8],
        mut parameters: TArray<Option<super::SkSLIRNode::IRNodeBox<dyn Variable>>, true>,
        returnTypePos: crate::src::sksl::SkSLPosition::Position,
        returnType: NonNull<dyn Type>,
    ) -> Option<NonNull<FunctionDeclaration>> {
        use super::SkSLIRNode::IRNodeBox;
        use super::{SkSLLayout::LayoutFlag, SkSLModifierFlags::ModifierFlag};
        modifiers
            .fLayout
            .checkPermittedLayout(context, pos, LayoutFlag::kNone);
        let mut flags = modifiers.fFlags;
        let config = context.fConfig.unwrap_unchecked().as_ref();
        if config.fSettings.fForceNoInline {
            flags.0 &= !ModifierFlag::kInline.0;
            flags.0 |= ModifierFlag::kNoInline.0;
        }
        let isMain = name == b"main";
        let intrinsic = if config.isBuiltinCode() {
            crate::src::sksl::SkSLIntrinsicList::FindIntrinsicKind(name)
        } else {
            IntrinsicKind::kNotIntrinsic
        };
        let mut decl = None;
        if !check_modifiers(context, modifiers.fPosition, flags)
            || !check_return_type(context, returnTypePos, returnType.as_ref())
            || !check_parameters(context, parameters.as_slice(), flags, intrinsic)
            || (isMain
                && !check_main_signature(context, pos, returnType.as_ref(), parameters.as_slice()))
            || !find_existing_declaration(
                context,
                pos,
                flags,
                intrinsic,
                name,
                parameters.as_slice(),
                returnTypePos,
                returnType,
                &mut decl,
            )
        {
            return None;
        }
        let mut finalParameters = TArray::new();
        finalParameters.reserve_exact(parameters.size());
        let mut table = context.fSymbolTable.unwrap_unchecked();
        // Adapted unique_ptr move slots are nullable while transferred in native order.
        for slot in parameters.as_mut_slice() {
            let owner = slot.take().unwrap_unchecked();
            let pointer = NonNull::new_unchecked(owner.as_ptr() as *mut dyn Variable);
            table.as_mut().takeOwnershipOfSymbol(Some(
                owner.cast(|p| p as *mut dyn super::SkSLSymbol::Symbol),
            ));
            finalParameters.push_back(Some(pointer));
        }
        if decl.is_some() {
            return decl;
        }
        let owner = IRNodeBox::new_with(|p: *mut Self| {
            p.write(Self::new(
                context,
                pos,
                flags,
                name,
                finalParameters,
                Some(returnType),
                intrinsic,
            ))
        });
        let result = NonNull::new_unchecked(owner.as_ptr() as *mut Self);
        table.as_mut().add(
            context,
            owner.cast(|p| p as *mut dyn super::SkSLSymbol::Symbol),
        );
        Some(result)
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.cpp:517-537
    pub fn mangledName(&self) -> Vec<u8> {
        if (self.isBuiltin() && self.definition().is_none()) || self.isMain() {
            return self.base.name_bytes().to_vec();
        }
        let mut name = self.base.name_bytes();
        let marker = if name.first() == Some(&b'$') {
            name = &name[1..];
            b"Q".as_slice()
        } else {
            b"".as_slice()
        };
        let mut result = name.to_vec();
        result.push(b'_');
        result.extend_from_slice(marker);
        result.extend_from_slice(self.returnType().abbreviatedName().to_bytes());
        for p in self.parameters() {
            result.extend_from_slice(
                unsafe { p.unwrap_unchecked().as_ref() }
                    .symbol_type()
                    .abbreviatedName()
                    .to_bytes(),
            );
        }
        result
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.cpp:539-549
    pub fn description(&self) -> Vec<u8> {
        let mut result = if self.fModifierFlags.0 != 0 {
            let mut text = self.fModifierFlags.description();
            text.push(b' ');
            text
        } else {
            Vec::new()
        };
        result.extend_from_slice(&self.returnType().displayName());
        result.push(b' ');
        result.extend_from_slice(self.base.name_bytes());
        result.push(b'(');
        let mut separator = crate::src::sksl::SkSLString::Separator();
        for p in self.parameters() {
            result.extend_from_slice(separator());
            result.extend_from_slice(&unsafe { p.unwrap_unchecked().as_ref() }.description());
        }
        result.push(b')');
        result
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.cpp:551-566
    pub fn matches(&self, other: &FunctionDeclaration) -> bool {
        if self.base.name_bytes() != other.base.name_bytes() {
            return false;
        }
        let params = self.parameters();
        let otherParams = other.parameters();
        if params.len() != otherParams.len() {
            return false;
        }
        for i in 0..params.len() {
            if !unsafe { params[i].unwrap_unchecked().as_ref() }
                .symbol_type()
                .matches(unsafe { otherParams[i].unwrap_unchecked().as_ref() }.symbol_type())
            {
                return false;
            }
        }
        true
    }
    // Source: src/sksl/ir/SkSLFunctionDeclaration.cpp:568-608
    pub unsafe fn determineFinalTypes(
        &self,
        arguments: &crate::src::sksl::SkSLDefines::ExpressionArray,
        mut outParameterTypes: core::pin::Pin<&mut ParamTypes>,
        outReturnType: &mut Option<NonNull<dyn Type>>,
    ) -> bool {
        use super::SkSLExpression::ExpressionExt;
        let parameters = self.parameters();
        debug_assert_eq!(arguments.as_base().size() as usize, parameters.len());
        outParameterTypes
            .as_mut()
            .reserve_exact(arguments.as_base().size());
        let mut genericIndex = -1;
        for i in 0..arguments.as_base().size() as usize {
            let parameter = parameters.variable(i).symbol_type();
            if !parameter.isGeneric() {
                outParameterTypes
                    .as_mut()
                    .push_back(Some(core::mem::transmute(NonNull::from(parameter))));
                continue;
            }
            if genericIndex == -1 {
                genericIndex = find_generic_index(
                    arguments.as_base().as_slice()[i]
                        .as_ref()
                        .unwrap_unchecked()
                        .expression_type(),
                    parameter,
                    true,
                );
                if genericIndex == -1 {
                    return false;
                }
            }
            outParameterTypes
                .as_mut()
                .push_back(Some(parameter.coercibleTypes()[genericIndex as usize]));
        }
        let ty = self.returnType();
        if ty.isGeneric() {
            if genericIndex == -1 {
                return false;
            }
            *outReturnType = Some(ty.coercibleTypes()[genericIndex as usize]);
        } else {
            *outReturnType = Some(core::mem::transmute(NonNull::from(ty)));
        }
        true
    }
}

impl super::SkSLIRNode::IRNode for FunctionDeclaration {
    fn node(&self) -> &super::SkSLIRNode::IRNodeData {
        &self.base.base
    }
    fn node_mut(
        self: core::pin::Pin<&mut Self>,
    ) -> core::pin::Pin<&mut super::SkSLIRNode::IRNodeData> {
        unsafe { self.map_unchecked_mut(|v| &mut v.base.base) }
    }
    fn description(&self) -> Vec<u8> {
        FunctionDeclaration::description(self)
    }
}
unsafe impl super::SkSLSymbol::Symbol for FunctionDeclaration {
    unsafe fn as_type_symbol(&self) -> &dyn Type {
        panic!("invalid native Type cast")
    }
    fn symbol_data(&self) -> &SymbolData {
        &self.base
    }
    fn symbol_data_mut(self: core::pin::Pin<&mut Self>) -> core::pin::Pin<&mut SymbolData> {
        unsafe { self.map_unchecked_mut(|v| &mut v.base) }
    }
}
