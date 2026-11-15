#![allow(non_snake_case, non_upper_case_globals)]

use std::fmt;

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
    pub const ALL: [CompositingReason; 57] = [
        CompositingReason::k3DTransform,
        CompositingReason::k3DScale,
        CompositingReason::k3DRotate,
        CompositingReason::k3DTranslate,
        CompositingReason::kTrivial3DTransform,
        CompositingReason::kIFrame,
        CompositingReason::kActiveTransformAnimation,
        CompositingReason::kActiveScaleAnimation,
        CompositingReason::kActiveRotateAnimation,
        CompositingReason::kActiveTranslateAnimation,
        CompositingReason::kActiveOpacityAnimation,
        CompositingReason::kActiveFilterAnimation,
        CompositingReason::kActiveBackdropFilterAnimation,
        CompositingReason::kAffectedByOuterViewportBoundsDelta,
        CompositingReason::kAffectedBySafeAreaBottom,
        CompositingReason::kFixedPosition,
        CompositingReason::kUndoOverscroll,
        CompositingReason::kStickyPosition,
        CompositingReason::kAnchorPosition,
        CompositingReason::kBackdropFilter,
        CompositingReason::kBackdropFilterMask,
        CompositingReason::kFixedBackdropInOverscrollAreaParent,
        CompositingReason::kRootScroller,
        CompositingReason::kViewport,
        CompositingReason::kWillChangeTransform,
        CompositingReason::kWillChangeScale,
        CompositingReason::kWillChangeRotate,
        CompositingReason::kWillChangeTranslate,
        CompositingReason::kWillChangeOpacity,
        CompositingReason::kWillChangeFilter,
        CompositingReason::kWillChangeBackdropFilter,
        CompositingReason::kWillChangeClipPath,
        CompositingReason::kWillChangeMixBlendMode,
        CompositingReason::kWillChangeMask,
        CompositingReason::kWillChangeOther,
        CompositingReason::kBackfaceInvisibility3DAncestor,
        CompositingReason::kTransform3DSceneLeaf,
        CompositingReason::kPerspectiveWith3DDescendants,
        CompositingReason::kPreserve3DWith3DDescendants,
        CompositingReason::kViewTransitionElement,
        CompositingReason::kViewTransitionPseudoElement,
        CompositingReason::kViewTransitionElementDescendantWithClipPath,
        CompositingReason::kOverflowScrolling,
        CompositingReason::kElementCapture,
        CompositingReason::kOverlap,
        CompositingReason::kBackfaceVisibilityHidden,
        CompositingReason::kFixedAttachmentBackground,
        CompositingReason::kCaret,
        CompositingReason::kVideo,
        CompositingReason::kCanvas,
        CompositingReason::kCanvasChild,
        CompositingReason::kPlugin,
        CompositingReason::kScrollbar,
        CompositingReason::kLinkHighlight,
        CompositingReason::kDevToolsOverlay,
        CompositingReason::kViewTransitionContent,
        CompositingReason::kUnboundedElement,
    ];

    pub const fn new() -> Self {
        Self(0)
    }
    pub const fn Has(&self, reason: CompositingReason) -> bool {
        self.0 & (1_u64 << reason as u8) != 0
    }
    pub const fn Put(&mut self, reason: CompositingReason) {
        self.0 |= 1_u64 << reason as u8;
    }
    pub const fn PutAll(&mut self, other: Self) {
        self.0 |= other.0;
    }
    pub const fn Remove(&mut self, reason: CompositingReason) {
        self.0 &= !(1_u64 << reason as u8);
    }
    pub const fn IsEmpty(&self) -> bool {
        self.0 == 0
    }

    pub const fn FromReason(reason: CompositingReason) -> Self {
        Self(1_u64 << reason as u8)
    }

    pub const fn FromReasons(reasons: &[CompositingReason]) -> Self {
        let mut result = Self::new();
        let mut index = 0;
        while index < reasons.len() {
            result.Put(reasons[index]);
            index += 1;
        }
        result
    }

    pub fn HasAny(&self, reasons: &[CompositingReason]) -> bool {
        reasons.iter().any(|reason| self.Has(*reason))
    }

    pub const fn Difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    pub const fn Intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub const fn is_empty(&self) -> bool {
        self.IsEmpty()
    }

    pub fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub fn iter_reasons(&self) -> impl Iterator<Item = CompositingReason> + '_ {
        Self::ALL.into_iter().filter(|reason| self.Has(*reason))
    }
}

/// Source: `platform/graphics/compositing_reasons.h:113-205`.
pub struct CompositingReasonCombos;

impl CompositingReasonCombos {
    pub const kPreventingSubpixelAccumulationReasons: CompositingReasons =
        CompositingReasons::FromReasons(&[
            CompositingReason::kWillChangeTransform,
            CompositingReason::kWillChangeScale,
            CompositingReason::kWillChangeRotate,
        ]);

    pub const kDirectReasonsForPaintOffsetTranslationProperty: CompositingReasons =
        CompositingReasons::FromReasons(&[
            CompositingReason::kFixedPosition,
            CompositingReason::kAffectedByOuterViewportBoundsDelta,
            CompositingReason::kUndoOverscroll,
            CompositingReason::kVideo,
            CompositingReason::kCanvas,
            CompositingReason::kCanvasChild,
            CompositingReason::kPlugin,
            CompositingReason::kIFrame,
            CompositingReason::kAffectedBySafeAreaBottom,
            CompositingReason::kFixedBackdropInOverscrollAreaParent,
        ]);

    pub const kFixedPositionReasons: CompositingReasons = CompositingReasons::FromReasons(&[
        CompositingReason::kFixedPosition,
        CompositingReason::kUndoOverscroll,
        CompositingReason::kAffectedByOuterViewportBoundsDelta,
        CompositingReason::kAffectedBySafeAreaBottom,
    ]);

    pub const kDirectReasonsForTransformProperty: CompositingReasons =
        CompositingReasons::FromReasons(&[
            CompositingReason::k3DTransform,
            CompositingReason::kTrivial3DTransform,
            CompositingReason::kWillChangeTransform,
            CompositingReason::kWillChangeOther,
            CompositingReason::kPerspectiveWith3DDescendants,
            CompositingReason::kPreserve3DWith3DDescendants,
            CompositingReason::kActiveTransformAnimation,
            CompositingReason::kViewTransitionElementDescendantWithClipPath,
            CompositingReason::kViewTransitionElement,
        ]);

    pub const kDirectReasonsForScaleProperty: CompositingReasons =
        CompositingReasons::FromReasons(&[
            CompositingReason::k3DScale,
            CompositingReason::kWillChangeScale,
            CompositingReason::kActiveScaleAnimation,
        ]);

    pub const kDirectReasonsForRotateProperty: CompositingReasons =
        CompositingReasons::FromReasons(&[
            CompositingReason::k3DRotate,
            CompositingReason::kWillChangeRotate,
            CompositingReason::kActiveRotateAnimation,
        ]);

    pub const kDirectReasonsForTranslateProperty: CompositingReasons =
        CompositingReasons::FromReasons(&[
            CompositingReason::k3DTranslate,
            CompositingReason::kWillChangeTranslate,
            CompositingReason::kActiveTranslateAnimation,
        ]);

    pub const kDirectReasonsForScrollTranslationProperty: CompositingReasons =
        CompositingReasons::FromReasons(&[
            CompositingReason::kRootScroller,
            CompositingReason::kOverflowScrolling,
        ]);

    pub const kDirectReasonsForEffectProperty: CompositingReasons =
        CompositingReasons::FromReasons(&[
            CompositingReason::kActiveOpacityAnimation,
            CompositingReason::kWillChangeOpacity,
            CompositingReason::kBackdropFilter,
            CompositingReason::kWillChangeBackdropFilter,
            CompositingReason::kWillChangeMixBlendMode,
            CompositingReason::kActiveBackdropFilterAnimation,
            CompositingReason::kViewTransitionPseudoElement,
            CompositingReason::kTransform3DSceneLeaf,
            CompositingReason::kElementCapture,
            CompositingReason::kCanvasChild,
            CompositingReason::kUnboundedElement,
        ]);

    pub const kDirectReasonsForFilterProperty: CompositingReasons =
        CompositingReasons::FromReasons(&[
            CompositingReason::kActiveFilterAnimation,
            CompositingReason::kWillChangeFilter,
        ]);

    pub const kDirectReasonsForBackdropFilter: CompositingReasons =
        CompositingReasons::FromReasons(&[
            CompositingReason::kBackdropFilter,
            CompositingReason::kActiveBackdropFilterAnimation,
            CompositingReason::kWillChangeBackdropFilter,
        ]);

    pub const kAuxiliaryReasonsForBackdropRoot: CompositingReasons =
        CompositingReasons::FromReasons(&[
            CompositingReason::kWillChangeClipPath,
            CompositingReason::kWillChangeMask,
        ]);

    pub const kAdditionalEffectCompositingTrigger: CompositingReasons =
        CompositingReasons::FromReasons(&[
            CompositingReason::k3DTransform,
            CompositingReason::kTrivial3DTransform,
            CompositingReason::kWillChangeTransform,
        ]);

    pub const kRequiresCullRectExpansion: CompositingReasons = {
        let mut reasons = CompositingReasons::FromReasons(&[
            CompositingReason::kStickyPosition,
            CompositingReason::kAnchorPosition,
        ]);
        reasons.PutAll(Self::kDirectReasonsForTransformProperty);
        reasons.PutAll(Self::kDirectReasonsForScaleProperty);
        reasons.PutAll(Self::kDirectReasonsForRotateProperty);
        reasons.PutAll(Self::kDirectReasonsForTranslateProperty);
        reasons.PutAll(Self::kDirectReasonsForScrollTranslationProperty);
        reasons
    };
}

impl CompositingReason {
    // Chromium: platform/graphics/compositing_reasons.h:21,101.
    pub const kMinValue: Self = Self::k3DTransform;
    pub const kMaxValue: Self = Self::kUnboundedElement;

    pub const fn Name(self) -> &'static str {
        match self {
            Self::k3DTransform => "3DTransform",
            Self::k3DScale => "3DScale",
            Self::k3DRotate => "3DRotate",
            Self::k3DTranslate => "3DTranslate",
            Self::kTrivial3DTransform => "Trivial3DTransform",
            Self::kIFrame => "IFrame",
            Self::kActiveTransformAnimation => "ActiveTransformAnimation",
            Self::kActiveScaleAnimation => "ActiveScaleAnimation",
            Self::kActiveRotateAnimation => "ActiveRotateAnimation",
            Self::kActiveTranslateAnimation => "ActiveTranslateAnimation",
            Self::kActiveOpacityAnimation => "ActiveOpacityAnimation",
            Self::kActiveFilterAnimation => "ActiveFilterAnimation",
            Self::kActiveBackdropFilterAnimation => "ActiveBackdropFilterAnimation",
            Self::kAffectedByOuterViewportBoundsDelta => "AffectedByOuterViewportBoundsDelta",
            Self::kAffectedBySafeAreaBottom => "AffectedBySafeAreaBottom",
            Self::kFixedPosition => "FixedPosition",
            Self::kUndoOverscroll => "UndoOverscroll",
            Self::kStickyPosition => "StickyPosition",
            Self::kAnchorPosition => "AnchorPosition",
            Self::kBackdropFilter => "BackdropFilter",
            Self::kBackdropFilterMask => "BackdropFilterMask",
            Self::kFixedBackdropInOverscrollAreaParent => "FixedBackdropInOverscrollAreaParent",
            Self::kRootScroller => "RootScroller",
            Self::kViewport => "Viewport",
            Self::kWillChangeTransform => "WillChangeTransform",
            Self::kWillChangeScale => "WillChangeScale",
            Self::kWillChangeRotate => "WillChangeRotate",
            Self::kWillChangeTranslate => "WillChangeTranslate",
            Self::kWillChangeOpacity => "WillChangeOpacity",
            Self::kWillChangeFilter => "WillChangeFilter",
            Self::kWillChangeBackdropFilter => "WillChangeBackdropFilter",
            Self::kWillChangeClipPath => "WillChangeClipPath",
            Self::kWillChangeMixBlendMode => "WillChangeMixBlendMode",
            Self::kWillChangeMask => "WillChangeMask",
            Self::kWillChangeOther => "WillChangeOther",
            Self::kBackfaceInvisibility3DAncestor => "BackfaceInvisibility3DAncestor",
            Self::kTransform3DSceneLeaf => "Transform3DSceneLeaf",
            Self::kPerspectiveWith3DDescendants => "PerspectiveWith3DDescendants",
            Self::kPreserve3DWith3DDescendants => "Preserve3DWith3DDescendants",
            Self::kViewTransitionElement => "ViewTransitionElement",
            Self::kViewTransitionPseudoElement => "ViewTransitionPseudoElement",
            Self::kViewTransitionElementDescendantWithClipPath => {
                "ViewTransitionElementDescendantWithClipPath"
            }
            Self::kOverflowScrolling => "OverflowScrolling",
            Self::kElementCapture => "ElementCapture",
            Self::kOverlap => "Overlap",
            Self::kBackfaceVisibilityHidden => "BackfaceVisibilityHidden",
            Self::kFixedAttachmentBackground => "FixedAttachmentBackground",
            Self::kCaret => "Caret",
            Self::kVideo => "Video",
            Self::kCanvas => "Canvas",
            Self::kCanvasChild => "CanvasChild",
            Self::kPlugin => "Plugin",
            Self::kScrollbar => "Scrollbar",
            Self::kLinkHighlight => "LinkHighlight",
            Self::kDevToolsOverlay => "DevToolsOverlay",
            Self::kViewTransitionContent => "ViewTransitionContent",
            Self::kUnboundedElement => "UnboundedElement",
        }
    }

    // Chromium: platform/graphics/compositing_reasons.cc:17-143.
    pub const fn Description(self) -> &'static str {
        match self {
            Self::k3DTransform => "Has a 3d transform.",
            Self::k3DScale => "Has a 3d scale.",
            Self::k3DRotate => "Has a 3d rotate.",
            Self::k3DTranslate => "Has a 3d translate.",
            Self::kTrivial3DTransform => "Has a trivial 3d transform.",
            Self::kIFrame => "Is an accelerated iFrame.",
            Self::kActiveTransformAnimation => {
                "Has an active accelerated transform animation or transition."
            }
            Self::kActiveScaleAnimation => {
                "Has an active accelerated scale animation or transition."
            }
            Self::kActiveRotateAnimation => {
                "Has an active accelerated rotate animation or transition."
            }
            Self::kActiveTranslateAnimation => {
                "Has an active accelerated translate animation or transition."
            }
            Self::kActiveOpacityAnimation => {
                "Has an active accelerated opacity animation or transition."
            }
            Self::kActiveFilterAnimation => {
                "Has an active accelerated filter animation or transition."
            }
            Self::kActiveBackdropFilterAnimation => {
                "Has an active accelerated backdrop filter animation or transition."
            }
            Self::kAffectedByOuterViewportBoundsDelta => {
                "Is fixed position affected by outer viewport bounds delta."
            }
            Self::kAffectedBySafeAreaBottom => "Is fixed position affected by safe area bottom.",
            Self::kFixedPosition => "Is fixed position in a scrollable view.",
            Self::kUndoOverscroll => {
                "Is fixed position that should undo overscroll of the viewport."
            }
            Self::kStickyPosition => "Is sticky position.",
            Self::kAnchorPosition => {
                "Is an anchor-positioned element translated by its anchor's scroll offset."
            }
            Self::kBackdropFilter => "Has a backdrop filter.",
            Self::kBackdropFilterMask => "Is a mask for backdrop filter.",
            Self::kFixedBackdropInOverscrollAreaParent => {
                "Is a fixed backdrop inside an overscroll area parent scroller."
            }
            Self::kRootScroller => "Is the document.rootScroller.",
            Self::kViewport => "Is for the visual viewport.",
            Self::kWillChangeTransform => "Has a will-change: transform compositing hint.",
            Self::kWillChangeScale => "Has a will-change: scale compositing hint.",
            Self::kWillChangeRotate => "Has a will-change: rotate compositing hint.",
            Self::kWillChangeTranslate => "Has a will-change: translate compositing hint.",
            Self::kWillChangeOpacity => "Has a will-change: opacity compositing hint.",
            Self::kWillChangeFilter => "Has a will-change: filter compositing hint.",
            Self::kWillChangeBackdropFilter => {
                "Has a will-change: backdrop-filter compositing hint."
            }
            Self::kWillChangeClipPath => "Has a will-change: clip-path compositing hint.",
            Self::kWillChangeMixBlendMode => "Has a will-change: mix-blend-mode compositing hint.",
            Self::kWillChangeMask => "Has a will-change: mask compositing hint.",
            Self::kWillChangeOther => {
                "Has a will-change compositing hint other than transform, opacity, filter and backdrop-filter."
            }
            Self::kBackfaceInvisibility3DAncestor => {
                "Ancestor in same 3D rendering context has a hidden backface."
            }
            Self::kTransform3DSceneLeaf => {
                "Leaf of a 3D scene, for flattening its descendants into that scene."
            }
            Self::kPerspectiveWith3DDescendants => {
                "Has a perspective transform that needs to be known by compositor because of 3d descendants."
            }
            Self::kPreserve3DWith3DDescendants => {
                "Has a preserves-3d property that needs to be known by compositor because of 3d descendants."
            }
            Self::kViewTransitionElement => "This element is shared during view transition.",
            Self::kViewTransitionPseudoElement => {
                "This element is a part of a pseudo-element tree representing the view transition."
            }
            Self::kViewTransitionElementDescendantWithClipPath => {
                "This element's ancestor is shared during view transition and it has a clip-path"
            }
            Self::kOverflowScrolling => {
                "Is a scrollable overflow element using accelerated scrolling."
            }
            Self::kElementCapture => "This element is undergoing element-level capture.",
            Self::kOverlap => "Overlaps other composited content.",
            Self::kBackfaceVisibilityHidden => "Has backface-visibility: hidden.",
            Self::kFixedAttachmentBackground => {
                "Is an accelerated background-attachment:fixed background."
            }
            Self::kCaret => "Is a caret in an editor.",
            Self::kVideo => "Is an accelerated video.",
            Self::kCanvas => {
                "Is an accelerated canvas, or is a display list backed canvas that was promoted to a layer based on a performance heuristic."
            }
            Self::kCanvasChild => "Is the direct child of a canvas with 'layoutSubtree' attribute.",
            Self::kPlugin => "Is an accelerated plugin.",
            Self::kScrollbar => "Is an accelerated scrollbar.",
            Self::kLinkHighlight => "Is a tap highlight on a link.",
            Self::kDevToolsOverlay => "Is DevTools overlay.",
            Self::kViewTransitionContent => {
                "The layer containing the contents of a view transition element."
            }
            Self::kUnboundedElement => "Is an active unbounded element.",
        }
    }
}

// Chromium: platform/graphics/compositing_reasons.cc:162-182.
pub fn ShortNames(reasons: CompositingReasons) -> Vec<&'static str> {
    reasons
        .iter_reasons()
        .map(CompositingReason::Name)
        .collect()
}

pub fn Descriptions(reasons: CompositingReasons) -> Vec<&'static str> {
    reasons
        .iter_reasons()
        .map(CompositingReason::Description)
        .collect()
}

pub fn ToString(reasons: CompositingReasons) -> String {
    ShortNames(reasons).join(",")
}

// Chromium: platform/graphics/compositing_reasons.cc:184-190.
impl fmt::Display for CompositingReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.Name())
    }
}

impl fmt::Display for CompositingReasons {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&ToString(*self))
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

    #[test]
    fn diagnostics_follow_reason_order() {
        assert_eq!(
            CompositingReasons::ALL.len(),
            CompositingReason::kMaxValue as usize + 1
        );
        assert!(CompositingReasons::ALL
            .iter()
            .enumerate()
            .all(|(index, reason)| index == *reason as usize));
        let reasons = CompositingReasons::FromReasons(&[
            CompositingReason::kStickyPosition,
            CompositingReason::kCanvas,
        ]);
        assert_eq!(ShortNames(reasons), vec!["StickyPosition", "Canvas"]);
        assert_eq!(
            Descriptions(reasons),
            vec![
                "Is sticky position.",
                "Is an accelerated canvas, or is a display list backed canvas that was promoted to a layer based on a performance heuristic."
            ]
        );
        assert_eq!(ToString(reasons), "StickyPosition,Canvas");
        assert_eq!(reasons.to_string(), "StickyPosition,Canvas");
    }
}
