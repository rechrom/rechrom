// Pinned upstream; no validation executed.
use super::{
    SkSLIRNode::*,
    SkSLProgramElement::{ProgramElement, ProgramElementData},
};
use crate::src::sksl::{
    SkSLContext::Context, SkSLErrorReporter::ErrorReporterExt, SkSLPosition::Position,
    SkSLProgramSettings::ProgramConfig,
};
use core::{pin::Pin, ptr::NonNull};
// Source: src/sksl/ir/SkSLExtension.h:26-28
// Source: src/sksl/ir/SkSLExtension.h:54-58
#[repr(C)]
pub struct Extension {
    base: ProgramElementData,
    fName: NonNull<[u8]>,
}
impl IRNodeType for Extension {
    const kIRNodeKind: i32 = ProgramElementKind::kExtension as i32;
}
impl Extension {
    // Source: src/sksl/ir/SkSLExtension.h:30-32
    /// Safety: name borrows the original immutable program/module source bytes.
    /// That source owner must remain alive until this element is destroyed.
    pub unsafe fn new(pos: Position, name: &[u8]) -> Self {
        Self {
            base: ProgramElementData::new(pos, ProgramElementKind::kExtension),
            fName: NonNull::from(name),
        }
    }
    // Source: src/sksl/ir/SkSLExtension.h:34-36
    pub fn name(&self) -> &[u8] {
        unsafe { self.fName.as_ref() }
    }
    // Source: src/sksl/ir/SkSLExtension.h:40-43
    // Source: src/sksl/ir/SkSLExtension.cpp:17-36
    /// Safety: name retains new()'s original source-byte lease.
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        name: &[u8],
        behaviorText: &[u8],
    ) -> Option<IRNodeBox<Self>> {
        if ProgramConfig::IsRuntimeEffect(context.fConfig.unwrap_unchecked().as_ref().fKind) {
            let mut errors = context.fErrors;
            errors
                .as_mut()
                .error(pos, b"unsupported directive '#extension'");
            return None;
        }
        if behaviorText == b"disable" {
            return None;
        }
        if behaviorText != b"require" && behaviorText != b"enable" && behaviorText != b"warn" {
            let mut errors = context.fErrors;
            errors
                .as_mut()
                .error(pos, b"expected 'require', 'enable', 'warn', or 'disable'");
            return None;
        }
        Some(Self::Make(context, pos, name))
    }
    // Source: src/sksl/ir/SkSLExtension.h:46-48
    // Source: src/sksl/ir/SkSLExtension.cpp:38-43
    /// Safety: name retains new()'s original source-byte lease.
    pub unsafe fn Make(context: &Context, pos: Position, name: &[u8]) -> IRNodeBox<Self> {
        debug_assert!(!ProgramConfig::IsRuntimeEffect(
            context.fConfig.unwrap_unchecked().as_ref().fKind
        ));
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, name)))
    }
}
unsafe impl ProgramElement for Extension {}
impl IRNode for Extension {
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base.base) }
    }
    // Source: src/sksl/ir/SkSLExtension.h:50-52
    fn description(&self) -> Vec<u8> {
        let mut text = b"#extension ".to_vec();
        text.extend_from_slice(self.name());
        text.extend_from_slice(b" : enable");
        text
    }
}
