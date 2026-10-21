// v47 additions/API alignment are UNVERIFIED; user prohibited all validation commands.
use super::{
    ir::{
        SkSLExpression::{Expression, ExpressionExt, Kind},
        SkSLFieldAccess::FieldAccess,
        SkSLIRNode::IRNode,
        SkSLIndexExpression::IndexExpression,
        SkSLModifierFlags::ModifierFlag,
        SkSLSwizzle::{Swizzle, SwizzleComponent},
        SkSLVariable::Storage,
        SkSLVariableReference::{VariableRefKind, VariableReference},
    },
    SkSLErrorReporter::{ErrorReporter, ErrorReporterExt},
    SkSLOperator::OperatorPrecedence,
};
use core::ptr::NonNull;
// Source: src/sksl/SkSLAnalysis.h:74-74
pub use super::analysis::SkSLReturnsInputAlpha::ReturnsInputAlpha;
// Source: src/sksl/SkSLAnalysis.h:88-88
pub use super::analysis::SkSLHasSideEffects::HasSideEffects;
// Source: src/sksl/SkSLAnalysis.h:97-97
pub use super::analysis::SkSLIsDynamicallyUniformExpression::IsDynamicallyUniformExpression;
// Source: src/sksl/SkSLAnalysis.h:130-134
#[derive(Default, Clone, Copy)]
pub struct LoopControlFlowInfo {
    pub fHasContinue: bool,
    pub fHasBreak: bool,
    pub fHasReturn: bool,
}
// Source: src/sksl/SkSLAnalysis.h:135-135
pub use super::analysis::SkSLGetLoopControlFlowInfo::GetLoopControlFlowInfo;
// Source: src/sksl/SkSLAnalysis.h:195-195
pub use super::analysis::SkSLIsConstantExpression::IsConstantExpression;
// Source: src/sksl/SkSLAnalysis.h:204-204
pub use super::analysis::SkSLIsConstantExpression::ValidateIndexingForES2;
// Source: src/sksl/SkSLAnalysis.h:111-111
pub use super::analysis::SkSLSwitchCaseContainsExit::SwitchCaseContainsUnconditionalExit;
// Source: src/sksl/SkSLAnalysis.h:117-117
pub use super::analysis::SkSLSwitchCaseContainsExit::SwitchCaseContainsConditionalExit;
// Source: src/sksl/SkSLAnalysis.h:223-230
pub use super::analysis::SkSLGetLoopUnrollInfo::GetLoopUnrollInfo;
// Source: src/sksl/SkSLAnalysis.h:233-233
pub use super::analysis::SkSLCanExitWithoutReturningValue::CanExitWithoutReturningValue;
// Source: src/sksl/SkSLAnalysis.h:142-144
// Native nullable assignment target; borrowed, never a new Variable owner.
#[derive(Default)]
pub struct AssignmentInfo {
    pub fAssignedVar: Option<NonNull<VariableReference>>,
}
// Unverified actual Analysis namespace projection; implementation is the complete
// native IsSameExpressionTree body, retaining all unresolved concrete IR cases.
pub use super::analysis::SkSLIsSameExpressionTree::IsSameExpressionTree;
// Source: src/sksl/SkSLAnalysis.h:174-174
// Rust namespace projection; original implementation source retains its own mapping.
pub use super::analysis::SkSLIsTrivialExpression::IsTrivialExpression;
// UNVERIFIED frozen-scope real inherited visitors; no ProgramElement stand-ins.
use super::{
    analysis::SkSLProgramVisitor::ProgramVisitor as PV,
    ir::{
        SkSLChildCall::ChildCall,
        SkSLFunctionCall::FunctionCall,
        SkSLFunctionDefinition::FunctionDefinition,
        SkSLProgram::Program,
        SkSLProgramElement::ProgramElement,
        SkSLReturnStatement::ReturnStatement,
        SkSLStatement::Statement,
        SkSLVariable::{Variable, VariableData},
    },
    SkSLContext::Context,
    SkSLSampleUsage::SampleUsage,
};
// Source: src/sksl/SkSLAnalysis.cpp:70-150
struct MergeSampleUsageVisitor<'a> {
    fContext: &'a Context,
    fChild: &'a dyn Variable,
    fMainCoordsParam: Option<NonNull<VariableData>>,
    fWritesToSampleCoords: bool,
    fUsage: SampleUsage,
    fElidedSampleCoordCount: i32,
}
impl<'a> MergeSampleUsageVisitor<'a> {
    fn new(context: &'a Context, child: &'a dyn Variable, writes: bool) -> Self {
        Self {
            fContext: context,
            fChild: child,
            fMainCoordsParam: None,
            fWritesToSampleCoords: writes,
            fUsage: SampleUsage::default(),
            fElidedSampleCoordCount: 0,
        }
    }
    // Source: src/sksl/SkSLAnalysis.cpp:77-81
    fn visit(&mut self, program: &Program) -> SampleUsage {
        self.fUsage = SampleUsage::default();
        visitProgram(self, program);
        self.fUsage
    }
    // Source: src/sksl/SkSLAnalysis.cpp:83-83
    fn elidedSampleCoordCount(&self) -> i32 {
        self.fElidedSampleCoordCount
    }
    // Source: src/sksl/SkSLAnalysis.cpp:93-98
    unsafe fn visitProgramElement(&mut self, pe: &dyn ProgramElement) -> bool {
        self.fMainCoordsParam = if pe.node().is::<FunctionDefinition>() {
            pe.node()
                .as_node::<FunctionDefinition>()
                .declaration()
                .getMainCoordsParameter()
                .map(|v| NonNull::from(v.variable_data()))
        } else {
            None
        };
        super::analysis::SkSLProgramVisitor::inheritedProgramElement(self, pe)
    }
    // Source: src/sksl/SkSLAnalysis.cpp:100-147
    unsafe fn visitExpression(&mut self, e: &dyn Expression) -> bool {
        match e.kind() {
            Kind::kChildCall => {
                let cc = e.node().as_node::<ChildCall>();
                if NonNull::from(cc.child().variable_data())
                    == NonNull::from(self.fChild.variable_data())
                {
                    let args = cc.arguments();
                    debug_assert!(!args.as_base().as_slice().is_empty());
                    let coords = args.as_base().as_slice()[0].as_ref().unwrap_unchecked();
                    if coords
                        .expression_type()
                        .matches(&**self.fContext.fTypes.as_ref().fFloat2)
                    {
                        if !self.fWritesToSampleCoords
                            && coords.node().is::<VariableReference>()
                            && coords
                                .node()
                                .as_node::<VariableReference>()
                                .variable()
                                .map(|v| NonNull::from(v.as_ref().variable_data()))
                                == self.fMainCoordsParam
                        {
                            self.fUsage.merge(&SampleUsage::PassThrough());
                            self.fElidedSampleCoordCount += 1;
                        } else {
                            self.fUsage.merge(&SampleUsage::Explicit());
                        }
                    } else {
                        self.fUsage.merge(&SampleUsage::PassThrough());
                    }
                }
            }
            Kind::kFunctionCall => {
                let call = e.node().as_node::<FunctionCall>();
                for arg in call.arguments().as_base().as_slice() {
                    let arg = arg.as_ref().unwrap_unchecked();
                    if arg.node().is::<VariableReference>()
                        && NonNull::from(
                            arg.node()
                                .as_node::<VariableReference>()
                                .variable()
                                .unwrap_unchecked()
                                .as_ref()
                                .variable_data(),
                        ) == NonNull::from(self.fChild.variable_data())
                    {
                        self.fUsage.merge(&SampleUsage::Explicit());
                        break;
                    }
                }
            }
            _ => {}
        }
        super::analysis::SkSLProgramVisitor::inheritedExpression(self, e)
    }
}
unsafe impl PV for MergeSampleUsageVisitor<'_> {
    fn visitExpression(&mut self, e: &dyn Expression) -> bool {
        unsafe { Self::visitExpression(self, e) }
    }
    fn visitStatement(&mut self, s: &dyn Statement) -> bool {
        unsafe { super::analysis::SkSLProgramVisitor::inheritedStatement(self, s) }
    }
    fn visitProgramElement(&mut self, pe: &dyn ProgramElement) -> bool {
        unsafe { Self::visitProgramElement(self, pe) }
    }
}
// Source: src/sksl/SkSLAnalysis.cpp:153-171
struct SampleOutsideMainVisitor;
impl SampleOutsideMainVisitor {
    // Source: src/sksl/SkSLAnalysis.cpp:157-162
    unsafe fn visitExpression(&mut self, e: &dyn Expression) -> bool {
        if e.node().is::<ChildCall>() {
            return true;
        }
        super::analysis::SkSLProgramVisitor::inheritedExpression(self, e)
    }
    // Source: src/sksl/SkSLAnalysis.cpp:164-168
    unsafe fn visitProgramElement(&mut self, p: &dyn ProgramElement) -> bool {
        p.node().is::<FunctionDefinition>()
            && !p
                .node()
                .as_node::<FunctionDefinition>()
                .declaration()
                .isMain()
            && super::analysis::SkSLProgramVisitor::inheritedProgramElement(self, p)
    }
}
unsafe impl PV for SampleOutsideMainVisitor {
    fn visitExpression(&mut self, e: &dyn Expression) -> bool {
        unsafe { Self::visitExpression(self, e) }
    }
    fn visitStatement(&mut self, s: &dyn Statement) -> bool {
        unsafe { super::analysis::SkSLProgramVisitor::inheritedStatement(self, s) }
    }
    fn visitProgramElement(&mut self, pe: &dyn ProgramElement) -> bool {
        unsafe { Self::visitProgramElement(self, pe) }
    }
}
// Source: src/sksl/SkSLAnalysis.cpp:173-196
struct ReturnsNonOpaqueColorVisitor;
impl ReturnsNonOpaqueColorVisitor {
    // Source: src/sksl/SkSLAnalysis.cpp:177-187
    unsafe fn visitStatement(&mut self, s: &dyn Statement) -> bool {
        if s.node().is::<ReturnStatement>() {
            let e = s.node().as_node::<ReturnStatement>().expression();
            let knownOpaque = e.as_ref().is_some_and(|e| {
                e.expression_type().slotCount() == 4
                    && super::SkSLConstantFolder::ConstantFolder::GetConstantValueForVariable(&**e)
                        .getConstantValue(3)
                        .unwrap_or(0.0)
                        == 1.0
            });
            return !knownOpaque;
        }
        super::analysis::SkSLProgramVisitor::inheritedStatement(self, s)
    }
    // Source: src/sksl/SkSLAnalysis.cpp:189-192
    fn visitExpression(&mut self, _e: &dyn Expression) -> bool {
        false
    }
}
unsafe impl PV for ReturnsNonOpaqueColorVisitor {
    fn visitExpression(&mut self, e: &dyn Expression) -> bool {
        unsafe { Self::visitExpression(self, e) }
    }
    fn visitStatement(&mut self, s: &dyn Statement) -> bool {
        unsafe { Self::visitStatement(self, s) }
    }
    fn visitProgramElement(&mut self, pe: &dyn ProgramElement) -> bool {
        unsafe { super::analysis::SkSLProgramVisitor::inheritedProgramElement(self, pe) }
    }
}
// Source: src/sksl/SkSLAnalysis.cpp:199-228
struct NodeCountVisitor {
    fCount: i32,
    fLimit: i32,
}
impl NodeCountVisitor {
    fn new(limit: i32) -> Self {
        Self {
            fCount: 0,
            fLimit: limit,
        }
    }
    // Source: src/sksl/SkSLAnalysis.cpp:203-206
    fn visit(&mut self, s: &dyn Statement) -> i32 {
        PV::visitStatement(self, s);
        self.fCount
    }
    // Source: src/sksl/SkSLAnalysis.cpp:208-211
    unsafe fn visitExpression(&mut self, e: &dyn Expression) -> bool {
        self.fCount += 1;
        self.fCount >= self.fLimit
            || super::analysis::SkSLProgramVisitor::inheritedExpression(self, e)
    }
    // Source: src/sksl/SkSLAnalysis.cpp:213-216
    unsafe fn visitProgramElement(&mut self, p: &dyn ProgramElement) -> bool {
        self.fCount += 1;
        self.fCount >= self.fLimit
            || super::analysis::SkSLProgramVisitor::inheritedProgramElement(self, p)
    }
    // Source: src/sksl/SkSLAnalysis.cpp:218-221
    unsafe fn visitStatement(&mut self, s: &dyn Statement) -> bool {
        self.fCount += 1;
        self.fCount >= self.fLimit
            || super::analysis::SkSLProgramVisitor::inheritedStatement(self, s)
    }
}
unsafe impl PV for NodeCountVisitor {
    fn visitExpression(&mut self, e: &dyn Expression) -> bool {
        unsafe { Self::visitExpression(self, e) }
    }
    fn visitStatement(&mut self, s: &dyn Statement) -> bool {
        unsafe { Self::visitStatement(self, s) }
    }
    fn visitProgramElement(&mut self, pe: &dyn ProgramElement) -> bool {
        unsafe { Self::visitProgramElement(self, pe) }
    }
}
// Source: src/sksl/SkSLAnalysis.cpp:230-256
struct VariableWriteVisitor {
    fVar: NonNull<VariableData>,
}
impl VariableWriteVisitor {
    fn new(var: &dyn Variable) -> Self {
        Self {
            fVar: NonNull::from(var.variable_data()),
        }
    }
    // Source: src/sksl/SkSLAnalysis.cpp:235-237
    fn visit(&mut self, s: &dyn Statement) -> bool {
        PV::visitStatement(self, s)
    }
    // Source: src/sksl/SkSLAnalysis.cpp:239-250
    unsafe fn visitExpression(&mut self, e: &dyn Expression) -> bool {
        if e.node().is::<VariableReference>() {
            let reference = e.node().as_node::<VariableReference>();
            if NonNull::from(
                reference
                    .variable()
                    .unwrap_unchecked()
                    .as_ref()
                    .variable_data(),
            ) == self.fVar
                && matches!(
                    reference.refKind(),
                    VariableRefKind::kWrite
                        | VariableRefKind::kReadWrite
                        | VariableRefKind::kPointer
                )
            {
                return true;
            }
        }
        super::analysis::SkSLProgramVisitor::inheritedExpression(self, e)
    }
}
unsafe impl PV for VariableWriteVisitor {
    fn visitExpression(&mut self, e: &dyn Expression) -> bool {
        unsafe { Self::visitExpression(self, e) }
    }
    fn visitStatement(&mut self, s: &dyn Statement) -> bool {
        unsafe { super::analysis::SkSLProgramVisitor::inheritedStatement(self, s) }
    }
    fn visitProgramElement(&mut self, pe: &dyn ProgramElement) -> bool {
        unsafe { super::analysis::SkSLProgramVisitor::inheritedProgramElement(self, pe) }
    }
}
// UNVERIFIED actual lvalue visitor; no assignment-only mock or substitute IR.
// Source: src/sksl/SkSLAnalysis.cpp:261-261
// Source: src/sksl/SkSLAnalysis.cpp:335-336
struct IsAssignableVisitor {
    fErrors: NonNull<dyn ErrorReporter>,
    fAssignedVar: Option<NonNull<VariableReference>>,
}
impl IsAssignableVisitor {
    // Source: src/sksl/SkSLAnalysis.cpp:263-263
    fn new(errors: NonNull<dyn ErrorReporter>) -> Self {
        Self {
            fErrors: errors,
            fAssignedVar: None,
        }
    }
    // Source: src/sksl/SkSLAnalysis.cpp:265-272
    unsafe fn visit(
        &mut self,
        expr: *mut dyn Expression,
        info: Option<&mut AssignmentInfo>,
    ) -> bool {
        let oldErrorCount = self.fErrors.as_ref().errorCount();
        self.visitExpression(expr, None);
        if let Some(info) = info {
            info.fAssignedVar = self.fAssignedVar;
        }
        self.fErrors.as_ref().errorCount() == oldErrorCount
    }
    // Source: src/sksl/SkSLAnalysis.cpp:274-318
    unsafe fn visitExpression(
        &mut self,
        expr: *mut dyn Expression,
        fieldAccess: Option<NonNull<FieldAccess>>,
    ) {
        match (*expr).kind() {
            Kind::kVariableReference => {
                let varRef = NonNull::new_unchecked(expr as *mut () as *mut VariableReference);
                let var = varRef.as_ref().variable().unwrap_unchecked();
                let var = var.as_ref();
                let fieldName = || match fieldAccess {
                    Some(field) => field
                        .as_ref()
                        .description_with_precedence(OperatorPrecedence::kExpression),
                    None => var.name().to_vec(),
                };
                if var.modifierFlags().isConst() || var.modifierFlags().isUniform() {
                    let mut message = b"cannot modify immutable variable '".to_vec();
                    message.extend_from_slice(&fieldName());
                    message.push(b'\'');
                    self.fErrors
                        .as_mut()
                        .error((*expr).node().fPosition, &message);
                } else if var.storage() == Storage::kGlobal
                    && var.modifierFlags().0 & ModifierFlag::kIn.0 != 0
                {
                    let mut message = b"cannot modify pipeline input variable '".to_vec();
                    message.extend_from_slice(&fieldName());
                    message.push(b'\'');
                    self.fErrors
                        .as_mut()
                        .error((*expr).node().fPosition, &message);
                } else {
                    crate::SkASSERT!(@source b"src/sksl/SkSLAnalysis.cpp\0".as_ptr(),291; @condition b"fAssignedVar == nullptr\0".as_ptr(); self.fAssignedVar.is_none());
                    self.fAssignedVar = Some(varRef);
                }
            }
            Kind::kFieldAccess => {
                let f = &*(expr as *const () as *const FieldAccess);
                let child = f.base().as_ref().unwrap_unchecked().as_ptr() as *mut dyn Expression;
                self.visitExpression(child, Some(NonNull::from(f)));
            }
            Kind::kSwizzle => {
                let swizzle = &*(expr as *const () as *const Swizzle);
                self.checkSwizzleWrite(swizzle);
                self.visitExpression(
                    swizzle.base().as_ref().unwrap_unchecked().as_ptr() as *mut dyn Expression,
                    fieldAccess,
                );
            }
            Kind::kIndex => {
                let index = &*(expr as *const () as *const IndexExpression);
                self.visitExpression(
                    index.base().as_ref().unwrap_unchecked().as_ptr() as *mut dyn Expression,
                    fieldAccess,
                );
            }
            Kind::kPoison => {}
            _ => self.fErrors.as_mut().error(
                (*expr).node().fPosition,
                b"cannot assign to this expression",
            ),
        }
    }
    // Source: src/sksl/SkSLAnalysis.cpp:321-333
    unsafe fn checkSwizzleWrite(&mut self, swizzle: &Swizzle) {
        let mut bits = 0i32;
        for &idx in swizzle.components().as_slice() {
            crate::SkASSERT!(@source b"src/sksl/SkSLAnalysis.cpp\0".as_ptr(),324; @condition b"idx >= SwizzleComponent::X && idx <= SwizzleComponent::W\0".as_ptr(); idx>=SwizzleComponent::X&&idx<=SwizzleComponent::W);
            let bit = 1i32 << idx;
            if bits & bit != 0 {
                self.fErrors.as_mut().error(
                    swizzle.node().fPosition,
                    b"cannot write to the same swizzle field more than once",
                );
                break;
            }
            bits |= bit;
        }
    }
}
// New translation below is unverified by explicit user instruction.
// Source: src/sksl/SkSLAnalysis.cpp:346-356
pub fn GetSampleUsage(
    program: &Program,
    child: &dyn Variable,
    writesToSampleCoords: bool,
    elidedSampleCoordCount: Option<&mut i32>,
) -> SampleUsage {
    let mut visitor = MergeSampleUsageVisitor::new(
        program.fContext.as_ref().unwrap().as_ref(),
        child,
        writesToSampleCoords,
    );
    let result = visitor.visit(program);
    if let Some(count) = elidedSampleCoordCount {
        *count += visitor.elidedSampleCoordCount();
    }
    result
}
// Source: src/sksl/SkSLAnalysis.cpp:358-366
pub unsafe fn ReferencesBuiltin(program: &super::ir::SkSLProgram::Program, builtin: i32) -> bool {
    use super::ir::SkSLVariable::Variable;
    crate::SkASSERT!(@source b"src/sksl/SkSLAnalysis.cpp\0".as_ptr(),359; @condition b"program.fUsage\0".as_ptr(); program.fUsage.is_some());
    let mut found = false;
    program
        .fUsage
        .as_ref()
        .unwrap_unchecked()
        .fVariableCounts
        .foreach(|variable, counts| {
            if found {
                return;
            }
            if counts.fRead > 0 && variable.as_ref().layout().fBuiltin == builtin {
                found = true;
            }
        });
    found
}
// Source: src/sksl/SkSLAnalysis.cpp:368-384
pub unsafe fn ReferencesSampleCoords(program: &Program) -> bool {
    for pe in program.fOwnedElements.iter() {
        let pe = pe.as_ref().unwrap_unchecked();
        if pe.node().is::<FunctionDefinition>() {
            let function = pe.node().as_node::<FunctionDefinition>().declaration();
            if function.isMain() {
                if let Some(coords) = function.getMainCoordsParameter() {
                    return program
                        .fUsage
                        .as_ref()
                        .unwrap_unchecked()
                        .getVariable(coords)
                        .fRead
                        > 0;
                }
            }
        }
    }
    false
}
// Source: src/sksl/SkSLAnalysis.cpp:386-388
pub unsafe fn ReferencesFragCoords(program: &Program) -> bool {
    ReferencesBuiltin(program, super::SkSLCompiler::SK_FRAGCOORD_BUILTIN)
}
// Source: src/sksl/SkSLAnalysis.cpp:390-393
pub fn CallsSampleOutsideMain(program: &Program) -> bool {
    visitProgram(&mut SampleOutsideMainVisitor, program)
}
// Source: src/sksl/SkSLAnalysis.cpp:395-404
pub unsafe fn CallsColorTransformIntrinsics(program: &super::ir::SkSLProgram::Program) -> bool {
    use super::{
        ir::SkSLFunctionDeclaration::FunctionDeclaration, SkSLIntrinsicList::IntrinsicKind,
    };
    let mut found = false;
    program
        .usage()
        .unwrap_unchecked()
        .fCallCounts
        .foreach(|symbol, count| {
            if found {
                return;
            }
            let function = &*symbol.as_ptr().cast::<FunctionDeclaration>();
            if *count != 0
                && (function.intrinsicKind() == IntrinsicKind::k_toLinearSrgb_IntrinsicKind
                    || function.intrinsicKind() == IntrinsicKind::k_fromLinearSrgb_IntrinsicKind)
            {
                found = true;
            }
        });
    found
}
// Source: src/sksl/SkSLAnalysis.cpp:406-409
pub fn ReturnsOpaqueColor(function: &FunctionDefinition) -> bool {
    !PV::visitProgramElement(&mut ReturnsNonOpaqueColorVisitor, function)
}
// Source: src/sksl/SkSLAnalysis.cpp:411-427
pub unsafe fn ContainsRTAdjust(expr: &dyn Expression) -> bool {
    struct ContainsRTAdjustVisitor;
    impl ContainsRTAdjustVisitor {
        unsafe fn visitExpression(&mut self, expr: &dyn Expression) -> bool {
            if expr.node().is::<VariableReference>() {
                let v = expr
                    .node()
                    .as_node::<VariableReference>()
                    .variable()
                    .unwrap_unchecked();
                if v.as_ref().name()
                    == &super::SkSLCompiler::RTADJUST_NAME
                        [..super::SkSLCompiler::RTADJUST_NAME.len() - 1]
                {
                    return true;
                }
            }
            super::analysis::SkSLProgramVisitor::visitExpression(
                &mut |child| self.visitExpression(child),
                expr,
            )
        }
    }
    ContainsRTAdjustVisitor.visitExpression(expr)
}
// Source: src/sksl/SkSLAnalysis.cpp:429-448
pub unsafe fn ContainsVariable(
    expr: &dyn Expression,
    var: &dyn super::ir::SkSLVariable::Variable,
) -> bool {
    struct ContainsVariableVisitor {
        fVariable: NonNull<super::ir::SkSLVariable::VariableData>,
    }
    impl ContainsVariableVisitor {
        unsafe fn visitExpression(&mut self, expr: &dyn Expression) -> bool {
            if expr.node().is::<VariableReference>() {
                let v = expr
                    .node()
                    .as_node::<VariableReference>()
                    .variable()
                    .unwrap_unchecked();
                if NonNull::from(v.as_ref().variable_data()) == self.fVariable {
                    return true;
                }
            }
            super::analysis::SkSLProgramVisitor::visitExpression(
                &mut |child| self.visitExpression(child),
                expr,
            )
        }
    }
    ContainsVariableVisitor {
        fVariable: NonNull::from(var.variable_data()),
    }
    .visitExpression(expr)
}
// Source: src/sksl/SkSLAnalysis.cpp:450-485
pub unsafe fn IsCompileTimeConstant(expr: &dyn super::ir::SkSLExpression::Expression) -> bool {
    use super::ir::SkSLExpression::{Expression, ExpressionExt, Kind};
    struct IsCompileTimeConstantVisitor {
        fIsConstant: bool,
    }
    impl IsCompileTimeConstantVisitor {
        unsafe fn visitExpression(&mut self, expr: &dyn Expression) -> bool {
            match expr.kind() {
                Kind::kLiteral => false,
                Kind::kConstructorArray
                | Kind::kConstructorCompound
                | Kind::kConstructorDiagonalMatrix
                | Kind::kConstructorMatrixResize
                | Kind::kConstructorSplat
                | Kind::kConstructorStruct => {
                    visitConstructorArguments(&mut |arg| self.visitExpression(arg), expr)
                }
                _ => {
                    self.fIsConstant = false;
                    true
                }
            }
        }
    }
    let mut visitor = IsCompileTimeConstantVisitor { fIsConstant: true };
    visitor.visitExpression(expr);
    visitor.fIsConstant
}
// Source: src/sksl/SkSLAnalysis.cpp:487-518
pub unsafe fn DetectVarDeclarationWithoutScope(
    stmt: &dyn super::ir::SkSLStatement::Statement,
    mut errors: Option<NonNull<dyn ErrorReporter>>,
) -> bool {
    use super::ir::{SkSLBlock::Block, SkSLVarDeclarations::VarDeclaration};
    let var;
    if stmt.node().is::<VarDeclaration>() {
        var = stmt.node().as_node::<VarDeclaration>().var();
    } else if stmt.node().is::<Block>() {
        let block = stmt.node().as_node::<Block>();
        if block.isScope() || block.children().empty() {
            return false;
        }
        let inner = block.children().as_slice()[0].as_ref().unwrap_unchecked();
        if !inner.node().is::<VarDeclaration>() {
            return false;
        }
        var = inner.node().as_node::<VarDeclaration>().var();
    } else {
        return false;
    }
    crate::SkASSERT!(@source b"src/sksl/SkSLAnalysis.cpp\0".as_ptr(),512; @condition b"var\0".as_ptr(); var.is_some());
    if let Some(errors) = &mut errors {
        let var = var.unwrap_unchecked().as_ref();
        let mut message = b"variable '".to_vec();
        message.extend_from_slice(var.name());
        message.extend_from_slice(b"' must be created in a scope");
        errors.as_mut().error(var.node().fPosition, &message);
    }
    true
}
// Source: src/sksl/SkSLAnalysis.cpp:520-522
pub fn NodeCountUpToLimit(function: &FunctionDefinition, limit: i32) -> i32 {
    NodeCountVisitor::new(limit).visit(
        &**function
            .body()
            .as_ref()
            .expect("native nonnull function body"),
    )
}
// Source: src/sksl/SkSLAnalysis.cpp:524-526
pub fn StatementWritesToVariable(stmt: &dyn Statement, var: &dyn Variable) -> bool {
    VariableWriteVisitor::new(var).visit(stmt)
}
// Source: src/sksl/SkSLAnalysis.cpp:528-531
// Actual native NoOpErrorReporter now translated in v49; still unverified.
pub unsafe fn IsAssignable(
    expr: *mut dyn Expression,
    info: Option<&mut AssignmentInfo>,
    errors: Option<NonNull<dyn ErrorReporter>>,
) -> bool {
    use super::analysis::SkSLNoOpErrorReporter::NoOpErrorReporter;
    let mut unusedErrors = NoOpErrorReporter::new();
    let reporter =
        errors.unwrap_or_else(|| NonNull::from(&mut unusedErrors as &mut dyn ErrorReporter));
    IsAssignableVisitor::new(reporter).visit(expr, info)
}
// Source: src/sksl/SkSLAnalysis.cpp:533-549
pub unsafe fn UpdateVariableRefKind(
    expr: *mut dyn Expression,
    kind: VariableRefKind,
    mut errors: Option<NonNull<dyn ErrorReporter>>,
) -> bool {
    let mut info = AssignmentInfo::default();
    if !IsAssignable(expr, Some(&mut info), errors) {
        return false;
    }
    let Some(mut assigned) = info.fAssignedVar else {
        if let Some(errors) = &mut errors {
            let mut message = b"can't assign to expression '".to_vec();
            message.extend_from_slice(&(*expr).description());
            message.push(b'\'');
            errors.as_mut().error((*expr).node().fPosition, &message);
        }
        return false;
    };
    assigned.as_mut().setRefKind(kind);
    true
}
// Source: src/sksl/SkSLAnalysis.cpp:554-561
pub fn visitProgram(
    visitor: &mut dyn super::analysis::SkSLProgramVisitor::ProgramVisitor,
    program: &super::ir::SkSLProgram::Program,
) -> bool {
    for element in program.elements() {
        if visitor.visitProgramElement(element.expect("native nonnull program element")) {
            return true;
        }
    }
    false
}

// Source: src/sksl/SkSLAnalysis.cpp:593-607
// Adapted partial-dispatch extraction: this helper is the original constructor
// case of TProgramVisitor::visitExpression, called only after the original Kind
// switch selected a constructor. Complete general dispatch lives in SkSLProgramVisitor.
unsafe fn visitConstructorArguments(
    visit: &mut impl FnMut(&dyn super::ir::SkSLExpression::Expression) -> bool,
    expr: &dyn super::ir::SkSLExpression::Expression,
) -> bool {
    use super::ir::SkSLExpression::ExpressionExt;
    debug_assert!(expr.isAnyConstructor());
    let c = expr.constructor_view();
    for arg in c.argumentSpan() {
        if visit(&**arg.as_ref().unwrap_unchecked()) {
            return true;
        }
    }
    false
}

// Language-only downcast projection. Its exclusive borrow comes from Pin, never
// from a shared node reference. Callers only mutate child-owner slots, not move T.
unsafe fn writer_expression_node<T: crate::src::sksl::ir::SkSLIRNode::IRNodeType>(
    e: core::pin::Pin<&mut dyn crate::src::sksl::ir::SkSLExpression::Expression>,
) -> &mut T {
    e.node_mut().as_node_mut::<T>().get_unchecked_mut()
}
unsafe fn writer_statement_node<T: crate::src::sksl::ir::SkSLIRNode::IRNodeType>(
    s: core::pin::Pin<&mut dyn crate::src::sksl::ir::SkSLStatement::Statement>,
) -> &mut T {
    s.node_mut().as_node_mut::<T>().get_unchecked_mut()
}

// Source: src/sksl/SkSLAnalysis.cpp:563-638
// Mutable instantiation of the same source algorithm, no duplicate source credit.
pub unsafe fn writerVisitExpression<
    W: crate::src::sksl::transform::SkSLProgramWriter::ProgramWriter + ?Sized,
>(
    writer: &mut W,
    e: core::pin::Pin<&mut dyn crate::src::sksl::ir::SkSLExpression::Expression>,
) -> bool {
    use crate::src::sksl::ir::{
        SkSLBinaryExpression::BinaryExpression,
        SkSLChildCall::ChildCall,
        SkSLExpression::{ExpressionExt, Kind},
        SkSLFieldAccess::FieldAccess,
        SkSLFunctionCall::FunctionCall,
        SkSLIndexExpression::IndexExpression,
        SkSLPostfixExpression::PostfixExpression,
        SkSLPrefixExpression::PrefixExpression,
        SkSLSwizzle::Swizzle,
        SkSLTernaryExpression::TernaryExpression,
    };
    match e.as_ref().get_ref().kind() {
        Kind::kEmpty
        | Kind::kFunctionReference
        | Kind::kLiteral
        | Kind::kMethodReference
        | Kind::kPoison
        | Kind::kSetting
        | Kind::kTypeReference
        | Kind::kVariableReference => false,
        Kind::kBinary => {
            let b = writer_expression_node::<BinaryExpression>(e);
            (b.left().is_some() && writer.visitExpressionPtr(b.left_mut()))
                || (b.right().is_some() && writer.visitExpressionPtr(b.right_mut()))
        }
        Kind::kChildCall => {
            let c = writer_expression_node::<ChildCall>(e);
            for arg in c.arguments_mut().as_base_mut().as_mut_slice() {
                if arg.is_some() && writer.visitExpressionPtr(arg) {
                    return true;
                }
            }
            false
        }
        Kind::kConstructorArray
        | Kind::kConstructorArrayCast
        | Kind::kConstructorCompound
        | Kind::kConstructorCompoundCast
        | Kind::kConstructorDiagonalMatrix
        | Kind::kConstructorMatrixResize
        | Kind::kConstructorScalarCast
        | Kind::kConstructorSplat
        | Kind::kConstructorStruct => {
            for arg in e.constructor_view_mut().argumentSpan_mut() {
                if writer.visitExpressionPtr(arg) {
                    return true;
                }
            }
            false
        }
        Kind::kFieldAccess => {
            writer.visitExpressionPtr(writer_expression_node::<FieldAccess>(e).base_mut())
        }
        Kind::kFunctionCall => {
            let c = writer_expression_node::<FunctionCall>(e);
            for arg in c.arguments_mut().as_base_mut().as_mut_slice() {
                if arg.is_some() && writer.visitExpressionPtr(arg) {
                    return true;
                }
            }
            false
        }
        Kind::kIndex => {
            let i = writer_expression_node::<IndexExpression>(e);
            writer.visitExpressionPtr(i.base_mut()) || writer.visitExpressionPtr(i.index_mut())
        }
        Kind::kPostfix => {
            writer.visitExpressionPtr(writer_expression_node::<PostfixExpression>(e).operand_mut())
        }
        Kind::kPrefix => {
            writer.visitExpressionPtr(writer_expression_node::<PrefixExpression>(e).operand_mut())
        }
        Kind::kSwizzle => {
            let s = writer_expression_node::<Swizzle>(e);
            s.base().is_some() && writer.visitExpressionPtr(s.base_mut())
        }
        Kind::kTernary => {
            let t = writer_expression_node::<TernaryExpression>(e);
            writer.visitExpressionPtr(t.test_mut())
                || (t.ifTrue().is_some() && writer.visitExpressionPtr(t.ifTrue_mut()))
                || (t.ifFalse().is_some() && writer.visitExpressionPtr(t.ifFalse_mut()))
        }
    }
}

// Source: src/sksl/SkSLAnalysis.cpp:640-696
pub unsafe fn writerVisitStatement<
    W: crate::src::sksl::transform::SkSLProgramWriter::ProgramWriter + ?Sized,
>(
    writer: &mut W,
    s: core::pin::Pin<&mut dyn crate::src::sksl::ir::SkSLStatement::Statement>,
) -> bool {
    use crate::src::sksl::ir::{
        SkSLBlock::Block, SkSLDoStatement::DoStatement,
        SkSLExpressionStatement::ExpressionStatement, SkSLForStatement::ForStatement,
        SkSLIfStatement::IfStatement, SkSLReturnStatement::ReturnStatement, SkSLStatement::Kind,
        SkSLSwitchCase::SwitchCase, SkSLSwitchStatement::SwitchStatement,
        SkSLVarDeclarations::VarDeclaration,
    };
    match s.as_ref().get_ref().kind() {
        Kind::kBreak | Kind::kContinue | Kind::kDiscard | Kind::kNop => false,
        Kind::kBlock => {
            let b = s.node_mut().as_node_mut::<Block>();
            for stmt in b.children_mut().as_mut_slice() {
                if stmt.is_some() && writer.visitStatementPtr(stmt) {
                    return true;
                }
            }
            false
        }
        Kind::kSwitchCase => {
            writer.visitStatementPtr(writer_statement_node::<SwitchCase>(s).statement_mut())
        }
        Kind::kDo => {
            let d = writer_statement_node::<DoStatement>(s);
            writer.visitExpressionPtr(d.test_mut()) || writer.visitStatementPtr(d.statement_mut())
        }
        Kind::kExpression => writer
            .visitExpressionPtr(writer_statement_node::<ExpressionStatement>(s).expression_mut()),
        Kind::kFor => {
            let f = writer_statement_node::<ForStatement>(s);
            (f.initializer().is_some() && writer.visitStatementPtr(f.initializer_mut()))
                || (f.test().is_some() && writer.visitExpressionPtr(f.test_mut()))
                || (f.next().is_some() && writer.visitExpressionPtr(f.next_mut()))
                || writer.visitStatementPtr(f.statement_mut())
        }
        Kind::kIf => {
            let i = writer_statement_node::<IfStatement>(s);
            (i.test().is_some() && writer.visitExpressionPtr(i.test_mut()))
                || (i.ifTrue().is_some() && writer.visitStatementPtr(i.ifTrue_mut()))
                || (i.ifFalse().is_some() && writer.visitStatementPtr(i.ifFalse_mut()))
        }
        Kind::kReturn => {
            let r = writer_statement_node::<ReturnStatement>(s);
            r.expression().is_some() && writer.visitExpressionPtr(r.expression_mut())
        }
        Kind::kSwitch => {
            let sw = writer_statement_node::<SwitchStatement>(s);
            writer.visitExpressionPtr(sw.value_mut())
                || writer.visitStatementPtr(sw.caseBlock_mut())
        }
        Kind::kVarDeclaration => {
            let v = writer_statement_node::<VarDeclaration>(s);
            v.value().is_some() && writer.visitExpressionPtr(v.value_mut())
        }
    }
}

// Source: src/sksl/SkSLAnalysis.cpp:698-720
pub unsafe fn writerVisitProgramElement<
    W: crate::src::sksl::transform::SkSLProgramWriter::ProgramWriter + ?Sized,
>(
    writer: &mut W,
    pe: core::pin::Pin<&mut dyn crate::src::sksl::ir::SkSLProgramElement::ProgramElement>,
) -> bool {
    use crate::src::sksl::ir::{
        SkSLFunctionDefinition::FunctionDefinition, SkSLProgramElement::Kind,
        SkSLVarDeclarations::GlobalVarDeclaration,
    };
    match pe.as_ref().get_ref().kind() {
        Kind::kExtension
        | Kind::kFunctionPrototype
        | Kind::kInterfaceBlock
        | Kind::kModifiers
        | Kind::kStructDefinition => false,
        Kind::kFunction => {
            let f = pe
                .node_mut()
                .as_node_mut::<FunctionDefinition>()
                .get_unchecked_mut();
            writer.visitStatementPtr(f.body_mut())
        }
        Kind::kGlobalVar => {
            let v = pe
                .node_mut()
                .as_node_mut::<GlobalVarDeclaration>()
                .get_unchecked_mut();
            writer.visitStatementPtr(v.declaration_slot_mut())
        }
    }
}

// Source: src/sksl/SkSLAnalysis.h:236-243
pub use super::analysis::SkSLGetReturnComplexity::GetReturnComplexity;
// Source: src/sksl/SkSLAnalysis.h:236-240
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnComplexity {
    kSingleSafeReturn,
    kScopedReturns,
    kEarlyReturns,
}
// Source: src/sksl/SkSLAnalysis.h:261-276
// Borrowed native stack address; constructor and destructor live in their actual cpp module.
pub struct SymbolTableStackBuilder {
    pub(crate) fStackToPop: Option<
        core::ptr::NonNull<Vec<core::ptr::NonNull<super::ir::SkSLSymbolTable::SymbolTable>>>,
    >,
}
impl SymbolTableStackBuilder {
    // Source: src/sksl/SkSLAnalysis.h:270-272
    pub fn foundSymbolTable(&self) -> bool {
        self.fStackToPop.is_some()
    }
}
// Source: src/sksl/SkSLAnalysis.h:79-79
pub use super::analysis::SkSLCheckProgramStructure::CheckProgramStructure;
// Source: src/sksl/SkSLAnalysis.h:210-210
pub use super::analysis::SkSLCheckSymbolTableCorrectness::CheckSymbolTableCorrectness;
// Source: src/sksl/SkSLAnalysis.h:248-248
pub use super::analysis::SkSLFinalizationChecks::DoFinalizationChecks;
