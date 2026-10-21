// Pinned native source; unverified as requested.
use super::{
    SkSLIRNode::*,
    SkSLLayout::Layout,
    SkSLModifierFlags::{ModifierFlag, ModifierFlags},
    SkSLModifiers::Modifiers,
    SkSLProgramElement::{ProgramElement, ProgramElementData},
};
use crate::src::sksl::{
    SkSLContext::Context, SkSLErrorReporter::ErrorReporterExt, SkSLPosition::Position,
    SkSLProgramSettings::ProgramConfig,
};
use core::pin::Pin;
// Source: src/sksl/ir/SkSLModifiersDeclaration.h:30-32
// Source: src/sksl/ir/SkSLModifiersDeclaration.h:57-62
#[repr(C)]
pub struct ModifiersDeclaration {
    base: ProgramElementData,
    fLayout: Layout,
    fFlags: ModifierFlags,
}
impl IRNodeType for ModifiersDeclaration {
    const kIRNodeKind: i32 = ProgramElementKind::kModifiers as i32;
}
impl ModifiersDeclaration {
    // Source: src/sksl/ir/SkSLModifiersDeclaration.h:34-37
    pub fn new(pos: Position, layout: Layout, flags: ModifierFlags) -> Self {
        Self {
            base: ProgramElementData::new(pos, ProgramElementKind::kModifiers),
            fLayout: layout,
            fFlags: flags,
        }
    }
    // Source: src/sksl/ir/SkSLModifiersDeclaration.h:39-43
    // Source: src/sksl/ir/SkSLModifiersDeclaration.cpp:23-57
    pub unsafe fn Convert(context: &Context, modifiers: &Modifiers) -> Option<IRNodeBox<Self>> {
        let kind = context.fConfig.unwrap_unchecked().as_ref().fKind;
        if !ProgramConfig::IsFragment(kind)
            && !ProgramConfig::IsVertex(kind)
            && !ProgramConfig::IsCompute(kind)
        {
            context.fErrors.as_ptr().as_mut().unwrap_unchecked().error(
                modifiers.fPosition,
                b"layout qualifiers are not allowed in this kind of program",
            );
            return None;
        }
        let layout = &modifiers.fLayout;
        if layout.fLocalSizeX >= 0 || layout.fLocalSizeY >= 0 || layout.fLocalSizeZ >= 0 {
            if layout.fLocalSizeX == 0 || layout.fLocalSizeY == 0 || layout.fLocalSizeZ == 0 {
                context
                    .fErrors
                    .as_ptr()
                    .as_mut()
                    .unwrap_unchecked()
                    .error(modifiers.fPosition, b"local size qualifiers cannot be zero");
                return None;
            }
            if !ProgramConfig::IsCompute(kind) {
                context.fErrors.as_ptr().as_mut().unwrap_unchecked().error(
                    modifiers.fPosition,
                    b"local size layout qualifiers are only allowed in a compute program",
                );
                return None;
            }
            if modifiers.fFlags.0 != ModifierFlag::kIn.0 {
                context.fErrors.as_ptr().as_mut().unwrap_unchecked().error(
                    modifiers.fPosition,
                    b"local size layout qualifiers must be defined using an 'in' declaration",
                );
                return None;
            }
        }
        Some(Self::Make(context, modifiers))
    }
    // Source: src/sksl/ir/SkSLModifiersDeclaration.cpp:59-68
    pub unsafe fn Make(context: &Context, modifiers: &Modifiers) -> IRNodeBox<Self> {
        let kind = context.fConfig.unwrap_unchecked().as_ref().fKind;
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLModifiersDeclaration.cpp\0".as_ptr(),62; @condition b"ProgramConfig::IsFragment(kind) || ProgramConfig::IsVertex(kind) || ProgramConfig::IsCompute(kind)\0".as_ptr(); ProgramConfig::IsFragment(kind)||ProgramConfig::IsVertex(kind)||ProgramConfig::IsCompute(kind));
        IRNodeBox::new_with(|p: *mut Self| {
            p.write(Self::new(
                modifiers.fPosition,
                modifiers.fLayout,
                modifiers.fFlags,
            ))
        })
    }
    // Source: src/sksl/ir/SkSLModifiersDeclaration.h:45-47
    pub fn layout(&self) -> &Layout {
        &self.fLayout
    }
    // Source: src/sksl/ir/SkSLModifiersDeclaration.h:49-51
    pub fn modifierFlags(&self) -> ModifierFlags {
        self.fFlags
    }
}
unsafe impl ProgramElement for ModifiersDeclaration {}
impl IRNode for ModifiersDeclaration {
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base.base) }
    }
    // Source: src/sksl/ir/SkSLModifiersDeclaration.h:53-55
    fn description(&self) -> Vec<u8> {
        let mut text = self.fLayout.paddedDescription();
        text.extend_from_slice(&self.fFlags.description());
        text.push(b';');
        text
    }
}
