// New translation is UNVERIFIED by explicit user instruction.
use super::{
    SkSLExpression::{Expression, ExpressionData, ExpressionExt},
    SkSLIRNode::*,
    SkSLType::Type,
};
use crate::src::sksl::{
    SkSLContext::Context,
    SkSLOperator::{Operator, OperatorKind, OperatorPrecedence},
    SkSLPosition::Position,
};
use core::{pin::Pin, ptr::NonNull};
unsafe fn borrow_type(ty: &dyn Type) -> NonNull<dyn Type> {
    core::mem::transmute::<NonNull<dyn Type + '_>, NonNull<dyn Type>>(NonNull::from(ty))
}
// Source: src/sksl/ir/SkSLSwizzle.h:32-39
// Transparent signed8 values preserve native component identity and validation domain.
pub mod SwizzleComponent {
    pub type Type = i8;
    pub const X: i8 = 0;
    pub const Y: i8 = 1;
    pub const Z: i8 = 2;
    pub const W: i8 = 3;
    pub const R: i8 = 4;
    pub const G: i8 = 5;
    pub const B: i8 = 6;
    pub const A: i8 = 7;
    pub const S: i8 = 8;
    pub const T: i8 = 9;
    pub const P: i8 = 10;
    pub const Q: i8 = 11;
    pub const UL: i8 = 12;
    pub const UT: i8 = 13;
    pub const UR: i8 = 14;
    pub const UB: i8 = 15;
    pub const ZERO: i8 = 16;
    pub const ONE: i8 = 17;
}
// Source: src/sksl/ir/SkSLSwizzle.h:46-46
pub type ComponentArray = crate::src::core::SkFixedArray::FixedArray<4, i8>;
// Source: src/sksl/ir/SkSLSwizzle.h:51-51
// Source: src/sksl/ir/SkSLSwizzle.h:122-123
// Adapted inherited state and contiguous field group preserve original field order.
#[repr(C)]
pub struct Swizzle {
    base: ExpressionData,
    fBase: Option<IRNodeBox<dyn Expression>>,
    fComponents: ComponentArray,
}
// Source: src/sksl/ir/SkSLSwizzle.h:53-53
impl IRNodeType for Swizzle {
    const kIRNodeKind: i32 = ExpressionKind::kSwizzle as i32;
}
// Source: src/sksl/ir/SkSLSwizzle.h:55-55
pub type Component = SwizzleComponent::Type;
impl Swizzle {
    // Source: src/sksl/ir/SkSLSwizzle.h:57-64
    pub unsafe fn new(
        context: &Context,
        pos: Position,
        base: Option<IRNodeBox<dyn Expression>>,
        components: &ComponentArray,
    ) -> Self {
        let ty = borrow_type(
            base.as_ref()
                .unwrap_unchecked()
                .expression_type()
                .componentType()
                .toCompound(context, components.size(), 1),
        );
        let result = Self {
            base: ExpressionData::new(pos, ExpressionKind::kSwizzle, Some(ty)),
            fBase: base,
            fComponents: components.clone(),
        };
        debug_assert!(result.components().size() >= 1 && result.components().size() <= 4);
        result
    }
    // Source: src/sksl/ir/SkSLSwizzle.h:88-90
    pub unsafe fn base_mut(&mut self) -> &mut Option<IRNodeBox<dyn Expression>> {
        &mut self.fBase
    }
    // Source: src/sksl/ir/SkSLSwizzle.h:92-94
    pub fn base(&self) -> &Option<IRNodeBox<dyn Expression>> {
        &self.fBase
    }
    // Source: src/sksl/ir/SkSLSwizzle.h:96-98
    pub fn components(&self) -> &ComponentArray {
        &self.fComponents
    }
}
impl IRNode for Swizzle {
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
unsafe impl Expression for Swizzle {
    fn expression_data(&self) -> &ExpressionData {
        &self.base
    }
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base) }
    }
    fn as_expression(&self) -> &dyn Expression {
        self
    }
    // Source: src/sksl/ir/SkSLSwizzle.h:100-103
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        Some(
            IRNodeBox::new_with(|p: *mut Self| {
                p.write(Self::new_typed(
                    pos,
                    borrow_type(self.expression_type()),
                    self.base().as_ref().unwrap_unchecked().clone_current(),
                    self.components(),
                ))
            })
            .cast(|p| p as *mut dyn Expression),
        )
    }
    fn description_with_precedence(&self, p: OperatorPrecedence) -> Vec<u8> {
        description(self, p)
    }
}
impl Swizzle {
    // Source: src/sksl/ir/SkSLSwizzle.h:114-120
    unsafe fn new_typed(
        pos: Position,
        ty: NonNull<dyn Type>,
        base: Option<IRNodeBox<dyn Expression>>,
        components: &ComponentArray,
    ) -> Self {
        let result = Self {
            base: ExpressionData::new(pos, ExpressionKind::kSwizzle, Some(ty)),
            fBase: base,
            fComponents: components.clone(),
        };
        debug_assert!(result.components().size() >= 1 && result.components().size() <= 4);
        result
    }
}
use super::{
    SkSLConstructor::AnyConstructor, SkSLConstructorCompound::ConstructorCompound,
    SkSLConstructorCompoundCast::ConstructorCompoundCast,
    SkSLConstructorScalarCast::ConstructorScalarCast, SkSLConstructorSplat::ConstructorSplat,
    SkSLLiteral::Literal,
};
use crate::src::sksl::{
    SkSLConstantFolder::ConstantFolder, SkSLDefines::ExpressionArray,
    SkSLErrorReporter::ErrorReporterExt,
};
use SwizzleComponent::*;
// Source: src/sksl/ir/SkSLSwizzle.cpp:34-86
fn validate_swizzle_domain(fields: &ComponentArray) -> bool {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum SwizzleDomain {
        kCoordinate,
        kColor,
        kUV,
        kRectangle,
    }
    let mut domain = None;
    for &field in fields.as_slice() {
        let fieldDomain = match field {
            X | Y | Z | W => SwizzleDomain::kCoordinate,
            R | G | B | A => SwizzleDomain::kColor,
            S | T | P | Q => SwizzleDomain::kUV,
            UL | UT | UR | UB => SwizzleDomain::kRectangle,
            ZERO | ONE => continue,
            _ => return false,
        };
        if domain.is_none() {
            domain = Some(fieldDomain);
        } else if domain != Some(fieldDomain) {
            return false;
        }
    }
    true
}
// Source: src/sksl/ir/SkSLSwizzle.cpp:88-110
fn mask_char(component: i8) -> u8 {
    match component {
        X => b'x',
        Y => b'y',
        Z => b'z',
        W => b'w',
        R => b'r',
        G => b'g',
        B => b'b',
        A => b'a',
        S => b's',
        T => b't',
        P => b'p',
        Q => b'q',
        UL => b'L',
        UT => b'T',
        UR => b'R',
        UB => b'B',
        ZERO => b'0',
        ONE => b'1',
        _ => {
            debug_assert!(false, "invalid native swizzle component");
            unsafe { core::hint::unreachable_unchecked() }
        }
    }
}
impl Swizzle {
    // Source: src/sksl/ir/SkSLSwizzle.cpp:112-118
    pub fn MaskString(components: &ComponentArray) -> Vec<u8> {
        let mut result = Vec::new();
        for &component in components.as_slice() {
            result.push(mask_char(component));
        }
        result
    }
}
// Source: src/sksl/ir/SkSLSwizzle.cpp:120-244
unsafe fn optimize_constructor_swizzle(
    context: &Context,
    pos: Position,
    base: &ConstructorCompound,
    components: ComponentArray,
) -> Option<IRNodeBox<dyn Expression>> {
    let baseArguments = base.argumentSpan();
    let _replacement: Option<IRNodeBox<dyn Expression>> = None;
    let exprType = base.expression_type();
    let componentType = exprType.componentType();
    let swizzleSize = components.size();
    #[derive(Clone, Copy)]
    struct ConstructorArgMap {
        fArgIndex: i8,
        fComponent: i8,
    }
    let numConstructorArgs = base.expression_type().columns();
    let mut argMap = [ConstructorArgMap {
        fArgIndex: 0,
        fComponent: 0,
    }; 4];
    let mut writeIdx = 0;
    for (argIdx, arg) in baseArguments.iter().enumerate() {
        let arg = arg.as_ref().unwrap_unchecked();
        let argType = arg.expression_type();
        if !argType.isScalar() && !argType.isVector() {
            return None;
        }
        let argSlots = argType.slotCount();
        for componentIdx in 0..argSlots {
            argMap[writeIdx].fArgIndex = argIdx as i8;
            argMap[writeIdx].fComponent = componentIdx as i8;
            writeIdx += 1;
        }
    }
    debug_assert_eq!(writeIdx, numConstructorArgs as usize);
    let mut exprUsed = [0i8; 4];
    for &c in components.as_slice() {
        exprUsed[argMap[c as usize].fArgIndex as usize] += 1;
    }
    for index in 0..numConstructorArgs {
        let constructorArgIndex = argMap[index as usize].fArgIndex;
        let baseArg = baseArguments[constructorArgIndex as usize]
            .as_ref()
            .unwrap_unchecked();
        if exprUsed[constructorArgIndex as usize] > 1
            && !crate::src::sksl::SkSLAnalysis::IsTrivialExpression(&**baseArg)
        {
            return None;
        }
        if exprUsed[constructorArgIndex as usize] != 1
            && crate::src::sksl::SkSLAnalysis::HasSideEffects(&**baseArg)
        {
            return None;
        }
    }
    struct ReorderedArgument {
        fArgIndex: i8,
        fComponents: ComponentArray,
    }
    // Native user-defined FixedArray copy constructor is nontrivial; keep the canonical non-memmove STArray lane.
    let mut reorderedArgs = Box::pin(crate::include::private::SkTArray::STArray::<
        ReorderedArgument,
        4,
        false,
    >::new());
    for &c in components.as_slice() {
        let argument = &argMap[c as usize];
        let baseArg = baseArguments[argument.fArgIndex as usize]
            .as_ref()
            .unwrap_unchecked();
        if baseArg.expression_type().isScalar() {
            crate::SkASSERT!(@source b"src/sksl/ir/SkSLSwizzle.cpp\0".as_ptr(),202; @condition b"argument.fComponent == 0\0".as_ptr(); (argument.fComponent)==(0));
            reorderedArgs.as_mut().push_back(ReorderedArgument {
                fArgIndex: argument.fArgIndex,
                fComponents: ComponentArray::new(),
            });
        } else {
            crate::SkASSERT!(@source b"src/sksl/ir/SkSLSwizzle.cpp\0".as_ptr(),207; @condition b"baseArg.type().isVector()\0".as_ptr(); baseArg.expression_type().isVector());
            debug_assert!(argument.fComponent < baseArg.expression_type().columns() as i8);
            if reorderedArgs.as_slice().is_empty()
                || reorderedArgs.as_slice().last().unwrap_unchecked().fArgIndex
                    != argument.fArgIndex
            {
                reorderedArgs.as_mut().push_back(ReorderedArgument {
                    fArgIndex: argument.fArgIndex,
                    fComponents: ComponentArray::from_values(&[argument.fComponent]),
                });
            } else {
                let back = reorderedArgs
                    .as_mut()
                    .as_mut_slice()
                    .last_mut()
                    .unwrap_unchecked();
                debug_assert!(!back.fComponents.empty());
                back.fComponents.push_back(argument.fComponent);
            }
        }
    }
    let mut newArgs = ExpressionArray::new();
    newArgs.as_mut().as_base_mut().reserve_exact(swizzleSize);
    for reorderedArg in reorderedArgs.as_slice() {
        let newArg = baseArguments[reorderedArg.fArgIndex as usize]
            .as_ref()
            .unwrap_unchecked()
            .clone_current()
            .unwrap_unchecked();
        if reorderedArg.fComponents.empty() {
            newArgs.as_mut().as_base_mut().push_back(Some(newArg));
        } else {
            newArgs.as_mut().as_base_mut().push_back(Some(Swizzle::Make(
                context,
                pos,
                newArg,
                reorderedArg.fComponents.clone(),
            )));
        }
    }
    Some(ConstructorCompound::Make(
        context,
        pos,
        componentType.toCompound(context, swizzleSize, 1),
        newArgs,
    ))
}
impl Swizzle {
    // Source: src/sksl/ir/SkSLSwizzle.cpp:246-439
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        maskPos: Position,
        base: IRNodeBox<dyn Expression>,
        componentString: &[u8],
    ) -> Option<IRNodeBox<dyn Expression>> {
        if componentString.len() > 4 {
            let mut errors = context.fErrors;
            errors.as_mut().error(
                Position::Range(maskPos.startOffset() + 4, maskPos.endOffset()),
                b"too many components in swizzle mask",
            );
            return None;
        }
        let mut components = ComponentArray::new();
        for (i, &field) in componentString.iter().enumerate() {
            let component = match field {
                b'0' => ZERO,
                b'1' => ONE,
                b'x' => X,
                b'r' => R,
                b's' => S,
                b'L' => UL,
                b'y' => Y,
                b'g' => G,
                b't' => T,
                b'T' => UT,
                b'z' => Z,
                b'b' => B,
                b'p' => P,
                b'R' => UR,
                b'w' => W,
                b'a' => A,
                b'q' => Q,
                b'B' => UB,
                _ => {
                    let mut message = b"invalid swizzle component '".to_vec();
                    message.push(field);
                    message.push(b'\'');
                    let mut errors = context.fErrors;
                    errors.as_mut().error(
                        Position::Range(
                            maskPos.startOffset() + i as i32,
                            maskPos.startOffset() + i as i32 + 1,
                        ),
                        &message,
                    );
                    return None;
                }
            };
            components.push_back(component);
        }
        if !validate_swizzle_domain(&components) {
            let mut message = b"invalid swizzle mask '".to_vec();
            message.extend_from_slice(&Self::MaskString(&components));
            message.push(b'\'');
            let mut errors = context.fErrors;
            errors.as_mut().error(maskPos, &message);
            return None;
        }
        let baseType = base.expression_type().scalarTypeForLiteral();
        if !baseType.isVector() && !baseType.isScalar() {
            let mut message = b"cannot swizzle value of type '".to_vec();
            message.extend_from_slice(&baseType.displayName());
            message.push(b'\'');
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &message);
            return None;
        }
        let mut maskComponents = ComponentArray::new();
        let mut foundXYZW = false;
        for i in 0..components.size() {
            let component = components[i as usize];
            match component {
                ZERO | ONE => continue,
                X | R | S | UL => {
                    foundXYZW = true;
                    maskComponents.push_back(X);
                    continue;
                }
                Y | G | T | UT => {
                    foundXYZW = true;
                    if baseType.columns() >= 2 {
                        maskComponents.push_back(Y);
                        continue;
                    }
                    foundXYZW = true;
                    if baseType.columns() >= 3 {
                        maskComponents.push_back(Z);
                        continue;
                    }
                    foundXYZW = true;
                    if baseType.columns() >= 4 {
                        maskComponents.push_back(W);
                        continue;
                    }
                }
                Z | B | P | UR => {
                    foundXYZW = true;
                    if baseType.columns() >= 3 {
                        maskComponents.push_back(Z);
                        continue;
                    }
                    foundXYZW = true;
                    if baseType.columns() >= 4 {
                        maskComponents.push_back(W);
                        continue;
                    }
                }
                W | A | Q | UB => {
                    foundXYZW = true;
                    if baseType.columns() >= 4 {
                        maskComponents.push_back(W);
                        continue;
                    }
                }
                _ => {}
            }
            {
                let mut message = b"invalid swizzle component '".to_vec();
                message.push(mask_char(component));
                message.push(b'\'');
                let mut errors = context.fErrors;
                errors.as_mut().error(
                    Position::Range(maskPos.startOffset() + i, maskPos.startOffset() + i + 1),
                    &message,
                );
                return None;
            }
        }
        if !foundXYZW {
            let mut errors = context.fErrors;
            errors
                .as_mut()
                .error(maskPos, b"swizzle must refer to base expression");
            return None;
        }
        let baseType = borrow_type(baseType);
        let base = super::SkSLType::coerceExpression(baseType.as_ref(), Some(base), context)?;
        let mut expr = Self::Make(context, pos, base, maskComponents.clone());
        if maskComponents.size() == components.size() {
            return Some(expr);
        }
        let mut constructorArgs = ExpressionArray::new();
        constructorArgs.as_mut().as_base_mut().reserve_exact(3);
        constructorArgs.as_mut().as_base_mut().push_back(Some(expr));
        let scalarType = baseType.as_ref().componentType();
        let mut swizzleComponents = ComponentArray::new();
        let mut maskFieldIdx = 0;
        let mut constantFieldIdx = maskComponents.size();
        let mut constantZeroIdx = -1;
        let mut constantOneIdx = -1;
        for i in 0..components.size() {
            match components[i as usize] {
                ZERO => {
                    if constantZeroIdx == -1 {
                        constructorArgs.as_mut().as_base_mut().push_back(Some(
                            Literal::Make(pos, 0.0, scalarType).cast(|p| p as *mut dyn Expression),
                        ));
                        constantZeroIdx = constantFieldIdx;
                        constantFieldIdx += 1;
                    }
                    swizzleComponents.push_back(constantZeroIdx as i8);
                }
                ONE => {
                    if constantOneIdx == -1 {
                        constructorArgs.as_mut().as_base_mut().push_back(Some(
                            Literal::Make(pos, 1.0, scalarType).cast(|p| p as *mut dyn Expression),
                        ));
                        constantOneIdx = constantFieldIdx;
                        constantFieldIdx += 1;
                    }
                    swizzleComponents.push_back(constantOneIdx as i8);
                }
                _ => {
                    swizzleComponents.push_back(maskFieldIdx as i8);
                    maskFieldIdx += 1;
                }
            }
        }
        expr = ConstructorCompound::Make(
            context,
            pos,
            scalarType.toCompound(context, constantFieldIdx, 1),
            constructorArgs,
        );
        Some(Self::Make(context, pos, expr, swizzleComponents))
    }
    // Source: src/sksl/ir/SkSLSwizzle.cpp:441-448
    pub fn IsIdentity(components: &ComponentArray) -> bool {
        for index in 0..components.size() {
            if components[index as usize] as i32 != index {
                return false;
            }
        }
        true
    }
    // Source: src/sksl/ir/SkSLSwizzle.cpp:450-531
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        mut expr: IRNodeBox<dyn Expression>,
        components: ComponentArray,
    ) -> IRNodeBox<dyn Expression> {
        let exprType = expr.expression_type();
        debug_assert!(exprType.isVector() || exprType.isScalar());
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLSwizzle.cpp\0".as_ptr(),457; @condition b"components.size() >= 1 && components.size() <= 4\0".as_ptr(); components.size()>=1&&components.size()<=4);
        debug_assert!(components.as_slice().iter().all(|&c| c >= X && c <= W));
        if exprType.isScalar() {
            let compound: &'static dyn Type =
                core::mem::transmute(exprType.toCompound(context, components.size(), 1));
            return ConstructorSplat::Make(context, pos, compound, expr);
        }
        if components.size() == exprType.columns() && Self::IsIdentity(&components) {
            expr.as_pin_mut().node_mut().setPosition(pos);
            return expr;
        }
        if expr.node().is::<Self>() {
            let base = &mut *(expr.as_ptr() as *mut () as *mut Self);
            let mut combined = ComponentArray::new();
            for &c in components.as_slice() {
                combined.push_back(base.components()[c as usize]);
            }
            let inner = base.base_mut().take().unwrap_unchecked();
            return Self::Make(context, pos, inner, combined);
        }
        let value = ConstantFolder::GetConstantValueForVariable(&*expr);
        if value.node().is::<ConstructorSplat>() {
            let splat = value.node().as_node::<ConstructorSplat>();
            return ConstructorSplat::Make(
                context,
                pos,
                splat
                    .expression_type()
                    .componentType()
                    .toCompound(context, components.size(), 1),
                splat
                    .argument()
                    .as_ref()
                    .unwrap_unchecked()
                    .clone_current()
                    .unwrap_unchecked(),
            );
        }
        if value.node().is::<ConstructorCompoundCast>() {
            let cast = value.node().as_node::<ConstructorCompoundCast>();
            let castType =
                cast.expression_type()
                    .componentType()
                    .toCompound(context, components.size(), 1);
            let swizzled = Self::Make(
                context,
                pos,
                cast.argument()
                    .as_ref()
                    .unwrap_unchecked()
                    .clone_current()
                    .unwrap_unchecked(),
                components,
            );
            return if castType.columns() > 1 {
                ConstructorCompoundCast::Make(context, pos, castType, swizzled)
            } else {
                ConstructorScalarCast::Make(context, pos, castType, swizzled)
            };
        }
        if value.node().is::<ConstructorCompound>() {
            let ctor = value.node().as_node::<ConstructorCompound>();
            if let Some(replacement) =
                optimize_constructor_swizzle(context, pos, ctor, components.clone())
            {
                return replacement;
            }
        }
        IRNodeBox::new_with(|p: *mut Self| {
            p.write(Self::new(context, pos, Some(expr), &components))
        })
        .cast(|p| p as *mut dyn Expression)
    }
    // Source: src/sksl/ir/SkSLSwizzle.cpp:533-548
    pub unsafe fn MakeExact(
        context: &Context,
        pos: Position,
        expr: IRNodeBox<dyn Expression>,
        components: ComponentArray,
    ) -> IRNodeBox<dyn Expression> {
        debug_assert!(expr.expression_type().isVector() || expr.expression_type().isScalar());
        crate::SkASSERT!(@source b"src/sksl/ir/SkSLSwizzle.cpp\0".as_ptr(),539; @condition b"components.size() >= 1 && components.size() <= 4\0".as_ptr(); components.size()>=1&&components.size()<=4);
        debug_assert!(components.as_slice().iter().all(|&c| c >= X && c <= W));
        IRNodeBox::new_with(|p: *mut Self| {
            p.write(Self::new(context, pos, Some(expr), &components))
        })
        .cast(|p| p as *mut dyn Expression)
    }
}
// Source: src/sksl/ir/SkSLSwizzle.cpp:550-553
fn description(this: &Swizzle, _p: OperatorPrecedence) -> Vec<u8> {
    unsafe {
        let mut result = this
            .base()
            .as_ref()
            .unwrap_unchecked()
            .description_with_precedence(OperatorPrecedence::kPostfix);
        result.push(b'.');
        result.extend_from_slice(&Swizzle::MaskString(this.components()));
        result
    }
}
