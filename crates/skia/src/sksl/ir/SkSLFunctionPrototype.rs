// Pinned native source; unverified as requested.
use super::{
    SkSLFunctionDeclaration::FunctionDeclaration,
    SkSLIRNode::*,
    SkSLProgramElement::{ProgramElement, ProgramElementData},
};
use crate::src::sksl::SkSLPosition::Position;
use core::{pin::Pin, ptr::NonNull};
// Source: src/sksl/ir/SkSLFunctionPrototype.h:20-22
// Source: src/sksl/ir/SkSLFunctionPrototype.h:36-40
#[repr(C)]
pub struct FunctionPrototype {
    base: ProgramElementData,
    fDeclaration: NonNull<FunctionDeclaration>,
}
impl IRNodeType for FunctionPrototype {
    const kIRNodeKind: i32 = ProgramElementKind::kFunctionPrototype as i32;
}
impl FunctionPrototype {
    // Source: src/sksl/ir/SkSLFunctionPrototype.h:24-26
    /// Safety: declaration is the actual symbol-table-owned declaration, retained
    /// through this program element's destruction by the original Program owner.
    pub unsafe fn new(pos: Position, declaration: NonNull<FunctionDeclaration>) -> Self {
        Self {
            base: ProgramElementData::new(pos, ProgramElementKind::kFunctionPrototype),
            fDeclaration: declaration,
        }
    }
    // Source: src/sksl/ir/SkSLFunctionPrototype.h:28-30
    pub fn declaration(&self) -> &FunctionDeclaration {
        unsafe { self.fDeclaration.as_ref() }
    }
}
unsafe impl ProgramElement for FunctionPrototype {}
impl IRNode for FunctionPrototype {
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base.base) }
    }
    // Source: src/sksl/ir/SkSLFunctionPrototype.h:32-34
    fn description(&self) -> Vec<u8> {
        let mut text = self.declaration().description();
        text.push(b';');
        text
    }
}
