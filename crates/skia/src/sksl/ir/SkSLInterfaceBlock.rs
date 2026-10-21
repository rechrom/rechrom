use super::{
    SkSLIRNode::*,
    SkSLProgramElement::{ProgramElement, ProgramElementData},
    SkSLVariable::Variable,
};
use crate::src::sksl::SkSLPosition::Position;
use core::{pin::Pin, ptr::NonNull};
// Source: src/sksl/ir/SkSLInterfaceBlock.h:38-38
// Source: src/sksl/ir/SkSLInterfaceBlock.h:95-95
#[repr(C)]
pub struct InterfaceBlock {
    base: ProgramElementData,
    fVariable: Option<NonNull<dyn Variable>>,
}
// Source: src/sksl/ir/SkSLInterfaceBlock.h:40-40
impl IRNodeType for InterfaceBlock {
    const kIRNodeKind: i32 = ProgramElementKind::kInterfaceBlock as i32;
}
impl InterfaceBlock {
    // Source: src/sksl/ir/SkSLInterfaceBlock.h:42-47
    pub unsafe fn new(pos: Position, mut var: NonNull<dyn Variable>) -> IRNodeBox<Self> {
        let result = IRNodeBox::new_with(|p: *mut Self| {
            p.write(Self {
                base: ProgramElementData::new(pos, ProgramElementKind::kInterfaceBlock),
                fVariable: Some(var),
            })
        });
        debug_assert!(var
            .as_ref()
            .symbol_type()
            .componentType()
            .isInterfaceBlock());
        Pin::new_unchecked(var.as_mut())
            .setInterfaceBlock(NonNull::new_unchecked(result.as_ptr() as *mut Self));
        result
    }
    // Source: src/sksl/ir/SkSLInterfaceBlock.h:72-74
    pub fn var(&self) -> Option<NonNull<dyn Variable>> {
        self.fVariable
    }
    // Source: src/sksl/ir/SkSLInterfaceBlock.h:76-78
    pub fn detachDeadVariable(&mut self) {
        self.fVariable = None;
    }
    // Source: src/sksl/ir/SkSLInterfaceBlock.h:80-82
    pub fn typeName(&self) -> &[u8] {
        unsafe { self.fVariable.unwrap_unchecked().as_ref() }
            .symbol_type()
            .componentType()
            .name()
    }
    // Source: src/sksl/ir/SkSLInterfaceBlock.h:84-86
    pub fn instanceName(&self) -> &[u8] {
        unsafe { self.fVariable.unwrap_unchecked().as_ref() }.name()
    }
    // Source: src/sksl/ir/SkSLInterfaceBlock.h:88-90
    pub fn arraySize(&self) -> i32 {
        let ty = unsafe { self.fVariable.unwrap_unchecked().as_ref() }.symbol_type();
        if ty.isArray() {
            ty.columns()
        } else {
            0
        }
    }
}
// Source: src/sksl/ir/SkSLInterfaceBlock.cpp:34-39
impl Drop for InterfaceBlock {
    fn drop(&mut self) {
        if let Some(mut var) = self.fVariable {
            unsafe {
                Pin::new_unchecked(var.as_mut()).detachDeadInterfaceBlock();
            }
        }
    }
}
// Source: src/sksl/ir/SkSLInterfaceBlock.cpp:41-51
pub unsafe fn find_rt_adjust_index(fields: &[super::SkSLType::Field]) -> Option<i32> {
    for (index, field) in fields.iter().enumerate() {
        let native = &crate::src::sksl::SkSLCompiler::RTADJUST_NAME;
        if field.fName.as_ref() == &native[..native.len() - 1] {
            return Some(index as i32);
        }
    }
    None
}
impl InterfaceBlock {
    // Source: src/sksl/ir/SkSLInterfaceBlock.cpp:52-114
    // Adapted polymorphic unique_ptr owners retain the same allocation and symbol-table lease.
    pub unsafe fn Convert(
        context: &crate::src::sksl::SkSLContext::Context,
        pos: Position,
        modifiers: &super::SkSLModifiers::Modifiers,
        typeName: &[u8],
        fields: crate::include::private::SkTArray::TArray<super::SkSLType::Field, true>,
        varName: &[u8],
        mut arraySize: i32,
    ) -> Option<IRNodeBox<Self>> {
        use super::{
            SkSLSymbol::Symbol,
            SkSLType::Type,
            SkSLVariable::{Variable, VariableStorage},
        };
        use crate::src::sksl::{
            SkSLErrorReporter::ErrorReporterExt, SkSLProgramSettings::ProgramConfig,
        };
        let kind = context.fConfig.unwrap_unchecked().as_ref().fKind;
        if !ProgramConfig::IsFragment(kind)
            && !ProgramConfig::IsVertex(kind)
            && !ProgramConfig::IsCompute(kind)
        {
            let mut errors = context.fErrors;
            errors.as_mut().error(
                pos,
                b"interface blocks are not allowed in this kind of program",
            );
            return None;
        }
        if let Some(index) = find_rt_adjust_index(fields.as_slice()) {
            let f = &fields.as_slice()[index as usize];
            if !f
                .fType
                .unwrap_unchecked()
                .as_ref()
                .matches(&**context.fTypes.as_ref().fFloat4)
            {
                let mut errors = context.fErrors;
                errors
                    .as_mut()
                    .error(f.fPosition, b"sk_RTAdjust must have type 'float4'");
                return None;
            }
        }
        let owner = <dyn Type>::MakeStructType(context, pos, typeName, fields, true);
        let baseType = NonNull::new_unchecked(owner.as_ptr() as *mut dyn Type);
        let mut table = context.fSymbolTable.unwrap_unchecked();
        table
            .as_mut()
            .add(context, owner.cast(|p| p as *mut dyn Symbol));
        let mut ty = baseType;
        if arraySize > 0 {
            arraySize = super::SkSLType::convertArraySizeValue(
                ty.as_ref(),
                context,
                pos,
                pos,
                arraySize as i64,
            ) as i32;
            if arraySize == 0 {
                return None;
            }
            ty = table.as_mut().addArrayDimension(context, ty, arraySize);
        }
        super::SkSLVarDeclarations::VarDeclaration::ErrorCheck(
            context,
            pos,
            modifiers.fPosition,
            &modifiers.fLayout,
            modifiers.fFlags,
            ty.as_ref(),
            baseType.as_ref(),
            VariableStorage::kGlobal,
        );
        let owner = <dyn Variable>::Convert(
            context,
            pos,
            modifiers.fPosition,
            &modifiers.fLayout,
            modifiers.fFlags,
            ty,
            pos,
            varName,
            VariableStorage::kGlobal,
        );
        let var = NonNull::new_unchecked(owner.as_ptr() as *mut dyn Variable);
        table
            .as_mut()
            .takeOwnershipOfSymbol(Some(owner.cast(|p| p as *mut dyn Symbol)));
        Some(Self::Make(context, pos, var))
    }
    // Source: src/sksl/ir/SkSLInterfaceBlock.cpp:116-138
    pub unsafe fn Make(
        context: &crate::src::sksl::SkSLContext::Context,
        pos: Position,
        variable: NonNull<dyn super::SkSLVariable::Variable>,
    ) -> IRNodeBox<Self> {
        use super::{SkSLFieldSymbol::FieldSymbol, SkSLSymbol::Symbol};
        use crate::src::sksl::SkSLProgramSettings::ProgramConfig;
        let kind = context.fConfig.unwrap_unchecked().as_ref().fKind;
        debug_assert!(
            ProgramConfig::IsFragment(kind)
                || ProgramConfig::IsVertex(kind)
                || ProgramConfig::IsCompute(kind)
        );
        debug_assert!(variable
            .as_ref()
            .symbol_type()
            .componentType()
            .isInterfaceBlock());
        let fields = variable.as_ref().symbol_type().componentType().fields();
        let mut table = context.fSymbolTable.unwrap_unchecked();
        if variable.as_ref().name().is_empty() {
            for (i, f) in fields.iter().enumerate() {
                let owner = IRNodeBox::new_with(|p: *mut FieldSymbol| {
                    p.write(FieldSymbol::new(f.fPosition, variable, i as i32))
                });
                table
                    .as_mut()
                    .add(context, owner.cast(|p| p as *mut dyn Symbol));
            }
        } else {
            table.as_mut().addWithoutOwnershipContext(
                context,
                NonNull::new_unchecked(variable.as_ptr() as *mut dyn Symbol),
            );
        }
        Self::new(pos, variable)
    }
}
impl IRNode for InterfaceBlock {
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|v| &mut v.base.base) }
    }
    // Source: src/sksl/ir/SkSLInterfaceBlock.cpp:140-159
    fn description(&self) -> Vec<u8> {
        unsafe {
            let var = self.fVariable.unwrap_unchecked().as_ref();
            let mut result = var.layout().description();
            result.extend_from_slice(&var.modifierFlags().description());
            result.push(b' ');
            result.extend_from_slice(self.typeName());
            result.extend_from_slice(b" {\n");
            let mut ty = var.symbol_type();
            if ty.isArray() {
                ty = ty.componentType();
            }
            for f in ty.fields() {
                result.extend_from_slice(&f.description());
                result.push(b'\n');
            }
            result.push(b'}');
            if !self.instanceName().is_empty() {
                result.push(b' ');
                result.extend_from_slice(self.instanceName());
                if self.arraySize() > 0 {
                    result.push(b'[');
                    result.extend_from_slice(self.arraySize().to_string().as_bytes());
                    result.push(b']');
                }
            }
            result.push(b';');
            result
        }
    }
}
unsafe impl ProgramElement for InterfaceBlock {}
