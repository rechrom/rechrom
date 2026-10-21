// Source: src/sksl/ir/SkSLSymbol.h:20-20
pub use crate::src::sksl::SkSLContext::Context;
// Source: src/sksl/ir/SkSLSymbol.h:21-21
pub use super::SkSLExpression::Expression;
// Source: src/sksl/ir/SkSLSymbol.h:22-22
pub use super::SkSLType::Type;
// v58 additions are UNVERIFIED; user prohibited all validation.
use super::SkSLIRNode::{IRNode, IRNodeData, SymbolKind};
use crate::src::sksl::SkSLPosition::Position;
use core::ptr::NonNull;
// Source: src/sksl/ir/SkSLSymbol.h:66-66
// Native private base alias is hoisted to module scope. Its stored subobject
// maps to actual IRNodeData; the abstract interface is the existing IRNode trait.
type INHERITED = IRNodeData;
// Source: src/sksl/ir/SkSLSymbol.h:27-27
// Source: src/sksl/ir/SkSLSymbol.h:63-64
// Adapted inheritance/borrowed string_view and Type pointer. Neither field owns
// name or Type; their backing must outlive every access to this Symbol.
#[repr(C)]
pub struct SymbolData {
    pub(crate) base: INHERITED,
    fName: NonNull<[u8]>,
    fType: Option<NonNull<dyn Type>>,
}
// Source: src/sksl/ir/SkSLSymbol.h:29-29
pub type Kind = SymbolKind;
impl SymbolData {
    // Uncredited inheritance projection for native final subclasses.
    pub(crate) fn name_bytes(&self) -> &[u8] {
        unsafe { self.fName.as_ref() }
    }
    pub(crate) unsafe fn set_name_bytes(&mut self, name: &[u8]) {
        self.fName = NonNull::from(name);
    }
    // Source: src/sksl/ir/SkSLSymbol.h:31-36
    /// Safety: native borrowed name and optional Type backing remain live for the
    /// Symbol lifetime; pointed-to Type is a valid actual polymorphic object.
    pub unsafe fn new(
        pos: Position,
        kind: Kind,
        name: &[u8],
        ty: Option<NonNull<dyn Type>>,
    ) -> Self {
        let value = Self {
            base: IRNodeData::new(pos, kind as i32),
            fName: NonNull::from(name),
            fType: ty,
        };
        debug_assert!(kind as i32 >= Kind::kFirst as i32 && kind as i32 <= Kind::kLast as i32);
        value
    }
}
// Source: src/sksl/ir/SkSLSymbol.h:38-38
// Actual dynamic destructor comes from the concrete IRNode owner.
/// Safety: base kind stays in SymbolKind range; projections reference this
/// object's actual native inheritance subobject, never a separate owner. Native
/// nonvirtual methods must not be overridden by a concrete Rust implementation.
pub unsafe trait Symbol: IRNode {
    // Language-only checked Type inheritance projection retains the actual subtype vtable.
    unsafe fn as_type_symbol(&self) -> &dyn Type;
    // Language-only native checked downcast. Every actual Variable subclass overrides this
    // with its real vtable; the default has only the invalid non-Variable cast domain.
    unsafe fn as_variable_symbol(&self) -> &dyn super::SkSLVariable::Variable {
        debug_assert!(self.kind() == Kind::kVariable);
        core::hint::unreachable_unchecked()
    }
    fn symbol_data(&self) -> &SymbolData;
    fn symbol_data_mut(self: core::pin::Pin<&mut Self>) -> core::pin::Pin<&mut SymbolData>;
    // Source: src/sksl/ir/SkSLSymbol.h:40-40
    unsafe fn instantiate(
        &self,
        context: &crate::src::sksl::SkSLContext::Context,
        pos: Position,
    ) -> Option<super::SkSLIRNode::IRNodeBox<dyn super::SkSLExpression::Expression>>
    where
        Self: Sized,
    {
        instantiate(self, context, pos)
    }
    // Source: src/sksl/ir/SkSLSymbol.h:42-45
    fn symbol_type(&self) -> &dyn Type {
        let p = self.symbol_data().fType;
        debug_assert!(p.is_some());
        unsafe { p.unwrap_unchecked().as_ref() }
    }
    // Source: src/sksl/ir/SkSLSymbol.h:47-49
    fn kind(&self) -> Kind {
        unsafe { core::mem::transmute(self.node().fKind) }
    }
    // Source: src/sksl/ir/SkSLSymbol.h:51-53
    fn name(&self) -> &[u8] {
        unsafe { self.symbol_data().fName.as_ref() }
    }
    // Source: src/sksl/ir/SkSLSymbol.h:58-60
    /// Safety: caller obeys SymbolTable rename/invalidation protocol and keeps
    /// newName backing live through the remaining Symbol lifetime.
    unsafe fn setName(self: core::pin::Pin<&mut Self>, newName: &[u8]) {
        self.symbol_data_mut().get_unchecked_mut().fName = NonNull::from(newName);
    }
}

// Source: src/sksl/ir/SkSLSymbol.cpp:24-52
pub unsafe fn instantiate(
    symbol: &dyn Symbol,
    context: &crate::src::sksl::SkSLContext::Context,
    pos: Position,
) -> Option<super::SkSLIRNode::IRNodeBox<dyn super::SkSLExpression::Expression>> {
    use super::{
        SkSLExpression::Expression,
        SkSLFieldAccess::{FieldAccess, OwnerKind},
        SkSLFieldSymbol::FieldSymbol,
        SkSLFunctionDeclaration::FunctionDeclaration,
        SkSLFunctionReference::FunctionReference,
        SkSLIRNode::{IRNodeBox, SymbolKind},
        SkSLTypeReference::TypeReference,
        SkSLVariableReference::{VariableRefKind, VariableReference},
    };
    match symbol.kind() {
        SymbolKind::kFunctionDeclaration => {
            let f = NonNull::from(symbol.node().as_node::<FunctionDeclaration>());
            Some(
                IRNodeBox::new_with(|p: *mut FunctionReference| {
                    p.write(FunctionReference::new(context, pos, Some(f)))
                })
                .cast(|p| p as *mut dyn Expression),
            )
        }
        SymbolKind::kVariable => {
            let var = symbol.as_variable_symbol();
            let var = core::mem::transmute::<
                NonNull<dyn super::SkSLVariable::Variable + '_>,
                NonNull<dyn super::SkSLVariable::Variable>,
            >(NonNull::from(var));
            Some(VariableReference::Make(pos, var, VariableRefKind::kRead))
        }
        SymbolKind::kField => {
            let field = symbol.node().as_node::<FieldSymbol>();
            let owner = core::mem::transmute::<
                NonNull<dyn super::SkSLVariable::Variable + '_>,
                NonNull<dyn super::SkSLVariable::Variable>,
            >(NonNull::from(field.owner()));
            let base = VariableReference::Make(pos, owner, VariableRefKind::kRead);
            Some(FieldAccess::Make(
                context,
                pos,
                base,
                field.fieldIndex(),
                OwnerKind::kAnonymousInterfaceBlock,
            ))
        }
        SymbolKind::kType => {
            let ty = core::mem::transmute::<NonNull<dyn Type + '_>, NonNull<dyn Type>>(
                NonNull::from(symbol.as_type_symbol()),
            );
            TypeReference::Convert(context, pos, ty).map(|p| p.cast(|p| p as *mut dyn Expression))
        }
        _ => {
            debug_assert!(false, "unsupported symbol type {}", symbol.node().fKind);
            None
        }
    }
}
