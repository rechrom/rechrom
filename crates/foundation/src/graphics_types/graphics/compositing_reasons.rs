#![allow(non_snake_case, non_upper_case_globals)]

// The C++ EnumSet stores one bit for each reason. All source reasons fit in u64.
// cpp: foundation/graphics_types/graphics/compositing_reasons.h:23-103
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositingReason {
    k3DTransform,
    k3DScale,
    k3DRotate,
    k3DTranslate,
    kTrivial3DTransform,
    kIFrame,
    kActiveTransformAnimation,
    kActiveScaleAnimation,
    kActiveRotateAnimation,
    kActiveTranslateAnimation,
    kActiveOpacityAnimation,
    kActiveFilterAnimation,
    kActiveBackdropFilterAnimation,
    kAffectedByOuterViewportBoundsDelta,
    kAffectedBySafeAreaBottom,
    kFixedPosition,
    kUndoOverscroll,
    kStickyPosition,
    kAnchorPosition,
    kBackdropFilter,
    kBackdropFilterMask,
    kFixedBackdropInOverscrollAreaParent,
    kRootScroller,
    kViewport,
    kWillChangeTransform,
    kWillChangeScale,
    kWillChangeRotate,
    kWillChangeTranslate,
    kWillChangeOpacity,
    kWillChangeFilter,
    kWillChangeBackdropFilter,
    kWillChangeClipPath,
    kWillChangeMixBlendMode,
    kWillChangeMask,
    kWillChangeOther,
    kBackfaceInvisibility3DAncestor,
    kTransform3DSceneLeaf,
    kPerspectiveWith3DDescendants,
    kPreserve3DWith3DDescendants,
    kViewTransitionElement,
    kViewTransitionPseudoElement,
    kViewTransitionElementDescendantWithClipPath,
    kOverflowScrolling,
    kElementCapture,
    kOverlap,
    kBackfaceVisibilityHidden,
    kFixedAttachmentBackground,
    kCaret,
    kVideo,
    kCanvas,
    kCanvasChild,
    kPlugin,
    kScrollbar,
    kLinkHighlight,
    kDevToolsOverlay,
    kViewTransitionContent,
    kUnboundedElement,
}

// cpp: foundation/graphics_types/graphics/compositing_reasons.h:105-105
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompositingReasons(u64);

impl CompositingReasons {
    pub const fn new() -> Self {
        Self(0)
    }
    pub const fn Has(&self, reason: CompositingReason) -> bool {
        self.0 & (1_u64 << reason as u8) != 0
    }
    pub fn Put(&mut self, reason: CompositingReason) {
        self.0 |= 1_u64 << reason as u8;
    }
    pub fn PutAll(&mut self, other: Self) {
        self.0 |= other.0;
    }
    pub fn Remove(&mut self, reason: CompositingReason) {
        self.0 &= !(1_u64 << reason as u8);
    }
    pub const fn IsEmpty(&self) -> bool {
        self.0 == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reasons_are_independent_bits() {
        let mut reasons = CompositingReasons::default();
        reasons.Put(CompositingReason::kFixedPosition);
        assert!(reasons.Has(CompositingReason::kFixedPosition));
        assert!(!reasons.Has(CompositingReason::kStickyPosition));
        reasons.Remove(CompositingReason::kFixedPosition);
        assert!(reasons.IsEmpty());
    }
}
