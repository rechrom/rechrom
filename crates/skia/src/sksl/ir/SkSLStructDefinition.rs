// UNVERIFIED: no compilation, tests, syntax or source checks were run.
use super::{
    SkSLIRNode::*,
    SkSLProgramElement::{ProgramElement, ProgramElementData},
    SkSLSymbol::Symbol,
    SkSLType::{Field, Type},
};
use crate::include::private::SkTArray::TArray;
use crate::src::sksl::{SkSLContext::Context, SkSLPosition::Position};
use core::{pin::Pin, ptr::NonNull};
// Source: src/sksl/ir/SkSLStructDefinition.h:33-35
// Source: src/sksl/ir/SkSLStructDefinition.h:55
#[repr(C)]
pub struct StructDefinition {
    base: ProgramElementData,
    fType: NonNull<dyn Type>,
}
impl IRNodeType for StructDefinition {
    const kIRNodeKind: i32 = ProgramElementKind::kStructDefinition as i32;
}
impl StructDefinition {
    // Source: src/sksl/ir/SkSLStructDefinition.h:37-39
    /// Safety: type remains owned by its native symbol table for this node's lifetime.
    pub unsafe fn new(pos: Position, ty: &dyn Type) -> Self {
        Self {
            base: ProgramElementData::new(pos, ProgramElementKind::kStructDefinition),
            fType: core::mem::transmute::<NonNull<dyn Type + '_>, NonNull<dyn Type>>(
                NonNull::from(ty),
            ),
        }
    }
    // Source: src/sksl/ir/SkSLStructDefinition.h:48-50
    pub fn r#type(&self) -> &dyn Type {
        unsafe { self.fType.as_ref() }
    }
    // Source: src/sksl/ir/SkSLStructDefinition.cpp:23-32
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        name: &[u8],
        fields: TArray<Field, true>,
    ) -> IRNodeBox<Self> {
        let owner = <dyn Type>::MakeStructType(context, pos, name, fields, false);
        let ty = NonNull::new_unchecked(owner.as_ptr() as *mut dyn Type);
        let mut table = context.fSymbolTable.unwrap_unchecked();
        table
            .as_mut()
            .add(context, owner.cast(|p| p as *mut dyn Symbol));
        Self::Make(pos, ty.as_ref())
    }
    // Source: src/sksl/ir/SkSLStructDefinition.cpp:34-36
    pub unsafe fn Make(pos: Position, ty: &dyn Type) -> IRNodeBox<Self> {
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, ty)))
    }
}
unsafe impl ProgramElement for StructDefinition {}
impl IRNode for StructDefinition {
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base.base) }
    }
    // Source: src/sksl/ir/SkSLStructDefinition.cpp:38-53
    fn description(&self) -> Vec<u8> {
        let mut s = b"struct ".to_vec();
        s.extend_from_slice(self.r#type().name());
        s.extend_from_slice(b" { ");
        for f in self.r#type().fields() {
            unsafe {
                s.extend_from_slice(&f.fLayout.description());
                s.extend_from_slice(&f.fModifierFlags.description());
                s.push(b' ');
                s.extend_from_slice(&f.fType.unwrap_unchecked().as_ref().description());
                s.push(b' ');
                s.extend_from_slice(f.fName.as_ref());
                s.extend_from_slice(b"; ");
            }
        }
        s.extend_from_slice(b"};");
        s
    }
}
