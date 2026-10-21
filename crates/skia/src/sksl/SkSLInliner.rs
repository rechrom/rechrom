// UNVERIFIED: no compilation, tests, syntax or source checks were run.
#![cfg(not(SK_ENABLE_OPTIMIZE_SIZE))]
use super::ir::{
    SkSLBinaryExpression::BinaryExpression,
    SkSLBlock::{Block, Kind as BK},
    SkSLBreakStatement::BreakStatement,
    SkSLChildCall::ChildCall,
    SkSLContinueStatement::ContinueStatement,
    SkSLDiscardStatement::DiscardStatement,
    SkSLDoStatement::DoStatement,
    SkSLEmptyExpression::EmptyExpression,
    SkSLExpression::{Expression, ExpressionExt, Kind as EK},
    SkSLExpressionStatement::ExpressionStatement,
    SkSLFieldAccess::FieldAccess,
    SkSLForStatement::{ForStatement, LoopUnrollInfo},
    SkSLFunctionCall::FunctionCall,
    SkSLFunctionDeclaration::FunctionDeclaration,
    SkSLFunctionDefinition::FunctionDefinition,
    SkSLIRNode::*,
    SkSLIfStatement::IfStatement,
    SkSLIndexExpression::IndexExpression,
    SkSLNop::Nop,
    SkSLPostfixExpression::PostfixExpression,
    SkSLPrefixExpression::PrefixExpression,
    SkSLProgramElement::ProgramElement,
    SkSLReturnStatement::ReturnStatement,
    SkSLSetting::Setting,
    SkSLStatement::{Kind as SK, Statement},
    SkSLSwitchCase::SwitchCase,
    SkSLSwitchStatement::SwitchStatement,
    SkSLSwizzle::Swizzle,
    SkSLSymbol::Symbol,
    SkSLSymbolTable::SymbolTable,
    SkSLTernaryExpression::TernaryExpression,
    SkSLType::{cloneType, Type},
    SkSLVarDeclarations::VarDeclaration,
    SkSLVariable::Variable,
    SkSLVariableReference::{VariableRefKind, VariableReference},
};
use super::{
    analysis::SkSLProgramUsage::ProgramUsage,
    SkSLAnalysis as Analysis,
    SkSLContext::Context,
    SkSLDefines::{ExpressionArray, StatementArray},
    SkSLErrorReporter::ErrorReporterExt,
    SkSLMangler::Mangler,
    SkSLOperator::{Kind as Op, Operator},
    SkSLPosition::{ForLoopPositions, Position},
    SkSLProgramSettings::ProgramSettings,
};
use core::{pin::Pin, ptr::NonNull};
use std::collections::HashMap;
type ES = Option<IRNodeBox<dyn Expression>>;
type SS = Option<IRNodeBox<dyn Statement>>;
type VM = HashMap<usize, IRNodeBox<dyn Expression>>;
unsafe fn var_key(v: NonNull<dyn Variable>) -> usize {
    v.as_ptr() as *mut () as usize
}
unsafe fn borrowed_type(t: &dyn Type) -> NonNull<dyn Type> {
    core::mem::transmute::<NonNull<dyn Type + '_>, NonNull<dyn Type>>(NonNull::from(t))
}
unsafe fn borrowed_var(t: &dyn Variable) -> NonNull<dyn Variable> {
    core::mem::transmute::<NonNull<dyn Variable + '_>, NonNull<dyn Variable>>(NonNull::from(t))
}
// Source: src/sksl/SkSLInliner.h:44-46
// Source: src/sksl/SkSLInliner.h:54
// Source: src/sksl/SkSLInliner.h:100-103
// Source: src/sksl/SkSLInliner.h:117-119
pub struct Inliner {
    fContext: NonNull<Context>,
    fMangler: Mangler,
    fInlinedStatementCounter: i32,
}
// Source: src/sksl/SkSLInliner.h:100-103
// Adapted private field order: native implicit destruction is replacement then
// inlined body. Named constructor expressions below keep their source order.
struct InlinedCall {
    // Source: src/sksl/SkSLInliner.h:102-102
    fReplacementExpr: ES,
    // Source: src/sksl/SkSLInliner.h:101-101
    fInlinedBody: Option<IRNodeBox<Block>>,
}
impl Inliner {
    pub unsafe fn new(context: NonNull<Context>) -> Self {
        Self {
            fContext: context,
            fMangler: Mangler::default(),
            fInlinedStatementCounter: 0,
        }
    }
    // Source: src/sksl/SkSLInliner.h:56
    unsafe fn settings(&self) -> &ProgramSettings {
        &self
            .fContext
            .as_ref()
            .fConfig
            .unwrap_unchecked()
            .as_ref()
            .fSettings
    }
}
// Source: src/sksl/SkSLInliner.cpp:80-82
unsafe fn is_scopeless_block(stmt: &dyn Statement) -> bool {
    stmt.node().is::<Block>() && !stmt.node().as_node::<Block>().isScope()
}
// Source: src/sksl/SkSLInliner.cpp:84-103
unsafe fn find_parent_statement(stack: &[NonNull<SS>]) -> Option<NonNull<SS>> {
    debug_assert!(!stack.is_empty());
    for slot in stack.iter().rev().skip(1) {
        if !is_scopeless_block(&**slot.as_ref().as_ref().unwrap_unchecked()) {
            return Some(*slot);
        }
    }
    None
}
// Source: src/sksl/SkSLInliner.cpp:105-110
unsafe fn clone_with_ref_kind(
    expr: &dyn Expression,
    kind: VariableRefKind,
    pos: Position,
) -> IRNodeBox<dyn Expression> {
    let clone = expr.clone_at(pos).unwrap_unchecked();
    Analysis::UpdateVariableRefKind(clone.as_ptr() as *mut dyn Expression, kind, None);
    clone
}
impl Inliner {
    // Source: src/sksl/SkSLInliner.cpp:115-131
    unsafe fn RemapVariable(variable: NonNull<dyn Variable>, map: &VM) -> NonNull<dyn Variable> {
        let Some(remap) = map.get(&var_key(variable)) else {
            debug_assert!(false, "rewrite map lacks variable");
            return variable;
        };
        if !remap.node().is::<VariableReference>() {
            debug_assert!(false, "rewrite map contains non-variable");
            return variable;
        }
        remap
            .node()
            .as_node::<VariableReference>()
            .variable()
            .unwrap_unchecked()
    }
    // Source: src/sksl/SkSLInliner.cpp:133-167
    unsafe fn ensureScopedBlocks(
        &self,
        body: Option<NonNull<dyn Statement>>,
        parent: Option<NonNull<dyn Statement>>,
    ) {
        let Some(body) = body else {
            return;
        };
        if !body.as_ref().node().is::<Block>() {
            return;
        }
        let Some(parent) = parent else {
            return;
        };
        let parent = parent.as_ref();
        if !(parent.node().is::<IfStatement>()
            || parent.node().is::<ForStatement>()
            || parent.node().is::<DoStatement>()
            || is_scopeless_block(parent))
        {
            return;
        }
        let mut root = NonNull::new_unchecked(body.as_ptr() as *mut () as *mut Block);
        let mut nested = root;
        loop {
            let b = nested.as_ref();
            if b.isScope() {
                return;
            }
            if b.children().size() == 1
                && b.children().as_slice()[0]
                    .as_ref()
                    .unwrap_unchecked()
                    .node()
                    .is::<Block>()
            {
                nested = NonNull::new_unchecked(
                    b.children().as_slice()[0]
                        .as_ref()
                        .unwrap_unchecked()
                        .as_ptr() as *mut () as *mut Block,
                );
                continue;
            }
            Pin::new_unchecked(root.as_mut()).setBlockKind(BK::kBracedScope);
            return;
        }
    }
    // Rust nullable-owner projection of the native recursive lambda; no replacement IR.
    unsafe fn expr_slot(
        &mut self,
        pos: Position,
        map: &mut VM,
        symbols: NonNull<SymbolTable>,
        e: &ES,
    ) -> ES {
        e.as_ref()
            .and_then(|e| self.inlineExpression(pos, map, symbols, &**e))
    }
    unsafe fn args(
        &mut self,
        pos: Position,
        map: &mut VM,
        symbols: NonNull<SymbolTable>,
        original: &[ES],
    ) -> Pin<Box<ExpressionArray>> {
        let mut args = ExpressionArray::new();
        args.as_mut()
            .as_base_mut()
            .reserve_exact(original.len() as i32);
        for e in original {
            let e = self.expr_slot(pos, map, symbols, e);
            args.as_mut().as_base_mut().push_back(e);
        }
        args
    }
    // Source: src/sksl/SkSLInliner.cpp:169-351
    unsafe fn inlineExpression(
        &mut self,
        pos: Position,
        map: &mut VM,
        mut symbols: NonNull<SymbolTable>,
        e: &dyn Expression,
    ) -> ES {
        let context = self.fContext.as_ref();
        macro_rules! expr {
            ($x:expr) => {
                self.expr_slot(pos, map, symbols, $x).unwrap_unchecked()
            };
        }
        macro_rules! ty {
            () => {
                cloneType(e.expression_type(), context, symbols.as_mut()).unwrap_unchecked()
            };
        }
        macro_rules! args {
            ($x:expr) => {
                self.args(pos, map, symbols, $x)
            };
        }
        let result = match e.kind() {
            EK::kBinary => {
                let b = e.node().as_node::<BinaryExpression>();
                let l = expr!(b.left());
                let r = expr!(b.right());
                BinaryExpression::Make(context, pos, l, b.getOperator(), r)
            }
            EK::kEmpty | EK::kLiteral => return e.clone_at(pos),
            EK::kChildCall => {
                let c = e.node().as_node::<ChildCall>();
                let child = c.child();
                let child = if let Some(remap) = map.get(&var_key(borrowed_var(child))) {
                    if remap.node().is::<VariableReference>() {
                        remap
                            .node()
                            .as_node::<VariableReference>()
                            .variable()
                            .unwrap_unchecked()
                    } else {
                        debug_assert!(false, "opaque child remapped to nonvariable");
                        borrowed_var(child)
                    }
                } else {
                    borrowed_var(child)
                };
                let ty = ty!();
                let args = args!(c.arguments().as_base().as_slice());
                ChildCall::Make(context, pos, ty, child, args)
            }
            EK::kConstructorArray => {
                let ty = ty!();
                let a = args!(e.constructor_view().argumentSpan());
                super::ir::SkSLConstructorArray::ConstructorArray::Make(
                    context,
                    pos,
                    ty.as_ref(),
                    a,
                )
            }
            EK::kConstructorArrayCast => {
                let ty = ty!();
                let a = expr!(&e.constructor_view().argumentSpan()[0]);
                super::ir::SkSLConstructorArrayCast::ConstructorArrayCast::Make(
                    context,
                    pos,
                    ty.as_ref(),
                    a,
                )
            }
            EK::kConstructorCompound => {
                let ty = ty!();
                let a = args!(e.constructor_view().argumentSpan());
                super::ir::SkSLConstructorCompound::ConstructorCompound::Make(
                    context,
                    pos,
                    ty.as_ref(),
                    a,
                )
            }
            EK::kConstructorCompoundCast => {
                let ty = ty!();
                let a = expr!(&e.constructor_view().argumentSpan()[0]);
                super::ir::SkSLConstructorCompoundCast::ConstructorCompoundCast::Make(
                    context,
                    pos,
                    ty.as_ref(),
                    a,
                )
            }
            EK::kConstructorDiagonalMatrix => {
                let ty = ty!();
                let a = expr!(&e.constructor_view().argumentSpan()[0]);
                super::ir::SkSLConstructorDiagonalMatrix::ConstructorDiagonalMatrix::Make(
                    context,
                    pos,
                    ty.as_ref(),
                    a,
                )
            }
            EK::kConstructorMatrixResize => {
                let ty = ty!();
                let a = expr!(&e.constructor_view().argumentSpan()[0]);
                super::ir::SkSLConstructorMatrixResize::ConstructorMatrixResize::Make(
                    context,
                    pos,
                    ty.as_ref(),
                    a,
                )
            }
            EK::kConstructorScalarCast => {
                let ty = ty!();
                let a = expr!(&e.constructor_view().argumentSpan()[0]);
                super::ir::SkSLConstructorScalarCast::ConstructorScalarCast::Make(
                    context,
                    pos,
                    ty.as_ref(),
                    a,
                )
            }
            EK::kConstructorSplat => {
                let ty = ty!();
                let a = expr!(&e.constructor_view().argumentSpan()[0]);
                super::ir::SkSLConstructorSplat::ConstructorSplat::Make(
                    context,
                    pos,
                    ty.as_ref(),
                    a,
                )
            }
            EK::kConstructorStruct => {
                let ty = ty!();
                let a = args!(e.constructor_view().argumentSpan());
                super::ir::SkSLConstructorStruct::ConstructorStruct::Make(
                    context,
                    pos,
                    ty.as_ref(),
                    a,
                )
            }
            EK::kFieldAccess => {
                let f = e.node().as_node::<FieldAccess>();
                let base = expr!(f.base());
                FieldAccess::Make(context, pos, base, f.fieldIndex(), f.ownerKind())
            }
            EK::kFunctionCall => {
                let f = e.node().as_node::<FunctionCall>();
                let ty = ty!();
                let a = args!(f.arguments().as_base().as_slice());
                FunctionCall::Make(context, pos, ty, f.function(), a)
            }
            EK::kFunctionReference => return e.clone_at(pos),
            EK::kIndex => {
                let i = e.node().as_node::<IndexExpression>();
                let b = expr!(i.base());
                let index = expr!(i.index());
                IndexExpression::Make(context, pos, b, index)
            }
            EK::kMethodReference => return e.clone_at(pos),
            EK::kPrefix => {
                let p = e.node().as_node::<PrefixExpression>();
                let b = expr!(p.operand());
                PrefixExpression::Make(context, pos, p.getOperator(), b)
            }
            EK::kPostfix => {
                let p = e.node().as_node::<PostfixExpression>();
                let b = expr!(p.operand());
                PostfixExpression::Make(context, pos, b, p.getOperator())
            }
            EK::kSetting => Setting::Make(context, pos, e.node().as_node::<Setting>().capsPtr()),
            EK::kSwizzle => {
                let s = e.node().as_node::<Swizzle>();
                let b = expr!(s.base());
                Swizzle::Make(context, pos, b, s.components().clone())
            }
            EK::kTernary => {
                let t = e.node().as_node::<TernaryExpression>();
                let test = expr!(t.test());
                let a = expr!(t.ifTrue());
                let b = expr!(t.ifFalse());
                TernaryExpression::Make(context, pos, test, a, b)
            }
            EK::kTypeReference => return e.clone_at(pos),
            EK::kVariableReference => {
                let v = e.node().as_node::<VariableReference>();
                if let Some(remap) = map.get(&var_key(v.variable().unwrap_unchecked())) {
                    clone_with_ref_kind(&**remap, v.refKind(), pos)
                } else {
                    return e.clone_at(pos);
                }
            }
            _ => {
                debug_assert!(false, "unsupported inline expression");
                return None;
            }
        };
        Some(result)
    }
}
impl Inliner {
    // Source: src/sksl/SkSLInliner.cpp:353-532
    unsafe fn inlineStatement(
        &mut self,
        pos: Position,
        map: &mut VM,
        mut symbols: NonNull<SymbolTable>,
        result: &mut ES,
        complexity: Analysis::ReturnComplexity,
        s: &dyn Statement,
        usage: &ProgramUsage,
        builtin: bool,
    ) -> SS {
        let context = self.fContext.as_ref();
        self.fInlinedStatementCounter += 1;
        macro_rules! st {
            ($x:expr) => {
                match $x {
                    Some(s) => self.inlineStatement(
                        pos, map, symbols, result, complexity, &**s, usage, builtin,
                    ),
                    None => None,
                }
            };
        }
        macro_rules! ex {
            ($x:expr) => {
                self.expr_slot(pos, map, symbols, $x)
            };
        }
        let out: IRNodeBox<dyn Statement> = match s.kind() {
            SK::kBlock => {
                let b = s.node().as_node::<Block>();
                let mut child = Box::new(SymbolTable::with_parent(Some(symbols), builtin));
                let childptr = NonNull::from(child.as_mut());
                let mut stmts = StatementArray::new_pinned();
                stmts.as_mut().reserve_exact(b.children().size());
                for s in b.children().as_slice() {
                    let s = match s {
                        Some(s) => self.inlineStatement(
                            pos, map, childptr, result, complexity, &**s, usage, builtin,
                        ),
                        None => None,
                    };
                    stmts.as_mut().push_back(s);
                }
                Block::Make(pos, stmts, b.blockKind(), Some(child))
            }
            SK::kBreak => BreakStatement::Make(pos),
            SK::kContinue => ContinueStatement::Make(pos),
            SK::kDiscard => DiscardStatement::Make(context, pos),
            SK::kDo => {
                let d = s.node().as_node::<DoStatement>();
                let body = st!(d.statement()).unwrap_unchecked();
                let test = ex!(d.test()).unwrap_unchecked();
                DoStatement::Make(context, pos, body, test)
            }
            SK::kExpression => {
                let e = s.node().as_node::<ExpressionStatement>();
                let e = ex!(e.expression()).unwrap_unchecked();
                ExpressionStatement::Make(context, e)
            }
            SK::kFor => {
                let f = s.node().as_node::<ForStatement>();
                let mut child = Box::new(SymbolTable::with_parent(Some(symbols), builtin));
                symbols = NonNull::from(child.as_mut());
                let init = st!(f.initializer());
                let test = ex!(f.test());
                let next = ex!(f.next());
                let body = st!(f.statement()).unwrap_unchecked();
                let unroll = f.unrollInfo().map(|i| {
                    Box::new(LoopUnrollInfo {
                        fIndex: Some(Self::RemapVariable(i.fIndex.unwrap_unchecked(), map)),
                        fStart: i.fStart,
                        fDelta: i.fDelta,
                        fCount: i.fCount,
                    })
                });
                ForStatement::Make(
                    context,
                    pos,
                    ForLoopPositions::default(),
                    init,
                    test,
                    next,
                    body,
                    unroll,
                    Some(child),
                )
            }
            SK::kIf => {
                let i = s.node().as_node::<IfStatement>();
                let test = ex!(i.test()).unwrap_unchecked();
                let a = st!(i.ifTrue()).unwrap_unchecked();
                let b = st!(i.ifFalse());
                IfStatement::Make(context, pos, test, a, b)
            }
            SK::kNop => Nop::Make(),
            SK::kReturn => {
                let r = s.node().as_node::<ReturnStatement>();
                if r.expression().is_none() {
                    return Some(Nop::Make());
                }
                if (complexity as i32) <= (Analysis::ReturnComplexity::kSingleSafeReturn as i32) {
                    *result = ex!(r.expression());
                    return Some(Nop::Make());
                }
                let target = clone_with_ref_kind(
                    &**result.as_ref().unwrap_unchecked(),
                    VariableRefKind::kWrite,
                    pos,
                );
                let value = ex!(r.expression()).unwrap_unchecked();
                ExpressionStatement::Make(
                    context,
                    BinaryExpression::Make(context, pos, target, Operator::new(Op::EQ), value),
                )
            }
            SK::kSwitch => {
                let s = s.node().as_node::<SwitchStatement>();
                let value = ex!(s.value()).unwrap_unchecked();
                let block = st!(s.caseBlock()).unwrap_unchecked();
                SwitchStatement::Make(context, pos, value, block)
            }
            SK::kSwitchCase => {
                let sc = s.node().as_node::<SwitchCase>();
                let body = st!(sc.statement());
                if sc.isDefault() {
                    SwitchCase::MakeDefault(pos, body).cast(|p| p as *mut dyn Statement)
                } else {
                    SwitchCase::Make(pos, sc.value(), body).cast(|p| p as *mut dyn Statement)
                }
            }
            SK::kVarDeclaration => {
                let decl = s.node().as_node::<VarDeclaration>();
                let initial = ex!(decl.value());
                let var = decl.var().unwrap_unchecked().as_ref();
                let name = self.fMangler.uniqueName(var.name(), symbols.as_ref());
                let name = symbols.as_mut().takeOwnershipOfString(name);
                let flags = super::transform::SkSLAddConstToVarModifiers::AddConstToVarModifiers(
                    var,
                    initial.as_ref().map(|e| &**e),
                    usage,
                );
                let ty = cloneType(var.symbol_type(), context, symbols.as_mut()).unwrap_unchecked();
                let owner = <dyn Variable>::Make(
                    pos,
                    var.modifiersPosition(),
                    var.layout(),
                    flags,
                    ty,
                    name.as_ref(),
                    Vec::new(),
                    builtin,
                    var.storage(),
                );
                let cloned = NonNull::new_unchecked(owner.as_ptr() as *mut dyn Variable);
                map.insert(
                    var_key(borrowed_var(var)),
                    VariableReference::Make(pos, cloned, VariableRefKind::kRead),
                );
                let base = cloneType(decl.baseType(), context, symbols.as_mut()).unwrap_unchecked();
                let out = VarDeclaration::Make(context, cloned, base, decl.arraySize(), initial)
                    .cast(|p| p as *mut dyn Statement);
                symbols
                    .as_mut()
                    .add(context, owner.cast(|p| p as *mut dyn Symbol));
                out
            }
            _ => {
                crate::SkASSERT!(@source b"src/sksl/SkSLInliner.cpp\0".as_ptr(),529; @condition b"false\0".as_ptr(); false);
                return None;
            }
        };
        Some(out)
    }
}
// Source: src/sksl/SkSLInliner.cpp:534-550
unsafe fn argument_needs_scratch_variable(
    arg: &dyn Expression,
    param: &dyn Variable,
    usage: &ProgramUsage,
) -> bool {
    let counts = usage.getVariable(param);
    if counts.fWrite == 0 {
        if if counts.fRead > 1 {
            Analysis::IsTrivialExpression(arg)
        } else {
            !Analysis::HasSideEffects(arg)
        } {
            return false;
        }
    }
    true
}
impl Inliner {
    // Source: src/sksl/SkSLInliner.cpp:552-648
    unsafe fn inlineCall(
        &mut self,
        call: &FunctionCall,
        mut symbols: NonNull<SymbolTable>,
        usage: &ProgramUsage,
        caller: &FunctionDeclaration,
    ) -> InlinedCall {
        let context = self.fContext.as_ref();
        crate::SkASSERT!(@source b"src/sksl/SkSLInliner.cpp\0".as_ptr(),566; @condition b"this->isSafeToInline(call.function().definition(), usage)\0".as_ptr(); self.isSafeToInline(call.function().definition(),usage));
        let args = call.arguments().as_base().as_slice();
        let pos = call.node().fPosition;
        let function = call.function().definition().unwrap_unchecked();
        let body = function
            .body()
            .as_ref()
            .unwrap_unchecked()
            .node()
            .as_node::<Block>();
        let complexity = Analysis::GetReturnComplexity(function);
        let mut stmts = StatementArray::new_pinned();
        let expected = 1 + args.len() + body.children().size() as usize;
        stmts.as_mut().reserve_exact(expected as i32);
        let mut result = None;
        if (complexity as i32) > (Analysis::ReturnComplexity::kSingleSafeReturn as i32)
            && !function.declaration().returnType().isVoid()
        {
            let var = <dyn Variable>::MakeScratchVariable(
                context,
                &mut self.fMangler,
                function.declaration().name(),
                borrowed_type(function.declaration().returnType()),
                symbols.as_mut(),
                None,
            );
            stmts.as_mut().push_back(var.fVarDecl);
            result = Some(VariableReference::Make(
                Position::default(),
                var.fVarSymbol.unwrap_unchecked(),
                VariableRefKind::kRead,
            ));
        }
        let mut map = VM::new();
        for (i, arg) in args.iter().enumerate() {
            let arg = arg.as_ref().unwrap_unchecked();
            let param = function.declaration().parameters()[i].unwrap_unchecked();
            if !argument_needs_scratch_variable(&**arg, param.as_ref(), usage) {
                map.insert(var_key(param), arg.clone_current().unwrap_unchecked());
                continue;
            }
            let var = <dyn Variable>::MakeScratchVariable(
                context,
                &mut self.fMangler,
                param.as_ref().name(),
                borrowed_type(arg.expression_type()),
                symbols.as_mut(),
                arg.clone_current(),
            );
            stmts.as_mut().push_back(var.fVarDecl);
            map.insert(
                var_key(param),
                VariableReference::Make(
                    Position::default(),
                    var.fVarSymbol.unwrap_unchecked(),
                    VariableRefKind::kRead,
                ),
            );
        }
        for s in body.children().as_slice() {
            let stmt = self.inlineStatement(
                pos,
                &mut map,
                symbols,
                &mut result,
                complexity,
                &**s.as_ref().unwrap_unchecked(),
                usage,
                caller.isBuiltin(),
            );
            stmts.as_mut().push_back(stmt);
        }
        debug_assert!(stmts.size() as usize <= expected);
        let body = Block::MakeBlock(pos, stmts, BK::kUnbracedBlock, None);
        if result.is_some() {
            InlinedCall {
                fInlinedBody: Some(body),
                fReplacementExpr: result,
            }
        } else if function.declaration().returnType().isVoid() {
            InlinedCall {
                fInlinedBody: Some(body),
                fReplacementExpr: Some(EmptyExpression::Make(pos, context)),
            }
        } else {
            debug_assert!(false, "nonvoid inline function lacks any return");
            let mut msg = b"inliner found non-void function '".to_vec();
            msg.extend_from_slice(function.declaration().name());
            msg.extend_from_slice(b"' that fails to return a value on any path");
            let mut errors = context.fErrors;
            errors.as_mut().error(function.node().fPosition, &msg);
            InlinedCall {
                fInlinedBody: None,
                fReplacementExpr: None,
            }
        }
    }
    // Source: src/sksl/SkSLInliner.cpp:650-656
    unsafe fn overInlineStatementLimit(&self) -> bool {
        !self
            .fContext
            .as_ref()
            .fConfig
            .unwrap_unchecked()
            .as_ref()
            .isBuiltinCode()
            && self.fInlinedStatementCounter >= 2500
    }
    // Source: src/sksl/SkSLInliner.cpp:658-695
    unsafe fn isSafeToInline(
        &self,
        definition: Option<&FunctionDefinition>,
        usage: &ProgramUsage,
    ) -> bool {
        if self.settings().fInlineThreshold <= 0 {
            return false;
        }
        if self.overInlineStatementLimit() {
            return false;
        }
        let Some(def) = definition else {
            return false;
        };
        if def.declaration().modifierFlags().isNoInline() {
            return false;
        }
        for param in def.declaration().parameters() {
            let param = param.unwrap_unchecked().as_ref();
            if param.modifierFlags().0 & super::ir::SkSLModifierFlags::ModifierFlag::kOut.0 != 0
                || param.symbol_type().isArray()
                || param.symbol_type().isStruct()
            {
                if usage.getVariable(param).fWrite > 0 {
                    return false;
                }
            }
        }
        (Analysis::GetReturnComplexity(def) as i32)
            < (Analysis::ReturnComplexity::kEarlyReturns as i32)
    }
}
// Source: src/sksl/SkSLInliner.cpp:698-708
#[derive(Clone, Copy)]
struct InlineCandidate {
    fSymbols: NonNull<SymbolTable>,
    fParentStmt: Option<NonNull<SS>>,
    fEnclosingStmt: NonNull<SS>,
    fCandidateExpr: NonNull<ES>,
    fEnclosingFunction: NonNull<FunctionDefinition>,
}
struct InlineCandidateList {
    fCandidates: Vec<InlineCandidate>,
}
// Source: src/sksl/SkSLInliner.cpp:710-986
// Adapted private field order: native reverse destruction frees enclosing
// statement stack storage before symbol stack storage. Borrowed pointers do not
// acquire or release their pointees.
struct InlineCandidateAnalyzer {
    // Source: src/sksl/SkSLInliner.cpp:723-723
    fEnclosingFunction: Option<NonNull<FunctionDefinition>>,
    // Source: src/sksl/SkSLInliner.cpp:721-721
    fEnclosingStmtStack: Vec<NonNull<SS>>,
    // Source: src/sksl/SkSLInliner.cpp:717-717
    fSymbolTableStack: Vec<NonNull<SymbolTable>>,
    // Source: src/sksl/SkSLInliner.cpp:713-713
    fCandidateList: Option<NonNull<InlineCandidateList>>,
}
unsafe fn smut<'a, T>(slot: &mut SS) -> &'a mut T {
    &mut *(slot.as_ref().unwrap_unchecked().as_ptr() as *mut () as *mut T)
}
unsafe fn emut<'a, T>(slot: &mut ES) -> &'a mut T {
    &mut *(slot.as_ref().unwrap_unchecked().as_ptr() as *mut () as *mut T)
}
impl InlineCandidateAnalyzer {
    fn new() -> Self {
        Self {
            fCandidateList: None,
            fSymbolTableStack: Vec::new(),
            fEnclosingStmtStack: Vec::new(),
            fEnclosingFunction: None,
        }
    }
    unsafe fn visit(
        &mut self,
        elements: &mut Vec<Option<IRNodeBox<dyn ProgramElement>>>,
        symbols: NonNull<SymbolTable>,
        list: &mut InlineCandidateList,
    ) {
        self.fCandidateList = Some(NonNull::from(list));
        self.fSymbolTableStack.push(symbols);
        for pe in elements {
            self.visitProgramElement(
                pe.as_mut()
                    .unwrap_unchecked()
                    .as_pin_mut()
                    .get_unchecked_mut(),
            );
        }
        self.fSymbolTableStack.pop();
        self.fCandidateList = None;
    }
    unsafe fn visitProgramElement(&mut self, pe: &mut dyn ProgramElement) {
        if pe.node().is::<FunctionDefinition>() {
            let f = &mut *(pe as *mut dyn ProgramElement as *mut () as *mut FunctionDefinition);
            let mut shadow = false;
            for param in f.declaration().parameters() {
                if self.fSymbolTableStack[0]
                    .as_ref()
                    .find(param.unwrap_unchecked().as_ref().name())
                    .is_some()
                {
                    shadow = true;
                    break;
                }
            }
            if !shadow {
                self.fEnclosingFunction = Some(NonNull::from(&mut *f));
                self.visitStatement(f.body_mut(), true);
            }
        }
    }
    unsafe fn visitStatement(&mut self, slot: &mut SS, viable: bool) {
        if slot.is_none() {
            return;
        }
        let stack = NonNull::from(&mut self.fSymbolTableStack);
        let scoped = Analysis::SymbolTableStackBuilder::new(slot.as_ref().map(|s| &**s), stack);
        if scoped.foundSymbolTable()
            && self
                .fSymbolTableStack
                .last()
                .unwrap_unchecked()
                .as_ref()
                .wouldShadowSymbolsFrom(self.fSymbolTableStack[0].as_ref())
        {
            return;
        }
        let old = self.fEnclosingStmtStack.len();
        if viable {
            self.fEnclosingStmtStack.push(NonNull::from(&mut *slot));
        }
        match slot.as_ref().unwrap_unchecked().kind() {
            SK::kBreak | SK::kContinue | SK::kDiscard | SK::kNop => {}
            SK::kBlock => {
                let b = smut::<Block>(slot);
                for s in Pin::new_unchecked(b).children_mut().as_mut_slice() {
                    self.visitStatement(s, true);
                }
            }
            SK::kDo => self.visitStatement(smut::<DoStatement>(slot).statement_mut(), true),
            SK::kExpression => {
                self.visitExpression(smut::<ExpressionStatement>(slot).expression_mut())
            }
            SK::kFor => {
                let f = smut::<ForStatement>(slot);
                self.visitStatement(f.initializer_mut(), false);
                self.visitStatement(f.statement_mut(), true);
            }
            SK::kIf => {
                let i = smut::<IfStatement>(slot);
                self.visitExpression(i.test_mut());
                self.visitStatement(i.ifTrue_mut(), true);
                self.visitStatement(i.ifFalse_mut(), true);
            }
            SK::kReturn => self.visitExpression(smut::<ReturnStatement>(slot).expression_mut()),
            SK::kSwitch => {
                let s = smut::<SwitchStatement>(slot);
                self.visitExpression(s.value_mut());
                for sc in s.cases_mut().as_mut_slice() {
                    self.visitStatement(smut::<SwitchCase>(sc).statement_mut(), true);
                }
            }
            SK::kVarDeclaration => self.visitExpression(smut::<VarDeclaration>(slot).value_mut()),
            _ => core::hint::unreachable_unchecked(),
        }
        self.fEnclosingStmtStack.truncate(old);
    }
    unsafe fn visitExpression(&mut self, slot: &mut ES) {
        if slot.is_none() {
            return;
        }
        match slot.as_ref().unwrap_unchecked().kind() {
            EK::kFieldAccess
            | EK::kFunctionReference
            | EK::kLiteral
            | EK::kMethodReference
            | EK::kSetting
            | EK::kTypeReference
            | EK::kVariableReference => {}
            EK::kBinary => {
                let b = emut::<BinaryExpression>(slot);
                self.visitExpression(b.left_mut());
                let op = b.getOperator().kind();
                if op != Op::LOGICALAND && op != Op::LOGICALOR {
                    self.visitExpression(b.right_mut());
                }
            }
            EK::kChildCall => {
                let c = emut::<ChildCall>(slot);
                for arg in c.arguments_mut().as_base_mut().as_mut_slice() {
                    self.visitExpression(arg);
                }
            }
            EK::kConstructorArray
            | EK::kConstructorArrayCast
            | EK::kConstructorCompound
            | EK::kConstructorCompoundCast
            | EK::kConstructorDiagonalMatrix
            | EK::kConstructorMatrixResize
            | EK::kConstructorScalarCast
            | EK::kConstructorSplat
            | EK::kConstructorStruct => {
                let mut ctor = slot
                    .as_mut()
                    .unwrap_unchecked()
                    .as_pin_mut()
                    .constructor_view_mut();
                for arg in ctor.as_mut().argumentSpan_mut() {
                    self.visitExpression(arg);
                }
            }
            EK::kFunctionCall => {
                for arg in emut::<FunctionCall>(slot)
                    .arguments_mut()
                    .as_base_mut()
                    .as_mut_slice()
                {
                    self.visitExpression(arg);
                }
                self.addInlineCandidate(NonNull::from(slot));
            }
            EK::kIndex => {
                let i = emut::<IndexExpression>(slot);
                self.visitExpression(i.base_mut());
                self.visitExpression(i.index_mut());
            }
            EK::kPostfix => self.visitExpression(emut::<PostfixExpression>(slot).operand_mut()),
            EK::kPrefix => self.visitExpression(emut::<PrefixExpression>(slot).operand_mut()),
            EK::kSwizzle => self.visitExpression(emut::<Swizzle>(slot).base_mut()),
            EK::kTernary => self.visitExpression(emut::<TernaryExpression>(slot).test_mut()),
            _ => core::hint::unreachable_unchecked(),
        }
    }
    unsafe fn addInlineCandidate(&mut self, candidate: NonNull<ES>) {
        self.fCandidateList
            .as_mut()
            .unwrap_unchecked()
            .as_mut()
            .fCandidates
            .push(InlineCandidate {
                fSymbols: *self.fSymbolTableStack.last().unwrap_unchecked(),
                fParentStmt: find_parent_statement(&self.fEnclosingStmtStack),
                fEnclosingStmt: *self.fEnclosingStmtStack.last().unwrap_unchecked(),
                fCandidateExpr: candidate,
                fEnclosingFunction: self.fEnclosingFunction.unwrap_unchecked(),
            });
    }
}
// Source: src/sksl/SkSLInliner.cpp:988-990
unsafe fn candidate_func(c: &InlineCandidate) -> &FunctionDeclaration {
    c.fCandidateExpr
        .as_ref()
        .as_ref()
        .unwrap_unchecked()
        .node()
        .as_node::<FunctionCall>()
        .function()
}
impl Inliner {
    // Source: src/sksl/SkSLInliner.cpp:992-1001
    unsafe fn functionCanBeInlined(
        &self,
        f: &FunctionDeclaration,
        usage: &ProgramUsage,
        cache: &mut HashMap<usize, bool>,
    ) -> bool {
        let key = f as *const _ as usize;
        if let Some(c) = cache.get(&key) {
            return *c;
        }
        let result = self.isSafeToInline(f.definition(), usage);
        cache.insert(key, result);
        result
    }
    // Source: src/sksl/SkSLInliner.cpp:1003-1028
    unsafe fn candidateCanBeInlined(
        &self,
        c: &InlineCandidate,
        usage: &ProgramUsage,
        cache: &mut HashMap<usize, bool>,
    ) -> bool {
        let f = candidate_func(c);
        if !self.functionCanBeInlined(f, usage, cache) {
            return false;
        }
        let call = c
            .fCandidateExpr
            .as_ref()
            .as_ref()
            .unwrap_unchecked()
            .node()
            .as_node::<FunctionCall>();
        for (i, arg) in call.arguments().as_base().as_slice().iter().enumerate() {
            let arg = arg.as_ref().unwrap_unchecked();
            if arg.expression_type().isOpaque() {
                let param = f.parameters()[i].unwrap_unchecked();
                if argument_needs_scratch_variable(&**arg, param.as_ref(), usage) {
                    return false;
                }
            }
        }
        true
    }
    // Source: src/sksl/SkSLInliner.cpp:1030-1038
    unsafe fn getFunctionSize(
        &self,
        f: &FunctionDeclaration,
        cache: &mut HashMap<usize, i32>,
    ) -> i32 {
        let key = f as *const _ as usize;
        if let Some(size) = cache.get(&key) {
            return *size;
        }
        let size = Analysis::NodeCountUpToLimit(
            f.definition().unwrap_unchecked(),
            self.settings().fInlineThreshold,
        );
        cache.insert(key, size);
        size
    }
    // Source: src/sksl/SkSLInliner.cpp:1040-1102
    unsafe fn buildCandidateList(
        &self,
        elements: &mut Vec<Option<IRNodeBox<dyn ProgramElement>>>,
        symbols: NonNull<SymbolTable>,
        usage: &ProgramUsage,
        list: &mut InlineCandidateList,
    ) {
        InlineCandidateAnalyzer::new().visit(elements, symbols, list);
        let candidates = &mut list.fCandidates;
        if candidates.is_empty() {
            return;
        }
        let mut cache = HashMap::new();
        candidates.retain(|c| self.candidateCanBeInlined(c, usage, &mut cache));
        if self.settings().fInlineThreshold == i32::MAX || candidates.is_empty() {
            return;
        }
        let mut sizes = HashMap::new();
        let mut costs = HashMap::<usize, i32>::new();
        for c in candidates.iter() {
            let f = candidate_func(c);
            let size = self.getFunctionSize(f, &mut sizes);
            let cost = costs.entry(f as *const _ as usize).or_default();
            *cost += size;
        }
        candidates.retain(|c| {
            let f = candidate_func(c);
            if f.modifierFlags().isInline() {
                return true;
            }
            if usage.getFunction(f) == 1 {
                return true;
            }
            costs[&(f as *const _ as usize)] <= self.settings().fInlineThreshold
        });
    }
    // Source: src/sksl/SkSLInliner.cpp:1104-1183
    /// Safety: actual IR owners, symbol tables, context and usage are exclusively
    /// leased; external references must not access mutable nodes during this pass.
    pub unsafe fn analyze(
        &mut self,
        elements: &mut Vec<Option<IRNodeBox<dyn ProgramElement>>>,
        symbols: NonNull<SymbolTable>,
        usage: &mut ProgramUsage,
    ) -> bool {
        if self.settings().fInlineThreshold <= 0 {
            return false;
        }
        if self.overInlineStatementLimit() {
            return false;
        }
        let mut list = InlineCandidateList {
            fCandidates: Vec::new(),
        };
        self.buildCandidateList(elements, symbols, usage, &mut list);
        let mut remappings = HashMap::<usize, NonNull<SS>>::new();
        let mut changed = false;
        for c in list.fCandidates {
            let call = c
                .fCandidateExpr
                .as_ref()
                .as_ref()
                .unwrap_unchecked()
                .node()
                .as_node::<FunctionCall>();
            let mut inlined = self.inlineCall(
                call,
                c.fSymbols,
                usage,
                c.fEnclosingFunction.as_ref().declaration(),
            );
            if inlined.fInlinedBody.is_none() && inlined.fReplacementExpr.is_none() {
                break;
            }
            let bodyptr = inlined
                .fInlinedBody
                .as_ref()
                .map(|b| NonNull::new_unchecked(b.as_ptr() as *mut Block as *mut dyn Statement));
            let parent = c.fParentStmt.and_then(|s| {
                s.as_ref()
                    .as_ref()
                    .map(|s| NonNull::new_unchecked(s.as_ptr() as *mut dyn Statement))
            });
            self.ensureScopedBlocks(bodyptr, parent);
            usage.addStatement(&**inlined.fInlinedBody.as_ref().unwrap_unchecked());
            let mut enclosing = c.fEnclosingStmt;
            while let Some(mapped) = remappings.get(&(enclosing.as_ptr() as usize)) {
                enclosing = *mapped;
            }
            let mut body = inlined.fInlinedBody.take().unwrap_unchecked();
            body.as_pin_mut()
                .children_mut()
                .push_back(enclosing.as_mut().take());
            *enclosing.as_mut() = Some(body.cast(|p| p as *mut dyn Statement));
            usage.removeExpression(&**c.fCandidateExpr.as_ref().as_ref().unwrap_unchecked());
            usage.addExpression(&**inlined.fReplacementExpr.as_ref().unwrap_unchecked());
            *c.fCandidateExpr.as_ptr() = inlined.fReplacementExpr.take();
            changed = true;
            let b = smut::<Block>(enclosing.as_mut());
            let mut children = Pin::new_unchecked(b).children_mut();
            let slot = NonNull::from(
                children
                    .as_mut()
                    .as_mut_slice()
                    .last_mut()
                    .unwrap_unchecked(),
            );
            remappings.insert(enclosing.as_ptr() as usize, slot);
            if self.overInlineStatementLimit() {
                break;
            }
        }
        changed
    }
}
