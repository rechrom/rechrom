// UNVERIFIED: no validation was run.
use super::{
    SkSLExpression::{Expression, ExpressionData, ExpressionExt},
    SkSLIRNode::*,
    SkSLType::Type,
};
use crate::src::sksl::{
    SkSLContext::Context, SkSLOperator::OperatorPrecedence, SkSLPosition::Position,
};
use core::{pin::Pin, ptr::NonNull};
// Source: src/sksl/ir/SkSLEmptyExpression.h:21-23
#[repr(C)]
pub struct EmptyExpression {
    base: ExpressionData,
}
impl IRNodeType for EmptyExpression {
    const kIRNodeKind: i32 = ExpressionKind::kEmpty as i32;
}
impl EmptyExpression {
    // Source: src/sksl/ir/SkSLEmptyExpression.h:25-27
    pub unsafe fn Make(pos: Position, context: &Context) -> IRNodeBox<dyn Expression> {
        Self::new(
            pos,
            NonNull::new_unchecked(context.fTypes.as_ref().fVoid.as_ptr() as *mut dyn Type),
        )
        .cast(|p| p as *mut dyn Expression)
    }
    // Source: src/sksl/ir/SkSLEmptyExpression.h:29-32
    pub unsafe fn new(pos: Position, ty: NonNull<dyn Type>) -> IRNodeBox<Self> {
        debug_assert!(ty.as_ref().isVoid());
        IRNodeBox::new_with(|p: *mut Self| {
            p.write(Self {
                base: ExpressionData::new(pos, ExpressionKind::kEmpty, Some(ty)),
            })
        })
    }
}
impl IRNode for EmptyExpression {
    fn node(&self) -> &IRNodeData {
        &self.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base.base) }
    }
    fn description(&self) -> Vec<u8> {
        self.expression_description()
    }
}
unsafe impl Expression for EmptyExpression {
    fn expression_data(&self) -> &ExpressionData {
        &self.base
    }
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base) }
    }
    fn as_expression(&self) -> &dyn Expression {
        self
    }
    // Source: src/sksl/ir/SkSLEmptyExpression.h:34-36
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        Some(
            Self::new(
                pos,
                core::mem::transmute::<NonNull<dyn Type + '_>, NonNull<dyn Type>>(NonNull::from(
                    self.expression_type(),
                )),
            )
            .cast(|p| p as *mut dyn Expression),
        )
    }
    // Source: src/sksl/ir/SkSLEmptyExpression.h:38-43
    fn description_with_precedence(&self, _: OperatorPrecedence) -> Vec<u8> {
        b"false".to_vec()
    }
}
