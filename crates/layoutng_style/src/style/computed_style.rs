use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use font_engine::{
    Font, FontDescription, FontHeight, FontPalette, FontSelectionValue, FontSizeAdjust,
    FontVariantEmoji, LayoutLocale,
};
use foundation::{
    g_null_atom, AtomicString, BlendMode, CSSBitset, CSSPropertyID, Color, EAspectRatioType,
    EBorderCollapse, EBorderStyle, EBoxDirection, EBoxOrient, EBoxSizing, EClear,
    EContentVisibility, EContinue, EDisplay, EFlexDirection, EFloat, EForcedColorAdjust,
    EFrameSizing, EGridLanesPack, EInsideLink, EInternalOverscrollContainer,
    EInternalOverscrollPosition, EInternalUnbounded, EIsolation, EOverflow, EOverflowWrap,
    EOverlay, EOverscrollContainerType, EPageMarginSafety, EPointerEvents, EPosition, EResize,
    EScrollTargetGroup, EScrollbarWidth, ETableLayout, ETextBoxTrim, ETextCombine,
    ETransformStyle3D, EUserModify, EUserSelect, EVisibility, EWordBreak, GCedHeapHashSet, HashSet,
    LayoutUnit, Length, LengthBox, LengthPoint, LineBreak, MakeGarbageCollected, Member,
    PhysicalToLogical, RuntimeEnabledFeatures, String, TabSize, TabSizeValueType, TextDirection,
    VectorExt, Visitor, WeakMember, WritingDirectionMode,
};

use crate::css::color_scheme_flags::ColorSchemeFlags;
use crate::css::style_color::StyleColor;
use crate::css::white_space::{self, EWhiteSpace};
use layoutng_geometry::geometry::box_sides::PhysicalBoxSides;
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_geometry::geometry::logical_size::{LogicalSize, ToLogicalSize};

use super::appearance::AppearanceValue;
use super::border_edge::BorderEdgeArray;
use super::border_image_length_box::BorderImageLengthBox;
use super::clip_path_operation::ClipPathOperation;
use super::color_scheme::mojom;
use super::computed_grid_track_list::ComputedGridTrackList;
use super::computed_style_base::{ComputedStyleBase, ComputedStyleBuilderBase};
use super::computed_style_constants::{
    Containment, EContainerType, EFillBox, EVerticalAlign, FlexWrapMode, GridAutoFlow,
    InternalGridAutoFlowAlgorithm, InternalGridAutoFlowDirection, ItemPosition, PositionVisibility,
    PseudoId, PseudoIdFlags, ScrollbarGutter, ViewportUnitFlag,
};
use super::computed_style_initial_values::ComputedStyleInitialValues;
use super::content_data::ContentData;
use super::counter_directives::{CounterDirectiveMap, CounterDirectives};
use super::cursor_data::CursorData;
use super::cursor_list::CursorList;
use super::default_anchor_data::DefaultAnchorData;
use super::display_style::DisplayStyle;
use super::fill_layer::FillLayer;
use super::filter_operations::FilterOperationVector;
use super::font_size_style::FontSizeStyle;
use super::forward::{
    CSSAnimationData, CSSTransitionData, CSSValue, CSSVariableData, Element, Longhand, Node,
    StyleRule,
};
use super::gap_data_list::GapDataList;
use super::grid_enums::GridTrackSizingDirection;
use super::grid_lanes_direction::GridLanesOrientation;
use super::grid_position::GridPosition;
use super::grid_track_list::GridTrackList;
use super::nine_piece_image::NinePieceImage;
use super::outline_type::OutlineType;
use super::scroll_marker_group::ScrollMarkerMode;
use super::shadow_list::ShadowList;
use super::shape_value::ShapeValue;
use super::style_cached_data::{PseudoElementStyleCache, StyleCachedData};
use super::style_highlight_data::StyleHighlightData;
use super::style_image::StyleImage;
use super::style_inherited_variables::StyleInheritedVariables;
use super::style_intrinsic_length::StyleIntrinsicLength;
use super::style_non_inherited_variables::StyleNonInheritedVariables;
use super::style_overflow_clip_margin::ReferenceBox as OverflowClipMarginReferenceBox;
use super::style_position_anchor::Type as StylePositionAnchorType;
use super::style_scrollbar_color::StyleScrollbarColor;
use super::svg_paint::SVGPaint;
use super::text_indent_flags::TextIndentFlags;
use super::transform_origin::TransformOrigin;

// The C++ derived object begins with its ComputedStyleBase subobject.
// cpp: layoutng_style/style/computed_style.h:228
// cpp: layoutng_style/style/computed_style.h:317-318
#[repr(C)]
pub struct ComputedStyle {
    pub(crate) base_: ComputedStyleBase,
    pub(crate) cached_data_: Cell<Member<StyleCachedData>>,
}

// cpp: layoutng_style/style/computed_style.h:347-350
impl foundation::Traceable for ComputedStyle {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.TraceAfterDispatch(visitor);
    }
}

// cpp: layoutng_style/style/computed_style.h:2913-2914
// cpp: layoutng_style/style/computed_style.h:2937-2940
// cpp: layoutng_style/style/computed_style.h:3643-3645
pub struct ComputedStyleBuilder {
    pub(crate) base_: ComputedStyleBuilderBase,
    pub(crate) has_own_animations_: Cell<bool>,
    pub(crate) has_own_transitions_: Cell<bool>,
}

// C++ constrains the Property template through its default constructor,
// Longhand inheritance, and color-resolution member. The concrete property
// types belong to the pending foundation style-values interface.
// cpp: layoutng_style/style/computed_style.h:2426-2429
#[allow(non_snake_case)]
pub trait VisitedDependentColorProperty: Default + AsRef<Longhand> {
    fn IsVisited(&self) -> bool;
    fn ColorIncludingFallback(
        &self,
        visited_link: bool,
        style: &ComputedStyle,
        is_current_color: Option<&mut bool>,
    ) -> Color;
}

// C++ nests this enum in ComputedStyle; Rust keeps the owner in its name.
// cpp: layoutng_style/style/computed_style.h:375-409
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComputedStyleDifference {
    kEqual,
    kNonInherited,
    kPseudoElementStyle,
    kIndependentInherited,
    kInherited,
    kDescendantAffecting,
}

// C++ nests these transform switches in ComputedStyle. Rust names include
// their owner to keep them distinct from method and module names.
// cpp: layoutng_style/style/computed_style.h:2098-2101
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputedStyleApplyTransformOrigin {
    kIncludeTransformOrigin,
    kExcludeTransformOrigin,
}

// cpp: layoutng_style/style/computed_style.h:2102
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputedStyleApplyMotionPath {
    kIncludeMotionPath,
    kExcludeMotionPath,
}

// cpp: layoutng_style/style/computed_style.h:2103-2106
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputedStyleApplyIndependentTransformProperties {
    kIncludeIndependentTransformProperties,
    kExcludeIndependentTransformProperties,
}

// cpp: layoutng_style/style/computed_style.h:2107-2110
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputedStyleApplyTransformOperations {
    kIncludeTransformOperations,
    kExcludeTransformOperations,
}

// cpp: layoutng_style/style/computed_style.h:2126-2129
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputedStyleTransformBoxContext {
    kLayoutBox,
    kSvg,
}

// cpp: layoutng_style/style/computed_style.h:2887-2905
// Preserve the six compile-time bounds checks before testing highlight bits.
const _: () = {
    const FIRST: u8 = PseudoId::kFirstPublicPseudoId.value();
    const LAST: u8 = PseudoId::kLastTrackedPublicPseudoId.value();
    assert!(
        PseudoId::kPseudoIdSelection.value() >= FIRST
            && PseudoId::kPseudoIdSelection.value() <= LAST
    );
    assert!(
        PseudoId::kPseudoIdSearchText.value() >= FIRST
            && PseudoId::kPseudoIdSearchText.value() <= LAST
    );
    assert!(
        PseudoId::kPseudoIdTargetText.value() >= FIRST
            && PseudoId::kPseudoIdTargetText.value() <= LAST
    );
    assert!(
        PseudoId::kPseudoIdSpellingError.value() >= FIRST
            && PseudoId::kPseudoIdSpellingError.value() <= LAST
    );
    assert!(
        PseudoId::kPseudoIdGrammarError.value() >= FIRST
            && PseudoId::kPseudoIdGrammarError.value() <= LAST
    );
    assert!(
        PseudoId::kPseudoIdHighlight.value() >= FIRST
            && PseudoId::kPseudoIdHighlight.value() <= LAST
    );
};

// C++ inheritance exposes the generated base interface to ComputedStyle.
// cpp: layoutng_style/style/computed_style.h:228
impl Deref for ComputedStyle {
    type Target = ComputedStyleBase;

    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}

// cpp: layoutng_style/style/computed_style.h:2913
impl Deref for ComputedStyleBuilder {
    type Target = ComputedStyleBuilderBase;

    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}

// cpp: layoutng_style/style/computed_style.h:2913
impl DerefMut for ComputedStyleBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_
    }
}

// cpp: layoutng_style/style/computed_style.h:933-936
impl PartialEq for ComputedStyle {
    fn eq(&self, other: &Self) -> bool {
        self.InheritedEqual(other)
            && self.NonInheritedEqual(other)
            && self.InheritedVariablesEqual(other)
    }
}

#[allow(non_snake_case)]
impl ComputedStyle {
    // These declarations have no definition in the supplied style sources.
    // cpp: layoutng_style/style/computed_style.h:728
    pub fn UsedScrollbarColor(&self) -> *mut StyleScrollbarColor {
        unsafe { ComputedStyleUsedScrollbarColor(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:778
    pub fn ListStyleStringValue(&self) -> &AtomicString {
        unsafe { &*ComputedStyleListStyleStringValue(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:787-789
    pub fn MarkerShouldBeInside(&self, parent: &Element, marker_style: &DisplayStyle<'_>) -> bool {
        unsafe { ComputedStyleMarkerShouldBeInside(self, parent, marker_style) }
    }

    // cpp: layoutng_style/style/computed_style.h:1459
    pub fn CanRenderBorderImage(&self) -> bool {
        unsafe { ComputedStyleCanRenderBorderImage(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:2548
    pub fn IsRenderedInTopLayer(&self, element: &Element) -> bool {
        unsafe { ComputedStyleIsRenderedInTopLayer(self, element) }
    }

    // cpp: layoutng_style/style/computed_style.h:821
    pub fn HighlightPseudoElementStylesDependOnRelativeUnits(&self) -> bool {
        unsafe { ComputedStyleHighlightPseudoElementStylesDependOnRelativeUnits(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:822
    pub fn HighlightPseudoElementStylesDependOnContainerUnits(&self) -> bool {
        unsafe { ComputedStyleHighlightPseudoElementStylesDependOnContainerUnits(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:823
    pub fn HighlightPseudoElementStylesHaveVariableReferences(&self) -> bool {
        unsafe { ComputedStyleHighlightPseudoElementStylesHaveVariableReferences(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:2735
    pub fn HasPropertyDependingOnCurrentColor(&self) -> bool {
        unsafe { ComputedStyleHasPropertyDependingOnCurrentColor(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:2867
    pub(crate) fn ScrollbarIsHiddenByCustomStyle(&self, element: *mut Element) -> bool {
        unsafe { ComputedStyleScrollbarIsHiddenByCustomStyle(self, element) }
    }

    // C++ FunctionRef also borrows its callable and permits captures.
    // cpp: layoutng_style/style/computed_style.h:819
    pub fn DependsOnFunc(&self, func: &dyn Fn(&ComputedStyle) -> bool) -> bool {
        unsafe { ComputedStyleDependsOnFunc(self, func) }
    }

    // cpp: layoutng_style/style/computed_style.h:2762
    pub(crate) fn DecorationColorIncludingFallback(&self, visited_link: bool) -> StyleColor {
        unsafe { ComputedStyleDecorationColorIncludingFallback(self, visited_link) }
    }

    // cpp: layoutng_style/style/computed_style.h:2783
    pub(crate) fn DiffNeedsReshape(&self, other: &Self, field_diff: u64) -> bool {
        unsafe { ComputedStyleDiffNeedsReshape(self, other, field_diff) }
    }

    // cpp: layoutng_style/style/computed_style.h:2784-2785
    pub(crate) fn DiffNeedsFullLayoutAndPaintInvalidation(
        &self,
        other: &Self,
        field_diff: u64,
    ) -> bool {
        unsafe { ComputedStyleDiffNeedsFullLayoutAndPaintInvalidation(self, other, field_diff) }
    }

    // cpp: layoutng_style/style/computed_style.h:2786-2787
    pub(crate) fn DiffNeedsRecomputeVisualOverflow(&self, other: &Self, field_diff: u64) -> bool {
        unsafe { ComputedStyleDiffNeedsRecomputeVisualOverflow(self, other, field_diff) }
    }

    // cpp: layoutng_style/style/computed_style.h:2788-2789
    pub(crate) fn DiffCompositingReasonsChanged(&self, other: &Self, field_diff: u64) -> bool {
        unsafe { ComputedStyleDiffCompositingReasonsChanged(self, other, field_diff) }
    }

    // cpp: layoutng_style/style/computed_style.h:2790-2791
    pub(crate) fn PotentialCompositingReasonsFor3DTransformChanged(&self, other: &Self) -> bool {
        unsafe { ComputedStylePotentialCompositingReasonsFor3DTransformChanged(self, other) }
    }

    // cpp: layoutng_style/style/computed_style.h:2793-2794
    pub(crate) fn PropertiesEqual(
        &self,
        properties: &foundation::Vector<CSSPropertyID>,
        other: &Self,
    ) -> bool {
        unsafe { ComputedStylePropertiesEqual(self, properties, other) }
    }

    // cpp: layoutng_style/style/computed_style.h:2795-2796
    pub fn CustomPropertiesEqual(
        &self,
        properties: &foundation::Vector<AtomicString>,
        other: &Self,
    ) -> bool {
        unsafe { ComputedStyleCustomPropertiesEqual(self, properties, other) }
    }

    // cpp: layoutng_style/style/computed_style.h:2455
    pub fn GetInterpolationQuality(&self) -> foundation::InterpolationQuality {
        unsafe { ComputedStyleGetInterpolationQuality(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:2846-2848
    pub(crate) fn ComputeDifferenceIgnoringInheritedFirstLineStyle(
        old_style: &Self,
        new_style: &Self,
    ) -> ComputedStyleDifference {
        // cpp: third_party/blink/renderer/core/style/computed_style.cc:439-500
        use ComputedStyleDifference::*;
        if old_style.ScrollTimelineName() != new_style.ScrollTimelineName()
            || old_style.ScrollTimelineAxis() != new_style.ScrollTimelineAxis()
            || old_style.ViewTimelineName() != new_style.ViewTimelineName()
            || old_style.ViewTimelineAxis() != new_style.ViewTimelineAxis()
            || old_style.ViewTimelineInset() != new_style.ViewTimelineInset()
            || old_style.TimelineScope() != new_style.TimelineScope()
        {
            return kDescendantAffecting;
        }
        if old_style.Display() != new_style.Display()
            && (old_style.BlockifiesChildren() != new_style.BlockifiesChildren()
                || old_style.InlinifiesChildren() != new_style.InlinifiesChildren())
        {
            return kDescendantAffecting;
        }
        if old_style.ScrollMarkerGroupNone() != new_style.ScrollMarkerGroupNone() {
            return kDescendantAffecting;
        }
        if !old_style.NonIndependentInheritedEqual(new_style)
            || old_style.JustifyItems() != new_style.JustifyItems()
            || old_style.AppliedTextDecorations() != new_style.AppliedTextDecorations()
        {
            return kInherited;
        }
        let non_inherited_equal = old_style.NonInheritedEqual(new_style);
        if !non_inherited_equal && old_style.ChildHasExplicitInheritance() {
            return kInherited;
        }
        let variables_independent =
            !old_style.HasVariableReference() && !old_style.HasVariableDeclaration();
        let inherited_variables_equal = old_style.InheritedVariablesEqual(new_style);
        if !inherited_variables_equal && !variables_independent {
            return kInherited;
        }
        if !old_style.IndependentInheritedEqual(new_style) || !inherited_variables_equal {
            return kIndependentInherited;
        }
        if non_inherited_equal {
            if Self::PseudoElementStylesEqual(old_style, new_style) {
                return kEqual;
            }
            return kPseudoElementStyle;
        }
        if old_style.EffectiveOverscrollContainerType()
            != new_style.EffectiveOverscrollContainerType()
        {
            return kDescendantAffecting;
        }
        if new_style.HasAnyPseudoElementStyles()
            || old_style.HasAnyPseudoElementStyles()
            || (old_style.Display() != new_style.Display()
                && (new_style.IsDisplayListItem() || old_style.IsDisplayListItem()))
        {
            return kPseudoElementStyle;
        }
        kNonInherited
    }

    // cpp: layoutng_style/style/computed_style.h:2802-2803
    pub(crate) fn GetInternalForcedCurrentColor(
        &self,
        is_current_color: Option<&mut bool>,
    ) -> Color {
        unsafe { ComputedStyleGetInternalForcedCurrentColor(self, is_current_color) }
    }

    // cpp: layoutng_style/style/computed_style.h:2804-2805
    pub(crate) fn GetInternalForcedVisitedCurrentColor(
        &self,
        is_current_color: Option<&mut bool>,
    ) -> Color {
        unsafe { ComputedStyleGetInternalForcedVisitedCurrentColor(self, is_current_color) }
    }

    // cpp: layoutng_style/style/computed_style.h:2807-2809
    pub(crate) fn VisitedDependentContextPaint(
        &self,
        context_paint: &SVGPaint,
        context_visited_paint: &SVGPaint,
    ) -> Color {
        unsafe {
            ComputedStyleVisitedDependentContextPaint(self, context_paint, context_visited_paint)
        }
    }

    // cpp: layoutng_style/style/computed_style.h:322
    pub(crate) fn HasCachedPseudoElementStyles(&self) -> bool {
        let cache = self.GetPseudoElementStyleCache();
        !cache.is_null() && unsafe { &*cache }.size() != 0
    }

    // cpp: layoutng_style/style/computed_style.h:323
    pub(crate) fn GetPseudoElementStyleCache(&self) -> *mut PseudoElementStyleCache {
        let cache = self.cached_data_.get().Get();
        if cache.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &*cache }.pseudo_element_styles_.Get()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:324
    pub(crate) fn EnsurePseudoElementStyleCache(&self) -> &mut PseudoElementStyleCache {
        // cpp: third_party/blink/renderer/core/style/computed_style.cc:173-180
        let data = self.EnsureCachedData();
        if data.pseudo_element_styles_.Get().is_null() {
            data.pseudo_element_styles_ =
                Member::from_ptr(MakeGarbageCollected(PseudoElementStyleCache::default()));
        }
        unsafe { &mut *data.pseudo_element_styles_.Get() }
    }

    // cpp: layoutng_style/style/computed_style.h:326
    pub(crate) fn GetVariableNamesCache(&self) -> *mut foundation::Vector<AtomicString> {
        unsafe { ComputedStyleGetVariableNamesCache(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:327
    pub(crate) fn EnsureVariableNamesCache(&self) -> &mut foundation::Vector<AtomicString> {
        unsafe { &mut *ComputedStyleEnsureVariableNamesCache(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:410-412
    pub fn ComputeDifference(
        old_style: *const ComputedStyle,
        new_style: *const ComputedStyle,
    ) -> ComputedStyleDifference {
        // cpp: third_party/blink/renderer/core/style/computed_style.cc:407-436
        if old_style == new_style {
            return ComputedStyleDifference::kEqual;
        }
        // Both pointers come from live GC-owned immutable computed styles.
        let (Some(old_style), Some(new_style)) =
            (unsafe { old_style.as_ref() }, unsafe { new_style.as_ref() })
        else {
            return ComputedStyleDifference::kInherited;
        };
        let cached = old_style
            .GetCachedPseudoElementStyleWithoutArgument(PseudoId::kPseudoIdFirstLineInherited);
        let inherited_first_line_diff = if let Some(cached) = unsafe { cached.as_ref() } {
            debug_assert!(new_style
                .GetCachedPseudoElementStyleWithoutArgument(PseudoId::kPseudoIdFirstLineInherited)
                .is_null());
            Self::ComputeDifferenceIgnoringInheritedFirstLineStyle(cached, new_style)
        } else {
            ComputedStyleDifference::kEqual
        };
        inherited_first_line_diff.max(Self::ComputeDifferenceIgnoringInheritedFirstLineStyle(
            old_style, new_style,
        ))
    }

    // cpp: layoutng_style/style/computed_style.h:414-417
    pub fn DiffAffectsContainerQueries(
        old_style: *const ComputedStyle,
        new_style: *const ComputedStyle,
    ) -> bool {
        unsafe { ComputedStyleDiffAffectsContainerQueries(old_style, new_style) }
    }

    // cpp: layoutng_style/style/computed_style.h:419-422
    pub fn NeedsReattachLayoutTree(
        element: &Element,
        old_style: *const ComputedStyle,
        new_style: *const ComputedStyle,
    ) -> bool {
        unsafe { ComputedStyleNeedsReattachLayoutTree(element, old_style, new_style) }
    }

    // cpp: layoutng_style/style/computed_style.h:424-427
    pub fn NeedsReinsertLayoutTree(old_style: &ComputedStyle, new_style: &ComputedStyle) -> bool {
        unsafe { ComputedStyleNeedsReinsertLayoutTree(old_style, new_style) }
    }

    // cpp: layoutng_style/style/computed_style.h:436-438
    pub fn GetCachedPseudoElementStyle(
        &self,
        pseudo_id: PseudoId,
        pseudo_argument: &AtomicString,
    ) -> *const ComputedStyle {
        // cpp: third_party/blink/renderer/core/style/computed_style.cc:641-655
        let cache = self.GetPseudoElementStyleCache();
        if cache.is_null() {
            return std::ptr::null();
        }
        let key = super::style_cached_data::PseudoElementStyleCacheKey {
            pseudo_type: pseudo_id,
            pseudo_argument: pseudo_argument.clone(),
        };
        unsafe { &*cache }
            .get(&key)
            .map_or(std::ptr::null(), |style| style.Get())
    }

    // C++ defaults pseudo_argument to g_null_atom.
    // cpp: layoutng_style/style/computed_style.h:436-438
    pub fn GetCachedPseudoElementStyleWithoutArgument(
        &self,
        pseudo_id: PseudoId,
    ) -> *const ComputedStyle {
        self.GetCachedPseudoElementStyle(pseudo_id, &g_null_atom)
    }

    // cpp: layoutng_style/style/computed_style.h:439-441
    pub fn AddCachedPseudoElementStyle(
        &self,
        pseudo_style: *const ComputedStyle,
        pseudo_id: PseudoId,
        pseudo_argument: &AtomicString,
    ) -> *const ComputedStyle {
        // cpp: third_party/blink/renderer/core/style/computed_style.cc:657-680
        let pseudo = unsafe { pseudo_style.as_ref() }.expect("cached pseudo style is non-null");
        debug_assert_eq!(pseudo.StyleType(), pseudo_id);
        let key = super::style_cached_data::PseudoElementStyleCacheKey {
            pseudo_type: pseudo_id,
            pseudo_argument: pseudo_argument.clone(),
        };
        let cache = self.EnsurePseudoElementStyleCache();
        debug_assert!(!cache.contains_key(&key), "pseudo style already cached");
        cache.insert(key, Member::from_ptr(pseudo_style as *mut ComputedStyle));
        pseudo_style
    }

    // cpp: layoutng_style/style/computed_style.h:442-445
    pub fn ReplaceCachedPseudoElementStyle(
        &self,
        pseudo_style: *const ComputedStyle,
        pseudo_id: PseudoId,
        pseudo_argument: &AtomicString,
    ) -> *const ComputedStyle {
        // cpp: third_party/blink/renderer/core/style/computed_style.cc:682-699
        let pseudo = unsafe { pseudo_style.as_ref() }.expect("cached pseudo style is non-null");
        debug_assert!(
            pseudo.StyleType() != PseudoId::kPseudoIdNone
                && pseudo.StyleType() != PseudoId::kPseudoIdFirstLineInherited
        );
        let cache = self.GetPseudoElementStyleCache();
        if !cache.is_null() {
            let key = super::style_cached_data::PseudoElementStyleCacheKey {
                pseudo_type: pseudo_id,
                pseudo_argument: pseudo_argument.clone(),
            };
            if let Some(cached) = unsafe { &mut *cache }.get_mut(&key) {
                assert!(unsafe { &*cached.Get() }.IsEnsuredInDisplayNone());
                *cached = Member::from_ptr(pseudo_style as *mut ComputedStyle);
                return pseudo_style;
            }
        }
        self.AddCachedPseudoElementStyle(pseudo_style, pseudo_id, pseudo_argument)
    }

    // cpp: layoutng_style/style/computed_style.h:446
    pub fn ClearCachedPseudoElementStyles(&self) {
        // cpp: third_party/blink/renderer/core/style/computed_style.cc:701-705
        let cache = self.GetPseudoElementStyleCache();
        if !cache.is_null() {
            unsafe { &mut *cache }.clear();
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1603
    pub fn ContentDataEquivalent(&self, other: &ComputedStyle) -> bool {
        unsafe { ComputedStyleContentDataEquivalent(self, other) }
    }

    // cpp: layoutng_style/style/computed_style.h:920-922
    pub fn ResolvedCaretTextColor(&self) -> Option<Color> {
        unsafe { ComputedStyleResolvedCaretTextColor(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:924-926
    pub fn AccentColorResolved(&self) -> Option<Color> {
        unsafe { ComputedStyleAccentColorResolved(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:928-931
    pub fn ScrollbarThumbColorResolved(&self) -> Option<Color> {
        unsafe { ComputedStyleScrollbarThumbColorResolved(self) }
    }

    pub fn ScrollbarTrackColorResolved(&self) -> Option<Color> {
        unsafe { ComputedStyleScrollbarTrackColorResolved(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:938
    pub fn InheritedEqual(&self, other: &ComputedStyle) -> bool {
        self.IndependentInheritedEqual(other) && self.NonIndependentInheritedEqual(other)
    }

    // cpp: layoutng_style/style/computed_style.h:939
    pub fn NonInheritedEqual(&self, other: &ComputedStyle) -> bool {
        self.base_.NonInheritedEqual(&other.base_)
    }

    // cpp: layoutng_style/style/computed_style.h:940
    pub fn IndependentInheritedEqual(&self, other: &ComputedStyle) -> bool {
        self.base_.IndependentInheritedEqual(&other.base_)
    }

    // cpp: layoutng_style/style/computed_style.h:941
    pub fn NonIndependentInheritedEqual(&self, other: &ComputedStyle) -> bool {
        self.base_.NonIndependentInheritedEqual(&other.base_)
    }

    // cpp: layoutng_style/style/computed_style.h:942
    pub fn InheritedEqualIncludingInheritedVariables(&self, other: &ComputedStyle) -> bool {
        self.base_
            .InheritedEqualIncludingInheritedVariables(&other.base_)
    }

    // cpp: layoutng_style/style/computed_style.h:953
    pub fn CopyChildDependentFlagsFrom(&self, other: &ComputedStyle) {
        unsafe { ComputedStyleCopyChildDependentFlagsFrom(self, other) }
    }

    // cpp: layoutng_style/style/computed_style.h:971-972
    pub fn HasVariables(&self) -> bool {
        unsafe { ComputedStyleHasVariables(self) }
    }

    // C++ wtf_size_t is a 32-bit count.
    // cpp: layoutng_style/style/computed_style.h:973
    pub fn GetVariableNamesCount(&self) -> u32 {
        unsafe { ComputedStyleGetVariableNamesCount(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:974
    pub fn GetVariableNames(&self) -> &foundation::Vector<AtomicString> {
        unsafe { &*ComputedStyleGetVariableNames(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:975
    pub fn InheritedVariables(&self) -> &StyleInheritedVariables {
        unsafe { &*ComputedStyleInheritedVariables(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:976
    pub fn NonInheritedVariables(&self) -> &StyleNonInheritedVariables {
        unsafe { &*ComputedStyleNonInheritedVariables(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:978-979
    pub fn GetVariableData(&self, name: &AtomicString) -> *mut CSSVariableData {
        unsafe { ComputedStyleGetVariableData(self, name) }
    }

    // C++ overload taking the inherited-property flag.
    // cpp: layoutng_style/style/computed_style.h:980-981
    pub fn GetVariableDataWithInheritance(
        &self,
        name: &AtomicString,
        is_inherited_property: bool,
    ) -> *mut CSSVariableData {
        unsafe { ComputedStyleGetVariableDataWithInheritance(self, name, is_inherited_property) }
    }

    // cpp: layoutng_style/style/computed_style.h:983
    pub fn GetVariableValue(&self, name: &AtomicString) -> *const CSSValue {
        unsafe { ComputedStyleGetVariableValue(self, name) }
    }

    // C++ overload taking the inherited-property flag.
    // cpp: layoutng_style/style/computed_style.h:984-985
    pub fn GetVariableValueWithInheritance(
        &self,
        name: &AtomicString,
        is_inherited_property: bool,
    ) -> *const CSSValue {
        unsafe { ComputedStyleGetVariableValueWithInheritance(self, name, is_inherited_property) }
    }

    // cpp: layoutng_style/style/computed_style.h:347-350
    pub fn TraceAfterDispatch(&self, visitor: &mut Visitor) {
        let cached_data = self.cached_data_.get();
        visitor.Trace(&cached_data);
        self.base_.TraceAfterDispatch(visitor);
    }

    // cpp: layoutng_style/style/computed_style.h:357-365
    pub fn NullifyEnsured(style: *const Self) -> *const Self {
        if style.is_null() {
            return std::ptr::null();
        }
        if unsafe { &*style }.IsEnsuredInDisplayNone() {
            return std::ptr::null();
        }
        style
    }

    // cpp: layoutng_style/style/computed_style.h:367-369
    pub fn IsNullOrEnsured(style: *const Self) -> bool {
        Self::NullifyEnsured(style).is_null()
    }

    // The exported C++ declaration has no definition in this package.
    // cpp: layoutng_style/style/computed_style.h:456
    pub fn GetBaseImportantSet(&self) -> *const CSSBitset {
        unsafe { ComputedStyleGetBaseImportantSet(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:458-463
    pub fn GetBaseComputedStyleOrThis(&self) -> *const Self {
        let base = self.GetBaseComputedStyle();
        if base.is_null() {
            self
        } else {
            base
        }
    }

    // cpp: layoutng_style/style/computed_style.h:465-467
    pub fn CustomHighlightNames(&self) -> *mut HashSet<AtomicString> {
        self.CustomHighlightNamesInternal()
            .as_deref()
            .map_or(std::ptr::null_mut(), |names| names as *const _ as *mut _)
    }

    // cpp: layoutng_style/style/computed_style.h:497-499
    pub fn AnchorNameDataEquivalent(&self, other: &Self) -> bool {
        foundation::ValuesEquivalent(self.AnchorName(), other.AnchorName())
    }

    // cpp: layoutng_style/style/computed_style.h:508-510
    pub fn HasAnchorFunctionsWithoutEvaluator(&self) -> bool {
        self.HasAnchorFunctions() && !self.HasAnchorEvaluator()
    }

    // cpp: layoutng_style/style/computed_style.h:512-518
    pub fn MayUseImplicitAnchor(&self) -> bool {
        self.GetDefaultAnchorData().GetType() == StylePositionAnchorType::kAuto
            && self.HasOutOfFlowPosition()
            && (self.HasAnchorFunctions()
                || self.AlignSelf().GetPosition() == ItemPosition::kAnchorCenter
                || self.JustifySelf().GetPosition() == ItemPosition::kAnchorCenter)
    }

    // C++ overloads the static and instance variants; Rust names the static one explicitly.
    // cpp: layoutng_style/style/computed_style.h:523-525
    pub fn HasBackdropFilterOperations(
        backdrop_filter: &super::filter_operations::FilterOperations,
    ) -> bool {
        !backdrop_filter.Operations().is_empty()
    }

    // cpp: layoutng_style/style/computed_style.h:526
    pub fn HasBackdropFilter(&self) -> bool {
        Self::HasBackdropFilterOperations(self.BackdropFilter())
    }

    // cpp: layoutng_style/style/computed_style.h:528
    pub fn HasReferenceFilter(&self) -> bool {
        self.Filter().HasReferenceFilter()
    }

    // cpp: layoutng_style/style/computed_style.h:533
    pub fn HasFilter(&self) -> bool {
        !self.Filter().Operations().is_empty()
    }

    // cpp: layoutng_style/style/computed_style.h:536-538
    pub fn HasBackgroundImage(&self) -> bool {
        self.BackgroundInternal().AnyLayerHasImage()
    }

    // cpp: layoutng_style/style/computed_style.h:539-541
    pub fn HasFixedAttachmentBackgroundImage(&self) -> bool {
        self.BackgroundInternal().AnyLayerHasFixedAttachmentImage()
    }

    // cpp: layoutng_style/style/computed_style.h:544-546
    pub fn BackgroundClip(&self) -> EFillBox {
        self.BackgroundInternal().Clip()
    }

    // cpp: layoutng_style/style/computed_style.h:550
    pub fn IsFixedToBottom(&self) -> bool {
        !self.Bottom().IsAuto() && self.Top().IsAuto()
    }

    // cpp: layoutng_style/style/computed_style.h:554-556
    pub fn BorderImageSlices(&self) -> &LengthBox {
        self.BorderImage().ImageSlices()
    }

    // cpp: layoutng_style/style/computed_style.h:559
    pub fn BorderImageSource(&self) -> *mut StyleImage {
        self.BorderImage().GetImage()
    }

    // cpp: layoutng_style/style/computed_style.h:562-564
    pub fn BorderImageWidth(&self) -> &BorderImageLengthBox {
        self.BorderImage().BorderSlices()
    }

    // cpp: layoutng_style/style/computed_style.h:567-569
    pub fn BorderImageOutset(&self) -> &BorderImageLengthBox {
        self.BorderImage().Outset()
    }

    // cpp: layoutng_style/style/computed_style.h:571-576
    pub fn BorderWidth(style: EBorderStyle, width: i32) -> i32 {
        if style == EBorderStyle::kNone || style == EBorderStyle::kHidden {
            return 0;
        }
        width
    }

    // cpp: layoutng_style/style/computed_style.h:578-589
    pub fn CollapsedBorderStyle(rule_style: EBorderStyle) -> EBorderStyle {
        if rule_style == EBorderStyle::kOutset {
            return EBorderStyle::kGroove;
        }
        if rule_style == EBorderStyle::kInset {
            return EBorderStyle::kRidge;
        }
        rule_style
    }

    // cpp: layoutng_style/style/computed_style.h:592-594
    pub fn BorderTopWidth(&self) -> i32 {
        Self::BorderWidth(self.BorderTopStyle(), *self.SpecifiedBorderTopWidth())
    }

    // cpp: layoutng_style/style/computed_style.h:595-597
    pub fn BorderBottomWidth(&self) -> i32 {
        Self::BorderWidth(self.BorderBottomStyle(), *self.SpecifiedBorderBottomWidth())
    }

    // cpp: layoutng_style/style/computed_style.h:598-600
    pub fn BorderLeftWidth(&self) -> i32 {
        Self::BorderWidth(self.BorderLeftStyle(), *self.SpecifiedBorderLeftWidth())
    }

    // cpp: layoutng_style/style/computed_style.h:601-603
    pub fn BorderRightWidth(&self) -> i32 {
        Self::BorderWidth(self.BorderRightStyle(), *self.SpecifiedBorderRightWidth())
    }

    // cpp: layoutng_style/style/computed_style.h:606-613
    pub fn ClipPath(&self) -> Option<*mut dyn ClipPathOperation> {
        if self.HasClipPath() {
            self.ClipPathInternal()
                .GetNonNull()
                .map(|path| path.as_ptr())
        } else {
            None
        }
    }

    // cpp: layoutng_style/style/computed_style.h:616
    pub fn GetContentData(&self) -> Option<*mut dyn ContentData> {
        self.ContentInternal()
            .as_ref()
            .and_then(|content| content.GetNonNull().map(|data| data.as_ptr()))
    }

    // cpp: layoutng_style/style/computed_style.h:622-632
    pub fn LineClamp(&self) -> i32 {
        if !RuntimeEnabledFeatures::CSSLineClampEnabled() {
            debug_assert_eq!(self.Continue(), EContinue::kNormal);
            if self.IsSpecifiedDisplayWebkitBox() {
                return self.WebkitLineClamp();
            }
        } else if self.IsEffectiveContinueCollapse() {
            return self.MaxLines().Lines() as i32;
        }
        0
    }

    // cpp: layoutng_style/style/computed_style.h:633-643
    pub fn IsEffectiveContinueCollapse(&self) -> bool {
        debug_assert!(RuntimeEnabledFeatures::CSSLineClampEnabled());
        match self.Continue() {
            EContinue::kNormal => false,
            EContinue::kCollapse => true,
            EContinue::kWebkitLegacy => self.IsSpecifiedDisplayWebkitBox(),
        }
    }

    // cpp: layoutng_style/style/computed_style.h:645-650
    pub fn HasLineClamp(&self) -> bool {
        if !RuntimeEnabledFeatures::CSSLineClampEnabled() {
            return self.IsSpecifiedDisplayWebkitBox() && self.WebkitLineClamp() != 0;
        }
        self.IsEffectiveContinueCollapse()
    }

    // cpp: layoutng_style/style/computed_style.h:654-665
    pub fn OutlineVisuallyEqual(&self, other: &Self) -> bool {
        if self.OutlineStyle() == EBorderStyle::kNone && other.OutlineStyle() == EBorderStyle::kNone
        {
            return true;
        }
        self.OutlineWidth() == other.OutlineWidth()
            && self.ResolvedColor(self.OutlineColor(), None)
                == other.ResolvedColor(other.OutlineColor(), None)
            && self.OutlineStyle() == other.OutlineStyle()
            && self.OutlineOffset() == other.OutlineOffset()
            && self.OutlineStyleIsAuto() == other.OutlineStyleIsAuto()
    }

    // cpp: layoutng_style/style/computed_style.h:671-674
    pub fn OutlineRectsShouldIncludeBlockInkOverflow(&self) -> OutlineType {
        if self.OutlineStyleIsAuto() {
            OutlineType::kIncludeBlockInkOverflow
        } else {
            OutlineType::kDontIncludeBlockInkOverflow
        }
    }

    // C++ uses PhysicalToLogicalGetter, whose getters read the same four
    // physical fields; the Rust value holds references to those fields.
    // cpp: layoutng_style/style/computed_style.h:678-685
    pub fn PhysicalScrollPaddingToLogicalGetter(&self) -> PhysicalToLogical<&Length> {
        PhysicalToLogical::new(
            self.GetWritingDirection(),
            self.ScrollPaddingTop(),
            self.ScrollPaddingRight(),
            self.ScrollPaddingBottom(),
            self.ScrollPaddingLeft(),
        )
    }

    // cpp: layoutng_style/style/computed_style.h:687-690
    pub fn ScrollPaddingBlockStart(&self) -> &Length {
        self.PhysicalScrollPaddingToLogicalGetter().BlockStart()
    }

    // cpp: layoutng_style/style/computed_style.h:692-695
    pub fn ScrollPaddingBlockEnd(&self) -> &Length {
        self.PhysicalScrollPaddingToLogicalGetter().BlockEnd()
    }

    // cpp: layoutng_style/style/computed_style.h:697-700
    pub fn ScrollPaddingInlineStart(&self) -> &Length {
        self.PhysicalScrollPaddingToLogicalGetter().InlineStart()
    }

    // cpp: layoutng_style/style/computed_style.h:702-705
    pub fn ScrollPaddingInlineEnd(&self) -> &Length {
        self.PhysicalScrollPaddingToLogicalGetter().InlineEnd()
    }

    // cpp: layoutng_style/style/computed_style.h:708-710
    pub fn IsScrollbarGutterAuto(&self) -> bool {
        self.ScrollbarGutter() == ScrollbarGutter::kScrollbarGutterAuto.value() as u32
    }

    // cpp: layoutng_style/style/computed_style.h:711-713
    pub fn IsScrollbarGutterStable(&self) -> bool {
        self.ScrollbarGutter() & ScrollbarGutter::kScrollbarGutterStable.value() as u32 != 0
    }

    // cpp: layoutng_style/style/computed_style.h:714-716
    pub fn IsScrollbarGutterBothEdges(&self) -> bool {
        self.ScrollbarGutter() & ScrollbarGutter::kScrollbarGutterBothEdges.value() as u32 != 0
    }

    // cpp: layoutng_style/style/computed_style.h:718-720
    pub fn UsesStandardScrollbarStyle(&self) -> bool {
        self.ScrollbarWidth() != EScrollbarWidth::kAuto || !self.ScrollbarColor().is_null()
    }

    // cpp: layoutng_style/style/computed_style.h:731
    pub fn ShapeOutside(&self) -> *mut ShapeValue {
        self.ShapeOutsideInternal().Get()
    }

    // cpp: layoutng_style/style/computed_style.h:734-736
    pub fn VerticalAlign(&self) -> EVerticalAlign {
        match self.VerticalAlignInternal() {
            0 => EVerticalAlign::kBaseline,
            1 => EVerticalAlign::kMiddle,
            2 => EVerticalAlign::kSub,
            3 => EVerticalAlign::kSuper,
            4 => EVerticalAlign::kTextTop,
            5 => EVerticalAlign::kTextBottom,
            6 => EVerticalAlign::kTop,
            7 => EVerticalAlign::kBottom,
            8 => EVerticalAlign::kBaselineMiddle,
            9 => EVerticalAlign::kLength,
            _ => unreachable!("invalid vertical-align bitfield value"),
        }
    }

    // cpp: layoutng_style/style/computed_style.h:744
    pub fn EffectiveZIndex(&self) -> i32 {
        if self.AllowsZIndex() {
            self.ZIndex()
        } else {
            0
        }
    }

    // cpp: layoutng_style/style/computed_style.h:748-750
    pub fn HasMaskBoxImageOutsets(&self) -> bool {
        self.MaskBoxImageInternal().HasImage() && self.MaskBoxImageOutset().NonZero()
    }

    // cpp: layoutng_style/style/computed_style.h:751-753
    pub fn MaskBoxImageOutsets(&self) -> PhysicalBoxStrut {
        self.ImageOutsets(self.MaskBoxImageInternal())
    }

    // cpp: layoutng_style/style/computed_style.h:754-756
    pub fn MaskBoxImageOutset(&self) -> &BorderImageLengthBox {
        self.MaskBoxImageInternal().Outset()
    }

    // cpp: layoutng_style/style/computed_style.h:759-761
    pub fn MaskBoxImageSlices(&self) -> &LengthBox {
        self.MaskBoxImageInternal().ImageSlices()
    }

    // cpp: layoutng_style/style/computed_style.h:764-766
    pub fn MaskBoxImageSource(&self) -> *mut StyleImage {
        self.MaskBoxImageInternal().GetImage()
    }

    // cpp: layoutng_style/style/computed_style.h:769-771
    pub fn MaskBoxImageWidth(&self) -> &BorderImageLengthBox {
        self.MaskBoxImageInternal().BorderSlices()
    }

    // cpp: layoutng_style/style/computed_style.h:779-781
    pub fn ListStyleTypeDataEquivalent(&self, other: &Self) -> bool {
        foundation::ValuesEquivalent(self.ListStyleType(), other.ListStyleType())
    }

    // cpp: layoutng_style/style/computed_style.h:796-798
    pub fn GetFontSizeStyle(&self) -> FontSizeStyle<'_> {
        FontSizeStyle::new(self.GetFont(), self.LineHeight(), self.EffectiveZoom())
    }

    // cpp: layoutng_style/style/computed_style.h:801-803
    pub fn GetFontDescription(&self) -> &FontDescription {
        unsafe { &*self.GetFont() }.GetFontDescription()
    }

    // cpp: layoutng_style/style/computed_style.h:804-806
    pub fn HasFontRelativeUnits(&self) -> bool {
        self.HasEmUnits() || self.HasRootRelativeUnits() || self.HasGlyphRelativeUnits()
    }

    // cpp: layoutng_style/style/computed_style.h:807-810
    pub fn HasAnyRelativeUnits(&self) -> bool {
        self.HasFontRelativeUnits()
            || self.HasContainerRelativeValue()
            || self.HasLogicalDirectionRelativeUnits()
            || self.HasViewportUnits()
    }

    // cpp: layoutng_style/style/computed_style.h:813-817
    pub fn DependsOnFontMetrics(&self) -> bool {
        self.HasGlyphRelativeUnits()
            || self.HasFontSizeAdjust()
            || self.CustomStyleCallbackDependsOnFont()
            || (self.StyleType() == PseudoId::kPseudoIdFirstLetter
                && !self.InitialLetter().IsNormal())
    }

    // cpp: layoutng_style/style/computed_style.h:826
    pub fn FontSize(&self) -> i32 {
        self.GetFontDescription().ComputedPixelSize()
    }

    // cpp: layoutng_style/style/computed_style.h:827-829
    pub fn SpecifiedFontSize(&self) -> f32 {
        self.GetFontDescription().SpecifiedSize()
    }

    // cpp: layoutng_style/style/computed_style.h:830-832
    pub fn ComputedFontSize(&self) -> f32 {
        self.GetFontDescription().ComputedSize()
    }

    // C++ has static and instance overloads; the instance form is suffixed.
    // cpp: layoutng_style/style/computed_style.h:833-835
    pub fn ComputedFontSizeAsFixed(font: &Font) -> LayoutUnit {
        LayoutUnit::FromFloatRound(font.GetFontDescription().ComputedSize())
    }

    // cpp: layoutng_style/style/computed_style.h:836-838
    pub fn ComputedFontSizeAsFixedValue(&self) -> LayoutUnit {
        Self::ComputedFontSizeAsFixed(unsafe { &*self.GetFont() })
    }

    // cpp: layoutng_style/style/computed_style.h:844-846
    pub fn HasFontSizeAdjust(&self) -> bool {
        self.GetFontDescription().HasSizeAdjust()
    }

    // cpp: layoutng_style/style/computed_style.h:841-843
    pub fn FontSizeAdjust(&self) -> FontSizeAdjust {
        self.GetFontDescription().SizeAdjust()
    }

    // cpp: layoutng_style/style/computed_style.h:849-851
    pub fn GetFontWeight(&self) -> FontSelectionValue {
        self.GetFontDescription().Weight()
    }

    // cpp: layoutng_style/style/computed_style.h:854-856
    pub fn GetFontStretch(&self) -> FontSelectionValue {
        self.GetFontDescription().Stretch()
    }

    // cpp: layoutng_style/style/computed_style.h:859-861
    pub fn GetFontStyle(&self) -> FontSelectionValue {
        self.GetFontDescription().Style()
    }

    // cpp: layoutng_style/style/computed_style.h:864-866
    pub fn GetFontPalette(&self) -> *const FontPalette {
        self.GetFontDescription().GetFontPalette()
    }

    // C++ overloads GetFontHeight with and without a baseline argument.
    // cpp: layoutng_style/style/computed_style.h:873-875
    pub fn GetFontHeightForDefaultBaseline(&self) -> FontHeight {
        self.GetFontHeight(self.GetFontBaseline())
    }

    // cpp: layoutng_style/style/computed_style.h:878-880
    pub fn Locale(&self) -> &AtomicString {
        unsafe { LayoutLocale::LocaleString(self.GetFontDescription().Locale()) }
    }

    // cpp: layoutng_style/style/computed_style.h:883
    pub fn LetterSpacing(&self) -> f32 {
        self.GetFontDescription().LetterSpacing()
    }

    // cpp: layoutng_style/style/computed_style.h:884-886
    pub fn ComputedLetterSpacing(&self) -> &Length {
        self.GetFontDescription().ComputedLetterSpacing()
    }

    // cpp: layoutng_style/style/computed_style.h:889
    pub fn WordSpacing(&self) -> f32 {
        self.GetFontDescription().WordSpacing()
    }

    // cpp: layoutng_style/style/computed_style.h:890-892
    pub fn ComputedWordSpacing(&self) -> &Length {
        self.GetFontDescription().ComputedWordSpacing()
    }

    // cpp: layoutng_style/style/computed_style.h:894-895
    pub fn HasFill(&self) -> bool {
        !self.FillPaint().IsNone()
    }

    // cpp: layoutng_style/style/computed_style.h:896-899
    pub fn IsFillColorCurrentColor(&self) -> bool {
        self.FillPaint().HasCurrentColor() || self.InternalVisitedFillPaint().HasCurrentColor()
    }

    // cpp: layoutng_style/style/computed_style.h:902-904
    pub fn HasMarkers(&self) -> bool {
        !self.MarkerStartResource().is_null()
            || !self.MarkerMidResource().is_null()
            || !self.MarkerEndResource().is_null()
    }

    // cpp: layoutng_style/style/computed_style.h:907
    pub fn HasStroke(&self) -> bool {
        !self.StrokePaint().IsNone()
    }

    // cpp: layoutng_style/style/computed_style.h:908-910
    pub fn HasVisibleStroke(&self) -> bool {
        self.HasStroke() && !self.StrokeWidth().IsZero()
    }

    // cpp: layoutng_style/style/computed_style.h:911-914
    pub fn IsStrokeColorCurrentColor(&self) -> bool {
        self.StrokePaint().HasCurrentColor() || self.InternalVisitedStrokePaint().HasCurrentColor()
    }

    // cpp: layoutng_style/style/computed_style.h:916
    pub fn IsCaretColorAuto(&self) -> bool {
        self.CaretColor().IsAutoColor()
    }

    // cpp: layoutng_style/style/computed_style.h:917-919
    pub fn IsCaretTextColorAuto(&self) -> bool {
        self.CaretColor().TextColor().IsAutoColor()
    }

    // cpp: layoutng_style/style/computed_style.h:944
    pub fn HasChildDependentFlags(&self) -> bool {
        self.ChildHasExplicitInheritance()
    }

    // cpp: third_party/blink/renderer/core/style/computed_style.cc:1779-1792
    // cpp: layoutng_style/style/computed_style.h:956
    pub fn GetCounterDirectivesMap(&self) -> *const CounterDirectiveMap {
        self.CounterDirectivesInternal().as_deref().map_or(std::ptr::null(), std::ptr::from_ref)
    }

    // cpp: layoutng_style/style/computed_style.h:957-958
    pub fn GetCounterDirectivesForIdentifier(
        &self,
        identifier: &AtomicString,
    ) -> CounterDirectives {
        self.CounterDirectivesInternal().as_ref().and_then(|map| map.get(identifier)).cloned().unwrap_or_default()
    }

    // cpp: layoutng_style/style/computed_style.h:946-951
    pub fn SetChildHasExplicitInheritance(&self) {
        debug_assert_ne!(self as *const Self, Self::GetInitialStyleSingleton());
        self.base_.SetChildHasExplicitInheritance();
    }

    // cpp: layoutng_style/style/computed_style.h:959-964
    pub fn CounterDirectivesEqual(&self, other: &Self) -> bool {
        let directives = self
            .CounterDirectivesInternal()
            .as_deref()
            .map_or(std::ptr::null(), |map| map as *const CounterDirectiveMap);
        let other_directives = other
            .CounterDirectivesInternal()
            .as_deref()
            .map_or(std::ptr::null(), |map| map as *const CounterDirectiveMap);
        foundation::ValuesEquivalent(directives, other_directives)
    }

    // cpp: layoutng_style/style/computed_style.h:966-969
    pub fn IsDeprecatedFlexbox(&self) -> bool {
        self.Display() == EDisplay::kWebkitBox || self.Display() == EDisplay::kWebkitInlineBox
    }

    // cpp: layoutng_style/style/computed_style.h:988-991
    pub fn SpecifiesColumns(&self) -> bool {
        !self.HasAutoColumnCount() || !self.HasAutoColumnWidth() || !self.HasAutoColumnHeight()
    }

    // cpp: layoutng_style/style/computed_style.h:992-994
    pub fn ColumnRuleIsTransparent(&self) -> bool {
        self.GapRuleColorIsTransparent(self.ColumnRuleColor())
    }

    // cpp: layoutng_style/style/computed_style.h:995-1001
    pub fn HasColumnRule(&self) -> bool {
        if !self.IsGapDecorationsContainer() {
            return false;
        }
        Self::HasRuleWidth(self.ColumnRuleWidth())
            && !self.ColumnRuleIsTransparent()
            && Self::BorderStylesAreVisible(self.ColumnRuleStyle())
    }

    // cpp: layoutng_style/style/computed_style.h:1003-1005
    pub fn RowRuleIsTransparent(&self) -> bool {
        self.GapRuleColorIsTransparent(self.RowRuleColor())
    }

    // cpp: layoutng_style/style/computed_style.h:1006-1013
    pub fn HasRowRule(&self) -> bool {
        if !self.IsGapDecorationsContainer() {
            return false;
        }
        Self::HasRuleWidth(self.RowRuleWidth())
            && !self.RowRuleIsTransparent()
            && Self::BorderStylesAreVisible(self.RowRuleStyle())
    }

    // cpp: layoutng_style/style/computed_style.h:1015-1020
    pub fn HasGapRule(&self) -> bool {
        if !self.MaybeHasGapDecorations() {
            return false;
        }
        self.HasColumnRule() || self.HasRowRule()
    }

    // cpp: layoutng_style/style/computed_style.h:1022-1027
    pub fn IsGapDecorationsContainer(&self) -> bool {
        self.SpecifiesColumns()
            || self.IsDisplayFlex()
            || self.IsDisplayWebkitBox()
            || self.IsDisplayGrid()
            || self.IsDisplayGridLanes()
    }

    // cpp: layoutng_style/style/computed_style.h:1030-1036
    pub fn ResolvedIsColumnFlexDirection(&self) -> bool {
        if self.IsDeprecatedFlexbox() {
            return self.BoxOrient() == EBoxOrient::kVertical;
        }
        self.FlexDirection() == EFlexDirection::kColumn
            || self.FlexDirection() == EFlexDirection::kColumnReverse
    }

    // cpp: layoutng_style/style/computed_style.h:1037-1043
    pub fn ResolvedIsReverseFlexDirection(&self) -> bool {
        if self.IsDeprecatedFlexbox() {
            return self.BoxDirection() == EBoxDirection::kReverse;
        }
        self.FlexDirection() == EFlexDirection::kRowReverse
            || self.FlexDirection() == EFlexDirection::kColumnReverse
    }

    // cpp: layoutng_style/style/computed_style.h:1044-1049
    pub fn ResolvedIsFlexWrapReverse(&self) -> bool {
        if self.IsDeprecatedFlexbox() {
            return false;
        }
        self.FlexWrap().GetWrapMode() == FlexWrapMode::kWrapReverse
    }

    // cpp: layoutng_style/style/computed_style.h:1050-1055
    pub fn ResolvedIsFlexNowrap(&self) -> bool {
        if self.IsDeprecatedFlexbox() {
            return true;
        }
        self.FlexWrap().GetWrapMode() == FlexWrapMode::kNowrap
    }

    // cpp: layoutng_style/style/computed_style.h:1056-1061
    pub fn ResolvedFlexLineCount(&self) -> Option<usize> {
        if self.IsDeprecatedFlexbox() || !self.FlexWrap().IsBalanced() {
            return None;
        }
        Some(self.FlexLineCount() as usize)
    }

    // cpp: layoutng_style/style/computed_style.h:1063-1068
    pub fn ResolvedFlexGrow(&self, box_style: &Self) -> f32 {
        if box_style.IsDeprecatedFlexbox() {
            return if self.BoxFlex() > 0.0 {
                self.BoxFlex()
            } else {
                0.0
            };
        }
        self.FlexGrow()
    }

    // cpp: layoutng_style/style/computed_style.h:1069-1074
    pub fn ResolvedFlexShrink(&self, box_style: &Self) -> f32 {
        if box_style.IsDeprecatedFlexbox() {
            return if self.BoxFlex() > 0.0 {
                self.BoxFlex()
            } else {
                0.0
            };
        }
        self.FlexShrink()
    }

    // cpp: layoutng_style/style/computed_style.h:1077-1080
    pub fn HasMask(&self) -> bool {
        self.MaskInternal().AnyLayerHasImage() || self.MaskBoxImageInternal().HasImage()
    }

    // cpp: layoutng_style/style/computed_style.h:1081
    pub fn MaskLayers(&self) -> &FillLayer {
        self.MaskInternal()
    }

    // cpp: layoutng_style/style/computed_style.h:1082
    pub fn MaskBoxImage(&self) -> &NinePieceImage {
        self.MaskBoxImageInternal()
    }

    // cpp: layoutng_style/style/computed_style.h:1083
    pub fn MaskBoxImageSlicesFill(&self) -> bool {
        self.MaskBoxImageInternal().Fill()
    }

    // cpp: layoutng_style/style/computed_style.h:1086
    pub fn HasTextCombine(&self) -> bool {
        self.TextCombine() != ETextCombine::kNone
    }

    // cpp: layoutng_style/style/computed_style.h:1089
    pub fn GetGridAutoFlow(&self) -> GridAutoFlow {
        self.GridAutoFlowInternal()
    }

    // cpp: layoutng_style/style/computed_style.h:1090-1094
    pub fn IsGridAutoFlowDirectionRow(&self) -> bool {
        let row = InternalGridAutoFlowDirection::kInternalAutoFlowDirectionRow as i32;
        (self.GridAutoFlowInternal() as i32 & row) == row
    }

    // cpp: layoutng_style/style/computed_style.h:1095-1099
    pub fn IsGridAutoFlowDirectionColumn(&self) -> bool {
        let column = InternalGridAutoFlowDirection::kInternalAutoFlowDirectionColumn as i32;
        (self.GridAutoFlowInternal() as i32 & column) == column
    }

    // cpp: layoutng_style/style/computed_style.h:1100-1104
    pub fn IsGridAutoFlowAlgorithmSparse(&self) -> bool {
        let sparse = InternalGridAutoFlowAlgorithm::kInternalAutoFlowAlgorithmSparse as i32;
        (self.GridAutoFlowInternal() as i32 & sparse) == sparse
    }

    // cpp: layoutng_style/style/computed_style.h:1105-1109
    pub fn IsGridAutoFlowAlgorithmDense(&self) -> bool {
        let dense = InternalGridAutoFlowAlgorithm::kInternalAutoFlowAlgorithmDense as i32;
        (self.GridAutoFlowInternal() as i32 & dense) == dense
    }

    // cpp: layoutng_style/style/computed_style.h:1112-1114
    pub fn GridTemplateColumns(&self) -> &ComputedGridTrackList {
        Self::ComputedGridTemplate(self.SpecifiedGridTemplateColumns())
    }

    // cpp: layoutng_style/style/computed_style.h:1116-1118
    pub fn GridTemplateRows(&self) -> &ComputedGridTrackList {
        Self::ComputedGridTemplate(self.SpecifiedGridTemplateRows())
    }

    // cpp: layoutng_style/style/computed_style.h:1121-1139
    pub fn GridLanesTrackSizingDirection(&self) -> GridTrackSizingDirection {
        match self.GetGridLanesDirection().orientation {
            GridLanesOrientation::kColumn => GridTrackSizingDirection::kForColumns,
            GridLanesOrientation::kNormal => {
                if !self.SpecifiedGridTemplateColumns().Get().is_null() {
                    GridTrackSizingDirection::kForColumns
                } else if !self.SpecifiedGridTemplateRows().Get().is_null() {
                    GridTrackSizingDirection::kForRows
                } else {
                    GridTrackSizingDirection::kForColumns
                }
            }
            GridLanesOrientation::kRow => GridTrackSizingDirection::kForRows,
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1143-1145
    pub fn IsReverseGridLanesFillDirection(&self) -> bool {
        self.GetGridLanesDirection().is_fill_reverse
    }

    // cpp: layoutng_style/style/computed_style.h:1150-1152
    pub fn IsReverseGridLanesTrackDirection(&self) -> bool {
        self.GetGridLanesDirection().is_track_reverse
    }

    // cpp: layoutng_style/style/computed_style.h:1154-1156
    pub fn IsGridLanesPackDense(&self) -> bool {
        self.GridLanesPack() == EGridLanesPack::kDense
    }

    // cpp: layoutng_style/style/computed_style.h:1161-1167
    pub fn HasGridTrackAxis(&self, track_direction: GridTrackSizingDirection) -> bool {
        if self.IsDisplayGrid() {
            return true;
        }
        debug_assert!(self.IsDisplayGridLanes());
        self.GridLanesTrackSizingDirection() == track_direction
    }

    // cpp: layoutng_style/style/computed_style.h:1170-1174
    pub fn AutoTracks(&self, track_direction: GridTrackSizingDirection) -> &GridTrackList {
        if track_direction == GridTrackSizingDirection::kForColumns {
            self.GridAutoColumns()
        } else {
            self.GridAutoRows()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1175-1179
    pub fn TemplateTracks(
        &self,
        track_direction: GridTrackSizingDirection,
    ) -> &ComputedGridTrackList {
        if track_direction == GridTrackSizingDirection::kForColumns {
            self.GridTemplateColumns()
        } else {
            self.GridTemplateRows()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1180-1184
    pub fn TrackStart(&self, track_direction: GridTrackSizingDirection) -> &GridPosition {
        if track_direction == GridTrackSizingDirection::kForColumns {
            self.GridColumnStart()
        } else {
            self.GridRowStart()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1185-1187
    pub fn TrackEnd(&self, track_direction: GridTrackSizingDirection) -> &GridPosition {
        if track_direction == GridTrackSizingDirection::kForColumns {
            self.GridColumnEnd()
        } else {
            self.GridRowEnd()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1190-1192
    pub fn GetWritingDirection(&self) -> WritingDirectionMode {
        WritingDirectionMode::new(self.GetWritingMode(), self.Direction())
    }

    // cpp: layoutng_style/style/computed_style.h:1193-1195
    pub fn IsHorizontalWritingMode(&self) -> bool {
        foundation::IsHorizontalWritingMode(self.GetWritingMode())
    }

    // cpp: layoutng_style/style/computed_style.h:1196-1198
    pub fn IsVerticalWritingMode(&self) -> bool {
        foundation::IsVerticalWritingMode(self.GetWritingMode())
    }

    // cpp: layoutng_style/style/computed_style.h:1199-1201
    pub fn IsHorizontalTypographicMode(&self) -> bool {
        foundation::IsHorizontalTypographicMode(self.GetWritingMode())
    }

    // cpp: layoutng_style/style/computed_style.h:1202-1204
    pub fn IsFlippedLinesWritingMode(&self) -> bool {
        foundation::IsFlippedLinesWritingMode(self.GetWritingMode())
    }

    // cpp: layoutng_style/style/computed_style.h:1205-1207
    pub fn IsFlippedBlocksWritingMode(&self) -> bool {
        foundation::IsFlippedBlocksWritingMode(self.GetWritingMode())
    }

    // cpp: layoutng_style/style/computed_style.h:1211-1213
    pub fn HasWillChangeScrollPosition(&self) -> bool {
        let will_change = self.WillChange();
        !will_change.is_null() && unsafe { (&*will_change).has_scroll_position_value() }
    }

    // cpp: layoutng_style/style/computed_style.h:1214-1216
    pub fn HasWillChangeTransformProperty(&self) -> bool {
        let will_change = self.WillChange();
        !will_change.is_null() && unsafe { (&*will_change).has_transform_property() }
    }

    // cpp: layoutng_style/style/computed_style.h:1217-1219
    pub fn HasWillChangeAnyTransformProperty(&self) -> bool {
        let will_change = self.WillChange();
        !will_change.is_null() && unsafe { (&*will_change).has_any_transform_property() }
    }

    // cpp: layoutng_style/style/computed_style.h:1250-1252
    pub fn LogicalWidth(&self) -> &Length {
        if self.IsHorizontalWritingMode() {
            self.Width()
        } else {
            self.Height()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1253-1255
    pub fn LogicalHeight(&self) -> &Length {
        if self.IsHorizontalWritingMode() {
            self.Height()
        } else {
            self.Width()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1256-1258
    pub fn LogicalMaxWidth(&self) -> &Length {
        if self.IsHorizontalWritingMode() {
            self.MaxWidth()
        } else {
            self.MaxHeight()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1259-1261
    pub fn LogicalMaxHeight(&self) -> &Length {
        if self.IsHorizontalWritingMode() {
            self.MaxHeight()
        } else {
            self.MaxWidth()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1262-1264
    pub fn LogicalMinWidth(&self) -> &Length {
        if self.IsHorizontalWritingMode() {
            self.MinWidth()
        } else {
            self.MinHeight()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1265-1267
    pub fn LogicalMinHeight(&self) -> &Length {
        if self.IsHorizontalWritingMode() {
            self.MinHeight()
        } else {
            self.MinWidth()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1269-1272
    pub fn ContainIntrinsicInlineSize(&self) -> &StyleIntrinsicLength {
        if self.IsHorizontalWritingMode() {
            self.ContainIntrinsicWidth()
        } else {
            self.ContainIntrinsicHeight()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1273-1276
    pub fn ContainIntrinsicBlockSize(&self) -> &StyleIntrinsicLength {
        if self.IsHorizontalWritingMode() {
            self.ContainIntrinsicHeight()
        } else {
            self.ContainIntrinsicWidth()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1278-1285
    pub fn EffectiveContainIntrinsicWidth(&self) -> StyleIntrinsicLength {
        let mut length = self.ContainIntrinsicWidth().clone();
        if self.HasSizeContainmentForViewTransitionScope()
            && RuntimeEnabledFeatures::ScopedViewTransitionSizeContainmentEnabled()
        {
            length.SetHasAuto();
        }
        length
    }

    // cpp: layoutng_style/style/computed_style.h:1286-1293
    pub fn EffectiveContainIntrinsicHeight(&self) -> StyleIntrinsicLength {
        let mut length = self.ContainIntrinsicHeight().clone();
        if self.HasSizeContainmentForViewTransitionScope()
            && RuntimeEnabledFeatures::ScopedViewTransitionSizeContainmentEnabled()
        {
            length.SetHasAuto();
        }
        length
    }

    // cpp: layoutng_style/style/computed_style.h:1295-1298
    pub fn EffectiveContainIntrinsicInlineSize(&self) -> StyleIntrinsicLength {
        if self.IsHorizontalWritingMode() {
            self.EffectiveContainIntrinsicWidth()
        } else {
            self.EffectiveContainIntrinsicHeight()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1299-1302
    pub fn EffectiveContainIntrinsicBlockSize(&self) -> StyleIntrinsicLength {
        if self.IsHorizontalWritingMode() {
            self.EffectiveContainIntrinsicHeight()
        } else {
            self.EffectiveContainIntrinsicWidth()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1303-1305
    pub fn IsResponsivelySized(&self) -> bool {
        self.FrameSizing() != EFrameSizing::kAuto
    }

    // cpp: layoutng_style/style/computed_style.h:1308-1310
    pub fn HasMarginBlockStartQuirk(&self) -> bool {
        self.MayHaveMargin() && self.MarginBlockStart().Quirk()
    }

    // cpp: layoutng_style/style/computed_style.h:1311-1313
    pub fn HasMarginBlockEndQuirk(&self) -> bool {
        self.MayHaveMargin() && self.MarginBlockEnd().Quirk()
    }

    // cpp: layoutng_style/style/computed_style.h:1314-1316
    pub fn MarginBlockStart(&self) -> &Length {
        self.MarginBlockStartUsing(self)
    }

    // cpp: layoutng_style/style/computed_style.h:1317
    pub fn MarginBlockEnd(&self) -> &Length {
        self.MarginBlockEndUsing(self)
    }

    // cpp: layoutng_style/style/computed_style.h:1318-1320
    pub fn MarginInlineStart(&self) -> &Length {
        self.MarginInlineStartUsing(self)
    }

    // cpp: layoutng_style/style/computed_style.h:1321
    pub fn MarginInlineEnd(&self) -> &Length {
        self.MarginInlineEndUsing(self)
    }

    // cpp: layoutng_style/style/computed_style.h:1322-1324
    pub fn MarginInlineStartUsing(&self, other: &Self) -> &Length {
        self.PhysicalMarginToLogical(other).InlineStart()
    }

    // cpp: layoutng_style/style/computed_style.h:1325-1327
    pub fn MarginInlineEndUsing(&self, other: &Self) -> &Length {
        self.PhysicalMarginToLogical(other).InlineEnd()
    }

    // cpp: layoutng_style/style/computed_style.h:1328-1330
    pub fn MarginBlockStartUsing(&self, other: &Self) -> &Length {
        self.PhysicalMarginToLogical(other).BlockStart()
    }

    // cpp: layoutng_style/style/computed_style.h:1331-1333
    pub fn MarginBlockEndUsing(&self, other: &Self) -> &Length {
        self.PhysicalMarginToLogical(other).BlockEnd()
    }

    // cpp: layoutng_style/style/computed_style.h:1336-1338
    pub fn PaddingBlockStart(&self) -> &Length {
        self.PhysicalPaddingToLogical().BlockStart()
    }

    // cpp: layoutng_style/style/computed_style.h:1339-1341
    pub fn PaddingBlockEnd(&self) -> &Length {
        self.PhysicalPaddingToLogical().BlockEnd()
    }

    // cpp: layoutng_style/style/computed_style.h:1342-1344
    pub fn PaddingInlineStart(&self) -> &Length {
        self.PhysicalPaddingToLogical().InlineStart()
    }

    // cpp: layoutng_style/style/computed_style.h:1345-1347
    pub fn PaddingInlineEnd(&self) -> &Length {
        self.PhysicalPaddingToLogical().InlineEnd()
    }

    // C++ overloads style and box forms of PaddingEqual.
    // cpp: layoutng_style/style/computed_style.h:1348-1353
    pub fn PaddingEqual(&self, other: &Self) -> bool {
        self.PaddingTop() == other.PaddingTop()
            && self.PaddingLeft() == other.PaddingLeft()
            && self.PaddingRight() == other.PaddingRight()
            && self.PaddingBottom() == other.PaddingBottom()
    }

    // cpp: layoutng_style/style/computed_style.h:1354-1357
    pub fn PaddingEqualBox(&self, other: &LengthBox) -> bool {
        self.PaddingTop() == other.Top()
            && self.PaddingLeft() == other.Left()
            && self.PaddingRight() == other.Right()
            && self.PaddingBottom() == other.Bottom()
    }

    // cpp: layoutng_style/style/computed_style.h:1361-1363
    pub fn HasBorderImageOutsets(&self) -> bool {
        self.BorderImage().HasImage() && self.BorderImage().Outset().NonZero()
    }

    // cpp: layoutng_style/style/computed_style.h:1360
    pub fn ImageOutsets(&self, image: &NinePieceImage) -> PhysicalBoxStrut {
        unsafe { ComputedStyleImageOutsets(self, image) }
    }

    // cpp: layoutng_style/style/computed_style.h:1364-1366
    pub fn BorderImageOutsets(&self) -> PhysicalBoxStrut {
        self.ImageOutsets(self.BorderImage())
    }

    // cpp: layoutng_style/style/computed_style.h:1367
    pub fn BorderImageSlicesFill(&self) -> bool {
        self.BorderImage().Fill()
    }

    // cpp: layoutng_style/style/computed_style.h:1369
    pub fn HasBorderShape(&self) -> bool {
        !self.BorderShape().Get().is_null()
    }

    // cpp: layoutng_style/style/computed_style.h:1371-1376
    pub fn BorderSizeEquals(&self, other: &Self) -> bool {
        self.BorderLeftWidth() == other.BorderLeftWidth()
            && self.BorderTopWidth() == other.BorderTopWidth()
            && self.BorderRightWidth() == other.BorderRightWidth()
            && self.BorderBottomWidth() == other.BorderBottomWidth()
    }

    // cpp: layoutng_style/style/computed_style.h:1378-1380
    pub fn BorderBlockEndWidth(&self) -> i32 {
        self.PhysicalBorderWidthToLogical().BlockEnd()
    }

    // cpp: layoutng_style/style/computed_style.h:1381-1383
    pub fn BorderBlockStartWidth(&self) -> i32 {
        self.PhysicalBorderWidthToLogical().BlockStart()
    }

    // cpp: layoutng_style/style/computed_style.h:1384-1386
    pub fn BorderInlineEndWidth(&self) -> i32 {
        self.PhysicalBorderWidthToLogical().InlineEnd()
    }

    // cpp: layoutng_style/style/computed_style.h:1387-1389
    pub fn BorderInlineStartWidth(&self) -> i32 {
        self.PhysicalBorderWidthToLogical().InlineStart()
    }

    // cpp: layoutng_style/style/computed_style.h:1391-1394
    pub fn HasBorder(&self) -> bool {
        self.BorderLeftWidth() != 0
            || self.BorderRightWidth() != 0
            || self.BorderTopWidth() != 0
            || self.BorderBottomWidth() != 0
    }

    // cpp: layoutng_style/style/computed_style.h:1395-1397
    pub fn HasBorderDecoration(&self) -> bool {
        self.HasBorder() || self.BorderImage().HasImage() || self.HasBorderShape()
    }

    // cpp: layoutng_style/style/computed_style.h:1398-1415
    pub fn HasBorderRadius(&self) -> bool {
        if self.HasBorderShape() {
            return false;
        }
        if !self.BorderTopLeftRadius().Width().IsZero() {
            return true;
        }
        if !self.BorderTopRightRadius().Width().IsZero() {
            return true;
        }
        if !self.BorderBottomLeftRadius().Width().IsZero() {
            return true;
        }
        if !self.BorderBottomRightRadius().Width().IsZero() {
            return true;
        }
        false
    }

    // cpp: layoutng_style/style/computed_style.h:1417-1453
    pub fn BorderVisuallyEqual(&self, other: &Self) -> bool {
        let side_equal = |color: &StyleColor,
                          other_color: &StyleColor,
                          style: EBorderStyle,
                          other_style: EBorderStyle,
                          width: i32,
                          other_width: i32| {
            if style == EBorderStyle::kNone && other_style == EBorderStyle::kNone {
                if !self.HasBorderShape() && !other.HasBorderShape() {
                    return true;
                }
            }
            if style == EBorderStyle::kHidden && other_style == EBorderStyle::kHidden {
                return true;
            }
            width == other_width
                && style == other_style
                && self.ResolvedColor(color, None) == other.ResolvedColor(other_color, None)
        };
        side_equal(
            self.BorderTopColor(),
            other.BorderTopColor(),
            self.BorderTopStyle(),
            other.BorderTopStyle(),
            *self.SpecifiedBorderTopWidth(),
            *other.SpecifiedBorderTopWidth(),
        ) && side_equal(
            self.BorderRightColor(),
            other.BorderRightColor(),
            self.BorderRightStyle(),
            other.BorderRightStyle(),
            *self.SpecifiedBorderRightWidth(),
            *other.SpecifiedBorderRightWidth(),
        ) && side_equal(
            self.BorderBottomColor(),
            other.BorderBottomColor(),
            self.BorderBottomStyle(),
            other.BorderBottomStyle(),
            *self.SpecifiedBorderBottomWidth(),
            *other.SpecifiedBorderBottomWidth(),
        ) && side_equal(
            self.BorderLeftColor(),
            other.BorderLeftColor(),
            self.BorderLeftStyle(),
            other.BorderLeftStyle(),
            *self.SpecifiedBorderLeftWidth(),
            *other.SpecifiedBorderLeftWidth(),
        ) && self.BorderImage() == other.BorderImage()
            && foundation::ValuesEquivalent(self.BorderShape(), other.BorderShape())
    }

    // cpp: layoutng_style/style/computed_style.h:1455-1457
    pub fn BorderVisualOverflowEqual(&self, other: &Self) -> bool {
        self.BorderImage().Outset() == other.BorderImage().Outset()
    }

    // cpp: layoutng_style/style/computed_style.h:1462
    pub fn IsFloating(&self) -> bool {
        self.Floating() != EFloat::kNone
    }

    // cpp: layoutng_style/style/computed_style.h:1463
    pub fn UnresolvedFloating(&self) -> EFloat {
        self.Floating()
    }

    // C++ overloads Floating with containing style and direction arguments.
    // cpp: layoutng_style/style/computed_style.h:1465-1467
    pub fn FloatingWithContainingStyle(&self, cb_style: &Self) -> EFloat {
        self.FloatingWithDirection(cb_style.Direction())
    }

    // cpp: layoutng_style/style/computed_style.h:1469-1479
    pub fn FloatingWithDirection(&self, cb_direction: TextDirection) -> EFloat {
        let value = self.Floating();
        match value {
            EFloat::kInlineStart => {
                if cb_direction == TextDirection::kLtr {
                    EFloat::kLeft
                } else {
                    EFloat::kRight
                }
            }
            EFloat::kInlineEnd => {
                if cb_direction == TextDirection::kLtr {
                    EFloat::kRight
                } else {
                    EFloat::kLeft
                }
            }
            _ => value,
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1482
    pub fn HasBlendMode(&self) -> bool {
        self.GetBlendMode() != BlendMode::kNormal
    }

    // cpp: layoutng_style/style/computed_style.h:1485-1488
    pub fn HasOffset(&self) -> bool {
        (!self.OffsetPosition().X().IsAuto() && !self.OffsetPosition().X().IsNone())
            || self.OffsetPath().GetNonNull().is_some()
    }

    // cpp: layoutng_style/style/computed_style.h:1491-1493
    pub fn IsLeftToRightDirection(&self) -> bool {
        self.Direction() == TextDirection::kLtr
    }

    // cpp: layoutng_style/style/computed_style.h:1496
    pub fn HasPerspective(&self) -> bool {
        self.Perspective() >= 0.0
    }

    // cpp: layoutng_style/style/computed_style.h:1498-1501
    pub fn UsedPerspective(&self) -> f32 {
        debug_assert!(self.HasPerspective());
        self.Perspective().max(1.0)
    }

    // cpp: layoutng_style/style/computed_style.h:1506-1512
    pub fn HasOutline(&self) -> bool {
        if *self.OutlineWidth() <= 0 {
            return self.OutlineStyleIsAuto()
                && RuntimeEnabledFeatures::OutlineDrawAutoStyleZeroWidthEnabled();
        }
        (self.OutlineStyle() as i32) > (EBorderStyle::kHidden as i32)
    }

    // cpp: layoutng_style/style/computed_style.h:1513-1515
    pub fn HasOutlineWithCurrentColor(&self) -> bool {
        self.HasOutline() && self.OutlineColor().DependsOnCurrentColor()
    }

    // C++ overloads the static and instance position predicates.
    // cpp: layoutng_style/style/computed_style.h:1518-1520
    pub fn HasOutOfFlowPositionValue(position: EPosition) -> bool {
        position == EPosition::kAbsolute || position == EPosition::kFixed
    }

    // cpp: layoutng_style/style/computed_style.h:1521-1523
    pub fn HasOutOfFlowPosition(&self) -> bool {
        Self::HasOutOfFlowPositionValue(self.GetPosition())
    }

    // cpp: layoutng_style/style/computed_style.h:1524-1528
    pub fn HasStickyConstrainedPosition(&self) -> bool {
        self.GetPosition() == EPosition::kSticky
            && (!self.Top().IsAuto()
                || !self.Left().IsAuto()
                || !self.Right().IsAuto()
                || !self.Bottom().IsAuto())
    }

    // cpp: layoutng_style/style/computed_style.h:1529-1537
    pub fn GetPositionValue(display: EDisplay, position_internal: EPosition) -> EPosition {
        if position_internal == EPosition::kSticky
            && (display == EDisplay::kTableColumnGroup || display == EDisplay::kTableColumn)
        {
            EPosition::kStatic
        } else {
            position_internal
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1538-1540
    pub fn GetPosition(&self) -> EPosition {
        Self::GetPositionValue(self.Display(), self.PositionInternal())
    }

    // cpp: layoutng_style/style/computed_style.h:1542-1544
    pub fn GetDefaultAnchorData(&self) -> DefaultAnchorData {
        DefaultAnchorData::new(self.PositionAnchor(), *self.GetPositionArea())
    }

    // cpp: layoutng_style/style/computed_style.h:1547
    pub fn HasClear(&self) -> bool {
        self.Clear() != EClear::kNone
    }

    // cpp: layoutng_style/style/computed_style.h:1548
    pub fn UnresolvedClear(&self) -> EClear {
        self.Clear()
    }

    // C++ overloads Clear with containing style and direction arguments.
    // cpp: layoutng_style/style/computed_style.h:1550-1552
    pub fn ClearWithContainingStyle(&self, cb_style: &Self) -> EClear {
        self.ClearWithDirection(cb_style.Direction())
    }

    // cpp: layoutng_style/style/computed_style.h:1554-1564
    pub fn ClearWithDirection(&self, cb_direction: TextDirection) -> EClear {
        let value = self.Clear();
        match value {
            EClear::kInlineStart => {
                if cb_direction == TextDirection::kLtr {
                    EClear::kLeft
                } else {
                    EClear::kRight
                }
            }
            EClear::kInlineEnd => {
                if cb_direction == TextDirection::kLtr {
                    EClear::kRight
                } else {
                    EClear::kLeft
                }
            }
            _ => value,
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1567
    pub fn ClipLeft(&self) -> &Length {
        self.Clip().Left()
    }

    // cpp: layoutng_style/style/computed_style.h:1568
    pub fn ClipRight(&self) -> &Length {
        self.Clip().Right()
    }

    // cpp: layoutng_style/style/computed_style.h:1569
    pub fn ClipTop(&self) -> &Length {
        self.Clip().Top()
    }

    // cpp: layoutng_style/style/computed_style.h:1570
    pub fn ClipBottom(&self) -> &Length {
        self.Clip().Bottom()
    }

    // cpp: layoutng_style/style/computed_style.h:1574-1576
    pub fn HasAutoLeftAndRightIgnoringPositionArea(&self) -> bool {
        self.Left().IsAuto() && self.Right().IsAuto()
    }

    // cpp: layoutng_style/style/computed_style.h:1577-1579
    pub fn HasAutoTopAndBottomIgnoringPositionArea(&self) -> bool {
        self.Top().IsAuto() && self.Bottom().IsAuto()
    }

    // cpp: layoutng_style/style/computed_style.h:1584-1587
    pub fn IsTopInsetNonAuto(&self) -> bool {
        !self.Top().IsAuto()
            || self
                .PositionAreaOffsets()
                .as_ref()
                .is_some_and(|offsets| !offsets.behaves_as_auto.top)
    }

    // cpp: layoutng_style/style/computed_style.h:1588-1591
    pub fn IsRightInsetNonAuto(&self) -> bool {
        !self.Right().IsAuto()
            || self
                .PositionAreaOffsets()
                .as_ref()
                .is_some_and(|offsets| !offsets.behaves_as_auto.right)
    }

    // cpp: layoutng_style/style/computed_style.h:1592-1596
    pub fn IsBottomInsetNonAuto(&self) -> bool {
        !self.Bottom().IsAuto()
            || self
                .PositionAreaOffsets()
                .as_ref()
                .is_some_and(|offsets| !offsets.behaves_as_auto.bottom)
    }

    // cpp: layoutng_style/style/computed_style.h:1597-1600
    pub fn IsLeftInsetNonAuto(&self) -> bool {
        !self.Left().IsAuto()
            || self
                .PositionAreaOffsets()
                .as_ref()
                .is_some_and(|offsets| !offsets.behaves_as_auto.left)
    }

    // cpp: layoutng_style/style/computed_style.h:1702-1704
    pub fn IsContainerForSizeContainerQueries(&self) -> bool {
        self.IsInlineOrBlockSizeContainer() && self.StyleType() == PseudoId::kPseudoIdNone
    }

    // cpp: layoutng_style/style/computed_style.h:1706-1708
    pub fn IsContainerForScrollStateContainerQueries(&self) -> bool {
        self.IsScrollStateContainer() && self.StyleType() == PseudoId::kPseudoIdNone
    }

    // cpp: layoutng_style/style/computed_style.h:1710-1712
    pub fn IsContainerForAnchoredContainerQueries(&self) -> bool {
        self.IsAnchoredContainer() && self.StyleType() == PseudoId::kPseudoIdNone
    }

    // cpp: layoutng_style/style/computed_style.h:1714-1719
    pub fn DependsOnContainerQueries(&self) -> bool {
        self.DependsOnSizeContainerQueries()
            || self.DependsOnStyleContainerQueries()
            || self.DependsOnScrollStateContainerQueries()
            || self.DependsOnAnchoredContainerQueries()
    }

    // C++ overloads the static and instance content-visibility predicates.
    // cpp: layoutng_style/style/computed_style.h:1721-1724
    pub fn IsContentVisibilityVisibleValue(content_visibility: EContentVisibility) -> bool {
        content_visibility == EContentVisibility::kVisible
    }

    // cpp: layoutng_style/style/computed_style.h:1726-1728
    pub fn IsContentVisibilityVisible(&self) -> bool {
        Self::IsContentVisibilityVisibleValue(self.ContentVisibility())
    }

    // C++ overloads the static calculation and the instance query.
    // cpp: layoutng_style/style/computed_style.h:1617-1658
    pub fn EffectiveContainmentValue(
        contain: u32,
        container_type: u32,
        content_visibility: EContentVisibility,
        skips_contents: bool,
        has_size_containment_for_vt_scope: bool,
        overscroll_container_type: EOverscrollContainerType,
    ) -> u32 {
        let mut effective = contain;
        if container_type & EContainerType::kContainerTypeInlineSize.value() as u32 != 0 {
            effective |= Containment::kContainsStyle.value() as u32;
            effective |= Containment::kContainsInlineSize.value() as u32;
        }
        if container_type & EContainerType::kContainerTypeBlockSize.value() as u32 != 0 {
            effective |= Containment::kContainsStyle.value() as u32;
            effective |= Containment::kContainsBlockSize.value() as u32;
        }
        if container_type & EContainerType::kContainerTypeAnchored.value() as u32 != 0 {
            effective |= Containment::kContainsStyle.value() as u32;
        }
        if !Self::IsContentVisibilityVisibleValue(content_visibility) {
            effective |= Containment::kContainsStyle.value() as u32;
            effective |= Containment::kContainsLayout.value() as u32;
            effective |= Containment::kContainsPaint.value() as u32;
        }
        if skips_contents
            || (has_size_containment_for_vt_scope
                && RuntimeEnabledFeatures::ScopedViewTransitionSizeContainmentEnabled())
        {
            effective |= Containment::kContainsSize.value() as u32;
        }
        if overscroll_container_type != EOverscrollContainerType::kNone {
            effective |= Containment::kContainsLayout.value() as u32;
        }
        effective
    }

    // cpp: layoutng_style/style/computed_style.h:1660-1667
    pub fn EffectiveContainment(&self) -> u32 {
        Self::EffectiveContainmentValue(
            self.Contain(),
            self.ContainerType(),
            self.ContentVisibility(),
            self.SkipsContents(),
            self.HasSizeContainmentForViewTransitionScope()
                && RuntimeEnabledFeatures::ScopedViewTransitionSizeContainmentEnabled(),
            self.EffectiveOverscrollContainerType(),
        )
    }

    // cpp: layoutng_style/style/computed_style.h:1669
    pub fn ContainsStyle(&self) -> bool {
        self.EffectiveContainment() & Containment::kContainsStyle.value() as u32 != 0
    }

    // cpp: layoutng_style/style/computed_style.h:1670
    pub fn ContainsPaint(&self) -> bool {
        self.EffectiveContainment() & Containment::kContainsPaint.value() as u32 != 0
    }

    // cpp: layoutng_style/style/computed_style.h:1671-1673
    pub fn ContainsLayout(&self) -> bool {
        self.EffectiveContainment() & Containment::kContainsLayout.value() as u32 != 0
    }

    // cpp: layoutng_style/style/computed_style.h:1674-1676
    pub fn ContainsSize(&self) -> bool {
        let size = Containment::kContainsSize.value() as u32;
        self.EffectiveContainment() & size == size
    }

    // cpp: layoutng_style/style/computed_style.h:1677-1679
    pub fn ContainsInlineSize(&self) -> bool {
        self.EffectiveContainment() & Containment::kContainsInlineSize.value() as u32 != 0
    }

    // cpp: layoutng_style/style/computed_style.h:1680-1682
    pub fn ContainsBlockSize(&self) -> bool {
        self.EffectiveContainment() & Containment::kContainsBlockSize.value() as u32 != 0
    }

    // cpp: layoutng_style/style/computed_style.h:1683-1685
    pub fn ContainsAnySize(&self) -> bool {
        self.EffectiveContainment() & Containment::kContainsSize.value() as u32 != 0
    }

    // C++ declares this static predicate without a definition in the style package.
    // cpp: layoutng_style/style/computed_style.h:1687-1690
    pub fn ShouldApplyAnyContainmentValue(
        element: &Element,
        display_style: &DisplayStyle<'_>,
        effective_containment: u32,
    ) -> bool {
        unsafe {
            ComputedStyleShouldApplyAnyContainment(element, display_style, effective_containment)
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1692-1695
    pub fn ShouldApplyAnyContainment(&self, element: &Element) -> bool {
        let display_style = self.GetDisplayStyle();
        Self::ShouldApplyAnyContainmentValue(element, &display_style, self.EffectiveContainment())
    }

    // cpp: layoutng_style/style/computed_style.h:1697-1700
    pub fn CanMatchSizeContainerQueries(&self, element: &Element) -> bool {
        unsafe { ComputedStyleCanMatchSizeContainerQueries(self, element) }
    }

    // cpp: layoutng_style/style/computed_style.h:1731-1733
    pub fn IsInterleavingRoot(style: *const ComputedStyle) -> bool {
        unsafe { ComputedStyleIsInterleavingRoot(style) }
    }

    // cpp: layoutng_style/style/computed_style.h:2068-2072
    pub fn Has3DTransformOperation(&self) -> bool {
        unsafe { ComputedStyleHas3DTransformOperation(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:1231-1233
    pub fn IsTextIndentEachLine(&self) -> bool {
        self.GetTextIndentFlags().bits() & TextIndentFlags::kEachLine.bits() != 0
    }

    // cpp: layoutng_style/style/computed_style.h:1234-1236
    pub fn IsTextIndentHanging(&self) -> bool {
        self.GetTextIndentFlags().bits() & TextIndentFlags::kHanging.bits() != 0
    }

    // cpp: layoutng_style/style/computed_style.h:2632-2634
    fn IsInlineSizeContainer(&self) -> bool {
        self.ContainerType() & EContainerType::kContainerTypeInlineSize.value() as u32 != 0
    }

    // cpp: layoutng_style/style/computed_style.h:2635-2637
    fn IsBlockSizeContainer(&self) -> bool {
        self.ContainerType() & EContainerType::kContainerTypeBlockSize.value() as u32 != 0
    }

    // cpp: layoutng_style/style/computed_style.h:2638-2640
    fn IsInlineOrBlockSizeContainer(&self) -> bool {
        self.ContainerType() & EContainerType::kContainerTypeSize.value() as u32 != 0
    }

    // cpp: layoutng_style/style/computed_style.h:2641-2643
    fn IsSizeContainer(&self) -> bool {
        let size = EContainerType::kContainerTypeSize.value() as u32;
        self.ContainerType() & size == size
    }

    // cpp: layoutng_style/style/computed_style.h:2644-2646
    fn IsScrollStateContainer(&self) -> bool {
        self.ContainerType() & EContainerType::kContainerTypeScrollState.value() as u32 != 0
    }

    // cpp: layoutng_style/style/computed_style.h:2647-2649
    fn IsAnchoredContainer(&self) -> bool {
        self.ContainerType() & EContainerType::kContainerTypeAnchored.value() as u32 != 0
    }

    // cpp: layoutng_style/style/computed_style.h:1735-1737
    pub fn IsAtomicInlineDisplayType(&self) -> bool {
        Self::IsAtomicInlineDisplayTypeValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:1738
    pub fn IsDisplayInlineType(&self) -> bool {
        Self::IsDisplayInlineTypeValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:1739-1741
    pub fn IsNonAtomicInlineDisplayType(&self) -> bool {
        Self::IsNonAtomicInlineDisplayTypeValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:1742-1744
    pub fn IsDisplayBlockContainer(&self) -> bool {
        Self::IsDisplayBlockContainerValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:1745
    pub fn IsDisplayListItem(&self) -> bool {
        Self::IsDisplayListItemValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:1746
    pub fn IsDisplayTable(&self) -> bool {
        Self::IsDisplayTableValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:1751-1753
    pub fn IsDisplayLayoutCustom(&self) -> bool {
        Self::IsDisplayLayoutCustomValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:1755
    pub fn IsDisplayTableType(&self) -> bool {
        Self::IsDisplayTableTypeValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:1757
    pub fn IsDisplayMath(&self) -> bool {
        Self::IsDisplayMathValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:1759-1764
    pub fn BlockifiesChildren(&self) -> bool {
        self.IsDisplayFlex()
            || self.IsDisplayGrid()
            || self.IsDisplayGridLanes()
            || self.IsDisplayMath()
            || self.IsDisplayLayoutCustom()
            || (self.Display() == EDisplay::kContents && self.IsInBlockifyingDisplay())
            || self.ForcesBlockifiesChildren()
    }

    // cpp: layoutng_style/style/computed_style.h:1766-1779
    pub fn InlinifiesChildren(&self) -> bool {
        let display = self.Display();
        if display == EDisplay::kRuby
            || display == EDisplay::kBlockRuby
            || display == EDisplay::kRubyText
        {
            return true;
        }
        self.IsInInlinifyingDisplay()
            && (display == EDisplay::kContents
                || display == EDisplay::kInline
                || display == EDisplay::kInlineListItem)
    }

    // cpp: layoutng_style/style/computed_style.h:1782
    pub fn HasIsolation(&self) -> bool {
        self.Isolation() != EIsolation::kAuto
    }

    // cpp: layoutng_style/style/computed_style.h:1784-1786
    pub fn GetDisplayStyle(&self) -> DisplayStyle<'_> {
        let content_data = self.GetContentData().map(|data| unsafe { &*data });
        DisplayStyle::new(self.Display(), self.StyleType(), content_data)
    }

    // cpp: layoutng_style/style/computed_style.h:1789-1791
    pub fn ContentBehavesAsNormal(&self) -> bool {
        self.GetDisplayStyle().ContentBehavesAsNormal()
    }

    // cpp: layoutng_style/style/computed_style.h:1792-1794
    pub fn ContentPreventsBoxGeneration(&self) -> bool {
        self.GetDisplayStyle().ContentPreventsBoxGeneration()
    }

    // cpp: layoutng_style/style/computed_style.h:1797
    pub fn Cursors(&self) -> *mut CursorList {
        self.CursorDataInternal().Get()
    }

    // cpp: layoutng_style/style/computed_style.h:1800-1802
    pub fn HasResize(&self) -> bool {
        self.StyleType() == PseudoId::kPseudoIdNone && self.Resize() != EResize::kNone
    }

    // cpp: layoutng_style/style/computed_style.h:1803
    pub fn UnresolvedResize(&self) -> EResize {
        self.Resize()
    }

    // cpp: layoutng_style/style/computed_style.h:1805-1817
    pub fn UsedResize(&self) -> EResize {
        let value = self.Resize();
        match value {
            EResize::kBlock => {
                if self.IsHorizontalWritingMode() {
                    EResize::kVertical
                } else {
                    EResize::kHorizontal
                }
            }
            EResize::kInline => {
                if self.IsHorizontalWritingMode() {
                    EResize::kHorizontal
                } else {
                    EResize::kVertical
                }
            }
            _ => value,
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1820-1825
    pub fn UsedPointerEvents(&self) -> EPointerEvents {
        if self.IsInert() {
            return EPointerEvents::kNone;
        }
        self.PointerEvents()
    }

    // cpp: layoutng_style/style/computed_style.h:1828-1833
    pub fn UsedUserModify(&self) -> EUserModify {
        if self.IsInert() {
            return EUserModify::kReadOnly;
        }
        self.UserModify()
    }

    // cpp: layoutng_style/style/computed_style.h:1836-1841
    pub fn UsedUserSelect(&self) -> EUserSelect {
        if self.IsInert() {
            return EUserSelect::kNone;
        }
        self.UserSelect()
    }

    // cpp: layoutng_style/style/computed_style.h:1843-1846
    pub fn IsSelectable(&self) -> bool {
        !self.IsInert()
            && !(self.UserSelect() == EUserSelect::kNone
                && self.UserModify() == EUserModify::kReadOnly)
    }

    // cpp: layoutng_style/style/computed_style.h:1848-1855
    pub fn IsFocusable(&self) -> bool {
        !self.IsEnsuredInDisplayNone()
            && !self.IsInert()
            && self.Visibility() == EVisibility::kVisible
            && (self.Display() != EDisplay::kContents
                || RuntimeEnabledFeatures::DisplayContentsFocusableEnabled())
    }

    // cpp: layoutng_style/style/computed_style.h:1858-1862
    pub fn ShouldTextBoxTrimStart(&self) -> bool {
        let text_box_trim = self.TextBoxTrim();
        text_box_trim == ETextBoxTrim::kTrimStart || text_box_trim == ETextBoxTrim::kTrimBoth
    }

    // cpp: layoutng_style/style/computed_style.h:1863-1867
    pub fn ShouldTextBoxTrimEnd(&self) -> bool {
        let text_box_trim = self.TextBoxTrim();
        text_box_trim == ETextBoxTrim::kTrimEnd || text_box_trim == ETextBoxTrim::kTrimBoth
    }

    // cpp: layoutng_style/style/computed_style.h:1873-1876
    pub fn AppliedTextDecorationData(
        &self,
    ) -> *mut super::applied_text_decoration::AppliedTextDecorationVector {
        if self.IsDecoratingBox() {
            self.EnsureAppliedTextDecorationsCache()
        } else {
            self.BaseTextDecorationData()
        }
    }

    // The source compares a combinable TextDecorationLine value with kNone.
    // Its raw six-bit zero comparison is equivalent while the foundation
    // bitmask representation remains pending.
    // cpp: layoutng_style/style/computed_style.h:1880-1888
    pub fn IsDecoratingBox(&self) -> bool {
        if self.GetTextDecorationLineBits() == 0 {
            return false;
        }
        if self.Display() == EDisplay::kContents {
            return false;
        }
        true
    }

    // cpp: layoutng_style/style/computed_style.h:1891-1901
    pub fn HasAppliedTextDecorations(&self) -> bool {
        if self.IsDecoratingBox() {
            return true;
        }
        let base = self.BaseTextDecorationData();
        if !base.is_null() {
            debug_assert!(unsafe { (&*base).size() != 0 });
            return true;
        }
        false
    }

    // cpp: layoutng_style/style/computed_style.h:1870
    pub fn TextDecorationVisualOverflowChanged(&self, other: &ComputedStyle) -> bool {
        unsafe { ComputedStyleTextDecorationVisualOverflowChanged(self, other) }
    }

    // cpp: layoutng_style/style/computed_style.h:1872
    pub fn AppliedTextDecorations(
        &self,
    ) -> &super::applied_text_decoration::AppliedTextDecorationVector {
        // cpp: third_party/blink/renderer/core/style/computed_style.cc:2243-2259
        if !self.HasAppliedTextDecorations() {
            thread_local! {
                static EMPTY: std::cell::OnceCell<foundation::Persistent<super::applied_text_decoration::AppliedTextDecorationVector>> = std::cell::OnceCell::new();
            }
            let empty = EMPTY.with(|slot| {
                slot.get_or_init(|| {
                    foundation::Persistent::from_ptr(MakeGarbageCollected(
                        super::applied_text_decoration::AppliedTextDecorationVector::default(),
                    ))
                })
                .Get()
            });
            return unsafe { &*empty };
        }
        if !self.IsDecoratingBox() {
            return unsafe { &*self.BaseTextDecorationData() };
        }
        unsafe { &*self.EnsureAppliedTextDecorationsCache() }
    }

    // cpp: layoutng_style/style/computed_style.h:1902-1908
    pub fn LastAppliedTextDecoration(
        &self,
    ) -> Option<super::applied_text_decoration::AppliedTextDecoration> {
        if self.HasAppliedTextDecorations() {
            return self.AppliedTextDecorations().iter().last().cloned();
        }
        None
    }

    // cpp: layoutng_style/style/computed_style.h:2362
    pub fn BorderObscuresBackground(&self) -> bool {
        unsafe { ComputedStyleBorderObscuresBackground(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:2363-2365
    pub fn GetBorderEdgeInfo(&self, edges: &mut BorderEdgeArray, sides: PhysicalBoxSides) {
        unsafe { ComputedStyleGetBorderEdgeInfo(self, edges, sides) }
    }

    // C++ supplies PhysicalBoxSides() as the default second argument.
    // cpp: layoutng_style/style/computed_style.h:2363-2365
    pub fn GetBorderEdgeInfoAll(&self, edges: &mut BorderEdgeArray) {
        self.GetBorderEdgeInfo(edges, PhysicalBoxSides::default())
    }

    // cpp: layoutng_style/style/computed_style.h:2382
    pub fn BoxDecorationOutsets(&self) -> PhysicalBoxStrut {
        unsafe { ComputedStyleBoxDecorationOutsets(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:2386-2396
    pub fn HasBackgroundRelatedColorReferencingCurrentColor(&self) -> bool {
        if self.BackgroundColor().IsCurrentColor()
            || self.InternalVisitedBackgroundColor().IsCurrentColor()
            || self.InternalForcedBackgroundColor().IsCurrentColor()
        {
            return true;
        }
        let shadows = self.BoxShadow();
        if shadows.is_null() {
            return false;
        }
        Self::ShadowListHasCurrentColor(shadows)
    }

    // cpp: layoutng_style/style/computed_style.h:2401-2403
    pub fn VisitedDependentColor(
        &self,
        color_property: &Longhand,
        is_current_color: Option<&mut bool>,
    ) -> Color {
        unsafe { ComputedStyleVisitedDependentColor(self, color_property, is_current_color) }
    }

    // C++ overload with an explicitly supplied unvisited color.
    // cpp: layoutng_style/style/computed_style.h:2405-2408
    pub fn VisitedDependentColorWithUnvisited(
        &self,
        unvisited_color: &Color,
        color_property: &Longhand,
        is_current_color: Option<&mut bool>,
    ) -> Color {
        unsafe {
            ComputedStyleVisitedDependentColorWithUnvisited(
                self,
                unvisited_color,
                color_property,
                is_current_color,
            )
        }
    }

    // cpp: layoutng_style/style/computed_style.h:2410-2412
    pub fn VisitedDependentGapColor(&self, gap_color: &StyleColor, is_column_rule: bool) -> Color {
        unsafe { ComputedStyleVisitedDependentGapColor(self, gap_color, is_column_rule) }
    }

    // cpp: layoutng_style/style/computed_style.h:2414-2417
    pub fn VisitedDependentContextFill(
        &self,
        context_paint: &SVGPaint,
        context_style: &ComputedStyle,
    ) -> Color {
        unsafe { ComputedStyleVisitedDependentContextFill(self, context_paint, context_style) }
    }

    // cpp: layoutng_style/style/computed_style.h:2418-2420
    pub fn VisitedDependentContextStroke(
        &self,
        context_paint: &SVGPaint,
        context_style: &ComputedStyle,
    ) -> Color {
        unsafe { ComputedStyleVisitedDependentContextStroke(self, context_paint, context_style) }
    }

    // cpp: layoutng_style/style/computed_style.h:2422-2441
    pub fn VisitedDependentColorFast<P: VisitedDependentColorProperty>(
        &self,
        color_property: &P,
        is_current_color: Option<&mut bool>,
    ) -> Color {
        debug_assert!(!P::default().IsVisited());
        if self.InsideLink() != EInsideLink::kInsideVisitedLink {
            let color = color_property.ColorIncludingFallback(false, self, None);
            debug_assert!(
                color == self.VisitedDependentColor(color_property.as_ref(), is_current_color)
            );
            return color;
        }
        self.VisitedDependentColor(color_property.as_ref(), is_current_color)
    }

    // cpp: layoutng_style/style/computed_style.h:2819
    fn ShadowListHasCurrentColor(shadows: *const ShadowList) -> bool {
        unsafe { ComputedStyleShadowListHasCurrentColor(shadows) }
    }

    // cpp: layoutng_style/style/computed_style.h:1912-1914
    pub fn OverflowInlineDirection(&self) -> EOverflow {
        if self.IsHorizontalWritingMode() {
            self.OverflowX()
        } else {
            self.OverflowY()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1915-1917
    pub fn OverflowBlockDirection(&self) -> EOverflow {
        if self.IsHorizontalWritingMode() {
            self.OverflowY()
        } else {
            self.OverflowX()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1922-1926
    pub fn IsOverflowVisibleAlongBothAxes(&self) -> bool {
        self.OverflowX() == EOverflow::kVisible && self.OverflowY() == EOverflow::kVisible
    }

    // cpp: layoutng_style/style/computed_style.h:1929-1934
    pub fn IsOverflowVisibleOrClip(&self) -> bool {
        (self.OverflowX() == EOverflow::kVisible || self.OverflowX() == EOverflow::kClip)
            && (self.OverflowY() == EOverflow::kVisible || self.OverflowY() == EOverflow::kClip)
    }

    // cpp: layoutng_style/style/computed_style.h:1938-1940
    pub fn IsOverflowValueScrollable(overflow: EOverflow) -> bool {
        overflow != EOverflow::kVisible && overflow != EOverflow::kClip
    }

    // cpp: layoutng_style/style/computed_style.h:1945-1948
    pub fn IsScrollContainer(&self) -> bool {
        Self::IsOverflowValueScrollable(self.OverflowX())
            || Self::IsOverflowValueScrollable(self.OverflowY())
    }

    // cpp: layoutng_style/style/computed_style.h:1952-1954
    pub fn IsOverflowValueScrollableInline(&self) -> bool {
        Self::IsOverflowValueScrollable(self.OverflowInlineDirection())
    }

    // cpp: layoutng_style/style/computed_style.h:1958-1960
    pub fn IsOverflowValueScrollableBlock(&self) -> bool {
        Self::IsOverflowValueScrollable(self.OverflowBlockDirection())
    }

    // cpp: layoutng_style/style/computed_style.h:1964-1966
    pub fn IsOverflowValueScrollableX(&self) -> bool {
        Self::IsOverflowValueScrollable(self.OverflowX())
    }

    // cpp: layoutng_style/style/computed_style.h:1970-1972
    pub fn IsOverflowValueScrollableY(&self) -> bool {
        Self::IsOverflowValueScrollable(self.OverflowY())
    }

    // cpp: layoutng_style/style/computed_style.h:1974-1976
    pub fn HasAutoScroll(overflow: EOverflow) -> bool {
        overflow == EOverflow::kAuto || overflow == EOverflow::kOverlay
    }

    // C++ overloads the static and instance ScrollsOverflow functions.
    // cpp: layoutng_style/style/computed_style.h:1978-1980
    pub fn ScrollsOverflowValue(overflow: EOverflow) -> bool {
        overflow == EOverflow::kScroll || Self::HasAutoScroll(overflow)
    }

    // cpp: layoutng_style/style/computed_style.h:1982-1984
    pub fn HasAutoHorizontalScroll(&self) -> bool {
        Self::HasAutoScroll(self.OverflowX())
    }

    // cpp: layoutng_style/style/computed_style.h:1986-1988
    pub fn HasAutoVerticalScroll(&self) -> bool {
        Self::HasAutoScroll(self.OverflowY())
    }

    // cpp: layoutng_style/style/computed_style.h:1990-1992
    pub fn ScrollsOverflowX(&self) -> bool {
        Self::ScrollsOverflowValue(self.OverflowX())
    }

    // cpp: layoutng_style/style/computed_style.h:1994-1996
    pub fn ScrollsOverflowY(&self) -> bool {
        Self::ScrollsOverflowValue(self.OverflowY())
    }

    // cpp: layoutng_style/style/computed_style.h:1998-2000
    pub fn ScrollsOverflow(&self) -> bool {
        self.ScrollsOverflowX() || self.ScrollsOverflowY()
    }

    // cpp: layoutng_style/style/computed_style.h:2004
    pub fn IsInert(&self) -> bool {
        self.IsHTMLInert() || self.IsCSSInert()
    }

    // cpp: layoutng_style/style/computed_style.h:2007-2010
    pub fn VisibleToHitTesting(&self) -> bool {
        self.Visibility() == EVisibility::kVisible
            && self.UsedPointerEvents() != EPointerEvents::kNone
    }

    // cpp: layoutng_style/style/computed_style.h:2014-2016
    pub fn HasNonIdentityTransformOperation(&self) -> bool {
        self.HasTransformOperations() && !self.Transform().IsIdentityOrTranslation()
    }

    // cpp: layoutng_style/style/computed_style.h:2019-2022
    pub fn HasCurrentTransformRelatedAnimation(&self) -> bool {
        self.HasCurrentTransformAnimation()
            || self.HasCurrentScaleAnimation()
            || self.HasCurrentRotateAnimation()
            || self.HasCurrentTranslateAnimation()
    }

    // cpp: layoutng_style/style/computed_style.h:2023-2031
    pub fn HasCurrentCompositableAnimation(&self) -> bool {
        self.HasCurrentOpacityAnimation()
            || self.HasCurrentTransformRelatedAnimation()
            || self.HasCurrentFilterAnimation()
            || self.HasCurrentBackdropFilterAnimation()
            || (RuntimeEnabledFeatures::CompositeClipPathAnimationEnabled()
                && self.HasCurrentClipPathAnimation())
            || (RuntimeEnabledFeatures::CompositeBGColorAnimationEnabled()
                && self.HasCurrentBackgroundColorAnimation())
    }

    // cpp: layoutng_style/style/computed_style.h:2032-2038
    pub fn ShouldCompositeForCurrentAnimations(&self) -> bool {
        self.HasCurrentOpacityAnimation()
            || self.HasCurrentTransformRelatedAnimation()
            || self.HasCurrentFilterAnimation()
            || self.HasCurrentBackdropFilterAnimation()
            || (RuntimeEnabledFeatures::CompositeClipPathAnimationEnabled()
                && self.HasCurrentClipPathAnimation())
    }

    // cpp: layoutng_style/style/computed_style.h:2039-2044
    pub fn IsRunningTransformRelatedAnimationOnCompositor(&self) -> bool {
        self.IsRunningTransformAnimationOnCompositor()
            || self.IsRunningScaleAnimationOnCompositor()
            || self.IsRunningRotateAnimationOnCompositor()
            || self.IsRunningTranslateAnimationOnCompositor()
    }

    // cpp: layoutng_style/style/computed_style.h:2047
    pub fn HasOpacity(&self) -> bool {
        self.Opacity() < 1.0
    }

    // cpp: layoutng_style/style/computed_style.h:2050-2058
    pub fn IsFixedTableLayout(&self) -> bool {
        if RuntimeEnabledFeatures::TableIsAutoFixedLayoutEnabled() {
            return self.TableLayout() == ETableLayout::kFixed && !self.LogicalWidth().IsAuto();
        }
        self.TableLayout() == ETableLayout::kFixed
            && !self.LogicalWidth().HasAuto()
            && !self.LogicalWidth().HasMaxContent()
    }

    // cpp: layoutng_style/style/computed_style.h:2060-2066
    pub fn TableBorderSpacing(&self) -> LogicalSize {
        if self.BorderCollapse() == EBorderCollapse::kCollapse {
            return LogicalSize::default();
        }
        LogicalSize::new(
            LayoutUnit::from_signed(i32::from(self.HorizontalBorderSpacing())),
            LayoutUnit::from_signed(i32::from(self.VerticalBorderSpacing())),
        )
    }

    // cpp: layoutng_style/style/computed_style.h:2073-2077
    pub fn HasTransform(&self) -> bool {
        self.HasTransformOperations()
            || self.HasOffset()
            || self.HasCurrentTransformRelatedAnimation()
            || !self.Translate().is_null()
            || !self.Rotate().is_null()
            || !self.Scale().is_null()
    }

    // cpp: layoutng_style/style/computed_style.h:2078-2080
    pub fn HasTransformOperations(&self) -> bool {
        !self.Transform().Operations().is_empty()
    }

    // cpp: layoutng_style/style/computed_style.h:2081-2094
    pub fn UsedTransformStyle3D(&self) -> ETransformStyle3D {
        if self.TransformStyle3D() == ETransformStyle3D::kFlat {
            return ETransformStyle3D::kFlat;
        }
        debug_assert_eq!(self.TransformStyle3D(), ETransformStyle3D::kPreserve3d);
        if self.HasGroupingPropertyForUsedTransformStyle3D() {
            ETransformStyle3D::kFlat
        } else {
            ETransformStyle3D::kPreserve3d
        }
    }

    // cpp: layoutng_style/style/computed_style.h:2095-2097
    pub fn Preserves3D(&self) -> bool {
        self.UsedTransformStyle3D() != ETransformStyle3D::kFlat
    }

    // cpp: layoutng_style/style/computed_style.h:2132
    pub fn HasBoxReflect(&self) -> bool {
        !self.BoxReflect().is_null()
    }

    // cpp: layoutng_style/style/computed_style.h:2136-2138
    pub fn HasFilterInducingProperty(&self) -> bool {
        self.HasNonInitialFilter() || self.HasBoxReflect()
    }

    // cpp: layoutng_style/style/computed_style.h:2142-2144
    pub fn HasNonInitialFilter(&self) -> bool {
        self.HasFilter() || self.HasWillChangeProperty(CSSPropertyID::kFilter)
    }

    // cpp: layoutng_style/style/computed_style.h:2148-2151
    pub fn HasNonInitialBackdropFilter(&self) -> bool {
        self.HasBackdropFilter() || self.HasWillChangeProperty(CSSPropertyID::kBackdropFilter)
    }

    // cpp: layoutng_style/style/computed_style.h:2155-2158
    pub fn HasNonInitialOpacity(&self) -> bool {
        self.HasOpacity()
            || self.HasCurrentOpacityAnimation()
            || self.HasWillChangeProperty(CSSPropertyID::kOpacity)
    }

    // cpp: layoutng_style/style/computed_style.h:2166-2176
    pub fn HasGroupingProperty(&self, has_box_reflection: bool) -> bool {
        if self.HasStackingGroupingProperty(has_box_reflection) {
            return true;
        }
        if !self.HasAutoClip() && self.HasOutOfFlowPosition() {
            return true;
        }
        false
    }

    // cpp: layoutng_style/style/computed_style.h:2180-2209
    pub fn HasStackingGroupingProperty(&self, has_box_reflection: bool) -> bool {
        if self.HasNonInitialOpacity() {
            return true;
        }
        if self.HasNonInitialFilter() {
            return true;
        }
        if has_box_reflection {
            return true;
        }
        if self.HasClipPath() {
            return true;
        }
        if self.HasIsolation() {
            return true;
        }
        if self.HasMask() {
            return true;
        }
        if self.HasBlendMode() {
            return true;
        }
        if self.HasNonInitialBackdropFilter() {
            return true;
        }
        if !self.ViewTransitionName().Get().is_null() || self.ElementIsViewTransitionParticipant() {
            return true;
        }
        false
    }

    // cpp: layoutng_style/style/computed_style.h:2214-2217
    pub fn HasGroupingPropertyForUsedTransformStyle3D(&self) -> bool {
        self.HasGroupingProperty(self.HasBoxReflect()) || !self.IsOverflowVisibleAlongBothAxes()
    }

    // cpp: layoutng_style/style/computed_style.h:2225-2228
    pub fn HasTransformRelatedProperty(&self) -> bool {
        self.HasTransform()
            || self.Preserves3D()
            || self.HasPerspective()
            || self.HasWillChangeAnyTransformProperty()
    }

    // cpp: layoutng_style/style/computed_style.h:2229-2231
    pub fn HasTransformRelatedPropertyForSVG(&self) -> bool {
        self.HasTransform() || self.HasWillChangeAnyTransformProperty()
    }

    // cpp: layoutng_style/style/computed_style.h:2235-2237
    pub fn HasSVGEffect(&self) -> bool {
        self.HasFilter() || self.HasClipPath() || self.HasMask()
    }

    // cpp: layoutng_style/style/computed_style.h:2252-2254
    pub fn HasPseudoElementStyle(&self, pseudo: PseudoId) -> bool {
        PseudoIdFlags::FromBits(self.PseudoElementStylesInternal()).Has(pseudo)
    }

    // cpp: layoutng_style/style/computed_style.h:2244-2247
    pub fn HasVisualOverflowingEffect(&self) -> bool {
        !self.BoxShadow().is_null()
            || self.HasBorderImageOutsets()
            || self.HasOutline()
            || self.HasMaskBoxImageOutsets()
            || self.HasGapRule()
            || self.HasBorderShape()
    }

    // cpp: layoutng_style/style/computed_style.h:2258-2260
    pub fn WhiteSpace(&self) -> EWhiteSpace {
        white_space::ToWhiteSpace(self.GetWhiteSpaceCollapse(), self.GetTextWrapMode())
    }

    // cpp: layoutng_style/style/computed_style.h:2263-2265
    pub fn ShouldPreserveWhiteSpaces(&self) -> bool {
        white_space::ShouldPreserveWhiteSpaces(self.GetWhiteSpaceCollapse())
    }

    // cpp: layoutng_style/style/computed_style.h:2266-2268
    pub fn ShouldCollapseWhiteSpaces(&self) -> bool {
        white_space::ShouldCollapseWhiteSpaces(self.GetWhiteSpaceCollapse())
    }

    // cpp: layoutng_style/style/computed_style.h:2269-2271
    pub fn ShouldPreserveBreaks(&self) -> bool {
        white_space::ShouldPreserveBreaks(self.GetWhiteSpaceCollapse())
    }

    // cpp: layoutng_style/style/computed_style.h:2272-2274
    pub fn ShouldCollapseBreaks(&self) -> bool {
        white_space::ShouldCollapseBreaks(self.GetWhiteSpaceCollapse())
    }

    // C++ UChar is a UTF-16 code unit, represented here by u16.
    // cpp: layoutng_style/style/computed_style.h:2275-2284
    pub fn IsCollapsibleWhiteSpace(&self, c: u16) -> bool {
        match c {
            0x20 | 0x09 => self.ShouldCollapseWhiteSpaces(),
            0x0a => self.ShouldCollapseBreaks(),
            _ => false,
        }
    }

    // cpp: layoutng_style/style/computed_style.h:2286-2288
    pub fn ShouldWrapLine(&self) -> bool {
        white_space::ShouldWrapLine(self.GetTextWrapMode())
    }

    // cpp: layoutng_style/style/computed_style.h:2289-2291
    pub fn ShouldWrapLineGreedy(&self) -> bool {
        white_space::ShouldWrapLineGreedy(self.GetTextWrapStyle())
    }

    // cpp: layoutng_style/style/computed_style.h:2292-2294
    pub fn ShouldBreakSpaces(&self) -> bool {
        white_space::ShouldBreakSpaces(self.GetWhiteSpaceCollapse())
    }

    // cpp: layoutng_style/style/computed_style.h:2295-2298
    pub fn ShouldBreakOnlyAfterWhiteSpace(&self) -> bool {
        (self.ShouldPreserveWhiteSpaces() && self.ShouldWrapLine())
            || self.GetLineBreak() == LineBreak::kAfterWhiteSpace
    }

    // cpp: layoutng_style/style/computed_style.h:2299-2301
    pub fn NeedsTrailingSpace(&self) -> bool {
        self.ShouldBreakOnlyAfterWhiteSpace() && self.ShouldWrapLine()
    }

    // cpp: layoutng_style/style/computed_style.h:2303-2307
    pub fn ShouldBreakWords(&self) -> bool {
        (self.WordBreak() == EWordBreak::kBreakWord
            || self.OverflowWrap() != EOverflowWrap::kNormal)
            && self.ShouldWrapLine()
    }

    // cpp: layoutng_style/style/computed_style.h:2310-2312
    pub fn ShouldPlaceBlockDirectionScrollbarOnLogicalLeft(&self) -> bool {
        !self.IsLeftToRightDirection() && self.IsHorizontalWritingMode()
    }

    // cpp: layoutng_style/style/computed_style.h:2367-2372
    pub fn HasBoxDecorations(&self) -> bool {
        self.HasBorderDecoration()
            || self.HasBorderRadius()
            || self.HasOutline()
            || self.HasEffectiveAppearance()
            || !self.BoxShadow().is_null()
            || self.HasFilterInducingProperty()
            || self.HasNonInitialBackdropFilter()
            || self.HasResize()
    }

    // cpp: layoutng_style/style/computed_style.h:2377-2380
    pub fn HasBoxDecorationBackground(&self) -> bool {
        self.HasBackground()
            || self.HasBorderDecoration()
            || self.HasEffectiveAppearance()
            || !self.BoxShadow().is_null()
    }

    // cpp: layoutng_style/style/computed_style.h:2385
    pub fn BackgroundLayers(&self) -> &FillLayer {
        self.BackgroundInternal()
    }

    // C++ overloads the value and instance appearance predicates.
    // cpp: layoutng_style/style/computed_style.h:2444-2446
    pub fn HasEffectiveAppearanceValue(effective_appearance: AppearanceValue) -> bool {
        effective_appearance != AppearanceValue::kNone
    }

    // cpp: layoutng_style/style/computed_style.h:2447-2449
    pub fn HasEffectiveAppearance(&self) -> bool {
        Self::HasEffectiveAppearanceValue(self.EffectiveAppearance())
    }

    // cpp: layoutng_style/style/computed_style.h:2457-2510
    pub fn CanGeneratePseudoElement(&self, pseudo: PseudoId) -> bool {
        if self.Display() == EDisplay::kNone {
            return false;
        }
        if self.IsEnsuredInDisplayNone() {
            return false;
        }
        if pseudo == PseudoId::kPseudoIdMarker {
            return self.IsDisplayListItem()
                && (self.HasPseudoElementStyle(PseudoId::kPseudoIdMarker)
                    || !self.ListStyleType().Get().is_null()
                    || self.GeneratesMarkerImage());
        }
        if pseudo == PseudoId::kPseudoIdBackdrop && self.Overlay() == EOverlay::kNone {
            return false;
        }
        if pseudo == PseudoId::kPseudoIdOverscrollBackdrop
            && !self.IsInternalOverscrollPositionAuto()
        {
            return false;
        }
        if pseudo == PseudoId::kPseudoIdScrollMarkerGroupBefore {
            return self.HasScrollMarkerGroupBefore() && self.IsScrollContainer();
        }
        if pseudo == PseudoId::kPseudoIdScrollMarkerGroupAfter {
            return self.HasScrollMarkerGroupAfter() && self.IsScrollContainer();
        }
        if pseudo == PseudoId::kPseudoIdScrollButtonBlockStart
            || pseudo == PseudoId::kPseudoIdScrollButtonInlineStart
            || pseudo == PseudoId::kPseudoIdScrollButtonInlineEnd
            || pseudo == PseudoId::kPseudoIdScrollButtonBlockEnd
        {
            return self.HasPseudoElementStyle(PseudoId::kPseudoIdScrollButton);
        }
        if !self.HasPseudoElementStyle(pseudo) {
            return false;
        }
        if self.Display() != EDisplay::kContents {
            return true;
        }
        pseudo == PseudoId::kPseudoIdCheckMark
            || pseudo == PseudoId::kPseudoIdBefore
            || pseudo == PseudoId::kPseudoIdAfter
            || pseudo == PseudoId::kPseudoIdExpandIcon
            || pseudo == PseudoId::kPseudoIdPickerIcon
            || pseudo == PseudoId::kPseudoIdInterestButton
    }

    // cpp: layoutng_style/style/computed_style.h:2512-2514
    pub fn HasScrollMarkerGroupBefore(&self) -> bool {
        let group = self.GetScrollMarkerGroup();
        !group.is_null() && unsafe { (&*group).PositionBefore() }
    }

    // cpp: layoutng_style/style/computed_style.h:2516-2518
    pub fn HasScrollMarkerGroupAfter(&self) -> bool {
        let group = self.GetScrollMarkerGroup();
        !group.is_null() && unsafe { (&*group).PositionAfter() }
    }

    // cpp: layoutng_style/style/computed_style.h:2521-2527
    pub fn ScrollMarkerGroupMode(&self) -> Option<ScrollMarkerMode> {
        let group = self.GetScrollMarkerGroup();
        if group.is_null() {
            None
        } else {
            Some(unsafe { (&*group).Mode() })
        }
    }

    // cpp: layoutng_style/style/computed_style.h:2529
    pub fn ScrollMarkerGroupNone(&self) -> bool {
        self.GetScrollMarkerGroup().is_null()
    }

    // cpp: layoutng_style/style/computed_style.h:2531-2534
    pub fn ScrollMarkerGroupEqual(&self, other: &Self) -> bool {
        foundation::ValuesEquivalent(self.GetScrollMarkerGroup(), other.GetScrollMarkerGroup())
    }

    // cpp: layoutng_style/style/computed_style.h:2536-2538
    pub fn ScrollTargetGroupNone(&self) -> bool {
        self.ScrollTargetGroup() == EScrollTargetGroup::kNone
    }

    // cpp: layoutng_style/style/computed_style.h:2540-2543
    pub fn ScrollMarginStrut(&self) -> PhysicalBoxStrut {
        PhysicalBoxStrut::new(
            LayoutUnit::from_f32(self.ScrollMarginTop()),
            LayoutUnit::from_f32(self.ScrollMarginRight()),
            LayoutUnit::from_f32(self.ScrollMarginBottom()),
            LayoutUnit::from_f32(self.ScrollMarginLeft()),
        )
    }

    // cpp: layoutng_style/style/computed_style.h:2564-2567
    pub fn LogicalAspectRatio(&self) -> LogicalSize {
        debug_assert_ne!(self.AspectRatio().GetType(), EAspectRatioType::kAuto);
        ToLogicalSize(self.AspectRatio().GetLayoutRatio(), self.GetWritingMode())
    }

    // C++ overloads the value and instance color-scheme selectors.
    // cpp: layoutng_style/style/computed_style.h:2550-2553
    pub fn UsedColorSchemeValue(is_dark_color_scheme: bool) -> mojom::blink::ColorScheme {
        if is_dark_color_scheme {
            mojom::blink::ColorScheme::kDark
        } else {
            mojom::blink::ColorScheme::kLight
        }
    }

    // cpp: layoutng_style/style/computed_style.h:2554-2556
    pub fn UsedColorScheme(&self) -> mojom::blink::ColorScheme {
        Self::UsedColorSchemeValue(self.DarkColorScheme())
    }

    // cpp: layoutng_style/style/computed_style.h:2558-2562
    pub fn GeneratesMarkerImage(&self) -> bool {
        if !self.IsDisplayListItem() {
            return false;
        }
        let image = self.ListStyleImage().Get();
        !image.is_null()
            && !unsafe { (&*image).ErrorOccurred() }
            && (unsafe { (&*image).IsLoading() } || unsafe { (&*image).IsLoaded() })
    }

    // cpp: layoutng_style/style/computed_style.h:2576
    pub fn ForceDark(&self) -> bool {
        self.DarkColorScheme() && self.ColorSchemeForced()
    }

    // cpp: layoutng_style/style/computed_style.h:2569-2574
    pub fn BoxSizingForAspectRatio(&self) -> EBoxSizing {
        if self.AspectRatio().GetType() == EAspectRatioType::kRatio {
            self.BoxSizing()
        } else {
            EBoxSizing::kContentBox
        }
    }

    // cpp: layoutng_style/style/computed_style.h:2578-2581
    pub fn HasStaticViewportUnits(&self) -> bool {
        self.ViewportUnitFlags() & ViewportUnitFlag::kStatic as u32 != 0
    }

    // cpp: layoutng_style/style/computed_style.h:2582-2585
    pub fn HasDynamicViewportUnits(&self) -> bool {
        self.ViewportUnitFlags() & ViewportUnitFlag::kDynamic as u32 != 0
    }

    // cpp: layoutng_style/style/computed_style.h:2600-2603
    pub fn HasPositionVisibility(&self, visibility: PositionVisibility) -> bool {
        (self.GetPositionVisibility().value() & visibility.value()) == visibility.value()
    }

    // cpp: layoutng_style/style/computed_style.h:2595-2598
    // The body is owned by layoutng/internal/form_control_sizing_service.cc.
    // Its Rust extension trait is implemented in that package so this crate
    // does not invent a distinct Node identity or unresolved forwarding body.

    // cpp: layoutng_style/style/computed_style.h:2605-2607
    pub fn HasAnimationTrigger(&self) -> bool {
        unsafe { ComputedStyleHasAnimationTrigger(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:2609
    pub fn HasBaseEffectiveAppearance(&self) -> bool {
        unsafe { ComputedStyleHasBaseEffectiveAppearance(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:2624-2626
    pub fn IsInternalOverscrollPositionAuto(&self) -> bool {
        self.InternalOverscrollPosition() == EInternalOverscrollPosition::kAuto
    }

    // cpp: layoutng_style/style/computed_style.h:2611-2616
    pub fn EffectiveOverscrollContainerType(&self) -> EOverscrollContainerType {
        if self.InternalOverscrollContainer() == EInternalOverscrollContainer::kNone {
            EOverscrollContainerType::kNone
        } else {
            self.OverscrollContainerType()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:2618-2622
    pub fn IsContentMovingOverscrollContainer(&self) -> bool {
        let type_ = self.EffectiveOverscrollContainerType();
        type_ == EOverscrollContainerType::kAuto || type_ == EOverscrollContainerType::kPush
    }

    // cpp: layoutng_style/style/computed_style.h:2627-2629
    pub fn IsUnboundedElementActive(&self) -> bool {
        self.InternalUnbounded() == EInternalUnbounded::kActive
    }

    // cpp: layoutng_style/style/computed_style.h:2737-2760
    pub fn BorderOutlineVisitedColorChanged(&self, other: &Self) -> bool {
        if self.BorderTopWidth() != 0
            && self.InternalVisitedBorderTopColor() != other.InternalVisitedBorderTopColor()
        {
            return true;
        }
        if self.BorderRightWidth() != 0
            && self.InternalVisitedBorderRightColor() != other.InternalVisitedBorderRightColor()
        {
            return true;
        }
        if self.BorderBottomWidth() != 0
            && self.InternalVisitedBorderBottomColor() != other.InternalVisitedBorderBottomColor()
        {
            return true;
        }
        if self.BorderLeftWidth() != 0
            && self.InternalVisitedBorderLeftColor() != other.InternalVisitedBorderLeftColor()
        {
            return true;
        }
        if *self.OutlineWidth() != 0
            && self.InternalVisitedOutlineColor() != other.InternalVisitedOutlineColor()
        {
            return true;
        }
        false
    }

    // cpp: layoutng_style/style/computed_style.h:2764
    pub fn HasAppearance(&self) -> bool {
        self.Appearance() != AppearanceValue::kNone
    }

    // C++ overloads the static and instance force-color checks.
    // cpp: layoutng_style/style/computed_style.h:2850-2856
    pub fn ShouldForceColorValue(
        in_forced_colors_mode: bool,
        forced_color_adjust: EForcedColorAdjust,
        unforced_color: &StyleColor,
    ) -> bool {
        in_forced_colors_mode
            && forced_color_adjust == EForcedColorAdjust::kAuto
            && !unforced_color.IsSystemColorIncludingDeprecated()
    }

    // cpp: layoutng_style/style/computed_style.h:2857-2863
    pub fn ShouldForceColor(&self, unforced_color: &StyleColor) -> bool {
        Self::ShouldForceColorValue(
            self.InForcedColorsMode(),
            self.ForcedColorAdjust(),
            unforced_color,
        )
    }

    // cpp: layoutng_style/style/computed_style.h:2250
    // cpp: layoutng_style/style/computed_style.h:2883-2885
    pub fn HasAnyPseudoElementStyles(&self) -> bool {
        self.PseudoElementStylesInternal() != 0
    }

    // cpp: layoutng_style/style/computed_style.h:2251
    // cpp: layoutng_style/style/computed_style.h:2907-2911
    pub fn HasAnyHighlightPseudoElementStyles(&self) -> bool {
        let flags = PseudoIdFlags::FromBits(self.PseudoElementStylesInternal());
        flags.Has(PseudoId::kPseudoIdSelection)
            || flags.Has(PseudoId::kPseudoIdSearchText)
            || flags.Has(PseudoId::kPseudoIdTargetText)
            || flags.Has(PseudoId::kPseudoIdSpellingError)
            || flags.Has(PseudoId::kPseudoIdGrammarError)
            || flags.Has(PseudoId::kPseudoIdHighlight)
    }

    // C++ overloads the value and instance display predicates; Rust names the
    // value predicates explicitly and preserves the same comparisons.
    // cpp: layoutng_style/style/computed_style.h:1747
    pub fn IsDisplayFlex(&self) -> bool {
        Self::IsDisplayFlexValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:1748
    pub fn IsDisplayWebkitBox(&self) -> bool {
        Self::IsDisplayWebkitBoxValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:1749
    pub fn IsDisplayGrid(&self) -> bool {
        Self::IsDisplayGridValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:1750
    pub fn IsDisplayGridLanes(&self) -> bool {
        Self::IsDisplayGridLanesValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:2651-2659
    pub fn IsDisplayBlockContainerValue(display: EDisplay) -> bool {
        display == EDisplay::kBlock
            || display == EDisplay::kListItem
            || display == EDisplay::kInlineBlock
            || display == EDisplay::kFlowRoot
            || display == EDisplay::kFlowRootListItem
            || display == EDisplay::kInlineFlowRootListItem
            || display == EDisplay::kTableCell
            || display == EDisplay::kTableCaption
    }

    // cpp: layoutng_style/style/computed_style.h:2661-2666
    pub fn IsDisplayListItemValue(display: EDisplay) -> bool {
        display == EDisplay::kListItem
            || display == EDisplay::kInlineListItem
            || display == EDisplay::kFlowRootListItem
            || display == EDisplay::kInlineFlowRootListItem
    }

    // cpp: layoutng_style/style/computed_style.h:2668-2670
    pub fn IsDisplayTableValue(display: EDisplay) -> bool {
        display == EDisplay::kTable || display == EDisplay::kInlineTable
    }

    // cpp: layoutng_style/style/computed_style.h:2315-2317
    pub fn BorderStyleIsVisible(style: EBorderStyle) -> bool {
        style != EBorderStyle::kNone && style != EBorderStyle::kHidden
    }

    // cpp: layoutng_style/style/computed_style.h:2320-2339
    pub fn BorderStylesAreVisible(styles: &GapDataList<EBorderStyle>) -> bool {
        for style in styles.GetGapDataList().iter() {
            if !style.IsRepeaterData() {
                if Self::BorderStyleIsVisible(style.GetValue()) {
                    return true;
                }
            } else {
                for repeated_style in style.GetValueRepeater().RepeatedValues().iter() {
                    if Self::BorderStyleIsVisible(*repeated_style) {
                        return true;
                    }
                }
            }
        }
        false
    }

    // cpp: layoutng_style/style/computed_style.h:2341-2360
    pub fn HasRuleWidth(widths: &GapDataList<i32>) -> bool {
        for width in widths.GetGapDataList().iter() {
            if !width.IsRepeaterData() {
                if width.GetValue() != 0 {
                    return true;
                }
            } else {
                for repeated_width in width.GetValueRepeater().RepeatedValues().iter() {
                    if *repeated_width != 0 {
                        return true;
                    }
                }
            }
        }
        false
    }

    // cpp: layoutng_style/style/computed_style.h:2672-2674
    pub fn IsDisplayFlexValue(display: EDisplay) -> bool {
        display == EDisplay::kFlex || display == EDisplay::kInlineFlex
    }

    // cpp: layoutng_style/style/computed_style.h:2676-2679
    pub fn IsDisplayWebkitBoxValue(display: EDisplay) -> bool {
        display == EDisplay::kWebkitBox || display == EDisplay::kWebkitInlineBox
    }

    // cpp: layoutng_style/style/computed_style.h:2681-2683
    pub fn IsDisplayGridValue(display: EDisplay) -> bool {
        display == EDisplay::kGrid || display == EDisplay::kInlineGrid
    }

    // cpp: layoutng_style/style/computed_style.h:2685-2688
    pub fn IsDisplayGridLanesValue(display: EDisplay) -> bool {
        display == EDisplay::kGridLanes || display == EDisplay::kInlineGridLanes
    }

    // cpp: layoutng_style/style/computed_style.h:2690-2692
    pub fn IsDisplayMathValue(display: EDisplay) -> bool {
        display == EDisplay::kMath || display == EDisplay::kBlockMath
    }

    // cpp: layoutng_style/style/computed_style.h:2694-2697
    pub fn IsDisplayLayoutCustomValue(display: EDisplay) -> bool {
        display == EDisplay::kLayoutCustom || display == EDisplay::kInlineLayoutCustom
    }

    // cpp: layoutng_style/style/computed_style.h:2699-2708
    pub fn IsAtomicInlineDisplayTypeValue(display: EDisplay) -> bool {
        display == EDisplay::kInlineBlock
            || display == EDisplay::kInlineFlex
            || display == EDisplay::kInlineFlowRootListItem
            || display == EDisplay::kInlineGrid
            || display == EDisplay::kInlineLayoutCustom
            || display == EDisplay::kInlineGridLanes
            || display == EDisplay::kInlineTable
            || display == EDisplay::kMath
            || display == EDisplay::kWebkitInlineBox
    }

    // cpp: layoutng_style/style/computed_style.h:2710-2713
    pub fn IsNonAtomicInlineDisplayTypeValue(display: EDisplay) -> bool {
        display == EDisplay::kInline
            || display == EDisplay::kInlineListItem
            || display == EDisplay::kRuby
    }

    // cpp: layoutng_style/style/computed_style.h:2715-2718
    pub fn IsDisplayInlineTypeValue(display: EDisplay) -> bool {
        Self::IsNonAtomicInlineDisplayTypeValue(display)
            || Self::IsAtomicInlineDisplayTypeValue(display)
    }

    // cpp: layoutng_style/style/computed_style.h:2720-2730
    pub fn IsDisplayTableTypeValue(display: EDisplay) -> bool {
        display == EDisplay::kTable
            || display == EDisplay::kInlineTable
            || display == EDisplay::kTableRowGroup
            || display == EDisplay::kTableHeaderGroup
            || display == EDisplay::kTableFooterGroup
            || display == EDisplay::kTableRow
            || display == EDisplay::kTableColumnGroup
            || display == EDisplay::kTableColumn
            || display == EDisplay::kTableCell
            || display == EDisplay::kTableCaption
    }

    // cpp: layoutng_style/style/computed_style.h:2586
    pub fn HasViewportUnits(&self) -> bool {
        self.ViewportUnitFlags() != 0
    }

    // cpp: layoutng_style/style/computed_style.h:2588-2593
    pub fn OverflowClipMarginHasAnEffect(&self) -> bool {
        self.OverflowClipMargin()
            .as_ref()
            .is_some_and(|clip_margin| {
                clip_margin.GetReferenceBox() != OverflowClipMarginReferenceBox::kPaddingBox
                    || clip_margin.GetMargin() != LayoutUnit::default()
            })
    }

    // cpp: layoutng_style/style/computed_style.h:2821-2826
    pub fn PhysicalMarginToLogical<'a>(&'a self, other: &Self) -> PhysicalToLogical<&'a Length> {
        PhysicalToLogical::new(
            other.GetWritingDirection(),
            self.MarginTop(),
            self.MarginRight(),
            self.MarginBottom(),
            self.MarginLeft(),
        )
    }

    // cpp: layoutng_style/style/computed_style.h:2828-2832
    pub fn PhysicalPaddingToLogical(&self) -> PhysicalToLogical<&Length> {
        PhysicalToLogical::new(
            self.GetWritingDirection(),
            self.PaddingTop(),
            self.PaddingRight(),
            self.PaddingBottom(),
            self.PaddingLeft(),
        )
    }

    // cpp: layoutng_style/style/computed_style.h:2834-2838
    pub fn PhysicalBorderWidthToLogical(&self) -> PhysicalToLogical<i32> {
        PhysicalToLogical::new(
            self.GetWritingDirection(),
            self.BorderTopWidth(),
            self.BorderRightWidth(),
            self.BorderBottomWidth(),
            self.BorderLeftWidth(),
        )
    }

    // cpp: layoutng_style/style/computed_style.h:2840-2844
    pub fn PhysicalBorderStyleToLogical(&self) -> PhysicalToLogical<EBorderStyle> {
        PhysicalToLogical::new(
            self.GetWritingDirection(),
            self.BorderTopStyle(),
            self.BorderRightStyle(),
            self.BorderBottomStyle(),
            self.BorderLeftStyle(),
        )
    }
}

#[allow(non_snake_case)]
impl ComputedStyleBuilder {
    // cpp: layoutng_style/style/computed_style.h:2957-2958
    pub fn AccessAnimations(&mut self) -> &mut CSSAnimationData {
        // cpp: core/style/computed_style.h:3002-3012.
        let own = self.has_own_animations_.get();
        let data = self.MutableAnimationsInternal();
        if !own {
            let value = unsafe { data.Get().as_ref() }.cloned().unwrap_or_default();
            *data = Member::from_ptr(MakeGarbageCollected(value));
        }
        let pointer = data.Get();
        self.has_own_animations_.set(true);
        unsafe { &mut *pointer }
    }

    // cpp: layoutng_style/style/computed_style.h:3467-3468
    pub fn AccessTransitions(&mut self) -> &mut CSSTransitionData {
        // cpp: core/style/computed_style.h:3526-3536.
        let own = self.has_own_transitions_.get();
        let data = self.MutableTransitionsInternal();
        if !own {
            let value = unsafe { data.Get().as_ref() }.cloned().unwrap_or_default();
            *data = Member::from_ptr(MakeGarbageCollected(value));
        }
        let pointer = data.Get();
        self.has_own_transitions_.set(true);
        unsafe { &mut *pointer }
    }

    // cpp: layoutng_style/style/computed_style.h:2990
    pub fn ClearBackgroundImage(&mut self) {
        unsafe { ComputedStyleBuilderClearBackgroundImage(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:3544-3545
    pub fn AddPaintImage(&mut self, image: *mut StyleImage) {
        unsafe { ComputedStyleBuilderAddPaintImage(self, image) }
    }

    // cpp: layoutng_style/style/computed_style.h:3565-3572
    pub fn SetUsedColorScheme(
        &mut self,
        flags: ColorSchemeFlags,
        preferred_color_scheme: crate::style::color_scheme::mojom::blink::PreferredColorScheme,
        force_dark: bool,
    ) {
        // cpp: core/style/computed_style.cc:3147-3191
        use crate::css::color_scheme_flags::ColorSchemeFlag as F;
        use crate::style::color_scheme::mojom::blink::PreferredColorScheme as P;
        let prefers_dark = preferred_color_scheme == P::kDark;
        let has_dark = flags & F::kDark as u8 != 0;
        let has_light = flags & F::kLight as u8 != 0;
        let has_only = flags & F::kOnly as u8 != 0;
        let dark_scheme = (has_dark && prefers_dark)
            || (has_dark && !has_light)
            || (force_dark && !has_only)
            || (force_dark && !prefers_dark);
        self.SetDarkColorScheme(dark_scheme);
        let forced_scheme = (!has_dark && dark_scheme) || (force_dark && !prefers_dark);
        self.SetColorSchemeForced(forced_scheme);
        self.SetColorSchemeFlagsIsNormal(flags == F::kNormal as u8);
    }

    // cpp: layoutng_style/style/computed_style.h:2953-2955
    pub fn HasPseudoElementStyle(&self, pseudo: PseudoId) -> bool {
        PseudoIdFlags::FromBits(self.PseudoElementStylesInternal()).Has(pseudo)
    }

    // cpp: layoutng_style/style/computed_style.h:2961-2963
    pub fn HasEffectiveAppearance(&self) -> bool {
        ComputedStyle::HasEffectiveAppearanceValue(self.EffectiveAppearance())
    }

    // cpp: layoutng_style/style/computed_style.h:2964-2969
    pub fn HasBaseAppearance(&self) -> bool {
        debug_assert!(
            RuntimeEnabledFeatures::AppearanceBaseEnabled()
                || self.Appearance() != AppearanceValue::kBase
        );
        self.Appearance() == AppearanceValue::kBaseSelect
            || self.Appearance() == AppearanceValue::kBase
    }

    // cpp: layoutng_style/style/computed_style.h:2972-2974
    pub fn MutableBackdropFilterOperations(&mut self) -> &mut FilterOperationVector {
        self.MutableBackdropFilterInternal().OperationsMut()
    }

    // cpp: layoutng_style/style/computed_style.h:2975-2977
    pub fn HasBackdropFilter(&self) -> bool {
        ComputedStyle::HasBackdropFilterOperations(self.BackdropFilter())
    }

    // cpp: layoutng_style/style/computed_style.h:2980
    pub fn AccessBackgroundLayers(&mut self) -> &mut FillLayer {
        self.MutableBackgroundInternal()
    }

    // cpp: layoutng_style/style/computed_style.h:2981-2986
    pub fn AdjustBackgroundLayers(&mut self) {
        if !self.BackgroundInternal().Next().is_null() {
            self.AccessBackgroundLayers().CullEmptyLayers();
            self.AccessBackgroundLayers().FillUnsetProperties();
        }
    }

    // cpp: layoutng_style/style/computed_style.h:2987-2989
    pub fn HasUrlBackgroundImage(&self) -> bool {
        self.BackgroundInternal().AnyLayerHasUrlImage()
    }

    // cpp: layoutng_style/style/computed_style.h:2993-2998
    pub fn SetBorderColorFrom(&mut self, other: &ComputedStyle) {
        self.SetBorderBottomColor(other.BorderBottomColor());
        self.SetBorderLeftColor(other.BorderLeftColor());
        self.SetBorderRightColor(other.BorderRightColor());
        self.SetBorderTopColor(other.BorderTopColor());
    }

    // cpp: layoutng_style/style/computed_style.h:3001-3004
    pub fn BorderTopWidth(&self) -> i32 {
        ComputedStyle::BorderWidth(self.BorderTopStyle(), *self.SpecifiedBorderTopWidth())
    }

    // cpp: layoutng_style/style/computed_style.h:3005-3008
    pub fn BorderBottomWidth(&self) -> i32 {
        ComputedStyle::BorderWidth(self.BorderBottomStyle(), *self.SpecifiedBorderBottomWidth())
    }

    // cpp: layoutng_style/style/computed_style.h:3009-3012
    pub fn BorderLeftWidth(&self) -> i32 {
        ComputedStyle::BorderWidth(self.BorderLeftStyle(), *self.SpecifiedBorderLeftWidth())
    }

    // cpp: layoutng_style/style/computed_style.h:3013-3016
    pub fn BorderRightWidth(&self) -> i32 {
        ComputedStyle::BorderWidth(self.BorderRightStyle(), *self.SpecifiedBorderRightWidth())
    }

    // cpp: layoutng_style/style/computed_style.h:3019-3024
    pub fn SetBorderImageOutset(&mut self, outset: &BorderImageLengthBox) {
        if self.BorderImage().Outset() == outset {
            return;
        }
        self.MutableBorderImageInternal().SetOutset(outset);
    }

    // cpp: layoutng_style/style/computed_style.h:3025-3030
    pub fn SetBorderImageSlices(&mut self, slices: &LengthBox) {
        if self.BorderImage().ImageSlices() == slices {
            return;
        }
        self.MutableBorderImageInternal().SetImageSlices(slices);
    }

    // cpp: layoutng_style/style/computed_style.h:3031-3036
    pub fn SetBorderImageSlicesFill(&mut self, fill: bool) {
        if self.BorderImage().Fill() == fill {
            return;
        }
        self.MutableBorderImageInternal().SetFill(fill);
    }

    // cpp: layoutng_style/style/computed_style.h:3037-3042
    pub fn SetBorderImageSource(&mut self, image: *mut StyleImage) {
        if self.BorderImage().GetImage() == image {
            return;
        }
        self.MutableBorderImageInternal().SetImage(image);
    }

    // cpp: layoutng_style/style/computed_style.h:3043-3048
    pub fn SetBorderImageWidth(&mut self, slices: &BorderImageLengthBox) {
        if self.BorderImage().BorderSlices() == slices {
            return;
        }
        self.MutableBorderImageInternal().SetBorderSlices(slices);
    }

    // cpp: layoutng_style/style/computed_style.h:3051-3054
    pub fn SetClip(&mut self, box_: &LengthBox) {
        self.SetHasAutoClipInternal(false);
        self.SetClipInternal(box_);
    }

    // cpp: layoutng_style/style/computed_style.h:3055-3058
    pub fn SetHasAutoClip(&mut self) {
        self.SetHasAutoClipInternal(true);
        self.SetClipInternal(&ComputedStyleInitialValues::InitialClip());
    }

    // cpp: layoutng_style/style/computed_style.h:3061-3064
    pub fn SetClipPath(&mut self, clip_path: Option<*mut dyn ClipPathOperation>) {
        self.SetHasClipPath(clip_path.is_some());
        self.SetClipPathInternal(match clip_path {
            Some(path) => Member::from_ptr(path),
            None => Member::default(),
        });
    }

    // cpp: layoutng_style/style/computed_style.h:3065
    pub fn MutableClipPath(&self) -> Option<*mut dyn ClipPathOperation> {
        self.ClipPathInternal()
            .GetNonNull()
            .map(|path| path.as_ptr())
    }

    // cpp: layoutng_style/style/computed_style.h:3068-3070
    pub fn GetCurrentColor(&self) -> Color {
        self.Color()
            .Resolve(Color::default(), self.UsedColorScheme(), None)
    }

    // cpp: layoutng_style/style/computed_style.h:3073-3076
    pub fn SetColumnCount(&mut self, count: u16) {
        self.SetHasAutoColumnCountInternal(false);
        self.SetColumnCountInternal(count.max(1));
    }

    // cpp: layoutng_style/style/computed_style.h:3077-3080
    pub fn SetHasAutoColumnCount(&mut self) {
        self.SetHasAutoColumnCountInternal(true);
        self.SetColumnCountInternal(ComputedStyleInitialValues::InitialColumnCount());
    }

    // cpp: layoutng_style/style/computed_style.h:3083-3086
    pub fn SetColumnRuleColor(&mut self, colors: &GapDataList<StyleColor>) {
        self.SetMaybeHasGapDecorations();
        self.SetColumnRuleColorInternal(colors);
    }

    // cpp: layoutng_style/style/computed_style.h:3089-3092
    pub fn SetRowRuleColor(&mut self, colors: &GapDataList<StyleColor>) {
        self.SetMaybeHasGapDecorations();
        self.SetRowRuleColorInternal(colors);
    }

    // cpp: layoutng_style/style/computed_style.h:3095-3098
    pub fn SetColumnRuleStyle(&mut self, styles: &GapDataList<EBorderStyle>) {
        self.SetMaybeHasGapDecorations();
        self.SetColumnRuleStyleInternal(styles);
    }

    // cpp: layoutng_style/style/computed_style.h:3101-3104
    pub fn SetRowRuleStyle(&mut self, styles: &GapDataList<EBorderStyle>) {
        self.SetMaybeHasGapDecorations();
        self.SetRowRuleStyleInternal(styles);
    }

    // cpp: layoutng_style/style/computed_style.h:3107-3110
    pub fn SetColumnRuleWidth(&mut self, widths: &GapDataList<i32>) {
        self.SetMaybeHasGapDecorations();
        self.SetColumnRuleWidthInternal(widths);
    }

    // cpp: layoutng_style/style/computed_style.h:3113-3116
    pub fn SetRowRuleWidth(&mut self, widths: &GapDataList<i32>) {
        self.SetMaybeHasGapDecorations();
        self.SetRowRuleWidthInternal(widths);
    }

    // cpp: layoutng_style/style/computed_style.h:3119-3122
    pub fn SetColumnWidth(&mut self, width: f32) {
        self.SetHasAutoColumnWidthInternal(false);
        self.SetColumnWidthInternal(width);
    }

    // cpp: layoutng_style/style/computed_style.h:3123-3126
    pub fn SetHasAutoColumnWidth(&mut self) {
        self.SetHasAutoColumnWidthInternal(true);
        self.SetColumnWidthInternal(0.0);
    }

    // cpp: layoutng_style/style/computed_style.h:3129-3132
    pub fn SetColumnHeight(&mut self, height: f32) {
        self.SetHasAutoColumnHeightInternal(false);
        self.SetColumnHeightInternal(height);
    }

    // cpp: layoutng_style/style/computed_style.h:3133-3136
    pub fn SetHasAutoColumnHeight(&mut self) {
        self.SetHasAutoColumnHeightInternal(true);
        self.SetColumnHeightInternal(0.0);
    }

    // cpp: layoutng_style/style/computed_style.h:3138-3142
    pub fn SetFontVariantEmoji(&mut self, emoji_variant: FontVariantEmoji) {
        let mut description = self.GetFontDescription().clone();
        description.SetVariantEmoji(emoji_variant);
        self.SetFontDescription(&description);
    }

    // cpp: layoutng_style/style/computed_style.h:3145-3154
    pub fn ShouldApplyAnyContainment(&self, element: &Element) -> bool {
        let effective_containment = ComputedStyle::EffectiveContainmentValue(
            self.Contain(),
            self.ContainerType(),
            self.ContentVisibility(),
            self.SkipsContents(),
            self.HasSizeContainmentForViewTransitionScope()
                && RuntimeEnabledFeatures::ScopedViewTransitionSizeContainmentEnabled(),
            self.EffectiveOverscrollContainerType(),
        );
        let display_style = self.GetDisplayStyle();
        ComputedStyle::ShouldApplyAnyContainmentValue(
            element,
            &display_style,
            effective_containment,
        )
    }

    // cpp: layoutng_style/style/computed_style.h:3157
    pub fn GetContentData(&self) -> Option<*mut dyn ContentData> {
        self.ContentInternal()
            .as_ref()
            .and_then(|content| content.GetNonNull().map(|data| data.as_ptr()))
    }

    // cpp: layoutng_style/style/computed_style.h:3160-3167
    pub fn AccessCounterDirectives(&mut self) -> &mut CounterDirectiveMap {
        self.MutableCounterDirectivesInternal()
            .get_or_insert_with(|| Box::new(CounterDirectiveMap::default()))
    }

    // cpp: layoutng_style/style/computed_style.h:3168-3174
    pub fn ClearIncrementDirectives(&mut self) {
        if let Some(map) = self.MutableCounterDirectivesInternal().as_mut() {
            for directive in map.values_mut() {
                directive.ClearIncrement();
            }
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3175-3181
    pub fn ClearResetDirectives(&mut self) {
        if let Some(map) = self.MutableCounterDirectivesInternal().as_mut() {
            for directive in map.values_mut() {
                directive.ClearReset();
            }
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3182-3188
    pub fn ClearSetDirectives(&mut self) {
        if let Some(map) = self.MutableCounterDirectivesInternal().as_mut() {
            for directive in map.values_mut() {
                directive.ClearSet();
            }
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3191-3199
    pub fn AddCursor(
        &mut self,
        image: *mut StyleImage,
        hot_spot_specified: bool,
        hot_spot: &foundation::Point,
    ) {
        if self.CursorDataInternal().Get().is_null() {
            self.SetCursorDataInternal(Member::from_ptr(foundation::MakeGarbageCollected(
                CursorList::default(),
            )));
        }
        unsafe { &mut *self.MutableCursorDataInternal().Get() }.push_back(CursorData::new(
            image,
            hot_spot_specified,
            hot_spot,
        ));
    }

    // cpp: layoutng_style/style/computed_style.h:3200
    pub fn SetCursorList(&mut self, list: *mut CursorList) {
        self.SetCursorDataInternal(Member::from_ptr(list));
    }

    // cpp: layoutng_style/style/computed_style.h:3201-3205
    pub fn ClearCursorList(&mut self) {
        if !self.CursorDataInternal().Get().is_null() {
            self.SetCursorDataInternal(Member::default());
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3206
    pub fn Cursors(&self) -> *mut CursorList {
        self.CursorDataInternal().Get()
    }

    // cpp: layoutng_style/style/computed_style.h:3209-3211
    pub fn IsDisplayInlineType(&self) -> bool {
        ComputedStyle::IsDisplayInlineTypeValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:3212-3214
    pub fn IsNonAtomicInlineDisplayType(&self) -> bool {
        ComputedStyle::IsNonAtomicInlineDisplayTypeValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:3215-3217
    pub fn IsAtomicInlineDisplayType(&self) -> bool {
        ComputedStyle::IsAtomicInlineDisplayTypeValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:3218
    pub fn IsDisplayMath(&self) -> bool {
        ComputedStyle::IsDisplayMathValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:3219-3221
    pub fn IsDisplayTable(&self) -> bool {
        ComputedStyle::IsDisplayTableValue(self.Display())
    }

    // cpp: layoutng_style/style/computed_style.h:3222-3227
    pub fn IsDisplayTableRowOrColumnType(&self) -> bool {
        matches!(
            self.Display(),
            EDisplay::kTableRow
                | EDisplay::kTableRowGroup
                | EDisplay::kTableColumn
                | EDisplay::kTableColumnGroup
        )
    }

    // cpp: layoutng_style/style/computed_style.h:3228-3230
    pub fn GetDisplayStyle(&self) -> DisplayStyle<'_> {
        let content_data = self.GetContentData().map(|data| unsafe { &*data });
        DisplayStyle::new(self.Display(), self.StyleType(), content_data)
    }

    // cpp: layoutng_style/style/computed_style.h:3233-3235
    pub fn MutableFilterOperations(&mut self) -> &mut FilterOperationVector {
        self.MutableFilterInternal().OperationsMut()
    }

    // cpp: layoutng_style/style/computed_style.h:3238
    pub fn IsFloating(&self) -> bool {
        self.Floating() != EFloat::kNone
    }

    // cpp: layoutng_style/style/computed_style.h:3241-3246
    pub fn SetFontDescription(&mut self, description: &FontDescription) {
        let font = unsafe { &*self.GetFont() };
        if font.GetFontDescription() != description {
            let selector = font.GetFontSelector();
            self.SetFont(Member::from_ptr(MakeGarbageCollected(
                Font::new_with_selector(description.clone(), selector),
            )));
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3247-3249
    pub fn GetFontDescription(&self) -> &FontDescription {
        unsafe { &*self.GetFont() }.GetFontDescription()
    }

    // cpp: layoutng_style/style/computed_style.h:3250
    pub fn FontSize(&self) -> i32 {
        self.GetFontDescription().ComputedPixelSize()
    }

    // cpp: layoutng_style/style/computed_style.h:3254-3256
    pub fn GetFontSizeStyle(&self) -> FontSizeStyle<'_> {
        FontSizeStyle::new(self.GetFont(), self.LineHeight(), self.EffectiveZoom())
    }

    // cpp: layoutng_style/style/computed_style.h:3259-3261
    pub fn GridTemplateColumns(&self) -> &ComputedGridTrackList {
        ComputedStyle::ComputedGridTemplate(self.SpecifiedGridTemplateColumns())
    }

    // cpp: layoutng_style/style/computed_style.h:3263-3265
    pub fn GridTemplateRows(&self) -> &ComputedGridTrackList {
        ComputedStyle::ComputedGridTemplate(self.SpecifiedGridTemplateRows())
    }

    // cpp: layoutng_style/style/computed_style.h:3268-3273
    pub fn SetLetterSpacing(&mut self, letter_spacing: &Length) {
        let mut description = self.GetFontDescription().clone();
        description.SetLetterSpacing(letter_spacing);
        self.SetFontDescription(&description);
    }

    // cpp: layoutng_style/style/computed_style.h:3275-3277
    pub fn HasInitialLineHeight(&self) -> bool {
        self.LineHeight() == &ComputedStyleInitialValues::InitialLineHeight()
    }

    // cpp: layoutng_style/style/computed_style.h:3280-3287
    pub fn SetMarginTop(&mut self, value: &Length) {
        if self.MarginTop() != value {
            if !value.IsZero() || value.IsAuto() {
                self.SetMayHaveMargin();
            }
            *self.MutableMarginTopInternal() = value.clone();
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3288-3295
    pub fn SetMarginRight(&mut self, value: &Length) {
        if self.MarginRight() != value {
            if !value.IsZero() || value.IsAuto() {
                self.SetMayHaveMargin();
            }
            *self.MutableMarginRightInternal() = value.clone();
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3296-3303
    pub fn SetMarginBottom(&mut self, value: &Length) {
        if self.MarginBottom() != value {
            if !value.IsZero() || value.IsAuto() {
                self.SetMayHaveMargin();
            }
            *self.MutableMarginBottomInternal() = value.clone();
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3304-3311
    pub fn SetMarginLeft(&mut self, value: &Length) {
        if self.MarginLeft() != value {
            if !value.IsZero() || value.IsAuto() {
                self.SetMayHaveMargin();
            }
            *self.MutableMarginLeftInternal() = value.clone();
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3314
    pub fn AccessMaskLayers(&mut self) -> &mut FillLayer {
        self.MutableMaskInternal()
    }

    // cpp: layoutng_style/style/computed_style.h:3315-3320
    pub fn AdjustMaskLayers(&mut self) {
        if !self.MaskInternal().Next().is_null() {
            self.AccessMaskLayers().CullEmptyLayers();
            self.AccessMaskLayers().FillUnsetProperties();
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3323
    pub fn MaskBoxImage(&self) -> &NinePieceImage {
        self.MaskBoxImageInternal()
    }

    // cpp: layoutng_style/style/computed_style.h:3324
    pub fn SetMaskBoxImage(&mut self, image: &NinePieceImage) {
        self.SetMaskBoxImageInternal(image);
    }

    // cpp: layoutng_style/style/computed_style.h:3325-3327
    pub fn SetMaskBoxImageOutset(&mut self, outset: &BorderImageLengthBox) {
        self.MutableMaskBoxImageInternal().SetOutset(outset);
    }

    // cpp: layoutng_style/style/computed_style.h:3328-3330
    pub fn SetMaskBoxImageSlices(&mut self, slices: &LengthBox) {
        self.MutableMaskBoxImageInternal().SetImageSlices(slices);
    }

    // cpp: layoutng_style/style/computed_style.h:3331-3333
    pub fn SetMaskBoxImageSlicesFill(&mut self, fill: bool) {
        self.MutableMaskBoxImageInternal().SetFill(fill);
    }

    // cpp: layoutng_style/style/computed_style.h:3334-3336
    pub fn SetMaskBoxImageSource(&mut self, image: *mut StyleImage) {
        self.MutableMaskBoxImageInternal().SetImage(image);
    }

    // cpp: layoutng_style/style/computed_style.h:3337-3339
    pub fn SetMaskBoxImageWidth(&mut self, slices: &BorderImageLengthBox) {
        self.MutableMaskBoxImageInternal().SetBorderSlices(slices);
    }

    // cpp: layoutng_style/style/computed_style.h:3340-3342
    pub fn MaskBoxImageSource(&self) -> *mut StyleImage {
        self.MaskBoxImageInternal().GetImage()
    }

    // cpp: layoutng_style/style/computed_style.h:3345-3348
    pub fn SetOpacity(&mut self, opacity: f32) {
        self.SetOpacityInternal(opacity.clamp(0.0, 1.0));
    }

    // cpp: layoutng_style/style/computed_style.h:3351
    pub fn SetOrphans(&mut self, orphans: i16) {
        self.SetOrphansInternal(orphans.max(1));
    }

    // cpp: layoutng_style/style/computed_style.h:3354-3357
    pub fn ScrollsOverflow(&self) -> bool {
        ComputedStyle::ScrollsOverflowValue(self.OverflowX())
            || ComputedStyle::ScrollsOverflowValue(self.OverflowY())
    }

    // cpp: layoutng_style/style/computed_style.h:3360-3365
    pub fn EffectiveOverscrollContainerType(&self) -> EOverscrollContainerType {
        if self.InternalOverscrollContainer() == EInternalOverscrollContainer::kNone {
            EOverscrollContainerType::kNone
        } else {
            self.OverscrollContainerType()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3368-3375
    pub fn SetPaddingTop(&mut self, value: &Length) {
        if self.PaddingTop() != value {
            if !value.IsZero() {
                self.SetMayHavePadding();
            }
            *self.MutablePaddingTopInternal() = value.clone();
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3376-3383
    pub fn SetPaddingRight(&mut self, value: &Length) {
        if self.PaddingRight() != value {
            if !value.IsZero() {
                self.SetMayHavePadding();
            }
            *self.MutablePaddingRightInternal() = value.clone();
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3384-3391
    pub fn SetPaddingBottom(&mut self, value: &Length) {
        if self.PaddingBottom() != value {
            if !value.IsZero() {
                self.SetMayHavePadding();
            }
            *self.MutablePaddingBottomInternal() = value.clone();
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3392-3399
    pub fn SetPaddingLeft(&mut self, value: &Length) {
        if self.PaddingLeft() != value {
            if !value.IsZero() {
                self.SetMayHavePadding();
            }
            *self.MutablePaddingLeftInternal() = value.clone();
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3401-3408
    pub fn SetPageMarginSafety(&mut self, value: EPageMarginSafety) {
        if self.GetPageMarginSafety() != value {
            if value != EPageMarginSafety::kNone {
                self.SetMayHaveMargin();
            }
            self.SetPageMarginSafetyInternal(value);
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3411-3413
    pub fn SetPerspectiveOriginX(&mut self, value: &Length) {
        let origin = self.PerspectiveOrigin().clone();
        self.SetPerspectiveOrigin(&LengthPoint::new(value, origin.Y()));
    }

    // cpp: layoutng_style/style/computed_style.h:3414-3416
    pub fn SetPerspectiveOriginY(&mut self, value: &Length) {
        let origin = self.PerspectiveOrigin().clone();
        self.SetPerspectiveOrigin(&LengthPoint::new(origin.X(), value));
    }

    // cpp: layoutng_style/style/computed_style.h:3419-3421
    pub fn GetPosition(&self) -> EPosition {
        ComputedStyle::GetPositionValue(self.Display(), self.PositionInternal())
    }

    // cpp: layoutng_style/style/computed_style.h:3422-3424
    pub fn HasOutOfFlowPosition(&self) -> bool {
        ComputedStyle::HasOutOfFlowPositionValue(self.GetPosition())
    }

    // cpp: layoutng_style/style/computed_style.h:3426-3428
    pub fn GetDefaultAnchorData(&self) -> DefaultAnchorData {
        DefaultAnchorData::new(self.PositionAnchor(), *self.GetPositionArea())
    }

    // cpp: layoutng_style/style/computed_style.h:3431-3435
    pub fn SetShapeImageThreshold(&mut self, threshold: f32) {
        self.SetShapeImageThresholdInternal(threshold.clamp(0.0, 1.0));
    }

    // cpp: layoutng_style/style/computed_style.h:3438
    pub fn ShapeOutside(&self) -> *mut ShapeValue {
        self.ShapeOutsideInternal().Get()
    }

    // cpp: layoutng_style/style/computed_style.h:3441-3451
    pub fn SetTabSize(&mut self, tab_size: &TabSize) {
        if tab_size.GetPixelSize(1.0, 0.0, 0.0) < 0.0 {
            if tab_size.IsSpaces() {
                self.SetTabSizeInternal(&TabSize::new(0.0, TabSizeValueType::kSpace));
            } else {
                self.SetTabSizeInternal(&TabSize::new(0.0, TabSizeValueType::kLength));
            }
        } else {
            self.SetTabSizeInternal(tab_size);
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3454-3457
    pub fn SetTransformOriginX(&mut self, value: &Length) {
        let origin = self.GetTransformOrigin().clone();
        self.SetTransformOrigin(&TransformOrigin::new(value, origin.Y(), origin.Z()));
    }

    // cpp: layoutng_style/style/computed_style.h:3458-3461
    pub fn SetTransformOriginY(&mut self, value: &Length) {
        let origin = self.GetTransformOrigin().clone();
        self.SetTransformOrigin(&TransformOrigin::new(origin.X(), value, origin.Z()));
    }

    // cpp: layoutng_style/style/computed_style.h:3462-3465
    pub fn SetTransformOriginZ(&mut self, value: f32) {
        let origin = self.GetTransformOrigin().clone();
        self.SetTransformOrigin(&TransformOrigin::new(origin.X(), origin.Y(), value));
    }

    // cpp: layoutng_style/style/computed_style.h:3471-3474
    pub fn SetBoxOrdinalGroup(&mut self, ordinal_group: u32) {
        self.SetBoxOrdinalGroupInternal(ordinal_group.min(u32::MAX - 1));
    }

    // cpp: layoutng_style/style/computed_style.h:3477-3479
    pub fn SetVerticalAlign(&mut self, align: EVerticalAlign) {
        self.SetVerticalAlignInternal(align as u32);
    }

    // cpp: layoutng_style/style/computed_style.h:3480-3483
    pub fn SetVerticalAlignLength(&mut self, length: &Length) {
        self.SetVerticalAlignInternal(EVerticalAlign::kLength as u32);
        self.SetVerticalAlignLengthInternal(length);
    }

    // cpp: layoutng_style/style/computed_style.h:3486
    pub fn SetWidows(&mut self, widows: i16) {
        self.SetWidowsInternal(widows.max(1));
    }

    // cpp: layoutng_style/style/computed_style.h:3489-3493
    pub fn SetWordSpacing(&mut self, word_spacing: &Length) {
        let mut description = self.GetFontDescription().clone();
        description.SetWordSpacing(word_spacing);
        self.SetFontDescription(&description);
    }

    // cpp: layoutng_style/style/computed_style.h:3496-3499
    pub fn SetZIndex(&mut self, value: i32) {
        self.SetHasAutoZIndexInternal(false);
        self.SetZIndexInternal(value);
    }

    // cpp: layoutng_style/style/computed_style.h:3500-3503
    pub fn SetHasAutoZIndex(&mut self) {
        self.SetHasAutoZIndexInternal(true);
        self.SetZIndexInternal(0);
    }

    // cpp: layoutng_style/style/computed_style.h:3509-3514
    pub fn GetBaseComputedStyle(&self) -> *const ComputedStyle {
        let base_data = self.BaseData();
        if base_data.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*base_data }.GetBaseComputedStyle()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3517-3521
    pub fn AddCallbackSelector(&mut self, selector: &String) {
        if !self.CallbackSelectors().Contains(selector) {
            self.MutableCallbackSelectorsInternal()
                .push_back(selector.clone());
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3524-3530
    pub fn AddDocumentRulesSelector(&mut self, selector: *mut StyleRule) {
        if self.DocumentRulesSelectors().Get().is_null() {
            *self.MutableDocumentRulesSelectorsInternal() = Member::from_ptr(MakeGarbageCollected(
                GCedHeapHashSet::<WeakMember<StyleRule>>::default(),
            ));
        }
        unsafe { &mut *self.DocumentRulesSelectors().Get() }.insert(WeakMember::from_ptr(selector));
    }

    // cpp: layoutng_style/style/computed_style.h:3533-3535
    pub fn AccessHighlightData(&mut self) -> &mut StyleHighlightData {
        self.MutableHighlightDataInternal()
    }

    // cpp: layoutng_style/style/computed_style.h:3538-3542
    pub fn SetCustomHighlightNames(&mut self, names: &HashSet<AtomicString>) {
        self.SetCustomHighlightNamesInternal(Some(Box::new(names.clone())));
    }

    // cpp: layoutng_style/style/computed_style.h:3548-3551
    pub fn ShouldPreserveParentColor(&self) -> bool {
        self.InForcedColorsMode()
            && self.ForcedColorAdjust() == EForcedColorAdjust::kPreserveParentColor
    }

    // cpp: layoutng_style/style/computed_style.h:3552-3555
    pub fn ShouldForceColor(&self, unforced_color: &StyleColor) -> bool {
        ComputedStyle::ShouldForceColorValue(
            self.InForcedColorsMode(),
            self.ForcedColorAdjust(),
            unforced_color,
        )
    }

    // cpp: layoutng_style/style/computed_style.h:3557-3563
    pub fn InitialColorForColorScheme(&self) -> StyleColor {
        StyleColor::from_color(if self.DarkColorScheme() {
            Color::kWhite
        } else {
            Color::kBlack
        })
    }

    // cpp: layoutng_style/style/computed_style.h:3574-3576
    pub fn UsedColorScheme(&self) -> mojom::blink::ColorScheme {
        ComputedStyle::UsedColorSchemeValue(self.DarkColorScheme())
    }

    // cpp: layoutng_style/style/computed_style.h:3579-3581
    pub fn InheritedVariables(&self) -> &StyleInheritedVariables {
        self.InheritedVariablesInternal()
    }

    // cpp: layoutng_style/style/computed_style.h:3582-3584
    pub fn NonInheritedVariables(&self) -> &StyleNonInheritedVariables {
        self.NonInheritedVariablesInternal()
    }

    // cpp: layoutng_style/style/computed_style.h:3585-3586
    pub fn GetVariableData(
        &self,
        name: &AtomicString,
        is_inherited_property: bool,
    ) -> *mut CSSVariableData {
        unsafe { ComputedStyleBuilderGetVariableData(self, name, is_inherited_property) }
    }

    // The C++ out-of-line implementation is absent from this Bazel package.
    // cpp: layoutng_style/style/computed_style.h:3587
    pub fn MutableInheritedVariables(&mut self) -> &mut StyleInheritedVariables {
        unsafe { &mut *ComputedStyleBuilderMutableInheritedVariables(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:3588
    pub fn MutableNonInheritedVariables(&mut self) -> &mut StyleNonInheritedVariables {
        unsafe { &mut *ComputedStyleBuilderMutableNonInheritedVariables(self) }
    }

    // cpp: layoutng_style/style/computed_style.h:3589
    pub fn SetInheritedVariablesFrom(&mut self, style: *const ComputedStyle) {
        unsafe { ComputedStyleBuilderSetInheritedVariablesFrom(self, style) }
    }

    // cpp: layoutng_style/style/computed_style.h:3590
    pub fn SetNonInheritedVariablesFrom(&mut self, style: *const ComputedStyle) {
        unsafe { ComputedStyleBuilderSetNonInheritedVariablesFrom(self, style) }
    }

    // cpp: layoutng_style/style/computed_style.h:3591-3599
    pub fn SetVariableData(
        &mut self,
        name: &AtomicString,
        value: *mut CSSVariableData,
        is_inherited_property: bool,
    ) {
        if is_inherited_property {
            self.MutableInheritedVariables().SetData(name, value);
        } else {
            self.MutableNonInheritedVariables().SetData(name, value);
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3600-3608
    pub fn SetVariableValue(
        &mut self,
        name: &AtomicString,
        value: *const CSSValue,
        is_inherited_property: bool,
    ) {
        if is_inherited_property {
            self.MutableInheritedVariables().SetValue(name, value);
        } else {
            self.MutableNonInheritedVariables().SetValue(name, value);
        }
    }

    // cpp: layoutng_style/style/computed_style.h:3610-3612
    pub fn WhiteSpace(&self) -> EWhiteSpace {
        white_space::ToWhiteSpace(self.GetWhiteSpaceCollapse(), self.GetTextWrapMode())
    }

    // cpp: layoutng_style/style/computed_style.h:3613-3616
    pub fn SetWhiteSpace(&mut self, whitespace: EWhiteSpace) {
        self.SetWhiteSpaceCollapse(white_space::ToWhiteSpaceCollapse(whitespace));
        self.SetTextWrapMode(white_space::ToTextWrapMode(whitespace));
    }

    // cpp: layoutng_style/style/computed_style.h:3619-3621
    pub fn GetWritingDirection(&self) -> WritingDirectionMode {
        WritingDirectionMode::new(self.GetWritingMode(), self.Direction())
    }

    // cpp: layoutng_style/style/computed_style.h:3623-3626
    pub fn SetHasStaticViewportUnits(&mut self) {
        let flags = self.ViewportUnitFlags() | ViewportUnitFlag::kStatic as u32;
        self.SetViewportUnitFlags(flags);
    }

    // cpp: layoutng_style/style/computed_style.h:3627-3630
    pub fn SetHasDynamicViewportUnits(&mut self) {
        let flags = self.ViewportUnitFlags() | ViewportUnitFlag::kDynamic as u32;
        self.SetViewportUnitFlags(flags);
    }

    // cpp: layoutng_style/style/computed_style.h:3633-3641
    pub fn SetContainIntrinsicSizeAuto(&mut self) {
        let mut width = self.ContainIntrinsicWidth().clone();
        width.SetHasAuto();
        self.SetContainIntrinsicWidth(&width);
        let mut height = self.ContainIntrinsicHeight().clone();
        height.SetHasAuto();
        self.SetContainIntrinsicHeight(&height);
    }
}

unsafe extern "Rust" {
    // cpp: layoutng_style/style/computed_style.h:2846-2848
    // cpp: layoutng_style/style/computed_style.h:2455
    fn ComputedStyleGetInterpolationQuality(
        style: &ComputedStyle,
    ) -> foundation::InterpolationQuality;
    // cpp: layoutng_style/style/computed_style.h:2802-2803
    fn ComputedStyleGetInternalForcedCurrentColor(
        style: &ComputedStyle,
        is_current_color: Option<&mut bool>,
    ) -> Color;
    // cpp: layoutng_style/style/computed_style.h:2804-2805
    fn ComputedStyleGetInternalForcedVisitedCurrentColor(
        style: &ComputedStyle,
        is_current_color: Option<&mut bool>,
    ) -> Color;
    // cpp: layoutng_style/style/computed_style.h:2807-2809
    fn ComputedStyleVisitedDependentContextPaint(
        style: &ComputedStyle,
        context_paint: &SVGPaint,
        context_visited_paint: &SVGPaint,
    ) -> Color;
    // cpp: layoutng_style/style/computed_style.h:819
    fn ComputedStyleDependsOnFunc(
        style: &ComputedStyle,
        func: &dyn Fn(&ComputedStyle) -> bool,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:2762
    fn ComputedStyleDecorationColorIncludingFallback(
        style: &ComputedStyle,
        visited_link: bool,
    ) -> StyleColor;
    // cpp: layoutng_style/style/computed_style.h:2783
    fn ComputedStyleDiffNeedsReshape(
        style: &ComputedStyle,
        other: &ComputedStyle,
        field_diff: u64,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:2784-2785
    fn ComputedStyleDiffNeedsFullLayoutAndPaintInvalidation(
        style: &ComputedStyle,
        other: &ComputedStyle,
        field_diff: u64,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:2786-2787
    fn ComputedStyleDiffNeedsRecomputeVisualOverflow(
        style: &ComputedStyle,
        other: &ComputedStyle,
        field_diff: u64,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:2788-2789
    fn ComputedStyleDiffCompositingReasonsChanged(
        style: &ComputedStyle,
        other: &ComputedStyle,
        field_diff: u64,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:2790-2791
    fn ComputedStylePotentialCompositingReasonsFor3DTransformChanged(
        style: &ComputedStyle,
        other: &ComputedStyle,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:2793-2794
    fn ComputedStylePropertiesEqual(
        style: &ComputedStyle,
        properties: &foundation::Vector<CSSPropertyID>,
        other: &ComputedStyle,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:2795-2796
    fn ComputedStyleCustomPropertiesEqual(
        style: &ComputedStyle,
        properties: &foundation::Vector<AtomicString>,
        other: &ComputedStyle,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:821
    fn ComputedStyleHighlightPseudoElementStylesDependOnRelativeUnits(
        style: &ComputedStyle,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:822
    fn ComputedStyleHighlightPseudoElementStylesDependOnContainerUnits(
        style: &ComputedStyle,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:823
    fn ComputedStyleHighlightPseudoElementStylesHaveVariableReferences(
        style: &ComputedStyle,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:2735
    fn ComputedStyleHasPropertyDependingOnCurrentColor(style: &ComputedStyle) -> bool;
    // cpp: layoutng_style/style/computed_style.h:2867
    fn ComputedStyleScrollbarIsHiddenByCustomStyle(
        style: &ComputedStyle,
        element: *mut Element,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:728
    fn ComputedStyleUsedScrollbarColor(style: &ComputedStyle) -> *mut StyleScrollbarColor;
    // cpp: layoutng_style/style/computed_style.h:778
    fn ComputedStyleListStyleStringValue(style: &ComputedStyle) -> *const AtomicString;
    // cpp: layoutng_style/style/computed_style.h:787-789
    fn ComputedStyleMarkerShouldBeInside(
        style: &ComputedStyle,
        parent: &Element,
        marker_style: &DisplayStyle<'_>,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:1459
    fn ComputedStyleCanRenderBorderImage(style: &ComputedStyle) -> bool;
    // cpp: layoutng_style/style/computed_style.h:2548
    fn ComputedStyleIsRenderedInTopLayer(style: &ComputedStyle, element: &Element) -> bool;
    // cpp: layoutng_style/style/computed_style.h:2401-2403
    fn ComputedStyleVisitedDependentColor(
        style: &ComputedStyle,
        color_property: &Longhand,
        is_current_color: Option<&mut bool>,
    ) -> Color;
    // cpp: layoutng_style/style/computed_style.h:2405-2408
    fn ComputedStyleVisitedDependentColorWithUnvisited(
        style: &ComputedStyle,
        unvisited_color: &Color,
        color_property: &Longhand,
        is_current_color: Option<&mut bool>,
    ) -> Color;
    // cpp: layoutng_style/style/computed_style.h:2410-2412
    fn ComputedStyleVisitedDependentGapColor(
        style: &ComputedStyle,
        gap_color: &StyleColor,
        is_column_rule: bool,
    ) -> Color;
    // cpp: layoutng_style/style/computed_style.h:2414-2417
    fn ComputedStyleVisitedDependentContextFill(
        style: &ComputedStyle,
        context_paint: &SVGPaint,
        context_style: &ComputedStyle,
    ) -> Color;
    // cpp: layoutng_style/style/computed_style.h:2418-2420
    fn ComputedStyleVisitedDependentContextStroke(
        style: &ComputedStyle,
        context_paint: &SVGPaint,
        context_style: &ComputedStyle,
    ) -> Color;
    // cpp: layoutng_style/style/computed_style.h:326
    fn ComputedStyleGetVariableNamesCache(
        style: &ComputedStyle,
    ) -> *mut foundation::Vector<AtomicString>;
    // cpp: layoutng_style/style/computed_style.h:327
    fn ComputedStyleEnsureVariableNamesCache(
        style: &ComputedStyle,
    ) -> *mut foundation::Vector<AtomicString>;
    // cpp: layoutng_style/style/computed_style.h:1360
    fn ComputedStyleImageOutsets(style: &ComputedStyle, image: &NinePieceImage)
        -> PhysicalBoxStrut;
    // cpp: layoutng_style/style/computed_style.h:2605-2607
    fn ComputedStyleHasAnimationTrigger(style: &ComputedStyle) -> bool;
    // cpp: layoutng_style/style/computed_style.h:2609
    fn ComputedStyleHasBaseEffectiveAppearance(style: &ComputedStyle) -> bool;
    // cpp: layoutng_style/style/computed_style.h:2957-2958
    // cpp: layoutng_style/style/computed_style.h:2990
    fn ComputedStyleBuilderClearBackgroundImage(builder: &mut ComputedStyleBuilder);
    // cpp: layoutng_style/style/computed_style.h:3544-3545
    fn ComputedStyleBuilderAddPaintImage(
        builder: &mut ComputedStyleBuilder,
        image: *mut StyleImage,
    );
    // cpp: layoutng_style/style/computed_style.h:920-922
    fn ComputedStyleResolvedCaretTextColor(style: &ComputedStyle) -> Option<Color>;
    // cpp: layoutng_style/style/computed_style.h:924-926
    fn ComputedStyleAccentColorResolved(style: &ComputedStyle) -> Option<Color>;
    // cpp: layoutng_style/style/computed_style.h:928-931
    fn ComputedStyleScrollbarThumbColorResolved(style: &ComputedStyle) -> Option<Color>;
    fn ComputedStyleScrollbarTrackColorResolved(style: &ComputedStyle) -> Option<Color>;
    // cpp: layoutng_style/style/computed_style.h:933-936
    // cpp: layoutng_style/style/computed_style.h:938
    // cpp: layoutng_style/style/computed_style.h:939
    // cpp: layoutng_style/style/computed_style.h:940
    // cpp: layoutng_style/style/computed_style.h:941
    // cpp: layoutng_style/style/computed_style.h:942
    // cpp: layoutng_style/style/computed_style.h:953
    fn ComputedStyleCopyChildDependentFlagsFrom(style: &ComputedStyle, other: &ComputedStyle);
    // cpp: layoutng_style/style/computed_style.h:971-972
    fn ComputedStyleHasVariables(style: &ComputedStyle) -> bool;
    // cpp: layoutng_style/style/computed_style.h:973
    fn ComputedStyleGetVariableNamesCount(style: &ComputedStyle) -> u32;
    // cpp: layoutng_style/style/computed_style.h:974
    fn ComputedStyleGetVariableNames(
        style: &ComputedStyle,
    ) -> *const foundation::Vector<AtomicString>;
    // cpp: layoutng_style/style/computed_style.h:975
    fn ComputedStyleInheritedVariables(style: &ComputedStyle) -> *const StyleInheritedVariables;
    // cpp: layoutng_style/style/computed_style.h:976
    fn ComputedStyleNonInheritedVariables(
        style: &ComputedStyle,
    ) -> *const StyleNonInheritedVariables;
    // cpp: layoutng_style/style/computed_style.h:978-979
    fn ComputedStyleGetVariableData(
        style: &ComputedStyle,
        name: &AtomicString,
    ) -> *mut CSSVariableData;
    // cpp: layoutng_style/style/computed_style.h:980-981
    fn ComputedStyleGetVariableDataWithInheritance(
        style: &ComputedStyle,
        name: &AtomicString,
        is_inherited_property: bool,
    ) -> *mut CSSVariableData;
    // cpp: layoutng_style/style/computed_style.h:983
    fn ComputedStyleGetVariableValue(style: &ComputedStyle, name: &AtomicString)
        -> *const CSSValue;
    // cpp: layoutng_style/style/computed_style.h:984-985
    fn ComputedStyleGetVariableValueWithInheritance(
        style: &ComputedStyle,
        name: &AtomicString,
        is_inherited_property: bool,
    ) -> *const CSSValue;
    // cpp: layoutng_style/style/computed_style.h:410-412
    // cpp: layoutng_style/style/computed_style.h:414-417
    fn ComputedStyleDiffAffectsContainerQueries(
        old_style: *const ComputedStyle,
        new_style: *const ComputedStyle,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:419-422
    fn ComputedStyleNeedsReattachLayoutTree(
        element: &Element,
        old_style: *const ComputedStyle,
        new_style: *const ComputedStyle,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:424-427
    fn ComputedStyleNeedsReinsertLayoutTree(
        old_style: &ComputedStyle,
        new_style: &ComputedStyle,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:1603
    fn ComputedStyleContentDataEquivalent(style: &ComputedStyle, other: &ComputedStyle) -> bool;
    // cpp: layoutng_style/style/computed_style.h:2362
    fn ComputedStyleBorderObscuresBackground(style: &ComputedStyle) -> bool;
    // cpp: layoutng_style/style/computed_style.h:2363-2365
    fn ComputedStyleGetBorderEdgeInfo(
        style: &ComputedStyle,
        edges: &mut BorderEdgeArray,
        sides: PhysicalBoxSides,
    );
    // cpp: layoutng_style/style/computed_style.h:2382
    fn ComputedStyleBoxDecorationOutsets(style: &ComputedStyle) -> PhysicalBoxStrut;
    // cpp: layoutng_style/style/computed_style.h:2819
    fn ComputedStyleShadowListHasCurrentColor(shadows: *const ShadowList) -> bool;
    // cpp: layoutng_style/style/computed_style.h:1870
    fn ComputedStyleTextDecorationVisualOverflowChanged(
        style: &ComputedStyle,
        other: &ComputedStyle,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:1872
    // cpp: layoutng_style/style/computed_style.h:1687-1690
    fn ComputedStyleShouldApplyAnyContainment(
        element: &Element,
        display_style: &DisplayStyle<'_>,
        effective_containment: u32,
    ) -> bool;
    // cpp: layoutng_style/style/computed_style.h:1697-1700
    fn ComputedStyleCanMatchSizeContainerQueries(style: &ComputedStyle, element: &Element) -> bool;
    // cpp: layoutng_style/style/computed_style.h:1731-1733
    fn ComputedStyleIsInterleavingRoot(style: *const ComputedStyle) -> bool;
    // cpp: layoutng_style/style/computed_style.h:2068-2072
    fn ComputedStyleHas3DTransformOperation(style: &ComputedStyle) -> bool;
    // cpp: layoutng_style/style/computed_style.h:456
    fn ComputedStyleGetBaseImportantSet(style: &ComputedStyle) -> *const CSSBitset;
    // cpp: layoutng_style/style/computed_style.h:3585-3586
    fn ComputedStyleBuilderGetVariableData(
        builder: &ComputedStyleBuilder,
        name: &AtomicString,
        is_inherited_property: bool,
    ) -> *mut CSSVariableData;
    // cpp: layoutng_style/style/computed_style.h:3587-3588
    fn ComputedStyleBuilderMutableInheritedVariables(
        builder: &mut ComputedStyleBuilder,
    ) -> *mut StyleInheritedVariables;
    fn ComputedStyleBuilderMutableNonInheritedVariables(
        builder: &mut ComputedStyleBuilder,
    ) -> *mut StyleNonInheritedVariables;
    // cpp: layoutng_style/style/computed_style.h:3589-3590
    fn ComputedStyleBuilderSetInheritedVariablesFrom(
        builder: &mut ComputedStyleBuilder,
        style: *const ComputedStyle,
    );
    fn ComputedStyleBuilderSetNonInheritedVariablesFrom(
        builder: &mut ComputedStyleBuilder,
        style: *const ComputedStyle,
    );
}

#[allow(non_snake_case)]
impl ComputedStyle {
    // cpp: third_party/blink/renderer/core/style/computed_style.cc:226-254
    fn PseudoElementStylesEqual(old_style: &Self, new_style: &Self) -> bool {
        if !old_style.HasAnyPseudoElementStyles() && !new_style.HasAnyPseudoElementStyles() {
            return true;
        }
        for value in
            PseudoId::kFirstPublicPseudoId.value()..=PseudoId::kLastTrackedPublicPseudoId.value()
        {
            let pseudo_id = PseudoId::from_bits(value);
            if (!old_style.HasPseudoElementStyle(pseudo_id)
                && !new_style.HasPseudoElementStyle(pseudo_id))
                || super::computed_style_constants::IsHighlightPseudoElement(pseudo_id)
            {
                continue;
            }
            let new_pseudo_style = new_style.GetCachedPseudoElementStyleWithoutArgument(pseudo_id);
            let Some(new_pseudo_style) = (unsafe { new_pseudo_style.as_ref() }) else {
                return false;
            };
            let old_pseudo_style = old_style.GetCachedPseudoElementStyleWithoutArgument(pseudo_id);
            if unsafe { old_pseudo_style.as_ref() }.is_some_and(|old| old != new_pseudo_style) {
                return false;
            }
        }
        true
    }
}

impl Eq for super::style_cached_data::PseudoElementStyleCacheKey {}
impl std::hash::Hash for super::style_cached_data::PseudoElementStyleCacheKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(
            super::style_cached_data::PseudoElementStyleCacheKeyHashTraits::GetHash(self),
        );
    }
}

#[cfg(test)]
mod difference_tests {
    use super::super::computed_style_base::FieldDifference as F;
    use super::*;

    fn initial() -> &'static ComputedStyle {
        unsafe { &*ComputedStyle::GetInitialStyleSingleton() }
    }
    fn changed(change: impl FnOnce(&mut ComputedStyleBuilder)) -> &'static ComputedStyle {
        let mut builder = ComputedStyleBuilder::from_style(initial());
        change(&mut builder);
        unsafe { &*builder.TakeStyle() }
    }

    #[test]
    fn difference_null_identity_and_equal_distinct_styles() {
        let _heap = foundation::LayoutHeapScope::new();
        use ComputedStyleDifference::*;
        assert_eq!(
            ComputedStyle::ComputeDifference(std::ptr::null(), std::ptr::null()),
            kEqual
        );
        assert_eq!(
            ComputedStyle::ComputeDifference(initial(), std::ptr::null()),
            kInherited
        );
        let same = changed(|_| {});
        assert_eq!(ComputedStyle::ComputeDifference(initial(), same), kEqual);
        assert_eq!(ComputedStyleBase::FieldInvalidationDiff(initial(), same), 0);
        // Independent default data groups exercise deep equality, including
        // distinct variable trie roots, instead of shared group identities.
        let independent = ComputedStyle::default();
        assert!(initial() == &independent);
        assert_eq!(
            ComputedStyle::ComputeDifference(initial(), &independent),
            kEqual
        );
        assert_eq!(
            ComputedStyleBase::FieldInvalidationDiff(initial(), &independent),
            0
        );
    }

    #[test]
    fn difference_propagation_and_precise_field_bits() {
        let _heap = foundation::LayoutHeapScope::new();
        use ComputedStyleDifference::*;
        let width = changed(|b| b.SetWidth(&Length::Fixed(100.0)));
        assert_eq!(
            ComputedStyle::ComputeDifference(initial(), width),
            kNonInherited
        );
        assert_eq!(
            ComputedStyleBase::FieldInvalidationDiff(initial(), width),
            (F::kLayout | F::kScrollAnchor).bits()
        );
        let color = changed(|b| b.SetColor(&StyleColor::from_color(Color::kWhite)));
        assert_eq!(
            ComputedStyle::ComputeDifference(initial(), color),
            kIndependentInherited
        );
        assert_eq!(
            ComputedStyleBase::FieldInvalidationDiff(initial(), color),
            (F::kAccentColor | F::kBorderVisual | F::kColor | F::kCurrentcolor | F::kOutline)
                .bits()
        );
        let line_height = changed(|b| b.SetLineHeight(&Length::Fixed(23.0)));
        assert_eq!(
            ComputedStyle::ComputeDifference(initial(), line_height),
            kInherited
        );
        let flex = changed(|b| b.SetDisplay(EDisplay::kFlex));
        assert_eq!(
            ComputedStyle::ComputeDifference(initial(), flex),
            kDescendantAffecting
        );
        let background = changed(|b| b.SetBackgroundColor(&StyleColor::from_color(Color::kWhite)));
        assert_eq!(
            ComputedStyleBase::FieldInvalidationDiff(initial(), background),
            (F::kAXStyle | F::kBackgroundColor).bits()
        );
        let opacity = changed(|b| b.SetOpacity(0.5));
        assert_eq!(
            ComputedStyleBase::FieldInvalidationDiff(initial(), opacity),
            F::kOpacity.bits()
        );
    }
    #[test]
    fn first_line_cache_and_public_pseudo_differences() {
        let _heap = foundation::LayoutHeapScope::new();
        let color = changed(|b| b.SetColor(&StyleColor::from_color(Color::kWhite)));
        let old = changed(|_| {});
        let mut cache = PseudoElementStyleCache::default();
        cache.insert(
            super::super::style_cached_data::PseudoElementStyleCacheKey {
                pseudo_type: PseudoId::kPseudoIdFirstLineInherited,
                pseudo_argument: g_null_atom.clone(),
            },
            Member::from_ptr(color as *const ComputedStyle as *mut ComputedStyle),
        );
        old.EnsureCachedData().pseudo_element_styles_ =
            Member::from_ptr(MakeGarbageCollected(cache));
        assert!(old == initial());
        assert_eq!(
            ComputedStyle::ComputeDifference(old, initial()),
            ComputedStyleDifference::kIndependentInherited
        );
        let pseudo = changed(|b| {
            b.SetPseudoElementStyles(PseudoIdFlags::from_list(&[PseudoId::kPseudoIdBefore]).Bits())
        });
        assert_eq!(
            ComputedStyle::ComputeDifference(initial(), pseudo),
            ComputedStyleDifference::kPseudoElementStyle
        );
    }

    #[test]
    fn native_timing_data_is_retained_by_computed_style_gc_edges() {
        let heap = foundation::LayoutHeapScope::new();
        let style = changed(|builder| {
            *builder.AccessAnimations().DurationListMut() = vec![Some(0.25), None];
            *builder.AccessTransitions().PropertyListMut() = vec![
                super::super::css_timing_data::TransitionProperty::Unknown(
                    AtomicString::from_str("--progress"),
                ),
            ];
        });
        let root = foundation::Persistent::from_ptr(style as *const _ as *mut ComputedStyle);
        let animations = foundation::WeakPersistent::from_ptr(style.Animations().Get() as *mut CSSAnimationData);
        let transitions = foundation::WeakPersistent::from_ptr(style.Transitions().Get() as *mut CSSTransitionData);
        drop(heap);
        assert!(!animations.Get().is_null());
        assert!(!transitions.Get().is_null());
        assert_eq!(unsafe { &*animations.Get() }.DurationList(), &[Some(0.25), None]);
        assert!(matches!(
            &unsafe { &*transitions.Get() }.PropertyList()[0],
            super::super::css_timing_data::TransitionProperty::Unknown(name)
                if name.Utf8() == "--progress"
        ));
        drop(root);
        foundation::CollectLayoutHeapForTesting();
        assert!(animations.Get().is_null());
        assert!(transitions.Get().is_null());
    }

    #[test]
    fn pseudo_cache_owns_styles_and_keys_by_argument() {
        let _heap = foundation::LayoutHeapScope::new();
        let owner =
            foundation::Persistent::from_ptr(changed(|_| {}) as *const _ as *mut ComputedStyle);
        let first = changed(|b| b.SetStyleType(PseudoId::kPseudoIdBefore));
        let second = changed(|b| {
            b.SetStyleType(PseudoId::kPseudoIdBefore);
            b.SetColor(&StyleColor::from_color(Color::kWhite));
        });
        let weak_first =
            foundation::WeakPersistent::from_ptr(first as *const _ as *mut ComputedStyle);
        let weak_second =
            foundation::WeakPersistent::from_ptr(second as *const _ as *mut ComputedStyle);
        let argument = AtomicString::from_str("named");
        let style = unsafe { &*owner.Get() };
        assert_eq!(
            style.AddCachedPseudoElementStyle(first, PseudoId::kPseudoIdBefore, &g_null_atom),
            first as *const ComputedStyle
        );
        style.AddCachedPseudoElementStyle(second, PseudoId::kPseudoIdBefore, &argument);
        drop(_heap);
        assert_eq!(weak_first.Get(), first as *const _ as *mut ComputedStyle);
        assert_eq!(weak_second.Get(), second as *const _ as *mut ComputedStyle);
        assert_eq!(
            style.GetCachedPseudoElementStyleWithoutArgument(PseudoId::kPseudoIdBefore),
            first as *const ComputedStyle
        );
        assert_eq!(
            style.GetCachedPseudoElementStyle(PseudoId::kPseudoIdBefore, &argument),
            second as *const ComputedStyle
        );
        style.ClearCachedPseudoElementStyles();
        assert!(!style.HasCachedPseudoElementStyles());
        foundation::CollectLayoutHeapForTesting();
        assert!(weak_first.Get().is_null());
        assert!(weak_second.Get().is_null());
    }

    #[test]
    fn pseudo_cache_replaces_only_ensured_display_none_style() {
        let _heap = foundation::LayoutHeapScope::new();
        let owner = changed(|_| {});
        let ensured = changed(|b| {
            b.SetStyleType(PseudoId::kPseudoIdBefore);
            b.SetIsEnsuredInDisplayNone();
        });
        let actual = changed(|b| b.SetStyleType(PseudoId::kPseudoIdBefore));
        owner.AddCachedPseudoElementStyle(ensured, PseudoId::kPseudoIdBefore, &g_null_atom);
        assert_eq!(
            owner.ReplaceCachedPseudoElementStyle(actual, PseudoId::kPseudoIdBefore, &g_null_atom),
            actual as *const ComputedStyle
        );
        assert_eq!(
            owner.GetCachedPseudoElementStyleWithoutArgument(PseudoId::kPseudoIdBefore),
            actual as *const ComputedStyle
        );
        let after = changed(|b| b.SetStyleType(PseudoId::kPseudoIdAfter));
        assert_eq!(
            owner.ReplaceCachedPseudoElementStyle(after, PseudoId::kPseudoIdAfter, &g_null_atom),
            after as *const ComputedStyle
        );
    }
}

impl foundation::Traceable for super::style_cached_data::PseudoElementStyleCacheKey {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.pseudo_argument);
    }
}
impl foundation::heap_hash_containers::StrongHeapMapKey
    for super::style_cached_data::PseudoElementStyleCacheKey
{
}
