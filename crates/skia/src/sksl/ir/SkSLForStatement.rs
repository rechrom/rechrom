// UNVERIFIED: user explicitly prohibited all validation.
use super::{
    SkSLExpression::{Expression, ExpressionExt},
    SkSLIRNode::*,
    SkSLStatement::{Kind, Statement, StatementData},
    SkSLSymbolTable::SymbolTable,
    SkSLType::Type,
};
use crate::src::sksl::{
    SkSLAnalysis,
    SkSLContext::Context,
    SkSLDefines::{ExpressionArray, StatementArray, SKSL_INT},
    SkSLErrorReporter::ErrorReporterExt,
    SkSLPosition::{ForLoopPositions, Position},
    SkSLProgramSettings::ProgramConfig,
};
use core::{mem::ManuallyDrop, pin::Pin, ptr::NonNull};
type ExprSlot = Option<IRNodeBox<dyn Expression>>;
type StmtSlot = Option<IRNodeBox<dyn Statement>>;
// Source: src/sksl/ir/SkSLForStatement.h:29-34
#[repr(C)]
pub struct LoopUnrollInfo {
    pub fIndex: Option<NonNull<dyn super::SkSLVariable::Variable>>,
    pub fStart: f64,
    pub fDelta: f64,
    pub fCount: i32,
}
// Source: src/sksl/ir/SkSLForStatement.h:39-39
// Source: src/sksl/ir/SkSLForStatement.h:88-88
// Source: src/sksl/ir/SkSLForStatement.h:136-136
// Source: src/sksl/ir/SkSLForStatement.h:92-92
// Source: src/sksl/ir/SkSLForStatement.h:100-100
// Source: src/sksl/ir/SkSLForStatement.h:108-108
// Source: src/sksl/ir/SkSLForStatement.h:116-116
// Source: src/sksl/ir/SkSLForStatement.h:141-141
#[repr(C)]
pub struct ForStatement {
    base: StatementData,
    fForLoopPositions: ForLoopPositions,
    fSymbolTable: ManuallyDrop<Option<Box<SymbolTable>>>,
    fInitializer: ManuallyDrop<StmtSlot>,
    fTest: ManuallyDrop<ExprSlot>,
    fNext: ManuallyDrop<ExprSlot>,
    fStatement: ManuallyDrop<StmtSlot>,
    fUnrollInfo: ManuallyDrop<Option<Box<LoopUnrollInfo>>>,
}
// Source: src/sksl/ir/SkSLForStatement.h:41-41
impl IRNodeType for ForStatement {
    const kIRNodeKind: i32 = Kind::kFor as i32;
}
impl ForStatement {
    // Source: src/sksl/ir/SkSLForStatement.h:43-58
    pub unsafe fn new(
        pos: Position,
        fForLoopPositions: ForLoopPositions,
        fInitializer: StmtSlot,
        fTest: ExprSlot,
        fNext: ExprSlot,
        fStatement: StmtSlot,
        fUnrollInfo: Option<Box<LoopUnrollInfo>>,
        fSymbolTable: Option<Box<SymbolTable>>,
    ) -> Self {
        Self {
            base: StatementData::new(pos, Kind::kFor),
            fForLoopPositions: fForLoopPositions,
            fSymbolTable: ManuallyDrop::new(fSymbolTable),
            fInitializer: ManuallyDrop::new(fInitializer),
            fTest: ManuallyDrop::new(fTest),
            fNext: ManuallyDrop::new(fNext),
            fStatement: ManuallyDrop::new(fStatement),
            fUnrollInfo: ManuallyDrop::new(fUnrollInfo),
        }
    }
    // Source: src/sksl/ir/SkSLForStatement.h:87-89
    pub fn forLoopPositions(&self) -> ForLoopPositions {
        self.fForLoopPositions
    }
    // Source: src/sksl/ir/SkSLForStatement.h:91-93
    pub unsafe fn initializer_mut(&mut self) -> &mut StmtSlot {
        &mut self.fInitializer
    }
    // Source: src/sksl/ir/SkSLForStatement.h:95-97
    pub fn initializer(&self) -> &StmtSlot {
        &self.fInitializer
    }
    // Source: src/sksl/ir/SkSLForStatement.h:99-101
    pub unsafe fn test_mut(&mut self) -> &mut ExprSlot {
        &mut self.fTest
    }
    // Source: src/sksl/ir/SkSLForStatement.h:103-105
    pub fn test(&self) -> &ExprSlot {
        &self.fTest
    }
    // Source: src/sksl/ir/SkSLForStatement.h:107-109
    pub unsafe fn next_mut(&mut self) -> &mut ExprSlot {
        &mut self.fNext
    }
    // Source: src/sksl/ir/SkSLForStatement.h:111-113
    pub fn next(&self) -> &ExprSlot {
        &self.fNext
    }
    // Source: src/sksl/ir/SkSLForStatement.h:115-117
    pub unsafe fn statement_mut(&mut self) -> &mut StmtSlot {
        &mut self.fStatement
    }
    // Source: src/sksl/ir/SkSLForStatement.h:119-121
    pub fn statement(&self) -> &StmtSlot {
        &self.fStatement
    }
    // Source: src/sksl/ir/SkSLForStatement.h:123-125
    pub fn symbols(&self) -> Option<NonNull<SymbolTable>> {
        self.fSymbolTable.as_deref().map(NonNull::from)
    }
    // Source: src/sksl/ir/SkSLForStatement.h:128-130
    pub fn unrollInfo(&self) -> Option<&LoopUnrollInfo> {
        self.fUnrollInfo.as_deref()
    }
}
// Native reverse-member destruction, including statements before their symbol owner.
impl Drop for ForStatement {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fUnrollInfo);
            ManuallyDrop::drop(&mut self.fStatement);
            ManuallyDrop::drop(&mut self.fNext);
            ManuallyDrop::drop(&mut self.fTest);
            ManuallyDrop::drop(&mut self.fInitializer);
            ManuallyDrop::drop(&mut self.fSymbolTable);
        }
    }
}
unsafe impl Statement for ForStatement {}
impl IRNode for ForStatement {
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base.base) }
    }
    fn description(&self) -> Vec<u8> {
        description(self)
    }
}
use super::{
    SkSLBlock::{Block, Kind as BlockKind},
    SkSLExpressionStatement::ExpressionStatement,
    SkSLNop::Nop,
    SkSLSymbol::Symbol,
    SkSLVarDeclarations::VarDeclaration,
};
// Source: src/sksl/ir/SkSLForStatement.cpp:27-44
unsafe fn is_vardecl_block_initializer(stmt: Option<&dyn Statement>) -> bool {
    let Some(stmt) = stmt else {
        return false;
    };
    if !stmt.node().is::<Block>() {
        return false;
    }
    let b = stmt.node().as_node::<Block>();
    if b.isScope() {
        return false;
    }
    for child in b.children().as_slice() {
        if !child
            .as_ref()
            .unwrap_unchecked()
            .node()
            .is::<VarDeclaration>()
        {
            return false;
        }
    }
    true
}
// Source: src/sksl/ir/SkSLForStatement.cpp:46-49
fn is_simple_initializer(stmt: Option<&dyn Statement>) -> bool {
    stmt.map_or(true, |s| {
        s.isEmpty() || s.node().is::<VarDeclaration>() || s.node().is::<ExpressionStatement>()
    })
}
// Source: src/sksl/ir/SkSLForStatement.cpp:51-68
fn description(this: &ForStatement) -> Vec<u8> {
    unsafe {
        let mut out = b"for (".to_vec();
        if let Some(stmt) = this.initializer() {
            out.extend_from_slice(&stmt.description());
        } else {
            out.push(b';');
        }
        out.push(b' ');
        if let Some(test) = this.test() {
            out.extend_from_slice(&test.description());
        }
        out.extend_from_slice(b"; ");
        if let Some(next) = this.next() {
            out.extend_from_slice(&next.description());
        }
        out.extend_from_slice(b") ");
        out.extend_from_slice(&this.statement().as_ref().unwrap_unchecked().description());
        out
    }
}
// Source: src/sksl/ir/SkSLForStatement.cpp:70-100
unsafe fn hoist_vardecl_symbols_into_outer_scope(
    context: &Context,
    initBlock: &Block,
    innerSymbols: NonNull<SymbolTable>,
    hoistedSymbols: NonNull<SymbolTable>,
) {
    struct SymbolHoister<'a> {
        fContext: &'a Context,
        fInnerSymbols: NonNull<SymbolTable>,
        fHoistedSymbols: NonNull<SymbolTable>,
    }
    impl SymbolHoister<'_> {
        unsafe fn visitExpression(&mut self, e: &dyn Expression) -> bool {
            crate::src::sksl::analysis::SkSLProgramVisitor::visitExpression(
                &mut |child| self.visitExpression(child),
                e,
            )
        }
        unsafe fn visitStatement(&mut self, stmt: &dyn Statement) -> bool {
            if stmt.node().is::<VarDeclaration>() {
                let var = stmt
                    .node()
                    .as_node::<VarDeclaration>()
                    .var()
                    .unwrap_unchecked();
                let symbol = NonNull::new_unchecked(var.as_ptr() as *mut dyn Symbol);
                self.fInnerSymbols.as_mut().moveSymbolTo(
                    self.fHoistedSymbols.as_mut(),
                    symbol,
                    self.fContext,
                );
                return false;
            }
            let this = self as *mut Self;
            crate::src::sksl::analysis::SkSLProgramVisitor::visitStatement(
                &mut |s| (*this).visitStatement(s),
                &mut |e| (*this).visitExpression(e),
                stmt,
            )
        }
    }
    SymbolHoister {
        fContext: context,
        fInnerSymbols: innerSymbols,
        fHoistedSymbols: hoistedSymbols,
    }
    .visitStatement(initBlock);
}
impl ForStatement {
    // Source: src/sksl/ir/SkSLForStatement.cpp:102-187
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        positions: ForLoopPositions,
        initializer: StmtSlot,
        mut test: ExprSlot,
        next: ExprSlot,
        statement: IRNodeBox<dyn Statement>,
        mut symbolTable: Option<Box<SymbolTable>>,
    ) -> StmtSlot {
        let isSimpleInitializer = is_simple_initializer(initializer.as_deref());
        let isVardeclBlockInitializer =
            !isSimpleInitializer && is_vardecl_block_initializer(initializer.as_deref());
        if !isSimpleInitializer && !isVardeclBlockInitializer {
            let mut errors = context.fErrors;
            errors.as_mut().error(
                initializer.as_ref().unwrap_unchecked().node().fPosition,
                b"invalid for loop initializer",
            );
            return None;
        }
        if test.is_some() {
            test =
                super::SkSLType::coerceExpression(&**context.fTypes.as_ref().fBool, test, context);
            if test.is_none() {
                return None;
            }
        }
        if next.as_ref().is_some_and(|n| n.isIncomplete(context)) {
            return None;
        }
        let unrollInfo;
        if context.fConfig.unwrap_unchecked().as_ref().strictES2Mode() {
            unrollInfo = crate::src::sksl::analysis::SkSLGetLoopUnrollInfo::GetLoopUnrollInfo(
                context,
                pos,
                &positions,
                initializer.as_deref(),
                &mut test,
                next.as_deref(),
                Some(&*statement),
                Some(context.fErrors),
            );
            if unrollInfo.is_none() {
                return None;
            }
        } else {
            unrollInfo = crate::src::sksl::analysis::SkSLGetLoopUnrollInfo::GetLoopUnrollInfo(
                context,
                pos,
                &positions,
                initializer.as_deref(),
                &mut test,
                next.as_deref(),
                Some(&*statement),
                None,
            );
        }
        if SkSLAnalysis::DetectVarDeclarationWithoutScope(&*statement, Some(context.fErrors)) {
            return None;
        }
        if isVardeclBlockInitializer {
            let mut hoistedSymbols = symbolTable.as_mut().unwrap_unchecked().insertNewParent();
            hoist_vardecl_symbols_into_outer_scope(
                context,
                initializer
                    .as_ref()
                    .unwrap_unchecked()
                    .node()
                    .as_node::<Block>(),
                NonNull::from(symbolTable.as_mut().unwrap_unchecked().as_mut()),
                NonNull::from(hoistedSymbols.as_mut()),
            );
            let mut scope = StatementArray::new_pinned();
            scope.as_mut().push_back(initializer);
            scope.as_mut().push_back(Some(Self::Make(
                context,
                pos,
                positions,
                None,
                test,
                next,
                statement,
                unrollInfo,
                symbolTable,
            )));
            return Some(Block::Make(
                pos,
                scope,
                BlockKind::kBracedScope,
                Some(hoistedSymbols),
            ));
        }
        Some(Self::Make(
            context,
            pos,
            positions,
            initializer,
            test,
            next,
            statement,
            unrollInfo,
            symbolTable,
        ))
    }
    // Source: src/sksl/ir/SkSLForStatement.cpp:189-205
    pub unsafe fn ConvertWhile(
        context: &Context,
        pos: Position,
        test: ExprSlot,
        statement: IRNodeBox<dyn Statement>,
    ) -> StmtSlot {
        if context.fConfig.unwrap_unchecked().as_ref().strictES2Mode() {
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, b"while loops are not supported");
            return None;
        }
        Self::Convert(
            context,
            pos,
            ForLoopPositions::default(),
            None,
            test,
            None,
            statement,
            None,
        )
    }
    // Source: src/sksl/ir/SkSLForStatement.cpp:207-240
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        positions: ForLoopPositions,
        initializer: StmtSlot,
        test: ExprSlot,
        next: ExprSlot,
        statement: IRNodeBox<dyn Statement>,
        unrollInfo: Option<Box<LoopUnrollInfo>>,
        symbolTable: Option<Box<SymbolTable>>,
    ) -> IRNodeBox<dyn Statement> {
        debug_assert!(
            is_simple_initializer(initializer.as_deref())
                || is_vardecl_block_initializer(initializer.as_deref())
        );
        debug_assert!(
            test.is_none()
                || test
                    .as_ref()
                    .unwrap_unchecked()
                    .expression_type()
                    .matches(&**context.fTypes.as_ref().fBool)
        );
        debug_assert!(!SkSLAnalysis::DetectVarDeclarationWithoutScope(
            &*statement,
            None
        ));
        debug_assert!(
            unrollInfo.is_some() || !context.fConfig.unwrap_unchecked().as_ref().strictES2Mode()
        );
        if let Some(info) = unrollInfo.as_ref() {
            if info.fCount <= 0 || statement.isEmpty() {
                return Nop::Make();
            }
        }
        IRNodeBox::new_with(|p: *mut Self| {
            p.write(Self::new(
                pos,
                positions,
                initializer,
                test,
                next,
                Some(statement),
                unrollInfo,
                symbolTable,
            ))
        })
        .cast(|p| p as *mut dyn Statement)
    }
}
