use crate::src::sksl::{
    ir::{
        SkSLConstructorSplat::ConstructorSplat,
        SkSLExpression::ExpressionExt,
        SkSLFunctionDefinition::FunctionDefinition,
        SkSLIRNode::*,
        SkSLLiteral::Literal,
        SkSLSwizzle::{ComponentArray, Swizzle, SwizzleComponent},
    },
    transform::SkSLProgramWriter::{self, ProgramWriter, UniquePtrExpression},
    SkSLContext::Context,
    SkSLModule::Module,
};
// Source: src/sksl/transform/SkSLReplaceSplatCastsWithSwizzles.cpp:27-28
pub unsafe fn ReplaceSplatCastsWithSwizzles(context: &Context, module: &mut Module) {
    // Source: src/sksl/transform/SkSLReplaceSplatCastsWithSwizzles.cpp:53-53
    struct SwizzleWriter<'a> {
        fContext: &'a Context,
    }
    impl<'a> SwizzleWriter<'a> {
        // Source: src/sksl/transform/SkSLReplaceSplatCastsWithSwizzles.cpp:30-30
        fn new(context: &'a Context) -> Self {
            Self { fContext: context }
        }
    }
    unsafe impl ProgramWriter for SwizzleWriter<'_> {
        // Source: src/sksl/transform/SkSLReplaceSplatCastsWithSwizzles.cpp:32-51
        unsafe fn visitExpressionPtr(&mut self, expr: &mut UniquePtrExpression) -> bool {
            if SkSLProgramWriter::inheritedExpression(
                self,
                expr.as_mut().unwrap_unchecked().as_pin_mut(),
            ) {
                return true;
            }
            if expr
                .as_ref()
                .unwrap_unchecked()
                .node()
                .is::<ConstructorSplat>()
            {
                let columns = expr.as_ref().unwrap_unchecked().expression_type().columns();
                let pos = expr.as_ref().unwrap_unchecked().node().fPosition;
                let splat = expr
                    .as_mut()
                    .unwrap_unchecked()
                    .as_pin_mut()
                    .node_mut()
                    .as_node_mut::<ConstructorSplat>()
                    .get_unchecked_mut();
                let arg = splat.argument_mut();
                let value = arg.as_ref().unwrap_unchecked();
                if !value.node().is::<Literal>()
                    || (value.expression_type().isFloat()
                        && value.expression_type().highPrecision())
                {
                    let mut components = ComponentArray::new();
                    for _ in 0..columns {
                        components.push_back(SwizzleComponent::X);
                    }
                    *expr = Some(Swizzle::MakeExact(
                        self.fContext,
                        pos,
                        arg.take().unwrap_unchecked(),
                        components,
                    ));
                }
            }
            false
        }
    }
    // Source: src/sksl/transform/SkSLReplaceSplatCastsWithSwizzles.cpp:58-63
    for pe in module.fElements.iter_mut() {
        let pe = pe.as_mut().unwrap_unchecked();
        if pe.node().is::<FunctionDefinition>() {
            let mut writer = SwizzleWriter::new(context);
            let def = pe
                .as_pin_mut()
                .node_mut()
                .as_node_mut::<FunctionDefinition>()
                .get_unchecked_mut();
            writer.visitStatementPtr(def.body_mut());
        }
    }
}
