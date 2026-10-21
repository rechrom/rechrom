// UNVERIFIED: no compilation, tests, syntax or source checks were run.
use super::SkSLRasterPipelineBuilder::{Builder, BuilderOp, Program as RPProgram, Slot, SlotRange};
use crate::src::sksl::{
    ir::{
        SkSLBinaryExpression::BinaryExpression,
        SkSLBlock::Block,
        SkSLBreakStatement::BreakStatement,
        SkSLChildCall::ChildCall,
        SkSLContinueStatement::ContinueStatement,
        SkSLDoStatement::DoStatement,
        SkSLExpression::{Expression, ExpressionExt, Kind as EK},
        SkSLExpressionStatement::ExpressionStatement,
        SkSLFieldAccess::FieldAccess,
        SkSLForStatement::ForStatement,
        SkSLFunctionCall::FunctionCall,
        SkSLFunctionDeclaration::FunctionDeclaration,
        SkSLFunctionDefinition::FunctionDefinition,
        SkSLIRNode::*,
        SkSLIfStatement::IfStatement,
        SkSLIndexExpression::IndexExpression,
        SkSLLiteral::Literal,
        SkSLPostfixExpression::PostfixExpression,
        SkSLPrefixExpression::PrefixExpression,
        SkSLProgram::Program,
        SkSLProgramElement::ProgramElement,
        SkSLReturnStatement::ReturnStatement,
        SkSLStatement::{Kind as SK, Statement},
        SkSLSwitchStatement::SwitchStatement,
        SkSLSwizzle::Swizzle,
        SkSLSymbol::Symbol,
        SkSLTernaryExpression::TernaryExpression,
        SkSLType::{NumberKind, Type, TypeKind},
        SkSLVarDeclarations::{GlobalVarDeclaration, VarDeclaration},
        SkSLVariable::Variable,
        SkSLVariableReference::VariableReference,
    },
    tracing::SkSLDebugTracePriv::{DebugTracePriv, FunctionDebugInfo, SlotDebugInfo},
    SkSLAnalysis as Analysis,
    SkSLConstantFolder::ConstantFolder,
    SkSLContext::Context,
    SkSLIntrinsicList::IntrinsicKind,
    SkSLOperator::{Kind as Op, Operator},
    SkSLPosition::Position,
};
use core::{marker::PhantomPinned, pin::Pin, ptr::NonNull};
use std::collections::{HashMap, HashSet};
type ES = Option<IRNodeBox<dyn Expression>>;
fn node_key(node: &dyn IRNode) -> usize {
    node.node() as *const IRNodeData as usize
}
fn var_key(var: &dyn Variable) -> usize {
    var.variable_data() as *const _ as usize
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:86-89
fn unsupported() -> bool {
    false
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:95-146
struct SlotManager {
    fSlotMap: HashMap<usize, SlotRange>,
    fSlotCount: i32,
    fSlotDebugInfo: Option<NonNull<Vec<SlotDebugInfo>>>,
}
impl SlotManager {
    fn new(info: Option<NonNull<Vec<SlotDebugInfo>>>) -> Self {
        Self {
            fSlotMap: HashMap::new(),
            fSlotCount: 0,
            fSlotDebugInfo: info,
        }
    }
    fn slotCount(&self) -> i32 {
        self.fSlotCount
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:148-179
struct AutoStack {
    fGenerator: NonNull<Generator>,
    fStackID: i32,
    fParentStackID: i32,
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:181-562
// Actual shared generator state, pinned before any AutoStack is created. Context
// borrows the real source program's compiler graph; this object never substitutes it.
// Adapted private field order: after the existing native destructor resets
// fTraceMask, automatic field cleanup runs in reverse native declaration order.
// This is not a stage/payload/arena ABI type. Constructor evaluation stays in
// native forward order below, and pinning keeps each installed field stationary.
struct Generator {
    _pin: PhantomPinned,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:507-507
    fLineOffsets: Vec<i32>,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:503-503
    fInsideCompoundStatement: i32,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:499-499
    fImmutableVariables: HashSet<usize>,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:498-498
    fImmutableSlotMap: HashMap<i32, crate::src::core::SkTHash::THashSet<Slot>>,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:496-496
    fReturnComplexityMap: HashMap<usize, Analysis::ReturnComplexity>,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:494-494
    fRecycledStacks: Vec<i32>,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:493-493
    fNextStackID: i32,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:492-492
    fCurrentStack: i32,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:491-491
    fCurrentBreakTarget: i32,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:490-490
    fCurrentContinueMask: Option<NonNull<AutoContinueMask>>,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:489-489
    fCurrentFunctionResult: SlotRange,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:488-488
    fCurrentFunction: Option<NonNull<FunctionDefinition>>,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:487-487
    fTraceMask: Option<AutoStack>,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:485-485
    fImmutableSlots: SlotManager,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:484-484
    fUniformSlots: SlotManager,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:483-483
    fProgramSlots: SlotManager,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:481-481
    fChildEffectMap: HashMap<usize, i32>,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:480-480
    fWriteTraceOps: bool,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:479-479
    fDebugTrace: Option<NonNull<DebugTracePriv>>,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:478-478
    fBuilder: Builder,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:477-477
    fContext: Context,
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:476-476
    fProgram: NonNull<Program>,
}
impl Generator {
    unsafe fn new(
        program: &Program,
        mut trace: Option<NonNull<DebugTracePriv>>,
        writeTraceOps: bool,
    ) -> Pin<Box<Self>> {
        let original = program.fContext.as_ref().unwrap_unchecked();
        let mut context = Context::new(original.fTypes.as_ref(), original.fErrors);
        context.fConfig = program.fConfig.as_ref().map(|c| NonNull::from(&**c));
        context.fModule = original.fModule;
        let slots = trace
            .as_mut()
            .map(|t| NonNull::from(&mut t.as_mut().fSlotInfo));
        let uniforms = trace
            .as_mut()
            .map(|t| NonNull::from(&mut t.as_mut().fUniformInfo));
        Box::pin(Self {
            fProgram: NonNull::from(program),
            fContext: context,
            fBuilder: Builder::new(),
            fDebugTrace: trace,
            fWriteTraceOps: writeTraceOps,
            fChildEffectMap: HashMap::new(),
            fProgramSlots: SlotManager::new(slots),
            fUniformSlots: SlotManager::new(uniforms),
            fImmutableSlots: SlotManager::new(None),
            fTraceMask: None,
            fCurrentFunction: None,
            fCurrentFunctionResult: SlotRange { index: 0, count: 0 },
            fCurrentContinueMask: None,
            fCurrentBreakTarget: -1,
            fCurrentStack: 0,
            fNextStackID: 0,
            fRecycledStacks: Vec::new(),
            fReturnComplexityMap: HashMap::new(),
            fImmutableSlotMap: HashMap::new(),
            fImmutableVariables: HashSet::new(),
            fInsideCompoundStatement: 0,
            fLineOffsets: Vec::new(),
            _pin: PhantomPinned,
        })
    }
    fn shouldWriteTraceOps(&self) -> bool {
        self.fDebugTrace.is_some() && self.fWriteTraceOps
    }
    fn traceMaskStackID(&self) -> i32 {
        self.fTraceMask.as_ref().unwrap().fStackID
    }
    fn IsUniform(var: &dyn Variable) -> bool {
        var.modifierFlags().isUniform()
    }
    fn IsOutParameter(var: &dyn Variable) -> bool {
        use crate::src::sksl::ir::SkSLModifierFlags::ModifierFlag as F;
        var.modifierFlags().0 & (F::kIn.0 | F::kOut.0) == F::kOut.0
    }
    fn IsInoutParameter(var: &dyn Variable) -> bool {
        use crate::src::sksl::ir::SkSLModifierFlags::ModifierFlag as F;
        var.modifierFlags().0 & (F::kIn.0 | F::kOut.0) == (F::kIn.0 | F::kOut.0)
    }
    fn hasVariableSlots(&self, v: &dyn Variable) -> bool {
        !Self::IsUniform(v) && !self.fImmutableVariables.contains(&var_key(v))
    }
    unsafe fn getVariableSlots(&mut self, v: &dyn Variable) -> SlotRange {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp\0".as_ptr(),228; @condition b"this->hasVariableSlots(v)\0".as_ptr(); self.hasVariableSlots(v));
        self.fProgramSlots.getVariableSlots(v)
    }
    unsafe fn getImmutableSlots(&mut self, v: &dyn Variable) -> SlotRange {
        debug_assert!(!Self::IsUniform(v) && self.fImmutableVariables.contains(&var_key(v)));
        self.fImmutableSlots.getVariableSlots(v)
    }
    unsafe fn getUniformSlots(&mut self, v: &dyn Variable) -> SlotRange {
        debug_assert!(Self::IsUniform(v) && !self.fImmutableVariables.contains(&var_key(v)));
        self.fUniformSlots.getVariableSlots(v)
    }
    unsafe fn getFunctionSlots(&mut self, site: &dyn IRNode, f: &FunctionDeclaration) -> SlotRange {
        self.fProgramSlots.getFunctionSlots(site, f)
    }
    unsafe fn returnComplexity(&mut self, f: &FunctionDefinition) -> Analysis::ReturnComplexity {
        *self
            .fReturnComplexityMap
            .entry(node_key(f))
            .or_insert_with(|| Analysis::GetReturnComplexity(f))
    }
    unsafe fn needsReturnMask(&mut self, f: &FunctionDefinition) -> bool {
        self.returnComplexity(f) as i32 >= Analysis::ReturnComplexity::kEarlyReturns as i32
    }
    unsafe fn needsFunctionResultSlots(&mut self, f: &FunctionDefinition) -> bool {
        self.shouldWriteTraceOps()
            || self.returnComplexity(f) as i32
                > Analysis::ReturnComplexity::kSingleSafeReturn as i32
    }
    fn popToSlotRange(&mut self, r: SlotRange) {
        self.fBuilder.pop_slots(r);
        if self.shouldWriteTraceOps() {
            self.fBuilder.trace_var(self.traceMaskStackID(), r);
        }
    }
    fn popToSlotRangeUnmasked(&mut self, r: SlotRange) {
        self.fBuilder.pop_slots_unmasked(r);
        if self.shouldWriteTraceOps() {
            self.fBuilder.trace_var(self.traceMaskStackID(), r);
        }
    }
    fn discardExpression(&mut self, slots: i32) {
        self.fBuilder.discard_stack(slots);
    }
    fn zeroSlotRangeUnmasked(&mut self, r: SlotRange) {
        self.fBuilder.zero_slots_unmasked(r);
        if self.shouldWriteTraceOps() {
            self.fBuilder.trace_var(self.traceMaskStackID(), r);
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:199-203
impl Drop for Generator {
    fn drop(&mut self) {
        drop(self.fTraceMask.take());
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:500-560
#[derive(Clone, Copy)]
struct TypedOps {
    fFloatOp: BuilderOp,
    fSignedOp: BuilderOp,
    fUnsignedOp: BuilderOp,
    fBooleanOp: BuilderOp,
}
const kAddOps: TypedOps = TypedOps {
    fFloatOp: BuilderOp::add_n_floats,
    fSignedOp: BuilderOp::add_n_ints,
    fUnsignedOp: BuilderOp::add_n_ints,
    fBooleanOp: BuilderOp::unsupported,
};
const kSubtractOps: TypedOps = TypedOps {
    fFloatOp: BuilderOp::sub_n_floats,
    fSignedOp: BuilderOp::sub_n_ints,
    fUnsignedOp: BuilderOp::sub_n_ints,
    fBooleanOp: BuilderOp::unsupported,
};
const kMultiplyOps: TypedOps = TypedOps {
    fFloatOp: BuilderOp::mul_n_floats,
    fSignedOp: BuilderOp::mul_n_ints,
    fUnsignedOp: BuilderOp::mul_n_ints,
    fBooleanOp: BuilderOp::unsupported,
};
const kDivideOps: TypedOps = TypedOps {
    fFloatOp: BuilderOp::div_n_floats,
    fSignedOp: BuilderOp::div_n_ints,
    fUnsignedOp: BuilderOp::div_n_uints,
    fBooleanOp: BuilderOp::unsupported,
};
const kLessThanOps: TypedOps = TypedOps {
    fFloatOp: BuilderOp::cmplt_n_floats,
    fSignedOp: BuilderOp::cmplt_n_ints,
    fUnsignedOp: BuilderOp::cmplt_n_uints,
    fBooleanOp: BuilderOp::unsupported,
};
const kLessThanEqualOps: TypedOps = TypedOps {
    fFloatOp: BuilderOp::cmple_n_floats,
    fSignedOp: BuilderOp::cmple_n_ints,
    fUnsignedOp: BuilderOp::cmple_n_uints,
    fBooleanOp: BuilderOp::unsupported,
};
const kEqualOps: TypedOps = TypedOps {
    fFloatOp: BuilderOp::cmpeq_n_floats,
    fSignedOp: BuilderOp::cmpeq_n_ints,
    fUnsignedOp: BuilderOp::cmpeq_n_ints,
    fBooleanOp: BuilderOp::cmpeq_n_ints,
};
const kNotEqualOps: TypedOps = TypedOps {
    fFloatOp: BuilderOp::cmpne_n_floats,
    fSignedOp: BuilderOp::cmpne_n_ints,
    fUnsignedOp: BuilderOp::cmpne_n_ints,
    fBooleanOp: BuilderOp::cmpne_n_ints,
};
const kModOps: TypedOps = TypedOps {
    fFloatOp: BuilderOp::mod_n_floats,
    fSignedOp: BuilderOp::unsupported,
    fUnsignedOp: BuilderOp::unsupported,
    fBooleanOp: BuilderOp::unsupported,
};
const kMinOps: TypedOps = TypedOps {
    fFloatOp: BuilderOp::min_n_floats,
    fSignedOp: BuilderOp::min_n_ints,
    fUnsignedOp: BuilderOp::min_n_uints,
    fBooleanOp: BuilderOp::min_n_uints,
};
const kMaxOps: TypedOps = TypedOps {
    fFloatOp: BuilderOp::max_n_floats,
    fSignedOp: BuilderOp::max_n_ints,
    fUnsignedOp: BuilderOp::max_n_uints,
    fBooleanOp: BuilderOp::max_n_uints,
};
const kMixOps: TypedOps = TypedOps {
    fFloatOp: BuilderOp::mix_n_floats,
    fSignedOp: BuilderOp::unsupported,
    fUnsignedOp: BuilderOp::unsupported,
    fBooleanOp: BuilderOp::unsupported,
};
const kInverseSqrtOps: TypedOps = TypedOps {
    fFloatOp: BuilderOp::invsqrt_float,
    fSignedOp: BuilderOp::unsupported,
    fUnsignedOp: BuilderOp::unsupported,
    fBooleanOp: BuilderOp::unsupported,
};
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:564-593
impl AutoStack {
    unsafe fn new(mut g: NonNull<Generator>) -> Self {
        let id = g.as_mut().createStack();
        Self {
            fGenerator: g,
            fStackID: id,
            fParentStackID: 0,
        }
    }
    unsafe fn enter(&mut self) {
        self.fParentStackID = self.fGenerator.as_ref().fCurrentStack;
        self.fGenerator.as_mut().setCurrentStack(self.fStackID);
    }
    unsafe fn exit(&mut self) {
        debug_assert!(self.fGenerator.as_ref().fCurrentStack == self.fStackID);
        self.fGenerator
            .as_mut()
            .setCurrentStack(self.fParentStackID);
    }
    unsafe fn pushClone(&self, n: i32) {
        self.pushCloneRange(SlotRange { index: 0, count: n }, n);
    }
    unsafe fn pushCloneRange(&self, r: SlotRange, offset: i32) {
        let mut g = self.fGenerator;
        g.as_mut()
            .fBuilder
            .push_clone_from_stack(r, self.fStackID, offset);
    }
    unsafe fn pushCloneIndirect(&self, r: SlotRange, dynamic: i32, offset: i32) {
        let mut g = self.fGenerator;
        g.as_mut()
            .fBuilder
            .push_clone_indirect_from_stack(r, dynamic, self.fStackID, offset);
    }
}
impl Drop for AutoStack {
    fn drop(&mut self) {
        unsafe {
            self.fGenerator.as_mut().recycleStack(self.fStackID);
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:595-648
struct AutoContinueMask {
    fContinueMaskStack: Option<AutoStack>,
    fGenerator: NonNull<Generator>,
    fPreviousContinueMask: Option<NonNull<AutoContinueMask>>,
    _pin: PhantomPinned,
}
impl AutoContinueMask {
    fn new(g: NonNull<Generator>) -> Pin<Box<Self>> {
        Box::pin(Self {
            fContinueMaskStack: None,
            fGenerator: g,
            fPreviousContinueMask: None,
            _pin: PhantomPinned,
        })
    }
    unsafe fn enable(self: Pin<&mut Self>) {
        let s = self.get_unchecked_mut();
        debug_assert!(s.fContinueMaskStack.is_none());
        s.fContinueMaskStack = Some(AutoStack::new(s.fGenerator));
        s.fPreviousContinueMask = s.fGenerator.as_ref().fCurrentContinueMask;
        let ptr = NonNull::from(&mut *s);
        s.fGenerator.as_mut().fCurrentContinueMask = Some(ptr);
    }
    unsafe fn enter(&mut self) {
        self.fContinueMaskStack.as_mut().unwrap_unchecked().enter();
    }
    unsafe fn exit(&mut self) {
        self.fContinueMaskStack.as_mut().unwrap_unchecked().exit();
    }
    unsafe fn enterLoopBody(&mut self) {
        if let Some(stack) = self.fContinueMaskStack.as_mut() {
            stack.enter();
            self.fGenerator.as_mut().fBuilder.push_constant_i(0, 1);
            stack.exit();
        }
    }
    unsafe fn exitLoopBody(&mut self) {
        if let Some(stack) = self.fContinueMaskStack.as_mut() {
            stack.enter();
            self.fGenerator
                .as_mut()
                .fBuilder
                .pop_and_reenable_loop_mask();
            stack.exit();
        }
    }
    fn stackID(&self) -> i32 {
        self.fContinueMaskStack.as_ref().unwrap().fStackID
    }
}
impl Drop for AutoContinueMask {
    fn drop(&mut self) {
        if let Some(previous) = self.fPreviousContinueMask {
            unsafe {
                self.fGenerator.as_mut().fCurrentContinueMask = Some(previous);
            }
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:650-671
struct AutoLoopTarget {
    fLoopTargetPtr: NonNull<i32>,
    fPreviousLoopTarget: i32,
    fLabelID: i32,
}
impl AutoLoopTarget {
    unsafe fn new(mut g: NonNull<Generator>, mut target: NonNull<i32>) -> Self {
        let label = g.as_mut().fBuilder.nextLabelID();
        let previous = *target.as_ref();
        *target.as_mut() = label;
        Self {
            fLoopTargetPtr: target,
            fPreviousLoopTarget: previous,
            fLabelID: label,
        }
    }
}
impl Drop for AutoLoopTarget {
    fn drop(&mut self) {
        unsafe {
            *self.fLoopTargetPtr.as_mut() = self.fPreviousLoopTarget;
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:673-1107
// Rust enum dispatch represents the native final LValue subclasses; every
// variant retains its actual parent owner/borrow and expression/stack leases.
enum LValueParent {
    Owned(Box<LValue>),
    Unowned(NonNull<LValue>),
}
impl LValueParent {
    unsafe fn get(&mut self) -> &mut LValue {
        match self {
            Self::Owned(p) => p,
            Self::Unowned(p) => p.as_mut(),
        }
    }
}
enum LValueKind {
    Scratch {
        expr: NonNull<dyn Expression>,
        slots: i32,
        g: Option<NonNull<Generator>>,
        stack: Option<AutoStack>,
    },
    Variable {
        var: NonNull<dyn Variable>,
        immutable: bool,
    },
    Swizzle {
        parent: Box<LValue>,
        components: Vec<i8>,
    },
    Slice {
        parent: LValueParent,
        initial: i32,
        slots: i32,
    },
    Dynamic {
        g: Option<NonNull<Generator>>,
        parent: Box<LValue>,
        stack: Option<AutoStack>,
        expr: NonNull<IndexExpression>,
    },
}
struct LValue {
    kind: Option<LValueKind>,
    fScratchExpression: ES,
}
impl LValue {
    fn new(kind: LValueKind) -> Box<Self> {
        unsafe {
            match &kind {
                LValueKind::Swizzle { components, .. } => {
                    debug_assert!(!components.is_empty() && components.len() <= 4)
                }
                LValueKind::Slice { initial, slots, .. } => {
                    debug_assert!(*initial >= 0 && *slots > 0)
                }
                LValueKind::Dynamic { expr, .. } => debug_assert!(expr
                    .as_ref()
                    .index()
                    .as_ref()
                    .unwrap_unchecked()
                    .expression_type()
                    .isInteger()),
                _ => {}
            }
        }
        Box::new(Self {
            kind: Some(kind),
            fScratchExpression: None,
        })
    }
    unsafe fn isWritable(&self) -> bool {
        match self.kind.as_ref().unwrap_unchecked() {
            LValueKind::Scratch { .. } => false,
            LValueKind::Variable { var, immutable } => {
                !*immutable && !Generator::IsUniform(var.as_ref())
            }
            LValueKind::Swizzle { parent, .. } => parent.isWritable(),
            LValueKind::Slice { parent, .. } => match parent {
                LValueParent::Owned(p) => p.isWritable(),
                LValueParent::Unowned(p) => p.as_ref().isWritable(),
            },
            LValueKind::Dynamic { parent, .. } => parent.isWritable(),
        }
    }
    unsafe fn fixedSlotRange(&mut self, g: &mut Generator) -> SlotRange {
        match self.kind.as_mut().unwrap_unchecked() {
            LValueKind::Scratch { slots, .. } => SlotRange {
                index: 0,
                count: *slots,
            },
            LValueKind::Variable { var, immutable } => {
                if *immutable {
                    g.getImmutableSlots(var.as_ref())
                } else if Generator::IsUniform(var.as_ref()) {
                    g.getUniformSlots(var.as_ref())
                } else {
                    g.getVariableSlots(var.as_ref())
                }
            }
            LValueKind::Swizzle { parent, .. } => parent.fixedSlotRange(g),
            LValueKind::Slice {
                parent,
                initial,
                slots,
            } => {
                let range = parent.get().fixedSlotRange(g);
                let adjusted = SlotRange {
                    index: range.index + *initial,
                    count: *slots,
                };
                debug_assert!(adjusted.index + adjusted.count <= range.index + range.count);
                adjusted
            }
            LValueKind::Dynamic { parent, expr, .. } => {
                let mut r = parent.fixedSlotRange(g);
                r.count = expr.as_ref().expression_type().slotCount() as i32;
                r
            }
        }
    }
    unsafe fn dynamicSlotRange(&mut self) -> Option<NonNull<AutoStack>> {
        match self.kind.as_mut().unwrap_unchecked() {
            LValueKind::Scratch { .. } | LValueKind::Variable { .. } => None,
            LValueKind::Swizzle { parent, .. } => parent.dynamicSlotRange(),
            LValueKind::Slice { parent, .. } => parent.get().dynamicSlotRange(),
            LValueKind::Dynamic { stack, .. } => {
                Some(NonNull::from(stack.as_mut().unwrap_unchecked()))
            }
        }
    }
    unsafe fn swizzle(&self) -> &[i8] {
        match self.kind.as_ref().unwrap_unchecked() {
            LValueKind::Swizzle { components, .. } => components,
            _ => &[],
        }
    }
    unsafe fn push(
        &mut self,
        g: &mut Generator,
        r: SlotRange,
        dynamic: Option<NonNull<AutoStack>>,
        swizzle: &[i8],
    ) -> bool {
        match self.kind.as_mut().unwrap_unchecked() {
            LValueKind::Scratch {
                expr,
                slots,
                g: owner,
                stack,
            } => {
                if stack.is_none() {
                    *owner = Some(NonNull::from(&mut *g));
                    *stack = Some(AutoStack::new(owner.unwrap_unchecked()));
                    let s = stack.as_mut().unwrap_unchecked();
                    s.enter();
                    if !g.pushExpression(expr.as_ref(), true) {
                        return unsupported();
                    }
                    s.exit();
                }
                let s = stack.as_ref().unwrap_unchecked();
                if let Some(d) = dynamic {
                    s.pushCloneIndirect(r, d.as_ref().fStackID, *slots);
                } else {
                    s.pushCloneRange(r, *slots);
                }
                if !swizzle.is_empty() {
                    g.fBuilder.swizzle(r.count, swizzle);
                }
                true
            }
            LValueKind::Variable { var, immutable } => {
                let full = if *immutable {
                    g.getImmutableSlots(var.as_ref())
                } else if Generator::IsUniform(var.as_ref()) {
                    g.getUniformSlots(var.as_ref())
                } else {
                    g.getVariableSlots(var.as_ref())
                };
                if *immutable {
                    if let Some(d) = dynamic {
                        g.fBuilder
                            .push_immutable_indirect(r, d.as_ref().fStackID, full);
                    } else {
                        g.fBuilder.push_immutable(r);
                    }
                } else if Generator::IsUniform(var.as_ref()) {
                    if let Some(d) = dynamic {
                        g.fBuilder
                            .push_uniform_indirect(r, d.as_ref().fStackID, full);
                    } else {
                        g.fBuilder.push_uniform(r);
                    }
                } else if let Some(d) = dynamic {
                    g.fBuilder.push_slots_indirect(r, d.as_ref().fStackID, full);
                } else {
                    g.fBuilder.push_slots(r);
                }
                if !swizzle.is_empty() {
                    g.fBuilder.swizzle(r.count, swizzle);
                }
                true
            }
            LValueKind::Swizzle { parent, components } => {
                if !swizzle.is_empty() {
                    debug_assert!(false, "unfolded swizzle-of-swizzle");
                    return unsupported();
                }
                parent.push(g, r, dynamic, components)
            }
            LValueKind::Slice { parent, .. } => parent.get().push(g, r, dynamic, swizzle),
            LValueKind::Dynamic { parent, .. } => parent.push(g, r, dynamic, swizzle),
        }
    }
    unsafe fn store(
        &mut self,
        g: &mut Generator,
        r: SlotRange,
        dynamic: Option<NonNull<AutoStack>>,
        swizzle: &[i8],
    ) -> bool {
        match self.kind.as_mut().unwrap_unchecked() {
            LValueKind::Scratch { .. } => {
                crate::SkDEBUGFAIL!(@source b"src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp\0".as_ptr(),768; b"scratch lvalues cannot be stored into\0".as_ptr());
                unsupported()
            }
            LValueKind::Variable { var, immutable } => {
                if *immutable {
                    crate::SkDEBUGFAIL!(@source b"src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp\0".as_ptr(),898; b"immutable values cannot be stored into\0".as_ptr());
                    return unsupported();
                }
                debug_assert!(!Generator::IsUniform(var.as_ref()));
                let full = g.getVariableSlots(var.as_ref());
                if swizzle.is_empty() {
                    if let Some(d) = dynamic {
                        g.fBuilder
                            .copy_stack_to_slots_indirect(r, d.as_ref().fStackID, full);
                    } else {
                        g.fBuilder.copy_stack_to_slots(r);
                    }
                } else if let Some(d) = dynamic {
                    g.fBuilder.swizzle_copy_stack_to_slots_indirect(
                        r,
                        d.as_ref().fStackID,
                        full,
                        swizzle,
                        swizzle.len() as i32,
                    );
                } else {
                    g.fBuilder
                        .swizzle_copy_stack_to_slots(r, swizzle, swizzle.len() as i32);
                }
                if g.shouldWriteTraceOps() {
                    if let Some(d) = dynamic {
                        g.fBuilder.trace_var_indirect(
                            g.traceMaskStackID(),
                            r,
                            d.as_ref().fStackID,
                            full,
                        );
                    } else {
                        g.fBuilder.trace_var(g.traceMaskStackID(), r);
                    }
                }
                true
            }
            LValueKind::Swizzle { parent, components } => {
                if !swizzle.is_empty() {
                    debug_assert!(false, "unfolded swizzle-of-swizzle");
                    return unsupported();
                }
                parent.store(g, r, dynamic, components)
            }
            LValueKind::Slice { parent, .. } => parent.get().store(g, r, dynamic, swizzle),
            LValueKind::Dynamic { parent, .. } => parent.store(g, r, dynamic, swizzle),
        }
    }
    unsafe fn evaluateDynamicIndices(&mut self, g: &mut Generator) -> bool {
        let LValueKind::Dynamic {
            g: owner,
            parent,
            stack,
            expr,
        } = self.kind.as_mut().unwrap_unchecked()
        else {
            core::hint::unreachable_unchecked()
        };
        debug_assert!(stack.is_none() && owner.is_none());
        *owner = Some(NonNull::from(&mut *g));
        *stack = Some(AutoStack::new(owner.unwrap_unchecked()));
        if !parent.swizzle().is_empty() {
            debug_assert!(false, "indexed swizzle requires native rewrite");
            return unsupported();
        }
        let s = stack.as_mut().unwrap_unchecked();
        s.enter();
        let e = expr.as_ref();
        if !g.pushExpression(&**e.index().as_ref().unwrap_unchecked(), true) {
            return unsupported();
        }
        let slots = e.expression_type().slotCount() as i32;
        if slots != 1 {
            g.fBuilder.push_constant_i(slots, 1);
            g.fBuilder.binary_op(BuilderOp::mul_n_ints, 1);
        }
        if let Some(parentStack) = parent.dynamicSlotRange() {
            parentStack.as_ref().pushClone(1);
            g.fBuilder.binary_op(BuilderOp::add_n_ints, 1);
        }
        s.exit();
        true
    }
}
impl Drop for LValue {
    fn drop(&mut self) {
        unsafe {
            match self.kind.take() {
                Some(LValueKind::Scratch {
                    g: Some(mut g),
                    mut stack,
                    slots,
                    ..
                }) => {
                    if let Some(s) = stack.as_mut() {
                        s.enter();
                        g.as_mut().discardExpression(slots);
                        s.exit();
                    }
                    drop(stack);
                }
                Some(LValueKind::Dynamic {
                    g,
                    mut stack,
                    parent,
                    ..
                }) => {
                    if let Some(s) = stack.as_mut() {
                        let mut g = g.unwrap_unchecked();
                        s.enter();
                        g.as_mut().discardExpression(1);
                        s.exit();
                    }
                    drop(stack);
                    drop(parent);
                }
                Some(other) => drop(other),
                None => {}
            }
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:1109-1231
impl SlotManager {
    unsafe fn addSlotDebugInfoForGroup(
        &mut self,
        name: &[u8],
        ty: &dyn Type,
        pos: Position,
        group: &mut i32,
        returnValue: bool,
    ) {
        match ty.typeKind() {
            TypeKind::kArray => {
                for i in 0..ty.columns() {
                    let mut n = name.to_vec();
                    n.extend_from_slice(format!("[{}]", i).as_bytes());
                    self.addSlotDebugInfoForGroup(&n, ty.componentType(), pos, group, returnValue);
                }
            }
            TypeKind::kStruct => {
                for f in ty.fields() {
                    let mut n = name.to_vec();
                    n.push(b'.');
                    n.extend_from_slice(f.fName.as_ref());
                    self.addSlotDebugInfoForGroup(
                        &n,
                        f.fType.unwrap_unchecked().as_ref(),
                        pos,
                        group,
                        returnValue,
                    );
                }
            }
            _ => {
                debug_assert!(matches!(
                    ty.typeKind(),
                    TypeKind::kScalar | TypeKind::kVector | TypeKind::kMatrix
                ));
                for i in 0..ty.slotCount() {
                    let info = SlotDebugInfo {
                        name: name.to_vec(),
                        columns: ty.columns() as u8,
                        rows: ty.rows() as u8,
                        componentIndex: i as u8,
                        groupIndex: *group,
                        numberKind: ty.componentType().numberKind(),
                        pos,
                        fnReturnValue: if returnValue { 1 } else { -1 },
                        ..SlotDebugInfo::default()
                    };
                    *group += 1;
                    self.fSlotDebugInfo
                        .as_mut()
                        .unwrap_unchecked()
                        .as_mut()
                        .push(info);
                }
            }
        }
    }
    unsafe fn addSlotDebugInfo(
        &mut self,
        name: &[u8],
        ty: &dyn Type,
        pos: Position,
        returnValue: bool,
    ) {
        let mut group = 0;
        self.addSlotDebugInfoForGroup(name, ty, pos, &mut group, returnValue);
        debug_assert!(group as usize == ty.slotCount());
    }
    unsafe fn createSlots(
        &mut self,
        name: Vec<u8>,
        ty: &dyn Type,
        pos: Position,
        returnValue: bool,
    ) -> SlotRange {
        let n = ty.slotCount() as i32;
        if n == 0 {
            return SlotRange { index: 0, count: 0 };
        }
        if let Some(mut info) = self.fSlotDebugInfo {
            debug_assert!(info.as_ref().len() == self.fSlotCount as usize);
            info.as_mut().reserve(n as usize);
            self.addSlotDebugInfo(&name, ty, pos, returnValue);
            debug_assert!(info.as_ref().len() == (self.fSlotCount + n) as usize);
        }
        let r = SlotRange {
            index: self.fSlotCount,
            count: n,
        };
        self.fSlotCount += n;
        r
    }
    fn mapVariableToSlots(&mut self, var: &dyn Variable, r: SlotRange) -> Option<SlotRange> {
        debug_assert!(var.symbol_type().slotCount() == r.count as usize);
        self.fSlotMap.insert(var_key(var), r)
    }
    fn unmapVariableSlots(&mut self, var: &dyn Variable) {
        self.fSlotMap.remove(&var_key(var));
    }
    unsafe fn getVariableSlots(&mut self, var: &dyn Variable) -> SlotRange {
        if let Some(r) = self.fSlotMap.get(&var_key(var)) {
            return *r;
        }
        let r = self.createSlots(
            var.name().to_vec(),
            var.symbol_type(),
            var.node().fPosition,
            false,
        );
        self.mapVariableToSlots(var, r);
        r
    }
    unsafe fn getFunctionSlots(&mut self, site: &dyn IRNode, f: &FunctionDeclaration) -> SlotRange {
        let key = node_key(site);
        if let Some(r) = self.fSlotMap.get(&key) {
            return *r;
        }
        let mut name = b"[".to_vec();
        name.extend_from_slice(f.name());
        name.extend_from_slice(b"].result");
        let r = self.createSlots(name, f.returnType(), f.node().fPosition, true);
        self.fSlotMap.insert(key, r);
        r
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:1233-1243
fn is_sliceable_swizzle(c: &[i8]) -> bool {
    for i in 1..c.len() {
        if c[i] != c[0] + i as i8 {
            return false;
        }
    }
    true
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:1245-1331
impl Generator {
    unsafe fn makeLValue(&mut self, e: &dyn Expression, allowScratch: bool) -> Option<Box<LValue>> {
        if e.node().is::<VariableReference>() {
            let var = e
                .node()
                .as_node::<VariableReference>()
                .variable()
                .unwrap_unchecked();
            return Some(LValue::new(LValueKind::Variable {
                var,
                immutable: self.fImmutableVariables.contains(&var_key(var.as_ref())),
            }));
        }
        if e.node().is::<Swizzle>() {
            let s = e.node().as_node::<Swizzle>();
            if let Some(base) =
                self.makeLValue(&**s.base().as_ref().unwrap_unchecked(), allowScratch)
            {
                let c = s.components().as_slice();
                if is_sliceable_swizzle(c) {
                    return Some(LValue::new(LValueKind::Slice {
                        parent: LValueParent::Owned(base),
                        initial: c[0] as i32,
                        slots: c.len() as i32,
                    }));
                }
                return Some(LValue::new(LValueKind::Swizzle {
                    parent: base,
                    components: c.to_vec(),
                }));
            }
            return None;
        }
        if e.node().is::<FieldAccess>() {
            let f = e.node().as_node::<FieldAccess>();
            if let Some(base) =
                self.makeLValue(&**f.base().as_ref().unwrap_unchecked(), allowScratch)
            {
                return Some(LValue::new(LValueKind::Slice {
                    parent: LValueParent::Owned(base),
                    initial: f.initialSlot() as i32,
                    slots: f.expression_type().slotCount() as i32,
                }));
            }
            return None;
        }
        if e.node().is::<IndexExpression>() {
            let i = e.node().as_node::<IndexExpression>();
            if let Some(rewritten) =
                crate::src::sksl::transform::SkSLRewriteIndexedSwizzle::RewriteIndexedSwizzle(
                    &self.fContext,
                    i,
                )
            {
                let mut lvalue = self.makeLValue(&*rewritten, allowScratch)?;
                lvalue.fScratchExpression = Some(rewritten);
                return Some(lvalue);
            }
            if let Some(base) =
                self.makeLValue(&**i.base().as_ref().unwrap_unchecked(), allowScratch)
            {
                let mut index = 0;
                if ConstantFolder::GetConstantInt(
                    &**i.index().as_ref().unwrap_unchecked(),
                    &mut index,
                ) {
                    let n = i.expression_type().slotCount() as i32;
                    return Some(LValue::new(LValueKind::Slice {
                        parent: LValueParent::Owned(base),
                        initial: (n as i64 * index) as i32,
                        slots: n,
                    }));
                }
                let mut value = LValue::new(LValueKind::Dynamic {
                    g: None,
                    parent: base,
                    stack: None,
                    expr: NonNull::from(i),
                });
                return if value.evaluateDynamicIndices(self) {
                    Some(value)
                } else {
                    None
                };
            }
            return None;
        }
        if allowScratch {
            let expr = core::mem::transmute::<NonNull<dyn Expression + '_>, NonNull<dyn Expression>>(
                NonNull::from(e),
            );
            return Some(LValue::new(LValueKind::Scratch {
                expr,
                slots: e.expression_type().slotCount() as i32,
                g: None,
                stack: None,
            }));
        }
        None
    }
    unsafe fn push(&mut self, value: &mut LValue) -> bool {
        let r = value.fixedSlotRange(self);
        let d = value.dynamicSlotRange();
        value.push(self, r, d, &[])
    }
    unsafe fn store(&mut self, value: &mut LValue) -> bool {
        debug_assert!(value.isWritable());
        let r = value.fixedSlotRange(self);
        let d = value.dynamicSlotRange();
        value.store(self, r, d, &[])
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:1333-1603
impl Generator {
    unsafe fn getFunctionDebugInfo(&mut self, f: &FunctionDeclaration) -> i32 {
        let mut name = f.description();
        if name.starts_with(b"noinline ") {
            name.drain(..9);
        }
        let trace = self.fDebugTrace.as_mut().unwrap_unchecked().as_mut();
        if let Some(i) = trace.fFuncInfo.iter().position(|i| i.name == name) {
            return i as i32;
        }
        let i = trace.fFuncInfo.len() as i32;
        trace.fFuncInfo.push(FunctionDebugInfo { name });
        i
    }
    fn createStack(&mut self) -> i32 {
        if let Some(id) = self.fRecycledStacks.pop() {
            return id;
        }
        self.fNextStackID += 1;
        self.fNextStackID
    }
    fn recycleStack(&mut self, id: i32) {
        self.fRecycledStacks.push(id);
    }
    fn setCurrentStack(&mut self, id: i32) {
        if self.fCurrentStack != id {
            self.fCurrentStack = id;
            self.fBuilder.set_current_stack(id);
        }
    }
    unsafe fn writeFunction(
        &mut self,
        site: &dyn IRNode,
        f: &FunctionDefinition,
        args: &[ES],
    ) -> Option<SlotRange> {
        let mut funcIndex = -1;
        if self.fDebugTrace.is_some() {
            funcIndex = self.getFunctionDebugInfo(f.declaration());
            crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp\0".as_ptr(),1386; @condition b"funcIndex >= 0\0".as_ptr(); funcIndex>=0);
            if self.shouldWriteTraceOps() {
                self.fBuilder
                    .trace_enter(self.traceMaskStackID(), funcIndex);
            }
        }
        let params = f.declaration().parameters();
        let mut lvalues: Vec<Option<Box<LValue>>> = Vec::new();
        let mut remaps = Vec::new();
        if f.declaration().isMain() {
            if self.shouldWriteTraceOps() {
                for var in params {
                    let r = self.getVariableSlots(var.unwrap_unchecked().as_ref());
                    self.fBuilder.trace_var(self.traceMaskStackID(), r);
                }
            }
        } else {
            lvalues.resize_with(args.len(), || None);
            for (i, arg) in args.iter().enumerate() {
                let arg = &**arg.as_ref().unwrap_unchecked();
                let param = params[i].unwrap_unchecked().as_ref();
                if arg.expression_type().isEffectChild() {
                    let var = arg
                        .node()
                        .as_node::<VariableReference>()
                        .variable()
                        .unwrap_unchecked();
                    if let Some(index) = self.fChildEffectMap.get(&var_key(var.as_ref())).copied() {
                        debug_assert!(!self.fChildEffectMap.contains_key(&var_key(param)));
                        self.fChildEffectMap.insert(var_key(param), index);
                    }
                    continue;
                }
                if Self::IsInoutParameter(param) || Self::IsOutParameter(param) {
                    let mut lv = self.makeLValue(arg, false)?;
                    if Self::IsInoutParameter(param) {
                        if !self.push(&mut lv) {
                            return None;
                        }
                        let r = self.getVariableSlots(param);
                        self.popToSlotRangeUnmasked(r);
                    }
                    lvalues[i] = Some(lv);
                    continue;
                }
                let counts = self
                    .fProgram
                    .as_ref()
                    .usage()
                    .unwrap_unchecked()
                    .getVariable(param);
                if counts.fRead == 0 {
                    if Analysis::HasSideEffects(arg) {
                        if !self.pushExpression(arg, false) {
                            return None;
                        }
                        self.discardExpression(arg.expression_type().slotCount() as i32);
                    }
                    continue;
                }
                if counts.fWrite == 0 && arg.node().is::<VariableReference>() {
                    let var = arg
                        .node()
                        .as_node::<VariableReference>()
                        .variable()
                        .unwrap_unchecked()
                        .as_ref();
                    if self.hasVariableSlots(var) {
                        let r = self.getVariableSlots(var);
                        let old = self.fProgramSlots.mapVariableToSlots(param, r);
                        remaps.push((params[i].unwrap_unchecked(), old));
                        continue;
                    }
                }
                if !self.pushExpression(arg, true) {
                    return None;
                }
                let r = self.getVariableSlots(param);
                self.popToSlotRangeUnmasked(r);
            }
        }
        let last = self.fCurrentFunctionResult;
        self.fCurrentFunctionResult = self.getFunctionSlots(site, f.declaration());
        if self.needsReturnMask(f) {
            self.fBuilder.enableExecutionMaskWrites();
            if !f.declaration().isMain() {
                self.fBuilder.push_return_mask();
            }
        }
        if !self.writeStatement(&**f.body().as_ref().unwrap_unchecked()) {
            return None;
        }
        if self.needsReturnMask(f) {
            if !f.declaration().isMain() {
                self.fBuilder.pop_return_mask();
            }
            self.fBuilder.disableExecutionMaskWrites();
        }
        let result = self.fCurrentFunctionResult;
        self.fCurrentFunctionResult = last;
        if self.fDebugTrace.is_some() && self.fWriteTraceOps {
            self.fBuilder.trace_exit(self.traceMaskStackID(), funcIndex);
        }
        for (i, lv) in lvalues.iter_mut().enumerate() {
            if let Some(lv) = lv {
                let param = params[i].unwrap_unchecked().as_ref();
                crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp\0".as_ptr(),1526; @condition b"IsInoutParameter(param) || IsOutParameter(param)\0".as_ptr(); Self::IsInoutParameter(param)||Self::IsOutParameter(param));
                let r = self.getVariableSlots(param);
                self.fBuilder.push_slots(r);
                if !self.store(lv) {
                    return None;
                }
                self.discardExpression(param.symbol_type().slotCount() as i32);
            }
        }
        for (v, r) in remaps {
            if let Some(r) = r {
                self.fProgramSlots.mapVariableToSlots(v.as_ref(), r);
            } else {
                self.fProgramSlots.unmapVariableSlots(v.as_ref());
            }
        }
        for (i, arg) in args.iter().enumerate() {
            if arg
                .as_ref()
                .unwrap_unchecked()
                .expression_type()
                .isEffectChild()
            {
                self.fChildEffectMap
                    .remove(&var_key(params[i].unwrap_unchecked().as_ref()));
            }
        }
        Some(result)
    }
    unsafe fn emitTraceLine(&mut self, pos: Position) {
        if self.shouldWriteTraceOps() && pos.valid() && self.fInsideCompoundStatement == 0 {
            debug_assert!(self.fLineOffsets.len() >= 2 && self.fLineOffsets[0] == 0);
            debug_assert!(
                *self.fLineOffsets.last().unwrap_unchecked()
                    == self
                        .fProgram
                        .as_ref()
                        .fSource
                        .as_ref()
                        .unwrap_unchecked()
                        .len() as i32
            );
            let line = self
                .fLineOffsets
                .partition_point(|x| *x <= pos.startOffset()) as i32;
            self.fBuilder.trace_line(self.traceMaskStackID(), line);
        }
    }
    unsafe fn pushTraceScopeMask(&mut self) {
        if self.shouldWriteTraceOps() {
            self.fBuilder.push_constant_i(0, 1);
            self.fTraceMask.as_ref().unwrap_unchecked().pushClone(1);
            self.fBuilder.select(1);
        }
    }
    fn discardTraceScopeMask(&mut self) {
        if self.shouldWriteTraceOps() {
            self.discardExpression(1);
        }
    }
    fn emitTraceScope(&mut self, delta: i32) {
        if self.shouldWriteTraceOps() {
            self.fBuilder.trace_scope(self.fCurrentStack, delta);
        }
    }
    unsafe fn calculateLineOffsets(&mut self) {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp\0".as_ptr(),1595; @condition b"fLineOffsets.empty()\0".as_ptr(); self.fLineOffsets.is_empty());
        self.fLineOffsets.push(0);
        let source = self.fProgram.as_ref().fSource.as_ref().unwrap_unchecked();
        for (i, b) in source.iter().enumerate() {
            if *b == b'\n' {
                self.fLineOffsets.push(i as i32);
            }
        }
        self.fLineOffsets.push(source.len() as i32);
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:1605-1754
impl Generator {
    unsafe fn writeGlobals(&mut self) -> bool {
        for element in self.fProgram.as_ref().elements() {
            let e = element.unwrap_unchecked();
            if !e.node().is::<GlobalVarDeclaration>() {
                continue;
            }
            let global = e.node().as_node::<GlobalVarDeclaration>();
            let decl = global.declaration().node().as_node::<VarDeclaration>();
            let var = decl.var().unwrap_unchecked().as_ref();
            if var.symbol_type().isEffectChild() {
                debug_assert!(!self.fChildEffectMap.contains_key(&var_key(var)));
                let index = self.fChildEffectMap.len() as i32;
                self.fChildEffectMap.insert(var_key(var), index);
                continue;
            }
            debug_assert!(!var.symbol_type().isVoid() && !var.symbol_type().isOpaque());
            let builtin = var.layout().fBuiltin;
            if builtin >= 0 {
                if builtin == crate::src::sksl::SkSLCompiler::SK_FRAGCOORD_BUILTIN {
                    let r = self.getVariableSlots(var);
                    self.fBuilder.store_device_xy01(r);
                    continue;
                }
                return unsupported();
            }
            if Self::IsUniform(var) {
                let r = self.getUniformSlots(var);
                if self.shouldWriteTraceOps() {
                    let copy = self.fProgramSlots.getVariableSlots(var);
                    self.fBuilder.push_uniform(r);
                    self.popToSlotRangeUnmasked(copy);
                }
                continue;
            }
            if !self.writeVarDeclaration(decl) {
                return unsupported();
            }
        }
        true
    }
    unsafe fn writeStatement(&mut self, s: &dyn Statement) -> bool {
        if !matches!(s.kind(), SK::kBlock | SK::kFor) {
            self.emitTraceLine(s.node().fPosition);
        }
        match s.kind() {
            SK::kBlock => self.writeBlock(s.node().as_node::<Block>()),
            SK::kBreak => self.writeBreakStatement(s.node().as_node::<BreakStatement>()),
            SK::kContinue => self.writeContinueStatement(s.node().as_node::<ContinueStatement>()),
            SK::kDo => self.writeDoStatement(s.node().as_node::<DoStatement>()),
            SK::kExpression => {
                self.writeExpressionStatement(s.node().as_node::<ExpressionStatement>())
            }
            SK::kFor => self.writeForStatement(s.node().as_node::<ForStatement>()),
            SK::kIf => self.writeIfStatement(s.node().as_node::<IfStatement>()),
            SK::kNop => true,
            SK::kReturn => self.writeReturnStatement(s.node().as_node::<ReturnStatement>()),
            SK::kSwitch => self.writeSwitchStatement(s.node().as_node::<SwitchStatement>()),
            SK::kVarDeclaration => self.writeVarDeclaration(s.node().as_node::<VarDeclaration>()),
            _ => unsupported(),
        }
    }
    unsafe fn writeBlock(&mut self, b: &Block) -> bool {
        let compound = b.blockKind() == crate::src::sksl::ir::SkSLBlock::Kind::kCompoundStatement;
        if compound {
            self.emitTraceLine(b.node().fPosition);
            self.fInsideCompoundStatement += 1;
        } else {
            self.pushTraceScopeMask();
            self.emitTraceScope(1);
        }
        for s in b.children().as_slice() {
            if !self.writeStatement(&**s.as_ref().unwrap_unchecked()) {
                return unsupported();
            }
        }
        if compound {
            self.fInsideCompoundStatement -= 1;
        } else {
            self.emitTraceScope(-1);
            self.discardTraceScopeMask();
        }
        true
    }
    fn writeBreakStatement(&mut self, _: &BreakStatement) -> bool {
        self.fBuilder
            .branch_if_all_lanes_active(self.fCurrentBreakTarget);
        self.fBuilder.mask_off_loop_mask();
        true
    }
    unsafe fn writeContinueStatement(&mut self, _: &ContinueStatement) -> bool {
        self.fBuilder.continue_op(
            self.fCurrentContinueMask
                .unwrap_unchecked()
                .as_ref()
                .stackID(),
        );
        true
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:1756-1979
impl Generator {
    unsafe fn writeDoStatement(&mut self, d: &DoStatement) -> bool {
        let ptr = NonNull::from(&mut *self);
        let target = AutoLoopTarget::new(ptr, NonNull::from(&mut self.fCurrentBreakTarget));
        self.fBuilder.enableExecutionMaskWrites();
        self.fBuilder.push_loop_mask();
        let info = Analysis::GetLoopControlFlowInfo(&**d.statement().as_ref().unwrap_unchecked());
        let mut mask = AutoContinueMask::new(ptr);
        if info.fHasContinue {
            mask.as_mut().enable();
        }
        let label = self.fBuilder.nextLabelID();
        self.fBuilder.label(label);
        mask.as_mut().get_unchecked_mut().enterLoopBody();
        if !self.writeStatement(&**d.statement().as_ref().unwrap_unchecked()) {
            return false;
        }
        mask.as_mut().get_unchecked_mut().exitLoopBody();
        let test = &**d.test().as_ref().unwrap_unchecked();
        self.emitTraceLine(test.node().fPosition);
        if !self.pushExpression(test, true) {
            return false;
        }
        self.fBuilder.merge_loop_mask();
        self.discardExpression(1);
        self.fBuilder.branch_if_any_lanes_active(label);
        self.fBuilder.label(target.fLabelID);
        self.fBuilder.pop_loop_mask();
        self.fBuilder.disableExecutionMaskWrites();
        true
    }
    unsafe fn writeMasklessForStatement(&mut self, f: &ForStatement) -> bool {
        let info = f.unrollInfo().unwrap_unchecked();
        debug_assert!(
            info.fCount > 0
                && f.initializer().is_some()
                && f.test().is_some()
                && f.next().is_some()
        );
        self.pushTraceScopeMask();
        self.emitTraceScope(1);
        let exit = self.fBuilder.nextLabelID();
        let body = self.fBuilder.nextLabelID();
        self.fBuilder.branch_if_no_lanes_active(exit);
        if !self.writeStatement(&**f.initializer().as_ref().unwrap_unchecked()) {
            return unsupported();
        }
        self.fBuilder.label(body);
        if !self.writeStatement(&**f.statement().as_ref().unwrap_unchecked()) {
            return unsupported();
        }
        let pos = f
            .next()
            .as_ref()
            .map(|e| e.node().fPosition)
            .or_else(|| f.test().as_ref().map(|e| e.node().fPosition))
            .unwrap_or(f.node().fPosition);
        self.emitTraceLine(pos);
        if info.fCount > 1 {
            let next = &**f.next().as_ref().unwrap_unchecked();
            if !self.pushExpression(next, false) {
                return unsupported();
            }
            self.discardExpression(next.expression_type().slotCount() as i32);
            if !self.pushExpression(&**f.test().as_ref().unwrap_unchecked(), true) {
                return unsupported();
            }
            self.fBuilder
                .branch_if_no_active_lanes_on_stack_top_equal(0, body);
            self.discardExpression(1);
        }
        self.fBuilder.label(exit);
        self.emitTraceScope(-1);
        self.discardTraceScopeMask();
        true
    }
    unsafe fn writeForStatement(&mut self, f: &ForStatement) -> bool {
        if f.unrollInfo().is_some_and(|i| i.fCount == 0) {
            return true;
        }
        let info = Analysis::GetLoopControlFlowInfo(&**f.statement().as_ref().unwrap_unchecked());
        if !info.fHasContinue && !info.fHasBreak && !info.fHasReturn && f.unrollInfo().is_some() {
            return self.writeMasklessForStatement(f);
        }
        self.pushTraceScopeMask();
        self.emitTraceScope(1);
        let ptr = NonNull::from(&mut *self);
        let target = AutoLoopTarget::new(ptr, NonNull::from(&mut self.fCurrentBreakTarget));
        if let Some(init) = f.initializer() {
            if !self.writeStatement(&**init) {
                return unsupported();
            }
        } else {
            self.emitTraceLine(f.node().fPosition);
        }
        let mut mask = AutoContinueMask::new(ptr);
        if info.fHasContinue {
            mask.as_mut().enable();
        }
        self.fBuilder.enableExecutionMaskWrites();
        self.fBuilder.push_loop_mask();
        let testlabel = self.fBuilder.nextLabelID();
        let bodylabel = self.fBuilder.nextLabelID();
        self.fBuilder.jump(testlabel);
        self.fBuilder.label(bodylabel);
        mask.as_mut().get_unchecked_mut().enterLoopBody();
        if !self.writeStatement(&**f.statement().as_ref().unwrap_unchecked()) {
            return unsupported();
        }
        mask.as_mut().get_unchecked_mut().exitLoopBody();
        let pos = f
            .next()
            .as_ref()
            .map(|e| e.node().fPosition)
            .or_else(|| f.test().as_ref().map(|e| e.node().fPosition))
            .unwrap_or(f.node().fPosition);
        self.emitTraceLine(pos);
        if let Some(next) = f.next() {
            if !self.pushExpression(&**next, false) {
                return unsupported();
            }
            self.discardExpression(next.expression_type().slotCount() as i32);
        }
        self.fBuilder.label(testlabel);
        if let Some(test) = f.test() {
            if !self.pushExpression(&**test, true) {
                return unsupported();
            }
            self.fBuilder.merge_loop_mask();
            self.discardExpression(1);
        }
        self.fBuilder.branch_if_any_lanes_active(bodylabel);
        self.fBuilder.label(target.fLabelID);
        self.fBuilder.pop_loop_mask();
        self.fBuilder.disableExecutionMaskWrites();
        self.emitTraceScope(-1);
        self.discardTraceScopeMask();
        true
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:1981-2201
impl Generator {
    unsafe fn writeExpressionStatement(&mut self, e: &ExpressionStatement) -> bool {
        let e = &**e.expression().as_ref().unwrap_unchecked();
        if !self.pushExpression(e, false) {
            return unsupported();
        }
        self.discardExpression(e.expression_type().slotCount() as i32);
        true
    }
    unsafe fn writeDynamicallyUniformIfStatement(&mut self, i: &IfStatement) -> bool {
        let test = &**i.test().as_ref().unwrap_unchecked();
        debug_assert!(Analysis::IsDynamicallyUniformExpression(test));
        let no = self.fBuilder.nextLabelID();
        let exit = self.fBuilder.nextLabelID();
        if !self.pushExpression(test, true) {
            return unsupported();
        }
        self.fBuilder
            .branch_if_no_active_lanes_on_stack_top_equal(!0, no);
        if !self.writeStatement(&**i.ifTrue().as_ref().unwrap_unchecked()) {
            return unsupported();
        }
        if let Some(s) = i.ifFalse() {
            self.fBuilder.jump(exit);
            self.fBuilder.label(no);
            if !self.writeStatement(&**s) {
                return unsupported();
            }
            self.fBuilder.label(exit);
        } else {
            self.fBuilder.label(no);
        }
        self.discardExpression(1);
        true
    }
    unsafe fn writeIfStatement(&mut self, i: &IfStatement) -> bool {
        let test = &**i.test().as_ref().unwrap_unchecked();
        if Analysis::IsDynamicallyUniformExpression(test) {
            return self.writeDynamicallyUniformIfStatement(i);
        }
        self.fBuilder.enableExecutionMaskWrites();
        self.fBuilder.push_condition_mask();
        if !self.pushExpression(test, true) {
            return unsupported();
        }
        self.fBuilder.merge_condition_mask();
        if !self.writeStatement(&**i.ifTrue().as_ref().unwrap_unchecked()) {
            return unsupported();
        }
        if let Some(s) = i.ifFalse() {
            self.fBuilder.merge_inv_condition_mask();
            if !self.writeStatement(&**s) {
                return unsupported();
            }
        }
        self.discardExpression(1);
        self.fBuilder.pop_condition_mask();
        self.fBuilder.disableExecutionMaskWrites();
        true
    }
    unsafe fn writeReturnStatement(&mut self, r: &ReturnStatement) -> bool {
        let f = self.fCurrentFunction.unwrap_unchecked();
        if let Some(e) = r.expression() {
            if !self.pushExpression(&**e, true) {
                return unsupported();
            }
            if self.needsFunctionResultSlots(f.as_ref()) {
                self.popToSlotRange(self.fCurrentFunctionResult);
            }
        }
        if self.fBuilder.executionMaskWritesAreEnabled() && self.needsReturnMask(f.as_ref()) {
            self.fBuilder.mask_off_return_mask();
        }
        true
    }
    unsafe fn writeSwitchStatement(&mut self, s: &SwitchStatement) -> bool {
        use crate::src::sksl::ir::SkSLSwitchCase::SwitchCase;
        let cases = s.cases().as_slice();
        debug_assert!(cases.iter().all(|s| s
            .as_ref()
            .unwrap_unchecked()
            .node()
            .is::<SwitchCase>()));
        let ptr = NonNull::from(&mut *self);
        let target = AutoLoopTarget::new(ptr, NonNull::from(&mut self.fCurrentBreakTarget));
        self.fBuilder.enableExecutionMaskWrites();
        self.fBuilder.push_loop_mask();
        if !self.pushExpression(&**s.value().as_ref().unwrap_unchecked(), true) {
            return unsupported();
        }
        self.fBuilder.push_loop_mask();
        self.fBuilder.mask_off_loop_mask();
        let mut found = false;
        for (index, stmt) in cases.iter().enumerate() {
            let skip = self.fBuilder.nextLabelID();
            let sc = stmt
                .as_ref()
                .unwrap_unchecked()
                .node()
                .as_node::<SwitchCase>();
            if sc.isDefault() {
                found = true;
                if index + 1 != cases.len() {
                    return unsupported();
                }
                self.fBuilder.pop_and_reenable_loop_mask();
            } else {
                self.fBuilder.case_op(sc.value() as i32);
            }
            self.fBuilder.branch_if_no_lanes_active(skip);
            if !self.writeStatement(&**sc.statement().as_ref().unwrap_unchecked()) {
                return unsupported();
            }
            self.fBuilder.label(skip);
        }
        self.discardExpression(if found { 1 } else { 2 });
        self.fBuilder.label(target.fLabelID);
        self.fBuilder.pop_loop_mask();
        self.fBuilder.disableExecutionMaskWrites();
        true
    }
    unsafe fn writeImmutableVarDeclaration(&mut self, d: &VarDeclaration) -> bool {
        if self.shouldWriteTraceOps() {
            return false;
        }
        let initial =
            ConstantFolder::GetConstantValueForVariable(&**d.value().as_ref().unwrap_unchecked());
        let var = d.var().unwrap_unchecked().as_ref();
        if self
            .fProgram
            .as_ref()
            .usage()
            .unwrap_unchecked()
            .getVariable(var)
            .fWrite
            != 1
        {
            return false;
        }
        let mut values = Vec::new();
        if !self.getImmutableValueForExpression(initial, &mut values) {
            return false;
        }
        self.fImmutableVariables.insert(var_key(var));
        if let Some(r) = self.findPreexistingImmutableData(&values) {
            self.fImmutableSlots.mapVariableToSlots(var, r);
        } else {
            let r = self.getImmutableSlots(var);
            self.storeImmutableValueToSlots(&values, r);
        }
        true
    }
    unsafe fn writeVarDeclaration(&mut self, d: &VarDeclaration) -> bool {
        if let Some(value) = d.value() {
            if self.writeImmutableVarDeclaration(d) {
                return true;
            }
            if !self.pushExpression(&**value, true) {
                return unsupported();
            }
            let r = self.getVariableSlots(d.var().unwrap_unchecked().as_ref());
            self.popToSlotRangeUnmasked(r);
        } else {
            let r = self.getVariableSlots(d.var().unwrap_unchecked().as_ref());
            self.zeroSlotRangeUnmasked(r);
        }
        true
    }
}

// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:2203-2348
impl Generator {
    unsafe fn pushExpression(&mut self, e: &dyn Expression, usesResult: bool) -> bool {
        use crate::src::sksl::ir::{
            SkSLConstructorDiagonalMatrix::ConstructorDiagonalMatrix,
            SkSLConstructorMatrixResize::ConstructorMatrixResize,
            SkSLConstructorSplat::ConstructorSplat,
        };
        match e.kind() {
            EK::kBinary => {
                let b = e.node().as_node::<BinaryExpression>();
                self.pushBinaryExpression(
                    &**b.left().as_ref().unwrap_unchecked(),
                    b.getOperator(),
                    &**b.right().as_ref().unwrap_unchecked(),
                )
            }
            EK::kChildCall => self.pushChildCall(e.node().as_node::<ChildCall>()),
            EK::kConstructorArray
            | EK::kConstructorArrayCast
            | EK::kConstructorCompound
            | EK::kConstructorStruct => self.pushConstructorCompound(e.constructor_view()),
            EK::kConstructorCompoundCast | EK::kConstructorScalarCast => {
                self.pushConstructorCast(e.constructor_view())
            }
            EK::kConstructorDiagonalMatrix => {
                self.pushConstructorDiagonalMatrix(e.node().as_node::<ConstructorDiagonalMatrix>())
            }
            EK::kConstructorMatrixResize => {
                self.pushConstructorMatrixResize(e.node().as_node::<ConstructorMatrixResize>())
            }
            EK::kConstructorSplat => {
                self.pushConstructorSplat(e.node().as_node::<ConstructorSplat>())
            }
            EK::kEmpty => true,
            EK::kFieldAccess | EK::kIndex => {
                if let Some(mut value) = self.makeLValue(e, true) {
                    self.push(&mut value)
                } else {
                    false
                }
            }
            EK::kFunctionCall => self.pushFunctionCall(e.node().as_node::<FunctionCall>()),
            EK::kLiteral => self.pushLiteral(e.node().as_node::<Literal>()),
            EK::kPrefix => {
                let p = e.node().as_node::<PrefixExpression>();
                self.pushPrefixExpression(
                    p.getOperator(),
                    &**p.operand().as_ref().unwrap_unchecked(),
                )
            }
            EK::kPostfix => {
                self.pushPostfixExpression(e.node().as_node::<PostfixExpression>(), usesResult)
            }
            EK::kSwizzle => self.pushSwizzle(e.node().as_node::<Swizzle>()),
            EK::kTernary => {
                let t = e.node().as_node::<TernaryExpression>();
                self.pushTernaryExpression(
                    &**t.test().as_ref().unwrap_unchecked(),
                    &**t.ifTrue().as_ref().unwrap_unchecked(),
                    &**t.ifFalse().as_ref().unwrap_unchecked(),
                )
            }
            EK::kVariableReference => {
                self.pushVariableReference(e.node().as_node::<VariableReference>())
            }
            _ => unsupported(),
        }
    }
    fn GetTypedOp(ty: &dyn Type, ops: TypedOps) -> BuilderOp {
        match ty.componentType().numberKind() {
            NumberKind::kFloat => ops.fFloatOp,
            NumberKind::kSigned => ops.fSignedOp,
            NumberKind::kUnsigned => ops.fUnsignedOp,
            NumberKind::kBoolean => ops.fBooleanOp,
            _ => BuilderOp::unsupported,
        }
    }
    fn unaryOp(&mut self, ty: &dyn Type, ops: TypedOps) -> bool {
        let op = Self::GetTypedOp(ty, ops);
        if op == BuilderOp::unsupported {
            return unsupported();
        }
        self.fBuilder.unary_op(op, ty.slotCount() as i32);
        true
    }
    fn binaryOp(&mut self, ty: &dyn Type, ops: TypedOps) -> bool {
        let op = Self::GetTypedOp(ty, ops);
        if op == BuilderOp::unsupported {
            return unsupported();
        }
        self.fBuilder.binary_op(op, ty.slotCount() as i32);
        true
    }
    fn ternaryOp(&mut self, ty: &dyn Type, ops: TypedOps) -> bool {
        let op = Self::GetTypedOp(ty, ops);
        if op == BuilderOp::unsupported {
            return unsupported();
        }
        self.fBuilder.ternary_op(op, ty.slotCount() as i32);
        true
    }
    fn foldWithMultiOp(&mut self, op: BuilderOp, mut n: i32) {
        while n >= 8 {
            self.fBuilder.binary_op(op, 4);
            n -= 4;
        }
        while n >= 6 {
            self.fBuilder.binary_op(op, 3);
            n -= 3;
        }
        while n >= 4 {
            self.fBuilder.binary_op(op, 2);
            n -= 2;
        }
        while n >= 2 {
            self.fBuilder.binary_op(op, 1);
            n -= 1;
        }
    }
    unsafe fn pushLValueOrExpression(
        &mut self,
        lvalue: Option<&mut LValue>,
        expr: &dyn Expression,
    ) -> bool {
        match lvalue {
            Some(l) => self.push(l),
            None => self.pushExpression(expr, true),
        }
    }
    unsafe fn pushMatrixMultiply(
        &mut self,
        mut lv: Option<&mut LValue>,
        left: &dyn Expression,
        right: &dyn Expression,
        lc: i32,
        lr: i32,
        rc: i32,
        rr: i32,
    ) -> bool {
        debug_assert!(
            (left.expression_type().isMatrix() || left.expression_type().isVector())
                && (right.expression_type().isMatrix() || right.expression_type().isVector())
        );
        self.fBuilder.pad_stack(rc * lr);
        if !self.pushLValueOrExpression(lv.as_deref_mut(), left)
            || !self.pushExpression(right, true)
        {
            return unsupported();
        }
        self.fBuilder.matrix_multiply(lc, lr, rc, rr);
        match lv {
            Some(l) => self.store(l),
            None => true,
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:2350-2677
impl Generator {
    fn foldComparisonOp(&mut self, op: Operator, n: i32) {
        match op.kind() {
            Op::EQEQ => self.foldWithMultiOp(BuilderOp::bitwise_and_n_ints, n),
            Op::NEQ => self.foldWithMultiOp(BuilderOp::bitwise_or_n_ints, n),
            _ => debug_assert!(false, "comparison requires == or !="),
        }
    }
    unsafe fn pushStructuredComparison(
        &mut self,
        left: &mut LValue,
        op: Operator,
        right: &mut LValue,
        ty: &dyn Type,
    ) -> bool {
        let fields = if ty.isStruct() {
            Some(ty.fields())
        } else {
            None
        };
        if let Some(fields) = fields {
            let mut slot = 0;
            for field in fields {
                let ft = field.fType.unwrap_unchecked().as_ref();
                let n = ft.slotCount() as i32;
                let mut l = LValue::new(LValueKind::Slice {
                    parent: LValueParent::Unowned(NonNull::from(&mut *left)),
                    initial: slot,
                    slots: n,
                });
                let mut r = LValue::new(LValueKind::Slice {
                    parent: LValueParent::Unowned(NonNull::from(&mut *right)),
                    initial: slot,
                    slots: n,
                });
                if !self.pushStructuredComparison(&mut l, op, &mut r, ft) {
                    return unsupported();
                }
                slot += n;
            }
            self.foldComparisonOp(op, fields.len() as i32);
            return true;
        }
        if ty.isArray() && ty.componentType().numberKind() == NumberKind::kNonnumeric {
            let ft = ty.componentType();
            let n = ft.slotCount() as i32;
            let mut slot = 0;
            for _ in 0..ty.columns() {
                let mut l = LValue::new(LValueKind::Slice {
                    parent: LValueParent::Unowned(NonNull::from(&mut *left)),
                    initial: slot,
                    slots: n,
                });
                let mut r = LValue::new(LValueKind::Slice {
                    parent: LValueParent::Unowned(NonNull::from(&mut *right)),
                    initial: slot,
                    slots: n,
                });
                if !self.pushStructuredComparison(&mut l, op, &mut r, ft) {
                    return unsupported();
                }
                slot += n;
            }
            self.foldComparisonOp(op, ty.columns());
            return true;
        }
        if !self.push(left) || !self.push(right) {
            return unsupported();
        }
        match op.kind() {
            Op::EQEQ => {
                if !self.binaryOp(ty, kEqualOps) {
                    return unsupported();
                }
            }
            Op::NEQ => {
                if !self.binaryOp(ty, kNotEqualOps) {
                    return unsupported();
                }
            }
            _ => debug_assert!(false, "comparison requires == or !="),
        };
        self.foldComparisonOp(op, ty.slotCount() as i32);
        true
    }
    unsafe fn pushBinaryExpression(
        &mut self,
        left: &dyn Expression,
        mut op: Operator,
        right: &dyn Expression,
    ) -> bool {
        match op.kind() {
            Op::GT => return self.pushBinaryExpression(right, Operator::new(Op::LT), left),
            Op::GTEQ => return self.pushBinaryExpression(right, Operator::new(Op::LTEQ), left),
            Op::EQEQ | Op::NEQ => {
                if left.expression_type().isStruct() || left.expression_type().isArray() {
                    crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp\0".as_ptr(),2455; @condition b"left.type().matches(right.type())\0".as_ptr(); left.expression_type().matches(right.expression_type()));
                    let mut l = self.makeLValue(left, true).unwrap_unchecked();
                    let mut r = self.makeLValue(right, true).unwrap_unchecked();
                    return self.pushStructuredComparison(
                        &mut l,
                        op,
                        &mut r,
                        left.expression_type(),
                    );
                }
                let mut v = 0.0;
                if ConstantFolder::GetConstantValue(left, &mut v)
                    && !ConstantFolder::GetConstantValue(right, &mut v)
                {
                    return self.pushBinaryExpression(right, op, left);
                }
            }
            Op::PLUS | Op::STAR | Op::BITWISEAND | Op::BITWISEXOR | Op::LOGICALXOR => {
                let mut v = 0.0;
                if ConstantFolder::GetConstantValue(left, &mut v)
                    && !ConstantFolder::GetConstantValue(right, &mut v)
                {
                    return self.pushBinaryExpression(right, op, left);
                }
            }
            Op::COMMA => {
                if Analysis::HasSideEffects(left) {
                    if !self.pushExpression(left, false) {
                        return unsupported();
                    }
                    self.discardExpression(left.expression_type().slotCount() as i32);
                }
                return self.pushExpression(right, true);
            }
            _ => {}
        }
        let lt = left.expression_type();
        let rt = right.expression_type();
        let mut vl = false;
        let mut vr = false;
        if !lt.matches(rt) {
            if lt.componentType().numberKind() != rt.componentType().numberKind() {
                return unsupported();
            }
            if lt.isScalar() && (rt.isVector() || rt.isMatrix()) {
                vl = true;
            } else if (lt.isVector() || lt.isMatrix()) && rt.isScalar() {
                vr = true;
            }
        }
        let ty = if vl { rt } else { lt };
        let mut lv = None;
        if op.isAssignment() {
            lv = self.makeLValue(left, false);
            if lv.is_none() {
                return unsupported();
            }
            if op.kind() == Op::EQ {
                return self.pushExpression(right, true)
                    && self.store(lv.as_mut().unwrap_unchecked());
            }
            op = op.removeAssignment();
        }
        if op.kind() == Op::STAR {
            if lt.isMatrix() && rt.isMatrix() {
                return self.pushMatrixMultiply(
                    lv.as_deref_mut(),
                    left,
                    right,
                    lt.columns(),
                    lt.rows(),
                    rt.columns(),
                    rt.rows(),
                );
            }
            if lt.isVector() && rt.isMatrix() {
                return self.pushMatrixMultiply(
                    lv.as_deref_mut(),
                    left,
                    right,
                    lt.columns(),
                    1,
                    rt.columns(),
                    rt.rows(),
                );
            }
            if lt.isMatrix() && rt.isVector() {
                return self.pushMatrixMultiply(
                    lv.as_deref_mut(),
                    left,
                    right,
                    lt.columns(),
                    lt.rows(),
                    1,
                    rt.columns(),
                );
            }
        }
        if !vl && !vr && !ty.matches(rt) {
            return unsupported();
        }
        if matches!(op.kind(), Op::LOGICALAND | Op::LOGICALOR) && Analysis::HasSideEffects(right) {
            debug_assert!(ty.componentType().isBoolean() && ty.slotCount() == 1);
            let value = if op.kind() == Op::LOGICALOR { 1.0 } else { 0.0 };
            let lit = Literal::new(Position::default(), value, rt);
            return if op.kind() == Op::LOGICALAND {
                self.pushTernaryExpression(left, right, &lit)
            } else {
                self.pushTernaryExpression(left, &lit, right)
            };
        }
        if !self.pushLValueOrExpression(lv.as_deref_mut(), left) {
            return unsupported();
        }
        if vl {
            self.fBuilder.push_duplicates(rt.slotCount() as i32 - 1);
        }
        if !self.pushExpression(right, true) {
            return unsupported();
        }
        if vr {
            self.fBuilder.push_duplicates(lt.slotCount() as i32 - 1);
        }
        let ok = match op.kind() {
            Op::PLUS => self.binaryOp(ty, kAddOps),
            Op::MINUS => self.binaryOp(ty, kSubtractOps),
            Op::STAR => self.binaryOp(ty, kMultiplyOps),
            Op::SLASH => self.binaryOp(ty, kDivideOps),
            Op::LT | Op::GT => {
                let ok = self.binaryOp(ty, kLessThanOps);
                debug_assert!(ty.slotCount() == 1);
                ok
            }
            Op::LTEQ | Op::GTEQ => {
                let ok = self.binaryOp(ty, kLessThanEqualOps);
                debug_assert!(ty.slotCount() == 1);
                ok
            }
            Op::EQEQ => {
                let ok = self.binaryOp(ty, kEqualOps);
                if ok {
                    self.foldComparisonOp(op, ty.slotCount() as i32);
                }
                ok
            }
            Op::NEQ => {
                let ok = self.binaryOp(ty, kNotEqualOps);
                if ok {
                    self.foldComparisonOp(op, ty.slotCount() as i32);
                }
                ok
            }
            Op::LOGICALAND | Op::BITWISEAND => {
                self.fBuilder
                    .binary_op(BuilderOp::bitwise_and_n_ints, ty.slotCount() as i32);
                true
            }
            Op::LOGICALOR | Op::BITWISEOR => {
                self.fBuilder
                    .binary_op(BuilderOp::bitwise_or_n_ints, ty.slotCount() as i32);
                true
            }
            Op::LOGICALXOR | Op::BITWISEXOR => {
                self.fBuilder
                    .binary_op(BuilderOp::bitwise_xor_n_ints, ty.slotCount() as i32);
                true
            }
            _ => unsupported(),
        };
        if !ok {
            return unsupported();
        }
        if let Some(l) = lv.as_mut() {
            self.store(l)
        } else {
            true
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:2679-2812
impl Generator {
    fn getImmutableBitsForSlot(&self, e: &dyn Expression, slot: usize) -> Option<i32> {
        let v = e.getConstantValue(slot as i32)?;
        match e.expression_type().slotType(slot).numberKind() {
            NumberKind::kFloat => Some((v as f32).to_bits() as i32),
            NumberKind::kSigned => Some(v as i32),
            NumberKind::kUnsigned => Some(v as u32 as i32),
            NumberKind::kBoolean => Some(if v != 0.0 { !0 } else { 0 }),
            _ => None,
        }
    }
    fn getImmutableValueForExpression(&self, e: &dyn Expression, out: &mut Vec<i32>) -> bool {
        if !e.supportsConstantValues() {
            return false;
        }
        let n = e.expression_type().slotCount();
        out.reserve_exact(n);
        for i in 0..n {
            let Some(v) = self.getImmutableBitsForSlot(e, i) else {
                return false;
            };
            out.push(v);
        }
        true
    }
    fn storeImmutableValueToSlots(&mut self, values: &[i32], r: SlotRange) {
        for i in 0..r.count {
            let slot = r.index + i;
            let bits = values[i as usize];
            self.fBuilder.store_immutable_value_i(slot, bits);
            self.fImmutableSlotMap.entry(bits).or_default().add(slot);
        }
    }
    fn findPreexistingImmutableData(&self, values: &[i32]) -> Option<SlotRange> {
        let mut sets = Vec::with_capacity(values.len());
        for v in values {
            sets.push(self.fImmutableSlotMap.get(v)?);
        }
        let mut least = 0;
        let mut count = i32::MAX;
        for (i, set) in sets.iter().enumerate() {
            if set.count() < count {
                least = i;
                count = set.count();
            }
        }
        let mut result = None;
        sets[least].foreach(|slot| {
            if result.is_some() {
                return;
            }
            let first = *slot - least as i32;
            for (i, set) in sets.iter().enumerate() {
                if !set.contains(&(first + i as i32)) {
                    return;
                }
            }
            result = Some(SlotRange {
                index: first,
                count: sets.len() as i32,
            });
        });
        result
    }
    unsafe fn pushImmutableData(&mut self, e: &dyn Expression) -> bool {
        let mut values = Vec::new();
        if !self.getImmutableValueForExpression(e, &mut values) {
            return false;
        }
        if let Some(r) = self.findPreexistingImmutableData(&values) {
            self.fBuilder.push_immutable(r);
            return true;
        }
        let r = self.fImmutableSlots.createSlots(
            e.description(),
            e.expression_type(),
            e.node().fPosition,
            false,
        );
        self.storeImmutableValueToSlots(&values, r);
        self.fBuilder.push_immutable(r);
        true
    }
    unsafe fn pushConstructorCompound(
        &mut self,
        c: &dyn crate::src::sksl::ir::SkSLConstructor::AnyConstructor,
    ) -> bool {
        if c.expression_type().slotCount() > 1 && self.pushImmutableData(c) {
            return true;
        }
        for arg in c.argumentSpan() {
            if !self.pushExpression(&**arg.as_ref().unwrap_unchecked(), true) {
                return unsupported();
            }
        }
        true
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:2814-3102
impl Generator {
    unsafe fn pushChildCall(&mut self, c: &ChildCall) -> bool {
        let child = *self
            .fChildEffectMap
            .get(&var_key(c.child()))
            .unwrap_unchecked();
        let args = c.arguments().as_base().as_slice();
        debug_assert!(!args.is_empty());
        let arg = &**args[0].as_ref().unwrap_unchecked();
        if !self.pushExpression(arg, true) {
            return unsupported();
        }
        match c.child().symbol_type().typeKind() {
            TypeKind::kShader => {
                debug_assert!(
                    args.len() == 1
                        && arg
                            .expression_type()
                            .matches(&**self.fContext.fTypes.as_ref().fFloat2)
                );
                self.fBuilder.pad_stack(2);
                self.fBuilder.exchange_src();
                self.fBuilder.invoke_shader(child);
            }
            TypeKind::kColorFilter => {
                debug_assert!(
                    args.len() == 1
                        && (arg
                            .expression_type()
                            .matches(&**self.fContext.fTypes.as_ref().fHalf4)
                            || arg
                                .expression_type()
                                .matches(&**self.fContext.fTypes.as_ref().fFloat4))
                );
                self.fBuilder.exchange_src();
                self.fBuilder.invoke_color_filter(child);
            }
            TypeKind::kBlender => {
                debug_assert!(
                    args.len() == 2
                        && args.iter().all(|a| a
                            .as_ref()
                            .unwrap_unchecked()
                            .expression_type()
                            .matches(&**self.fContext.fTypes.as_ref().fHalf4)
                            || a.as_ref()
                                .unwrap_unchecked()
                                .expression_type()
                                .matches(&**self.fContext.fTypes.as_ref().fFloat4))
                );
                if !self.pushExpression(&**args[1].as_ref().unwrap_unchecked(), true) {
                    return unsupported();
                }
                self.fBuilder.pop_dst_rgba();
                self.fBuilder.exchange_src();
                self.fBuilder.invoke_blender(child);
            }
            _ => {
                #[cfg(sk_debug)]
                {
                    let mut description = c.child().symbol_type().description();
                    description.push(0);
                    crate::SkDEBUGFAILF!(@source b"src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp\0".as_ptr(),2872; "cannot sample from type '%s'",description.as_ptr());
                }
            }
        };
        self.fBuilder.exchange_src();
        true
    }
    unsafe fn pushConstructorCast(
        &mut self,
        c: &dyn crate::src::sksl::ir::SkSLConstructor::AnyConstructor,
    ) -> bool {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp\0".as_ptr(),2884; @condition b"c.argumentSpan().size() == 1\0".as_ptr(); c.argumentSpan().len()==1);
        let inner = &**c.argumentSpan()[0].as_ref().unwrap_unchecked();
        let ty = c.expression_type();
        debug_assert!(inner.expression_type().slotCount() == ty.slotCount());
        if !self.pushExpression(inner, true) {
            return unsupported();
        }
        let a = inner.expression_type().componentType().numberKind();
        let b = ty.componentType().numberKind();
        let n = ty.slotCount() as i32;
        if a == b {
            return true;
        }
        match a {
            NumberKind::kSigned => {
                if b == NumberKind::kUnsigned {
                    return true;
                }
                if b == NumberKind::kFloat {
                    self.fBuilder.unary_op(BuilderOp::cast_to_float_from_int, n);
                    return true;
                }
            }
            NumberKind::kUnsigned => {
                if b == NumberKind::kSigned {
                    return true;
                }
                if b == NumberKind::kFloat {
                    self.fBuilder
                        .unary_op(BuilderOp::cast_to_float_from_uint, n);
                    return true;
                }
            }
            NumberKind::kBoolean => {
                if b == NumberKind::kFloat {
                    self.fBuilder.push_constant_f(1.0);
                } else if matches!(b, NumberKind::kSigned | NumberKind::kUnsigned) {
                    self.fBuilder.push_constant_i(1, 1);
                } else {
                    #[cfg(sk_debug)]
                    {
                        let mut description = ty.description();
                        description.push(0);
                        crate::SkDEBUGFAILF!(@source b"src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp\0".as_ptr(),2930; "unexpected cast from bool to %s",description.as_ptr());
                    }
                    return unsupported();
                }
                self.fBuilder.push_duplicates(n - 1);
                self.fBuilder.binary_op(BuilderOp::bitwise_and_n_ints, n);
                return true;
            }
            NumberKind::kFloat => {
                if b == NumberKind::kSigned {
                    self.fBuilder.unary_op(BuilderOp::cast_to_int_from_float, n);
                    return true;
                }
                if b == NumberKind::kUnsigned {
                    self.fBuilder
                        .unary_op(BuilderOp::cast_to_uint_from_float, n);
                    return true;
                }
            }
            NumberKind::kNonnumeric => {}
            _ => return unsupported(),
        }
        if b == NumberKind::kBoolean {
            self.fBuilder.push_zeros(n);
            return self.binaryOp(inner.expression_type(), kNotEqualOps);
        }
        #[cfg(sk_debug)]
        {
            let mut outer_description = ty.description();
            outer_description.push(0);
            let mut inner_description = inner.expression_type().description();
            inner_description.push(0);
            crate::SkDEBUGFAILF!(@source b"src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp\0".as_ptr(),2958; "unexpected cast from %s to %s",outer_description.as_ptr(),inner_description.as_ptr());
        }
        unsupported()
    }
    unsafe fn pushConstructorDiagonalMatrix(
        &mut self,
        c: &crate::src::sksl::ir::SkSLConstructorDiagonalMatrix::ConstructorDiagonalMatrix,
    ) -> bool {
        if self.pushImmutableData(c) {
            return true;
        }
        self.fBuilder.push_zeros(1);
        let arg = &**c.constructor_view().argumentSpan()[0]
            .as_ref()
            .unwrap_unchecked();
        if !self.pushExpression(arg, true) {
            return unsupported();
        }
        self.fBuilder
            .diagonal_matrix(c.expression_type().columns(), c.expression_type().rows());
        true
    }
    unsafe fn pushConstructorMatrixResize(
        &mut self,
        c: &crate::src::sksl::ir::SkSLConstructorMatrixResize::ConstructorMatrixResize,
    ) -> bool {
        let arg = &**c.constructor_view().argumentSpan()[0]
            .as_ref()
            .unwrap_unchecked();
        if !self.pushExpression(arg, true) {
            return unsupported();
        }
        self.fBuilder.matrix_resize(
            arg.expression_type().columns(),
            arg.expression_type().rows(),
            c.expression_type().columns(),
            c.expression_type().rows(),
        );
        true
    }
    unsafe fn pushConstructorSplat(
        &mut self,
        c: &crate::src::sksl::ir::SkSLConstructorSplat::ConstructorSplat,
    ) -> bool {
        let arg = &**c.constructor_view().argumentSpan()[0]
            .as_ref()
            .unwrap_unchecked();
        if !self.pushExpression(arg, true) {
            return unsupported();
        }
        self.fBuilder
            .push_duplicates(c.expression_type().slotCount() as i32 - 1);
        true
    }
    unsafe fn pushFunctionCall(&mut self, c: &FunctionCall) -> bool {
        if c.function().isIntrinsic() {
            return self.pushIntrinsicCall(c);
        }
        let last = self.fCurrentFunction;
        let f = c.function().definition().unwrap_unchecked();
        self.fCurrentFunction = Some(NonNull::from(f));
        let skip = self.fBuilder.nextLabelID();
        self.fBuilder.branch_if_no_lanes_active(skip);
        let Some(r) = self.writeFunction(c, f, c.arguments().as_base().as_slice()) else {
            return unsupported();
        };
        if self.needsFunctionResultSlots(f) {
            self.fBuilder.push_slots(r);
        }
        self.fCurrentFunction = last;
        self.fBuilder.label(skip);
        true
    }
    unsafe fn pushIntrinsicCall(&mut self, c: &FunctionCall) -> bool {
        let args = c.arguments().as_base().as_slice();
        let k = c.function().intrinsicKind();
        match args.len() {
            1 => self.pushIntrinsic1(k, &**args[0].as_ref().unwrap_unchecked()),
            2 => self.pushIntrinsic2(
                k,
                &**args[0].as_ref().unwrap_unchecked(),
                &**args[1].as_ref().unwrap_unchecked(),
            ),
            3 => self.pushIntrinsic3(
                k,
                &**args[0].as_ref().unwrap_unchecked(),
                &**args[1].as_ref().unwrap_unchecked(),
                &**args[2].as_ref().unwrap_unchecked(),
            ),
            _ => unsupported(),
        }
    }
    fn pushAbsFloatIntrinsic(&mut self, n: i32) -> bool {
        self.fBuilder.push_constant_u(0x7fffffff, n);
        self.fBuilder.binary_op(BuilderOp::bitwise_and_n_ints, n);
        true
    }
    fn pushLengthIntrinsic(&mut self, n: i32) -> bool {
        if n == 1 {
            return self.pushAbsFloatIntrinsic(1);
        }
        self.fBuilder.push_clone(n, 0);
        self.fBuilder.dot_floats(n);
        self.fBuilder.unary_op(BuilderOp::sqrt_float, 1);
        true
    }
    unsafe fn pushVectorizedExpression(&mut self, e: &dyn Expression, ty: &dyn Type) -> bool {
        if !self.pushExpression(e, true) {
            return unsupported();
        }
        let n = ty.slotCount();
        let m = e.expression_type().slotCount();
        if n > m {
            debug_assert!(m == 1);
            self.fBuilder.push_duplicates((n - m) as i32);
        }
        true
    }
    unsafe fn intrinsicTyped1(&mut self, ops: TypedOps, a: &dyn Expression) -> bool {
        self.pushExpression(a, true) && self.unaryOp(a.expression_type(), ops)
    }
    unsafe fn intrinsicOp1(&mut self, op: BuilderOp, a: &dyn Expression) -> bool {
        if !self.pushExpression(a, true) {
            return unsupported();
        }
        self.fBuilder
            .unary_op(op, a.expression_type().slotCount() as i32);
        true
    }
    unsafe fn intrinsicTyped2(
        &mut self,
        ops: TypedOps,
        a: &dyn Expression,
        b: &dyn Expression,
    ) -> bool {
        self.pushExpression(a, true)
            && self.pushVectorizedExpression(b, a.expression_type())
            && self.binaryOp(a.expression_type(), ops)
    }
    unsafe fn intrinsicOp2(
        &mut self,
        op: BuilderOp,
        a: &dyn Expression,
        b: &dyn Expression,
    ) -> bool {
        if !self.pushExpression(a, true) || !self.pushVectorizedExpression(b, a.expression_type()) {
            return unsupported();
        }
        self.fBuilder
            .binary_op(op, a.expression_type().slotCount() as i32);
        true
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:3104-3335
impl Generator {
    unsafe fn pushIntrinsic1(&mut self, k: IntrinsicKind, a: &dyn Expression) -> bool {
        let ty = a.expression_type();
        let n = ty.slotCount() as i32;
        match k {
            IntrinsicKind::k_abs_IntrinsicKind => {
                if ty.componentType().isFloat() {
                    if !self.pushExpression(a, true) {
                        return unsupported();
                    }
                    self.pushAbsFloatIntrinsic(n)
                } else {
                    self.intrinsicOp1(BuilderOp::abs_int, a)
                }
            }
            IntrinsicKind::k_any_IntrinsicKind | IntrinsicKind::k_all_IntrinsicKind => {
                if !self.pushExpression(a, true) {
                    return unsupported();
                }
                self.foldWithMultiOp(
                    if k == IntrinsicKind::k_any_IntrinsicKind {
                        BuilderOp::bitwise_or_n_ints
                    } else {
                        BuilderOp::bitwise_and_n_ints
                    },
                    n,
                );
                true
            }
            IntrinsicKind::k_acos_IntrinsicKind => self.intrinsicOp1(BuilderOp::acos_float, a),
            IntrinsicKind::k_asin_IntrinsicKind => self.intrinsicOp1(BuilderOp::asin_float, a),
            IntrinsicKind::k_atan_IntrinsicKind => self.intrinsicOp1(BuilderOp::atan_float, a),
            IntrinsicKind::k_ceil_IntrinsicKind => self.intrinsicOp1(BuilderOp::ceil_float, a),
            IntrinsicKind::k_cos_IntrinsicKind => self.intrinsicOp1(BuilderOp::cos_float, a),
            IntrinsicKind::k_degrees_IntrinsicKind => {
                let lit = Literal::new(
                    Position::default(),
                    57.2957795131f32 as f64,
                    ty.componentType(),
                );
                self.pushBinaryExpression(a, Operator::new(Op::STAR), &lit)
            }
            IntrinsicKind::k_floatBitsToInt_IntrinsicKind
            | IntrinsicKind::k_floatBitsToUint_IntrinsicKind
            | IntrinsicKind::k_intBitsToFloat_IntrinsicKind
            | IntrinsicKind::k_uintBitsToFloat_IntrinsicKind => self.pushExpression(a, true),
            IntrinsicKind::k_exp_IntrinsicKind => self.intrinsicOp1(BuilderOp::exp_float, a),
            IntrinsicKind::k_exp2_IntrinsicKind => self.intrinsicOp1(BuilderOp::exp2_float, a),
            IntrinsicKind::k_floor_IntrinsicKind => self.intrinsicOp1(BuilderOp::floor_float, a),
            IntrinsicKind::k_fract_IntrinsicKind => {
                if !self.pushExpression(a, true) {
                    return unsupported();
                }
                self.fBuilder.push_clone(n, 0);
                self.fBuilder.unary_op(BuilderOp::floor_float, n);
                self.binaryOp(ty, kSubtractOps)
            }
            IntrinsicKind::k_inverse_IntrinsicKind => {
                debug_assert!(ty.isMatrix() && ty.rows() == ty.columns());
                if !self.pushExpression(a, true) {
                    return unsupported();
                }
                self.fBuilder.inverse_matrix(ty.rows());
                true
            }
            IntrinsicKind::k_inversesqrt_IntrinsicKind => self.intrinsicTyped1(kInverseSqrtOps, a),
            IntrinsicKind::k_length_IntrinsicKind => {
                self.pushExpression(a, true) && self.pushLengthIntrinsic(n)
            }
            IntrinsicKind::k_log_IntrinsicKind | IntrinsicKind::k_log2_IntrinsicKind => {
                if !self.pushExpression(a, true) {
                    return unsupported();
                }
                self.fBuilder.unary_op(
                    if k == IntrinsicKind::k_log_IntrinsicKind {
                        BuilderOp::log_float
                    } else {
                        BuilderOp::log2_float
                    },
                    n,
                );
                true
            }
            IntrinsicKind::k_normalize_IntrinsicKind => {
                if !self.pushExpression(a, true) {
                    return unsupported();
                }
                if n > 1 {
                    self.fBuilder.push_clone(n, 0);
                    self.fBuilder.push_clone(n, 0);
                    self.fBuilder.dot_floats(n);
                    #[cfg(SK_USE_RSQRT_IN_RP_NORMALIZE)]
                    {
                        self.fBuilder.unary_op(BuilderOp::invsqrt_float, 1);
                        self.fBuilder.push_duplicates(n - 1);
                        self.binaryOp(ty, kMultiplyOps)
                    }
                    #[cfg(not(SK_USE_RSQRT_IN_RP_NORMALIZE))]
                    {
                        self.fBuilder.unary_op(BuilderOp::sqrt_float, 1);
                        self.fBuilder.push_duplicates(n - 1);
                        self.binaryOp(ty, kDivideOps)
                    }
                } else {
                    self.fBuilder.push_clone(n, 0);
                    self.pushAbsFloatIntrinsic(1) && self.binaryOp(ty, kDivideOps)
                }
            }
            IntrinsicKind::k_not_IntrinsicKind => {
                self.pushPrefixExpression(Operator::new(Op::LOGICALNOT), a)
            }
            IntrinsicKind::k_radians_IntrinsicKind => {
                let lit = Literal::new(
                    Position::default(),
                    0.01745329251f32 as f64,
                    ty.componentType(),
                );
                self.pushBinaryExpression(a, Operator::new(Op::STAR), &lit)
            }
            IntrinsicKind::k_saturate_IntrinsicKind => {
                let zero = Literal::new(Position::default(), 0.0, ty.componentType());
                let one = Literal::new(Position::default(), 1.0, ty.componentType());
                self.pushIntrinsic3(IntrinsicKind::k_clamp_IntrinsicKind, a, &zero, &one)
            }
            IntrinsicKind::k_sign_IntrinsicKind => {
                if !self.pushExpression(a, true) {
                    return unsupported();
                }
                if ty.componentType().isFloat() {
                    let max =
                        Literal::new(Position::default(), f32::MAX as f64, ty.componentType());
                    if !self.pushVectorizedExpression(&max, ty) || !self.binaryOp(ty, kMultiplyOps)
                    {
                        return unsupported();
                    }
                }
                let neg = Literal::new(Position::default(), -1.0, ty.componentType());
                if !self.pushVectorizedExpression(&neg, ty) || !self.binaryOp(ty, kMaxOps) {
                    return unsupported();
                }
                let one = Literal::new(Position::default(), 1.0, ty.componentType());
                self.pushVectorizedExpression(&one, ty) && self.binaryOp(ty, kMinOps)
            }
            IntrinsicKind::k_sin_IntrinsicKind => self.intrinsicOp1(BuilderOp::sin_float, a),
            IntrinsicKind::k_sqrt_IntrinsicKind => self.intrinsicOp1(BuilderOp::sqrt_float, a),
            IntrinsicKind::k_tan_IntrinsicKind => self.intrinsicOp1(BuilderOp::tan_float, a),
            IntrinsicKind::k_transpose_IntrinsicKind => {
                debug_assert!(ty.isMatrix());
                if !self.pushExpression(a, true) {
                    return unsupported();
                }
                self.fBuilder.transpose(ty.columns(), ty.rows());
                true
            }
            IntrinsicKind::k_trunc_IntrinsicKind => {
                if !self.pushExpression(a, true) {
                    return unsupported();
                }
                self.fBuilder.unary_op(BuilderOp::cast_to_int_from_float, n);
                self.fBuilder.unary_op(BuilderOp::cast_to_float_from_int, n);
                true
            }
            IntrinsicKind::k_fromLinearSrgb_IntrinsicKind
            | IntrinsicKind::k_toLinearSrgb_IntrinsicKind => {
                debug_assert!(ty.matches(&**self.fContext.fTypes.as_ref().fHalf3));
                if !self.pushExpression(a, true) {
                    return unsupported();
                }
                if k == IntrinsicKind::k_fromLinearSrgb_IntrinsicKind {
                    self.fBuilder.invoke_from_linear_srgb();
                } else {
                    self.fBuilder.invoke_to_linear_srgb();
                }
                true
            }
            _ => unsupported(),
        }
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:3352-3515
    unsafe fn pushIntrinsic2(
        &mut self,
        k: IntrinsicKind,
        a: &dyn Expression,
        b: &dyn Expression,
    ) -> bool {
        let ty = a.expression_type();
        let n = ty.slotCount() as i32;
        match k {
            IntrinsicKind::k_equal_IntrinsicKind
            | IntrinsicKind::k_notEqual_IntrinsicKind
            | IntrinsicKind::k_lessThan_IntrinsicKind
            | IntrinsicKind::k_greaterThan_IntrinsicKind
            | IntrinsicKind::k_lessThanEqual_IntrinsicKind
            | IntrinsicKind::k_greaterThanEqual_IntrinsicKind
            | IntrinsicKind::k_matrixCompMult_IntrinsicKind
            | IntrinsicKind::k_pow_IntrinsicKind => debug_assert!(ty.matches(b.expression_type())),
            IntrinsicKind::k_min_IntrinsicKind
            | IntrinsicKind::k_max_IntrinsicKind
            | IntrinsicKind::k_mod_IntrinsicKind
            | IntrinsicKind::k_step_IntrinsicKind => debug_assert!(ty
                .componentType()
                .matches(b.expression_type().componentType())),
            _ => {}
        }
        match k {
            IntrinsicKind::k_atan_IntrinsicKind => {
                self.intrinsicOp2(BuilderOp::atan2_n_floats, a, b)
            }
            IntrinsicKind::k_cross_IntrinsicKind => {
                debug_assert!(ty.matches(b.expression_type()) && n == 3);
                let mut sub = AutoStack::new(NonNull::from(&mut *self));
                sub.enter();
                if !self.pushExpression(a, true) {
                    return unsupported();
                }
                sub.exit();
                sub.pushClone(3);
                self.fBuilder.swizzle(3, &[1, 2, 0]);
                sub.enter();
                self.fBuilder.swizzle(3, &[2, 0, 1]);
                sub.exit();
                sub.enter();
                if !self.pushExpression(b, true) {
                    return unsupported();
                }
                sub.exit();
                sub.pushClone(3);
                self.fBuilder.swizzle(3, &[2, 0, 1]);
                self.fBuilder.binary_op(BuilderOp::mul_n_floats, 3);
                sub.enter();
                self.fBuilder.swizzle(3, &[1, 2, 0]);
                self.fBuilder.binary_op(BuilderOp::mul_n_floats, 3);
                sub.exit();
                sub.pushClone(3);
                self.fBuilder.binary_op(BuilderOp::sub_n_floats, 3);
                sub.enter();
                self.discardExpression(3);
                sub.exit();
                true
            }
            IntrinsicKind::k_distance_IntrinsicKind => {
                debug_assert!(n == b.expression_type().slotCount() as i32);
                self.pushBinaryExpression(a, Operator::new(Op::MINUS), b)
                    && self.pushLengthIntrinsic(n)
            }
            IntrinsicKind::k_dot_IntrinsicKind => {
                debug_assert!(ty.matches(b.expression_type()));
                if !self.pushExpression(a, true) || !self.pushExpression(b, true) {
                    return unsupported();
                }
                self.fBuilder.dot_floats(n);
                true
            }
            IntrinsicKind::k_equal_IntrinsicKind => self.intrinsicTyped2(kEqualOps, a, b),
            IntrinsicKind::k_notEqual_IntrinsicKind => self.intrinsicTyped2(kNotEqualOps, a, b),
            IntrinsicKind::k_lessThan_IntrinsicKind => self.intrinsicTyped2(kLessThanOps, a, b),
            IntrinsicKind::k_greaterThan_IntrinsicKind => self.intrinsicTyped2(kLessThanOps, b, a),
            IntrinsicKind::k_lessThanEqual_IntrinsicKind => {
                self.intrinsicTyped2(kLessThanEqualOps, a, b)
            }
            IntrinsicKind::k_greaterThanEqual_IntrinsicKind => {
                self.intrinsicTyped2(kLessThanEqualOps, b, a)
            }
            IntrinsicKind::k_min_IntrinsicKind => self.intrinsicTyped2(kMinOps, a, b),
            IntrinsicKind::k_matrixCompMult_IntrinsicKind => {
                self.intrinsicTyped2(kMultiplyOps, a, b)
            }
            IntrinsicKind::k_max_IntrinsicKind => self.intrinsicTyped2(kMaxOps, a, b),
            IntrinsicKind::k_mod_IntrinsicKind => self.intrinsicTyped2(kModOps, a, b),
            IntrinsicKind::k_pow_IntrinsicKind => self.intrinsicOp2(BuilderOp::pow_n_floats, a, b),
            IntrinsicKind::k_reflect_IntrinsicKind => {
                debug_assert!(ty.matches(b.expression_type()) && ty.componentType().isFloat());
                if !self.pushExpression(a, true) || !self.pushExpression(b, true) {
                    return unsupported();
                }
                self.fBuilder.push_clone(2 * n, 0);
                self.fBuilder.dot_floats(n);
                self.fBuilder.push_constant_f(2.0);
                self.fBuilder.binary_op(BuilderOp::mul_n_floats, 1);
                self.fBuilder.push_duplicates(n - 1);
                self.fBuilder.binary_op(BuilderOp::mul_n_floats, n);
                self.fBuilder.binary_op(BuilderOp::sub_n_floats, n);
                true
            }
            IntrinsicKind::k_step_IntrinsicKind => {
                let bt = b.expression_type();
                if !self.pushVectorizedExpression(a, bt)
                    || !self.pushExpression(b, true)
                    || !self.binaryOp(bt, kLessThanEqualOps)
                {
                    return unsupported();
                }
                let one = Literal::new(Position::default(), 1.0, bt.componentType());
                if !self.pushVectorizedExpression(&one, bt) {
                    return unsupported();
                }
                self.fBuilder
                    .binary_op(BuilderOp::bitwise_and_n_ints, bt.slotCount() as i32);
                true
            }
            _ => unsupported(),
        }
    }
    // Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:3517-3637
    unsafe fn pushIntrinsic3(
        &mut self,
        k: IntrinsicKind,
        a: &dyn Expression,
        b: &dyn Expression,
        c: &dyn Expression,
    ) -> bool {
        let ty = a.expression_type();
        let n = ty.slotCount() as i32;
        match k {
            IntrinsicKind::k_clamp_IntrinsicKind => {
                debug_assert!(
                    ty.componentType()
                        .matches(b.expression_type().componentType())
                        && ty
                            .componentType()
                            .matches(c.expression_type().componentType())
                );
                if !self.pushExpression(a, true)
                    || !self.pushVectorizedExpression(b, ty)
                    || !self.binaryOp(ty, kMaxOps)
                    || !self.pushVectorizedExpression(c, ty)
                    || !self.binaryOp(ty, kMinOps)
                {
                    return unsupported();
                }
                true
            }
            IntrinsicKind::k_faceforward_IntrinsicKind => {
                debug_assert!(ty.matches(b.expression_type()) && ty.matches(c.expression_type()));
                if !self.pushExpression(a, true) {
                    return unsupported();
                }
                self.fBuilder.push_constant_f(0.0);
                if !self.pushExpression(b, true) || !self.pushExpression(c, true) {
                    return unsupported();
                }
                self.fBuilder.dot_floats(n);
                self.fBuilder.binary_op(BuilderOp::cmple_n_floats, 1);
                self.fBuilder.push_constant_u(0x80000000, 1);
                self.fBuilder.binary_op(BuilderOp::bitwise_and_n_ints, 1);
                self.fBuilder.push_duplicates(n - 1);
                self.fBuilder.binary_op(BuilderOp::bitwise_xor_n_ints, n);
                true
            }
            IntrinsicKind::k_mix_IntrinsicKind => {
                debug_assert!(ty.matches(b.expression_type()));
                if c.expression_type().componentType().isFloat() {
                    if !self.pushVectorizedExpression(c, ty)
                        || !self.pushExpression(a, true)
                        || !self.pushExpression(b, true)
                    {
                        return unsupported();
                    }
                    self.ternaryOp(ty, kMixOps)
                } else if c.expression_type().componentType().isBoolean() {
                    if !self.pushExpression(c, true)
                        || !self.pushExpression(a, true)
                        || !self.pushExpression(b, true)
                    {
                        return unsupported();
                    }
                    self.fBuilder.ternary_op(BuilderOp::mix_n_ints, n);
                    true
                } else {
                    unsupported()
                }
            }
            IntrinsicKind::k_refract_IntrinsicKind => {
                let padding = 4 - n;
                if !self.pushExpression(a, true) {
                    return unsupported();
                }
                self.fBuilder.push_zeros(padding);
                if !self.pushExpression(b, true) {
                    return unsupported();
                }
                self.fBuilder.push_zeros(padding);
                if !self.pushExpression(c, true) {
                    return unsupported();
                }
                self.fBuilder.refract_floats();
                self.fBuilder.discard_stack(padding);
                true
            }
            IntrinsicKind::k_smoothstep_IntrinsicKind => {
                debug_assert!(
                    ty.componentType().isFloat()
                        && b.expression_type().matches(ty)
                        && c.expression_type().componentType().isFloat()
                );
                if !self.pushVectorizedExpression(a, c.expression_type())
                    || !self.pushVectorizedExpression(b, c.expression_type())
                    || !self.pushExpression(c, true)
                {
                    return unsupported();
                }
                self.fBuilder.ternary_op(
                    BuilderOp::smoothstep_n_floats,
                    c.expression_type().slotCount() as i32,
                );
                true
            }
            _ => unsupported(),
        }
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:3639-3785
impl Generator {
    fn pushLiteral(&mut self, l: &Literal) -> bool {
        match l.expression_type().numberKind() {
            NumberKind::kFloat => self.fBuilder.push_constant_f(l.floatValue()),
            NumberKind::kSigned => self.fBuilder.push_constant_i(l.intValue() as i32, 1),
            NumberKind::kUnsigned => self.fBuilder.push_constant_u(l.intValue() as u32, 1),
            NumberKind::kBoolean => self
                .fBuilder
                .push_constant_i(if l.boolValue() != 0 { !0 } else { 0 }, 1),
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
        true
    }
    unsafe fn pushPostfixExpression(&mut self, p: &PostfixExpression, uses: bool) -> bool {
        let arg = &**p.operand().as_ref().unwrap_unchecked();
        if !uses {
            return self.pushPrefixExpression(p.getOperator(), arg);
        }
        let Some(mut lv) = self.makeLValue(arg, false) else {
            return unsupported();
        };
        if !self.push(&mut lv) {
            return unsupported();
        }
        let ty = p.expression_type();
        let n = ty.slotCount() as i32;
        self.fBuilder.push_clone(n, 0);
        let one = Literal::new(Position::default(), 1.0, ty.componentType());
        if !self.pushVectorizedExpression(&one, ty) {
            return unsupported();
        }
        let ok = match p.getOperator().kind() {
            Op::PLUSPLUS => self.binaryOp(ty, kAddOps),
            Op::MINUSMINUS => self.binaryOp(ty, kSubtractOps),
            _ => core::hint::unreachable_unchecked(),
        };
        if !ok || !self.store(&mut lv) {
            return unsupported();
        }
        self.discardExpression(n);
        true
    }
    unsafe fn pushPrefixExpression(&mut self, op: Operator, e: &dyn Expression) -> bool {
        let ty = e.expression_type();
        let n = ty.slotCount() as i32;
        match op.kind() {
            Op::BITWISENOT | Op::LOGICALNOT => {
                if !self.pushExpression(e, true) {
                    return unsupported();
                }
                self.fBuilder.push_constant_u(!0, n);
                self.fBuilder.binary_op(BuilderOp::bitwise_xor_n_ints, n);
                true
            }
            Op::MINUS => {
                if !self.pushExpression(e, true) {
                    return unsupported();
                }
                if ty.componentType().isFloat() {
                    self.fBuilder.push_constant_u(0x80000000, n);
                    self.fBuilder.binary_op(BuilderOp::bitwise_xor_n_ints, n);
                } else {
                    self.fBuilder.push_constant_i(-1, n);
                    self.fBuilder.binary_op(BuilderOp::mul_n_ints, n);
                }
                true
            }
            Op::PLUSPLUS => {
                let one = Literal::new(Position::default(), 1.0, ty.componentType());
                self.pushBinaryExpression(e, Operator::new(Op::PLUSEQ), &one)
            }
            Op::MINUSMINUS => {
                let minus = Literal::new(e.node().fPosition, -1.0, ty.componentType());
                self.pushBinaryExpression(e, Operator::new(Op::PLUSEQ), &minus)
            }
            _ => unsupported(),
        }
    }
    unsafe fn pushSwizzle(&mut self, s: &Swizzle) -> bool {
        let c = s.components().as_slice();
        debug_assert!(!c.is_empty() && c.len() <= 4);
        let simple = is_sliceable_swizzle(c);
        let base = &**s.base().as_ref().unwrap_unchecked();
        if simple && base.node().is::<VariableReference>() {
            return self.pushVariableReferencePartial(
                base.node().as_node::<VariableReference>(),
                SlotRange {
                    index: c[0] as i32,
                    count: c.len() as i32,
                },
            );
        }
        if !self.pushExpression(base, true) {
            return false;
        }
        if simple && c[0] == 0 {
            let discarded = base.expression_type().slotCount() as i32 - c.len() as i32;
            debug_assert!(discarded >= 0);
            self.fBuilder.discard_stack(discarded);
            return true;
        }
        self.fBuilder
            .swizzle(base.expression_type().slotCount() as i32, c);
        true
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:3787-3996
impl Generator {
    unsafe fn pushDynamicallyUniformTernaryExpression(
        &mut self,
        test: &dyn Expression,
        a: &dyn Expression,
        b: &dyn Expression,
    ) -> bool {
        crate::SkASSERT!(@source b"src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp\0".as_ptr(),3794; @condition b"Analysis::IsDynamicallyUniformExpression(test)\0".as_ptr(); Analysis::IsDynamicallyUniformExpression(test));
        let no = self.fBuilder.nextLabelID();
        let exit = self.fBuilder.nextLabelID();
        let mut stack = AutoStack::new(NonNull::from(&mut *self));
        stack.enter();
        if !self.pushExpression(test, true) {
            return unsupported();
        }
        self.fBuilder
            .branch_if_no_active_lanes_on_stack_top_equal(!0, no);
        stack.exit();
        if !self.pushExpression(a, true) {
            return unsupported();
        }
        self.fBuilder.jump(exit);
        self.discardExpression(a.expression_type().slotCount() as i32);
        self.fBuilder.label(no);
        if !self.pushExpression(b, true) {
            return unsupported();
        }
        self.fBuilder.label(exit);
        stack.enter();
        self.discardExpression(1);
        stack.exit();
        true
    }
    unsafe fn pushTernaryExpression(
        &mut self,
        test: &dyn Expression,
        a: &dyn Expression,
        b: &dyn Expression,
    ) -> bool {
        if Analysis::IsDynamicallyUniformExpression(test) {
            return self.pushDynamicallyUniformTernaryExpression(test, a, b);
        }
        let bside = Analysis::HasSideEffects(b);
        let aside = Analysis::HasSideEffects(a);
        let trivial = Analysis::IsTrivialExpression(a);
        let cleanup = self.fBuilder.nextLabelID();
        let n = a.expression_type().slotCount() as i32;
        if !bside && !aside && trivial {
            if !self.pushVectorizedExpression(test, a.expression_type())
                || !self.pushExpression(b, true)
                || !self.pushExpression(a, true)
            {
                return unsupported();
            }
            self.fBuilder.ternary_op(BuilderOp::mix_n_ints, n);
            return true;
        }
        self.fBuilder.enableExecutionMaskWrites();
        let mut stack = AutoStack::new(NonNull::from(&mut *self));
        stack.enter();
        self.fBuilder.push_condition_mask();
        if !self.pushExpression(test, true) {
            return unsupported();
        }
        stack.exit();
        if !bside {
            if !self.pushExpression(b, true) {
                return unsupported();
            }
            stack.enter();
            self.fBuilder.merge_condition_mask();
            stack.exit();
            if !trivial {
                self.fBuilder.branch_if_no_lanes_active(cleanup);
            }
            if !self.pushExpression(a, true) {
                return unsupported();
            }
            self.fBuilder.select(n);
            self.fBuilder.label(cleanup);
        } else {
            stack.enter();
            self.fBuilder.merge_condition_mask();
            stack.exit();
            if !self.pushExpression(a, true) {
                return unsupported();
            }
            stack.enter();
            self.fBuilder.merge_inv_condition_mask();
            stack.exit();
            if !self.pushExpression(b, true) {
                return unsupported();
            }
            self.fBuilder.select(n);
        }
        stack.enter();
        self.discardExpression(1);
        self.fBuilder.pop_condition_mask();
        stack.exit();
        self.fBuilder.disableExecutionMaskWrites();
        true
    }
    unsafe fn pushVariableReference(&mut self, v: &VariableReference) -> bool {
        let var = v.variable().unwrap_unchecked().as_ref();
        if v.expression_type().isScalar() || v.expression_type().isVector() {
            if let Some(expr) = ConstantFolder::GetConstantValueOrNull(v) {
                return self.pushExpression(expr, true);
            }
            if self.fImmutableVariables.contains(&var_key(var)) {
                return self.pushExpression(var.initialValue().unwrap_unchecked(), true);
            }
        }
        self.pushVariableReferencePartial(
            v,
            SlotRange {
                index: 0,
                count: v.expression_type().slotCount() as i32,
            },
        )
    }
    unsafe fn pushVariableReferencePartial(
        &mut self,
        v: &VariableReference,
        subset: SlotRange,
    ) -> bool {
        let var = v.variable().unwrap_unchecked().as_ref();
        if Self::IsUniform(var) {
            let mut r = self.getUniformSlots(var);
            debug_assert!(r.count == var.symbol_type().slotCount() as i32);
            r.index += subset.index;
            r.count = subset.count;
            self.fBuilder.push_uniform(r);
        } else if self.fImmutableVariables.contains(&var_key(var)) {
            if subset.count == 1 {
                let value = var.initialValue().unwrap_unchecked();
                if let Some(bits) = self.getImmutableBitsForSlot(value, subset.index as usize) {
                    self.fBuilder.push_constant_i(bits, 1);
                    return true;
                }
            }
            let mut r = self.getImmutableSlots(var);
            debug_assert!(r.count == var.symbol_type().slotCount() as i32);
            r.index += subset.index;
            r.count = subset.count;
            self.fBuilder.push_immutable(r);
        } else {
            let mut r = self.getVariableSlots(var);
            debug_assert!(r.count == var.symbol_type().slotCount() as i32);
            r.index += subset.index;
            r.count = subset.count;
            self.fBuilder.push_slots(r);
        }
        true
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:3998-4091
impl Generator {
    unsafe fn writeProgram(&mut self, function: &FunctionDefinition) -> bool {
        self.fCurrentFunction = Some(NonNull::from(function));
        if let Some(mut trace) = self.fDebugTrace {
            trace
                .as_mut()
                .setSource(self.fProgram.as_ref().fSource.as_ref().unwrap_unchecked());
            if self.fWriteTraceOps {
                let mut mask = AutoStack::new(NonNull::from(&mut *self));
                mask.enter();
                self.fBuilder.push_device_xy01();
                self.fBuilder.discard_stack(2);
                self.fBuilder
                    .push_constant_f(trace.as_ref().fTraceCoord.fX as f32 + 0.5);
                self.fBuilder
                    .push_constant_f(trace.as_ref().fTraceCoord.fY as f32 + 0.5);
                self.fBuilder.binary_op(BuilderOp::cmpeq_n_floats, 2);
                self.fBuilder.binary_op(BuilderOp::bitwise_and_n_ints, 1);
                mask.exit();
                self.fTraceMask = Some(mask);
                self.calculateLineOffsets();
            }
        }
        let decl = function.declaration();
        let key = |v: Option<&dyn Variable>| v.map(var_key);
        let coords = key(decl.getMainCoordsParameter());
        let input = key(decl.getMainInputColorParameter());
        let dest = key(decl.getMainDestColorParameter());
        for param in decl.parameters() {
            let param = param.unwrap_unchecked().as_ref();
            let id = Some(var_key(param));
            if id == coords {
                let r = self.getVariableSlots(param);
                debug_assert!(r.count == 2);
                self.fBuilder.store_src_rg(r);
            } else if id == input {
                let r = self.getVariableSlots(param);
                debug_assert!(r.count == 4);
                self.fBuilder.store_src(r);
            } else if id == dest {
                let r = self.getVariableSlots(param);
                debug_assert!(r.count == 4);
                self.fBuilder.store_dst(r);
            } else {
                debug_assert!(false, "invalid main parameter");
                return unsupported();
            }
        }
        self.fBuilder.init_lane_masks();
        if !self.writeGlobals() {
            return unsupported();
        }
        let Some(main) = self.writeFunction(function, function, &[]) else {
            return unsupported();
        };
        debug_assert!(main.count == 4);
        if self.needsFunctionResultSlots(function) {
            self.fBuilder.load_src(main);
        } else {
            self.fBuilder.pop_src_rgba();
        }
        if let Some(mask) = self.fTraceMask.as_mut() {
            mask.enter();
            self.fBuilder.discard_stack(1);
            mask.exit();
        }
        true
    }
    unsafe fn finish(&mut self) -> Box<RPProgram> {
        self.fBuilder.finish(
            self.fProgramSlots.slotCount(),
            self.fUniformSlots.slotCount(),
            self.fImmutableSlots.slotCount(),
            self.fDebugTrace,
        )
    }
}
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.h:30-33
// Source: src/sksl/codegen/SkSLRasterPipelineCodeGenerator.cpp:4093-4102
/// Safety: source program/context/type/symbol/pool leases remain live throughout
/// generation; debugTrace remains valid and exclusively mutable through generation
/// and retains the native trace backing lease for the returned RP program.
pub unsafe fn MakeRasterPipelineProgram(
    program: &Program,
    function: &FunctionDefinition,
    debugTrace: Option<NonNull<DebugTracePriv>>,
    writeTraceOps: bool,
) -> Option<Box<RPProgram>> {
    let mut generator = Generator::new(program, debugTrace, writeTraceOps);
    let g = generator.as_mut().get_unchecked_mut();
    if !g.writeProgram(function) {
        return None;
    }
    Some(g.finish())
}
