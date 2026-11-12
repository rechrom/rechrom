#![allow(non_snake_case)]

use std::collections::HashSet;

/// The document-scoped features currently emitted by translated LayoutImage
/// code. Keep the feature identity at the DOM boundary, matching Blink's
/// `WebFeature`/`UseCounter` ownership rather than leaking telemetry flags into
/// Layout or Paint inputs.
///
/// cpp: public/mojom/use_counter/metrics/web_feature.mojom
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WebFeature {
    kLcpCandidateRemovedWhilePaintTimePending,
    kEditingAppleInterchangeNewline,
    kExecutedJavaScriptURL,
    kExecutedNonTrivialJavaScriptURL,
    kReplaceDocumentViaJavaScriptURL,
    kMediaElementTextTrackList,
    kTrustedTypesCreatePolicy,
    kTrustedTypesCreatePolicyWithEmptyName,
    kTrustedTypesPolicyCreated,
    kTrustedTypesDefaultPolicyCreated,
    kTrustedTypesAssignmentError,
    kTrustedTypesIsCheck,
    kOpaqueRange,
    kVTTCueParser,
    kVTTCue,
    kVTTCueRender,
    kVTTCueRenderVertical,
    kVTTCueRenderRtl,
    kVTTCueRenderSnapToLinesFalse,
    kVTTCueRenderLineNotAuto,
    kVTTCueRenderPositionNot50,
    kVTTCueRenderSizeNot100,
    kVTTCueRenderAlignNotCenter,
    kGetBBoxForElementWithZeroWidthOrHeight,
    kGetBBoxForNestedSVGElementWithZeroWidthOrHeight,

    kAutoExpandedDetailsForScrollToTextFragment,
    kTextDirectiveInShadowDOM,
    kInvalidFragmentDirective,
    kTextFragmentBlockedByForceLoadAtTop,
    kBaseElement,
    kContentSecurityPolicyWithBaseElement,
    kBaseWithDataHref,
    kBaseWithCrossOriginHref,
    kAnchorCaseSensitiveMatch,
    kAnchorCaseInsensitiveMatch,

    kTemporalInputTypeChooserByTrustedClick,
    kTemporalInputTypeChooserByUntrustedClick,
    kTemporalInputTypeIgnoreUntrustedClick,
    kScrollbarUseScrollbarButtonReversedDirection,
    kScrollbarUseVerticalScrollbarButton,
    kScrollbarUseHorizontalScrollbarButton,
    kVerticalScrollbarThumbScrollingWithMouse,
    kVerticalScrollbarThumbScrollingWithTouch,
    kHorizontalScrollbarThumbScrollingWithMouse,
    kHorizontalScrollbarThumbScrollingWithTouch,
    kScrollbarUseVerticalScrollbarTrack,
    kScrollbarUseHorizontalScrollbarTrack,
    kColorInputTypeChooserByTrustedClick,
    kColorInputTypeChooserByUntrustedClick,
    kInputTypeFileSecureOrigin,
    kInputTypeFileInsecureOrigin,
    kInputTypeFileSecureOriginOpenChooser,
    kInputTypeFileInsecureOriginOpenChooser,
    kPopupOpenWhileFileChooserOpened,
    kDisallowedSelectChild,
    kDisallowedOptGroupChild,
    kNonPhrasingContentOptionChild,
    kInteractiveContentOptionChild,
    kInteractiveContentLegendChild,
    kSelectElementAppearanceBaseSelect,
    kSelectElementPickerAppearanceBaseSelect,
    kPopoverTypeInvalid,
    kValidPopoverAttribute,
    kPopoverTypeAuto,
    kPopoverTypeHint,
    kPopoverTypeManual,
    kPopoverShown,
    kPopoverTypeAutoShown,
    kPopoverTypeHintShown,
    kPopoverTypeManualShown,
    kShowModalForElementInFullscreenStack,
    kDialogClosedby,
    kSelectionToStringWithShadowTree,
    kWindowFindWithShadowTree,
    kInputTypeHidden,
    kInputTypeColor,
    kInputTypeImage,
    kClientHintsContentDPR,
    kAnimatedImageUsedMoreThanOnce,
}

/// Sticky, document-owned feature usage. Blink's UseCounter records a feature
/// once per Document; callers may therefore report from repeated notifications
/// and paint phases without manufacturing an event count.
///
/// cpp: core/frame/use_counter.h
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WebDXFeature {
    kMetaTextScale,
    kLinkRelExpect,
}

#[derive(Default)]
pub struct UseCounter {
    css: std::rc::Rc<foundation::css_use_counter::CSSUseCounter>,
    counted_features: std::cell::RefCell<HashSet<WebFeature>>,
    counted_webdx_features: std::cell::RefCell<HashSet<WebDXFeature>>,
}

impl UseCounter {
    pub fn CSS(&self) -> std::rc::Rc<foundation::css_use_counter::CSSUseCounter> {
        self.css.clone()
    }
    pub fn CountWebDX(&self, feature: WebDXFeature) {
        self.counted_webdx_features.borrow_mut().insert(feature);
    }
    pub fn IsWebDXCounted(&self, feature: WebDXFeature) -> bool {
        self.counted_webdx_features.borrow().contains(&feature)
    }
    pub fn Count(&self, feature: WebFeature) {
        self.counted_features.borrow_mut().insert(feature);
    }

    pub fn IsCounted(&self, feature: WebFeature) -> bool {
        self.counted_features.borrow().contains(&feature)
    }
}
