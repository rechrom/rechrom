// New translation: unverified by explicit user instruction; no compile/test/oracle was run.
use super::{
    SkSLConstructor::*,
    SkSLExpression::{ComparisonResult, Expression, ExpressionData, ExpressionExt},
    SkSLIRNode::*,
    SkSLType::Type,
};
use crate::src::sksl::{
    SkSLContext::Context, SkSLDefines::ExpressionArray, SkSLOperator::OperatorPrecedence,
    SkSLPosition::Position,
};
use core::{pin::Pin, ptr::NonNull};
// Source: src/sksl/ir/SkSLConstructorCompound.h:33-33
// Adapted native inherited subobject only; no extra constructor state.
#[repr(C)]
pub struct ConstructorCompound {
    pub(crate) base: MultiArgumentConstructorData,
}
// Source: src/sksl/ir/SkSLConstructorCompound.h:35-35
impl IRNodeType for ConstructorCompound {
    const kIRNodeKind: i32 = ExpressionKind::kConstructorCompound as i32;
}
impl ConstructorCompound {
    // Source: src/sksl/ir/SkSLConstructorCompound.h:37-38
    /// Safety: Type and child-expression borrowed dependencies retain their native lifetime/pool lease.
    pub unsafe fn new(pos: Position, ty: &dyn Type, arg: Pin<Box<ExpressionArray>>) -> Self {
        let ty =
            core::mem::transmute::<NonNull<dyn Type + '_>, NonNull<dyn Type>>(NonNull::from(ty));
        Self {
            base: MultiArgumentConstructorData::new(
                pos,
                ExpressionKind::kConstructorCompound,
                ty,
                arg,
            ),
        }
    }
    pub fn arguments(&self) -> &ExpressionArray {
        self.base.arguments()
    }
    pub fn arguments_mut(&mut self) -> Pin<&mut ExpressionArray> {
        self.base.arguments_mut()
    }
}
impl IRNode for ConstructorCompound {
    fn node(&self) -> &IRNodeData {
        &self.base.base.base.base
    }
    fn node_mut(self: Pin<&mut Self>) -> Pin<&mut IRNodeData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base.base.base.base) }
    }
    fn description(&self) -> Vec<u8> {
        self.expression_description()
    }
}
unsafe impl Expression for ConstructorCompound {
    fn expression_data(&self) -> &ExpressionData {
        &self.base.base.base
    }
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base.base.base) }
    }
    fn as_expression(&self) -> &dyn Expression {
        self
    }
    unsafe fn constructor_view(&self) -> &dyn AnyConstructor {
        self
    }
    unsafe fn constructor_view_mut(self: Pin<&mut Self>) -> Pin<&mut dyn AnyConstructor> {
        self
    }
    // Source: src/sksl/ir/SkSLConstructorCompound.h:50-52
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        let arg = self.arguments().clone();
        Some(
            IRNodeBox::new_with(|p: *mut Self| {
                p.write(Self::new(pos, self.expression_type(), arg))
            })
            .cast(|p| p as *mut dyn Expression),
        )
    }
    fn supportsConstantValues(&self) -> bool {
        self.constructor_supportsConstantValues()
    }
    fn getConstantValue(&self, n: i32) -> Option<f64> {
        self.constructor_getConstantValue(n)
    }
    fn compareConstant(&self, other: &dyn Expression) -> ComparisonResult {
        self.constructor_compareConstant(other)
    }
    fn description_with_precedence(&self, p: OperatorPrecedence) -> Vec<u8> {
        self.constructor_description(p)
    }
}
unsafe impl AnyConstructor for ConstructorCompound {
    fn argumentSpan(&self) -> &[Option<IRNodeBox<dyn Expression>>] {
        self.base.argumentSpan()
    }
    fn argumentSpan_mut(self: Pin<&mut Self>) -> &mut [Option<IRNodeBox<dyn Expression>>] {
        unsafe { self.get_unchecked_mut().base.argumentSpan_mut() }
    }
}
use super::SkSLType::{
    checkForOutOfRangeLiteralExpr, checkForOutOfRangeLiteralValue, isAllowedInES2Context,
};
use super::{SkSLConstructorSplat::ConstructorSplat, SkSLLiteral::Literal};
use crate::src::sksl::SkSLConstantFolder::ConstantFolder;
// Source: src/sksl/ir/SkSLConstructorCompound.cpp:28-45
fn is_safe_to_eliminate(ty: &dyn Type, arg: &dyn Expression) -> bool {
    if ty.isScalar() {
        debug_assert!(arg.expression_type().matches(ty));
        return true;
    }
    if ty.isVector() && arg.expression_type().matches(ty) {
        return true;
    }
    false
}
// Source: src/sksl/ir/SkSLConstructorCompound.cpp:47-76
unsafe fn make_splat_from_arguments<'a>(
    ty: &dyn Type,
    args: &'a ExpressionArray,
) -> Option<&'a dyn Expression> {
    if ty.isMatrix() {
        return None;
    }
    let mut splatExpression: Option<&dyn Expression> = None;
    for (index, arg) in args.as_base().as_slice().iter().enumerate() {
        let arg = arg.as_ref().unwrap_unchecked();
        let expr: &dyn Expression;
        if arg.expression_type().isScalar() {
            expr = &**arg;
        } else if arg.node().is::<ConstructorSplat>() {
            expr = &**arg
                .node()
                .as_node::<ConstructorSplat>()
                .argument()
                .as_ref()
                .unwrap_unchecked();
        } else {
            return None;
        }
        if index == 0 {
            splatExpression = Some(expr);
            continue;
        }
        // Real out-of-scope implementation, not a literal-only comparison substitute.
        if !crate::src::sksl::SkSLAnalysis::IsSameExpressionTree(
            expr,
            splatExpression.unwrap_unchecked(),
        ) {
            return None;
        }
    }
    splatExpression
}
impl ConstructorCompound {
    // Source: src/sksl/ir/SkSLConstructorCompound.cpp:78-157
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        ty: &dyn Type,
        mut args: Pin<Box<ExpressionArray>>,
    ) -> IRNodeBox<dyn Expression> {
        debug_assert!(isAllowedInES2Context(ty, context));
        debug_assert!(args.as_base().as_slice().iter().all(|arg| {
            let argType = arg.as_ref().unwrap_unchecked().expression_type();
            (argType.isScalar() || argType.isVector() || argType.isMatrix())
                && argType.componentType().matches(ty.componentType())
        }));
        debug_assert_eq!(
            ty.slotCount(),
            args.as_base()
                .as_slice()
                .iter()
                .fold(0usize, |n, arg| n + arg
                    .as_ref()
                    .unwrap_unchecked()
                    .expression_type()
                    .slotCount())
        );
        if args.as_base().size() == 1
            && is_safe_to_eliminate(
                ty,
                &**args.as_base().as_slice()[0].as_ref().unwrap_unchecked(),
            )
        {
            let mut arg = args.as_mut().as_base_mut().as_mut_slice()[0]
                .take()
                .unwrap_unchecked();
            arg.as_pin_mut().node_mut().setPosition(pos);
            return arg;
        }
        debug_assert!(ty.isVector() || ty.isMatrix());
        if context
            .fConfig
            .unwrap_unchecked()
            .as_ref()
            .fSettings
            .fOptimize
        {
            let mut fields = 0;
            for arg in args.as_base().as_slice() {
                let arg = arg.as_ref().unwrap_unchecked();
                fields += if arg.node().is::<Self>() {
                    arg.node().as_node::<Self>().arguments().as_base().size()
                } else {
                    1
                };
            }
            if fields > args.as_base().size() {
                let mut flattened = ExpressionArray::new();
                flattened.as_mut().as_base_mut().reserve_exact(fields);
                for arg in args.as_mut().as_base_mut().as_mut_slice() {
                    if !arg.as_ref().unwrap_unchecked().node().is::<Self>() {
                        flattened.as_mut().as_base_mut().push_back(arg.take());
                        continue;
                    }
                    let compositeCtor =
                        &mut *(arg.as_ref().unwrap_unchecked().as_ptr() as *mut () as *mut Self);
                    for inner in compositeCtor.arguments_mut().as_base_mut().as_mut_slice() {
                        flattened.as_mut().as_base_mut().push_back(inner.take());
                    }
                }
                args = flattened;
            }
        }
        for arg in args.as_mut().as_base_mut().as_mut_slice() {
            *arg = Some(ConstantFolder::MakeConstantValueForVariable(
                pos,
                arg.take().unwrap_unchecked(),
            ));
        }
        if context
            .fConfig
            .unwrap_unchecked()
            .as_ref()
            .fSettings
            .fOptimize
        {
            if let Some(splat) = make_splat_from_arguments(ty, &args) {
                return ConstructorSplat::Make(
                    context,
                    pos,
                    ty,
                    splat.clone_current().unwrap_unchecked(),
                );
            }
        }
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, ty, args)))
            .cast(|p| p as *mut dyn Expression)
    }
    // Source: src/sksl/ir/SkSLConstructorCompound.cpp:159-170
    pub unsafe fn MakeFromConstants(
        context: &Context,
        pos: Position,
        returnType: &dyn Type,
        value: &[f64],
    ) -> IRNodeBox<dyn Expression> {
        let numSlots = returnType.slotCount() as i32;
        let mut array = ExpressionArray::new();
        array.as_mut().as_base_mut().reserve_exact(numSlots);
        for index in 0..numSlots {
            array.as_mut().as_base_mut().push_back(Some(
                Literal::Make(pos, value[index as usize], returnType.componentType())
                    .cast(|p| p as *mut dyn Expression),
            ));
        }
        Self::Make(context, pos, returnType, array)
    }
}
