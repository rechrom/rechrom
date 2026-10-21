use crate::include::core::{SkBlender::SkBlender, SkRefCnt::sk_sp};
use crate::src::core::{
    SkBlendMode::SkBlendMode,
    SkData::SkData,
    SkKnownRuntimeEffects::{GetKnownRuntimeEffect, StableKey},
};

/// Source: include/effects/SkBlenders.h:16-27
pub struct SkBlenders;

impl SkBlenders {
    /// Source: src/effects/SkBlenders.cpp:20-56
    pub fn Arithmetic(
        k1: f32,
        k2: f32,
        k3: f32,
        k4: f32,
        enforce_premul: bool,
    ) -> sk_sp<dyn SkBlender> {
        if !crate::src::core::SkFloatingPoint::SkIsFinite(&[k1, k2, k3, k4]) {
            return Default::default();
        }

        use crate::include::core::SkScalar::{SK_ScalarNearlyZero, SkScalarNearlyEqual};
        for (coefficients, mode) in [
            ([0.0, 1.0, 0.0, 0.0], SkBlendMode::kSrc),
            ([0.0, 0.0, 1.0, 0.0], SkBlendMode::kDst),
            ([0.0, 0.0, 0.0, 0.0], SkBlendMode::kClear),
        ] {
            if [k1, k2, k3, k4]
                .into_iter()
                .zip(coefficients)
                .all(|(actual, expected)| {
                    SkScalarNearlyEqual(actual, expected, SK_ScalarNearlyZero)
                })
            {
                return <dyn SkBlender>::Mode(mode);
            }
        }

        let effect = GetKnownRuntimeEffect(StableKey::kArithmetic)
            .expect("native arithmetic runtime effect");
        let uniforms = [k1, k2, k3, k4, if enforce_premul { 0.0 } else { 1.0 }];
        let data = unsafe {
            SkData::MakeWithCopy(
                uniforms.as_ptr().cast::<u8>(),
                core::mem::size_of_val(&uniforms),
            )
        };
        effect.makeBlender(Some(data), &[])
    }
}

/// Source: src/effects/SkBlenders.cpp:20-56
pub fn Arithmetic(
    k1: f32,
    k2: f32,
    k3: f32,
    k4: f32,
    enforce_premul: bool,
) -> sk_sp<dyn SkBlender> {
    SkBlenders::Arithmetic(k1, k2, k3, k4, enforce_premul)
}
