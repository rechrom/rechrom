//! cc PathEffect values. Native Skia creation remains a typed pure-value
//! operation and cannot replace the source Dash/Corner ownership or codec.
#![allow(non_snake_case, non_camel_case_types)]
use std::sync::Arc;
// Source: cc/paint/path_effect.h:38-46.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    kNull,
    kDash,
    kCorner,
}
// Source: cc/paint/path_effect.cc:30-63,65-87; .h:17-58.
#[derive(Debug)]
pub enum PathEffect {
    Dash { intervals: Vec<f32>, phase: f32 },
    Corner { radius: f32 },
}
pub trait PathEffectBackend {
    type NativeEffect;
    fn MakeDash(&self, intervals: &[f32], phase: f32) -> Option<Self::NativeEffect>;
    fn MakeCorner(&self, radius: f32) -> Option<Self::NativeEffect>;
}
pub trait PathEffectWriter {
    fn SerializedSizeScalar() -> usize;
    fn SerializedSizeOfElements(count: usize) -> usize;
    fn WriteIntervals(&mut self, intervals: &[f32]);
    fn WriteScalar(&mut self, value: f32);
}
pub trait PathEffectReader {
    fn ReadIntervals(&mut self) -> Vec<f32>;
    fn ReadScalar(&mut self) -> f32;
    fn valid(&self) -> bool;
}
impl PathEffect {
    // Source: cc/paint/path_effect.cc:91-104.
    pub fn MakeDash(intervals: &[f32], count: i32, phase: f32) -> Arc<Self> {
        let count = usize::try_from(count).expect("native nonnegative interval count");
        Arc::new(Self::Dash {
            intervals: intervals[..count].to_vec(),
            phase,
        })
    }
    pub fn MakeCorner(radius: f32) -> Arc<Self> {
        Arc::new(Self::Corner { radius })
    }
    pub fn kind(&self) -> Type {
        match self {
            Self::Dash { .. } => Type::kDash,
            Self::Corner { .. } => Type::kCorner,
        }
    }
    // Source: cc/paint/path_effect.cc:55-59,80-86,106-108.
    pub fn GetSkPathEffect<B: PathEffectBackend>(&self, backend: &B) -> Option<B::NativeEffect> {
        match self {
            Self::Dash { intervals, phase } => backend.MakeDash(intervals, *phase),
            Self::Corner { radius } => backend.MakeCorner(*radius),
        }
    }
    pub fn dash_interval_count(&self) -> usize {
        match self {
            Self::Dash { intervals, .. } => intervals.len(),
            Self::Corner { .. } => 0,
        }
    }
    // Source: cc/paint/path_effect.cc:37-39,70-72,110-125.
    pub fn EqualsForTesting(&self, other: &Self) -> bool {
        if self.kind() != other.kind() {
            return false;
        }
        match (self, other) {
            (
                Self::Dash {
                    intervals: a,
                    phase: p,
                },
                Self::Dash {
                    intervals: b,
                    phase: q,
                },
            ) => p == q && a == b,
            (Self::Corner { radius: a }, Self::Corner { radius: b }) => a == b,
            _ => unreachable!("source type-matched effect"),
        }
    }
    // Source: cc/paint/path_effect.cc:42-53,75-80.
    pub fn SerializedDataSize<W: PathEffectWriter>(&self) -> usize {
        match self {
            Self::Dash { intervals, .. } => W::SerializedSizeOfElements(intervals.len())
                .checked_add(W::SerializedSizeScalar())
                .expect("source CheckedNumeric ValueOrDie"),
            Self::Corner { .. } => W::SerializedSizeScalar(),
        }
    }
    pub fn SerializeData<W: PathEffectWriter>(&self, writer: &mut W) {
        match self {
            Self::Dash { intervals, phase } => {
                writer.WriteIntervals(intervals);
                writer.WriteScalar(*phase)
            }
            Self::Corner { radius } => writer.WriteScalar(*radius),
        }
    }
    // Source: cc/paint/path_effect.cc:127-144.
    pub fn Deserialize(reader: &mut impl PathEffectReader, kind: Type) -> Option<Arc<Self>> {
        match kind {
            Type::kDash => {
                let intervals = reader.ReadIntervals();
                let phase = reader.ReadScalar();
                reader.valid().then(|| {
                    Self::MakeDash(
                        &intervals,
                        i32::try_from(intervals.len()).expect("source checked_cast<int>"),
                        phase,
                    )
                })
            }
            Type::kCorner => {
                let radius = reader.ReadScalar();
                reader.valid().then(|| Self::MakeCorner(radius))
            }
            Type::kNull => unreachable!("source NOTREACHED null path effect"),
        }
    }
}
impl super::paint_flags::PaintValueEquals for PathEffect {
    fn EqualsForTesting(&self, other: &Self) -> bool {
        Self::EqualsForTesting(self, other)
    }
}
// Source scaffolding: cc/paint/path_effect.h:1-16,60-61; .cc:1-29,89-90,146.
// Translation status: complete
