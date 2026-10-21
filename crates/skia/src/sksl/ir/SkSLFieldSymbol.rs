use super::{
    SkSLIRNode::{IRNode, IRNodeData, IRNodeType, SymbolKind},
    SkSLSymbol::{Symbol, SymbolData},
    SkSLType::Type,
    SkSLVariable::Variable,
};
use crate::src::sksl::SkSLPosition::Position;
use core::{pin::Pin, ptr::NonNull};
// Source: src/sksl/ir/SkSLFieldSymbol.h:23-23
// Source: src/sksl/ir/SkSLFieldSymbol.h:48-49
// Adapted native inheritance and borrowed pointer; no field owns its Variable.
#[repr(C)]
pub struct FieldSymbol {
    base: SymbolData,
    fOwner: NonNull<dyn Variable>,
    fFieldIndex: i32,
}
// Source: src/sksl/ir/SkSLFieldSymbol.h:25-25
impl IRNodeType for FieldSymbol {
    const kIRNodeKind: i32 = SymbolKind::kField as i32;
}
impl FieldSymbol {
    // Source: src/sksl/ir/SkSLFieldSymbol.h:27-31
    /// Safety: owner, its Type and borrowed field names remain alive for this symbol's lifetime.
    pub unsafe fn new(pos: Position, owner: NonNull<dyn Variable>, fieldIndex: i32) -> Self {
        let field = &owner.as_ref().symbol_type().fields()[fieldIndex as usize];
        Self {
            base: SymbolData::new(pos, SymbolKind::kField, field.fName.as_ref(), field.fType),
            fOwner: owner,
            fFieldIndex: fieldIndex,
        }
    }
    // Source: src/sksl/ir/SkSLFieldSymbol.h:33-35
    pub fn fieldIndex(&self) -> i32 {
        self.fFieldIndex
    }
    // Source: src/sksl/ir/SkSLFieldSymbol.h:37-39
    pub fn owner(&self) -> &dyn Variable {
        unsafe { self.fOwner.as_ref() }
    }
}
impl IRNode for FieldSymbol {
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base.base) }
    }
    // Source: src/sksl/ir/SkSLFieldSymbol.h:41-45
    fn description(&self) -> Vec<u8> {
        if self.owner().name().is_empty() {
            self.name().to_vec()
        } else {
            let mut result = self.owner().description();
            result.push(b'.');
            result.extend_from_slice(self.name());
            result
        }
    }
}
unsafe impl Symbol for FieldSymbol {
    fn symbol_data(&self) -> &SymbolData {
        &self.base
    }
    fn symbol_data_mut(self: Pin<&mut Self>) -> Pin<&mut SymbolData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base) }
    }
    unsafe fn as_type_symbol(&self) -> &dyn Type {
        debug_assert!(false, "FieldSymbol is not Type");
        core::hint::unreachable_unchecked()
    }
}
