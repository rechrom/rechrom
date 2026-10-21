// UNVERIFIED: no compilation, tests, syntax or source checks were run.
use super::SkSLProgramWriter::{inheritedStatement, ProgramWriter};
use crate::src::sksl::{
    ir::{
        SkSLBlock::{Block, Kind as BlockKind},
        SkSLExpression::Expression,
        SkSLIRHelpers::IRHelpers,
        SkSLIRNode::{IRNode, IRNodeBox},
        SkSLNop::Nop,
        SkSLStatement::{Kind, Statement},
        SkSLSymbol::Symbol,
        SkSLSymbolTable::SymbolTable,
        SkSLVarDeclarations::VarDeclaration,
    },
    SkSLContext::Context,
    SkSLDefines::StatementArray,
    SkSLPosition::Position,
};
use core::{pin::Pin, ptr::NonNull};
type StmtSlot = Option<IRNodeBox<dyn Statement>>;
// Source: src/sksl/transform/SkSLHoistSwitchVarDeclarationsAtTopLevel.cpp:39-78
struct HoistSwitchVarDeclsVisitor<'a> {
    fContext: &'a Context,
    fVarDeclarations: Vec<NonNull<StmtSlot>>,
}
unsafe impl ProgramWriter for HoistSwitchVarDeclsVisitor<'_> {
    unsafe fn visitExpressionPtr(&mut self, _expr: &mut Option<IRNodeBox<dyn Expression>>) -> bool {
        false
    }
    unsafe fn visitStatementPtr(&mut self, stmt: &mut StmtSlot) -> bool {
        let value = stmt.as_ref().unwrap_unchecked();
        match value.kind() {
            Kind::kSwitchCase => {
                return inheritedStatement(self, stmt.as_mut().unwrap_unchecked().as_pin_mut())
            }
            Kind::kBlock => {
                if !value.node().as_node::<Block>().isScope() {
                    return inheritedStatement(self, stmt.as_mut().unwrap_unchecked().as_pin_mut());
                }
            }
            Kind::kVarDeclaration => self.fVarDeclarations.push(NonNull::from(stmt)),
            _ => {}
        }
        false
    }
}
// Source: src/sksl/transform/SkSLHoistSwitchVarDeclarationsAtTopLevel.cpp:35-38
// Source: src/sksl/transform/SkSLHoistSwitchVarDeclarationsAtTopLevel.cpp:80-137
/// Safety: cases and symbols are exclusively leased; IR storage and context borrowed
/// objects retain their native pool/symbol-table lifetime throughout this transform.
pub unsafe fn HoistSwitchVarDeclarationsAtTopLevel(
    context: &Context,
    cases: &mut Pin<Box<StatementArray>>,
    switchSymbols: &mut SymbolTable,
    pos: Position,
) -> Option<IRNodeBox<Block>> {
    let mut visitor = HoistSwitchVarDeclsVisitor {
        fContext: context,
        fVarDeclarations: Vec::new(),
    };
    for sc in cases.as_mut().as_mut_slice() {
        visitor.visitStatementPtr(sc);
    }
    if visitor.fVarDeclarations.is_empty() {
        return None;
    }
    let mut blockSymbols = switchSymbols.insertNewParent();
    let mut blockStmts = StatementArray::new_pinned();
    blockStmts
        .as_mut()
        .reserve_exact(visitor.fVarDeclarations.len() as i32 + 1);
    for mut declarationSlot in visitor.fVarDeclarations {
        // Source slot arrays remain fixed during collection and transfer. Their
        // actual owners are moved once, without reallocating any source block.
        let slot = declarationSlot.as_mut();
        let decl =
            &mut *(slot.as_ref().unwrap_unchecked().as_ptr() as *mut () as *mut VarDeclaration);
        let var = decl.var().unwrap_unchecked();
        let isConst = var.as_ref().modifierFlags().isConst();
        let replacement = if decl.value().is_some() && !isConst {
            let helper = IRHelpers::new(context);
            let target = helper.Ref(var);
            helper.Assign(target, decl.value_mut().take().unwrap_unchecked())
        } else {
            debug_assert!(
                !isConst
                    || crate::src::sksl::analysis::SkSLIsConstantExpression::IsConstantExpression(
                        &**decl.value().as_ref().unwrap_unchecked()
                    )
            );
            Nop::Make()
        };
        blockStmts.as_mut().push_back(slot.take());
        *slot = Some(replacement);
        switchSymbols.moveSymbolTo(
            &mut blockSymbols,
            NonNull::new_unchecked(var.as_ptr() as *mut dyn Symbol),
            context,
        );
    }
    Some(Block::MakeBlock(
        pos,
        blockStmts,
        BlockKind::kBracedScope,
        Some(blockSymbols),
    ))
}
