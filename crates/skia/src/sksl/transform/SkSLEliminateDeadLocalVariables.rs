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
    SkSLBinaryExpression::BinaryExpression,
    SkSLExpressionStatement::ExpressionStatement,
    SkSLVarDeclarations::VarDeclaration,
    SkSLVariable::{Variable, VariableStorage},
    SkSLVariableReference::VariableReference,
};
// Source: src/sksl/transform/SkSLEliminateDeadLocalVariables.cpp:40-142
struct Eliminator<'a> {
    context: &'a Context,
    usage: &'a mut ProgramUsage,
    dead: std::collections::HashSet<usize>,
    changed: bool,
    assignment: bool,
}
unsafe impl ProgramWriter for Eliminator<'_> {
    unsafe fn visitExpressionPtr(&mut self, e: &mut ES) -> bool {
        if e.as_ref()
            .unwrap_unchecked()
            .node()
            .is::<BinaryExpression>()
        {
            let b = e
                .as_ref()
                .unwrap_unchecked()
                .node()
                .as_node::<BinaryExpression>();
            if let Some(v) = b.isAssignmentIntoVariable() {
                if self.dead.contains(
                    &(v.as_ref().variable().unwrap_unchecked().as_ptr() as *mut () as usize),
                ) {
                    self.usage
                        .removeExpression(&**e.as_ref().unwrap_unchecked());
                    let b = e
                        .as_mut()
                        .unwrap_unchecked()
                        .as_pin_mut()
                        .node_mut()
                        .as_node_mut::<BinaryExpression>()
                        .get_unchecked_mut();
                    let rhs = b.right_mut().take();
                    *e = rhs;
                    self.usage.addExpression(&**e.as_ref().unwrap_unchecked());
                    self.assignment = true;
                    return self.visitExpressionPtr(e);
                }
            }
        }
        if e.as_ref()
            .unwrap_unchecked()
            .node()
            .is::<VariableReference>()
        {
            debug_assert!(!self.dead.contains(
                &(e.as_ref()
                    .unwrap_unchecked()
                    .node()
                    .as_node::<VariableReference>()
                    .variable()
                    .unwrap_unchecked()
                    .as_ptr() as *mut () as usize)
            ));
        }
        SkSLProgramWriter::inheritedExpression(self, e.as_mut().unwrap_unchecked().as_pin_mut())
    }
    unsafe fn visitStatementPtr(&mut self, s: &mut SS) -> bool {
        if s.as_ref().unwrap_unchecked().node().is::<VarDeclaration>() {
            let d = s
                .as_ref()
                .unwrap_unchecked()
                .node()
                .as_node::<VarDeclaration>();
            let v = d.var().unwrap_unchecked();
            let counts = self.usage.getVariable(v.as_ref());
            debug_assert!(counts.fVarExists != 0);
            if counts.fVarExists != 0
                && counts.fRead == 0
                && v.as_ref().storage() == VariableStorage::kLocal
            {
                self.dead.insert(v.as_ptr() as *mut () as usize);
                self.usage.removeStatement(&**s.as_ref().unwrap_unchecked());
                if v.as_ref().initialValue().is_some() {
                    let value = sm::<VarDeclaration>(s)
                        .value_mut()
                        .take()
                        .unwrap_unchecked();
                    *s = Some(ExpressionStatement::Make(self.context, value));
                    self.usage.addStatement(&**s.as_ref().unwrap_unchecked());
                } else {
                    *s = Some(Nop::Make());
                }
                self.changed = true;
                return self.visitStatementPtr(s);
            }
        }
        let result =
            SkSLProgramWriter::inheritedStatement(self, s.as_mut().unwrap_unchecked().as_pin_mut());
        if self.assignment {
            self.assignment = false;
            if s.as_ref()
                .unwrap_unchecked()
                .node()
                .is::<ExpressionStatement>()
            {
                let e = s
                    .as_ref()
                    .unwrap_unchecked()
                    .node()
                    .as_node::<ExpressionStatement>();
                if !SkSLAnalysis::HasSideEffects(&**e.expression().as_ref().unwrap_unchecked()) {
                    self.usage.removeStatement(e);
                    *s = Some(Nop::Make());
                }
            }
        }
        result
    }
}
// Source: src/sksl/transform/SkSLEliminateDeadLocalVariables.cpp:144-156
unsafe fn eliminate(context: &Context, elements: &mut [PS], usage: &mut ProgramUsage) -> bool {
    let mut hasDead = false;
    usage.fVariableCounts.foreach(|var, c| {
        use crate::src::sksl::ir::SkSLVariable::Variable;
        if c.fVarExists != 0 && c.fRead == 0 && var.as_ref().storage() == VariableStorage::kLocal {
            hasDead = true;
        }
    });
    let mut v = Eliminator {
        context,
        usage,
        dead: Default::default(),
        changed: false,
        assignment: false,
    };
    if hasDead {
        for pe in elements {
            if pe
                .as_ref()
                .unwrap_unchecked()
                .node()
                .is::<FunctionDefinition>()
            {
                v.visitProgramElement(pe.as_mut().unwrap_unchecked().as_pin_mut());
            }
        }
    }
    v.changed
}
// Source: src/sksl/transform/SkSLEliminateDeadLocalVariables.cpp:158-162
pub unsafe fn EliminateDeadLocalVariablesForModule(
    context: &Context,
    module: &mut Module,
    usage: &mut ProgramUsage,
) -> bool {
    eliminate(context, &mut module.fElements, usage)
}
// Source: src/sksl/transform/SkSLEliminateDeadLocalVariables.cpp:164-170
pub unsafe fn EliminateDeadLocalVariables(program: &mut Program) -> bool {
    if program
        .fConfig
        .as_ref()
        .unwrap_unchecked()
        .fSettings
        .fRemoveDeadVariables
    {
        eliminate(
            program.fContext.as_ref().unwrap_unchecked(),
            &mut program.fOwnedElements,
            program.fUsage.as_deref_mut().unwrap_unchecked(),
        )
    } else {
        false
    }
}
