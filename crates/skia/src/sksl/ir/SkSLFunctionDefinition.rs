use super::{
    SkSLFunctionDeclaration::FunctionDeclaration, SkSLIRNode::IRNodeBox,
    SkSLProgramElement::ProgramElementData, SkSLStatement::Statement,
};
use core::ptr::NonNull;
// Source: src/sksl/ir/SkSLFunctionDefinition.h:28-28
// Source: src/sksl/ir/SkSLFunctionDefinition.h:79-80
// Final class actual state; native Convert/Make and its real Finalizer follow
// the header implementation below, using canonical declaration and IR owners.
#[repr(C)]
pub struct FunctionDefinition {
    pub(crate) base: ProgramElementData,
    pub(crate) fDeclaration: Option<NonNull<FunctionDeclaration>>,
    pub(crate) fBody: Option<IRNodeBox<dyn Statement>>,
}
// Source: src/sksl/ir/SkSLFunctionDefinition.h:28-28
unsafe impl super::SkSLProgramElement::ProgramElement for FunctionDefinition {}
// Source: src/sksl/ir/SkSLFunctionDefinition.h:30-30
impl super::SkSLIRNode::IRNodeType for FunctionDefinition {
    const kIRNodeKind: i32 = super::SkSLIRNode::ProgramElementKind::kFunction as i32;
}
impl FunctionDefinition {
    // Source: src/sksl/ir/SkSLFunctionDefinition.h:32-37
    /// Safety: any nonnull declaration lives through this definition; body follows its original
    /// allocation-pool destruction lease. No absent FunctionDeclaration vtable is invented.
    pub unsafe fn new(
        pos: crate::src::sksl::SkSLPosition::Position,
        declaration: Option<NonNull<FunctionDeclaration>>,
        body: Option<IRNodeBox<dyn Statement>>,
    ) -> Self {
        Self {
            base: ProgramElementData::new(pos, super::SkSLIRNode::ProgramElementKind::kFunction),
            fDeclaration: declaration,
            fBody: body,
        }
    }

    // Source: src/sksl/ir/SkSLFunctionDefinition.h:62-64
    pub fn declaration(&self) -> &FunctionDeclaration {
        unsafe {
            self.fDeclaration
                .expect("native declaration reference requires a nonnull pointer")
                .as_ref()
        }
    }
    // Source: src/sksl/ir/SkSLFunctionDefinition.h:66-72
    pub fn body_mut(&mut self) -> &mut Option<IRNodeBox<dyn Statement>> {
        &mut self.fBody
    }
    pub fn body(&self) -> &Option<IRNodeBox<dyn Statement>> {
        &self.fBody
    }
}

// Source: src/sksl/ir/SkSLFunctionDefinition.h:74-76
// Actual admitted inline description closes the real inherited virtual protocol;
// cpp Convert/Make are translated below; the header constructor remains the
// actual storage constructor and does not substitute for semantic conversion.
impl super::SkSLIRNode::IRNode for FunctionDefinition {
    fn node(&self) -> &super::SkSLIRNode::IRNodeData {
        &self.base.base
    }
    fn node_mut(
        self: core::pin::Pin<&mut Self>,
    ) -> core::pin::Pin<&mut super::SkSLIRNode::IRNodeData> {
        unsafe { self.map_unchecked_mut(|value| &mut value.base.base) }
    }
    fn description(&self) -> Vec<u8> {
        let mut text = self.declaration().description();
        text.push(b' ');
        text.extend_from_slice(
            &self
                .body()
                .as_ref()
                .expect("native nonnull function body")
                .description(),
        );
        text
    }
}

// UNVERIFIED direct pinned-source translation. Unique IR owners remain attached
// to their actual allocation pool. Context/declaration/symbol borrows must outlive
// every retained native pointer; conversion exclusively visits the owned body.
use super::SkSLSymbol::Symbol;
use super::{
    SkSLBinaryExpression::BinaryExpression,
    SkSLBlock::Block,
    SkSLExpression::{Expression, Kind as ExpressionKind},
    SkSLExpressionStatement::ExpressionStatement,
    SkSLIRHelpers::IRHelpers,
    SkSLIRNode::{IRNode, IRNodeType},
    SkSLNop::Nop,
    SkSLReturnStatement::ReturnStatement,
    SkSLStatement::Kind as StatementKind,
    SkSLVarDeclarations::VarDeclaration,
    SkSLVariable::{Storage, Variable},
    SkSLVariableReference::VariableReference,
};
use crate::src::sksl::transform::SkSLProgramWriter::{self, ProgramWriter};
use crate::src::sksl::{
    SkSLAnalysis, SkSLContext::Context, SkSLErrorReporter::ErrorReporterExt,
    SkSLPosition::Position, SkSLProgramSettings::ProgramConfig,
};
type StmtSlot = Option<IRNodeBox<dyn Statement>>;
type ExprSlot = Option<IRNodeBox<dyn Expression>>;
// Native unique_ptr references reborrow the existing pinned allocation. These
// projections do not move the node or manufacture an owning allocation.
unsafe fn statement_mut<T: IRNodeType>(owner: &mut IRNodeBox<dyn Statement>) -> &mut T {
    owner
        .as_pin_mut()
        .node_mut()
        .as_node_mut::<T>()
        .get_unchecked_mut()
}
unsafe fn expression_mut<T: IRNodeType>(owner: &mut IRNodeBox<dyn Expression>) -> &mut T {
    owner
        .as_pin_mut()
        .node_mut()
        .as_node_mut::<T>()
        .get_unchecked_mut()
}

// Source: src/sksl/ir/SkSLFunctionDefinition.cpp:43-86
unsafe fn append_rtadjust_fixup_to_vertex_main(
    context: &Context,
    _decl: &FunctionDeclaration,
    body: &mut Block,
) {
    use super::{
        SkSLFieldSymbol::FieldSymbol,
        SkSLSwizzle::{ComponentArray, SwizzleComponent as C},
        SkSLSymbol::{self, Symbol},
    };
    use crate::src::sksl::SkSLCompiler::{POSITION_NAME, RTADJUST_NAME};
    let Some(rtAdjust) = context
        .fSymbolTable
        .unwrap_unchecked()
        .as_ref()
        .find(&RTADJUST_NAME[..RTADJUST_NAME.len() - 1])
    else {
        return;
    };
    // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:49-81
    struct AppendRTAdjustFixupHelper<'a> {
        base: IRHelpers<'a>,
        fSkPositionField: NonNull<FieldSymbol>,
        fRTAdjust: NonNull<dyn Symbol>,
    }
    impl<'a> AppendRTAdjustFixupHelper<'a> {
        // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:50-55
        unsafe fn new(context: &'a Context, rtAdjust: NonNull<dyn Symbol>) -> Self {
            let position = context
                .fSymbolTable
                .unwrap_unchecked()
                .as_ref()
                .find(&POSITION_NAME[..POSITION_NAME.len() - 1])
                .unwrap_unchecked();
            Self {
                base: IRHelpers::new(context),
                fSkPositionField: NonNull::from(position.as_ref().node().as_node::<FieldSymbol>()),
                fRTAdjust: rtAdjust,
            }
        }
        // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:57-59
        unsafe fn Pos(&self) -> IRNodeBox<dyn Expression> {
            let field = self.fSkPositionField.as_ref();
            let owner = core::mem::transmute::<NonNull<dyn Variable + '_>, NonNull<dyn Variable>>(
                NonNull::from(field.owner()),
            );
            self.base.Field(owner, field.fieldIndex())
        }
        // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:61-63
        unsafe fn Adjust(&self) -> IRNodeBox<dyn Expression> {
            SkSLSymbol::instantiate(self.fRTAdjust.as_ref(), self.base.fContext, Position::new())
                .unwrap_unchecked()
        }
        // Braced ComponentArray initializer projection, original X/Y/Z/W identity.
        unsafe fn components(values: &[i8]) -> ComponentArray {
            let mut c = ComponentArray::new();
            for &value in values {
                c.push_back(value);
            }
            c
        }
        // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:65-77
        unsafe fn makeFixupStmt(&self) -> IRNodeBox<dyn Statement> {
            let h = &self.base;
            h.Assign(
                self.Pos(),
                h.CtorXYZW(
                    h.Add(
                        h.Mul(
                            h.Swizzle(self.Pos(), Self::components(&[C::X, C::Y])),
                            h.Swizzle(self.Adjust(), Self::components(&[C::X, C::Z])),
                        ),
                        h.Mul(
                            h.Swizzle(self.Pos(), Self::components(&[C::W, C::W])),
                            h.Swizzle(self.Adjust(), Self::components(&[C::Y, C::W])),
                        ),
                    ),
                    h.Float(0.0),
                    h.Swizzle(self.Pos(), Self::components(&[C::W])),
                ),
            )
        }
    }
    // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:83-84
    let helper = AppendRTAdjustFixupHelper::new(context, rtAdjust);
    core::pin::Pin::new_unchecked(body)
        .children_mut()
        .push_back(Some(helper.makeFixupStmt()));
}
impl FunctionDefinition {
    // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:88-352
    /// Safety: native compilation context, symbols/types/declaration and attached
    /// IR pool remain valid; the actual unique body is exclusively mutable. The
    /// generic input projects nullable unique_ptr while accepting the parser's
    /// already nonnull IRNodeBox directly, preserving the native null-body check.
    pub unsafe fn Convert<B: Into<StmtSlot>>(
        context: &Context,
        pos: Position,
        function: &FunctionDeclaration,
        body: B,
    ) -> Option<IRNodeBox<Self>> {
        // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:92-93
        // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:300-313
        // Rust flattens native class state before method definitions. Vec's last
        // element is forward_list::front; push/pop preserve the native stack order.
        struct Finalizer<'a> {
            fContext: &'a Context,
            fFunction: &'a FunctionDeclaration,
            fBreakableLevel: i32,
            fSlotsUsed: usize,
            fContinuableLevel: Vec<i32>,
            fUninitializedVarDecl: Option<NonNull<VarDeclaration>>,
        }
        impl<'a> Finalizer<'a> {
            // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:94-101
            unsafe fn new(
                context: &'a Context,
                function: &'a FunctionDeclaration,
                pos: Position,
            ) -> Self {
                let mut this = Self {
                    fContext: context,
                    fFunction: function,
                    fBreakableLevel: 0,
                    fSlotsUsed: 0,
                    fContinuableLevel: vec![0],
                    fUninitializedVarDecl: None,
                };
                for variable in function.parameters() {
                    this.addLocalVariable(variable.unwrap_unchecked().as_ref(), pos);
                }
                this
            }
            // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:108-129
            unsafe fn addLocalVariable(&mut self, var: &dyn Variable, pos: Position) {
                let ty = var.symbol_type();
                if ty.isOrContainsUnsizedArray() {
                    if var.storage() != Storage::kParameter {
                        let mut errors = self.fContext.fErrors;
                        errors
                            .as_mut()
                            .error(pos, b"unsized arrays are not permitted here");
                    }
                    return;
                }
                let previous = self.fSlotsUsed;
                self.fSlotsUsed =
                    crate::src::core::SkSafeMath::SkSafeMath::Add(self.fSlotsUsed, ty.slotCount());
                let limit = crate::src::sksl::SkSLDefines::kVariableSlotLimit as usize;
                if previous < limit && self.fSlotsUsed >= limit {
                    let mut message = b"variable '".to_vec();
                    message.extend_from_slice(var.name());
                    message.extend_from_slice(b"' exceeds the stack size limit");
                    let mut errors = self.fContext.fErrors;
                    errors.as_mut().error(pos, &message);
                }
            }
            // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:131-197
            unsafe fn fuseVariableDeclarationsWithInitialization(&mut self, stmt: &mut StmtSlot) {
                match stmt.as_ref().unwrap_unchecked().kind() {
                    StatementKind::kNop | StatementKind::kBlock => {}
                    StatementKind::kVarDeclaration => {
                        let decl =
                            statement_mut::<VarDeclaration>(stmt.as_mut().unwrap_unchecked());
                        if decl.value().is_none() {
                            self.fUninitializedVarDecl = Some(NonNull::from(decl));
                        } else {
                            self.fUninitializedVarDecl = None;
                        }
                    }
                    StatementKind::kExpression => {
                        if let Some(mut decl) = self.fUninitializedVarDecl.take() {
                            let initializer = {
                                let expression = statement_mut::<ExpressionStatement>(
                                    stmt.as_mut().unwrap_unchecked(),
                                )
                                .expression_mut()
                                .as_mut()
                                .unwrap_unchecked();
                                if !expression.node().is::<BinaryExpression>() {
                                    return;
                                }
                                let binary = expression_mut::<BinaryExpression>(expression);
                                if binary.getOperator().kind()
                                    != crate::src::sksl::SkSLOperator::OperatorKind::EQ
                                {
                                    return;
                                }
                                let left = binary.left().as_ref().unwrap_unchecked();
                                if !left.node().is::<VariableReference>() {
                                    return;
                                }
                                let variable = left
                                    .node()
                                    .as_node::<VariableReference>()
                                    .variable()
                                    .unwrap_unchecked();
                                let declared = decl.as_ref().var().unwrap_unchecked();
                                if !core::ptr::addr_eq(variable.as_ptr(), declared.as_ptr()) {
                                    return;
                                }
                                if SkSLAnalysis::ContainsVariable(
                                    &**binary.right().as_ref().unwrap_unchecked(),
                                    variable.as_ref(),
                                ) {
                                    return;
                                }
                                binary.right_mut().take()
                            };
                            *decl.as_mut().value_mut() = initializer;
                            *stmt = Some(Nop::Make());
                        }
                    }
                    _ => self.fUninitializedVarDecl = None,
                }
            }
            // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:199-201
            fn functionReturnsValue(&self) -> bool {
                !self.fFunction.returnType().isVoid()
            }
        }
        // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:103-106
        impl Drop for Finalizer<'_> {
            fn drop(&mut self) {
                crate::SkASSERT!(@source b"src/sksl/ir/SkSLFunctionDefinition.cpp\0".as_ptr(),104; @condition b"fBreakableLevel == 0\0".as_ptr(); (self.fBreakableLevel)==(0));
                debug_assert_eq!(self.fContinuableLevel, vec![0]);
            }
        }
        unsafe impl ProgramWriter for Finalizer<'_> {
            // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:203-206
            unsafe fn visitExpressionPtr(&mut self, _expr: &mut ExprSlot) -> bool {
                false
            }
            // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:208-297
            unsafe fn visitStatementPtr(&mut self, stmt: &mut StmtSlot) -> bool {
                if self
                    .fContext
                    .fConfig
                    .unwrap_unchecked()
                    .as_ref()
                    .fSettings
                    .fOptimize
                {
                    self.fuseVariableDeclarationsWithInitialization(stmt);
                }
                let pos = stmt.as_ref().unwrap_unchecked().node().fPosition;
                match stmt.as_ref().unwrap_unchecked().kind() {
                    StatementKind::kVarDeclaration => {
                        let decl = stmt
                            .as_ref()
                            .unwrap_unchecked()
                            .node()
                            .as_node::<VarDeclaration>();
                        self.addLocalVariable(decl.var().unwrap_unchecked().as_ref(), pos);
                    }
                    StatementKind::kReturn => {
                        if ProgramConfig::IsVertex(
                            self.fContext.fConfig.unwrap_unchecked().as_ref().fKind,
                        ) && self.fFunction.isMain()
                        {
                            let mut errors = self.fContext.fErrors;
                            errors.as_mut().error(
                                pos,
                                b"early returns from vertex programs are not supported",
                            );
                        }
                        let returnsValue = self.functionReturnsValue();
                        let ret =
                            statement_mut::<ReturnStatement>(stmt.as_mut().unwrap_unchecked());
                        if ret.expression().is_some() {
                            if returnsValue {
                                let value = ret.expression_mut().take();
                                ret.setExpression(super::SkSLType::coerceExpression(
                                    self.fFunction.returnType(),
                                    value,
                                    self.fContext,
                                ));
                            } else {
                                let mut errors = self.fContext.fErrors;
                                errors.as_mut().error(
                                    ret.expression()
                                        .as_ref()
                                        .unwrap_unchecked()
                                        .node()
                                        .fPosition,
                                    b"may not return a value from a void function",
                                );
                                ret.setExpression(None);
                            }
                        } else if returnsValue {
                            let mut message = b"expected function to return '".to_vec();
                            message.extend_from_slice(&self.fFunction.returnType().displayName());
                            message.push(b'\'');
                            let mut errors = self.fContext.fErrors;
                            errors.as_mut().error(ret.node().fPosition, &message);
                        }
                    }
                    StatementKind::kDo | StatementKind::kFor => {
                        self.fBreakableLevel += 1;
                        *self.fContinuableLevel.last_mut().unwrap_unchecked() += 1;
                        let result = SkSLProgramWriter::inheritedStatement(
                            self,
                            stmt.as_mut().unwrap_unchecked().as_pin_mut(),
                        );
                        *self.fContinuableLevel.last_mut().unwrap_unchecked() -= 1;
                        self.fBreakableLevel -= 1;
                        return result;
                    }
                    StatementKind::kSwitch => {
                        self.fBreakableLevel += 1;
                        self.fContinuableLevel.push(0);
                        let result = SkSLProgramWriter::inheritedStatement(
                            self,
                            stmt.as_mut().unwrap_unchecked().as_pin_mut(),
                        );
                        self.fContinuableLevel.pop();
                        self.fBreakableLevel -= 1;
                        return result;
                    }
                    StatementKind::kBreak => {
                        if self.fBreakableLevel == 0 {
                            let mut errors = self.fContext.fErrors;
                            errors
                                .as_mut()
                                .error(pos, b"break statement must be inside a loop or switch");
                        }
                    }
                    StatementKind::kContinue => {
                        if *self.fContinuableLevel.last().unwrap_unchecked() == 0 {
                            let message: &[u8] =
                                if self.fContinuableLevel.iter().any(|&level| level > 0) {
                                    b"continue statement cannot be used in a switch"
                                } else {
                                    b"continue statement must be inside a loop"
                                };
                            let mut errors = self.fContext.fErrors;
                            errors.as_mut().error(pos, message);
                        }
                    }
                    _ => {}
                }
                SkSLProgramWriter::inheritedStatement(
                    self,
                    stmt.as_mut().unwrap_unchecked().as_pin_mut(),
                )
            }
        }
        // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:318-322
        let mut body = body.into();
        if function.isIntrinsic() {
            let mut message = b"intrinsic function '".to_vec();
            message.extend_from_slice(function.name());
            message.extend_from_slice(b"' should not have a definition");
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &message);
            return None;
        }
        // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:324-330
        if body.as_ref().is_none_or(|body| {
            !body.node().is::<Block>() || !body.node().as_node::<Block>().isScope()
        }) {
            let mut message = b"function body '".to_vec();
            message.extend_from_slice(&function.description());
            message.extend_from_slice(b"' must be a braced block");
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &message);
            return None;
        }
        // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:332-337
        if function.definition().is_some() {
            let mut message = b"function '".to_vec();
            message.extend_from_slice(&function.description());
            message.extend_from_slice(b"' was already defined");
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &message);
            return None;
        }
        // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:339-344
        {
            let mut finalizer = Finalizer::new(context, function, pos);
            finalizer.visitStatementPtr(&mut body);
        }
        if function.isMain()
            && ProgramConfig::IsVertex(context.fConfig.unwrap_unchecked().as_ref().fKind)
        {
            append_rtadjust_fixup_to_vertex_main(
                context,
                function,
                statement_mut::<Block>(body.as_mut().unwrap_unchecked()),
            );
        }
        // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:346-351
        if crate::src::sksl::analysis::SkSLCanExitWithoutReturningValue::CanExitWithoutReturningValue(function,&**body.as_ref().unwrap_unchecked()){let mut message=b"function '".to_vec();message.extend_from_slice(function.name());message.extend_from_slice(b"' can exit without returning a value");let mut errors=context.fErrors;errors.as_mut().error(body.as_ref().unwrap_unchecked().node().fPosition,&message);}
        Some(Self::Make(context, pos, function, body.unwrap_unchecked()))
    }
    // Source: src/sksl/ir/SkSLFunctionDefinition.cpp:354-363
    pub unsafe fn Make(
        _context: &Context,
        pos: Position,
        function: &FunctionDeclaration,
        body: IRNodeBox<dyn Statement>,
    ) -> IRNodeBox<Self> {
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLFunctionDefinition.cpp\0".as_ptr(),358; @condition b"!function.isIntrinsic()\0".as_ptr(); !function.isIntrinsic());
        debug_assert!(body.node().as_node::<Block>().isScope());
        debug_assert!(function.definition().is_none());
        IRNodeBox::new_with(|p: *mut Self| {
            p.write(Self::new(pos, Some(NonNull::from(function)), Some(body)))
        })
    }
}
