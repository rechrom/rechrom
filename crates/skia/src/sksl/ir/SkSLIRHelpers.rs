// UNVERIFIED direct pinned-source translation; validation prohibited by user.
use super::{
    SkSLBinaryExpression::BinaryExpression,
    SkSLConstructorCompound::ConstructorCompound,
    SkSLExpression::Expression,
    SkSLExpressionStatement::ExpressionStatement,
    SkSLFieldAccess::{FieldAccess, OwnerKind},
    SkSLIRNode::IRNodeBox,
    SkSLIndexExpression::IndexExpression,
    SkSLLiteral::Literal,
    SkSLStatement::Statement,
    SkSLSwizzle::{ComponentArray, Swizzle},
    SkSLVariable::Variable,
    SkSLVariableReference::{VariableRefKind, VariableReference},
};
use crate::src::sksl::{
    SkSLAnalysis,
    SkSLContext::Context,
    SkSLDefines::ExpressionArray,
    SkSLOperator::{Operator, OperatorKind},
    SkSLPosition::Position,
};
use core::ptr::NonNull;
// Source: src/sksl/ir/SkSLIRHelpers.h:39-40
// Source: src/sksl/ir/SkSLIRHelpers.h:109-109
// Native Context reference is borrowed; expression/statement owners remain the
// actual allocation-pool IRNodeBox, never a second owner or copied IR graph.
pub struct IRHelpers<'a> {
    pub fContext: &'a Context,
}
impl<'a> IRHelpers<'a> {
    // Source: src/sksl/ir/SkSLIRHelpers.h:40-40
    pub unsafe fn new(context: &'a Context) -> Self {
        Self { fContext: context }
    }
    // Source: src/sksl/ir/SkSLIRHelpers.h:46-48
    pub unsafe fn Ref(&self, var: NonNull<dyn Variable>) -> IRNodeBox<dyn Expression> {
        VariableReference::Make(Position::new(), var, VariableRefKind::kRead)
    }
    // Source: src/sksl/ir/SkSLIRHelpers.h:50-53
    pub unsafe fn Field(&self, var: NonNull<dyn Variable>, idx: i32) -> IRNodeBox<dyn Expression> {
        FieldAccess::Make(
            self.fContext,
            Position::new(),
            self.Ref(var),
            idx,
            OwnerKind::kAnonymousInterfaceBlock,
        )
    }
    // Source: src/sksl/ir/SkSLIRHelpers.h:55-58
    pub unsafe fn Swizzle(
        &self,
        base: IRNodeBox<dyn Expression>,
        components: ComponentArray,
    ) -> IRNodeBox<dyn Expression> {
        let pos = base.node().fPosition;
        Swizzle::Make(self.fContext, pos, base, components)
    }
    // Source: src/sksl/ir/SkSLIRHelpers.h:60-64
    pub unsafe fn Index(
        &self,
        base: IRNodeBox<dyn Expression>,
        idx: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Expression> {
        let pos = base.node().fPosition.rangeThrough(idx.node().fPosition);
        IndexExpression::Make(self.fContext, pos, base, idx)
    }
    // Source: src/sksl/ir/SkSLIRHelpers.h:66-71
    pub unsafe fn Binary(
        &self,
        left: IRNodeBox<dyn Expression>,
        op: Operator,
        right: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Expression> {
        let pos = left.node().fPosition.rangeThrough(right.node().fPosition);
        BinaryExpression::Make(self.fContext, pos, left, op, right)
    }
    // Source: src/sksl/ir/SkSLIRHelpers.h:73-76
    pub unsafe fn Mul(
        &self,
        left: IRNodeBox<dyn Expression>,
        right: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Expression> {
        self.Binary(left, Operator::new(OperatorKind::STAR), right)
    }
    // Source: src/sksl/ir/SkSLIRHelpers.h:78-81
    pub unsafe fn Add(
        &self,
        left: IRNodeBox<dyn Expression>,
        right: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Expression> {
        self.Binary(left, Operator::new(OperatorKind::PLUS), right)
    }
    // Source: src/sksl/ir/SkSLIRHelpers.h:83-85
    pub unsafe fn Float(&self, value: f32) -> IRNodeBox<dyn Expression> {
        Literal::MakeFloat(
            Position::new(),
            value,
            &**self.fContext.fTypes.as_ref().fFloat,
        )
        .cast(|p| p as *mut dyn Expression)
    }
    // Source: src/sksl/ir/SkSLIRHelpers.h:87-89
    pub unsafe fn Int(&self, value: i32) -> IRNodeBox<dyn Expression> {
        Literal::MakeInt(
            Position::new(),
            value as _,
            &**self.fContext.fTypes.as_ref().fInt,
        )
        .cast(|p| p as *mut dyn Expression)
    }
    // Source: src/sksl/ir/SkSLIRHelpers.h:91-100
    pub unsafe fn CtorXYZW(
        &self,
        xy: IRNodeBox<dyn Expression>,
        z: IRNodeBox<dyn Expression>,
        w: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Expression> {
        let mut args = ExpressionArray::new();
        args.as_mut().as_base_mut().push_back(Some(xy));
        args.as_mut().as_base_mut().push_back(Some(z));
        args.as_mut().as_base_mut().push_back(Some(w));
        ConstructorCompound::Make(
            self.fContext,
            Position::new(),
            &**self.fContext.fTypes.as_ref().fFloat4,
            args,
        )
    }
    // Source: src/sksl/ir/SkSLIRHelpers.h:102-107
    pub unsafe fn Assign(
        &self,
        left: IRNodeBox<dyn Expression>,
        right: IRNodeBox<dyn Expression>,
    ) -> IRNodeBox<dyn Statement> {
        let updated = SkSLAnalysis::UpdateVariableRefKind(
            left.as_ptr() as *mut dyn Expression,
            VariableRefKind::kWrite,
            None,
        );
        debug_assert!(updated);
        ExpressionStatement::Make(
            self.fContext,
            self.Binary(left, Operator::new(OperatorKind::EQ), right),
        )
    }
}
