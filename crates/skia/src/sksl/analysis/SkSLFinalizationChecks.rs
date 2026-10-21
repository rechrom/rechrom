use crate::src::sksl::ir::SkSLIRNode::IRNode;
// UNVERIFIED: user prohibited validation.
use crate::src::sksl::{
    analysis::SkSLProgramVisitor::{self, ProgramVisitor},
    ir::{
        SkSLExpression::{Expression, ExpressionExt},
        SkSLFunctionCall::FunctionCall,
        SkSLFunctionDeclaration::FunctionDeclaration,
        SkSLFunctionDefinition::FunctionDefinition,
        SkSLProgram::Program,
        SkSLProgramElement::ProgramElement,
        SkSLStatement::Statement,
        SkSLSymbol::Symbol,
    },
    SkSLContext::Context,
    SkSLErrorReporter::ErrorReporterExt,
};
use core::ptr::NonNull;
unsafe fn error(c: &Context, p: crate::src::sksl::SkSLPosition::Position, m: &[u8]) {
    let mut e = c.fErrors;
    e.as_mut().error(p, m);
}
use crate::src::sksl::{
    analysis::SkSLProgramUsage::ProgramUsage,
    ir::{
        SkSLExpression::Kind as EK, SkSLInterfaceBlock::InterfaceBlock,
        SkSLModifierFlags::ModifierFlag as F, SkSLModifiersDeclaration::ModifiersDeclaration,
        SkSLProgramElement::Kind as PK, SkSLVarDeclarations::GlobalVarDeclaration,
    },
    SkSLPosition::Position,
    SkSLProgramSettings::ProgramConfig,
};
// Source: src/sksl/analysis/SkSLFinalizationChecks.cpp:47-201
struct Visitor<'a> {
    context: &'a Context,
    usage: &'a ProgramUsage,
    slots: usize,
    bindings: std::collections::HashSet<u64>,
    local: [i32; 3],
}
impl Visitor<'_> {
    unsafe fn checkGlobalVariableSizeLimit(&mut self, g: &GlobalVarDeclaration) {
        if !ProgramConfig::IsRuntimeEffect(self.context.fConfig.unwrap_unchecked().as_ref().fKind) {
            return;
        }
        let d = g.varDeclaration();
        let var = d.var().unwrap_unchecked().as_ref();
        let prev = self.slots;
        self.slots = crate::src::core::SkSafeMath::SkSafeMath::Add(
            self.slots,
            var.symbol_type().slotCount(),
        );
        let limit = crate::src::sksl::SkSLDefines::kVariableSlotLimit as usize;
        if prev < limit && self.slots >= limit {
            let mut msg = b"global variable '".to_vec();
            msg.extend_from_slice(var.name());
            msg.extend_from_slice(b"' exceeds the size limit");
            error(self.context, d.node().position(), &msg);
        }
    }
    unsafe fn checkBindUniqueness(&mut self, b: &InterfaceBlock) {
        let var = b.var().unwrap_unchecked().as_ref();
        let set = var.layout().fSet;
        let binding = var.layout().fBinding;
        if binding != -1 {
            let key = ((set as u64) << 32).wrapping_add(binding as u64);
            if !self.bindings.insert(key) {
                let msg = if set != -1 {
                    format!(
                        "layout(set={}, binding={}) has already been defined",
                        set, binding
                    )
                } else {
                    format!("layout(binding={}) has already been defined", binding)
                };
                error(self.context, b.node().position(), msg.as_bytes());
            }
        }
    }
    unsafe fn checkOutParamsAreAssigned(&mut self, f: &FunctionDefinition) {
        let decl = f.declaration();
        for param in decl.parameters() {
            let p = param.unwrap_unchecked().as_ref();
            if p.modifierFlags().0 & (F::kIn.0 | F::kOut.0) == F::kOut.0
                && self.usage.getVariable(p).fWrite <= 0
            {
                let mut msg = b"function '".to_vec();
                msg.extend_from_slice(decl.name());
                msg.extend_from_slice(b"' never assigns a value to out parameter '");
                msg.extend_from_slice(p.name());
                msg.push(b'\'');
                error(self.context, p.node().position(), &msg);
            }
        }
    }
    unsafe fn checkWorkgroupLocalSize(&mut self, d: &ModifiersDeclaration) {
        let values = [
            d.layout().fLocalSizeX,
            d.layout().fLocalSizeY,
            d.layout().fLocalSizeZ,
        ];
        for i in 0..3 {
            if values[i] >= 0 {
                if self.local[i] >= 0 {
                    error(
                        self.context,
                        d.node().position(),
                        format!(
                            "'local_size_{}' was specified more than once",
                            ["x", "y", "z"][i]
                        )
                        .as_bytes(),
                    );
                } else {
                    self.local[i] = values[i];
                }
            }
        }
    }
}
unsafe impl ProgramVisitor for Visitor<'_> {
    fn visitProgramElement(&mut self, pe: &dyn ProgramElement) -> bool {
        unsafe {
            match pe.kind() {
                PK::kGlobalVar => {
                    self.checkGlobalVariableSizeLimit(pe.node().as_node::<GlobalVarDeclaration>())
                }
                PK::kInterfaceBlock => {
                    self.checkBindUniqueness(pe.node().as_node::<InterfaceBlock>())
                }
                PK::kFunction => {
                    self.checkOutParamsAreAssigned(pe.node().as_node::<FunctionDefinition>())
                }
                PK::kModifiers => {
                    self.checkWorkgroupLocalSize(pe.node().as_node::<ModifiersDeclaration>())
                }
                _ => {}
            }
            SkSLProgramVisitor::inheritedProgramElement(self, pe)
        }
    }
    fn visitExpression(&mut self, e: &dyn Expression) -> bool {
        unsafe {
            match e.kind() {
                EK::kFunctionCall => {
                    let d = e.node().as_node::<FunctionCall>().function();
                    if !d.isBuiltin() && d.definition().is_none() {
                        let mut msg = b"function '".to_vec();
                        msg.extend(d.description());
                        msg.extend_from_slice(b"' is not defined");
                        error(self.context, e.node().position(), &msg);
                    }
                }
                EK::kFunctionReference | EK::kMethodReference | EK::kTypeReference => {
                    crate::SkDEBUGFAIL!(@source b"src/sksl/analysis/SkSLFinalizationChecks.cpp\0".as_ptr(),174; b"invalid reference-expr, should have been reported by coerce()\0".as_ptr());
                    error(self.context, e.node().position(), b"invalid expression");
                }
                _ => {
                    if e.expression_type()
                        .matches(&**self.context.fTypes.as_ref().fInvalid)
                    {
                        error(self.context, e.node().position(), b"invalid expression");
                    }
                }
            }
            SkSLProgramVisitor::inheritedExpression(self, e)
        }
    }
    fn visitStatement(&mut self, s: &dyn Statement) -> bool {
        unsafe { SkSLProgramVisitor::inheritedStatement(self, s) }
    }
}
// Source: src/sksl/analysis/SkSLFinalizationChecks.cpp:205-217
pub unsafe fn DoFinalizationChecks(program: &Program) {
    let context = program.fContext.as_ref().unwrap_unchecked();
    let mut v = Visitor {
        context,
        usage: program.usage().unwrap_unchecked(),
        slots: 0,
        bindings: Default::default(),
        local: [-1; 3],
    };
    for pe in program.fOwnedElements.iter() {
        v.visitProgramElement(&**pe.as_ref().unwrap_unchecked());
    }
    if ProgramConfig::IsCompute(program.fConfig.as_ref().unwrap_unchecked().fKind)
        && !v.local.iter().any(|x| *x >= 0)
    {
        error(
            context,
            Position::default(),
            b"compute programs must specify a workgroup size",
        );
    }
}
