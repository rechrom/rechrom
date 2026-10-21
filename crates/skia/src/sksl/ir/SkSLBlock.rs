use crate::src::sksl::{
    ir::{
        SkSLIRNode::{IRNode, IRNodeBox, IRNodeData, IRNodeType},
        SkSLNop::Nop,
        SkSLStatement::{Kind as StatementKind, Statement, StatementData},
        SkSLSymbolTable::SymbolTable,
    },
    SkSLDefines::StatementArray,
    SkSLPosition::Position,
};
use core::{mem::ManuallyDrop, pin::Pin};

// Source: src/sksl/ir/SkSLBlock.h:26-26
// Adapted inheritance and contiguous Rust field group. Native field order is
// retained; explicit Drop destroys children before their symbol-table owner.
// Pin<Box<StatementArray>> retains the real inline2 array at a stable address,
// with an extra Rust allocation; this is not a native layout/ABI claim.
#[repr(C)]
pub struct Block {
    base: StatementData,
    // Source: src/sksl/ir/SkSLBlock.h:107-109
    fSymbolTable: ManuallyDrop<Option<Box<SymbolTable>>>,
    fChildren: ManuallyDrop<Pin<Box<StatementArray>>>,
    fBlockKind: Kind,
}

// Source: src/sksl/ir/SkSLBlock.h:28-28
impl IRNodeType for Block {
    const kIRNodeKind: i32 = StatementKind::kBlock as i32;
}

// Source: src/sksl/ir/SkSLBlock.h:32-39
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    kUnbracedBlock,
    kBracedScope,
    kCompoundStatement,
}

impl Block {
    // Source: src/sksl/ir/SkSLBlock.h:41-48
    pub fn new(
        pos: Position,
        statements: Pin<Box<StatementArray>>,
        kind: Kind,
        symbols: Option<Box<SymbolTable>>,
    ) -> Self {
        Self {
            base: StatementData::new(pos, StatementKind::kBlock),
            fSymbolTable: ManuallyDrop::new(symbols),
            fChildren: ManuallyDrop::new(statements),
            fBlockKind: kind,
        }
    }

    // Source: src/sksl/ir/SkSLBlock.h:71-73
    pub fn children(&self) -> &StatementArray {
        self.fChildren.as_ref().get_ref()
    }

    // Source: src/sksl/ir/SkSLBlock.h:75-77
    pub fn children_mut(self: Pin<&mut Self>) -> Pin<&mut StatementArray> {
        unsafe { self.get_unchecked_mut().fChildren.as_mut() }
    }

    // Source: src/sksl/ir/SkSLBlock.h:79-81
    pub fn isScope(&self) -> bool {
        self.fBlockKind == Kind::kBracedScope
    }

    // Source: src/sksl/ir/SkSLBlock.h:83-85
    pub fn blockKind(&self) -> Kind {
        self.fBlockKind
    }

    // Source: src/sksl/ir/SkSLBlock.h:87-89
    pub fn setBlockKind(self: Pin<&mut Self>, kind: Kind) {
        unsafe {
            self.get_unchecked_mut().fBlockKind = kind;
        }
    }

    // Source: src/sksl/ir/SkSLBlock.h:91-93
    pub fn symbolTable(&self) -> Option<&SymbolTable> {
        self.fSymbolTable.as_deref()
    }
}

unsafe impl Statement for Block {
    // Source: src/sksl/ir/SkSLBlock.h:95-102
    fn isEmpty(&self) -> bool {
        for stmt in self.children().as_slice() {
            if !stmt.as_ref().expect("native non-null statement").isEmpty() {
                return false;
            }
        }
        true
    }
}

// Uncounted language destructor glue for the native implicit reverse field
// destruction: fBlockKind (trivial), fChildren, fSymbolTable, inherited base.
impl Drop for Block {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fChildren);
            ManuallyDrop::drop(&mut self.fSymbolTable);
        }
    }
}

impl Block {
    // Source: src/sksl/ir/SkSLBlock.cpp:14-57
    /// Safety: native current-pool lifetime and no-unwind allocation/destruction
    /// protocol must hold. Input slots are non-null native statements.
    pub unsafe fn Make(
        pos: Position,
        mut statements: Pin<Box<StatementArray>>,
        kind: Kind,
        symbols: Option<Box<SymbolTable>>,
    ) -> IRNodeBox<dyn Statement> {
        if kind == Kind::kBracedScope || symbols.as_ref().is_some_and(|s| s.count() != 0) {
            return Self::MakeBlock(pos, statements, kind, symbols)
                .cast(|p| p as *mut dyn Statement);
        }
        if statements.empty() {
            return Nop::Make();
        }
        if statements.size() > 1 {
            let mut foundStatement = None;
            for index in 0..statements.size() {
                if !statements.as_slice()[index as usize]
                    .as_ref()
                    .expect("native non-null statement")
                    .isEmpty()
                {
                    if foundStatement.is_none() {
                        foundStatement = Some(index);
                        continue;
                    }
                    return Self::MakeBlock(pos, statements, kind, None)
                        .cast(|p| p as *mut dyn Statement);
                }
            }
            if let Some(index) = foundStatement {
                return statements
                    .as_mut()
                    .at_mut(index)
                    .take()
                    .expect("native non-null statement");
            }
        }
        statements
            .as_mut()
            .at_mut(0)
            .take()
            .expect("native non-null statement")
    }

    // Source: src/sksl/ir/SkSLBlock.cpp:59-66
    /// Safety: same pool and no-unwind protocol as Make.
    pub unsafe fn MakeBlock(
        pos: Position,
        statements: Pin<Box<StatementArray>>,
        kind: Kind,
        symbols: Option<Box<SymbolTable>>,
    ) -> IRNodeBox<Self> {
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, statements, kind, symbols)))
    }

    // Source: src/sksl/ir/SkSLBlock.cpp:68-95
    /// Safety: same pool and no-unwind protocol as Make.
    pub unsafe fn MakeCompoundStatement(
        mut existing: Option<IRNodeBox<dyn Statement>>,
        additional: Option<IRNodeBox<dyn Statement>>,
    ) -> Option<IRNodeBox<dyn Statement>> {
        if existing.as_ref().map_or(true, |s| s.isEmpty()) {
            return additional;
        }
        if additional.as_ref().map_or(true, |s| s.isEmpty()) {
            return existing;
        }
        if existing.as_ref().unwrap().node().is::<Block>() {
            // Project from the exclusive COMPLETE trait-object allocation,
            // retaining original provenance, never from a shared base borrow.
            let statement = existing.as_mut().unwrap().as_pin_mut();
            let block_ptr = Pin::get_unchecked_mut(statement) as *mut dyn Statement as *mut Block;
            let mut block = Pin::new_unchecked(&mut *block_ptr);
            if block.blockKind() == Kind::kCompoundStatement {
                block.as_mut().children_mut().push_back(additional);
                return existing;
            }
        }
        let pos = existing
            .as_ref()
            .unwrap()
            .node()
            .fPosition
            .rangeThrough(additional.as_ref().unwrap().node().fPosition);
        let mut stmts = StatementArray::new_pinned();
        stmts.as_mut().reserve_exact(2);
        stmts.as_mut().push_back(existing);
        stmts.as_mut().push_back(additional);
        Some(Self::Make(pos, stmts, Kind::kCompoundStatement, None))
    }
}

impl IRNode for Block {
    // Source: src/sksl/ir/SkSLBlock.cpp:97-112
    fn description(&self) -> Vec<u8> {
        let mut result = Vec::new();
        let isScope = self.isScope() || self.isEmpty();
        if isScope {
            result.extend_from_slice(b"{");
        }
        for stmt in self.children().as_slice() {
            result.extend_from_slice(b"\n");
            result.extend_from_slice(
                &stmt
                    .as_ref()
                    .expect("native non-null statement")
                    .description(),
            );
        }
        result.extend_from_slice(if isScope { b"\n}\n" } else { b"\n" });
        result
    }

    // Uncounted projection of the actual inherited base, no duplicate state.
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|v| &mut v.base.base) }
    }
}
