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
use super::SkSLLiteral::Literal;
use crate::src::sksl::{
    SkSLErrorReporter::ErrorReporterExt, SkSLProgramSettings::ProgramConfig, SkSLUtil::ShaderCaps,
};
// Source: src/sksl/ir/SkSLSetting.h:38-38
// Adapted native bool member pointer: byte offset into the actual ShaderCaps object.
// Safety: offset must designate a live bool field in that exact non-inherited native layout.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CapsPtr {
    offset: usize,
}
impl CapsPtr {
    pub unsafe fn from_member_offset(offset: usize) -> Self {
        Self { offset }
    }
    unsafe fn read(self, caps: &ShaderCaps) -> bool {
        *(caps as *const ShaderCaps as *const u8)
            .add(self.offset)
            .cast::<bool>()
    }
}
// Source: src/sksl/ir/SkSLSetting.h:34-34
// Source: src/sksl/ir/SkSLSetting.h:71-71
#[repr(C)]
pub struct Setting {
    base: ExpressionData,
    fCapsPtr: CapsPtr,
}
// Source: src/sksl/ir/SkSLSetting.h:36-36
impl IRNodeType for Setting {
    const kIRNodeKind: i32 = ExpressionKind::kSetting as i32;
}
impl Setting {
    // Source: src/sksl/ir/SkSLSetting.h:40-42
    pub unsafe fn new(pos: Position, capsPtr: CapsPtr, ty: NonNull<dyn Type>) -> Self {
        Self {
            base: ExpressionData::new(pos, ExpressionKind::kSetting, Some(ty)),
            fCapsPtr: capsPtr,
        }
    }
}
impl IRNode for Setting {
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
unsafe impl Expression for Setting {
    fn expression_data(&self) -> &ExpressionData {
        &self.base
    }
    fn expression_data_mut(self: Pin<&mut Self>) -> Pin<&mut ExpressionData> {
        unsafe { self.map_unchecked_mut(|s| &mut s.base) }
    }
    fn as_expression(&self) -> &dyn Expression {
        self
    }
    // Source: src/sksl/ir/SkSLSetting.h:58-60
    unsafe fn clone_at(&self, pos: Position) -> Option<IRNodeBox<dyn Expression>> {
        Some(
            IRNodeBox::new_with(|p: *mut Self| {
                p.write(Self::new(
                    pos,
                    self.fCapsPtr,
                    borrow_type(self.expression_type()),
                ))
            })
            .cast(|p| p as *mut dyn Expression),
        )
    }
    fn description_with_precedence(&self, p: OperatorPrecedence) -> Vec<u8> {
        description(self, p)
    }
}
impl Setting {
    // Source: src/sksl/ir/SkSLSetting.h:64-64
    pub fn capsPtr(&self) -> CapsPtr {
        self.fCapsPtr
    }
}
// Source: src/sksl/ir/SkSLSetting.h:66-68
fn description(this: &Setting, _: OperatorPrecedence) -> Vec<u8> {
    unsafe {
        let mut out = b"sk_Caps.".to_vec();
        out.extend_from_slice(this.name());
        out
    }
}
// Source: src/sksl/ir/SkSLSetting.cpp:27-49
// Adapted static immutable lookup retains native string/member identity. Linear search
// changes only lookup container, not the bool-field lease. Actual ShaderCaps and generation enum are translated in v54.
fn caps_lookup_table() -> &'static [(&'static [u8], CapsPtr)] {
    static TABLE: [(&[u8], CapsPtr); 8] = [
        (
            b"mustDoOpBetweenFloorAndAbs",
            CapsPtr {
                offset: core::mem::offset_of!(ShaderCaps, fMustDoOpBetweenFloorAndAbs),
            },
        ),
        (
            b"mustGuardDivisionEvenAfterExplicitZeroCheck",
            CapsPtr {
                offset: core::mem::offset_of!(
                    ShaderCaps,
                    fMustGuardDivisionEvenAfterExplicitZeroCheck
                ),
            },
        ),
        (
            b"atan2ImplementedAsAtanYOverX",
            CapsPtr {
                offset: core::mem::offset_of!(ShaderCaps, fAtan2ImplementedAsAtanYOverX),
            },
        ),
        (
            b"floatIs32Bits",
            CapsPtr {
                offset: core::mem::offset_of!(ShaderCaps, fFloatIs32Bits),
            },
        ),
        (
            b"integerSupport",
            CapsPtr {
                offset: core::mem::offset_of!(ShaderCaps, fIntegerSupport),
            },
        ),
        (
            b"builtinDeterminantSupport",
            CapsPtr {
                offset: core::mem::offset_of!(ShaderCaps, fBuiltinDeterminantSupport),
            },
        ),
        (
            b"rewriteMatrixVectorMultiply",
            CapsPtr {
                offset: core::mem::offset_of!(ShaderCaps, fRewriteMatrixVectorMultiply),
            },
        ),
        (
            b"PerlinNoiseRoundingFix",
            CapsPtr {
                offset: core::mem::offset_of!(ShaderCaps, fPerlinNoiseRoundingFix),
            },
        ),
    ];
    &TABLE
}
impl Setting {
    // Source: src/sksl/ir/SkSLSetting.cpp:53-60
    pub unsafe fn name(&self) -> &'static [u8] {
        for &(name, capsPtr) in caps_lookup_table() {
            if capsPtr == self.fCapsPtr {
                return name;
            }
        }
        core::hint::unreachable_unchecked()
    }
    // Source: src/sksl/ir/SkSLSetting.cpp:62-79
    pub unsafe fn Convert(
        context: &Context,
        pos: Position,
        name: &[u8],
    ) -> Option<IRNodeBox<dyn Expression>> {
        let config = context.fConfig;
        debug_assert!(config.is_some());
        if !ProgramConfig::AllowsPrivateIdentifiers(config.unwrap_unchecked().as_ref().fKind) {
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, b"name 'sk_Caps' is reserved");
            return None;
        }
        let capsPtr = caps_lookup_table()
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| *value);
        let Some(capsPtr) = capsPtr else {
            let mut msg = b"unknown capability flag '".to_vec();
            msg.extend_from_slice(name);
            msg.push(b'\'');
            let mut errors = context.fErrors;
            errors.as_mut().error(pos, &msg);
            return None;
        };
        Some(Self::Make(context, pos, capsPtr))
    }
    // Source: src/sksl/ir/SkSLSetting.cpp:81-85
    pub unsafe fn Make(
        context: &Context,
        pos: Position,
        capsPtr: CapsPtr,
    ) -> IRNodeBox<dyn Expression> {
        debug_assert!(ProgramConfig::AllowsPrivateIdentifiers(
            context.fConfig.unwrap_unchecked().as_ref().fKind
        ));
        let ty = borrow_type(&**context.fTypes.as_ref().fBool);
        IRNodeBox::new_with(|p: *mut Self| p.write(Self::new(pos, capsPtr, ty)))
            .cast(|p| p as *mut dyn Expression)
    }
    // Source: src/sksl/ir/SkSLSetting.cpp:87-89
    pub unsafe fn toLiteral(&self, caps: &ShaderCaps) -> IRNodeBox<dyn Expression> {
        Literal::MakeBool(
            self.node().fPosition,
            self.fCapsPtr.read(caps),
            self.expression_type(),
        )
        .cast(|p| p as *mut dyn Expression)
    }
}
