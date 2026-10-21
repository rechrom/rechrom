// UNVERIFIED: user prohibited all validation.
use super::{
    SkSLExpression::{Expression, ExpressionData, ExpressionExt},
    SkSLIRNode::*,
    SkSLType::Type,
};
use crate::src::sksl::{
    SkSLContext::Context, SkSLOperator::OperatorPrecedence, SkSLPosition::Position,
};
use core::{pin::Pin, ptr::NonNull};
unsafe fn borrow_type(t: &dyn Type) -> NonNull<dyn Type> {
    core::mem::transmute::<NonNull<dyn Type + '_>, NonNull<dyn Type>>(NonNull::from(t))
}
// Source: src/sksl/ir/SkSLPoison.h:17-17
#[repr(C)]
pub struct Poison {
    base: ExpressionData,
}
// Source: src/sksl/ir/SkSLPoison.h:19-19
impl IRNodeType for Poison {
    const kIRNodeKind: i32 = ExpressionKind::kPoison as i32;
}
impl Poison {
    // Source: src/sksl/ir/SkSLPoison.h:21-23
    pub unsafe fn Make(pos: Position, context: &Context) -> IRNodeBox<dyn Expression> {
        let ty = borrow_type(&**context.fTypes.as_ref().fPoison);
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, ty)))
            .cast(|p| p as *mut dyn Expression)
    }
    // Source: src/sksl/ir/SkSLPoison.h:25-26
    pub unsafe fn new(pos: Position, ty: NonNull<dyn Type>) -> Self {
        Self {
            base: ExpressionData::new(pos, ExpressionKind::kPoison, Some(ty)),
        }
    }
}
impl IRNode for Poison {
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
unsafe impl Expression for Poison {
    fn expression_data(&self) -> &ExpressionData {
        &self.base
    }
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base) }
    }
    fn as_expression(&self) -> &dyn Expression {
        self
    }
    // Source: src/sksl/ir/SkSLPoison.h:28-30
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        Some(
            IRNodeBox::new_with(|p: *mut Self| {
                p.write(Self::new(pos, borrow_type(self.expression_type())))
            })
            .cast(|p| p as *mut dyn Expression),
        )
    }
    // Source: src/sksl/ir/SkSLPoison.h:32-34
    fn description_with_precedence(&self, _: OperatorPrecedence) -> Vec<u8> {
        let tag = &crate::src::sksl::SkSLCompiler::POISON_TAG;
        tag[..tag.len() - 1].to_vec()
    }
}
