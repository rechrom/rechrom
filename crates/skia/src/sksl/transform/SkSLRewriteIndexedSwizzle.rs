// UNVERIFIED: no validation was run.
use crate::src::sksl::{
    ir::{
        SkSLConstructorCompound::ConstructorCompound,
        SkSLExpression::{Expression, ExpressionExt},
        SkSLIRNode::{IRNode, IRNodeBox},
        SkSLIndexExpression::IndexExpression,
        SkSLSwizzle::Swizzle,
    },
    SkSLContext::Context,
};
// Source: src/sksl/transform/SkSLRewriteIndexedSwizzle.cpp:21-49
pub unsafe fn RewriteIndexedSwizzle(
    context: &Context,
    index: &IndexExpression,
) -> Option<IRNodeBox<dyn Expression>> {
    if !index
        .base()
        .as_ref()
        .unwrap_unchecked()
        .node()
        .is::<Swizzle>()
    {
        return None;
    }
    let swizzle = index
        .base()
        .as_ref()
        .unwrap_unchecked()
        .node()
        .as_node::<Swizzle>();
    let mut values = [0.0f64; 4];
    for (i, c) in swizzle.components().as_slice().iter().enumerate() {
        values[i] = *c as f64;
    }
    let ty = context
        .fTypes
        .as_ref()
        .fInt
        .toCompound(context, swizzle.components().size(), 1);
    let vector =
        ConstructorCompound::MakeFromConstants(context, index.node().fPosition, ty, &values);
    let inner = IndexExpression::Make(
        context,
        index.node().fPosition,
        vector,
        index
            .index()
            .as_ref()
            .unwrap_unchecked()
            .clone_current()
            .unwrap_unchecked(),
    );
    Some(IndexExpression::Make(
        context,
        index.node().fPosition,
        swizzle
            .base()
            .as_ref()
            .unwrap_unchecked()
            .clone_current()
            .unwrap_unchecked(),
        inner,
    ))
}
