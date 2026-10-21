use crate::include::core::SkPathTypes::*;
use crate::src::core::SkPathEnums::*;
use crate::src::core::SkPathIter::SkPathIter;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRect::SkRect;

// Source: src/core/SkPathRaw.h:29-37
#[derive(Clone, Copy, Debug)]
pub struct SkPathRaw<'a> {
    pub fPoints: &'a [SkPoint],
    pub fVerbs: &'a [SkPathVerb],
    pub fConics: &'a [f32],
    pub fBounds: SkRect,
    pub fFillType: SkPathFillType,
    pub fConvexity: SkPathConvexity,
    pub fSegmentMask: u8,
}

// Source: src/core/SkPathRaw.h:39-39
impl<'a> SkPathRaw<'a> {
    #[inline]
    pub fn points(&self) -> &'a [SkPoint] {
        self.fPoints
    }
}

// Source: src/core/SkPathRaw.h:40-40
impl<'a> SkPathRaw<'a> {
    #[inline]
    pub fn verbs(&self) -> &'a [SkPathVerb] {
        self.fVerbs
    }
}

// Source: src/core/SkPathRaw.h:41-41
impl<'a> SkPathRaw<'a> {
    #[inline]
    pub fn conics(&self) -> &'a [f32] {
        self.fConics
    }
}

// Source: src/core/SkPathRaw.h:42-42
impl<'a> SkPathRaw<'a> {
    #[inline]
    pub fn bounds(&self) -> SkRect {
        self.fBounds
    }
}

// Source: src/core/SkPathRaw.h:43-43
impl<'a> SkPathRaw<'a> {
    #[inline]
    pub fn fillType(&self) -> SkPathFillType {
        self.fFillType
    }
}

// Source: src/core/SkPathRaw.h:44-44
impl<'a> SkPathRaw<'a> {
    #[inline]
    pub fn convexity(&self) -> SkPathConvexity {
        self.fConvexity
    }
}

// Source: src/core/SkPathRaw.h:45-45
impl<'a> SkPathRaw<'a> {
    #[inline]
    pub fn segmentMasks(&self) -> u32 {
        self.fSegmentMask as u32
    }
}

// Source: src/core/SkPathRaw.h:47-47
impl<'a> SkPathRaw<'a> {
    #[inline]
    pub fn empty(&self) -> bool {
        self.fVerbs.is_empty()
    }
}

// Source: src/core/SkPathRaw.h:48-48
impl<'a> SkPathRaw<'a> {
    #[inline]
    pub fn isInverseFillType(&self) -> bool {
        SkPathFillType_IsInverse(self.fFillType)
    }
}

// Source: src/core/SkPathRaw.h:49-49
impl<'a> SkPathRaw<'a> {
    #[inline]
    pub fn isKnownToBeConvex(&self) -> bool {
        SkPathConvexity_IsConvex(self.fConvexity)
    }
}

// Source: src/core/SkPathRaw.h:53-53
impl<'a> SkPathRaw<'a> {
    #[inline]
    pub fn iter(&self) -> SkPathIter<'a> {
        SkPathIter::new(self.fPoints, self.fVerbs, self.fConics)
    }
}

// Source: src/core/SkPathRaw.h:55-59
impl<'a> SkPathRaw<'a> {
    #[inline]
    pub fn Empty(ft: SkPathFillType) -> Self {
        Self {
            fPoints: &[],
            fVerbs: &[],
            fConics: &[],
            fBounds: SkRect::MakeEmpty(),
            fFillType: ft,
            fConvexity: SkPathConvexity::kConvex_Degenerate,
            fSegmentMask: 0,
        }
    }
}

// Source: src/core/SkPathRaw.cpp:13-20
pub const gVerbToSegmentMask: [u8; 6] = [0, 1, 2, 4, 8, 0];

// Source: src/core/SkPathRaw.cpp:22-30
impl crate::src::core::SkPathPriv::SkPathPriv {
    pub fn ComputeSegmentMask(verbs: &[SkPathVerb]) -> u8 {
        let mut mask = 0u32;
        for &v in verbs {
            let i = v as usize;
            debug_assert!(i < gVerbToSegmentMask.len());
            mask |= gVerbToSegmentMask[i] as u32;
        }
        mask as u8
    }
}

// Source: src/core/SkPathRaw.cpp:32-37
impl SkPathRaw<'_> {
    pub fn isRect(&self) -> Option<SkRect> {
        crate::src::core::SkPathPriv::SkPathPriv::IsRectContour(
            self.fPoints,
            self.fVerbs,
            self.fSegmentMask as u32,
            false,
        )
        .map(|r| r.fRect)
    }
}
