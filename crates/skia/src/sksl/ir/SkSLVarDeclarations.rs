use super::{
    SkSLExpression::Expression,
    SkSLIRNode::*,
    SkSLProgramElement::{ProgramElement, ProgramElementData},
    SkSLStatement::{Statement, StatementData},
    SkSLType::Type,
    SkSLVariable::Variable,
};
use crate::src::sksl::SkSLPosition::Position;
use core::{pin::Pin, ptr::NonNull};
// Source: src/sksl/ir/SkSLVarDeclarations.h:37-37
// Source: src/sksl/ir/SkSLVarDeclarations.h:122-125
#[repr(C)]
pub struct VarDeclaration {
    base: StatementData,
    fVar: Option<NonNull<dyn Variable>>,
    fBaseType: NonNull<dyn Type>,
    fArraySize: i32,
    fValue: Option<IRNodeBox<dyn Expression>>,
}
// Source: src/sksl/ir/SkSLVarDeclarations.h:39-39
impl IRNodeType for VarDeclaration {
    const kIRNodeKind: i32 = StatementKind::kVarDeclaration as i32;
}
impl VarDeclaration {
    // Source: src/sksl/ir/SkSLVarDeclarations.h:41-49
    pub unsafe fn new(
        var: NonNull<dyn Variable>,
        baseType: NonNull<dyn Type>,
        arraySize: i32,
        value: Option<IRNodeBox<dyn Expression>>,
    ) -> Self {
        Self {
            base: StatementData::new(
                var.as_ref().node().fPosition,
                StatementKind::kVarDeclaration,
            ),
            fVar: Some(var),
            fBaseType: baseType,
            fArraySize: arraySize,
            fValue: value,
        }
    }
}
// Source: src/sksl/ir/SkSLVarDeclarations.h:51-56
impl Drop for VarDeclaration {
    fn drop(&mut self) {
        if let Some(mut var) = self.fVar {
            unsafe {
                Pin::new_unchecked(var.as_mut()).detachDeadVarDeclaration();
            }
        }
    }
}
impl VarDeclaration {
    // Source: src/sksl/ir/SkSLVarDeclarations.h:90-92
    pub fn baseType(&self) -> &dyn Type {
        unsafe { self.fBaseType.as_ref() }
    }
    // Source: src/sksl/ir/SkSLVarDeclarations.h:94-96
    pub fn var(&self) -> Option<NonNull<dyn Variable>> {
        self.fVar
    }
    // Source: src/sksl/ir/SkSLVarDeclarations.h:98-100
    pub fn detachDeadVariable(&mut self) {
        self.fVar = None;
    }
    // Source: src/sksl/ir/SkSLVarDeclarations.h:102-104
    pub fn arraySize(&self) -> i32 {
        self.fArraySize
    }
    // Source: src/sksl/ir/SkSLVarDeclarations.h:106-108
    pub fn value_mut(&mut self) -> &mut Option<IRNodeBox<dyn Expression>> {
        &mut self.fValue
    }
    // Source: src/sksl/ir/SkSLVarDeclarations.h:110-112
    pub fn value(&self) -> &Option<IRNodeBox<dyn Expression>> {
        &self.fValue
    }
}
// Source: src/sksl/ir/SkSLVarDeclarations.cpp:33-78
unsafe fn check_valid_uniform_type(
    pos: Position,
    t: &dyn Type,
    context: &crate::src::sksl::SkSLContext::Context,
    _topLevel: bool,
) -> bool {
    use crate::src::sksl::{
        SkSLErrorReporter::ErrorReporterExt, SkSLProgramSettings::ProgramConfig,
    };
    let report = || {
        let mut message = b"variables of type '".to_vec();
        message.extend_from_slice(&t.displayName());
        message.extend_from_slice(b"' may not be uniform");
        let mut errors = context.fErrors;
        errors.as_mut().error(pos, &message);
    };
    if ProgramConfig::IsRuntimeEffect(context.fConfig.unwrap_unchecked().as_ref().fKind) {
        if t.isEffectChild() {
            return true;
        }
        let ct = t.componentType();
        if ct.isSigned() && ct.bitWidth() == 32 && (t.isScalar() || t.isVector()) {
            return true;
        }
        if ct.isFloat()
            && (t.isScalar() || t.isVector() || (t.isMatrix() && t.rows() == t.columns()))
        {
            return true;
        }
        report();
        return false;
    }
    let mut errorPosition = Position::default();
    if !t.isAllowedInUniform(Some(&mut errorPosition)) {
        report();
        if errorPosition.valid() {
            let mut errors = context.fErrors;
            errors.as_mut().error(errorPosition, b"caused by:");
        }
        return false;
    }
    true
}
impl IRNode for VarDeclaration {
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|v| &mut v.base.base) }
    }
    // Source: src/sksl/ir/SkSLVarDeclarations.cpp:82-94
    fn description(&self) -> Vec<u8> {
        unsafe {
            let var = self.fVar.unwrap_unchecked().as_ref();
            let mut result = var.layout().paddedDescription();
            result.extend_from_slice(&var.modifierFlags().paddedDescription());
            result.extend_from_slice(&self.baseType().description());
            result.push(b' ');
            result.extend_from_slice(var.name());
            if self.arraySize() > 0 {
                result.push(b'[');
                result.extend_from_slice(self.arraySize().to_string().as_bytes());
                result.push(b']');
            }
            if let Some(value) = self.value() {
                result.extend_from_slice(b" = ");
                result.extend_from_slice(&value.description());
            }
            result.push(b';');
            result
        }
    }
}
unsafe impl Statement for VarDeclaration {}
// Source: src/sksl/ir/SkSLVarDeclarations.h:134-134
// Source: src/sksl/ir/SkSLVarDeclarations.h:166-166
#[repr(C)]
pub struct GlobalVarDeclaration {
    base: ProgramElementData,
    // Preserve the native replaceable unique_ptr slot for ProgramWriter.
    fDeclaration: Option<IRNodeBox<dyn Statement>>,
}
// Source: src/sksl/ir/SkSLVarDeclarations.h:136-136
impl IRNodeType for GlobalVarDeclaration {
    const kIRNodeKind: i32 = ProgramElementKind::kGlobalVar as i32;
}
impl GlobalVarDeclaration {
    // Source: src/sksl/ir/SkSLVarDeclarations.h:138-143
    // Adapted stable-address construction retains the Variable backlink before publication.
    pub unsafe fn new(decl: IRNodeBox<dyn Statement>) -> IRNodeBox<Self> {
        let result = IRNodeBox::new_with(|p: *mut Self| {
            p.write(Self {
                base: ProgramElementData::new(
                    decl.node().fPosition,
                    ProgramElementKind::kGlobalVar,
                ),
                fDeclaration: Some(decl),
            })
        });
        debug_assert!(result.declaration().node().is::<VarDeclaration>());
        let mut var = result.varDeclaration().var().unwrap_unchecked();
        Pin::new_unchecked(var.as_mut())
            .setGlobalVarDeclaration(NonNull::new_unchecked(result.as_ptr() as *mut Self));
        result
    }
    // Source: src/sksl/ir/SkSLVarDeclarations.h:145-147
    pub fn declaration_mut(&mut self) -> &mut IRNodeBox<dyn Statement> {
        self.fDeclaration
            .as_mut()
            .expect("native nonnull global declaration")
    }
    // Source: src/sksl/ir/SkSLVarDeclarations.h:145-147
    // Same native getter, exposing its nullable unique_ptr for mutable traversal.
    pub fn declaration_slot_mut(&mut self) -> &mut Option<IRNodeBox<dyn Statement>> {
        &mut self.fDeclaration
    }
    // Source: src/sksl/ir/SkSLVarDeclarations.h:149-151
    pub fn declaration(&self) -> &IRNodeBox<dyn Statement> {
        self.fDeclaration
            .as_ref()
            .expect("native nonnull global declaration")
    }
    // Source: src/sksl/ir/SkSLVarDeclarations.h:153-155
    pub unsafe fn varDeclaration_mut(&mut self) -> &mut VarDeclaration {
        debug_assert!(self.declaration().node().is::<VarDeclaration>());
        self.declaration_mut()
            .as_pin_mut()
            .node_mut()
            .as_node_mut::<VarDeclaration>()
            .get_unchecked_mut()
    }
    // Source: src/sksl/ir/SkSLVarDeclarations.h:157-159
    pub fn varDeclaration(&self) -> &VarDeclaration {
        debug_assert!(self.declaration().node().is::<VarDeclaration>());
        unsafe { &*(self.declaration().as_ptr() as *const () as *const VarDeclaration) }
    }
}
impl IRNode for GlobalVarDeclaration {
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|v| &mut v.base.base) }
    }
    // Source: src/sksl/ir/SkSLVarDeclarations.h:161-163
    fn description(&self) -> Vec<u8> {
        self.declaration().description()
    }
}
unsafe impl ProgramElement for GlobalVarDeclaration {}
// Source: src/sksl/ir/SkSLVarDeclarations.cpp:96-293
impl VarDeclaration {
    pub unsafe fn ErrorCheck(
        context: &crate::src::sksl::SkSLContext::Context,
        pos: Position,
        modifiersPosition: Position,
        layout: &super::SkSLLayout::Layout,
        flags: super::SkSLModifierFlags::ModifierFlags,
        ty: &dyn Type,
        base: &dyn Type,
        storage: super::SkSLVariable::Storage,
    ) {
        use super::{
            SkSLLayout::LayoutFlag as L,
            SkSLModifierFlags::{ModifierFlag as F, ModifierFlags},
            SkSLType::TypeKind,
            SkSLVariable::Storage,
        };
        use crate::src::sksl::{
            SkSLErrorReporter::ErrorReporterExt, SkSLProgramKind::ProgramKind,
            SkSLProgramSettings::ProgramConfig,
        };
        debug_assert!(if ty.isArray() {
            base.matches(ty.componentType())
        } else {
            base.matches(ty)
        });
        let config = context.fConfig.unwrap_unchecked().as_ref();
        let mut errors = context.fErrors;
        let mut type_error = |prefix: &[u8], suffix: &[u8]| {
            let mut message = prefix.to_vec();
            message.extend_from_slice(&base.displayName());
            message.extend_from_slice(suffix);
            errors.as_mut().error(pos, &message);
        };
        if base.componentType().isOpaque()
            && !base.componentType().isAtomic()
            && storage != Storage::kGlobal
        {
            type_error(b"variables of type '", b"' must be global");
        }
        if flags.0 & F::kIn.0 != 0 && base.isMatrix() {
            errors
                .as_mut()
                .error(pos, b"'in' variables may not have matrix type");
        }
        if flags.0 & F::kIn.0 != 0 && ty.isUnsizedArray() {
            errors
                .as_mut()
                .error(pos, b"'in' variables may not have unsized array type");
        }
        if flags.0 & F::kOut.0 != 0 && ty.isUnsizedArray() {
            errors
                .as_mut()
                .error(pos, b"'out' variables may not have unsized array type");
        }
        if flags.0 & F::kIn.0 != 0 && flags.isUniform() {
            errors
                .as_mut()
                .error(pos, b"'in uniform' variables not permitted");
        }
        if flags.isReadOnly() && flags.isWriteOnly() {
            errors.as_mut().error(
                pos,
                b"'readonly' and 'writeonly' qualifiers cannot be combined",
            );
        }
        if flags.isUniform() && flags.isBuffer() {
            errors
                .as_mut()
                .error(pos, b"'uniform buffer' variables not permitted");
        }
        if flags.isWorkgroup() && flags.0 & (F::kIn.0 | F::kOut.0) != 0 {
            errors
                .as_mut()
                .error(pos, b"in / out variables may not be declared workgroup");
        }
        if flags.isUniform() {
            check_valid_uniform_type(pos, base, context, true);
        }
        if base.isEffectChild() && !flags.isUniform() {
            let mut message = b"variables of type '".to_vec();
            message.extend_from_slice(&base.displayName());
            message.extend_from_slice(b"' must be uniform");
            errors.as_mut().error(pos, &message);
        }
        if base.isEffectChild() && config.fKind == ProgramKind::kMeshVertex {
            errors
                .as_mut()
                .error(pos, b"effects are not permitted in mesh vertex shaders");
        }
        if base.isOrContainsAtomic() {
            let member = storage == Storage::kInterfaceBlock;
            let writable = flags.isBuffer() && !flags.isReadOnly();
            if !flags.isWorkgroup()
                && !(if base.isInterfaceBlock() {
                    writable
                } else {
                    member
                })
            {
                errors.as_mut().error(pos,b"atomics are only permitted in workgroup variables and writable storage blocks");
            }
        }
        if layout.fFlags.0 & L::kColor.0 != 0 {
            if !ProgramConfig::IsRuntimeEffect(config.fKind) {
                errors
                    .as_mut()
                    .error(pos, b"'layout(color)' is only permitted in runtime effects");
            }
            if !flags.isUniform() {
                errors.as_mut().error(
                    pos,
                    b"'layout(color)' is only permitted on 'uniform' variables",
                );
            }
            let valid = |t: &dyn Type| {
                t.isVector()
                    && t.componentType().isFloat()
                    && (t.columns() == 3 || t.columns() == 4)
            };
            if !valid(base) {
                let mut message =
                    b"'layout(color)' is not permitted on variables of type '".to_vec();
                message.extend_from_slice(&base.displayName());
                message.push(b'\'');
                errors.as_mut().error(pos, &message);
            }
        }
        let mut permitted = F::kConst.0 | F::kHighp.0 | F::kMediump.0 | F::kLowp.0;
        if storage == Storage::kGlobal {
            permitted |= F::kUniform.0;
            if !ProgramConfig::IsRuntimeEffect(config.fKind) {
                if base.isInterfaceBlock() {
                    permitted |= F::kBuffer.0;
                    if flags.isBuffer() {
                        permitted |= F::kReadOnly.0 | F::kWriteOnly.0;
                    }
                    let fields = base.fields();
                    let end = fields.len() as i32 - if flags.isBuffer() { 1 } else { 0 };
                    for i in 0..end {
                        let field = &fields[i as usize];
                        if field.fType.unwrap_unchecked().as_ref().isUnsizedArray() {
                            errors.as_mut().error(
                                field.fPosition,
                                b"unsized array must be the last member of a storage block",
                            );
                        }
                    }
                }
                if !base.isOpaque() {
                    permitted |= F::kIn.0 | F::kOut.0;
                }
                if ProgramConfig::IsFragment(config.fKind)
                    && base.isStruct()
                    && !base.isInterfaceBlock()
                {
                    permitted |= F::kPixelLocal.0;
                }
                if ProgramConfig::IsCompute(config.fKind) {
                    if !base.isOpaque() || base.isAtomic() {
                        permitted |= F::kWorkgroup.0;
                    }
                } else {
                    permitted |= F::kFlat.0 | F::kNoPerspective.0;
                }
            }
        }
        let mut layouts = L::kAll.0;
        if base.isStorageTexture() {
            if layout.fFlags.0 & L::kAllPixelFormats.0 == 0 {
                errors
                    .as_mut()
                    .error(pos, b"storage textures must declare a pixel format");
            }
        } else {
            layouts &= !L::kAllPixelFormats.0;
        }
        match base.typeKind() {
            TypeKind::kSampler => {}
            TypeKind::kTexture => layouts &= !L::kSampler.0,
            TypeKind::kSeparateSampler => layouts &= !L::kTexture.0,
            _ => layouts &= !(L::kTexture.0 | L::kSampler.0),
        }
        let bindings = base.typeKind() == TypeKind::kSampler
            || base.typeKind() == TypeKind::kSeparateSampler
            || base.typeKind() == TypeKind::kTexture
            || base.isInterfaceBlock();
        if storage != Storage::kGlobal || (flags.isUniform() && !bindings) {
            layouts &= !L::kBinding.0;
            layouts &= !L::kSet.0;
            layouts &= !L::kAllBackends.0;
        }
        if ProgramConfig::IsRuntimeEffect(config.fKind) {
            layouts &= L::kColor.0;
        }
        if layout.fFlags.0 & (L::kSet.0 | L::kBinding.0) != 0
            || flags.0 & (F::kIn.0 | F::kOut.0) != 0
        {
            layouts &= !L::kPushConstant.0;
        }
        if !config.isBuiltinCode() {
            layouts &= !L::kBuiltin.0;
        }
        flags.checkPermittedFlags(context, modifiersPosition, ModifierFlags(permitted));
        layout.checkPermittedLayout(context, modifiersPosition, L(layouts));
    }
}

// New source-faithful bodies are unverified. IsConstantExpression and the array
// coercion dependency remain unresolved; these dependent regions stay pending.
impl VarDeclaration {
    // Source: src/sksl/ir/SkSLVarDeclarations.cpp:295-367
    pub unsafe fn ErrorCheckAndCoerce(
        context: &crate::src::sksl::SkSLContext::Context,
        var: &dyn Variable,
        baseType: &dyn Type,
        value: &mut Option<IRNodeBox<dyn Expression>>,
    ) -> bool {
        use super::{SkSLModifierFlags::ModifierFlag, SkSLVariable::Storage};
        use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
        if baseType.matches(&**context.fTypes.as_ref().fInvalid) {
            let mut errors = context.fErrors;
            errors.as_mut().error(var.node().fPosition, b"invalid type");
            return false;
        }
        if baseType.isVoid() {
            let mut errors = context.fErrors;
            errors.as_mut().error(
                var.node().fPosition,
                b"variables of type 'void' are not allowed",
            );
            return false;
        }
        Self::ErrorCheck(
            context,
            var.node().fPosition,
            var.modifiersPosition(),
            var.layout(),
            var.modifierFlags(),
            var.symbol_type(),
            baseType,
            var.storage(),
        );
        if let Some(initial) = value {
            let mut errors = context.fErrors;
            let pos = initial.node().fPosition;
            if var.symbol_type().isOpaque() || var.symbol_type().isOrContainsAtomic() {
                let mut message = b"opaque type '".to_vec();
                message.extend_from_slice(&var.symbol_type().displayName());
                message.extend_from_slice(b"' cannot use initializer expressions");
                errors.as_mut().error(pos, &message);
                return false;
            }
            if var.modifierFlags().0 & ModifierFlag::kIn.0 != 0 {
                errors
                    .as_mut()
                    .error(pos, b"'in' variables cannot use initializer expressions");
                return false;
            }
            if var.modifierFlags().isUniform() {
                errors.as_mut().error(
                    pos,
                    b"'uniform' variables cannot use initializer expressions",
                );
                return false;
            }
            if var.storage() == Storage::kInterfaceBlock {
                errors.as_mut().error(
                    pos,
                    b"initializers are not permitted on interface block fields",
                );
                return false;
            }
            if context.fConfig.unwrap_unchecked().as_ref().strictES2Mode()
                && var.symbol_type().isOrContainsArray()
            {
                errors.as_mut().error(
                    pos,
                    b"initializers are not permitted on arrays (or structs containing arrays)",
                );
                return false;
            }
            *value = super::SkSLType::coerceExpression(var.symbol_type(), value.take(), context);
            if value.is_none() {
                return false;
            }
        }
        if var.modifierFlags().isConst() {
            let Some(initial) = value else {
                let mut errors = context.fErrors;
                errors.as_mut().error(
                    var.node().fPosition,
                    b"'const' variables must be initialized",
                );
                return false;
            };
            if !crate::src::sksl::SkSLAnalysis::IsConstantExpression(&**initial) {
                let mut errors = context.fErrors;
                errors.as_mut().error(
                    initial.node().fPosition,
                    b"'const' variable initializer must be a constant expression",
                );
                return false;
            }
        }
        if var.storage() == Storage::kInterfaceBlock && var.symbol_type().isOpaque() {
            let mut message = b"opaque type '".to_vec();
            message.extend_from_slice(&var.symbol_type().displayName());
            message.extend_from_slice(b"' is not permitted in an interface block");
            let mut errors = context.fErrors;
            errors.as_mut().error(var.node().fPosition, &message);
            return false;
        }
        if var.storage() == Storage::kGlobal {
            if let Some(initial) = value {
                if !crate::src::sksl::SkSLAnalysis::IsConstantExpression(&**initial) {
                    let mut errors = context.fErrors;
                    errors.as_mut().error(
                        initial.node().fPosition,
                        b"global variable initializer must be a constant expression",
                    );
                    return false;
                }
            }
        }
        true
    }
    // Source: src/sksl/ir/SkSLVarDeclarations.cpp:369-393
    pub unsafe fn Convert(
        context: &crate::src::sksl::SkSLContext::Context,
        overallPos: Position,
        modifiers: &super::SkSLModifiers::Modifiers,
        ty: NonNull<dyn Type>,
        namePos: Position,
        name: &[u8],
        storage: super::SkSLVariable::Storage,
        value: Option<IRNodeBox<dyn Expression>>,
    ) -> Option<IRNodeBox<Self>> {
        debug_assert!(storage != super::SkSLVariable::Storage::kParameter);
        // Native null guards on Convert/Make are eliminated only because those actual producers never return null; the Rust owner type records that invariant.
        let var = <dyn Variable>::Convert(
            context,
            overallPos,
            modifiers.fPosition,
            &modifiers.fLayout,
            modifiers.fFlags,
            ty,
            namePos,
            name,
            storage,
        );
        Self::ConvertVariable(context, var, value)
    }
    // Source: src/sksl/ir/SkSLVarDeclarations.cpp:395-433
    pub unsafe fn ConvertVariable(
        context: &crate::src::sksl::SkSLContext::Context,
        var: IRNodeBox<dyn Variable>,
        mut value: Option<IRNodeBox<dyn Expression>>,
    ) -> Option<IRNodeBox<Self>> {
        use super::{SkSLSymbol::Symbol, SkSLVariable::Storage};
        use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
        let mut baseType = var.symbol_type();
        let mut arraySize = 0;
        if baseType.isArray() {
            arraySize = baseType.columns();
            baseType = baseType.componentType();
        }
        if !Self::ErrorCheckAndCoerce(context, &*var, baseType, &mut value) {
            return None;
        }
        let varPtr = NonNull::new_unchecked(var.as_ptr() as *mut dyn Variable);
        let basePtr = core::mem::transmute::<NonNull<dyn Type + '_>, NonNull<dyn Type>>(
            NonNull::from(baseType),
        );
        let varDecl = Self::Make(context, varPtr, basePtr, arraySize, value);
        if var.storage() == Storage::kGlobal || var.storage() == Storage::kInterfaceBlock {
            if context
                .fSymbolTable
                .unwrap_unchecked()
                .as_ref()
                .find(var.name())
                .is_some()
            {
                let mut message = b"symbol '".to_vec();
                message.extend_from_slice(var.name());
                message.extend_from_slice(b"' was already defined");
                let mut errors = context.fErrors;
                errors.as_mut().error(var.node().fPosition, &message);
                return None;
            }
            let rtadjust = &crate::src::sksl::SkSLCompiler::RTADJUST_NAME;
            if var.name() == &rtadjust[..rtadjust.len() - 1] {
                if !var
                    .symbol_type()
                    .matches(&**context.fTypes.as_ref().fFloat4)
                {
                    let mut errors = context.fErrors;
                    errors
                        .as_mut()
                        .error(var.node().fPosition, b"sk_RTAdjust must have type 'float4'");
                    return None;
                }
            }
        }
        let mut table = context.fSymbolTable.unwrap_unchecked();
        table
            .as_mut()
            .add(context, var.cast(|p| p as *mut dyn Symbol));
        Some(varDecl)
    }
    // Source: src/sksl/ir/SkSLVarDeclarations.cpp:435-466
    pub unsafe fn Make(
        context: &crate::src::sksl::SkSLContext::Context,
        mut var: NonNull<dyn Variable>,
        baseType: NonNull<dyn Type>,
        arraySize: i32,
        value: Option<IRNodeBox<dyn Expression>>,
    ) -> IRNodeBox<Self> {
        use super::{SkSLModifierFlags::ModifierFlag, SkSLVariable::Storage};
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLVarDeclarations.cpp\0".as_ptr(),440; @condition b"!baseType->isArray()\0".as_ptr(); !baseType.as_ref().isArray());
        debug_assert!(var.as_ref().storage() != Storage::kParameter);
        debug_assert!(!var.as_ref().modifierFlags().isConst() || value.is_some());
        debug_assert!(
            !var.as_ref().modifierFlags().isConst()
                || crate::src::sksl::SkSLAnalysis::IsConstantExpression(
                    &**value.as_ref().unwrap_unchecked()
                )
        );
        debug_assert!(
            !(value.is_some()
                && var.as_ref().storage() == Storage::kGlobal
                && !crate::src::sksl::SkSLAnalysis::IsConstantExpression(
                    &**value.as_ref().unwrap_unchecked()
                ))
        );
        debug_assert!(
            !(var.as_ref().storage() == Storage::kInterfaceBlock
                && var.as_ref().symbol_type().isOpaque())
        );
        debug_assert!(!(var.as_ref().storage() == Storage::kInterfaceBlock && value.is_some()));
        debug_assert!(!(value.is_some() && var.as_ref().symbol_type().isOpaque()));
        debug_assert!(
            !(value.is_some() && var.as_ref().modifierFlags().0 & ModifierFlag::kIn.0 != 0)
        );
        debug_assert!(!(value.is_some() && var.as_ref().modifierFlags().isUniform()));
        debug_assert!(
            !(value.is_some()
                && var.as_ref().symbol_type().isOrContainsArray()
                && context.fConfig.unwrap_unchecked().as_ref().strictES2Mode())
        );
        let result =
            IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(var, baseType, arraySize, value)));
        Pin::new_unchecked(var.as_mut())
            .setVarDeclaration(NonNull::new_unchecked(result.as_ptr() as *mut Self));
        result
    }
}
