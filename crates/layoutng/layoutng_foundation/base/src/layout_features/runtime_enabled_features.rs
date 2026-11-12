// C++: src/foundation/layout_features/runtime_enabled_features.h

use std::sync::atomic::{AtomicBool, Ordering};

// Pinned runtime_enabled_features.json5:3299-3304: experimental, disabled
// unless explicitly enabled through the runtime feature owner.
static GENERATE_DRAG_OVERLAY_BEFORE_DRAG_START_ENABLED: AtomicBool = AtomicBool::new(false);

// runtime_enabled_features.json5:4158-4159: no enabled status.
static WARN_ON_CONTENT_VISIBILITY_RENDER_ACCESS_ENABLED: AtomicBool = AtomicBool::new(false);
static MOBILE_LAYOUT_THEME_ENABLED: AtomicBool = AtomicBool::new(false);
// DocumentPolicyNegotiation is experimental at the fixed revision. The
// runtime owner can explicitly admit it; iframe policy is not hard-disabled.
static DOCUMENT_POLICY_NEGOTIATION_ENABLED: AtomicBool = AtomicBool::new(false);

static CSS_TEXT_DECORATION_INSET_ENABLED: AtomicBool = AtomicBool::new(false);
// Pinned runtime_enabled_features.json5: CSSCaretColorWithOptionalSecondValue
// is experimental. Parser, Layout and Paint share this one runtime owner.
static CSS_CARET_COLOR_WITH_OPTIONAL_SECOND_VALUE_ENABLED: AtomicBool = AtomicBool::new(false);

// C++ class RuntimeEnabledFeatures -> Rust struct + impl.
pub struct RuntimeEnabledFeatures;
static MIDDLE_CLICK_AUTOSCROLL: AtomicBool = AtomicBool::new(false);
// Pinned runtime_enabled_features.json5:4824-4825,6315-6316 (stable).
static PRESERVE_HTML_EQUIVALENT_TAGS_IN_TYPING_STYLE_ENABLED: AtomicBool = AtomicBool::new(true);
static TEXT_SCALE_META_TAG_ENABLED: AtomicBool = AtomicBool::new(true);

impl RuntimeEnabledFeatures {
    // Pinned runtime_enabled_features.json5:4597-4600, stable; PagePopup is
    // absent on Android/iOS at the same source dependency gate.
    pub const fn PagePopupCopyPasteEnabled() -> bool {
        !cfg!(any(target_os = "android", target_os = "ios"))
    }
    // runtime_enabled_features.json5:4149-4151, test-only default.
    pub fn MiddleClickAutoscrollEnabled() -> bool {
        MIDDLE_CLICK_AUTOSCROLL.load(Ordering::Relaxed)
    }
    pub fn SetMiddleClickAutoscrollEnabled(value: bool) {
        MIDDLE_CLICK_AUTOSCROLL.store(value, Ordering::Relaxed);
    }
    pub fn CSSCaretColorWithOptionalSecondValueEnabled() -> bool {
        CSS_CARET_COLOR_WITH_OPTIONAL_SECOND_VALUE_ENABLED.load(Ordering::Relaxed)
    }
    pub fn SetCSSCaretColorWithOptionalSecondValueEnabled(enabled: bool) {
        CSS_CARET_COLOR_WITH_OPTIONAL_SECOND_VALUE_ENABLED.store(enabled, Ordering::Relaxed);
    }
    // Chromium runtime_enabled_features.json5:5615-5617 (stable).
    pub const fn SeparateDeferModuleScriptTasksEnabled() -> bool {
        true
    }

    pub fn PreserveHtmlEquivalentTagsInTypingStyleEnabled() -> bool {
        PRESERVE_HTML_EQUIVALENT_TAGS_IN_TYPING_STYLE_ENABLED.load(Ordering::Relaxed)
    }
    pub fn SetPreserveHtmlEquivalentTagsInTypingStyleEnabled(value: bool) {
        PRESERVE_HTML_EQUIVALENT_TAGS_IN_TYPING_STYLE_ENABLED.store(value, Ordering::Relaxed);
    }
    pub fn TextScaleMetaTagEnabled() -> bool {
        TEXT_SCALE_META_TAG_ENABLED.load(Ordering::Relaxed)
    }
    pub fn SetTextScaleMetaTagEnabled(value: bool) {
        TEXT_SCALE_META_TAG_ENABLED.store(value, Ordering::Relaxed);
    }
    /// Pinned runtime_enabled_features.json5:5852-5857 (stable).
    pub const fn SpatNavUsesCursorInheritanceEnabled() -> bool {
        true
    }

    // runtime_enabled_features.json5:6485-6490, stable.
    pub const fn TrustedTypesHTMLEnabled() -> bool {
        true
    }

    pub fn WarnOnContentVisibilityRenderAccessEnabled() -> bool {
        WARN_ON_CONTENT_VISIBILITY_RENDER_ACCESS_ENABLED.load(Ordering::Relaxed)
    }
    pub fn SetWarnOnContentVisibilityRenderAccessEnabled(enabled: bool) {
        WARN_ON_CONTENT_VISIBILITY_RENDER_ACCESS_ENABLED.store(enabled, Ordering::Relaxed);
    }

    // Pinned runtime_enabled_features.json5:4411-4418: stable.
    pub const fn OpaqueRangeEnabled() -> bool {
        true
    }
    pub fn DocumentPolicyNegotiationEnabled() -> bool {
        DOCUMENT_POLICY_NEGOTIATION_ENABLED.load(Ordering::Relaxed)
    }
    pub fn SetDocumentPolicyNegotiationEnabled(enabled: bool) {
        DOCUMENT_POLICY_NEGOTIATION_ENABLED.store(enabled, Ordering::Relaxed);
    }
    pub const fn DocumentPolicySyncXHREnabled() -> bool {
        false
    }
    pub const fn DocumentPolicyExpectNoLinkedResourcesEnabled() -> bool {
        true
    }
    pub const fn DocumentPolicyNetworkEfficiencyGuardrailsEnabled() -> bool {
        false
    }
    pub const fn DocumentPolicyJSProfilingModeEnabled() -> bool {
        false
    }
    // runtime_enabled_features.json5:4314-4315, stable product admission.
    pub const fn NonEmptyVisibleTextSelectionForTextFragmentEnabled() -> bool {
        true
    }
    pub fn MobileLayoutThemeEnabled() -> bool {
        MOBILE_LAYOUT_THEME_ENABLED.load(Ordering::Relaxed)
    }
    pub fn SetMobileLayoutThemeEnabled(enabled: bool) {
        MOBILE_LAYOUT_THEME_ENABLED.store(enabled, Ordering::Relaxed);
    }

    // runtime_enabled_features.json5:3823-3824 stable;805-806 experimental.
    pub const fn LazyLoadVideoAndAudioEnabled() -> bool {
        true
    }
    pub const fn AutoPictureInPictureVideoHeuristicsEnabled() -> bool {
        false
    }
    // runtime_enabled_features.json5:3019-3024 (stable).
    pub const fn FindBufferCollapseSkippedSpaceEnabled() -> bool {
        true
    }
    pub const fn FindBufferMatchAcrossIgnoredNodesEnabled() -> bool {
        true
    }
    pub fn GenerateDragOverlayBeforeDragStartEnabled() -> bool {
        GENERATE_DRAG_OVERLAY_BEFORE_DRAG_START_ENABLED.load(Ordering::Relaxed)
    }
    pub fn SetGenerateDragOverlayBeforeDragStartEnabled(enabled: bool) {
        GENERATE_DRAG_OVERLAY_BEFORE_DRAG_START_ENABLED.store(enabled, Ordering::Relaxed);
    }

    // runtime_enabled_features.json5:5422-5423,6288-6292: stable.
    pub const fn ScrollToTextFragmentUniqueFragmentsEnabled() -> bool {
        true
    }
    pub const fn TextFragmentIdentifiersEnabled() -> bool {
        true
    }

    // runtime_enabled_features.json5:2163-2165,3523-3525,3673-3677,3779-3781.
    pub const fn DateTimeInputTypeEarlyAdvanceFixEnabled() -> bool {
        true
    }
    pub const fn HTMLInputElementDropWebkitClearButtonEnabled() -> bool {
        true
    }
    pub const fn InputMultipleFieldsUIEnabled() -> bool {
        !cfg!(any(target_os = "android", target_os = "ios"))
    }
    pub const fn LangAttributeAwareFormControlUIEnabled() -> bool {
        false
    }

    // runtime_enabled_features.json5:2422-2427, Android off/default stable.
    pub const fn DocumentPictureInPictureAPIEnabled() -> bool {
        !cfg!(target_os = "android")
    }
    // runtime_enabled_features.json5:4068-4070,6654-6655: no enabled status.
    pub const fn MediaEngagementBypassAutoplayPoliciesEnabled() -> bool {
        false
    }
    pub const fn VideoAutoFullscreenEnabled() -> bool {
        false
    }
    // runtime_enabled_features.json5:3827-3833: Android stable, elsewhere off.
    pub const fn LeftClickToHandleSuggestionEnabled() -> bool {
        cfg!(target_os = "android")
    }

    // runtime_enabled_features.json5:4964-4970.
    pub const fn RasterInducingScrollEnabled() -> bool {
        cfg!(any(target_os = "linux", target_os = "windows"))
    }

    // Chromium runtime_enabled_features.json5 at revision 6c1d401f:
    // 2499-2500,3496-3497,3511-3512,4588-4589,5685-5687.
    pub const fn DOMActivateBubblesInheritanceEnabled() -> bool {
        true
    }
    pub const fn HTMLCommandActionsV2Enabled() -> bool {
        false
    }
    pub const fn HTMLCommandForScrollCommandsEnabled() -> bool {
        false
    }
    pub const fn OverscrollGesturesEnabled() -> bool {
        false
    }
    pub const fn ShadowRootReferenceTargetEnabled() -> bool {
        true
    }
    // 2143-2145,2247-2252,3724-3725,3850-3851,4129-4130,
    // 4583-4584,4716-4717,4725-4726,4060-4061.
    pub const fn CustomizableComboboxEnabled() -> bool {
        false
    }
    pub const fn DialogCloseWhenOpenRemovedEnabled() -> bool {
        false
    }
    pub const fn DialogNewFocusBehaviorEnabled() -> bool {
        false
    }
    pub const fn InterestEventsNonComposedEnabled() -> bool {
        true
    }
    pub const fn LightDismissFromClickEnabled() -> bool {
        false
    }
    pub const fn MenuElementsEnabled() -> bool {
        false
    }
    pub const fn OverlayPropertyEnabled() -> bool {
        true
    }
    pub const fn PopoverHintNestedShowExceptionEnabled() -> bool {
        true
    }
    pub const fn PopoverHintNewBehaviorEnabled() -> bool {
        true
    }
    pub const fn MediaElementMutedDefaultStateEnabled() -> bool {
        false
    }

    // runtime_enabled_features.json5:5159-5160: stable.
    pub const fn RespectOverscrollBehaviorForScrollBubblingEnabled() -> bool {
        true
    }
    // runtime_enabled_features.json5:1934-1941: stable.
    pub const fn CSSScrollSnapChangeEventEnabled() -> bool {
        true
    }
    pub const fn CSSScrollSnapChangingEventEnabled() -> bool {
        true
    }

    // Source runtime_enabled_features.json5:948-950,4914-4915,6002-6003,
    // 4709-4710: stable; 6116-6117: experimental;3945:no enabled status.
    pub const fn BoundaryEventDispatchTracksNodeRemovalEnabled() -> bool {
        true
    }
    pub const fn PseudoElementsFocusableEnabled() -> bool {
        true
    }
    pub const fn SuppressPointerStreamAfterDragEnabled() -> bool {
        true
    }
    pub const fn PointerRawUpdateOnlyInSecureContextEnabled() -> bool {
        true
    }
    pub const fn SyntheticMouseHoverOverInactivePageEnabled() -> bool {
        false
    }
    pub const fn LongPressLinkSelectTextEnabled() -> bool {
        false
    }
    // Source manifest:6416-6417,6380-6382 (base_feature:none),
    // 6396-6400 (Win stable),6295-6296 (Android stable).
    pub const fn TouchTextEditingRedesignEnabled() -> bool {
        false
    }
    pub const fn TouchDragAndContextMenuEnabled() -> bool {
        false
    }
    pub const fn TouchDragOnShortPressEnabled() -> bool {
        cfg!(target_os = "windows")
    }
    pub const fn TextFragmentTapOpensContextMenuEnabled() -> bool {
        cfg!(target_os = "android")
    }

    // Chromium runtime_enabled_features.json5:3483-3487, stable kill switch.
    pub const fn HTMLAreaElementDisplayNoneEnabled() -> bool {
        true
    }

    // Chromium runtime_enabled_features.json5:2571-2572: experimental.
    pub const fn ElementCanvasTransformEnabled() -> bool {
        false
    }
    // Chromium runtime_enabled_features.json5:3260-3263 has no enabled status.
    pub const fn FractionalScrollOffsetsEnabled() -> bool {
        false
    }
    // Chromium runtime_enabled_features.json5:753-755: stable.
    pub const fn AuthorSpecifiedLayoutScrollSnapBehaviorEnabled() -> bool {
        true
    }

    // Chromium runtime_enabled_features.json5:3452-3461: stable.
    pub const fn HitTestBorderRadiusForStackingContextEnabled() -> bool {
        true
    }
    pub const fn HitTestContainerTransformStateForPreserve3dEnabled() -> bool {
        true
    }
    // Chromium runtime_enabled_features.json5:5127-5128: experimental.
    pub const fn ReplacedNormalFlowStackingInlinePaintEnabled() -> bool {
        false
    }
    pub const fn TextAreaEmptyPlaceholderBreakEnabled() -> bool {
        true
    }
    pub const fn NoExtendSelectionToUserSelectNoneOutOfFlowEnabled() -> bool {
        true
    }
    pub const fn NoExtendSelectionToUserSelectNoneOutOfFlowUnlessEditableEnabled() -> bool {
        true
    }
    pub const fn SelectionEditingBoundarySlottedContentEnabled() -> bool {
        // Chromium runtime_enabled_features.json5:5534-5535: stable.
        true
    }
    pub const fn CSSPositionStickyStaticScrollPositionEnabled() -> bool {
        // Chromium runtime_enabled_features.json5:1804-1807: test-only.
        false
    }

    pub const fn PaintUnderInvalidationCheckingEnabled() -> bool {
        // Chromium exposes this as a diagnostic runtime feature. Rechrom's
        // source compositor rejects configurations without the required
        // diagnostic raster backend, so ordinary page paint keeps it off.
        false
    }

    pub const fn AnchorPositionAdjustmentWithoutOverflowEnabled() -> bool {
        true
    }

    pub const fn AnnotationSpaceOnStartEnabled() -> bool {
        true
    }

    pub const fn AppearanceBaseEnabled() -> bool {
        false
    }

    pub const fn BaseAppearanceInlineSizingEnabled() -> bool {
        true
    }

    pub const fn BoxDecorationBreakCloneLineBreakingEnabled() -> bool {
        false
    }

    pub const fn CSSLineClampAsShorthandEnabled() -> bool {
        false
    }

    pub const fn CSSLineClampEnabled() -> bool {
        // Chromium runtime_enabled_features.json5:1719-1725: experimental/test,
        // disabled until a host ExecutionContext explicitly enables them.
        false
    }

    pub const fn CSSLineClampLineBreakingEllipsisEnabled() -> bool {
        false
    }

    pub const fn CSSListCounterAccountingEnabled() -> bool {
        false
    }

    pub fn CSSTextDecorationInsetEnabled() -> bool {
        // Chromium runtime_enabled_features.json5:2020-2023: experimental.
        CSS_TEXT_DECORATION_INSET_ENABLED.load(Ordering::Relaxed)
    }

    pub fn SetCSSTextDecorationInsetEnabled(enabled: bool) {
        CSS_TEXT_DECORATION_INSET_ENABLED.store(enabled, Ordering::Relaxed);
    }

    pub const fn CollapseZeroWidthSpaceWhenReuseItemEnabled() -> bool {
        true
    }

    pub const fn CompositeBGColorAnimationEnabled() -> bool {
        true
    }

    pub const fn CompositeClipPathAnimationEnabled() -> bool {
        true
    }

    pub const fn CssTextFitEnabled() -> bool {
        true
    }

    pub const fn CssTextFitReshapingEnabled() -> bool {
        false
    }

    // Chromium runtime_enabled_features.json5:1433-1436: stable.
    pub const fn CreateInlineContentsAnonymousBlockEnabled() -> bool {
        true
    }

    pub const fn CustomScrollbarApplyMinimumThumbLengthEnabled() -> bool {
        true
    }

    pub const fn DisableEllipsisWhenScrolledEnabled() -> bool {
        false
    }

    pub const fn DisplayContentsFocusableEnabled() -> bool {
        false
    }

    pub const fn FirstLineOnListItemEnabled() -> bool {
        true
    }

    pub const fn FirstLineTextMetricsEnabled() -> bool {
        true
    }

    pub const fn FirstLineTextTransformEnabled() -> bool {
        true
    }

    pub const fn FragmentedOofInCbEnabled() -> bool {
        false
    }

    pub const fn InlineCursorSkipNonIfcEnabled() -> bool {
        true
    }

    pub const fn LayoutIgnoreMarginsForStickyEnabled() -> bool {
        false
    }

    pub const fn LayoutOOFCollectInlinesFixEnabled() -> bool {
        true
    }

    pub const fn LayoutTableCellAlignmentSafeEnabled() -> bool {
        true
    }

    pub const fn LineBreakAfterSpaceBeforeOpenTagEnabled() -> bool {
        true
    }

    pub const fn LineBreakerHanKerningEndEnabled() -> bool {
        true
    }

    pub const fn MemoryConsumerForNGShapeCacheEnabled() -> bool {
        false
    }

    pub const fn OffsetMappingReuseFullWidthSpaceFixEnabled() -> bool {
        true
    }

    pub const fn OutlineDrawAutoStyleZeroWidthEnabled() -> bool {
        true
    }

    pub const fn ScopedViewTransitionSizeContainmentEnabled() -> bool {
        true
    }

    pub const fn ScoreLineBreakerAbortEnabled() -> bool {
        true
    }

    pub const fn SingleAxisScrollContainersEnabled() -> bool {
        false
    }

    pub const fn SingleAxisScrollContainersForScrollSnapEnabled() -> bool {
        false
    }

    pub const fn SkipOofItemForBreakCandidateEnabled() -> bool {
        true
    }

    pub const fn SmallerViewportUnitsEnabled() -> bool {
        true
    }

    pub const fn StackingContextIsNotStackedEnabled() -> bool {
        true
    }

    pub const fn SvgNewZoomEnabled() -> bool {
        false
    }

    pub const fn SvgFilterPaintsForHiddenContentEnabled() -> bool {
        // Chromium runtime_enabled_features.json5:6036-6039: stable.
        true
    }

    pub const fn SvgInlineRootPixelSnappingScaleAdjustmentEnabled() -> bool {
        // Chromium runtime_enabled_features.json5:6056-6059: test-only. The
        // production default is disabled; retained test fixtures may enable
        // the exported per-viewport fact to exercise the source branch.
        false
    }

    pub const fn TabAlignmentWithFloatsEnabled() -> bool {
        true
    }

    pub const fn TabSizeAncestorEnabled() -> bool {
        true
    }

    pub const fn TabSizeInRubyBaseEnabled() -> bool {
        true
    }

    pub const fn TabSizeWithSpacingEnabled() -> bool {
        true
    }

    pub const fn TableIsAutoFixedLayoutEnabled() -> bool {
        false
    }

    pub const fn TextAutoSpaceIgnoreRubyAnnotationEnabled() -> bool {
        true
    }

    pub const fn TextAreaResizerFixedSizeEnabled() -> bool {
        // Chromium runtime_enabled_features.json5:6241-6244: stable.
        true
    }

    pub const fn TextBoxTrimForNestedListEnabled() -> bool {
        true
    }

    pub const fn TextEmphasisAsRubyEnabled() -> bool {
        true
    }

    pub const fn TextEmphasisPositionAutoEnabled() -> bool {
        false
    }

    pub const fn TextEmphasisWithRubyEnabled() -> bool {
        true
    }

    pub const fn TextOverflowClipWithSelectionEnabled() -> bool {
        true
    }

    pub const fn TextOverflowStringEnabled() -> bool {
        false
    }

    pub const fn TreeRubyPlacementEnabled() -> bool {
        true
    }

    pub const fn UnboundedElementEnabled() -> bool {
        // The embedding does not provide unbounded surface activation.
        false
    }

    pub const fn UsePositionForPointInFlexibleBoxWithSingleChildElementEnabled() -> bool {
        true
    }

    pub const fn WordSpacingWhiteSpacePreEnabled() -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_decoration_inset_retains_experimental_default_and_override() {
        RuntimeEnabledFeatures::SetCSSTextDecorationInsetEnabled(false);
        assert!(!RuntimeEnabledFeatures::CSSTextDecorationInsetEnabled());
        RuntimeEnabledFeatures::SetCSSTextDecorationInsetEnabled(true);
        assert!(RuntimeEnabledFeatures::CSSTextDecorationInsetEnabled());
        RuntimeEnabledFeatures::SetCSSTextDecorationInsetEnabled(false);
    }
}

impl RuntimeEnabledFeatures {
    // runtime_enabled_features.json5:313-316: test only, no base feature.
    pub const fn AdTaggingEnabled() -> bool {
        false
    }
}

impl RuntimeEnabledFeatures {
    // runtime_enabled_features.json5:4077-4080, test / origin-trial only.
    pub const fn MediaPlaybackWhileNotVisiblePermissionPolicyEnabled() -> bool {
        false
    }
}

impl RuntimeEnabledFeatures {
    /// runtime_enabled_features.json5:4304, stable at the pinned revision.
    pub fn NoNbspForInterElementSpaceOnCopyEnabled() -> bool {
        true
    }
    /// runtime_enabled_features.json5:7409-7410, test-only and not enabled by
    /// the browser product's feature admission.
    pub fn XMLSerializerConsistentDefaultNsDeclMatchingEnabled() -> bool {
        false
    }
}

impl RuntimeEnabledFeatures {
    // Pinned form-control feature statuses from runtime_enabled_features.json5.
    pub const fn ClampUnfocusedSelectionCacheEnabled() -> bool {
        true
    }
    pub const fn DispatchSelectionchangeEventPerElementEnabled() -> bool {
        true
    }
    pub const fn InputDisabledHandlerFixEnabled() -> bool {
        true
    }
    pub const fn SanitizeIDNEmailFormInputEnabled() -> bool {
        true
    }
    pub const fn NumberInputFullWidthCharsEnabled() -> bool {
        true
    }
    pub const fn NonStandardAppearanceValueSliderVerticalEnabled() -> bool {
        true
    }
    pub const fn DisableFormControlChangeEventDuringMutationEnabled() -> bool {
        true
    }
    pub const fn CSSUserValidAndUserInvalidForRadioEnabled() -> bool {
        true
    }
    pub const fn PasswordRevealEnabled() -> bool {
        false
    }
    pub const fn EmailVerificationStatusIndicatorEnabled() -> bool {
        false
    }
    pub const fn SelectedcontentSpecEnabled() -> bool {
        true
    }
    pub const fn SelectedcontentMultipleEnabled() -> bool {
        false
    }
    pub const fn SelectedcontentelementAttributeEnabled() -> bool {
        false
    }
    pub const fn FilterableSelectEnabled() -> bool {
        false
    }
    pub const fn InputInSelectEnabled() -> bool {
        false
    }
    pub const fn SelectAutofillPopoverPreviewEnabled() -> bool {
        true
    }
    pub const fn CustomizableSelectMultiplePopupEnabled() -> bool {
        false
    }
    pub const fn SelectRemoveOverflowHiddenEnabled() -> bool {
        true
    }
    pub const fn UnclosedFormControlIsInvalidEnabled() -> bool {
        false
    }
    pub const fn WebMCPEnabled() -> bool {
        false
    }
}

impl RuntimeEnabledFeatures {
    /// runtime_enabled_features.json5:7369-7370, stable.
    pub fn WordSkipSpacesPunctuationFixEnabled() -> bool {
        true
    }
    /// runtime_enabled_features.json5:5543-5544, stable.
    pub fn SelectionFocusAffinityEnabled() -> bool {
        true
    }
    /// runtime_enabled_features.json5:4161-4162, stable (source spelling).
    pub fn ModifyParagraphCrossEditingoundaryEnabled() -> bool {
        true
    }
    /// runtime_enabled_features.json5:904-906 has no stable status.
    pub fn BidiVisualOrderCaretMovementEnabled() -> bool {
        false
    }
}

impl RuntimeEnabledFeatures {
    /// runtime_enabled_features.json5:3636-3637, stable.
    pub fn InlineBlockLineNavigationEnabled() -> bool {
        true
    }
}

impl RuntimeEnabledFeatures {
    /// runtime_enabled_features.json5:6640-6641, stable.
    pub fn UseUndoStepElementDispatchBeforeInputEnabled() -> bool {
        true
    }
    /// runtime_enabled_features.json5:6175-6176, stable.
    pub fn TargetRangesForBackwardDeletionUnitEnabled() -> bool {
        true
    }
}

impl RuntimeEnabledFeatures {
    // Pinned runtime_enabled_features.json5:3008-3010,5910-5911: stable.
    pub fn FilterContainerLevelStylesEnabled() -> bool {
        true
    }
    pub fn SplitTextNotCleanupDummySpansEnabled() -> bool {
        true
    }
}

// runtime_enabled_features.json5:7240-7253, fixed revision.
static WEB_VTT_REGIONS_ENABLED: AtomicBool = AtomicBool::new(false);
static WEB_VTT_CUE_LAYOUT_BY_POSITION_ALIGNMENT_ENABLED: AtomicBool = AtomicBool::new(true);
impl RuntimeEnabledFeatures {
    pub fn WebVTTRegionsEnabled() -> bool {
        WEB_VTT_REGIONS_ENABLED.load(Ordering::Relaxed)
    }
    pub fn SetWebVTTRegionsEnabled(value: bool) {
        WEB_VTT_REGIONS_ENABLED.store(value, Ordering::Relaxed);
    }
    pub fn WebVTTCueLayoutByPositionAlignmentEnabled() -> bool {
        WEB_VTT_CUE_LAYOUT_BY_POSITION_ALIGNMENT_ENABLED.load(Ordering::Relaxed)
    }
    pub fn SetWebVTTCueLayoutByPositionAlignmentEnabled(value: bool) {
        WEB_VTT_CUE_LAYOUT_BY_POSITION_ALIGNMENT_ENABLED.store(value, Ordering::Relaxed);
    }
}

impl RuntimeEnabledFeatures {
    // Pinned runtime_enabled_features.json5:4198,4336,4343,5079: stable.
    pub fn MoveParagraphsPreserveInlineStructureEnabled() -> bool {
        true
    }
    pub fn NormalizeNbspForPasteAndDropEnabled() -> bool {
        true
    }
    pub fn NormalizeNbspRichTextOnlyEnabled() -> bool {
        true
    }
    pub fn RemoveCollapsedPlaceholderForContentEditableEnabled() -> bool {
        true
    }
}

impl RuntimeEnabledFeatures {
    // Pinned runtime_enabled_features.json5:5115-5116, test-only.
    pub fn RemoveVisibleSelectionInDOMSelectionEnabled() -> bool {
        false
    }
}

impl RuntimeEnabledFeatures {
    // Pinned runtime_enabled_features.json5:2201-2202,3957-3958 stable.
    pub fn DelegatesFocusTextControlInputFixEnabled() -> bool {
        true
    }
    pub fn MacDisableCtrlHomeEndEnabled() -> bool {
        true
    }
}

// Pinned runtime_enabled_features.json5:2663-2664,3316-3317,6581-6582.
static ENTROPY_IGNORED_FOR_FIRST_VIDEO_FRAME_LCP: AtomicBool = AtomicBool::new(false);
static GEOMETRY_MAPPER_SINGULAR_TRANSFORM_FIX: AtomicBool = AtomicBool::new(true);
static USE_LARGEST_PAINTED_IMAGE_FOR_LCP_CANDIDATE: AtomicBool = AtomicBool::new(true);
impl RuntimeEnabledFeatures {
    pub fn EntropyIgnoredForFirstVideoFrameLCPEnabled() -> bool {
        ENTROPY_IGNORED_FOR_FIRST_VIDEO_FRAME_LCP.load(Ordering::Relaxed)
    }
    pub fn SetEntropyIgnoredForFirstVideoFrameLCPEnabled(value: bool) {
        ENTROPY_IGNORED_FOR_FIRST_VIDEO_FRAME_LCP.store(value, Ordering::Relaxed);
    }
    pub fn GeometryMapperSingularTransformFixEnabled() -> bool {
        GEOMETRY_MAPPER_SINGULAR_TRANSFORM_FIX.load(Ordering::Relaxed)
    }
    pub fn SetGeometryMapperSingularTransformFixEnabled(value: bool) {
        GEOMETRY_MAPPER_SINGULAR_TRANSFORM_FIX.store(value, Ordering::Relaxed);
    }
    pub fn UseLargestPaintedImageForLCPCandidateEnabled() -> bool {
        USE_LARGEST_PAINTED_IMAGE_FOR_LCP_CANDIDATE.load(Ordering::Relaxed)
    }
    pub fn SetUseLargestPaintedImageForLCPCandidateEnabled(value: bool) {
        USE_LARGEST_PAINTED_IMAGE_FOR_LCP_CANDIDATE.store(value, Ordering::Relaxed);
    }
}

// runtime_enabled_features.json5:6341-6343, status test (disabled in production).
static THROTTLED_HISTORY_API_THROWS_SECURITY_ERROR: AtomicBool = AtomicBool::new(false);
impl RuntimeEnabledFeatures {
    pub fn ThrottledHistoryAPIThrowsSecurityErrorEnabled() -> bool {
        THROTTLED_HISTORY_API_THROWS_SECURITY_ERROR.load(Ordering::Relaxed)
    }
    pub fn SetThrottledHistoryAPIThrowsSecurityErrorEnabled(value: bool) {
        THROTTLED_HISTORY_API_THROWS_SECURITY_ERROR.store(value, Ordering::Relaxed);
    }
}

// Pinned runtime_enabled_features.json5:5822-5837. Detection's generated
// base::Feature and both runtime features are enabled by default; each has
// its own original switch (the base flag is the field-trial kill switch).
static SOFT_NAVIGATION_DETECTION_BASE_FEATURE: AtomicBool = AtomicBool::new(true);
static SOFT_NAVIGATION_INCLUDE_REPLACE_STATE: AtomicBool = AtomicBool::new(true);
static SOFT_NAVIGATION_HEURISTICS: AtomicBool = AtomicBool::new(true);
impl RuntimeEnabledFeatures {
    pub fn SoftNavigationDetectionBaseFeatureEnabled() -> bool {
        SOFT_NAVIGATION_DETECTION_BASE_FEATURE.load(Ordering::Relaxed)
    }
    pub fn SetSoftNavigationDetectionBaseFeatureEnabled(value: bool) {
        SOFT_NAVIGATION_DETECTION_BASE_FEATURE.store(value, Ordering::Relaxed);
    }
    pub fn SoftNavigationDetectionIncludeReplaceStateEnabled() -> bool {
        SOFT_NAVIGATION_INCLUDE_REPLACE_STATE.load(Ordering::Relaxed)
    }
    pub fn SetSoftNavigationDetectionIncludeReplaceStateEnabled(value: bool) {
        SOFT_NAVIGATION_INCLUDE_REPLACE_STATE.store(value, Ordering::Relaxed);
    }
    pub fn SoftNavigationHeuristicsEnabled() -> bool {
        SOFT_NAVIGATION_HEURISTICS.load(Ordering::Relaxed)
    }
    pub fn SetSoftNavigationHeuristicsEnabled(value: bool) {
        SOFT_NAVIGATION_HEURISTICS.store(value, Ordering::Relaxed);
    }
}

// Pinned runtime_enabled_features.json5:1359-1362 (experimental/origin trial).
static CONTAINER_TIMING_ENABLED: AtomicBool = AtomicBool::new(false);
// runtime_enabled_features.json5:2555-2556, source status "test".
static EDITING_USE_DOM_POSITION_API: AtomicBool = AtomicBool::new(false);
impl RuntimeEnabledFeatures {
    pub fn EditingUseDomPositionApiEnabled() -> bool {
        EDITING_USE_DOM_POSITION_API.load(Ordering::Relaxed)
    }
    pub fn SetEditingUseDomPositionApiEnabled(value: bool) {
        EDITING_USE_DOM_POSITION_API.store(value, Ordering::Relaxed);
    }
    pub fn ContainerTimingEnabled() -> bool {
        CONTAINER_TIMING_ENABLED.load(Ordering::Relaxed)
    }
    pub fn SetContainerTimingEnabled(value: bool) {
        CONTAINER_TIMING_ENABLED.store(value, Ordering::Relaxed);
    }
}
