// Copyright 2026 The Chromium Authors. BSD-style license.
//! Source-aligned keyframed float/transform curves used by cc animations.
//!
//! Source: ui/gfx/animation/keyframe/keyframed_animation_curve.{h,cc}

use crate::{
    cubic_bezier::CubicBezier,
    gfx_keyframe_model::{AnimationCurve, LimitDirection},
};
use scroll_animation::TimeDelta;

#[derive(Clone, Debug)]
pub enum TimingFunction {
    Linear,
    CubicBezier(CubicBezier),
    Steps { count: u32, jump_at_start: bool },
}

impl TimingFunction {
    pub fn Transform(&self, progress: f64) -> f64 {
        let progress = progress.clamp(0.0, 1.0);
        match self {
            Self::Linear => progress,
            Self::CubicBezier(curve) => curve.Solve(progress),
            Self::Steps {
                count,
                jump_at_start,
            } => {
                let count = (*count).max(1) as f64;
                let step = (progress * count).floor() + u8::from(*jump_at_start) as f64;
                (step / count).clamp(0.0, 1.0)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TransformOperation {
    Translate { x: f64, y: f64, z: f64 },
    Scale { x: f64, y: f64, z: f64 },
    RotateZ { degrees: f64 },
}

pub type TransformOperations = Vec<TransformOperation>;

fn interpolate_operation(
    from: &TransformOperation,
    to: &TransformOperation,
    progress: f64,
) -> Option<TransformOperation> {
    let lerp = |a: f64, b: f64| a + (b - a) * progress;
    Some(match (from, to) {
        (
            TransformOperation::Translate {
                x: ax,
                y: ay,
                z: az,
            },
            TransformOperation::Translate {
                x: bx,
                y: by,
                z: bz,
            },
        ) => TransformOperation::Translate {
            x: lerp(*ax, *bx),
            y: lerp(*ay, *by),
            z: lerp(*az, *bz),
        },
        (
            TransformOperation::Scale {
                x: ax,
                y: ay,
                z: az,
            },
            TransformOperation::Scale {
                x: bx,
                y: by,
                z: bz,
            },
        ) => TransformOperation::Scale {
            x: lerp(*ax, *bx),
            y: lerp(*ay, *by),
            z: lerp(*az, *bz),
        },
        (
            TransformOperation::RotateZ { degrees: a },
            TransformOperation::RotateZ { degrees: b },
        ) => TransformOperation::RotateZ {
            degrees: lerp(*a, *b),
        },
        _ => return None,
    })
}

fn identity_operation_like(operation: &TransformOperation) -> TransformOperation {
    match operation {
        TransformOperation::Translate { .. } => TransformOperation::Translate {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        TransformOperation::Scale { .. } => TransformOperation::Scale {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        TransformOperation::RotateZ { .. } => TransformOperation::RotateZ { degrees: 0.0 },
    }
}

/// Whether the operation lists can use Chromium's per-operation interpolation.
///
/// A matching shorter list is padded with identity operations of the types in
/// the longer list. A type mismatch inside the shared prefix requires the
/// matrix-decomposition fallback and is deliberately not claimed here.
pub fn CanInterpolateTransformOperations(
    from: &[TransformOperation],
    to: &[TransformOperation],
) -> bool {
    from.iter()
        .zip(to)
        .all(|(from, to)| std::mem::discriminant(from) == std::mem::discriminant(to))
}

pub fn InterpolateTransformOperations(
    from: &[TransformOperation],
    to: &[TransformOperation],
    progress: f64,
) -> Option<TransformOperations> {
    if !CanInterpolateTransformOperations(from, to) {
        return None;
    }
    let mut result = Vec::with_capacity(from.len().max(to.len()));
    for index in 0..from.len().max(to.len()) {
        let operation = match (from.get(index), to.get(index)) {
            (Some(from), Some(to)) => interpolate_operation(from, to, progress)?,
            (Some(from), None) => {
                interpolate_operation(from, &identity_operation_like(from), progress)?
            }
            (None, Some(to)) => interpolate_operation(&identity_operation_like(to), to, progress)?,
            (None, None) => unreachable!("operation index is within the longest list"),
        };
        result.push(operation);
    }
    Some(result)
}

/// Column-major transform produced by applying the operation list in CSS order.
pub fn TransformOperationsToMatrix(operations: &[TransformOperation]) -> [f64; 16] {
    fn multiply(left: [f64; 16], right: [f64; 16]) -> [f64; 16] {
        let mut result = [0.0; 16];
        for column in 0..4 {
            for row in 0..4 {
                for component in 0..4 {
                    result[column * 4 + row] +=
                        left[component * 4 + row] * right[column * 4 + component];
                }
            }
        }
        result
    }
    let mut result = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    for operation in operations {
        let matrix = match operation {
            TransformOperation::Translate { x, y, z } => [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, *x, *y, *z, 1.0,
            ],
            TransformOperation::Scale { x, y, z } => [
                *x, 0.0, 0.0, 0.0, 0.0, *y, 0.0, 0.0, 0.0, 0.0, *z, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
            TransformOperation::RotateZ { degrees } => {
                let (sin, cos) = degrees.to_radians().sin_cos();
                [
                    cos, sin, 0.0, 0.0, -sin, cos, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
                ]
            }
        };
        result = multiply(result, matrix);
    }
    result
}

#[derive(Clone, Debug)]
struct Keyframe<V> {
    time: TimeDelta,
    value: V,
    easing: TimingFunction,
}

fn interval<V>(keyframes: &[Keyframe<V>], time: TimeDelta) -> (usize, usize, f64) {
    debug_assert!(!keyframes.is_empty());
    if keyframes.len() == 1 || time <= keyframes[0].time {
        return (0, 0, 0.0);
    }
    let last = keyframes.len() - 1;
    if time >= keyframes[last].time {
        return (last, last, 1.0);
    }
    let upper = keyframes
        .iter()
        .position(|keyframe| keyframe.time >= time)
        .unwrap_or(last);
    let lower = upper - 1;
    let duration = keyframes[upper].time - keyframes[lower].time;
    let progress = if duration.is_zero() {
        1.0
    } else {
        (time - keyframes[lower].time) / duration
    };
    (lower, upper, progress)
}

#[derive(Clone, Debug, Default)]
pub struct KeyframedFloatAnimationCurve {
    keyframes: Vec<Keyframe<f32>>,
}

impl KeyframedFloatAnimationCurve {
    pub fn AddKeyframe(&mut self, time: TimeDelta, value: f32, easing: TimingFunction) {
        self.keyframes.push(Keyframe {
            time,
            value,
            easing,
        });
        self.keyframes.sort_by_key(|keyframe| keyframe.time);
    }
    pub fn GetValue(&self, time: TimeDelta, _limit: LimitDirection) -> f32 {
        let (lower, upper, progress) = interval(&self.keyframes, time);
        if lower == upper {
            return self.keyframes[lower].value;
        }
        let progress = self.keyframes[lower].easing.Transform(progress) as f32;
        let from = self.keyframes[lower].value;
        from + (self.keyframes[upper].value - from) * progress
    }
}

impl AnimationCurve for KeyframedFloatAnimationCurve {
    fn Duration(&self) -> TimeDelta {
        self.keyframes
            .last()
            .map_or_default(|keyframe| keyframe.time)
    }
}

#[derive(Clone, Debug, Default)]
pub struct KeyframedTransformAnimationCurve {
    keyframes: Vec<Keyframe<TransformOperations>>,
}

impl KeyframedTransformAnimationCurve {
    pub fn AddKeyframe(
        &mut self,
        time: TimeDelta,
        value: TransformOperations,
        easing: TimingFunction,
    ) {
        self.keyframes.push(Keyframe {
            time,
            value,
            easing,
        });
        self.keyframes.sort_by_key(|keyframe| keyframe.time);
    }
    pub fn GetValue(&self, time: TimeDelta, _limit: LimitDirection) -> TransformOperations {
        let (lower, upper, progress) = interval(&self.keyframes, time);
        if lower == upper {
            return self.keyframes[lower].value.clone();
        }
        let progress = self.keyframes[lower].easing.Transform(progress);
        InterpolateTransformOperations(
            &self.keyframes[lower].value,
            &self.keyframes[upper].value,
            progress,
        )
        .expect("compositor transform keyframes retain compatible operations")
    }
    pub fn PreservesAxisAlignment(&self) -> bool {
        self.keyframes.iter().all(|keyframe| {
            keyframe.value.iter().all(|operation| {
                !matches!(operation, TransformOperation::RotateZ { degrees } if degrees.rem_euclid(90.0) != 0.0)
            })
        })
    }
}

impl AnimationCurve for KeyframedTransformAnimationCurve {
    fn Duration(&self) -> TimeDelta {
        self.keyframes
            .last()
            .map_or_default(|keyframe| keyframe.time)
    }
}
