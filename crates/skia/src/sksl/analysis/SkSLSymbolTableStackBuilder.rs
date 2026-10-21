use crate::src::sksl::ir::{
    SkSLBlock::Block,
    SkSLForStatement::ForStatement,
    SkSLStatement::{Kind, Statement},
    SkSLSymbolTable::SymbolTable,
};
use core::ptr::NonNull;

pub use crate::src::sksl::SkSLAnalysis::SymbolTableStackBuilder;
impl SymbolTableStackBuilder {
    // Source: src/sksl/analysis/SkSLSymbolTableStackBuilder.cpp:22-44
    /// Safety: stack remains at its address until this guard drops; nested guards
    /// must drop in reverse order. Statements and symbol tables retain their
    /// native IR owner leases. Only sequential accesses to stack are permitted.
    pub unsafe fn new(
        stmt: Option<&dyn Statement>,
        mut stack: NonNull<Vec<NonNull<SymbolTable>>>,
    ) -> Self {
        let mut result = Self { fStackToPop: None };
        if let Some(stmt) = stmt {
            let symbols = match stmt.kind() {
                Kind::kBlock => stmt
                    .node()
                    .as_node::<Block>()
                    .symbolTable()
                    .map(NonNull::from),
                Kind::kFor => stmt.node().as_node::<ForStatement>().symbols(),
                _ => None,
            };
            if let Some(symbols) = symbols {
                stack.as_mut().push(symbols);
                result.fStackToPop = Some(stack);
            }
        }
        result
    }
}
// Source: src/sksl/analysis/SkSLSymbolTableStackBuilder.cpp:46-50
impl Drop for SymbolTableStackBuilder {
    fn drop(&mut self) {
        if let Some(mut stack) = self.fStackToPop {
            unsafe {
                stack
                    .as_mut()
                    .pop()
                    .expect("native nonempty symbol-table stack");
            }
        }
    }
}
