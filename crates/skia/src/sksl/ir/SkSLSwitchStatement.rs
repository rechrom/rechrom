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
// Source: src/sksl/ir/SkSLSwitchStatement.h:30-30
// Source: src/sksl/ir/SkSLSwitchStatement.h:59-59
// Source: src/sksl/ir/SkSLSwitchStatement.h:67-67
#[repr(C)]
pub struct SwitchStatement {
    base: StatementData,
    fValue: ManuallyDrop<ExprSlot>,
    fCaseBlock: ManuallyDrop<StmtSlot>,
}
// Source: src/sksl/ir/SkSLSwitchStatement.h:32-32
impl IRNodeType for SwitchStatement {
    const kIRNodeKind: i32 = Kind::kSwitch as i32;
}
impl SwitchStatement {
    // Source: src/sksl/ir/SkSLSwitchStatement.h:34-39
    pub unsafe fn new(pos: Position, fValue: ExprSlot, fCaseBlock: StmtSlot) -> Self {
        Self {
            base: StatementData::new(pos, Kind::kSwitch),
            fValue: ManuallyDrop::new(fValue),
            fCaseBlock: ManuallyDrop::new(fCaseBlock),
        }
    }
    // Source: src/sksl/ir/SkSLSwitchStatement.h:58-60
    pub unsafe fn value_mut(&mut self) -> &mut ExprSlot {
        &mut self.fValue
    }
    // Source: src/sksl/ir/SkSLSwitchStatement.h:62-64
    pub fn value(&self) -> &ExprSlot {
        &self.fValue
    }
    // Source: src/sksl/ir/SkSLSwitchStatement.h:66-68
    pub unsafe fn caseBlock_mut(&mut self) -> &mut StmtSlot {
        &mut self.fCaseBlock
    }
    // Source: src/sksl/ir/SkSLSwitchStatement.h:70-72
    pub fn caseBlock(&self) -> &StmtSlot {
        &self.fCaseBlock
    }
}
// Native reverse-member destruction, including statements before their symbol owner.
impl Drop for SwitchStatement {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fCaseBlock);
            ManuallyDrop::drop(&mut self.fValue);
        }
    }
}
unsafe impl Statement for SwitchStatement {}
impl IRNode for SwitchStatement {
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
    SkSLBreakStatement::BreakStatement,
    SkSLNop::Nop,
    SkSLSwitchCase::SwitchCase,
};
use crate::src::sksl::SkSLConstantFolder::ConstantFolder;
impl SwitchStatement {
    // Language projection of native Block-valued case owner, never a copied case array.
    // Source: src/sksl/ir/SkSLSwitchStatement.h:74-76
    pub unsafe fn cases_mut(&mut self) -> Pin<&mut StatementArray> {
        let block =
            &mut *(self.fCaseBlock.as_ref().unwrap_unchecked().as_ptr() as *mut () as *mut Block);
        Pin::new_unchecked(block).children_mut()
    }
    // Source: src/sksl/ir/SkSLSwitchStatement.h:78-80
    pub unsafe fn cases(&self) -> &StatementArray {
        self.caseBlock()
            .as_ref()
            .unwrap_unchecked()
            .node()
            .as_node::<Block>()
            .children()
    }
}
// Source: src/sksl/ir/SkSLSwitchStatement.cpp:36-38
fn description(this: &SwitchStatement) -> Vec<u8> {
    unsafe {
        let mut out = b"switch (".to_vec();
        out.extend_from_slice(&this.value().as_ref().unwrap_unchecked().description());
        out.extend_from_slice(b") ");
        out.extend_from_slice(&this.caseBlock().as_ref().unwrap_unchecked().description());
        out
    }
}
// Source: src/sksl/ir/SkSLSwitchStatement.cpp:40-64
unsafe fn find_duplicate_case_values(
    cases: &StatementArray,
) -> Pin<Box<crate::include::private::SkTArray::TArray<NonNull<SwitchCase>, true>>> {
    let mut duplicateCases = crate::include::private::SkTArray::TArray::new_pinned();
    let mut intValues = crate::src::core::SkTHash::THashSet::<SKSL_INT>::default();
    let mut foundDefault = false;
    for stmt in cases.as_slice() {
        let sc = stmt
            .as_ref()
            .unwrap_unchecked()
            .node()
            .as_node::<SwitchCase>();
        if sc.isDefault() {
            if foundDefault {
                duplicateCases.as_mut().push_back(NonNull::from(sc));
                continue;
            }
            foundDefault = true;
        } else {
            let value = sc.value();
            if intValues.contains(&value) {
                duplicateCases.as_mut().push_back(NonNull::from(sc));
                continue;
            }
            intValues.add(value);
        }
    }
    duplicateCases
}
// Source: src/sksl/ir/SkSLSwitchStatement.cpp:66-82
unsafe fn remove_break_statements(stmt: &mut StmtSlot) {
    struct RemoveBreaksWriter;
    unsafe impl crate::src::sksl::transform::SkSLProgramWriter::ProgramWriter for RemoveBreaksWriter {
        unsafe fn visitStatementPtr(&mut self, stmt: &mut StmtSlot) -> bool {
            if stmt
                .as_ref()
                .unwrap_unchecked()
                .node()
                .is::<BreakStatement>()
            {
                *stmt = Some(Nop::Make());
                return false;
            }
            crate::src::sksl::transform::SkSLProgramWriter::inheritedStatement(
                self,
                stmt.as_mut().unwrap_unchecked().as_pin_mut(),
            )
        }
        unsafe fn visitExpressionPtr(&mut self, _expr: &mut ExprSlot) -> bool {
            false
        }
    }
    crate::src::sksl::transform::SkSLProgramWriter::ProgramWriter::visitStatementPtr(
        &mut RemoveBreaksWriter,
        stmt,
    );
}
// Source: src/sksl/ir/SkSLSwitchStatement.cpp:84-144
unsafe fn block_for_case(
    caseBlock: *mut dyn Statement,
    caseToCapture: NonNull<SwitchCase>,
) -> bool {
    let block = &mut *(caseBlock as *mut () as *mut Block);
    let mut cases = Pin::new_unchecked(block).children_mut();
    let mut iter = 0usize;
    while iter < cases.size() as usize {
        let sc = cases.as_slice()[iter]
            .as_ref()
            .unwrap_unchecked()
            .node()
            .as_node::<SwitchCase>();
        if core::ptr::eq(sc, caseToCapture.as_ptr()) {
            break;
        }
        iter += 1;
    }
    let mut startIter = iter;
    let mut removeBreakStatements = false;
    while iter < cases.size() as usize {
        let stmt = cases.as_slice()[iter]
            .as_ref()
            .unwrap_unchecked()
            .node()
            .as_node::<SwitchCase>()
            .statement()
            .as_ref()
            .unwrap_unchecked();
        if crate::src::sksl::analysis::SkSLSwitchCaseContainsExit::SwitchCaseContainsConditionalExit(
            &**stmt,
        ) {
            return false;
        }
        if crate::src::sksl::analysis::SkSLSwitchCaseContainsExit::SwitchCaseContainsUnconditionalExit(&**stmt){removeBreakStatements=true;iter+=1;break;}
        iter += 1;
    }
    let numElements = iter - startIter;
    for index in 0..numElements {
        let sc = &mut *(cases.as_slice()[startIter]
            .as_ref()
            .unwrap_unchecked()
            .as_ptr() as *mut () as *mut SwitchCase);
        let moved = sc.statement_mut().take();
        cases.as_mut().as_mut_slice()[index] = moved;
        startIter += 1;
    }
    let excess = cases.size() as usize - numElements;
    cases.as_mut().pop_back_n(excess as i32);
    if removeBreakStatements {
        remove_break_statements(cases.as_mut().as_mut_slice().last_mut().unwrap_unchecked());
    }
    true
}
impl SwitchStatement {
    // Source: src/sksl/ir/SkSLSwitchStatement.cpp:146-214
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        value: ExprSlot,
        mut caseValues: Pin<Box<ExpressionArray>>,
        mut caseStatements: Pin<Box<StatementArray>>,
        mut symbolTable: Box<SymbolTable>,
    ) -> StmtSlot {
        debug_assert!(caseValues.as_base().size() == caseStatements.size());
        let value =
            super::SkSLType::coerceExpression(&**context.fTypes.as_ref().fInt, value, context)?;
        let mut cases = StatementArray::new_pinned();
        for i in 0..caseValues.as_base().size() as usize {
            if caseValues.as_base().as_slice()[i].is_some() {
                let casePos = caseValues.as_base().as_slice()[i]
                    .as_ref()
                    .unwrap_unchecked()
                    .node()
                    .fPosition;
                let source = caseValues.as_mut().as_base_mut().as_mut_slice()[i].take();
                let caseValue =
                    super::SkSLType::coerceExpression(value.expression_type(), source, context)?;
                let mut intValue = core::mem::MaybeUninit::<SKSL_INT>::uninit();
                if !ConstantFolder::GetConstantInt(&*caseValue, intValue.as_mut_ptr()) {
                    let mut errors = context.fErrors;
                    errors
                        .as_mut()
                        .error(casePos, b"case value must be a constant integer");
                    return None;
                }
                let stmt = caseStatements.as_mut().as_mut_slice()[i].take();
                cases.as_mut().push_back(Some(
                    SwitchCase::Make(casePos, intValue.assume_init(), stmt)
                        .cast(|p| p as *mut dyn Statement),
                ));
            } else {
                let stmt = caseStatements.as_mut().as_mut_slice()[i].take();
                cases.as_mut().push_back(Some(
                    SwitchCase::MakeDefault(pos, stmt).cast(|p| p as *mut dyn Statement),
                ));
            }
        }
        let duplicateCases = find_duplicate_case_values(&cases);
        if !duplicateCases.empty() {
            for sc in duplicateCases.as_slice() {
                let sc = sc.as_ref();
                let mut errors = context.fErrors;
                if sc.isDefault() {
                    errors
                        .as_mut()
                        .error(sc.node().fPosition, b"duplicate default case");
                } else {
                    let mut msg = b"duplicate case value '".to_vec();
                    msg.extend_from_slice(sc.value().to_string().as_bytes());
                    msg.push(b'\'');
                    errors.as_mut().error(sc.node().fPosition, &msg);
                }
            }
            return None;
        }
        let mut block=crate::src::sksl::transform::SkSLHoistSwitchVarDeclarationsAtTopLevel::HoistSwitchVarDeclarationsAtTopLevel(context,&mut cases,&mut symbolTable,pos);
        let switchStmt = Self::Make(
            context,
            pos,
            value,
            Block::MakeBlock(pos, cases, BlockKind::kBracedScope, Some(symbolTable))
                .cast(|p| p as *mut dyn Statement),
        );
        if let Some(mut block) = block {
            block
                .as_pin_mut()
                .children_mut()
                .push_back(Some(switchStmt));
            Some(block.cast(|p| p as *mut dyn Statement))
        } else {
            Some(switchStmt)
        }
    }
    // Source: src/sksl/ir/SkSLSwitchStatement.cpp:216-271
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        value: IRNodeBox<dyn Expression>,
        mut caseBlock: IRNodeBox<dyn Statement>,
    ) -> IRNodeBox<dyn Statement> {
        let cases = caseBlock.node().as_node::<Block>().children();
        debug_assert!(cases.as_slice().iter().all(|stmt| stmt
            .as_ref()
            .unwrap_unchecked()
            .node()
            .is::<SwitchCase>()));
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLSwitchStatement.cpp\0".as_ptr(),227; @condition b"find_duplicate_case_values(cases).empty()\0".as_ptr(); find_duplicate_case_values(cases).empty());
        if context
            .fConfig
            .unwrap_unchecked()
            .as_ref()
            .fSettings
            .fOptimize
        {
            let mut switchValue = core::mem::MaybeUninit::<SKSL_INT>::uninit();
            if ConstantFolder::GetConstantInt(&*value, switchValue.as_mut_ptr()) {
                let switchValue = switchValue.assume_init();
                let mut defaultCase = None;
                let mut matchingCase = None;
                for stmt in cases.as_slice() {
                    let sc = stmt
                        .as_ref()
                        .unwrap_unchecked()
                        .node()
                        .as_node::<SwitchCase>();
                    if sc.isDefault() {
                        defaultCase = Some(NonNull::from(sc));
                        continue;
                    }
                    if sc.value() == switchValue {
                        matchingCase = Some(NonNull::from(sc));
                        break;
                    }
                }
                if matchingCase.is_none() {
                    if defaultCase.is_none() {
                        let block = &mut *(caseBlock.as_ptr() as *mut () as *mut Block);
                        Pin::new_unchecked(block).children_mut().clear();
                        return caseBlock;
                    }
                    matchingCase = defaultCase;
                }
                if block_for_case(
                    caseBlock.as_ptr() as *mut dyn Statement,
                    matchingCase.unwrap_unchecked(),
                ) {
                    return caseBlock;
                }
            }
        }
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, Some(value), Some(caseBlock))))
            .cast(|p| p as *mut dyn Statement)
    }
}
