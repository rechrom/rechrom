#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// cpp: foundation/graphics_types/graphics/paint_invalidation_reason.h:24-75
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PaintInvalidationReason {
    kNone,
    kIncremental,
    kHitTest,
    kStyle,
    kOutline,
    kImage,
    kBackplate,
    kBackground,
    kSelection,
    kCaret,
    kLayout,
    kAppeared,
    kDisappeared,
    kScrollControl,
    kSubtree,
    kSVGResource,
    kDocumentMarker,
    kJustCreated,
    kReordered,
    kChunkAppeared,
    kChunkDisappeared,
    kChunkUncacheable,
    kChunkReordered,
    kPaintProperty,
    kFullLayer,
    kUncacheable,
}

impl PaintInvalidationReason {
    // C++ enum aliases share discriminants with the last member of each range.
    // cpp: foundation/graphics_types/graphics/paint_invalidation_reason.h:29-75
    pub const kNonFullMax: Self = Self::kHitTest;
    pub const kNonLayoutMax: Self = Self::kCaret;
    pub const kLayoutMax: Self = Self::kDocumentMarker;
    pub const kMax: Self = Self::kUncacheable;
}

// cpp: foundation/graphics_types/graphics/paint_invalidation_reason.h:80-83
pub const fn IsFullPaintInvalidationReason(reason: PaintInvalidationReason) -> bool {
    (reason as u8) > (PaintInvalidationReason::kNonFullMax as u8)
}

// cpp: foundation/graphics_types/graphics/paint_invalidation_reason.h:85-89
pub const fn IsNonLayoutFullPaintInvalidationReason(reason: PaintInvalidationReason) -> bool {
    (reason as u8) > (PaintInvalidationReason::kNonFullMax as u8)
        && (reason as u8) <= (PaintInvalidationReason::kNonLayoutMax as u8)
}

// cpp: foundation/graphics_types/graphics/paint_invalidation_reason.h:91-95
pub const fn IsLayoutFullPaintInvalidationReason(reason: PaintInvalidationReason) -> bool {
    (reason as u8) > (PaintInvalidationReason::kNonLayoutMax as u8)
        && (reason as u8) <= (PaintInvalidationReason::kLayoutMax as u8)
}

// cpp: foundation/graphics_types/graphics/paint_invalidation_reason.h:97-101
pub const fn IsLayoutPaintInvalidationReason(reason: PaintInvalidationReason) -> bool {
    reason as u8 == PaintInvalidationReason::kIncremental as u8
        || IsLayoutFullPaintInvalidationReason(reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reason_boundaries_match_source_ranges() {
        assert!(!IsFullPaintInvalidationReason(
            PaintInvalidationReason::kHitTest
        ));
        assert!(IsNonLayoutFullPaintInvalidationReason(
            PaintInvalidationReason::kStyle
        ));
        assert!(!IsNonLayoutFullPaintInvalidationReason(
            PaintInvalidationReason::kLayout
        ));
        assert!(IsLayoutFullPaintInvalidationReason(
            PaintInvalidationReason::kDocumentMarker
        ));
        assert!(!IsLayoutFullPaintInvalidationReason(
            PaintInvalidationReason::kJustCreated
        ));
        assert!(IsLayoutPaintInvalidationReason(
            PaintInvalidationReason::kIncremental
        ));
    }
}
