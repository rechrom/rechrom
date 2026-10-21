// UNVERIFIED: no validation performed.
use crate::src::sksl::{
    analysis::SkSLProgramUsage::ProgramUsage,
    ir::{
        SkSLExpression::{Expression, ExpressionExt},
        SkSLFunctionDefinition::FunctionDefinition,
        SkSLIRNode::*,
        SkSLNop::Nop,
        SkSLProgram::Program,
        SkSLProgramElement::ProgramElement,
        SkSLStatement::{Kind, Statement},
    },
    transform::SkSLProgramWriter::{self, ProgramWriter},
    SkSLAnalysis,
    SkSLContext::Context,
    SkSLModule::Module,
};
use core::{pin::Pin, ptr::NonNull};
type ES = Option<IRNodeBox<dyn Expression>>;
type SS = Option<IRNodeBox<dyn Statement>>;
type PS = Option<IRNodeBox<dyn ProgramElement>>;
unsafe fn sm<T: IRNodeType>(s: &mut SS) -> &mut T {
    s.as_mut()
        .unwrap_unchecked()
        .as_pin_mut()
        .node_mut()
        .as_node_mut::<T>()
        .get_unchecked_mut()
}
use crate::src::sksl::ir::{
    SkSLIfStatement::IfStatement, SkSLSwitchCase::SwitchCase, SkSLSwitchStatement::SwitchStatement,
};
// Source: src/sksl/transform/SkSLEliminateUnreachableCode.cpp:35-195
struct Eliminator<'a> {
    usage: &'a mut ProgramUsage,
    function: Vec<bool>,
    block: Vec<bool>,
}
unsafe impl ProgramWriter for Eliminator<'_> {
    unsafe fn visitExpressionPtr(&mut self, _: &mut ES) -> bool {
        false
    }
    unsafe fn visitStatementPtr(&mut self, s: &mut SS) -> bool {
        if *self.function.last().unwrap_unchecked() || *self.block.last().unwrap_unchecked() {
            if !s.as_ref().unwrap_unchecked().node().is::<Nop>() {
                self.usage.removeStatement(&**s.as_ref().unwrap_unchecked());
                *s = Some(Nop::Make());
            }
            return false;
        }
        match s.as_ref().unwrap_unchecked().kind() {
            Kind::kReturn | Kind::kDiscard => *self.function.last_mut().unwrap_unchecked() = true,
            Kind::kBreak | Kind::kContinue => *self.block.last_mut().unwrap_unchecked() = true,
            Kind::kExpression | Kind::kNop | Kind::kVarDeclaration => {}
            Kind::kBlock => {
                return SkSLProgramWriter::inheritedStatement(
                    self,
                    s.as_mut().unwrap_unchecked().as_pin_mut(),
                )
            }
            Kind::kDo => {
                self.block.push(false);
                let result = SkSLProgramWriter::inheritedStatement(
                    self,
                    s.as_mut().unwrap_unchecked().as_pin_mut(),
                );
                self.block.pop();
                return result;
            }
            Kind::kFor => {
                self.function.push(false);
                self.block.push(false);
                let result = SkSLProgramWriter::inheritedStatement(
                    self,
                    s.as_mut().unwrap_unchecked().as_pin_mut(),
                );
                self.block.pop();
                self.function.pop();
                return result;
            }
            Kind::kIf => {
                let i = sm::<IfStatement>(s);
                self.function.push(false);
                self.block.push(false);
                let mut result = i.ifTrue().is_some() && self.visitStatementPtr(i.ifTrue_mut());
                let ft = self.function.pop().unwrap_unchecked();
                let bt = self.block.pop().unwrap_unchecked();
                self.function.push(false);
                self.block.push(false);
                result |= i.ifFalse().is_some() && self.visitStatementPtr(i.ifFalse_mut());
                let ff = self.function.pop().unwrap_unchecked();
                let bf = self.block.pop().unwrap_unchecked();
                *self.function.last_mut().unwrap_unchecked() |= ft && ff;
                *self.block.last_mut().unwrap_unchecked() |= bt && bf;
                return result;
            }
            Kind::kSwitch => {
                let sw = sm::<SwitchStatement>(s);
                let mut result = false;
                let mut without = false;
                let mut default = false;
                let mut cases = sw.cases_mut();
                for c in cases.as_mut().as_mut_slice() {
                    self.function.push(false);
                    self.block.push(false);
                    let sc = sm::<SwitchCase>(c);
                    result |= self.visitStatementPtr(sc.statement_mut());
                    if sc.isDefault() {
                        without |= !*self.function.last().unwrap_unchecked();
                        default = true;
                    } else {
                        without |= !*self.function.last().unwrap_unchecked()
                            && *self.block.last().unwrap_unchecked();
                    }
                    self.function.pop();
                    self.block.pop();
                }
                *self.function.last_mut().unwrap_unchecked() |= !without && default;
                return result;
            }
            Kind::kSwitchCase => core::hint::unreachable_unchecked(),
        }
        false
    }
}
// Source: src/sksl/transform/SkSLEliminateUnreachableCode.cpp:197-204
unsafe fn eliminate(elements: &mut [PS], usage: &mut ProgramUsage) {
    for pe in elements {
        if pe
            .as_ref()
            .unwrap_unchecked()
            .node()
            .is::<FunctionDefinition>()
        {
            let mut v = Eliminator {
                usage,
                function: vec![false],
                block: vec![false],
            };
            let f = pe
                .as_mut()
                .unwrap_unchecked()
                .as_pin_mut()
                .node_mut()
                .as_node_mut::<FunctionDefinition>()
                .get_unchecked_mut();
            v.visitStatementPtr(f.body_mut());
        }
    }
}
// Source: src/sksl/transform/SkSLEliminateUnreachableCode.cpp:206-208
pub unsafe fn EliminateUnreachableCodeForModule(module: &mut Module, usage: &mut ProgramUsage) {
    eliminate(&mut module.fElements, usage)
}
// Source: src/sksl/transform/SkSLEliminateUnreachableCode.cpp:210-214
pub unsafe fn EliminateUnreachableCode(program: &mut Program) {
    eliminate(
        &mut program.fOwnedElements,
        program.fUsage.as_deref_mut().unwrap_unchecked(),
    )
}
