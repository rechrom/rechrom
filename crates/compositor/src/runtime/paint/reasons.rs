//! Exact native CompositingReasons value domain and diagnostic metadata.
use crate::adapter::paint_artifact_compositor::{CompositingReason, CompositingReasons};
// Source: third_party/blink/renderer/platform/graphics/compositing_reasons.h:20-106.
#[derive(Clone, Default, Debug)]
pub struct CpuCompositingReasons {
    bits: u64,
}
const REASONS: &[(&str, &str, &str)] = &[
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:23-23.
    ("3DTransform", "3DTransform", "Has a 3d transform."),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:24-24.
    ("3DScale", "3DScale", "Has a 3d scale."),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:25-25.
    ("3DRotate", "3DRotate", "Has a 3d rotate."),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:26-26.
    ("3DTranslate", "3DTranslate", "Has a 3d translate."),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:27-28.
    (
        "Trivial3DTransform",
        "Trivial3DTransform",
        "Has a trivial 3d transform.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:29-29.
    ("IFrame", "IFrame", "Is an accelerated iFrame."),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:30-31.
    (
        "ActiveTransformAnimation",
        "ActiveTransformAnimation",
        "Has an active accelerated transform animation or transition.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:32-33.
    (
        "ActiveScaleAnimation",
        "ActiveScaleAnimation",
        "Has an active accelerated scale animation or transition.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:34-35.
    (
        "ActiveRotateAnimation",
        "ActiveRotateAnimation",
        "Has an active accelerated rotate animation or transition.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:36-37.
    (
        "ActiveTranslateAnimation",
        "ActiveTranslateAnimation",
        "Has an active accelerated translate animation or transition.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:38-39.
    (
        "ActiveOpacityAnimation",
        "ActiveOpacityAnimation",
        "Has an active accelerated opacity animation or transition.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:40-41.
    (
        "ActiveFilterAnimation",
        "ActiveFilterAnimation",
        "Has an active accelerated filter animation or transition.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:42-44.
    (
        "ActiveBackdropFilterAnimation",
        "ActiveBackdropFilterAnimation",
        "Has an active accelerated backdrop filter animation or transition.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:45-47.
    (
        "AffectedByOuterViewportBoundsDelta",
        "AffectedByOuterViewportBoundsDelta",
        "Is fixed position affected by outer viewport bounds delta.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:48-49.
    (
        "AffectedBySafeAreaBottom",
        "AffectedBySafeAreaBottom",
        "Is fixed position affected by safe area bottom.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:50-51.
    (
        "FixedPosition",
        "FixedPosition",
        "Is fixed position in a scrollable view.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:52-53.
    (
        "UndoOverscroll",
        "UndoOverscroll",
        "Is fixed position that should undo overscroll of the viewport.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:54-55.
    ("StickyPosition", "StickyPosition", "Is sticky position."),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:56-58.
    (
        "AnchorPosition",
        "AnchorPosition",
        "Is an anchor-positioned element translated by its anchor's scroll offset.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:59-60.
    ("BackdropFilter", "BackdropFilter", "Has a backdrop filter."),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:61-62.
    (
        "BackdropFilterMask",
        "BackdropFilterMask",
        "Is a mask for backdrop filter.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:63-65.
    (
        "FixedBackdropInOverscrollAreaParent",
        "FixedBackdropInOverscrollAreaParent",
        "Is a fixed backdrop inside an overscroll area parent scroller.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:66-67.
    (
        "RootScroller",
        "RootScroller",
        "Is the document.rootScroller.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:68-68.
    ("Viewport", "Viewport", "Is for the visual viewport."),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:69-70.
    (
        "WillChangeTransform",
        "WillChangeTransform",
        "Has a will-change: transform compositing hint.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:71-72.
    (
        "WillChangeScale",
        "WillChangeScale",
        "Has a will-change: scale compositing hint.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:73-74.
    (
        "WillChangeRotate",
        "WillChangeRotate",
        "Has a will-change: rotate compositing hint.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:75-76.
    (
        "WillChangeTranslate",
        "WillChangeTranslate",
        "Has a will-change: translate compositing hint.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:77-78.
    (
        "WillChangeOpacity",
        "WillChangeOpacity",
        "Has a will-change: opacity compositing hint.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:79-80.
    (
        "WillChangeFilter",
        "WillChangeFilter",
        "Has a will-change: filter compositing hint.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:81-82.
    (
        "WillChangeBackdropFilter",
        "WillChangeBackdropFilter",
        "Has a will-change: backdrop-filter compositing hint.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:83-84.
    (
        "WillChangeClipPath",
        "WillChangeClipPath",
        "Has a will-change: clip-path compositing hint.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:85-86.
    (
        "WillChangeMixBlendMode",
        "WillChangeMixBlendMode",
        "Has a will-change: mix-blend-mode compositing hint.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:87-88.
    (
        "WillChangeMask",
        "WillChangeMask",
        "Has a will-change: mask compositing hint.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:89-91.
    (
        "WillChangeOther",
        "WillChangeOther",
        "Has a will-change compositing hint other than transform, opacity, filter and backdrop-filter.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:92-94.
    (
        "BackfaceInvisibility3DAncestor",
        "BackfaceInvisibility3DAncestor",
        "Ancestor in same 3D rendering context has a hidden backface.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:95-96.
    (
        "Transform3DSceneLeaf",
        "Transform3DSceneLeaf",
        "Leaf of a 3D scene, for flattening its descendants into that scene.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:97-100.
    (
        "PerspectiveWith3DDescendants",
        "PerspectiveWith3DDescendants",
        "Has a perspective transform that needs to be known by compositor because of 3d descendants.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:101-104.
    (
        "Preserve3DWith3DDescendants",
        "Preserve3DWith3DDescendants",
        "Has a preserves-3d property that needs to be known by compositor because of 3d descendants.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:105-106.
    (
        "ViewTransitionElement",
        "ViewTransitionElement",
        "This element is shared during view transition.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:107-110.
    (
        "ViewTransitionPseudoElement",
        "ViewTransitionPseudoElement",
        "This element is a part of a pseudo-element tree representing the view transition.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:111-114.
    (
        "ViewTransitionElementDescendantWithClipPath",
        "ViewTransitionElementDescendantWithClipPath",
        "This element's ancestor is shared during view transition and it has a clip-path",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:115-116.
    (
        "OverflowScrolling",
        "OverflowScrolling",
        "Is a scrollable overflow element using accelerated scrolling.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:117-118.
    (
        "ElementCapture",
        "ElementCapture",
        "This element is undergoing element-level capture.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:119-120.
    ("Overlap", "Overlap", "Overlaps other composited content."),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:121-122.
    (
        "BackfaceVisibilityHidden",
        "BackfaceVisibilityHidden",
        "Has backface-visibility: hidden.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:123-124.
    (
        "FixedAttachmentBackground",
        "FixedAttachmentBackground",
        "Is an accelerated background-attachment:fixed background.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:125-125.
    ("Caret", "Caret", "Is a caret in an editor."),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:126-126.
    ("Video", "Video", "Is an accelerated video."),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:127-129.
    (
        "Canvas",
        "Canvas",
        "Is an accelerated canvas, or is a display list backed canvas that was promoted to a layer based on a performance heuristic.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:130-131.
    (
        "CanvasChild",
        "CanvasChild",
        "Is the direct child of a canvas with 'layoutSubtree' attribute.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:132-132.
    ("Plugin", "Plugin", "Is an accelerated plugin."),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:133-134.
    ("Scrollbar", "Scrollbar", "Is an accelerated scrollbar."),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:135-136.
    (
        "LinkHighlight",
        "LinkHighlight",
        "Is a tap highlight on a link.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:137-138.
    ("DevToolsOverlay", "DevToolsOverlay", "Is DevTools overlay."),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:139-140.
    (
        "ViewTransitionContent",
        "ViewTransitionContent",
        "The layer containing the contents of a view transition element.",
    ),
    // Source: third_party/blink/renderer/platform/graphics/compositing_reasons.cc:141-142.
    (
        "UnboundedElement",
        "UnboundedElement",
        "Is an active unbounded element.",
    ),
];
impl CpuCompositingReasons {
    pub fn from_native(reasons: &[&str]) -> Self {
        let mut value = Self::default();
        for name in reasons {
            value.put_name(name);
        }
        value
    }
    fn put_name(&mut self, name: &str) {
        let i = REASONS
            .iter()
            .position(|r| r.0 == name)
            .expect("source compositor reason domain");
        self.bits |= 1u64 << i;
    }
    pub fn descriptions(&self) -> Vec<&'static str> {
        REASONS
            .iter()
            .enumerate()
            .filter_map(|(i, r)| (self.bits & (1u64 << i) != 0).then_some(r.2))
            .collect()
    }
    pub fn short_names(&self) -> Vec<&'static str> {
        REASONS
            .iter()
            .enumerate()
            .filter_map(|(i, r)| (self.bits & (1u64 << i) != 0).then_some(r.1))
            .collect()
    }
}
impl CompositingReasons for CpuCompositingReasons {
    fn empty(&self) -> bool {
        self.bits == 0
    }
    fn Put(&mut self, r: CompositingReason) {
        let name = match r {
            CompositingReason::OverflowScrolling => "OverflowScrolling",
            CompositingReason::FixedAttachmentBackground => "FixedAttachmentBackground",
            CompositingReason::Caret => "Caret",
            CompositingReason::Scrollbar => "Scrollbar",
            CompositingReason::Canvas => "Canvas",
            CompositingReason::DevToolsOverlay => "DevToolsOverlay",
            CompositingReason::Plugin => "Plugin",
            CompositingReason::Video => "Video",
            CompositingReason::IFrame => "IFrame",
            CompositingReason::LinkHighlight => "LinkHighlight",
            CompositingReason::Viewport => "Viewport",
            CompositingReason::ViewTransitionContent => "ViewTransitionContent",
            CompositingReason::BackfaceVisibilityHidden => "BackfaceVisibilityHidden",
            CompositingReason::Overlap => "Overlap",
        };
        self.put_name(name)
    }
    fn PutAll(&mut self, other: Self) {
        self.bits |= other.bits;
    }
}
