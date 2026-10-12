// C++: src/foundation/layout_features/runtime_enabled_features.h

// C++ class RuntimeEnabledFeatures -> Rust struct + impl.
pub struct RuntimeEnabledFeatures;

impl RuntimeEnabledFeatures {
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
        true
    }

    pub const fn UsePositionForPointInFlexibleBoxWithSingleChildElementEnabled() -> bool {
        true
    }

    pub const fn WordSpacingWhiteSpacePreEnabled() -> bool {
        true
    }
}
