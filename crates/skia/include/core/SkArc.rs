use crate::src::core::SkRect::SkRect;
// Source: include/core/SkArc.h:16-19
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    kArc,
    kWedge,
}
// Source: include/core/SkArc.h:15-15
#[derive(Clone, Copy, Debug)]
pub struct SkArc {
    // Source: include/core/SkArc.h:55-55
    pub fOval: SkRect,
    // Source: include/core/SkArc.h:58-60
    pub fStartAngle: f32,
    pub fSweepAngle: f32,
    // Source: include/core/SkArc.h:62-62
    pub fType: Type,
}
// Source: include/core/SkArc.h:21-23
impl Default for SkArc {
    #[inline]
    fn default() -> Self {
        Self {
            fOval: SkRect::MakeEmpty(),
            fStartAngle: 0.0,
            fSweepAngle: 0.0,
            fType: Type::kArc,
        }
    }
}
// Source: include/core/SkArc.h:25-25
impl SkArc {
    #[inline]
    pub fn oval(&self) -> &SkRect {
        &self.fOval
    }
}
// Source: include/core/SkArc.h:26-26
impl SkArc {
    #[inline]
    pub fn startAngle(&self) -> f32 {
        self.fStartAngle
    }
}
// Source: include/core/SkArc.h:27-27
impl SkArc {
    #[inline]
    pub fn sweepAngle(&self) -> f32 {
        self.fSweepAngle
    }
}
// Source: include/core/SkArc.h:28-28
impl SkArc {
    #[inline]
    pub fn isWedge(&self) -> bool {
        self.fType == Type::kWedge
    }
}
// Source: include/core/SkArc.h:30-33
impl PartialEq for SkArc {
    #[inline]
    fn eq(&self, b: &Self) -> bool {
        self.fOval == b.fOval
            && self.fStartAngle == b.fStartAngle
            && self.fSweepAngle == b.fSweepAngle
            && self.fType == b.fType
    }
    // Source: include/core/SkArc.h:35-35
    #[inline]
    fn ne(&self, b: &Self) -> bool {
        !self.eq(b)
    }
}
// Source: include/core/SkArc.h:38-43
impl SkArc {
    #[inline]
    pub fn Make(oval: &SkRect, start: f32, sweep: f32, arc_type: Type) -> Self {
        Self::from_parts(*oval, start, sweep, arc_type)
    }
}
// Source: include/core/SkArc.h:46-52
impl SkArc {
    #[inline]
    pub fn MakeWithCenter(oval: &SkRect, start: f32, sweep: f32, useCenter: bool) -> Self {
        Self::from_parts(
            *oval,
            start,
            sweep,
            if useCenter { Type::kWedge } else { Type::kArc },
        )
    }
}
// Source: include/core/SkArc.h:65-66
impl SkArc {
    #[inline]
    fn from_parts(oval: SkRect, start: f32, sweep: f32, arc_type: Type) -> Self {
        Self {
            fOval: oval,
            fStartAngle: start,
            fSweepAngle: sweep,
            fType: arc_type,
        }
    }
}
