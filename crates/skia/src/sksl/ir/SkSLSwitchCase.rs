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
// Source: src/sksl/ir/SkSLSwitchCase.h:26-26
// Source: src/sksl/ir/SkSLSwitchCase.h:38-38
// Source: src/sksl/ir/SkSLSwitchCase.h:43-43
// Source: src/sksl/ir/SkSLSwitchCase.h:47-47
#[repr(C)]
pub struct SwitchCase {
    base: StatementData,
    fDefault: bool,
    fValue: SKSL_INT,
    fStatement: ManuallyDrop<StmtSlot>,
}
// Source: src/sksl/ir/SkSLSwitchCase.h:28-28
impl IRNodeType for SwitchCase {
    const kIRNodeKind: i32 = Kind::kSwitchCase as i32;
}
impl SwitchCase {
    // Source: src/sksl/ir/SkSLSwitchCase.h:37-39
    pub fn isDefault(&self) -> bool {
        self.fDefault
    }
    // Source: src/sksl/ir/SkSLSwitchCase.h:41-44
    pub fn value(&self) -> SKSL_INT {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLSwitchCase.h\0".as_ptr(),42; @condition b"!this->isDefault()\0".as_ptr(); !self.isDefault());
        self.fValue
    }
    // Source: src/sksl/ir/SkSLSwitchCase.h:46-48
    pub unsafe fn statement_mut(&mut self) -> &mut StmtSlot {
        &mut self.fStatement
    }
    // Source: src/sksl/ir/SkSLSwitchCase.h:50-52
    pub fn statement(&self) -> &StmtSlot {
        &self.fStatement
    }
}
// Native reverse-member destruction, including statements before their symbol owner.
impl Drop for SwitchCase {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fStatement);
        }
    }
}
unsafe impl Statement for SwitchCase {}
impl IRNode for SwitchCase {
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
impl SwitchCase {
    // Source: src/sksl/ir/SkSLSwitchCase.h:57-61
    unsafe fn new(pos: Position, fDefault: bool, fValue: SKSL_INT, fStatement: StmtSlot) -> Self {
        Self {
            base: StatementData::new(pos, Kind::kSwitchCase),
            fDefault,
            fValue,
            fStatement: ManuallyDrop::new(fStatement),
        }
    }
}
impl SwitchCase {
    // Source: src/sksl/ir/SkSLSwitchCase.cpp:12-17
    pub unsafe fn Make(pos: Position, value: SKSL_INT, statement: StmtSlot) -> IRNodeBox<Self> {
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, false, value, statement)))
    }
    // Source: src/sksl/ir/SkSLSwitchCase.cpp:19-23
    pub unsafe fn MakeDefault(pos: Position, statement: StmtSlot) -> IRNodeBox<Self> {
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, true, -1, statement)))
    }
}
// Source: src/sksl/ir/SkSLSwitchCase.cpp:25-28
fn description(this: &SwitchCase) -> Vec<u8> {
    unsafe {
        let mut out = if this.fDefault {
            b"default: \n".to_vec()
        } else {
            let mut out = b"case ".to_vec();
            out.extend_from_slice(this.fValue.to_string().as_bytes());
            out.extend_from_slice(b": \n");
            out
        };
        out.extend_from_slice(&this.fStatement.as_ref().unwrap_unchecked().description());
        out
    }
}
