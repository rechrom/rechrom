/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2003, 2004, 2005, 2006, 2007, 2008, 2009, 2010, 2011 Apple Inc.
 * All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 *
 */

// cpp: third_party/blink/renderer/core/css/resolver/style_resolver.h:187-194,397-408
// Complete StyleResolver production control flow, preserving source ownership.
// External operations are mandatory, typed backend requirements.
//
// Source ledger: physical counts every source line; effective removes blank and
// comment-only lines and retains declarations, delimiters and preprocessor
// lines. mapped + omitted + pending = effective. Mapped counts source lines,
// never Rust adapter declarations or test scaffolding.
//                          physical effective mapped omitted pending
// New lifecycle batch h:        46        45     43       2       0
// New lifecycle batch cc:      147       130    120      10       0
// Rule-matching batch h:        16        16     16       0       0
// Rule-matching batch cc:      668       485    479       6       0
// Base-style batch h:           69        62     59       3       0
// Base-style batch cc:        1583      1122    908     214       0
// Completion batch h:           99        93     91       2       0
// Completion batch cc:        1092       879    826      53       0
// Complete source h:           445       278    229      49       0
// Complete source cc:         3806      2735   2333     402       0
// Complete source total:      4251      3013   2562     451       0
// The complete h mapping includes the 20 previously mapped enum lines.
// Lifecycle omitted: h:71,304; cc:649,3326-3334 (destructor and GC Trace).
// Rule batch ranges: h:145,338-345,410-413,425-427; cc:401-407,571-591,
// 661-1032,1036-1058,1062-1074,1101-1332. Rule batch omitted: cc:734-736
// (forward declaration),1204,1214 (preprocessor),1278 (CFI annotation).
// The rule-matching batch added 495 mapped effective lines. Debug assertions and the UA
// group comparison at cc:1205-1213 are mapped, not omitted.
// Base-style h ranges: 74-76,83-87,226,310-326,346,348-366,368-389,395.
// Base-style cc ranges: 149-152,154-174,176-208,210-224,226-245,247-252,
// 254-259,261-266,268-282,284-289,429-507,509-567,593-602,607-615,618-629,
// 1334-1341,1343-1349,1378-1505,1507-1521,1523-1530,1532-1564,1566-1595,
// 1597-1704,1736-1944,1958-2095,2572-2675,2677-2692,2771-2896,2898-2912,
// 2914-3001,3181-3226,3228-3238,3240-3251,3253-3277,3348-3367.
// Base-style omitted helper definitions: cc:291-388 (debug diff),395-399
// (debug reset),409-427 (use count),1351-1363 (statistics). These definitions
// are included in the batch physical/effective totals above.
// Base-style omitted h:349,351-352 (stack/access annotations and unused is_hit).
// Base-style omitted cc within mapped functions: 456-460,474-485,488,493,
// 495-500,1383,1387-1388,1412,1417-1418,1431-1442,1453-1469,1483,1569,
// 1804-1813,1964,1969-1983,1996-1998,2059-2088,2599-2604,2652,2817-2818,
// 2905-2906,3183-3186,3201,3204-3205,3208-3217,3225 (debug/tracing/metrics).
// This batch adds 967 mapped effective lines. Eight cc preprocessor lines
// within this batch were already globally omitted; new omissions total 209.
// Completion h ranges: 108-114,126-129,135-138,142-144,149-151,155,
// 158-163,167-169,175-183,195-201,204-207,209-213,231-239,242-244,
// 252-255,261-267,274-280,284,302,327-337.
// Completion cc ranges: 631-643,2097-2124,2145-2350,2429-2570,2694-2751,
// 3003-3179,3282-3313,3369-3804. All production bodies in these ranges are mapped.
// Completion omitted h:178 (stack annotation),302 (ApplyTriggerData has only
// a declaration; there is no definition in the complete source translation unit).
// Completion omitted cc:2146,2148,2177 (stack/access labels),2191,2430,
// 2447,2481,3430-3433,3439-3447,3452,3562-3569,3589-3611,3713,3760-3761
// (debug checks and metrics-only branches). Macro bodies at 3396,3399-3402
// are mapped through the typed comparison/assignment macros in Rust.
// Completion adds 917 mapped effective lines and 49 new omissions. Six
// preprocessor lines in its scope were already globally omitted, hence its
// batch totals are physical1191/effective972/mapped917/omitted55/pending0.
// The complete source has zero pending effective lines. No internal/pending
// backend hook or default/fake external implementation is supplied.
// Other global omissions: preprocessor/include/guard/namespace boilerplate,
// forward declarations, access labels, GC Trace/destructor and test friendship.

use std::cell::{OnceCell, RefCell};
use std::rc::Rc;

use crate::css_property_names::CSSPropertyID;
use crate::css_value_keywords::CSSValueID;
use crate::parser::css_parser_mode::CSSParserMode;
use crate::properties::css_property::CSSProperty;
use crate::resolver::cascade_filter::CascadeFilter;
use crate::resolver::cascade_origin::CascadeOrigin;
use crate::resolver::match_flags::MatchFlag;

use foundation::{
    AtomicString, EDisplay, EInsideLink, EInteractivity, EOrder, EOverflow, EPosition, EUserModify,
    HashFloat, HashInt, PhysicalOffset, TextDirection,
};
use layoutng_style::style::computed_style_constants::PseudoId;
use layoutng_style::style::computed_style_initial_values::ComputedStyleInitialValues;
use layoutng_style::style::gap_data_list::GapDataList;

use crate::{ColorSchemeFlags, PreferredColorScheme};

// cpp: style_resolver.h:187-194
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum CSSRuleFilter {
    kUACSSRules = 1 << 1,
    kUserCSSRules = 1 << 2,
    kAuthorCSSRules = 1 << 3,
    kUAAndUserCSSRules = Self::kUACSSRules as u32 | Self::kUserCSSRules as u32,
    kAllButUACSSRules = Self::kUserCSSRules as u32 | Self::kAuthorCSSRules as u32,
    kAllCSSRules = Self::kUAAndUserCSSRules as u32 | Self::kAuthorCSSRules as u32,
}

// cpp: style_resolver.h:397-408
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum UASheetCacheKeyIndex {
    kHTMLUASheet,
    kSVGUASheet,
    kMathMLUASheet,
    kFullscreenUASheet,
    kQuirksUASheet,
    kViewSourceUASheet,
    kForcedColorsUASheet,
    kJSONUASheet,
    kViewTransitionUASheet,
    kPseudoElementUASheet,
}

/// Dependencies belonging to Document, style storage, fonts and media. Every
/// operation is required: production callers must supply their real objects and
/// behavior. No fallback Document, style, cache or font is constructed here.
///
/// ComputedStyle must be an owning value/handle. A builder's TakeStyle
/// must retain the produced style independently of the consumed builder. The
/// initial singleton, inheriting builder and locale/font operations preserve
/// their Chromium contracts; this interface does not implement those classes.
pub trait StyleResolverBackend {
    type Document;
    type ComputedStyle;
    type ComputedStyleBuilder;
    type MatchedPropertiesCache;
    type SelectorFilter;
    type StyleRuleUsageTracker;
    type StyleEngine;
    type LocalFrame;
    type LocalFrameView;
    type Settings;
    type FontDescription;
    type LayoutLocale;
    type StyleInitialData;
    type TapHighlightColor;
    type RuleSet;
    type RuleSetGroup;

    fn NewRuleSetGroup(&self, rule_set_group_index: u32) -> Self::RuleSetGroup;

    fn NewMatchedPropertiesCache(&self) -> Self::MatchedPropertiesCache;
    fn NewSelectorFilter(&self) -> Self::SelectorFilter;
    fn ClearMatchedPropertiesCache(&self, cache: &mut Self::MatchedPropertiesCache);
    fn ClearViewportDependent(&self, cache: &mut Self::MatchedPropertiesCache);

    fn GetStyleEngine<'d>(&self, document: &'d Self::Document) -> &'d Self::StyleEngine;
    fn GetFrame<'d>(&self, document: &'d Self::Document) -> Option<&'d Self::LocalFrame>;
    fn View<'d>(&self, document: &'d Self::Document) -> Option<&'d Self::LocalFrameView>;
    fn GetSettings<'d>(&self, document: &'d Self::Document) -> Option<&'d Self::Settings>;
    fn Printing(&self, document: &Self::Document) -> bool;
    fn VisuallyOrdered(&self, document: &Self::Document) -> bool;
    fn InForcedColorsMode(&self, document: &Self::Document) -> bool;
    fn InDesignMode(&self, document: &Self::Document) -> bool;
    fn ContentLanguage<'d>(&self, document: &'d Self::Document) -> &'d AtomicString;
    fn LayoutZoomFactor(&self, frame: &Self::LocalFrame) -> f32;
    fn MediaType<'v>(&self, view: &'v Self::LocalFrameView) -> &'v str;
    fn UpdateActiveStyle(&self, engine: &Self::StyleEngine);
    fn GetPageColorSchemes(&self, engine: &Self::StyleEngine) -> ColorSchemeFlags;
    fn GetPreferredColorScheme(&self, engine: &Self::StyleEngine) -> PreferredColorScheme;
    fn GetForceDarkModeEnabled(&self, engine: &Self::StyleEngine) -> bool;
    fn MaybeCreateAndGetInitialData<'e>(
        &self,
        engine: &'e Self::StyleEngine,
    ) -> Option<&'e Self::StyleInitialData>;
    fn PreferDefaultScrollbarStylesEnabled(&self) -> bool;
    fn GetPrefersDefaultScrollbarStyles(&self, settings: &Self::Settings) -> bool;

    fn GetInitialStyleSingleton(&self) -> &Self::ComputedStyle;
    fn NewComputedStyleBuilder(&self, initial: &Self::ComputedStyle) -> Self::ComputedStyleBuilder;
    fn NewComputedStyleBuilderInheritingFrom(
        &self,
        initial: &Self::ComputedStyle,
        parent: &Self::ComputedStyle,
    ) -> Self::ComputedStyleBuilder;
    fn TakeStyle(&self, builder: Self::ComputedStyleBuilder) -> Self::ComputedStyle;
    fn ClampLineWidth(&self, value: f32) -> i32;
    fn SetBorderTopWidth(&self, builder: &mut Self::ComputedStyleBuilder, width: i32);
    fn SetBorderRightWidth(&self, builder: &mut Self::ComputedStyleBuilder, width: i32);
    fn SetBorderBottomWidth(&self, builder: &mut Self::ComputedStyleBuilder, width: i32);
    fn SetBorderLeftWidth(&self, builder: &mut Self::ComputedStyleBuilder, width: i32);
    fn SetOutlineWidth(&self, builder: &mut Self::ComputedStyleBuilder, width: i32);
    fn SetColumnRuleWidthInternal(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        width: GapDataList<i32>,
    );
    fn SetRowRuleWidthInternal(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        width: GapDataList<i32>,
    );
    fn SetRtlOrdering(&self, builder: &mut Self::ComputedStyleBuilder, ordering: EOrder);
    fn SetZoom(&self, builder: &mut Self::ComputedStyleBuilder, zoom: f32);
    fn SetEffectiveZoom(&self, builder: &mut Self::ComputedStyleBuilder, zoom: f32);
    fn SetInForcedColorsMode(&self, builder: &mut Self::ComputedStyleBuilder, enabled: bool);
    fn InitialTapHighlightColor(&self) -> Self::TapHighlightColor;
    fn SetTapHighlightColor(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        color: Self::TapHighlightColor,
    );
    fn SetUsedColorScheme(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        page_schemes: ColorSchemeFlags,
        preferred: PreferredColorScheme,
        force_dark: bool,
    );
    fn GetFontDescription(&self, builder: &Self::ComputedStyleBuilder) -> Self::FontDescription;
    fn GetLayoutLocale(&self, language: &AtomicString) -> &Self::LayoutLocale;
    fn SetLocale(&self, description: &mut Self::FontDescription, locale: &Self::LayoutLocale);
    fn SetFontDescription(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        description: Self::FontDescription,
    );
    fn SetUserModify(&self, builder: &mut Self::ComputedStyleBuilder, modify: EUserModify);
    fn CreateInitialFont(
        &self,
        document: &Self::Document,
        builder: &mut Self::ComputedStyleBuilder,
    );
    fn SetInitialData(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        initial_data: &Self::StyleInitialData,
    );
    fn SetPrefersDefaultScrollbarStyles(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: bool,
    );
}

// cpp: style_resolver.h:66-72,415-437
// No Clone/Copy implementation: Chromium deletes copying and assignment.
// Trace (cc:3326-3334) is GC boilerplate; backend values must own their
// dependencies and retain any GC roots required by their concrete storage.
// The media-controls cache below is independent of the shared UA group cache.
pub struct StyleResolver<'a, B: StyleResolverBackend> {
    backend: &'a B,
    // cpp: style_resolver.h:415
    matched_properties_cache_: B::MatchedPropertiesCache,
    // cpp: style_resolver.h:421
    initial_style_: OnceCell<B::ComputedStyle>,
    // cpp: style_resolver.h:422
    selector_filter_: B::SelectorFilter,
    // cpp: style_resolver.h:425-427
    media_controls_cache_key_: Option<Rc<B::RuleSet>>,
    media_controls_cached_rule_set_group_: B::RuleSetGroup,
    // cpp: style_resolver.h:429-430
    document_: &'a B::Document,
    tracker_: Option<&'a B::StyleRuleUsageTracker>,
    // cpp: style_resolver.h:433-434
    count_computed_style_bytes_: bool,
    computed_style_bytes_used_: usize,
    // cpp: style_resolver.h:436-437
    print_media_type_: bool,
    was_viewport_resized_: bool,
}

impl<'a, B: StyleResolverBackend> StyleResolver<'a, B> {
    // cpp: style_resolver.cc:645-647; h:68-70
    pub fn new(document: &'a B::Document, backend: &'a B) -> Self {
        let mut resolver = Self {
            backend,
            matched_properties_cache_: backend.NewMatchedPropertiesCache(),
            initial_style_: OnceCell::new(),
            selector_filter_: backend.NewSelectorFilter(),
            media_controls_cache_key_: None,
            media_controls_cached_rule_set_group_: backend.NewRuleSetGroup(0),
            document_: document,
            tracker_: None,
            count_computed_style_bytes_: false,
            computed_style_bytes_used_: 0,
            print_media_type_: false,
            was_viewport_resized_: false,
        };
        resolver.UpdateMediaType();
        resolver
    }

    // cpp: style_resolver.cc:649; h:71
    // The default destructor requires no custom Drop implementation.

    // cpp: style_resolver.cc:651-653; h:72
    pub fn Dispose(&mut self) {
        self.backend
            .ClearMatchedPropertiesCache(&mut self.matched_properties_cache_);
    }

    // cpp: style_resolver.cc:655-657; h:223
    pub fn SetRuleUsageTracker(&mut self, tracker: Option<&'a B::StyleRuleUsageTracker>) {
        self.tracker_ = tracker;
    }

    // cpp: style_resolver.cc:2352-2357; h:90
    pub fn InitialStyle(&self) -> &B::ComputedStyle {
        self.initial_style_
            .get_or_init(|| self.CreateInitialStyle())
    }

    // cpp: style_resolver.cc:2359-2361; h:91
    pub fn InvalidateInitialStyle(&mut self) {
        self.initial_style_.take();
    }

    // cpp: style_resolver.cc:2363-2365; h:94
    pub fn CreateComputedStyleBuilder(&self) -> B::ComputedStyleBuilder {
        self.backend.NewComputedStyleBuilder(self.InitialStyle())
    }

    // cpp: style_resolver.cc:2367-2370; h:97-98
    pub fn CreateComputedStyleBuilderInheritingFrom(
        &self,
        parent_style: &B::ComputedStyle,
    ) -> B::ComputedStyleBuilder {
        self.backend
            .NewComputedStyleBuilderInheritingFrom(self.InitialStyle(), parent_style)
    }

    // cpp: style_resolver.cc:2372-2378; h:106
    pub fn InitialZoom(&self) -> f32 {
        let document = self.GetDocument();
        if let Some(frame) = self.backend.GetFrame(document) {
            return if !self.backend.Printing(document) {
                self.backend.LayoutZoomFactor(frame)
            } else {
                1.0
            };
        }
        1.0
    }

    // cpp: style_resolver.cc:2380-2384; h:418
    fn CreateInitialStyle(&self) -> B::ComputedStyle {
        let mut builder = self
            .backend
            .NewComputedStyleBuilder(self.backend.GetInitialStyleSingleton());
        self.SetZoomedInitialLineWidths(self.InitialZoom(), &mut builder);
        self.backend.TakeStyle(builder)
    }

    // cpp: style_resolver.cc:1078-1099; h:417
    fn SetZoomedInitialLineWidths(&self, zoom: f32, builder: &mut B::ComputedStyleBuilder) {
        self.backend.SetBorderTopWidth(
            builder,
            self.backend
                .ClampLineWidth(ComputedStyleInitialValues::InitialBorderTopWidth() as f32 * zoom),
        );
        self.backend.SetBorderRightWidth(
            builder,
            self.backend.ClampLineWidth(
                ComputedStyleInitialValues::InitialBorderRightWidth() as f32 * zoom,
            ),
        );
        self.backend.SetBorderBottomWidth(
            builder,
            self.backend.ClampLineWidth(
                ComputedStyleInitialValues::InitialBorderBottomWidth() as f32 * zoom,
            ),
        );
        self.backend.SetBorderLeftWidth(
            builder,
            self.backend
                .ClampLineWidth(ComputedStyleInitialValues::InitialBorderLeftWidth() as f32 * zoom),
        );
        self.backend.SetOutlineWidth(
            builder,
            self.backend
                .ClampLineWidth(ComputedStyleInitialValues::InitialOutlineWidth() as f32 * zoom),
        );
        self.backend.SetColumnRuleWidthInternal(
            builder,
            GapDataList::from_value(&self.backend.ClampLineWidth(
                ComputedStyleInitialValues::InitialColumnRuleWidth().GetLegacyValue() as f32 * zoom,
            )),
        );
        self.backend.SetRowRuleWidthInternal(
            builder,
            GapDataList::from_value(&self.backend.ClampLineWidth(
                ComputedStyleInitialValues::InitialRowRuleWidth().GetLegacyValue() as f32 * zoom,
            )),
        );
    }

    // cpp: style_resolver.cc:2386-2423; h:103
    pub fn InitialStyleBuilderForElement(&self) -> B::ComputedStyleBuilder {
        let engine = self.backend.GetStyleEngine(self.GetDocument());
        let mut builder = self.CreateComputedStyleBuilder();
        self.backend.SetRtlOrdering(
            &mut builder,
            if self.backend.VisuallyOrdered(self.GetDocument()) {
                EOrder::kVisual
            } else {
                EOrder::kLogical
            },
        );
        self.backend.SetZoom(&mut builder, self.InitialZoom());
        self.backend
            .SetEffectiveZoom(&mut builder, self.InitialZoom());
        self.backend.SetInForcedColorsMode(
            &mut builder,
            self.backend.InForcedColorsMode(self.GetDocument()),
        );
        self.backend
            .SetTapHighlightColor(&mut builder, self.backend.InitialTapHighlightColor());
        self.backend.SetUsedColorScheme(
            &mut builder,
            self.backend.GetPageColorSchemes(engine),
            self.backend.GetPreferredColorScheme(engine),
            self.backend.GetForceDarkModeEnabled(engine),
        );
        let mut document_font_description = self.backend.GetFontDescription(&builder);
        self.backend.SetLocale(
            &mut document_font_description,
            self.backend
                .GetLayoutLocale(self.backend.ContentLanguage(self.GetDocument())),
        );
        self.backend
            .SetFontDescription(&mut builder, document_font_description);
        self.backend.SetUserModify(
            &mut builder,
            if self.backend.InDesignMode(self.GetDocument()) {
                EUserModify::kReadWrite
            } else {
                EUserModify::kReadOnly
            },
        );
        self.backend
            .CreateInitialFont(self.GetDocument(), &mut builder);
        if let Some(initial_data) = self.backend.MaybeCreateAndGetInitialData(engine) {
            self.backend.SetInitialData(&mut builder, initial_data);
        }
        if self.backend.PreferDefaultScrollbarStylesEnabled() {
            if let Some(settings) = self.backend.GetSettings(self.GetDocument()) {
                if self.backend.GetPrefersDefaultScrollbarStyles(settings) {
                    self.backend
                        .SetPrefersDefaultScrollbarStyles(&mut builder, true);
                }
            }
        }
        builder
    }

    // cpp: style_resolver.cc:2425-2427; h:104
    pub fn InitialStyleForElement(&self) -> B::ComputedStyle {
        self.backend.TakeStyle(self.InitialStyleBuilderForElement())
    }

    // cpp: style_resolver.cc:2753-2755; h:216
    pub fn InvalidateMatchedPropertiesCache(&mut self) {
        self.backend
            .ClearMatchedPropertiesCache(&mut self.matched_properties_cache_);
    }

    // cpp: style_resolver.cc:2757-2759; h:217
    pub fn InvalidateMatchedPropertiesCacheForViewportUnits(&mut self) {
        self.backend
            .ClearViewportDependent(&mut self.matched_properties_cache_);
    }

    // cpp: style_resolver.cc:2761-2765; h:219
    pub fn SetResizedForViewportUnits(&mut self) {
        self.was_viewport_resized_ = true;
        self.backend
            .UpdateActiveStyle(self.backend.GetStyleEngine(self.GetDocument()));
        self.backend
            .ClearViewportDependent(&mut self.matched_properties_cache_);
    }

    // cpp: style_resolver.cc:2767-2769; h:220
    pub fn ClearResizedForViewportUnits(&mut self) {
        self.was_viewport_resized_ = false;
    }

    // cpp: style_resolver.h:221
    pub fn WasViewportResized(&self) -> bool {
        self.was_viewport_resized_
    }

    // cpp: style_resolver.cc:3315-3324; h:224
    pub fn UpdateMediaType(&mut self) {
        if let Some(view) = self.backend.View(self.GetDocument()) {
            let was_print = self.print_media_type_;
            self.print_media_type_ = self.backend.MediaType(view).eq_ignore_ascii_case("print");
            if was_print != self.print_media_type_ {
                self.backend
                    .ClearViewportDependent(&mut self.matched_properties_cache_);
            }
        }
    }

    // cpp: style_resolver.cc:3336-3338; h:393
    fn IsForcedColorsModeEnabled(&self) -> bool {
        self.backend.InForcedColorsMode(self.GetDocument())
    }

    // cpp: style_resolver.h:173
    pub fn GetSelectorFilter(&mut self) -> &mut B::SelectorFilter {
        &mut self.selector_filter_
    }

    // cpp: style_resolver.h:391
    fn GetDocument(&self) -> &B::Document {
        self.document_
    }

    // cpp: style_resolver.h:293-296
    pub fn SetCountComputedStyleBytes(&mut self, enabled: bool) {
        self.count_computed_style_bytes_ = enabled;
        self.computed_style_bytes_used_ = 0;
    }

    // cpp: style_resolver.h:297-300
    pub fn GetComputedStyleBytesUsed(&self) -> usize {
        debug_assert!(self.count_computed_style_bytes_);
        self.computed_style_bytes_used_
    }
}

/// Shared function-local declaration storage from cc:571-591. The backend
/// supplies one persistent instance per CSS runtime, preserving identity across
/// documents; allocation and empty/property decisions remain in StyleResolver.
pub struct StyleResolverDeclarationCache<T> {
    left_to_right: OnceCell<Rc<T>>,
    right_to_left: OnceCell<Rc<T>>,
    document_element: OnceCell<Rc<T>>,
    forced_colors: OnceCell<Rc<T>>,
    universal_overlay: OnceCell<Rc<T>>,
}

impl<T> StyleResolverDeclarationCache<T> {
    pub const fn new() -> Self {
        Self {
            left_to_right: OnceCell::new(),
            right_to_left: OnceCell::new(),
            document_element: OnceCell::new(),
            forced_colors: OnceCell::new(),
            universal_overlay: OnceCell::new(),
        }
    }
}

/// Required operations on DOM, ElementRuleCollector, ScopedStyleResolver,
/// CSSDefaultStyleSheets and RuleSetGroup. No StyleResolver helper is forwarded
/// through this interface. Shared caches expose storage, never hit/miss logic.
pub trait StyleResolverRuleMatchingBackend: StyleResolverBackend {
    type Element;
    type TreeScope;
    type ShadowRoot;
    type HTMLSlotElement;
    type ScopedStyleResolver;
    type ElementRuleCollector;
    type StyleResolverState;
    type CSSStyleSheet;
    type TextTrack;
    type DOMTokenList;
    type TokenSet;
    type PartNames;
    type PartNamesMap;
    type RootNode;
    type CSSPropertyValueSet;
    /// An owning guard that restores the collector's previous rule tree scope
    /// on Drop, just like ElementRuleCollector::ScopedRuleTreeScope.
    type ScopedRuleTreeScope;

    fn PseudoElementUltimateOriginatingElement<'e>(
        &self,
        element: &'e Self::Element,
    ) -> &'e Self::Element;
    fn ElementTreeScope<'e>(&self, element: &'e Self::Element) -> &'e Self::TreeScope;
    fn ParentTreeScope<'t>(&self, scope: &'t Self::TreeScope) -> Option<&'t Self::TreeScope>;
    fn TreeScopeResolver<'t>(
        &self,
        scope: &'t Self::TreeScope,
    ) -> Option<&'t Self::ScopedStyleResolver>;
    fn ResolverTreeScope<'r>(&self, resolver: &'r Self::ScopedStyleResolver)
        -> &'r Self::TreeScope;
    fn TreeScopeRoot<'t>(&self, scope: &'t Self::TreeScope) -> &'t Self::RootNode;
    fn IsHTMLElement(&self, element: &Self::Element) -> bool;
    fn IsSVGElement(&self, element: &Self::Element) -> bool;
    fn IsVTTElement(&self, element: &Self::Element) -> bool;
    fn IsVTTCueBackgroundBox(&self, element: &Self::Element) -> bool;
    fn IsPseudoElement(&self, element: &Self::Element) -> bool;
    fn IsStyledElement(&self, element: &Self::Element) -> bool;
    fn IsInShadowTree(&self, element: &Self::Element) -> bool;
    fn IsMediaElement(&self, element: &Self::Element) -> bool;
    fn NamespaceURI<'e>(&self, element: &'e Self::Element) -> &'e AtomicString;
    fn MathMLNamespaceURI(&self) -> &AtomicString;
    fn CorrespondingElement<'e>(&self, svg_element: &'e Self::Element)
        -> Option<&'e Self::Element>;
    fn ShadowPseudoId<'e>(&self, element: &'e Self::Element) -> &'e AtomicString;
    fn ElementPseudoIdForStyling(&self, element: &Self::Element) -> PseudoId;
    fn PseudoIdForShadowElementName(&self, name: &AtomicString) -> PseudoId;
    fn IsTransitionPseudoElement(&self, pseudo_id: PseudoId) -> bool;
    fn GetShadowRoot<'e>(&self, element: &'e Self::Element) -> Option<&'e Self::ShadowRoot>;
    fn ContainingShadowRoot<'e>(&self, element: &'e Self::Element) -> Option<&'e Self::ShadowRoot>;
    fn ShadowRootResolver<'s>(
        &self,
        root: &'s Self::ShadowRoot,
    ) -> Option<&'s Self::ScopedStyleResolver>;
    fn ShadowRootIsUserAgent(&self, root: &Self::ShadowRoot) -> bool;
    fn ShadowRootHost<'s>(&self, root: &'s Self::ShadowRoot) -> &'s Self::Element;
    fn OwnerShadowHost<'e>(&self, element: &'e Self::Element) -> Option<&'e Self::Element>;
    fn AssignedSlot<'e>(&self, element: &'e Self::Element) -> Option<&'e Self::HTMLSlotElement>;
    fn SlotAssignedSlot<'s>(
        &self,
        slot: &'s Self::HTMLSlotElement,
    ) -> Option<&'s Self::HTMLSlotElement>;
    fn SlotTreeScope<'s>(&self, slot: &'s Self::HTMLSlotElement) -> &'s Self::TreeScope;
    fn VTTElementTrack<'e>(&self, element: &'e Self::Element) -> Option<&'e Self::TextTrack>;
    fn VTTCueBackgroundBoxTrack<'e>(
        &self,
        element: &'e Self::Element,
    ) -> Option<&'e Self::TextTrack>;
    fn TextTrackCSSStyleSheets<'t>(
        &self,
        track: &'t Self::TextTrack,
    ) -> &'t [Rc<Self::CSSStyleSheet>];
    fn ElementDocument<'e>(&self, element: &'e Self::Element) -> &'e Self::Document;
    fn EnsureVTTOriginatingElement<'s>(&self, engine: &'s Self::StyleEngine) -> &'s Self::Element;
    /// StyleEngine::RuleSetForSheet with the source's empty mixin map.
    fn RuleSetForSheet(
        &self,
        engine: &Self::StyleEngine,
        sheet: &Self::CSSStyleSheet,
    ) -> Option<Rc<Self::RuleSet>>;
    fn GetPart<'e>(&self, element: &'e Self::Element) -> Option<&'e Self::DOMTokenList>;
    fn PartLength(&self, part: &Self::DOMTokenList) -> usize;
    fn PartTokenSet<'p>(&self, part: &'p Self::DOMTokenList) -> &'p Self::TokenSet;
    fn NewPartNames(&self, tokens: &Self::TokenSet) -> Self::PartNames;
    fn HasPartNamesMap(&self, element: &Self::Element) -> bool;
    fn PartNamesMap<'e>(&self, element: &'e Self::Element) -> &'e Self::PartNamesMap;
    fn PushPartNamesMap(&self, names: &mut Self::PartNames, map: &Self::PartNamesMap);
    fn InlineStyle(&self, element: &Self::Element) -> Option<Rc<Self::CSSPropertyValueSet>>;
    fn PresentationAttributeStyle(
        &self,
        element: &Self::Element,
    ) -> Option<Rc<Self::CSSPropertyValueSet>>;
    fn AdditionalPresentationAttributeStyle(
        &self,
        element: &Self::Element,
    ) -> Option<Rc<Self::CSSPropertyValueSet>>;
    fn HasDirectionAuto(&self, html_element: &Self::Element) -> bool;
    fn CachedDirectionality(&self, html_element: &Self::Element) -> TextDirection;
    fn AnimatedSMILStyleProperties(
        &self,
        svg_element: &Self::Element,
    ) -> Option<Rc<Self::CSSPropertyValueSet>>;
    fn StateGetElement<'s>(&self, state: &'s Self::StyleResolverState) -> &'s Self::Element;
    fn StateIsForPseudoElement(&self, state: &Self::StyleResolverState) -> bool;

    fn DeclarationCache(&self) -> &StyleResolverDeclarationCache<Self::CSSPropertyValueSet>;
    fn NewMutableCSSPropertyValueSet(&self, mode: CSSParserMode) -> Rc<Self::CSSPropertyValueSet>;
    fn PropertyValueSetIsEmpty(&self, set: &Self::CSSPropertyValueSet) -> bool;
    fn SetLonghandProperty(
        &self,
        set: &Self::CSSPropertyValueSet,
        property: CSSPropertyID,
        value: CSSValueID,
    );

    fn ClearMatchedRules(&self, collector: &mut Self::ElementRuleCollector);
    fn CollectorPseudoId(&self, collector: &Self::ElementRuleCollector) -> PseudoId;
    fn BeginAddingAuthorRulesForTreeScope(
        &self,
        collector: &mut Self::ElementRuleCollector,
        scope: &Self::TreeScope,
    );
    fn NewScopedRuleTreeScope(
        &self,
        collector: &mut Self::ElementRuleCollector,
        scope: &Self::TreeScope,
    ) -> Self::ScopedRuleTreeScope;
    fn CollectMatchingShadowHostRules(
        &self,
        resolver: &Self::ScopedStyleResolver,
        collector: &mut Self::ElementRuleCollector,
    );
    fn CollectMatchingSlottedRules(
        &self,
        resolver: &Self::ScopedStyleResolver,
        collector: &mut Self::ElementRuleCollector,
    );
    fn CollectMatchingPartPseudoRules(
        &self,
        resolver: &Self::ScopedStyleResolver,
        collector: &mut Self::ElementRuleCollector,
        part_names: &Self::PartNames,
    );
    fn CollectMatchingElementScopeRules(
        &self,
        resolver: &Self::ScopedStyleResolver,
        root: &Self::RootNode,
        collector: &mut Self::ElementRuleCollector,
        part_shadow_host: Option<&Self::PartNames>,
    );
    fn SortAndTransferMatchedRules(
        &self,
        collector: &mut Self::ElementRuleCollector,
        origin: CascadeOrigin,
        is_vtt_embedded_style: bool,
        tracker: Option<&Self::StyleRuleUsageTracker>,
    );
    fn AddElementStyleProperties(
        &self,
        collector: &mut Self::ElementRuleCollector,
        properties: Option<&Self::CSSPropertyValueSet>,
        origin: CascadeOrigin,
        is_cacheable: bool,
        is_inline_style: bool,
    );
    fn AddTryStyleProperties(&self, collector: &mut Self::ElementRuleCollector);
    fn AddTryTacticsStyleProperties(&self, collector: &mut Self::ElementRuleCollector);
    fn CollectMatchingUserRules(
        &self,
        engine: &Self::StyleEngine,
        collector: &mut Self::ElementRuleCollector,
    );
    fn SetMatchingUARules(&self, collector: &mut Self::ElementRuleCollector, value: bool);
    /// Constructs the external MatchRequest from these exact inputs, then calls
    /// ElementRuleCollector::CollectMatchingRules. No matching decisions live here.
    fn CollectMatchingRules(
        &self,
        collector: &mut Self::ElementRuleCollector,
        group: &Self::RuleSetGroup,
        scope: Option<&Self::TreeScope>,
        vtt_originating_element: Option<&Self::Element>,
        part_names: Option<&Self::PartNames>,
    );

    fn DefaultHtmlStyle(&self) -> Rc<Self::RuleSet>;
    fn DefaultSVGStyle(&self) -> Rc<Self::RuleSet>;
    fn DefaultMathMLStyle(&self) -> Rc<Self::RuleSet>;
    fn DefaultFullscreenStyle(&self) -> Rc<Self::RuleSet>;
    fn DefaultHtmlQuirksStyle(&self) -> Rc<Self::RuleSet>;
    fn DefaultViewSourceStyle(&self) -> Rc<Self::RuleSet>;
    fn DefaultForcedColorStyle(&self) -> Rc<Self::RuleSet>;
    fn DefaultJSONDocumentStyle(&self) -> Rc<Self::RuleSet>;
    fn DefaultPseudoElementStyleOrNull(&self) -> Option<Rc<Self::RuleSet>>;
    fn DefaultForcedColorsMediaControlsStyle(&self) -> Rc<Self::RuleSet>;
    fn DefaultMediaControlsStyle(&self) -> Rc<Self::RuleSet>;
    fn HasFullscreenElements(&self) -> bool;
    fn InQuirksMode(&self, document: &Self::Document) -> bool;
    fn IsViewSource(&self, document: &Self::Document) -> bool;
    fn IsJSONDocument(&self, document: &Self::Document) -> bool;
    fn ActiveViewTransitionStyle(
        &self,
        engine: &Self::StyleEngine,
        element: &Self::Element,
    ) -> Option<Rc<Self::RuleSet>>;
    /// This is CSSDefaultStyleSheets::RuleSetGroupCache, shared across resolvers.
    /// The sheet owner must clear it when rule-set pointers for a key change.
    fn RuleSetGroupCache(&self) -> &RefCell<Vec<(u32, Self::RuleSetGroup)>>;
    fn RuleSetGroupAddRuleSet(&self, group: &mut Self::RuleSetGroup, rules: &Rc<Self::RuleSet>);
    fn RuleSetGroupIsFull(&self, group: &Self::RuleSetGroup) -> bool;
    fn RuleSetGroupIsEmpty(&self, group: &Self::RuleSetGroup) -> bool;
    fn RuleSetGroupAssertEqualTo(&self, group: &Self::RuleSetGroup, reference: &Self::RuleSetGroup);

    fn SetZIndex(&self, builder: &mut Self::ComputedStyleBuilder, value: i32);
    fn SetForcesStackingContext(&self, builder: &mut Self::ComputedStyleBuilder, value: bool);
    fn SetDisplay(&self, builder: &mut Self::ComputedStyleBuilder, value: EDisplay);
    fn SetPosition(&self, builder: &mut Self::ComputedStyleBuilder, value: EPosition);
    fn SetOverflowX(&self, builder: &mut Self::ComputedStyleBuilder, value: EOverflow);
    fn SetOverflowY(&self, builder: &mut Self::ComputedStyleBuilder, value: EOverflow);
    fn ApplyVisionDeficiencyStyle(
        &self,
        engine: &Self::StyleEngine,
        builder: &mut Self::ComputedStyleBuilder,
    );
}

// cpp: style_resolver.cc:676-679
struct UAShadowPseudoResult {
    use_parent_resolver: bool,
    cascade_style_attribute_in_parent_scope: bool,
}

impl<'a, B: StyleResolverRuleMatchingBackend> StyleResolver<'a, B> {
    // cpp: style_resolver.cc:254-259
    fn UltimateOriginatingElementOrSelf<'e>(&self, element: &'e B::Element) -> &'e B::Element {
        if !self.backend.IsPseudoElement(element) {
            element
        } else {
            self.backend
                .PseudoElementUltimateOriginatingElement(element)
        }
    }

    // cpp: style_resolver.cc:154-174
    fn IsPseudoElementWithUAStyle(pseudo_id: PseudoId) -> bool {
        matches!(
            pseudo_id,
            PseudoId::kPseudoIdMarker
                | PseudoId::kPseudoIdScrollButtonBlockStart
                | PseudoId::kPseudoIdScrollButtonInlineStart
                | PseudoId::kPseudoIdScrollButtonInlineEnd
                | PseudoId::kPseudoIdScrollButtonBlockEnd
                | PseudoId::kPseudoIdScrollMarker
                | PseudoId::kPseudoIdOverscrollAreaParent
                | PseudoId::kPseudoIdViewTransition
                | PseudoId::kPseudoIdViewTransitionGroup
                | PseudoId::kPseudoIdViewTransitionGroupChildren
                | PseudoId::kPseudoIdViewTransitionImagePair
                | PseudoId::kPseudoIdViewTransitionOld
                | PseudoId::kPseudoIdViewTransitionNew
                | PseudoId::kPseudoIdSkeleton
        )
    }

    // cpp: style_resolver.cc:401-407
    fn GetPseudoId(
        &self,
        element: &B::Element,
        collector: Option<&B::ElementRuleCollector>,
    ) -> PseudoId {
        if self.backend.IsPseudoElement(element) {
            return self.backend.ElementPseudoIdForStyling(element);
        }
        collector.map_or(PseudoId::kPseudoIdNone, |collector| {
            self.backend.CollectorPseudoId(collector)
        })
    }

    // cpp: style_resolver.cc:571-580
    fn LeftToRightDeclaration(&self) -> Rc<B::CSSPropertyValueSet> {
        let declaration = self
            .backend
            .DeclarationCache()
            .left_to_right
            .get_or_init(|| {
                self.backend
                    .NewMutableCSSPropertyValueSet(CSSParserMode::kHTMLQuirksMode)
            });
        if self.backend.PropertyValueSetIsEmpty(declaration) {
            self.backend.SetLonghandProperty(
                declaration,
                CSSPropertyID::kDirection,
                CSSValueID::kLtr,
            );
        }
        Rc::clone(declaration)
    }

    // cpp: style_resolver.cc:582-591
    fn RightToLeftDeclaration(&self) -> Rc<B::CSSPropertyValueSet> {
        let declaration = self
            .backend
            .DeclarationCache()
            .right_to_left
            .get_or_init(|| {
                self.backend
                    .NewMutableCSSPropertyValueSet(CSSParserMode::kHTMLQuirksMode)
            });
        if self.backend.PropertyValueSetIsEmpty(declaration) {
            self.backend.SetLonghandProperty(
                declaration,
                CSSPropertyID::kDirection,
                CSSValueID::kRtl,
            );
        }
        Rc::clone(declaration)
    }

    // cpp: style_resolver.cc:661-674
    fn ScopedResolverFor<'e>(&self, element: &'e B::Element) -> Option<&'e B::ScopedStyleResolver>
    where
        B::TreeScope: 'e,
    {
        let mut tree_scope = self.backend.ElementTreeScope(element);
        if self.backend.IsSVGElement(element) {
            if let Some(corresponding) = self.backend.CorrespondingElement(element) {
                tree_scope = self.backend.ElementTreeScope(corresponding);
            }
        }
        if let Some(resolver) = self.backend.TreeScopeResolver(tree_scope) {
            debug_assert!(!self.backend.IsVTTElement(element));
            return Some(resolver);
        }
        None
    }

    // cpp: style_resolver.cc:681-713
    fn UAShadowPseudoCascading(&self, element: &B::Element) -> UAShadowPseudoResult {
        let Some(tree_scope) = self
            .backend
            .ParentTreeScope(self.backend.ElementTreeScope(element))
        else {
            return UAShadowPseudoResult {
                use_parent_resolver: false,
                cascade_style_attribute_in_parent_scope: false,
            };
        };
        let shadow_pseudo_id = self.backend.ShadowPseudoId(element);
        let is_vtt = self.backend.IsVTTElement(element);
        if shadow_pseudo_id.empty() && !is_vtt {
            return UAShadowPseudoResult {
                use_parent_resolver: false,
                cascade_style_attribute_in_parent_scope: false,
            };
        }
        let parent_resolver = self.backend.TreeScopeResolver(tree_scope);
        if parent_resolver.is_none() {
            return UAShadowPseudoResult {
                use_parent_resolver: true,
                cascade_style_attribute_in_parent_scope: false,
            };
        }
        let units = shadow_pseudo_id.utf16_units().unwrap_or(&[]);
        let begins_with_dash = units.starts_with(&[45]);
        debug_assert!(shadow_pseudo_id.empty() || !begins_with_dash
            || units.starts_with(&[45, 119, 101, 98, 107, 105, 116, 45])
            || units.starts_with(&[45, 105, 110, 116, 101, 114, 110, 97, 108, 45]),
            "shadow pseudo IDs should either begin with -webkit- or -internal- or not begin with a -");
        UAShadowPseudoResult {
            use_parent_resolver: true,
            cascade_style_attribute_in_parent_scope: begins_with_dash,
        }
    }

    // cpp: style_resolver.cc:718-732
    fn MatchHostRules(
        &self,
        element: &B::Element,
        collector: &mut B::ElementRuleCollector,
        tracker: Option<&B::StyleRuleUsageTracker>,
    ) {
        let resolver = self
            .backend
            .GetShadowRoot(element)
            .and_then(|root| self.backend.ShadowRootResolver(root));
        let Some(resolver) = resolver else {
            return;
        };
        self.backend.ClearMatchedRules(collector);
        self.backend.BeginAddingAuthorRulesForTreeScope(
            collector,
            self.backend.ResolverTreeScope(resolver),
        );
        self.backend
            .CollectMatchingShadowHostRules(resolver, collector);
        self.backend
            .SortAndTransferMatchedRules(collector, CascadeOrigin::kAuthor, false, tracker);
    }

    // cpp: style_resolver.cc:737-769
    fn MatchSlottedRulesForUAHost(
        &self,
        element: &B::Element,
        collector: &mut B::ElementRuleCollector,
        tracker: Option<&B::StyleRuleUsageTracker>,
    ) {
        if self
            .backend
            .PseudoIdForShadowElementName(self.backend.ShadowPseudoId(element))
            == PseudoId::kPseudoIdNone
        {
            return;
        }
        let host = self.backend.OwnerShadowHost(element);
        debug_assert!(host.is_some());
        self.MatchSlottedRules(
            host.expect("UA shadow pseudo requires a shadow host"),
            collector,
            tracker,
        );
    }

    // cpp: style_resolver.cc:775-802
    fn MatchSlottedRules(
        &self,
        element: &B::Element,
        collector: &mut B::ElementRuleCollector,
        tracker: Option<&B::StyleRuleUsageTracker>,
    ) {
        self.MatchSlottedRulesForUAHost(element, collector, tracker);
        let mut resolvers = Vec::new();
        let Some(mut slot) = self.backend.AssignedSlot(element) else {
            return;
        };
        loop {
            if let Some(resolver) = self
                .backend
                .TreeScopeResolver(self.backend.SlotTreeScope(slot))
            {
                resolvers.push((slot, resolver));
            }
            let Some(next_slot) = self.backend.SlotAssignedSlot(slot) else {
                break;
            };
            slot = next_slot;
        }
        for (slot, resolver) in resolvers.into_iter().rev() {
            self.backend.ClearMatchedRules(collector);
            self.backend
                .BeginAddingAuthorRulesForTreeScope(collector, self.backend.SlotTreeScope(slot));
            self.backend
                .CollectMatchingSlottedRules(resolver, collector);
            self.backend.SortAndTransferMatchedRules(
                collector,
                CascadeOrigin::kAuthor,
                false,
                tracker,
            );
        }
    }

    // cpp: style_resolver.cc:804-812
    fn GetTextTrackFromElement<'e>(&self, element: &'e B::Element) -> Option<&'e B::TextTrack> {
        if self.backend.IsVTTElement(element) {
            return self.backend.VTTElementTrack(element);
        }
        if self.backend.IsVTTCueBackgroundBox(element) {
            return self.backend.VTTCueBackgroundBoxTrack(element);
        }
        None
    }

    // cpp: style_resolver.cc:814-860
    fn MatchVTTRules(
        &self,
        element: &B::Element,
        collector: &mut B::ElementRuleCollector,
        tracker: Option<&B::StyleRuleUsageTracker>,
    ) {
        let Some(text_track) = self.GetTextTrackFromElement(element) else {
            return;
        };
        let styles = self.backend.TextTrackCSSStyleSheets(text_track);
        if !styles.is_empty() {
            self.backend.ClearMatchedRules(collector);
            let style_engine = self
                .backend
                .GetStyleEngine(self.backend.ElementDocument(element));
            let vtt_originating_element = self.backend.EnsureVTTOriginatingElement(style_engine);
            let mut rule_set_group_index = 0;
            let mut rule_set_group = self.backend.NewRuleSetGroup(rule_set_group_index);
            rule_set_group_index += 1;
            for style in styles {
                let Some(rule_set) = self.backend.RuleSetForSheet(style_engine, style) else {
                    continue;
                };
                self.backend
                    .RuleSetGroupAddRuleSet(&mut rule_set_group, &rule_set);
                if self.backend.RuleSetGroupIsFull(&rule_set_group) {
                    self.backend.CollectMatchingRules(
                        collector,
                        &rule_set_group,
                        None,
                        Some(vtt_originating_element),
                        None,
                    );
                    rule_set_group = self.backend.NewRuleSetGroup(rule_set_group_index);
                    rule_set_group_index += 1;
                }
            }
            if !self.backend.RuleSetGroupIsEmpty(&rule_set_group) {
                self.backend.CollectMatchingRules(
                    collector,
                    &rule_set_group,
                    None,
                    Some(vtt_originating_element),
                    None,
                );
            }
            self.backend.SortAndTransferMatchedRules(
                collector,
                CascadeOrigin::kAuthor,
                true,
                tracker,
            );
        }
    }

    // cpp: style_resolver.cc:862-879
    fn MatchHostPartRules(
        &self,
        element: &B::Element,
        collector: &mut B::ElementRuleCollector,
        _tracker: Option<&B::StyleRuleUsageTracker>,
    ) {
        let Some(part) = self.backend.GetPart(element) else {
            return;
        };
        if self.backend.PartLength(part) == 0 || !self.backend.IsInShadowTree(element) {
            return;
        }
        let current_names = self.backend.NewPartNames(self.backend.PartTokenSet(part));
        let tree_scope = self.backend.ElementTreeScope(element);
        if let Some(resolver) = self.backend.TreeScopeResolver(tree_scope) {
            self.backend
                .CollectMatchingPartPseudoRules(resolver, collector, &current_names);
        }
    }

    // cpp: style_resolver.cc:881-890
    fn MatchStyleAttribute(
        &self,
        element: &B::Element,
        collector: &mut B::ElementRuleCollector,
        _tracker: Option<&B::StyleRuleUsageTracker>,
    ) {
        if self.backend.IsStyledElement(element) {
            if self.backend.InlineStyle(element).is_some()
                && self.backend.CollectorPseudoId(collector) == PseudoId::kPseudoIdNone
            {
                let inline_style = self.backend.InlineStyle(element);
                self.backend.AddElementStyleProperties(
                    collector,
                    inline_style.as_deref(),
                    CascadeOrigin::kAuthor,
                    true,
                    true,
                );
            }
        }
    }

    // cpp: style_resolver.cc:894-915
    fn MatchElementScopeRules(
        &self,
        element: &B::Element,
        collector: &mut B::ElementRuleCollector,
        tracker: Option<&B::StyleRuleUsageTracker>,
    ) {
        let element_scope_resolver = self.ScopedResolverFor(element);
        let spr = self.UAShadowPseudoCascading(element);
        self.backend
            .BeginAddingAuthorRulesForTreeScope(collector, self.backend.ElementTreeScope(element));
        if let Some(element_scope_resolver) = element_scope_resolver {
            let _scope = self.backend.NewScopedRuleTreeScope(
                collector,
                self.backend.ResolverTreeScope(element_scope_resolver),
            );
            self.backend.ClearMatchedRules(collector);
            self.backend.CollectMatchingElementScopeRules(
                element_scope_resolver,
                self.backend
                    .TreeScopeRoot(self.backend.ElementTreeScope(element)),
                collector,
                None,
            );
            self.MatchHostPartRules(element, collector, tracker);
            self.backend.SortAndTransferMatchedRules(
                collector,
                CascadeOrigin::kAuthor,
                false,
                tracker,
            );
        }
        if !spr.cascade_style_attribute_in_parent_scope {
            self.MatchStyleAttribute(element, collector, tracker);
        }
    }

    // cpp: style_resolver.cc:946-955 (local set_part_names lambda)
    fn SetPartNames(
        &self,
        element: &B::Element,
        current_part_names: &mut Option<B::PartNames>,
    ) -> bool {
        if let Some(part) = self.backend.GetPart(element) {
            if self.backend.PartLength(part) != 0 && self.backend.IsInShadowTree(element) {
                *current_part_names =
                    Some(self.backend.NewPartNames(self.backend.PartTokenSet(part)));
                return true;
            }
        }
        *current_part_names = None;
        false
    }

    // cpp: style_resolver.cc:917-1032
    fn MatchOuterScopeRules(
        &self,
        matching_element: &B::Element,
        collector: &mut B::ElementRuleCollector,
        tracker: Option<&B::StyleRuleUsageTracker>,
    ) {
        #[derive(PartialEq, Eq)]
        enum MatchingState {
            kDone,
            kShadowPseudo,
            kPart,
            kPartAboveShadowPseudo,
        }
        let mut state = MatchingState::kDone;
        let mut current_part_names = None;
        let mut style_attribute_cascaded_in_parent_scope = false;
        if self.SetPartNames(matching_element, &mut current_part_names) {
            state = MatchingState::kPart;
        } else {
            let spr = self.UAShadowPseudoCascading(matching_element);
            if spr.use_parent_resolver {
                state = MatchingState::kShadowPseudo;
                style_attribute_cascaded_in_parent_scope =
                    spr.cascade_style_attribute_in_parent_scope;
            }
        }
        let mut owner_shadow_host = self.backend.OwnerShadowHost(matching_element);
        while let Some(element) = owner_shadow_host {
            if state == MatchingState::kDone {
                break;
            }
            let tree_scope = self.backend.ElementTreeScope(element);
            if let Some(resolver) = self.backend.TreeScopeResolver(tree_scope) {
                self.backend.ClearMatchedRules(collector);
                self.backend.BeginAddingAuthorRulesForTreeScope(
                    collector,
                    self.backend.ResolverTreeScope(resolver),
                );
                if state == MatchingState::kPart {
                    self.backend.CollectMatchingPartPseudoRules(
                        resolver,
                        collector,
                        current_part_names
                            .as_ref()
                            .expect("part state requires part names"),
                    );
                } else {
                    self.backend.CollectMatchingElementScopeRules(
                        resolver,
                        self.backend.TreeScopeRoot(tree_scope),
                        collector,
                        current_part_names.as_ref(),
                    );
                }
                self.backend.SortAndTransferMatchedRules(
                    collector,
                    CascadeOrigin::kAuthor,
                    false,
                    tracker,
                );
                if style_attribute_cascaded_in_parent_scope {
                    self.MatchStyleAttribute(matching_element, collector, tracker);
                }
            }
            if state == MatchingState::kShadowPseudo {
                assert!(current_part_names.is_none());
                style_attribute_cascaded_in_parent_scope = false;
                if self.SetPartNames(element, &mut current_part_names) {
                    state = MatchingState::kPartAboveShadowPseudo;
                } else {
                    state = MatchingState::kDone;
                }
            } else {
                assert!(current_part_names.is_some());
                if self.backend.HasPartNamesMap(element) {
                    self.backend.PushPartNamesMap(
                        current_part_names
                            .as_mut()
                            .expect("part state requires part names"),
                        self.backend.PartNamesMap(element),
                    );
                } else {
                    state = MatchingState::kDone;
                }
            }
            owner_shadow_host = self.backend.OwnerShadowHost(element);
        }
    }

    // cpp: style_resolver.cc:1036-1039; h:341
    fn MatchPositionTryRules(&self, collector: &mut B::ElementRuleCollector) {
        self.backend.AddTryStyleProperties(collector);
        self.backend.AddTryTacticsStyleProperties(collector);
    }

    // cpp: style_resolver.cc:1041-1051; h:342
    fn MatchAuthorRules(&self, element: &B::Element, collector: &mut B::ElementRuleCollector) {
        let originating_element = self.UltimateOriginatingElementOrSelf(element);
        self.MatchHostRules(originating_element, collector, self.tracker_);
        self.MatchSlottedRules(originating_element, collector, self.tracker_);
        self.MatchElementScopeRules(element, collector, self.tracker_);
        self.MatchOuterScopeRules(originating_element, collector, self.tracker_);
        self.MatchVTTRules(element, collector, self.tracker_);
        self.MatchPositionTryRules(collector);
    }

    // cpp: style_resolver.cc:1053-1058; h:339
    fn MatchUserRules(&self, collector: &mut B::ElementRuleCollector) {
        self.backend.ClearMatchedRules(collector);
        self.backend
            .CollectMatchingUserRules(self.backend.GetStyleEngine(self.GetDocument()), collector);
        self.backend.SortAndTransferMatchedRules(
            collector,
            CascadeOrigin::kUser,
            false,
            self.tracker_,
        );
    }

    // cpp: style_resolver.cc:1062-1074
    fn IsInMediaUAShadow(&self, element: &B::Element) -> bool {
        let element = self.UltimateOriginatingElementOrSelf(element);
        let Some(mut root) = self.backend.ContainingShadowRoot(element) else {
            return false;
        };
        if !self.backend.ShadowRootIsUserAgent(root) {
            return false;
        }
        let outer_root = loop {
            let outer_root = root;
            let next_root = self
                .backend
                .ContainingShadowRoot(self.backend.ShadowRootHost(root));
            match next_root {
                Some(next_root) if self.backend.ShadowRootIsUserAgent(next_root) => {
                    root = next_root
                }
                _ => break outer_root,
            }
        };
        self.backend
            .IsMediaElement(self.backend.ShadowRootHost(outer_root))
    }

    // cpp: style_resolver.cc:1101-1156; h:410-413
    fn ForEachUARulesForElement<F>(
        &self,
        element: &B::Element,
        collector: Option<&B::ElementRuleCollector>,
        mut func: F,
    ) where
        F: FnMut(Rc<B::RuleSet>, UASheetCacheKeyIndex),
    {
        let sheets = self.backend;
        if sheets.IsHTMLElement(element)
            || sheets.IsPseudoElement(element)
            || sheets.IsVTTElement(element)
        {
            func(
                sheets.DefaultHtmlStyle(),
                UASheetCacheKeyIndex::kHTMLUASheet,
            );
        } else if sheets.IsSVGElement(element) {
            func(sheets.DefaultSVGStyle(), UASheetCacheKeyIndex::kSVGUASheet);
        } else if sheets.NamespaceURI(element) == sheets.MathMLNamespaceURI() {
            func(
                sheets.DefaultMathMLStyle(),
                UASheetCacheKeyIndex::kMathMLUASheet,
            );
        }
        if sheets.HasFullscreenElements() {
            func(
                sheets.DefaultFullscreenStyle(),
                UASheetCacheKeyIndex::kFullscreenUASheet,
            );
        }
        if sheets.InQuirksMode(self.GetDocument()) {
            func(
                sheets.DefaultHtmlQuirksStyle(),
                UASheetCacheKeyIndex::kQuirksUASheet,
            );
        }
        if sheets.IsViewSource(self.GetDocument()) {
            func(
                sheets.DefaultViewSourceStyle(),
                UASheetCacheKeyIndex::kViewSourceUASheet,
            );
        }
        if self.IsForcedColorsModeEnabled() {
            func(
                sheets.DefaultForcedColorStyle(),
                UASheetCacheKeyIndex::kForcedColorsUASheet,
            );
        }
        if sheets.IsJSONDocument(self.GetDocument()) {
            func(
                sheets.DefaultJSONDocumentStyle(),
                UASheetCacheKeyIndex::kJSONUASheet,
            );
        }
        let pseudo_id = self.GetPseudoId(element, collector);
        if pseudo_id == PseudoId::kPseudoIdNone {
            return;
        }
        if Self::IsPseudoElementWithUAStyle(pseudo_id)
            && sheets.DefaultPseudoElementStyleOrNull().is_some()
        {
            let rules = sheets
                .DefaultPseudoElementStyleOrNull()
                .expect("the default pseudo-element sheet was present above");
            func(rules, UASheetCacheKeyIndex::kPseudoElementUASheet);
        }
        if sheets.IsTransitionPseudoElement(pseudo_id) {
            if let Some(rules) =
                sheets.ActiveViewTransitionStyle(sheets.GetStyleEngine(self.GetDocument()), element)
            {
                func(rules, UASheetCacheKeyIndex::kViewTransitionUASheet);
            }
        }
    }

    // cpp: style_resolver.cc:1158-1248; h:338
    fn MatchUARules(&mut self, element: &B::Element, collector: &mut B::ElementRuleCollector) {
        let backend = self.backend;
        backend.SetMatchingUARules(collector, true);
        let mut cache_key = 0u32;
        self.ForEachUARulesForElement(element, Some(collector), |_, index| {
            cache_key |= 1 << index as u32;
        });
        let can_use_cache =
            (cache_key & (1 << UASheetCacheKeyIndex::kViewTransitionUASheet as u32)) == 0;
        let mut cache = backend.RuleSetGroupCache().borrow_mut();
        let rule_set_group_index = cache.iter().position(|(key, _)| *key == cache_key);
        let index = if rule_set_group_index.is_none() || !can_use_cache {
            let index = match rule_set_group_index {
                None => {
                    cache.push((cache_key, backend.NewRuleSetGroup(0)));
                    cache.len() - 1
                }
                Some(index) => {
                    cache[index].1 = backend.NewRuleSetGroup(0);
                    index
                }
            };
            self.ForEachUARulesForElement(element, Some(collector), |rules, _| {
                backend.RuleSetGroupAddRuleSet(&mut cache[index].1, &rules);
            });
            index
        } else {
            rule_set_group_index.expect("cache hit must have a group")
        };
        let rule_set_group = &cache[index].1;
        #[cfg(debug_assertions)]
        {
            let mut reference = backend.NewRuleSetGroup(0);
            self.ForEachUARulesForElement(element, Some(collector), |rules, _| {
                backend.RuleSetGroupAddRuleSet(&mut reference, &rules);
            });
            backend.RuleSetGroupAssertEqualTo(rule_set_group, &reference);
        }
        if !backend.RuleSetGroupIsEmpty(rule_set_group) {
            backend.ClearMatchedRules(collector);
            backend.CollectMatchingRules(collector, rule_set_group, None, None, None);
            backend.SortAndTransferMatchedRules(
                collector,
                CascadeOrigin::kUserAgent,
                false,
                self.tracker_,
            );
        }
        drop(cache);
        if self.IsInMediaUAShadow(element) {
            let rule_set = if self.IsForcedColorsModeEnabled() {
                backend.DefaultForcedColorsMediaControlsStyle()
            } else {
                backend.DefaultMediaControlsStyle()
            };
            if !self
                .media_controls_cache_key_
                .as_ref()
                .is_some_and(|key| Rc::ptr_eq(key, &rule_set))
            {
                self.media_controls_cached_rule_set_group_ = backend.NewRuleSetGroup(0);
                backend.RuleSetGroupAddRuleSet(
                    &mut self.media_controls_cached_rule_set_group_,
                    &rule_set,
                );
                self.media_controls_cache_key_ = Some(rule_set);
            }
            backend.ClearMatchedRules(collector);
            backend.CollectMatchingRules(
                collector,
                &self.media_controls_cached_rule_set_group_,
                None,
                None,
                None,
            );
            backend.SortAndTransferMatchedRules(
                collector,
                CascadeOrigin::kUserAgent,
                false,
                self.tracker_,
            );
        }
        backend.SetMatchingUARules(collector, false);
    }

    // cpp: style_resolver.cc:1250-1276; h:340
    fn MatchPresentationalHints(
        &self,
        state: &B::StyleResolverState,
        collector: &mut B::ElementRuleCollector,
    ) {
        let element = self.backend.StateGetElement(state);
        if self.backend.IsStyledElement(element) && !self.backend.StateIsForPseudoElement(state) {
            let properties = self.backend.PresentationAttributeStyle(element);
            self.backend.AddElementStyleProperties(
                collector,
                properties.as_deref(),
                CascadeOrigin::kAuthorPresentationalHint,
                true,
                false,
            );
            let additional_properties = self.backend.AdditionalPresentationAttributeStyle(element);
            self.backend.AddElementStyleProperties(
                collector,
                additional_properties.as_deref(),
                CascadeOrigin::kAuthorPresentationalHint,
                true,
                false,
            );
            if self.backend.IsHTMLElement(element) && self.backend.HasDirectionAuto(element) {
                let direction = if self.backend.CachedDirectionality(element) == TextDirection::kLtr
                {
                    self.LeftToRightDeclaration()
                } else {
                    self.RightToLeftDeclaration()
                };
                self.backend.AddElementStyleProperties(
                    collector,
                    Some(&direction),
                    CascadeOrigin::kAuthorPresentationalHint,
                    true,
                    false,
                );
            }
        }
    }

    // cpp: style_resolver.cc:1279-1313; h:343-345
    fn MatchAllRules(
        &mut self,
        state: &B::StyleResolverState,
        collector: &mut B::ElementRuleCollector,
        include_smil_properties: bool,
    ) {
        let element = self.backend.StateGetElement(state);
        self.MatchUARules(element, collector);
        self.MatchUserRules(collector);
        self.MatchPresentationalHints(state, collector);
        self.MatchAuthorRules(element, collector);
        if self.backend.IsStyledElement(element) && !self.backend.StateIsForPseudoElement(state) {
            self.backend.BeginAddingAuthorRulesForTreeScope(
                collector,
                self.backend.ElementTreeScope(element),
            );
            let svg_element = self.backend.IsSVGElement(element).then_some(element);
            if let Some(svg_element) = svg_element.filter(|_| include_smil_properties) {
                let svg_element = self
                    .backend
                    .CorrespondingElement(svg_element)
                    .unwrap_or(svg_element);
                let properties = self.backend.AnimatedSMILStyleProperties(svg_element);
                self.backend.AddElementStyleProperties(
                    collector,
                    properties.as_deref(),
                    CascadeOrigin::kAuthor,
                    false,
                    false,
                );
            }
        }
    }

    // cpp: style_resolver.cc:1315-1332; h:145
    pub fn StyleForViewport(&self) -> B::ComputedStyle {
        let mut builder = self.InitialStyleBuilderForElement();
        self.backend.SetZIndex(&mut builder, 0);
        self.backend.SetForcesStackingContext(&mut builder, true);
        self.backend.SetDisplay(&mut builder, EDisplay::kBlock);
        self.backend.SetPosition(&mut builder, EPosition::kAbsolute);
        self.backend.SetOverflowX(&mut builder, EOverflow::kAuto);
        self.backend.SetOverflowY(&mut builder, EOverflow::kAuto);
        self.backend.ApplyVisionDeficiencyStyle(
            self.backend.GetStyleEngine(self.GetDocument()),
            &mut builder,
        );
        self.backend.TakeStyle(builder)
    }
}

// Typed field selectors at the ComputedStyleBuilder boundary. They dispatch
// only one external getter/setter; no StyleResolver decision is implemented by
// the backend. Names are the source's fields (Has* and Is* prefixes shortened).
#[derive(Clone, Copy)]
pub enum ResolverStyleFlag {
    AnchorFunctions,
    AnchorEvaluator,
    RootRelativeUnits,
    GlyphRelativeUnits,
    LineHeightRelativeUnits,
    FontRelativeUnits,
    SiblingFunctions,
    InlineStyleLostCascade,
    ExplicitInheritance,
    CanAffectAnimations,
    OriginalDisplayInlineType,
    EmUnits,
    StaticViewportUnits,
    DynamicViewportUnits,
    ContainerRelativeValue,
    LogicalDirectionRelativeUnits,
    ElementDependentRandomFunctions,
    EnsuredInDisplayNone,
    SkipsContents,
    IsLink,
    HTMLInert,
    HTMLInertIsInherited,
    CSSInert,
    CSSInertIsInherited,
    NonUniversalHighlightPseudoStyles,
    NonUaHighlightPseudoStyles,
    HighlightsDependOnSizeContainerQueries,
    AffectedByDrag,
    AffectedByFocusWithin,
    AffectedByHover,
    AffectedByActive,
    StartingStyle,
    DependsOnSizeContainerQueries,
    DependsOnStyleContainerQueries,
    DependsOnScrollStateContainerQueries,
    DependsOnAnchoredContainerQueries,
    FirstLineDependsOnSizeContainerQueries,
}
#[derive(Clone, Copy)]
pub enum ResolverLengthFlag {
    Em,
    RootFontRelative,
    GlyphRelative,
    Viewport,
    SmallLargeViewport,
    DynamicViewport,
    ContainerRelative,
    TreeScopedReference,
    AnchorRelative,
    LogicalDirectionRelative,
    LhRelative,
    RlhRelative,
    SiblingRelative,
    ElementDependentRandom,
}
#[derive(Clone, Copy)]
pub enum ResolverMatchResultFlag {
    NonUniversalHighlightPseudoStyles,
    NonUaHighlightPseudoStyles,
    HighlightsDependOnSizeContainerQueries,
    SizeContainerQueries,
    StyleContainerQueries,
    ScrollStateContainerQueries,
    AnchoredContainerQueries,
    FirstLineSizeContainerQueries,
    StaticViewportUnits,
    DynamicViewportUnits,
    RootUnitContainerQueries,
    ConditionallyAffectsAnimations,
}
#[derive(Clone, Copy)]
pub enum ResolverAnimationProperty {
    CustomProperties,
    Revert,
    FontAffecting,
    LineHeight,
    Zoom,
    Display,
}
#[derive(Clone, Copy)]
pub enum ResolverRecalcFlag {
    EnsuringStyle,
    CanUseIncrementalStyle,
    SizeContainer,
    AnchoredContainer,
}
#[derive(Clone, Copy)]
pub enum ResolverRuntimeFeature {
    OverlayProperty,
    OverlayGlobalRuleRemoval,
    CSSZoomAnimation,
    InheritUserModifyWithoutContenteditable,
}
#[derive(Clone, Copy)]
pub enum ResolverMathMLKind {
    Space,
    Padded,
    Fraction,
    Operator,
    Other,
}
#[derive(Clone, Copy)]
pub enum ResolverMathMLProperty {
    Baseline,
    PaddedDepth,
    PaddedLSpace,
    PaddedVOffset,
    FractionBarThickness,
    LSpace,
    RSpace,
    MinSize,
    MaxSize,
}
#[derive(Clone, Copy)]
pub enum ResolverVisitedProperty {
    Color,
    CaretColor,
    FillPaint,
    StrokePaint,
    TextEmphasisColor,
    TextFillColor,
    TextStrokeColor,
}

// cpp: style_resolver.h:348,353-366
// The unused, uninitialized source is_hit field (h:352) is source-only.
struct CacheSuccess<K, E> {
    key: K,
    cached_matched_properties: Option<Rc<E>>,
}
impl<K, E> CacheSuccess<K, E> {
    fn IsHit(&self) -> bool {
        self.cached_matched_properties.is_some()
    }
    fn IsStyleAdjusted<B: StyleResolverBaseStyleBackend<CacheEntry = E>>(
        &self,
        backend: &B,
    ) -> bool {
        backend.ElementTypeIsStyleAdjusted(
            backend.CacheEntryElementType(
                self.cached_matched_properties
                    .as_ref()
                    .expect("cache hit requires an entry"),
            ),
        )
    }
}

/// Mandatory operations belonging to StyleResolverState, StyleCascade,
/// MatchedPropertiesCache, StyleAdjuster, CSSAnimations and DOM/style objects.
/// State/Cascade/Collector handles retain and alias the real shared storage;
/// an owning returned handle never clones an Element or its identity.
/// Bool field selectors dispatch exact getters/setters only.
pub trait StyleResolverBaseStyleBackend: StyleResolverRuleMatchingBackend + Sized {
    type StyleRequest;
    type StyleRecalcContext;
    type StyleCascade;
    type MatchResult;
    type CacheKey;
    type CacheEntry;
    type ElementTypeForCache;
    type CSSPropertyValue;
    type CSSValue;
    type LengthConversionFlags;
    type StyleBaseData;
    type CSSBitset;
    type ElementAnimations;
    type AnimationData;
    type ActiveInterpolationsMap;
    type AnchorEvaluator;
    type ContainerNode;
    type StyleRule;
    type CustomHighlightNames;
    type CurrentColor;

    fn RetainElement(&self, element: &Self::Element) -> Rc<Self::Element>;
    fn ElementComputedStyle(&self, element: &Self::Element) -> Option<Rc<Self::ComputedStyle>>;
    fn ParentElement(&self, element: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ParentElementOrDocumentFragment(
        &self,
        element: &Self::Element,
    ) -> Option<Rc<Self::ContainerNode>>;
    fn ElementTreeRoot(&self, element: &Self::Element) -> Rc<Self::ContainerNode>;
    fn DocumentElement(&self, document: &Self::Document) -> Option<Rc<Self::Element>>;
    fn IsLink(&self, element: &Self::Element) -> bool;
    fn IsBodyElement(&self, element: &Self::Element) -> bool;
    fn IsMathMLElement(&self, element: &Self::Element) -> bool;
    fn IsAnchorOrAreaElement(&self, element: &Self::Element) -> bool;
    fn IsAtShadowBoundary(&self, element: &Self::Element) -> bool;
    fn IsInlineIndependentStyleChange(&self, element: &Self::Element) -> bool;
    fn HasCustomStyleCallbacks(&self, element: &Self::Element) -> bool;
    fn HasContenteditableAttribute(&self, element: &Self::Element) -> bool;
    fn ImplicitAnchorElement(&self, element: &Self::Element) -> Option<Rc<Self::Element>>;
    fn GetCascadeFilter(&self, element: &Self::Element) -> CascadeFilter;
    fn MarkChildrenAffectedByForwardPositionalRules(&self, node: &Self::ContainerNode);
    fn MarkChildrenAffectedByBackwardPositionalRules(&self, node: &Self::ContainerNode);
    fn SetUsesTreeCountingFunctions(&self, engine: &Self::StyleEngine);
    fn SetUsesRootRelativeUnits(&self, engine: &Self::StyleEngine, value: bool);
    fn SetUsesGlyphRelativeUnits(&self, engine: &Self::StyleEngine, value: bool);
    fn SetUsesLineHeightUnits(&self, engine: &Self::StyleEngine, value: bool);
    fn AddViewportUnitFlags(&self, document: &Self::Document, flags: u32);
    fn SetTextLinkTextColor(&self, document: &Self::Document, color: Self::CurrentColor);
    fn ActiveModalDialog(&self, document: &Self::Document) -> Option<Rc<Self::Element>>;
    fn FullscreenElementFrom(&self, document: &Self::Document) -> Option<Rc<Self::Element>>;
    fn RuntimeFeatureEnabled(&self, feature: ResolverRuntimeFeature) -> bool;

    fn NewStyleRequest(&self) -> Self::StyleRequest;
    fn SetRequestParentOverride(
        &self,
        request: &mut Self::StyleRequest,
        style: Option<&Self::ComputedStyle>,
    );
    fn SetRequestLayoutParentOverride(
        &self,
        request: &mut Self::StyleRequest,
        style: Option<&Self::ComputedStyle>,
    );
    fn RequestIsPseudoStyle(&self, request: &Self::StyleRequest) -> bool;
    fn RequestPseudoId(&self, request: &Self::StyleRequest) -> PseudoId;
    fn RequestUAOnly(&self, request: &Self::StyleRequest) -> bool;
    fn RequestExcludesSMIL(&self, request: &Self::StyleRequest) -> bool;
    fn NewStyleRecalcContext(&self) -> Self::StyleRecalcContext;
    fn RecalcFlag(&self, context: &Self::StyleRecalcContext, flag: ResolverRecalcFlag) -> bool;
    fn NewStyleResolverState(
        &self,
        document: &Self::Document,
        element: &Self::Element,
        context: Option<&Self::StyleRecalcContext>,
        request: &Self::StyleRequest,
    ) -> Self::StyleResolverState;
    fn NewStyleCascade(&self, state: &mut Self::StyleResolverState) -> Self::StyleCascade;
    fn StateElement(&self, state: &Self::StyleResolverState) -> Rc<Self::Element>;
    fn StateAnimatingElement(&self, state: &Self::StyleResolverState) -> Option<Rc<Self::Element>>;
    fn StatePseudoElement(&self, state: &Self::StyleResolverState) -> Option<Rc<Self::Element>>;
    fn StateStyledElement(&self, state: &Self::StyleResolverState) -> Option<Rc<Self::Element>>;
    fn StateParentStyle(&self, state: &Self::StyleResolverState)
        -> Option<Rc<Self::ComputedStyle>>;
    fn StateLayoutParentStyle(
        &self,
        state: &Self::StyleResolverState,
    ) -> Option<Rc<Self::ComputedStyle>>;
    fn StateOriginatingElementStyle(
        &self,
        state: &Self::StyleResolverState,
    ) -> Option<Rc<Self::ComputedStyle>>;
    fn StateSetParentStyle(
        &self,
        state: &mut Self::StyleResolverState,
        style: Rc<Self::ComputedStyle>,
    );
    fn StateSetLayoutParentStyle(
        &self,
        state: &mut Self::StyleResolverState,
        style: Rc<Self::ComputedStyle>,
    );
    fn StateIsForHighlight(&self, state: &Self::StyleResolverState) -> bool;
    fn StateHadNoMatchedProperties(&self, state: &Self::StyleResolverState) -> bool;
    fn StateSetHadNoMatchedProperties(&self, state: &mut Self::StyleResolverState);
    fn StateCanTriggerAnimations(&self, state: &Self::StyleResolverState) -> bool;
    fn StateCanAffectAnimations(&self, state: &Self::StyleResolverState) -> bool;
    fn StateAffectsCompositorSnapshots(&self, state: &Self::StyleResolverState) -> bool;
    fn StateInsideLink(&self, state: &Self::StyleResolverState) -> EInsideLink;
    fn StateStyleBuilder<'s>(
        &self,
        state: &'s Self::StyleResolverState,
    ) -> &'s Self::ComputedStyleBuilder;
    fn StateStyleBuilderMut<'s>(
        &self,
        state: &'s mut Self::StyleResolverState,
    ) -> &'s mut Self::ComputedStyleBuilder;
    fn StateCreateNewClonedStyle(
        &self,
        state: &mut Self::StyleResolverState,
        style: &Self::ComputedStyle,
    );
    fn StateCreateNewStyle(
        &self,
        state: &mut Self::StyleResolverState,
        initial: &Self::ComputedStyle,
        parent: &Self::ComputedStyle,
        at_shadow_boundary: bool,
    );
    fn StateTakeStyle(&self, state: &mut Self::StyleResolverState) -> Rc<Self::ComputedStyle>;
    fn StateLoadPendingResources(&self, state: &mut Self::StyleResolverState);
    fn StateUpdateFont(&self, state: &mut Self::StyleResolverState);
    fn StateSetTreeScopedReference(&self, state: &mut Self::StyleResolverState);
    fn StateSetConditionallyAffectsAnimations(&self, state: &mut Self::StyleResolverState);
    fn StateSetComputedStyleFlagsFromAuthorFlags(
        &self,
        state: &mut Self::StyleResolverState,
        flags: u64,
    );
    fn StateRejectedLegacyOverlapping(&self, state: &Self::StyleResolverState) -> bool;

    fn StyleFlag(&self, style: &Self::ComputedStyle, flag: ResolverStyleFlag) -> bool;
    fn BuilderFlag(&self, builder: &Self::ComputedStyleBuilder, flag: ResolverStyleFlag) -> bool;
    fn SetBuilderFlag(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        flag: ResolverStyleFlag,
        value: bool,
    );
    fn BuilderDisplay(&self, builder: &Self::ComputedStyleBuilder) -> EDisplay;
    fn StyleDisplay(&self, style: &Self::ComputedStyle) -> EDisplay;
    fn IsDisplayInlineType(&self, display: EDisplay) -> bool;
    fn StyleInsideLink(&self, style: &Self::ComputedStyle) -> EInsideLink;
    fn SetInsideLink(&self, builder: &mut Self::ComputedStyleBuilder, inside_link: EInsideLink);
    fn SetStyleType(&self, builder: &mut Self::ComputedStyleBuilder, pseudo_id: PseudoId);
    fn BuilderStyleType(&self, builder: &Self::ComputedStyleBuilder) -> PseudoId;
    fn CopyHighlightPropertiesFrom(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        originating: &Self::ComputedStyle,
    );
    fn BuilderPositionAnchorIsName(&self, builder: &Self::ComputedStyleBuilder) -> bool;
    fn StylePositionAnchorIsName(&self, style: &Self::ComputedStyle) -> bool;
    fn BuilderHasPositionTryFallbacks(&self, builder: &Self::ComputedStyleBuilder) -> bool;
    fn StyleHasPositionTryFallbacks(&self, style: &Self::ComputedStyle) -> bool;
    fn BuilderInteractivity(&self, builder: &Self::ComputedStyleBuilder) -> EInteractivity;
    fn BuilderInteractivityIsInherited(&self, builder: &Self::ComputedStyleBuilder) -> bool;
    fn BuilderCurrentColor(&self, builder: &Self::ComputedStyleBuilder) -> Self::CurrentColor;
    fn BuilderViewportUnitFlags(&self, builder: &Self::ComputedStyleBuilder) -> u32;
    fn BuilderCloneStyle(&self, builder: &Self::ComputedStyleBuilder) -> Rc<Self::ComputedStyle>;
    fn StyleBaseData(&self, style: &Self::ComputedStyle) -> Option<Rc<Self::StyleBaseData>>;
    fn BuilderBaseData(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> Option<Rc<Self::StyleBaseData>>;
    fn SetBaseData(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        data: Option<Rc<Self::StyleBaseData>>,
    );
    fn BaseComputedStyle(&self, data: &Self::StyleBaseData) -> Option<Rc<Self::ComputedStyle>>;
    fn BaseImportantSet(&self, data: &Self::StyleBaseData) -> Option<Rc<Self::CSSBitset>>;
    fn NewStyleBaseData(
        &self,
        style: Rc<Self::ComputedStyle>,
        important: Option<Rc<Self::CSSBitset>>,
    ) -> Rc<Self::StyleBaseData>;
    fn BuilderBaseComputedStyle(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> Rc<Self::ComputedStyle>;
    fn SetVisitedPropertyFromUnvisited(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        property: ResolverVisitedProperty,
    );
    fn StyleUserModify(&self, style: &Self::ComputedStyle) -> EUserModify;
    fn StyleEffectiveZoom(&self, style: &Self::ComputedStyle) -> f32;
    fn StyleInheritedVariablesHash(&self, style: &Self::ComputedStyle) -> u32;
    fn StyleInheritedBitFieldsHash(&self, style: &Self::ComputedStyle) -> u32;
    fn StyleFontComputedSize(&self, style: &Self::ComputedStyle) -> f32;
    fn StyleSetChildHasExplicitInheritance(&self, style: &Self::ComputedStyle);

    fn PropertySetProperties<'p>(
        &self,
        set: &'p Self::CSSPropertyValueSet,
    ) -> &'p [Self::CSSPropertyValue];
    fn PropertyID(&self, property: &Self::CSSPropertyValue) -> CSSPropertyID;
    fn PropertyName(
        &self,
        property: &Self::CSSPropertyValue,
    ) -> crate::css_property_name::CSSPropertyName;
    fn PropertyValue<'p>(&self, property: &'p Self::CSSPropertyValue) -> &'p Self::CSSValue;
    fn ValueIsUnparsedDeclaration(&self, value: &Self::CSSValue) -> bool;
    fn ValueIsPendingSubstitution(&self, value: &Self::CSSValue) -> bool;
    fn ValueIsCascadeDependentKeyword(&self, value: &Self::CSSValue) -> bool;
    fn ValueIsMathFunction(&self, value: &Self::CSSValue) -> bool;
    fn MathValueHasAnchorFunctions(&self, value: &Self::CSSValue) -> bool;
    /// StyleBuilder::ApplyProperty after CSSValue::EnsureScopedValue(scope).
    fn ApplyScopedProperty(
        &self,
        name: crate::css_property_name::CSSPropertyName,
        state: &mut Self::StyleResolverState,
        value: &Self::CSSValue,
        scope: &Self::TreeScope,
    );
    fn TakeLengthConversionFlags(
        &self,
        state: &mut Self::StyleResolverState,
    ) -> Self::LengthConversionFlags;
    fn LengthFlagsEmpty(&self, flags: &Self::LengthConversionFlags) -> bool;
    fn HasLengthFlag(&self, flags: &Self::LengthConversionFlags, flag: ResolverLengthFlag) -> bool;
    fn MathMLKind(&self, element: &Self::Element) -> ResolverMathMLKind;
    /// Calls the selected MathMLElement AddMath*IfNeeded method with the state's
    /// builder and CssToLengthConversionData; selection stays in StyleResolver.
    fn AddMathPropertyIfNeeded(
        &self,
        element: &Self::Element,
        state: &mut Self::StyleResolverState,
        property: ResolverMathMLProperty,
    );
    fn StateAnchorEvaluator(
        &self,
        state: &Self::StyleResolverState,
    ) -> Option<Rc<Self::AnchorEvaluator>>;
    fn ComputeAnchorCenterOffsets(
        &self,
        evaluator: &Self::AnchorEvaluator,
        builder: &Self::ComputedStyleBuilder,
    ) -> Option<PhysicalOffset>;
    fn SetAnchorCenterOffset(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        offset: Option<PhysicalOffset>,
    );

    fn EnsureUAStyleForElement(&self, engine: &Self::StyleEngine, element: &Self::Element);
    fn EnsureUAStyleForPseudoElement(&self, engine: &Self::StyleEngine, pseudo_id: PseudoId);
    fn NewCollectorForCascade(
        &self,
        state: &Self::StyleResolverState,
        context: &Self::StyleRecalcContext,
        selector_filter: &mut Self::SelectorFilter,
        cascade: &mut Self::StyleCascade,
    ) -> Self::ElementRuleCollector;
    fn SetPseudoElementStyleRequest(
        &self,
        collector: &mut Self::ElementRuleCollector,
        request: &Self::StyleRequest,
    );
    fn CascadeMatchResult(&self, cascade: &Self::StyleCascade) -> Rc<Self::MatchResult>;
    fn NewMatchResult(&self) -> Self::MatchResult;
    fn NewCollectorForResult(
        &self,
        state: &Self::StyleResolverState,
        context: &Self::StyleRecalcContext,
        selector_filter: &mut Self::SelectorFilter,
        result: &mut Self::MatchResult,
    ) -> Self::ElementRuleCollector;
    fn AddCascadeMatchedProperties(
        &self,
        cascade: &mut Self::StyleCascade,
        properties: &Self::CSSPropertyValueSet,
        origin: CascadeOrigin,
    );
    fn NewInitialColorValue(&self) -> Self::CSSValue;
    fn NewInheritedValue(&self) -> Self::CSSValue;
    fn NewIdentifierValue(&self, value: CSSValueID) -> Self::CSSValue;
    fn SetProperty(
        &self,
        set: &Self::CSSPropertyValueSet,
        property: CSSPropertyID,
        value: &Self::CSSValue,
        important: bool,
    );
    fn MatchResultHasMatchedProperties(&self, result: &Self::MatchResult) -> bool;
    fn MatchResultIsCacheable(&self, result: &Self::MatchResult) -> bool;
    fn MatchResultFlag(&self, result: &Self::MatchResult, flag: ResolverMatchResultFlag) -> bool;
    fn MatchResultHasFlag(&self, result: &Self::MatchResult, flag: MatchFlag) -> bool;
    fn CustomHighlightNames<'r>(
        &self,
        result: &'r Self::MatchResult,
    ) -> &'r Self::CustomHighlightNames;
    fn CustomHighlightNamesEmpty(&self, names: &Self::CustomHighlightNames) -> bool;
    fn SetCustomHighlightNames(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        names: &Self::CustomHighlightNames,
    );
    fn MatchResultPseudoElementBits(&self, result: &Self::MatchResult) -> u32;
    fn SetPseudoElementStyles(&self, builder: &mut Self::ComputedStyleBuilder, bits: u32);
    fn CascadeApply(&self, cascade: &mut Self::StyleCascade, filter: CascadeFilter);
    fn CascadeInlineStyleLost(&self, cascade: &Self::StyleCascade) -> bool;
    fn CascadeReleaseImportantSet(
        &self,
        cascade: &mut Self::StyleCascade,
    ) -> Option<Rc<Self::CSSBitset>>;
    fn CascadeAddInterpolations(
        &self,
        cascade: &mut Self::StyleCascade,
        interpolations: &Self::ActiveInterpolationsMap,
        origin: CascadeOrigin,
    );
    fn AdjustComputedStyle(
        &self,
        state: &mut Self::StyleResolverState,
        element_if_not_pseudo: Option<&Self::Element>,
    );
    fn RunUncacheableStyleAdjustment(
        &self,
        state: &mut Self::StyleResolverState,
        element: &Self::Element,
        pseudo_or_element: Option<&Self::Element>,
        styled_element: Option<&Self::Element>,
    );

    fn NewCacheKey(&self, result: &Self::MatchResult, inherited_hash: u32) -> Self::CacheKey;
    fn CacheKeyIsCacheable(&self, key: &Self::CacheKey) -> bool;
    fn GetElementTypeCacheKey(
        &self,
        layout_parent: &Self::ComputedStyle,
        element: &Self::Element,
    ) -> Self::ElementTypeForCache;
    fn UnadjustedElementType(&self) -> Self::ElementTypeForCache;
    fn CacheEntryElementType<'e>(
        &self,
        entry: &'e Self::CacheEntry,
    ) -> &'e Self::ElementTypeForCache;
    fn ElementTypeIsStyleAdjusted(&self, element_type: &Self::ElementTypeForCache) -> bool;
    fn CacheEntryStyle(&self, entry: &Self::CacheEntry) -> Rc<Self::ComputedStyle>;
    fn FindMatchedProperties(
        &self,
        cache: &mut Self::MatchedPropertiesCache,
        key: &Self::CacheKey,
        element_type: &Self::ElementTypeForCache,
        style_type: PseudoId,
        state: &Self::StyleResolverState,
    ) -> Option<Rc<Self::CacheEntry>>;
    fn MatchedPropertiesIsCacheable(&self, state: &Self::StyleResolverState) -> bool;
    fn AddMatchedProperties(
        &self,
        cache: &mut Self::MatchedPropertiesCache,
        key: &Self::CacheKey,
        element_type: &Self::ElementTypeForCache,
        style: Rc<Self::ComputedStyle>,
        parent: Option<Rc<Self::ComputedStyle>>,
        layout_parent: Option<Rc<Self::ComputedStyle>>,
        originating: Option<Rc<Self::ComputedStyle>>,
    );

    fn WatchedSelectorsRuleSet(&self, engine: &Self::StyleEngine) -> Option<Rc<Self::RuleSet>>;
    fn DocumentRulesSelectorsRuleSet(
        &self,
        engine: &Self::StyleEngine,
    ) -> Option<Rc<Self::RuleSet>>;
    fn SetMatchingRulesFromNoStyleSheet(
        &self,
        collector: &mut Self::ElementRuleCollector,
        value: bool,
    );
    fn SetCollectingStyleRulesMode(&self, collector: &mut Self::ElementRuleCollector);
    fn CompactRulesIfNeeded(&self, rules: &Self::RuleSet);
    fn RuleSetGroupAddRuleSetBorrowed(&self, group: &mut Self::RuleSetGroup, rules: &Self::RuleSet);
    fn CollectMatchingRulesInContainer(
        &self,
        collector: &mut Self::ElementRuleCollector,
        group: &Self::RuleSetGroup,
        scope: Option<&Self::ContainerNode>,
    );
    fn MatchedStyleRuleList(
        &self,
        collector: &Self::ElementRuleCollector,
    ) -> Option<Vec<Rc<Self::StyleRule>>>;
    fn SelectorsText(&self, rule: &Self::StyleRule) -> foundation::String;
    fn AddCallbackSelector(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        selector: foundation::String,
    );
    fn AddDocumentRulesSelector(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        rule: &Rc<Self::StyleRule>,
    );

    fn ElementAnimations(&self, element: &Self::Element) -> Option<Rc<Self::ElementAnimations>>;
    fn ElementHasAnimations(&self, element: &Self::Element) -> bool;
    fn ElementAnimationsIsAnimationStyleChange(&self, animations: &Self::ElementAnimations)
        -> bool;
    fn AnimatingProperty(
        &self,
        animations: &Self::ElementAnimations,
        property: ResolverAnimationProperty,
    ) -> bool;
    fn IsAnimatingStandardPropertiesAtDefaultPriority(
        &self,
        animations: &Self::ElementAnimations,
        important: Option<&Self::CSSBitset>,
    ) -> bool;
    fn BuilderHasAnimations(&self, builder: &Self::ComputedStyleBuilder) -> bool;
    fn BuilderHasTransitions(&self, builder: &Self::ComputedStyleBuilder) -> bool;
    fn ScrollTimelineNameEmpty(&self, builder: &Self::ComputedStyleBuilder) -> bool;
    fn ViewTimelineNameEmpty(&self, builder: &Self::ComputedStyleBuilder) -> bool;
    fn TimelineScopeIsNone(&self, builder: &Self::ComputedStyleBuilder) -> bool;
    fn CSSAnimationsHasTimelines(&self, animations: &Self::ElementAnimations) -> bool;
    fn AnimationUpdateEmpty(&self, state: &Self::StyleResolverState) -> bool;
    fn HasPreviousActiveInterpolationsForAnimations(
        &self,
        animations: &Self::ElementAnimations,
    ) -> bool;
    fn CurrentAnimationData(&self) -> Option<Rc<Self::AnimationData>>;
    fn StoreOldStyleIfNeeded(&self, data: &Self::AnimationData, element: &Self::Element);
    fn SetPendingAnimationUpdate(
        &self,
        data: &Self::AnimationData,
        element: &Self::Element,
        state: &Self::StyleResolverState,
    );
    /// These CSSAnimations calls only unpack their exact source operands from
    /// State; StyleResolver keeps all eligibility, ordering and cascade logic.
    fn CalculateTimelineUpdate(
        &self,
        state: &mut Self::StyleResolverState,
        animating: &Self::Element,
    );
    fn CalculateAnimationUpdate(
        &self,
        state: &mut Self::StyleResolverState,
        animating: &Self::Element,
        originating: &Self::Element,
        resolver: &StyleResolver<'_, Self>,
        can_trigger: bool,
    );
    fn CalculateTransitionUpdate(
        &self,
        state: &mut Self::StyleResolverState,
        animating: &Self::Element,
        context: &Self::StyleRecalcContext,
        can_trigger: bool,
    );
    fn AnimationUpdateHasActiveInterpolations(&self, state: &Self::StyleResolverState) -> bool;
    fn ActiveInterpolationsForAnimations(
        &self,
        state: &Self::StyleResolverState,
    ) -> Rc<Self::ActiveInterpolationsMap>;
    fn ActiveInterpolationsForTransitions(
        &self,
        state: &Self::StyleResolverState,
    ) -> Rc<Self::ActiveInterpolationsMap>;
    fn IsHighlightPseudoElement(&self, pseudo_id: PseudoId) -> bool;
    fn CancelCompositedAnimationsAffectingProperties(
        &self,
        animations: &Self::ElementAnimations,
        important: &Self::CSSBitset,
    );
    fn CalculateCompositorAnimationUpdate(
        &self,
        state: &mut Self::StyleResolverState,
        animating: &Self::Element,
        originating: &Self::Element,
        base_style: &Self::ComputedStyle,
        parent: Option<&Self::ComputedStyle>,
        viewport_resized: bool,
        affects_snapshots: bool,
    );
    fn SnapshotCompositorKeyframes(
        &self,
        animating: &Self::Element,
        state: &Self::StyleResolverState,
        base_style: &Self::ComputedStyle,
        parent: Option<&Self::ComputedStyle>,
    );
    fn UpdateAnimationFlags(&self, animating: &Self::Element, state: &mut Self::StyleResolverState);
}

impl<'a, B: StyleResolverBaseStyleBackend> StyleResolver<'a, B> {
    // cpp: style_resolver.cc:149-152
    fn IsForPseudoElement(&self, element: &B::Element, request: &B::StyleRequest) -> bool {
        self.backend.IsPseudoElement(element) || self.backend.RequestIsPseudoStyle(request)
    }

    // cpp: style_resolver.cc:176-208
    fn ShouldStoreOldStyle(
        &self,
        context: &B::StyleRecalcContext,
        state: &B::StyleResolverState,
    ) -> bool {
        let b = self.backend;
        let builder = b.StateStyleBuilder(state);
        let element = b.StateElement(state);
        let anchored = b.RecalcFlag(context, ResolverRecalcFlag::SizeContainer)
            || b.RecalcFlag(context, ResolverRecalcFlag::AnchoredContainer)
            || b.BuilderFlag(builder, ResolverStyleFlag::AnchorFunctions)
            || b.BuilderPositionAnchorIsName(builder)
            || b.ImplicitAnchorElement(&element).is_some()
            || ((b.IsPseudoElement(&element) || b.StateIsForPseudoElement(state)) && {
                let parent = b
                    .StateParentStyle(state)
                    .expect("pseudo style requires a parent style");
                b.StyleFlag(&parent, ResolverStyleFlag::AnchorFunctions)
                    || b.StylePositionAnchorIsName(&parent)
            })
            || b.BuilderHasPositionTryFallbacks(builder);
        anchored && b.StateCanAffectAnimations(state)
    }

    // cpp: style_resolver.cc:210-224
    fn ShouldSetPendingUpdate(&self, state: &B::StyleResolverState, element: &B::Element) -> bool {
        if !self.backend.AnimationUpdateEmpty(state) {
            return true;
        }
        if let Some(animations) = self.backend.ElementAnimations(element) {
            return self
                .backend
                .HasPreviousActiveInterpolationsForAnimations(&animations);
        }
        false
    }

    // cpp: style_resolver.cc:226-245
    fn SetAnimationUpdateIfNeeded(
        &self,
        context: &B::StyleRecalcContext,
        state: &B::StyleResolverState,
        element: &B::Element,
    ) {
        if let Some(data) = self.backend.CurrentAnimationData() {
            if self.ShouldStoreOldStyle(context, state) {
                self.backend.StoreOldStyleIfNeeded(&data, element);
            }
        }
        if !self.ShouldSetPendingUpdate(state, element) {
            return;
        }
        if let Some(data) = self.backend.CurrentAnimationData() {
            self.backend
                .SetPendingAnimationUpdate(&data, element, state);
        }
    }

    // cpp: style_resolver.cc:247-252
    fn GetElementAnimations(
        &self,
        state: &B::StyleResolverState,
    ) -> Option<Rc<B::ElementAnimations>> {
        self.backend
            .StateAnimatingElement(state)
            .and_then(|element| self.backend.ElementAnimations(&element))
    }

    // cpp: style_resolver.cc:261-266
    fn HasAnimationsOrTransitions(&self, state: &B::StyleResolverState) -> bool {
        let builder = self.backend.StateStyleBuilder(state);
        self.backend.BuilderHasAnimations(builder)
            || self.backend.BuilderHasTransitions(builder)
            || self
                .backend
                .StateAnimatingElement(state)
                .is_some_and(|element| self.backend.ElementHasAnimations(&element))
    }

    // cpp: style_resolver.cc:268-282
    fn HasTimelines(&self, state: &B::StyleResolverState) -> bool {
        let b = self.backend;
        let builder = b.StateStyleBuilder(state);
        if !b.ScrollTimelineNameEmpty(builder) {
            return true;
        }
        if !b.ViewTimelineNameEmpty(builder) {
            return true;
        }
        if !b.TimelineScopeIsNone(builder) {
            return true;
        }
        if let Some(animations) = self.GetElementAnimations(state) {
            return b.CSSAnimationsHasTimelines(&animations);
        }
        false
    }

    // cpp: style_resolver.cc:284-289
    fn IsAnimationStyleChange(&self, element: &B::Element) -> bool {
        self.backend
            .ElementAnimations(element)
            .is_some_and(|animations| {
                self.backend
                    .ElementAnimationsIsAnimationStyleChange(&animations)
            })
    }

    // cpp: style_resolver.cc:429-507; use-counter-only branches omitted.
    fn ApplyLengthConversionFlags(&self, state: &mut B::StyleResolverState) {
        let b = self.backend;
        let flags = b.TakeLengthConversionFlags(state);
        if b.LengthFlagsEmpty(&flags) {
            return;
        }
        let builder = b.StateStyleBuilderMut(state);
        for (source, target) in [
            (ResolverLengthFlag::Em, ResolverStyleFlag::EmUnits),
            (
                ResolverLengthFlag::RootFontRelative,
                ResolverStyleFlag::RootRelativeUnits,
            ),
            (
                ResolverLengthFlag::GlyphRelative,
                ResolverStyleFlag::GlyphRelativeUnits,
            ),
        ] {
            if b.HasLengthFlag(&flags, source) {
                b.SetBuilderFlag(builder, target, true);
            }
        }
        if b.HasLengthFlag(&flags, ResolverLengthFlag::Viewport)
            || b.HasLengthFlag(&flags, ResolverLengthFlag::SmallLargeViewport)
        {
            b.SetBuilderFlag(builder, ResolverStyleFlag::StaticViewportUnits, true);
        }
        if b.HasLengthFlag(&flags, ResolverLengthFlag::DynamicViewport) {
            b.SetBuilderFlag(builder, ResolverStyleFlag::DynamicViewportUnits, true);
        }
        if b.HasLengthFlag(&flags, ResolverLengthFlag::ContainerRelative) {
            b.SetBuilderFlag(
                builder,
                ResolverStyleFlag::DependsOnSizeContainerQueries,
                true,
            );
            b.SetBuilderFlag(builder, ResolverStyleFlag::ContainerRelativeValue, true);
        }
        if b.HasLengthFlag(&flags, ResolverLengthFlag::TreeScopedReference) {
            b.StateSetTreeScopedReference(state);
        }
        let builder = b.StateStyleBuilderMut(state);
        for (source, target) in [
            (
                ResolverLengthFlag::AnchorRelative,
                ResolverStyleFlag::AnchorFunctions,
            ),
            (
                ResolverLengthFlag::LogicalDirectionRelative,
                ResolverStyleFlag::LogicalDirectionRelativeUnits,
            ),
            (
                ResolverLengthFlag::LhRelative,
                ResolverStyleFlag::LineHeightRelativeUnits,
            ),
        ] {
            if b.HasLengthFlag(&flags, source) {
                b.SetBuilderFlag(builder, target, true);
            }
        }
        if b.HasLengthFlag(&flags, ResolverLengthFlag::RlhRelative) {
            b.SetBuilderFlag(builder, ResolverStyleFlag::LineHeightRelativeUnits, true);
            b.SetBuilderFlag(builder, ResolverStyleFlag::RootRelativeUnits, true);
        }
        if b.HasLengthFlag(&flags, ResolverLengthFlag::SiblingRelative) {
            b.SetBuilderFlag(builder, ResolverStyleFlag::SiblingFunctions, true);
        }
        if b.HasLengthFlag(&flags, ResolverLengthFlag::ElementDependentRandom) {
            b.SetBuilderFlag(
                builder,
                ResolverStyleFlag::ElementDependentRandomFunctions,
                true,
            );
        }
    }

    // cpp: style_resolver.cc:509-567
    fn ApplyInertness(&self, state: &mut B::StyleResolverState) {
        let b = self.backend;
        let mut html_inert = None;
        let mut css_inert = None;
        let builder = b.StateStyleBuilder(state);
        if b.BuilderInteractivity(builder) == EInteractivity::kInert
            && !b.BuilderInteractivityIsInherited(builder)
        {
            css_inert = Some(true);
        }
        let element = b.StateElement(state);
        let document = b.ElementDocument(&element);
        let modal_element = b
            .ActiveModalDialog(document)
            .or_else(|| b.FullscreenElementFrom(document));
        if let Some(modal) = modal_element {
            if std::ptr::eq(modal.as_ref(), element.as_ref()) {
                if html_inert.is_none() {
                    html_inert = Some(false);
                }
                if css_inert.is_none() {
                    css_inert = Some(false);
                }
            } else if b
                .DocumentElement(document)
                .is_some_and(|root| std::ptr::eq(root.as_ref(), element.as_ref()))
            {
                html_inert = Some(true);
            }
        }
        if let Some(base_data) = b.BuilderBaseData(b.StateStyleBuilder(state)) {
            if b.StyleDisplay(
                &b.BaseComputedStyle(&base_data)
                    .expect("animation base data requires a style"),
            ) == EDisplay::kNone
            {
                html_inert = Some(true);
            }
        }
        let builder = b.StateStyleBuilderMut(state);
        if let Some(value) = html_inert {
            b.SetBuilderFlag(builder, ResolverStyleFlag::HTMLInert, value);
            b.SetBuilderFlag(builder, ResolverStyleFlag::HTMLInertIsInherited, false);
        }
        if let Some(value) = css_inert {
            b.SetBuilderFlag(builder, ResolverStyleFlag::CSSInert, value);
            b.SetBuilderFlag(builder, ResolverStyleFlag::CSSInertIsInherited, false);
        }
    }

    // cpp: style_resolver.cc:593-602
    fn DocumentElementUserAgentDeclarations(&self) -> Rc<B::CSSPropertyValueSet> {
        let b = self.backend;
        let decl = b
            .DeclarationCache()
            .document_element
            .get_or_init(|| b.NewMutableCSSPropertyValueSet(CSSParserMode::kHTMLStandardMode));
        if b.PropertyValueSetIsEmpty(decl) {
            b.SetProperty(
                decl,
                CSSPropertyID::kColor,
                &b.NewInitialColorValue(),
                false,
            );
        }
        Rc::clone(decl)
    }
    // cpp: style_resolver.cc:607-615
    fn ForcedColorsUserAgentDeclarations(&self) -> Rc<B::CSSPropertyValueSet> {
        let b = self.backend;
        let decl = b
            .DeclarationCache()
            .forced_colors
            .get_or_init(|| b.NewMutableCSSPropertyValueSet(CSSParserMode::kHTMLStandardMode));
        if b.PropertyValueSetIsEmpty(decl) {
            b.SetProperty(decl, CSSPropertyID::kColor, &b.NewInheritedValue(), false);
        }
        Rc::clone(decl)
    }
    // cpp: style_resolver.cc:618-629
    fn UniversalOverlayUserAgentDeclaration(&self) -> Rc<B::CSSPropertyValueSet> {
        let b = self.backend;
        let decl = b
            .DeclarationCache()
            .universal_overlay
            .get_or_init(|| b.NewMutableCSSPropertyValueSet(CSSParserMode::kHTMLStandardMode));
        if b.PropertyValueSetIsEmpty(decl) {
            b.SetProperty(
                decl,
                CSSPropertyID::kOverlay,
                &b.NewIdentifierValue(CSSValueID::kNone),
                true,
            );
        }
        Rc::clone(decl)
    }

    // cpp: style_resolver.cc:1334-1341
    fn GetBaseData(&self, state: &B::StyleResolverState) -> Option<Rc<B::StyleBaseData>> {
        let element = self.backend.StateAnimatingElement(state)?;
        let old_style = self.backend.ElementComputedStyle(&element)?;
        self.backend.StyleBaseData(&old_style)
    }
    // cpp: style_resolver.cc:1343-1349
    fn CachedAnimationBaseComputedStyle(
        &self,
        state: &B::StyleResolverState,
    ) -> Option<Rc<B::ComputedStyle>> {
        self.GetBaseData(state)
            .and_then(|data| self.backend.BaseComputedStyle(&data))
    }

    // cpp: style_resolver.cc:1378-1505; h:74-76
    // Debug/tracing/use-counter/statistics omitted.
    pub fn ResolveStyle(
        &mut self,
        element: Option<&B::Element>,
        context: &B::StyleRecalcContext,
        request: &B::StyleRequest,
    ) -> Option<Rc<B::ComputedStyle>> {
        let element = element?;
        let b = self.backend;
        let mut state =
            b.NewStyleResolverState(self.GetDocument(), element, Some(context), request);
        let mut cascade = b.NewStyleCascade(&mut state);
        self.ApplyBaseStyle(element, context, request, &mut state, &mut cascade);
        if b.RecalcFlag(context, ResolverRecalcFlag::EnsuringStyle) {
            b.SetBuilderFlag(
                b.StateStyleBuilderMut(&mut state),
                ResolverStyleFlag::EnsuredInDisplayNone,
                true,
            );
        }
        if self.IsForPseudoElement(element, request) && b.StateHadNoMatchedProperties(&state) {
            return Some(b.StateTakeStyle(&mut state));
        }
        if self.ApplyAnimatedStyle(&mut state, &mut cascade, context) {
            b.AdjustComputedStyle(
                &mut state,
                if self.IsForPseudoElement(element, request) {
                    None
                } else {
                    Some(element)
                },
            );
            self.RunUncacheableAdjustment(
                &mut state,
                element,
                self.IsForPseudoElement(element, request),
            );
        }
        self.ApplyAnchorData(&mut state);
        self.ApplyInertness(&mut state);
        if !self.IsForPseudoElement(element, request) {
            if b.IsBodyElement(element) {
                b.SetTextLinkTextColor(
                    self.GetDocument(),
                    b.BuilderCurrentColor(b.StateStyleBuilder(&state)),
                );
            }
            if b.IsMathMLElement(element) {
                self.ApplyMathMLCustomStyleProperties(element, &mut state);
            }
        }
        if let Some(animating) = b.StateAnimatingElement(&state) {
            self.SetAnimationUpdateIfNeeded(context, &state, &animating);
        }
        b.AddViewportUnitFlags(
            self.GetDocument(),
            b.BuilderViewportUnitFlags(b.StateStyleBuilder(&state)),
        );
        if b.BuilderFlag(
            b.StateStyleBuilder(&state),
            ResolverStyleFlag::RootRelativeUnits,
        ) {
            b.SetUsesRootRelativeUnits(b.GetStyleEngine(self.GetDocument()), true);
        }
        if b.BuilderFlag(
            b.StateStyleBuilder(&state),
            ResolverStyleFlag::GlyphRelativeUnits,
        ) {
            b.SetUsesGlyphRelativeUnits(b.GetStyleEngine(self.GetDocument()), true);
        }
        if b.BuilderFlag(
            b.StateStyleBuilder(&state),
            ResolverStyleFlag::LineHeightRelativeUnits,
        ) {
            b.SetUsesLineHeightUnits(b.GetStyleEngine(self.GetDocument()), true);
        }
        if b.BuilderFlag(
            b.StateStyleBuilder(&state),
            ResolverStyleFlag::SiblingFunctions,
        ) {
            let tree_counting_element = b.StateElement(&state);
            let tree_counting_element =
                self.UltimateOriginatingElementOrSelf(&tree_counting_element);
            if let Some(parent) = b.ParentElementOrDocumentFragment(tree_counting_element) {
                b.MarkChildrenAffectedByForwardPositionalRules(&parent);
                b.MarkChildrenAffectedByBackwardPositionalRules(&parent);
                b.SetUsesTreeCountingFunctions(b.GetStyleEngine(self.GetDocument()));
            }
        }
        b.StateLoadPendingResources(&mut state);
        Some(b.StateTakeStyle(&mut state))
    }

    // cpp: style_resolver.cc:1507-1521; h:83-87
    pub fn ResolveBaseStyle(
        &mut self,
        element: &B::Element,
        parent: Option<&B::ComputedStyle>,
        layout_parent: Option<&B::ComputedStyle>,
        context: &B::StyleRecalcContext,
    ) -> Rc<B::ComputedStyle> {
        let b = self.backend;
        let mut request = b.NewStyleRequest();
        b.SetRequestParentOverride(&mut request, parent);
        b.SetRequestLayoutParentOverride(&mut request, layout_parent);
        let mut state =
            b.NewStyleResolverState(self.GetDocument(), element, Some(context), &request);
        let mut cascade = b.NewStyleCascade(&mut state);
        self.ApplyBaseStyleNoCache(element, context, &request, &mut state, &mut cascade);
        b.StateTakeStyle(&mut state)
    }

    // cpp: style_resolver.cc:1523-1530
    fn FindStyleType(&self, element: &B::Element, request: &B::StyleRequest) -> PseudoId {
        if self.backend.IsPseudoElement(element) {
            self.backend.ElementPseudoIdForStyling(element)
        } else {
            self.backend.RequestPseudoId(request)
        }
    }
    // cpp: style_resolver.cc:1532-1564; h:310-315
    fn InitStyle(
        &self,
        element: &B::Element,
        request: &B::StyleRequest,
        source: &B::ComputedStyle,
        parent: Option<&B::ComputedStyle>,
        originating: Option<&B::ComputedStyle>,
        state: &mut B::StyleResolverState,
    ) {
        let b = self.backend;
        let parent = parent.expect("initial style setup requires a parent");
        if b.StateIsForHighlight(state) {
            b.StateCreateNewClonedStyle(state, parent);
            b.CopyHighlightPropertiesFrom(
                b.StateStyleBuilderMut(state),
                originating.expect("highlight style requires an originating element style"),
            );
        } else {
            b.StateCreateNewStyle(
                state,
                source,
                parent,
                !self.IsForPseudoElement(element, request) && b.IsAtShadowBoundary(element),
            );
        }
        b.SetStyleType(
            b.StateStyleBuilderMut(state),
            self.FindStyleType(element, request),
        );
        if !b.RequestIsPseudoStyle(request) && b.IsLink(element) {
            b.SetBuilderFlag(
                b.StateStyleBuilderMut(state),
                ResolverStyleFlag::IsLink,
                true,
            );
        }
    }

    // cpp: style_resolver.cc:1566-1595; h:346
    fn ApplyMathMLCustomStyleProperties(
        &self,
        element: &B::Element,
        state: &mut B::StyleResolverState,
    ) {
        use ResolverMathMLProperty::*;
        let properties: &[ResolverMathMLProperty] = match self.backend.MathMLKind(element) {
            ResolverMathMLKind::Space => &[Baseline],
            ResolverMathMLKind::Padded => &[Baseline, PaddedDepth, PaddedLSpace, PaddedVOffset],
            ResolverMathMLKind::Fraction => &[FractionBarThickness],
            ResolverMathMLKind::Operator => &[LSpace, RSpace, MinSize, MaxSize],
            ResolverMathMLKind::Other => &[],
        };
        for property in properties {
            self.backend
                .AddMathPropertyIfNeeded(element, state, *property);
        }
    }

    // cpp: style_resolver.cc:1597-1704
    fn CanApplyInlineStyleIncrementally(
        &self,
        element: &B::Element,
        state: &B::StyleResolverState,
        request: &B::StyleRequest,
    ) -> bool {
        let b = self.backend;
        if !b.IsInlineIndependentStyleChange(element) {
            return false;
        }
        let Some(old_style) = b.ElementComputedStyle(element) else {
            return false;
        };
        if self.IsForPseudoElement(element, request) {
            return false;
        }
        if b.StyleInsideLink(&old_style) != EInsideLink::kNotInsideLink {
            return false;
        }
        if b.StyleFlag(&old_style, ResolverStyleFlag::InlineStyleLostCascade) {
            return false;
        }
        if b.HasCustomStyleCallbacks(element) {
            return false;
        }
        if b.StateParentStyle(state).is_none() {
            return false;
        }
        if self.GetElementAnimations(state).is_some() || b.StyleBaseData(&old_style).is_some() {
            return false;
        }
        if b.StyleFlag(&old_style, ResolverStyleFlag::AnchorEvaluator) {
            return false;
        }
        if let Some(inline_style) = b.InlineStyle(element) {
            for property in b.PropertySetProperties(&inline_style) {
                if !CSSProperty::Get(b.PropertyID(property)).IsIdempotent() {
                    return false;
                }
                let value = b.PropertyValue(property);
                if b.ValueIsUnparsedDeclaration(value)
                    || b.ValueIsPendingSubstitution(value)
                    || b.ValueIsCascadeDependentKeyword(value)
                {
                    return false;
                }
                if b.ValueIsMathFunction(value) && b.MathValueHasAnchorFunctions(value) {
                    return false;
                }
            }
        }
        true
    }

    // Repeated StyleAdjuster argument selection in cc:1421-1425,1868-1871,
    // 2045-2049. Selection stays here rather than in the external adjuster.
    fn RunUncacheableAdjustment(
        &self,
        state: &mut B::StyleResolverState,
        element: &B::Element,
        is_pseudo: bool,
    ) {
        let b = self.backend;
        let pseudo = if is_pseudo {
            b.StatePseudoElement(state)
        } else {
            None
        };
        let styled = b.StateStyledElement(state);
        b.RunUncacheableStyleAdjustment(
            state,
            element,
            if is_pseudo {
                pseudo.as_deref()
            } else {
                Some(element)
            },
            styled.as_deref(),
        );
    }

    // cpp: style_resolver.cc:1736-1944; h:322-326
    fn ApplyBaseStyleNoCache(
        &mut self,
        element: &B::Element,
        context: &B::StyleRecalcContext,
        request: &B::StyleRequest,
        state: &mut B::StyleResolverState,
        cascade: &mut B::StyleCascade,
    ) {
        let b = self.backend;
        b.EnsureUAStyleForElement(b.GetStyleEngine(self.GetDocument()), element);
        if !b.RequestIsPseudoStyle(request) {
            if self.IsForcedColorsModeEnabled() {
                b.AddCascadeMatchedProperties(
                    cascade,
                    &self.ForcedColorsUserAgentDeclarations(),
                    CascadeOrigin::kUserAgent,
                );
            }
            if b.RuntimeFeatureEnabled(ResolverRuntimeFeature::OverlayProperty)
                && !b.RuntimeFeatureEnabled(ResolverRuntimeFeature::OverlayGlobalRuleRemoval)
            {
                b.AddCascadeMatchedProperties(
                    cascade,
                    &self.UniversalOverlayUserAgentDeclaration(),
                    CascadeOrigin::kUserAgent,
                );
            }
            if b.DocumentElement(self.GetDocument())
                .is_some_and(|root| std::ptr::eq(root.as_ref(), element))
            {
                b.AddCascadeMatchedProperties(
                    cascade,
                    &self.DocumentElementUserAgentDeclarations(),
                    CascadeOrigin::kUserAgent,
                );
            }
        }
        let mut collector =
            b.NewCollectorForCascade(state, context, &mut self.selector_filter_, cascade);
        if b.IsPseudoElement(element) {
            b.EnsureUAStyleForPseudoElement(
                b.GetStyleEngine(self.GetDocument()),
                b.ElementPseudoIdForStyling(element),
            );
        } else if b.RequestIsPseudoStyle(request) {
            b.SetPseudoElementStyleRequest(&mut collector, request);
            b.EnsureUAStyleForPseudoElement(
                b.GetStyleEngine(self.GetDocument()),
                b.RequestPseudoId(request),
            );
        }
        if b.StateParentStyle(state).is_none() {
            b.StateSetParentStyle(state, Rc::new(self.InitialStyleForElement()));
            let parent = b
                .StateParentStyle(state)
                .expect("parent style was initialized above");
            b.StateSetLayoutParentStyle(state, parent);
        }
        if b.RequestUAOnly(request) {
            self.MatchUARules(element, &mut collector);
        } else {
            self.MatchAllRules(state, &mut collector, !b.RequestExcludesSMIL(request));
        }
        let match_result = b.CascadeMatchResult(cascade);
        if self.IsForPseudoElement(element, request)
            && !b.MatchResultHasMatchedProperties(&match_result)
        {
            b.StateSetHadNoMatchedProperties(state);
        }
        let (cache_success, element_type_for_cache) =
            self.ApplyMatchedCache(state, request, &match_result);
        let element_if_not_pseudo = if self.IsForPseudoElement(element, request) {
            None
        } else {
            Some(element)
        };
        if cache_success.IsHit() {
            if !cache_success.IsStyleAdjusted(b) {
                b.AdjustComputedStyle(state, element_if_not_pseudo);
            }
        } else {
            self.ApplyPropertiesFromCascade(state, cascade);
            let display = b.BuilderDisplay(b.StateStyleBuilder(state));
            b.SetBuilderFlag(
                b.StateStyleBuilderMut(state),
                ResolverStyleFlag::OriginalDisplayInlineType,
                b.IsDisplayInlineType(display),
            );
            if b.ElementTypeIsStyleAdjusted(&element_type_for_cache) {
                b.AdjustComputedStyle(state, element_if_not_pseudo);
                self.MaybeAddToMatchedPropertiesCache(
                    state,
                    &cache_success.key,
                    &element_type_for_cache,
                );
            } else {
                self.MaybeAddToMatchedPropertiesCache(
                    state,
                    &cache_success.key,
                    &b.UnadjustedElementType(),
                );
                b.AdjustComputedStyle(state, element_if_not_pseudo);
            }
        }
        let is_for_pseudo_element = b.StateIsForPseudoElement(state);
        self.RunUncacheableAdjustment(state, element, is_for_pseudo_element);
        for (source, target) in [
            (
                ResolverMatchResultFlag::NonUniversalHighlightPseudoStyles,
                ResolverStyleFlag::NonUniversalHighlightPseudoStyles,
            ),
            (
                ResolverMatchResultFlag::NonUaHighlightPseudoStyles,
                ResolverStyleFlag::NonUaHighlightPseudoStyles,
            ),
            (
                ResolverMatchResultFlag::HighlightsDependOnSizeContainerQueries,
                ResolverStyleFlag::HighlightsDependOnSizeContainerQueries,
            ),
        ] {
            b.SetBuilderFlag(
                b.StateStyleBuilderMut(state),
                target,
                b.MatchResultFlag(&match_result, source),
            );
        }
        for (source, target) in [
            (
                MatchFlag::kAffectedByDrag,
                ResolverStyleFlag::AffectedByDrag,
            ),
            (
                MatchFlag::kAffectedByFocusWithin,
                ResolverStyleFlag::AffectedByFocusWithin,
            ),
            (
                MatchFlag::kAffectedByHover,
                ResolverStyleFlag::AffectedByHover,
            ),
            (
                MatchFlag::kAffectedByActive,
                ResolverStyleFlag::AffectedByActive,
            ),
            (
                MatchFlag::kAffectedByStartingStyle,
                ResolverStyleFlag::StartingStyle,
            ),
        ] {
            if b.MatchResultHasFlag(&match_result, source) {
                b.SetBuilderFlag(b.StateStyleBuilderMut(state), target, true);
            }
        }
        for (source, target) in [
            (
                ResolverMatchResultFlag::SizeContainerQueries,
                ResolverStyleFlag::DependsOnSizeContainerQueries,
            ),
            (
                ResolverMatchResultFlag::StyleContainerQueries,
                ResolverStyleFlag::DependsOnStyleContainerQueries,
            ),
            (
                ResolverMatchResultFlag::ScrollStateContainerQueries,
                ResolverStyleFlag::DependsOnScrollStateContainerQueries,
            ),
            (
                ResolverMatchResultFlag::AnchoredContainerQueries,
                ResolverStyleFlag::DependsOnAnchoredContainerQueries,
            ),
            (
                ResolverMatchResultFlag::FirstLineSizeContainerQueries,
                ResolverStyleFlag::FirstLineDependsOnSizeContainerQueries,
            ),
            (
                ResolverMatchResultFlag::StaticViewportUnits,
                ResolverStyleFlag::StaticViewportUnits,
            ),
            (
                ResolverMatchResultFlag::DynamicViewportUnits,
                ResolverStyleFlag::DynamicViewportUnits,
            ),
            (
                ResolverMatchResultFlag::RootUnitContainerQueries,
                ResolverStyleFlag::RootRelativeUnits,
            ),
        ] {
            if b.MatchResultFlag(&match_result, source) {
                b.SetBuilderFlag(b.StateStyleBuilderMut(state), target, true);
            }
        }
        if b.MatchResultFlag(
            &match_result,
            ResolverMatchResultFlag::ConditionallyAffectsAnimations,
        ) {
            b.StateSetConditionallyAffectsAnimations(state);
        }
        let names = b.CustomHighlightNames(&match_result);
        if !b.CustomHighlightNamesEmpty(names) {
            b.SetCustomHighlightNames(b.StateStyleBuilderMut(state), names);
        }
        b.SetPseudoElementStyles(
            b.StateStyleBuilderMut(state),
            b.MatchResultPseudoElementBits(&match_result),
        );
        let inside_link = b.StateInsideLink(state);
        b.SetInsideLink(b.StateStyleBuilderMut(state), inside_link);
        self.ApplyCallbackSelectors(state);
        if b.IsLink(element) && b.IsAnchorOrAreaElement(element) {
            self.ApplyDocumentRulesSelectors(state, Some(&b.ElementTreeRoot(element)));
        }
        self.ApplyAnchorData(state);
    }

    // cpp: style_resolver.cc:1958-2095; h:317-321
    // Recompute-and-diff debug verification and statistics are omitted.
    fn ApplyBaseStyle(
        &mut self,
        element: &B::Element,
        context: &B::StyleRecalcContext,
        request: &B::StyleRequest,
        state: &mut B::StyleResolverState,
        cascade: &mut B::StyleCascade,
    ) {
        let b = self.backend;
        if b.StateCanTriggerAnimations(state) && self.CanReuseBaseComputedStyle(state) {
            let base = self
                .CachedAnimationBaseComputedStyle(state)
                .expect("reuse eligibility requires cached animation base style");
            b.StateCreateNewClonedStyle(state, &base);
            let base_data = self.GetBaseData(state);
            b.SetBaseData(b.StateStyleBuilderMut(state), base_data);
            b.SetStyleType(
                b.StateStyleBuilderMut(state),
                if b.IsPseudoElement(element) {
                    b.ElementPseudoIdForStyling(element)
                } else {
                    b.RequestPseudoId(request)
                },
            );
            if b.StateParentStyle(state).is_none() {
                b.StateSetParentStyle(state, Rc::new(self.InitialStyleForElement()));
                let parent = b
                    .StateParentStyle(state)
                    .expect("parent style was initialized above");
                b.StateSetLayoutParentStyle(state, parent);
            }
            return;
        }
        if b.RecalcFlag(context, ResolverRecalcFlag::CanUseIncrementalStyle)
            && self.CanApplyInlineStyleIncrementally(element, state, request)
        {
            b.StateCreateNewClonedStyle(
                state,
                &b.ElementComputedStyle(element)
                    .expect("incremental eligibility requires a computed style"),
            );
            b.SetBuilderFlag(
                b.StateStyleBuilderMut(state),
                ResolverStyleFlag::SkipsContents,
                false,
            );
            let mut author_flags = 0u64;
            if let Some(inline_style) = b.InlineStyle(element) {
                for property in b.PropertySetProperties(&inline_style) {
                    b.ApplyScopedProperty(
                        b.PropertyName(property),
                        state,
                        b.PropertyValue(property),
                        b.ElementTreeScope(element),
                    );
                    author_flags |= CSSProperty::Get(b.PropertyID(property)).GetFlags();
                }
            }
            b.StateSetComputedStyleFlagsFromAuthorFlags(state, author_flags);
            self.ApplyLengthConversionFlags(state);
            b.AdjustComputedStyle(
                state,
                if self.IsForPseudoElement(element, request) {
                    None
                } else {
                    Some(element)
                },
            );
            self.RunUncacheableAdjustment(
                state,
                element,
                self.IsForPseudoElement(element, request),
            );
            b.StateLoadPendingResources(state);
            self.ApplyAnchorData(state);
            return;
        }
        self.ApplyBaseStyleNoCache(element, context, request, state, cascade);
    }

    // cpp: style_resolver.cc:2572-2675; h:379-381; debug/statistics omitted.
    fn ApplyAnimatedStyle(
        &self,
        state: &mut B::StyleResolverState,
        cascade: &mut B::StyleCascade,
        context: &B::StyleRecalcContext,
    ) -> bool {
        let b = self.backend;
        let element = b.StateElement(state);
        let element = b.RetainElement(self.UltimateOriginatingElementOrSelf(&element));
        let Some(animating) = b.StateAnimatingElement(state) else {
            return false;
        };
        if self.HasTimelines(state) {
            b.CalculateTimelineUpdate(state, &animating);
        }
        if !self.HasAnimationsOrTransitions(state) {
            return false;
        }
        if !self.IsAnimationStyleChange(&animating)
            || b.BuilderBaseData(b.StateStyleBuilder(state)).is_none()
        {
            let base = b.NewStyleBaseData(
                b.BuilderCloneStyle(b.StateStyleBuilder(state)),
                b.CascadeReleaseImportantSet(cascade),
            );
            b.SetBaseData(b.StateStyleBuilderMut(state), Some(base));
        }
        let can_trigger = b.StateCanTriggerAnimations(state);
        b.CalculateAnimationUpdate(state, &animating, &element, self, can_trigger);
        b.CalculateTransitionUpdate(state, &animating, context, can_trigger);
        let apply = b.AnimationUpdateHasActiveInterpolations(state);
        if apply {
            let animations = b.ActiveInterpolationsForAnimations(state);
            let transitions = b.ActiveInterpolationsForTransitions(state);
            b.CascadeAddInterpolations(cascade, &animations, CascadeOrigin::kAnimation);
            b.CascadeAddInterpolations(cascade, &transitions, CascadeOrigin::kTransition);
            let originating = b.StateElement(state);
            let mut filter =
                b.GetCascadeFilter(self.UltimateOriginatingElementOrSelf(&originating));
            let style_type = b.BuilderStyleType(b.StateStyleBuilder(state));
            if style_type == PseudoId::kPseudoIdMarker {
                filter = filter.Add(CSSProperty::kValidForMarker);
            }
            if b.IsHighlightPseudoElement(style_type) {
                filter = filter.Add(CSSProperty::kValidForHighlight);
            }
            filter = filter.Add(CSSProperty::kNotAnimation);
            b.CascadeApply(cascade, filter);
            b.StateLoadPendingResources(state);
            self.ApplyLengthConversionFlags(state);
        }
        if let Some(animations) = b.ElementAnimations(&animating) {
            if let Some(data) = b.BuilderBaseData(b.StateStyleBuilder(state)) {
                if let Some(important) = b.BaseImportantSet(&data) {
                    b.CancelCompositedAnimationsAffectingProperties(&animations, &important);
                }
            }
        }
        let base = b.BuilderBaseComputedStyle(b.StateStyleBuilder(state));
        let parent = b.StateParentStyle(state);
        let affects = b.StateAffectsCompositorSnapshots(state);
        b.CalculateCompositorAnimationUpdate(
            state,
            &animating,
            &element,
            &base,
            parent.as_deref(),
            self.WasViewportResized(),
            affects,
        );
        // Fetch styles again after CalculateCompositorAnimationUpdate as in C++.
        let base = b.BuilderBaseComputedStyle(b.StateStyleBuilder(state));
        let parent = b.StateParentStyle(state);
        b.SnapshotCompositorKeyframes(&animating, state, &base, parent.as_deref());
        b.UpdateAnimationFlags(&animating, state);
        apply
    }

    // cpp: style_resolver.cc:2677-2692; h:382
    fn ApplyAnchorData(&self, state: &mut B::StyleResolverState) {
        let b = self.backend;
        if let Some(evaluator) = b.StateAnchorEvaluator(state) {
            if let Some(offset) =
                b.ComputeAnchorCenterOffsets(&evaluator, b.StateStyleBuilder(state))
            {
                b.SetAnchorCenterOffset(b.StateStyleBuilderMut(state), Some(offset));
            }
            b.SetBuilderFlag(
                b.StateStyleBuilderMut(state),
                ResolverStyleFlag::AnchorEvaluator,
                true,
            );
        }
    }

    // cpp: style_resolver.cc:2771-2896; h:368-371
    fn ApplyMatchedCache(
        &mut self,
        state: &mut B::StyleResolverState,
        request: &B::StyleRequest,
        result: &B::MatchResult,
    ) -> (
        CacheSuccess<B::CacheKey, B::CacheEntry>,
        B::ElementTypeForCache,
    ) {
        let b = self.backend;
        let element = b.StateElement(state);
        let parent = b
            .StateParentStyle(state)
            .expect("MPC lookup requires parent style");
        let layout_parent = b
            .StateLayoutParentStyle(state)
            .expect("MPC lookup requires layout parent style");
        let originating = b.StateOriginatingElementStyle(state);
        let mut inherited_hash = b.StyleInheritedVariablesHash(if b.StateIsForHighlight(state) {
            originating
                .as_deref()
                .expect("highlight requires originating style")
        } else {
            &parent
        });
        inherited_hash ^= b.StyleInheritedBitFieldsHash(&parent);
        inherited_hash ^= HashFloat(b.StyleFontComputedSize(&parent));
        inherited_hash ^= HashInt(b.StyleDisplay(&layout_parent) as u32);
        let key = b.NewCacheKey(result, inherited_hash);
        let mut can_use_cache = b.MatchResultIsCacheable(result);
        if !b.GetCascadeFilter(&element).IsEmpty() {
            can_use_cache = false;
        }
        let element_type = b.GetElementTypeCacheKey(&layout_parent, &element);
        let style_type = self.FindStyleType(&element, request);
        let entry = if can_use_cache {
            b.FindMatchedProperties(
                &mut self.matched_properties_cache_,
                &key,
                &element_type,
                style_type,
                state,
            )
        } else {
            None
        };
        if let Some(entry) = &entry {
            let style_to_clone = b.CacheEntryStyle(entry);
            self.InitStyle(
                &element,
                request,
                &style_to_clone,
                Some(&style_to_clone),
                Some(&style_to_clone),
                state,
            );
            if b.StyleFlag(&style_to_clone, ResolverStyleFlag::CanAffectAnimations) {
                b.SetBuilderFlag(
                    b.StateStyleBuilderMut(state),
                    ResolverStyleFlag::CanAffectAnimations,
                    true,
                );
            }
            b.SetBuilderFlag(
                b.StateStyleBuilderMut(state),
                ResolverStyleFlag::OriginalDisplayInlineType,
                b.StyleFlag(
                    &style_to_clone,
                    ResolverStyleFlag::OriginalDisplayInlineType,
                ),
            );
            if b.BuilderFlag(
                b.StateStyleBuilder(state),
                ResolverStyleFlag::ExplicitInheritance,
            ) {
                b.StyleSetChildHasExplicitInheritance(&parent);
            }
            b.StateUpdateFont(state);
        } else {
            self.InitStyle(
                &element,
                request,
                self.InitialStyle(),
                Some(&parent),
                originating.as_deref(),
                state,
            );
            if !b.StateIsForHighlight(state) && b.StyleEffectiveZoom(&parent) != self.InitialZoom()
            {
                self.SetZoomedInitialLineWidths(
                    b.StyleEffectiveZoom(&parent),
                    b.StateStyleBuilderMut(state),
                );
            }
            self.ExpandInheritedVisitedProperties(state);
            if !self.IsForPseudoElement(&element, request) && b.AssignedSlot(&element).is_some() {
                if let Some(parent_element) = b.ParentElement(&element) {
                    if !b.RuntimeFeatureEnabled(
                        ResolverRuntimeFeature::InheritUserModifyWithoutContenteditable,
                    ) || !b.HasContenteditableAttribute(&element)
                    {
                        if let Some(host_style) = b.ElementComputedStyle(&parent_element) {
                            b.SetUserModify(
                                b.StateStyleBuilderMut(state),
                                b.StyleUserModify(&host_style),
                            );
                        }
                    }
                }
            }
        }
        (
            CacheSuccess {
                key,
                cached_matched_properties: entry,
            },
            element_type,
        )
    }

    // cpp: style_resolver.cc:2898-2912; h:372-375
    fn MaybeAddToMatchedPropertiesCache(
        &mut self,
        state: &mut B::StyleResolverState,
        key: &B::CacheKey,
        element_type: &B::ElementTypeForCache,
    ) {
        let b = self.backend;
        b.StateLoadPendingResources(state);
        if b.CacheKeyIsCacheable(key) && b.MatchedPropertiesIsCacheable(state) {
            b.AddMatchedProperties(
                &mut self.matched_properties_cache_,
                key,
                element_type,
                b.BuilderCloneStyle(b.StateStyleBuilder(state)),
                b.StateParentStyle(state),
                b.StateLayoutParentStyle(state),
                if b.StateIsForHighlight(state) {
                    b.StateOriginatingElementStyle(state)
                } else {
                    None
                },
            );
        }
    }

    // cpp: style_resolver.cc:2914-3001; h:226
    pub fn CanReuseBaseComputedStyle(&self, state: &B::StyleResolverState) -> bool {
        let b = self.backend;
        let Some(animations) = self.GetElementAnimations(state) else {
            return false;
        };
        if !b.ElementAnimationsIsAnimationStyleChange(&animations) {
            return false;
        }
        let Some(data) = self.GetBaseData(state) else {
            return false;
        };
        let Some(base) = b.BaseComputedStyle(&data) else {
            return false;
        };
        if b.AnimatingProperty(&animations, ResolverAnimationProperty::CustomProperties) {
            return false;
        }
        if b.AnimatingProperty(&animations, ResolverAnimationProperty::Revert) {
            return false;
        }
        if b.AnimatingProperty(&animations, ResolverAnimationProperty::FontAffecting)
            && b.StyleFlag(&base, ResolverStyleFlag::FontRelativeUnits)
        {
            return false;
        }
        if b.AnimatingProperty(&animations, ResolverAnimationProperty::LineHeight)
            && b.StyleFlag(&base, ResolverStyleFlag::LineHeightRelativeUnits)
        {
            return false;
        }
        if b.RuntimeFeatureEnabled(ResolverRuntimeFeature::CSSZoomAnimation)
            && b.AnimatingProperty(&animations, ResolverAnimationProperty::Zoom)
        {
            return false;
        }
        let important = b.BaseImportantSet(&data);
        if b.IsAnimatingStandardPropertiesAtDefaultPriority(&animations, important.as_deref()) {
            return false;
        }
        if b.StyleHasPositionTryFallbacks(&base) {
            return false;
        }
        if b.StyleFlag(&base, ResolverStyleFlag::AnchorFunctions)
            || b.StyleFlag(&base, ResolverStyleFlag::AnchorEvaluator)
        {
            return false;
        }
        if b.StyleDisplay(&base) == EDisplay::kNone
            && b.AnimatingProperty(&animations, ResolverAnimationProperty::Display)
        {
            return false;
        }
        true
    }

    // cpp: style_resolver.cc:3181-3226; h:377; byte/use-count metrics omitted.
    fn ApplyPropertiesFromCascade(
        &self,
        state: &mut B::StyleResolverState,
        cascade: &mut B::StyleCascade,
    ) {
        let b = self.backend;
        let filter = b.GetCascadeFilter(&b.StateElement(state));
        b.CascadeApply(cascade, filter.Add(CSSProperty::kNotLegacyOverlapping));
        if b.StateRejectedLegacyOverlapping(state) {
            b.CascadeApply(cascade, filter.Add(CSSProperty::kOverlapping));
        }
        b.SetBuilderFlag(
            b.StateStyleBuilderMut(state),
            ResolverStyleFlag::InlineStyleLostCascade,
            b.CascadeInlineStyleLost(cascade),
        );
        self.ApplyLengthConversionFlags(state);
    }

    // cpp: style_resolver.cc:3228-3238; h:384
    fn ApplyCallbackSelectors(&mut self, state: &mut B::StyleResolverState) {
        let b = self.backend;
        let rule_set = b.WatchedSelectorsRuleSet(b.GetStyleEngine(self.GetDocument()));
        let Some(rules) =
            self.CollectMatchingRulesFromUnconnectedRuleSet(state, rule_set.as_deref(), None)
        else {
            return;
        };
        for rule in rules {
            b.AddCallbackSelector(b.StateStyleBuilderMut(state), b.SelectorsText(&rule));
        }
    }
    // cpp: style_resolver.cc:3240-3251; h:385
    fn ApplyDocumentRulesSelectors(
        &mut self,
        state: &mut B::StyleResolverState,
        scope: Option<&B::ContainerNode>,
    ) {
        let b = self.backend;
        let rule_set = b.DocumentRulesSelectorsRuleSet(b.GetStyleEngine(self.GetDocument()));
        let Some(rules) =
            self.CollectMatchingRulesFromUnconnectedRuleSet(state, rule_set.as_deref(), scope)
        else {
            return;
        };
        for rule in rules {
            b.AddDocumentRulesSelector(b.StateStyleBuilderMut(state), &rule);
        }
    }
    // cpp: style_resolver.cc:3253-3277; h:386-389
    fn CollectMatchingRulesFromUnconnectedRuleSet(
        &mut self,
        state: &B::StyleResolverState,
        rule_set: Option<&B::RuleSet>,
        scope: Option<&B::ContainerNode>,
    ) -> Option<Vec<Rc<B::StyleRule>>> {
        let rules = rule_set?;
        let b = self.backend;
        let mut result = b.NewMatchResult();
        let mut collector = b.NewCollectorForResult(
            state,
            &b.NewStyleRecalcContext(),
            &mut self.selector_filter_,
            &mut result,
        );
        b.SetMatchingRulesFromNoStyleSheet(&mut collector, true);
        b.SetCollectingStyleRulesMode(&mut collector);
        b.CompactRulesIfNeeded(rules);
        let mut group = b.NewRuleSetGroup(0);
        // RuleSetGroup retains the shared rule set; this lookup does not clone
        // any rule content. The mandatory handle operation is on the sheet.
        b.RuleSetGroupAddRuleSetBorrowed(&mut group, rules);
        b.CollectMatchingRulesInContainer(&mut collector, &group, scope);
        b.SortAndTransferMatchedRules(&mut collector, CascadeOrigin::kAuthor, false, self.tracker_);
        b.SetMatchingRulesFromNoStyleSheet(&mut collector, false);
        b.MatchedStyleRuleList(&collector)
    }

    // cpp: style_resolver.cc:3348-3367; h:395
    fn ExpandInheritedVisitedProperties(&self, state: &mut B::StyleResolverState) {
        let b = self.backend;
        if b.StateParentStyle(state)
            .is_some_and(|parent| b.StyleInsideLink(&parent) == EInsideLink::kNotInsideLink)
            && b.StateInsideLink(state) == EInsideLink::kInsideVisitedLink
        {
            for property in [
                ResolverVisitedProperty::Color,
                ResolverVisitedProperty::CaretColor,
                ResolverVisitedProperty::FillPaint,
                ResolverVisitedProperty::StrokePaint,
                ResolverVisitedProperty::TextEmphasisColor,
                ResolverVisitedProperty::TextFillColor,
                ResolverVisitedProperty::TextStrokeColor,
            ] {
                b.SetVisitedPropertyFromUnvisited(b.StateStyleBuilderMut(state), property);
            }
        }
    }
}

// Required operations of page/layout, DOM, CSS values and fonts. These methods
// are individual external-class operations; StyleResolver owns their ordering,
// iteration, eligibility and fallback decisions below.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageMarginSlot {
    TopLeft,
    TopCenter,
    TopRight,
    RightTop,
    RightMiddle,
    RightBottom,
    BottomLeft,
    BottomCenter,
    BottomRight,
    LeftTop,
    LeftMiddle,
    LeftBottom,
    TopLeftCorner,
    TopRightCorner,
    BottomRightCorner,
    BottomLeftCorner,
}
#[derive(Clone, Copy, Debug)]
pub struct ResolverPrintMargins {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
    pub ignore_css_margins: bool,
}
// cpp: style_resolver.h:175-183. None represents the source's null pointers.
pub struct FindKeyframesRuleResult<'t, R, T> {
    pub rule: Option<Rc<R>>,
    pub tree_scope: Option<&'t T>,
}
pub trait StyleResolverCompleteBackend: StyleResolverBaseStyleBackend {
    type Text;
    type RuleIndexList;
    type CascadedValues;
    type ContainerSelector;
    type StyleRuleKeyframes;
    type StyleRulePositionTry;
    type PropertyHandle;
    type CompositorKeyframeValue;
    type Font;
    type FontSelector;
    type FilterOperations;
    type AppliedTextDecorationData;
    type LayoutObject;
    type LayoutView;
    type PrintingPhysicalSize;
    type PageRuleCollector;
    type PageMarginsStyle;
    type ViewTransitionClassList;
    type FillLayers: PartialEq;
    type ScrollSnapType: PartialEq;
    type ScrollBehavior: PartialEq;
    type ScrollbarGutter: PartialEq;
    type ScrollbarColor;

    fn RetainComputedStyle(&self, style: &Self::ComputedStyle) -> Rc<Self::ComputedStyle>;
    fn StateParserMode(&self, state: &Self::StyleResolverState) -> CSSParserMode;
    fn StateEnsureParentStyle(&self, state: &mut Self::StyleResolverState);
    fn StateHasUnsupportedGuaranteedInvalid(&self, state: &Self::StyleResolverState) -> bool;
    fn StateElementLinkState(&self, state: &Self::StyleResolverState) -> EInsideLink;
    fn RecalcContextFromAncestors(&self, element: &Self::Element) -> Self::StyleRecalcContext;
    fn SetCollectorInsideLink(
        &self,
        collector: &mut Self::ElementRuleCollector,
        value: EInsideLink,
    );
    fn SetCollectingCSSRulesMode(&self, collector: &mut Self::ElementRuleCollector);
    fn SetSuppressVisited(&self, collector: &mut Self::ElementRuleCollector, value: bool);
    fn CollectorAddMatchedRulesToTracker(
        &self,
        collector: &Self::ElementRuleCollector,
        tracker: Option<&Self::StyleRuleUsageTracker>,
    );
    fn MatchedCSSRuleList(
        &self,
        collector: &Self::ElementRuleCollector,
    ) -> Option<Self::RuleIndexList>;
    fn CascadeGetCascadedValues(&self, cascade: &Self::StyleCascade) -> Self::CascadedValues;
    fn TextLayoutParentElement(&self, text: &Self::Text) -> Option<Rc<Self::Element>>;
    fn IsEnsuredInDisplayNone(&self, style: &Self::ComputedStyle) -> bool;
    fn IsEnsured(&self, style: &Self::ComputedStyle) -> bool;
    fn SetIsEnsuredInDisplayNone(&self, builder: &mut Self::ComputedStyleBuilder);
    fn ElementIsConnected(&self, element: &Self::Element) -> bool;
    fn FlatTreeParentElement(&self, element: &Self::Element) -> Option<Rc<Self::Element>>;
    fn PseudoIsLayoutSiblingOfOriginatingElement(&self, element: &Self::Element) -> bool;
    fn ContainerSelectorSelectsSizeContainers(&self, selector: &Self::ContainerSelector) -> bool;
    fn ContainerQueryFindContainer(
        &self,
        start: Option<&Self::Element>,
        selector: &Self::ContainerSelector,
        scope: Option<&Self::TreeScope>,
    ) -> Option<Rc<Self::Element>>;
    fn SetRequestPseudo(
        &self,
        request: &mut Self::StyleRequest,
        pseudo: PseudoId,
        argument: &AtomicString,
    );
    fn SetSearchTextRequestNotCurrent(&self, request: &mut Self::StyleRequest);
    fn GetPseudoElement(
        &self,
        element: &Self::Element,
        pseudo: PseudoId,
    ) -> Option<Rc<Self::Element>>;
    fn FindViewTransitionGroupPseudoElement(
        &self,
        transition: &Self::Element,
        argument: &AtomicString,
    ) -> Option<Rc<Self::Element>>;
    fn ViewTransitionClassList(&self, group: &Self::Element) -> Self::ViewTransitionClassList;
    fn SetRequestPseudoIdentList(
        &self,
        request: &mut Self::StyleRequest,
        classes: Self::ViewTransitionClassList,
    );
    fn ScopedKeyframeStylesForAnimation(
        &self,
        resolver: &Self::ScopedStyleResolver,
        name: &AtomicString,
    ) -> Option<Rc<Self::StyleRuleKeyframes>>;
    fn EngineKeyframeStylesForAnimation(
        &self,
        engine: &Self::StyleEngine,
        name: &AtomicString,
    ) -> Option<Rc<Self::StyleRuleKeyframes>>;
    fn KeyframesRules(&self, rules: &Self::RuleSet) -> Vec<Rc<Self::StyleRuleKeyframes>>;
    fn KeyframesName<'r>(&self, rule: &'r Self::StyleRuleKeyframes) -> &'r AtomicString;
    fn SetHasUnresolvedKeyframesRule(&self, resolver: &Self::ScopedStyleResolver);
    fn DocumentTreeScope<'d>(&self, document: &'d Self::Document) -> &'d Self::TreeScope;
    fn PositionTryForName(
        &self,
        resolver: &Self::ScopedStyleResolver,
        name: &AtomicString,
    ) -> Option<Rc<Self::StyleRulePositionTry>>;
    fn PositionTryRules(&self, rules: &Self::RuleSet) -> Vec<Rc<Self::StyleRulePositionTry>>;
    fn PositionTryName<'r>(&self, rule: &'r Self::StyleRulePositionTry) -> &'r AtomicString;
    fn UpdateViewportSize(&self, engine: &Self::StyleEngine);
    fn PropertyHandleName(
        &self,
        property: &Self::PropertyHandle,
    ) -> crate::css_property_name::CSSPropertyName;
    fn SetNamedProperty(
        &self,
        set: &Self::CSSPropertyValueSet,
        property: &crate::css_property_name::CSSPropertyName,
        value: &Self::CSSValue,
    );
    fn BeginCascadeAuthorRulesForTreeScope(
        &self,
        cascade: &mut Self::StyleCascade,
        scope: &Self::TreeScope,
    );
    fn NewCompositorKeyframeValue(
        &self,
        property: &Self::PropertyHandle,
        style: &Self::ComputedStyle,
        offset: f64,
    ) -> Self::CompositorKeyframeValue;
    fn RetainCSSValue(&self, value: &Self::CSSValue) -> Rc<Self::CSSValue>;
    fn UnparsedVariableDataNeedsResolution(&self, value: &Self::CSSValue) -> bool;
    fn ResolveSubstitutions(
        &self,
        state: &mut Self::StyleResolverState,
        value: &Self::CSSValue,
        scope: &Self::TreeScope,
    ) -> Option<Rc<Self::CSSValue>>;
    fn HasRandomFunctions(&self, value: &Self::CSSValue) -> bool;
    fn NewEmptyLengthConversionFlags(&self) -> Self::LengthConversionFlags;
    fn ComputedPropertyValue(
        &self,
        property: &crate::css_property_name::CSSPropertyName,
        document: &Self::Document,
        style: &Self::ComputedStyle,
    ) -> Option<Rc<Self::CSSValue>>;
    fn CascadeResolve(
        &self,
        state: &mut Self::StyleResolverState,
        property: &crate::css_property_name::CSSPropertyName,
        value: &Self::CSSValue,
        scope: &Self::TreeScope,
    ) -> Option<Rc<Self::CSSValue>>;
    fn PropertySetCSSValue(
        &self,
        set: &Self::CSSPropertyValueSet,
        property: CSSPropertyID,
    ) -> Rc<Self::CSSValue>;
    fn ApplyPropertyEnsuredScoped(
        &self,
        property: CSSPropertyID,
        state: &mut Self::StyleResolverState,
        value: &Self::CSSValue,
        scope: &Self::TreeScope,
    );
    fn NewNumberValue(&self, value: f64) -> Self::CSSValue;
    fn NewPixelValue(&self, value: f64) -> Self::CSSValue;
    fn SetFont(&self, builder: &mut Self::ComputedStyleBuilder, font: Rc<Self::Font>);
    fn RetainFont(&self, font: &Self::Font) -> Rc<Self::Font>;
    fn StyleFont(&self, style: &Self::ComputedStyle) -> Rc<Self::Font>;
    fn StyleFilter(&self, style: &Self::ComputedStyle) -> Self::FilterOperations;
    fn StyleUnicodeBidi(&self, style: &Self::ComputedStyle) -> foundation::UnicodeBidi;
    fn SetUnicodeBidi(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: foundation::UnicodeBidi,
    );
    fn AppliedTextDecorationData(
        &self,
        style: &Self::ComputedStyle,
    ) -> Self::AppliedTextDecorationData;
    fn SetBaseTextDecorationData(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        data: Self::AppliedTextDecorationData,
    );
    fn InheritedEqual(&self, first: &Self::ComputedStyle, second: &Self::ComputedStyle) -> bool;
    fn InitialLetterSize(&self, style: &Self::ComputedStyle) -> f32;
    fn FontCapHeight(&self, font: &Self::Font) -> f32;
    fn ComputedLineHeight(&self, style: &Self::ComputedStyle) -> f32;
    fn StyleFontDescription(&self, style: &Self::ComputedStyle) -> Self::FontDescription;
    fn SetFontComputedSize(&self, description: &mut Self::FontDescription, size: f32);
    fn SetFontSpecifiedSize(&self, description: &mut Self::FontDescription, size: f32);
    fn FontSelector(&self, font: &Self::Font) -> Rc<Self::FontSelector>;
    fn NewFont(
        &self,
        description: &Self::FontDescription,
        selector: &Self::FontSelector,
    ) -> Rc<Self::Font>;
    fn BuilderFontHeight(&self, builder: &Self::ComputedStyleBuilder) -> f32;
    fn SetLineHeight(&self, builder: &mut Self::ComputedStyleBuilder, height: foundation::Length);
    fn SetVerticalAlignBaseline(&self, builder: &mut Self::ComputedStyleBuilder);

    fn GetLayoutView(&self, document: &Self::Document) -> &Self::LayoutView;
    fn InitialContainingBlockSizeForPrinting(
        &self,
        view: &Self::LayoutView,
    ) -> Self::PrintingPhysicalSize;
    fn DefaultPageDescriptionRoundedPhysicalSize(
        &self,
        frame: &Self::LocalFrame,
    ) -> Self::PrintingPhysicalSize;
    fn SetInitialContainingBlockSizeForPrinting(
        &self,
        view: &Self::LayoutView,
        size: Self::PrintingPhysicalSize,
    );
    fn PrintMargins(&self, frame: &Self::LocalFrame) -> ResolverPrintMargins;
    fn NewPageRuleCollector(
        &self,
        parent: &Self::ComputedStyle,
        at_rule: crate::parser::css_at_rule_id::CSSAtRuleID,
        index: u32,
        name: &AtomicString,
        cascade: &mut Self::StyleCascade,
    ) -> Self::PageRuleCollector;
    fn PageCollectorMatchUARules(
        &self,
        collector: &mut Self::PageRuleCollector,
        rules: &Self::RuleSet,
        origin: CascadeOrigin,
        tree_scope: Option<&Self::TreeScope>,
    );
    fn ScopedMatchPageRules(
        &self,
        resolver: &Self::ScopedStyleResolver,
        collector: &mut Self::PageRuleCollector,
    );
    fn NewPageMarginsStyle(&self) -> Self::PageMarginsStyle;
    fn SetPageMarginStyle(
        &self,
        margins: &mut Self::PageMarginsStyle,
        slot: PageMarginSlot,
        style: Rc<Self::ComputedStyle>,
    );
    fn SetIsPageMarginBox(&self, builder: &mut Self::ComputedStyleBuilder, value: bool);

    fn ElementLayoutObject(&self, element: &Self::Element) -> Option<Rc<Self::LayoutObject>>;
    fn ShouldApplyAnyContainment(&self, object: &Self::LayoutObject) -> bool;
    fn FirstBodyElement(&self, document: &Self::Document) -> Option<Rc<Self::Element>>;
    fn LayoutViewStyle(&self, view: &Self::LayoutView) -> Rc<Self::ComputedStyle>;
    fn SetLayoutViewStyle(&self, view: &Self::LayoutView, style: Self::ComputedStyle);
    fn HasBackground(&self, style: &Self::ComputedStyle) -> bool;
    fn StyleImageAnimation(&self, style: &Self::ComputedStyle) -> foundation::ImageAnimationEnum;
    fn SetImageAnimation(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        animation: foundation::ImageAnimationEnum,
    );
    fn VisitedDependentColor(
        &self,
        style: &Self::ComputedStyle,
        property: CSSPropertyID,
    ) -> foundation::Color;
    fn SetStyleColor(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        property: CSSPropertyID,
        color: foundation::Color,
    );
    fn NewBackgroundFillLayers(&self) -> Self::FillLayers;
    fn BackgroundLayers(&self, style: &Self::ComputedStyle) -> Self::FillLayers;
    fn FillLayerCount(&self, layers: &Self::FillLayers) -> usize;
    fn SetLayerClipBorder(&self, layers: &mut Self::FillLayers, index: usize);
    fn LayerAttachmentIsScroll(&self, layers: &Self::FillLayers, index: usize) -> bool;
    fn SetLayerAttachmentLocal(&self, layers: &mut Self::FillLayers, index: usize);
    fn SetBackgroundLayers(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        layers: Self::FillLayers,
    );
    fn ImageRendering(&self, style: &Self::ComputedStyle) -> foundation::EImageRendering;
    fn SetImageRendering(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        rendering: foundation::EImageRendering,
    );
    fn IsOverflowVisibleAlongBothAxes(&self, style: &Self::ComputedStyle) -> bool;
    fn HasCustomScrollbarStyle(
        &self,
        style: &Self::ComputedStyle,
        document_element: Option<&Self::Element>,
    ) -> bool;
    fn PropagateOverscrollBehaviorFromRootEnabled(&self) -> bool;
    fn IsInOutermostMainFrame(&self, document: &Self::Document) -> bool;
    fn ChromeClientSetOverscrollBehavior(
        &self,
        document: &Self::Document,
        x: foundation::EOverscrollBehavior,
        y: foundation::EOverscrollBehavior,
    );
    fn CanvasTextColor(&self) -> foundation::Color;
    fn UpdateFontOrientation(&self, builder: &mut Self::ComputedStyleBuilder);
    fn InitialScrollSnapType(&self) -> Self::ScrollSnapType;
    fn AutoScrollBehavior(&self) -> Self::ScrollBehavior;
    fn AutoScrollbarGutter(&self) -> Self::ScrollbarGutter;
    fn ViewportStyleWritingMode(&self, style: &Self::ComputedStyle) -> foundation::WritingMode;
    fn ViewportBuilderWritingMode(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> foundation::WritingMode;
    fn SetViewportWritingMode(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: foundation::WritingMode,
    );
    fn ViewportStyleDirection(&self, style: &Self::ComputedStyle) -> TextDirection;
    fn ViewportBuilderDirection(&self, builder: &Self::ComputedStyleBuilder) -> TextDirection;
    fn SetViewportDirection(&self, builder: &mut Self::ComputedStyleBuilder, value: TextDirection);
    fn ViewportStyleOverscrollBehaviorX(
        &self,
        style: &Self::ComputedStyle,
    ) -> foundation::EOverscrollBehavior;
    fn ViewportBuilderOverscrollBehaviorX(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> foundation::EOverscrollBehavior;
    fn SetViewportOverscrollBehaviorX(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: foundation::EOverscrollBehavior,
    );
    fn ViewportStyleOverscrollBehaviorY(
        &self,
        style: &Self::ComputedStyle,
    ) -> foundation::EOverscrollBehavior;
    fn ViewportBuilderOverscrollBehaviorY(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> foundation::EOverscrollBehavior;
    fn SetViewportOverscrollBehaviorY(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: foundation::EOverscrollBehavior,
    );
    fn ViewportStyleOverflowX(&self, style: &Self::ComputedStyle) -> EOverflow;
    fn ViewportBuilderOverflowX(&self, builder: &Self::ComputedStyleBuilder) -> EOverflow;
    fn SetViewportOverflowX(&self, builder: &mut Self::ComputedStyleBuilder, value: EOverflow);
    fn ViewportStyleOverflowY(&self, style: &Self::ComputedStyle) -> EOverflow;
    fn ViewportBuilderOverflowY(&self, builder: &Self::ComputedStyleBuilder) -> EOverflow;
    fn SetViewportOverflowY(&self, builder: &mut Self::ComputedStyleBuilder, value: EOverflow);
    fn ViewportStyleOverflowAnchor(
        &self,
        style: &Self::ComputedStyle,
    ) -> foundation::EOverflowAnchor;
    fn ViewportBuilderOverflowAnchor(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> foundation::EOverflowAnchor;
    fn SetViewportOverflowAnchor(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: foundation::EOverflowAnchor,
    );
    fn ViewportStyleEffectiveTouchAction(
        &self,
        style: &Self::ComputedStyle,
    ) -> foundation::TouchAction;
    fn ViewportBuilderEffectiveTouchAction(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> foundation::TouchAction;
    fn SetViewportEffectiveTouchAction(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: foundation::TouchAction,
    );
    fn ViewportStyleScrollBehavior(&self, style: &Self::ComputedStyle) -> Self::ScrollBehavior;
    fn ViewportBuilderScrollBehavior(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> Self::ScrollBehavior;
    fn SetViewportScrollBehavior(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: Self::ScrollBehavior,
    );
    fn ViewportStyleDarkColorScheme(&self, style: &Self::ComputedStyle) -> bool;
    fn ViewportBuilderDarkColorScheme(&self, builder: &Self::ComputedStyleBuilder) -> bool;
    fn SetViewportDarkColorScheme(&self, builder: &mut Self::ComputedStyleBuilder, value: bool);
    fn ViewportStyleColorSchemeForced(&self, style: &Self::ComputedStyle) -> bool;
    fn ViewportBuilderColorSchemeForced(&self, builder: &Self::ComputedStyleBuilder) -> bool;
    fn SetViewportColorSchemeForced(&self, builder: &mut Self::ComputedStyleBuilder, value: bool);
    fn ViewportStyleScrollbarGutter(&self, style: &Self::ComputedStyle) -> Self::ScrollbarGutter;
    fn ViewportBuilderScrollbarGutter(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> Self::ScrollbarGutter;
    fn SetViewportScrollbarGutter(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: Self::ScrollbarGutter,
    );
    fn ViewportStyleScrollbarWidth(
        &self,
        style: &Self::ComputedStyle,
    ) -> foundation::EScrollbarWidth;
    fn ViewportBuilderScrollbarWidth(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> foundation::EScrollbarWidth;
    fn SetViewportScrollbarWidth(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: foundation::EScrollbarWidth,
    );
    fn ViewportStyleScrollbarColor(
        &self,
        style: &Self::ComputedStyle,
    ) -> Option<Rc<Self::ScrollbarColor>>;
    fn ViewportBuilderScrollbarColor(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> Option<Rc<Self::ScrollbarColor>>;
    fn SetViewportScrollbarColor(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: Option<Rc<Self::ScrollbarColor>>,
    );
    fn ViewportStyleForcedColorAdjust(
        &self,
        style: &Self::ComputedStyle,
    ) -> foundation::EForcedColorAdjust;
    fn ViewportBuilderForcedColorAdjust(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> foundation::EForcedColorAdjust;
    fn SetViewportForcedColorAdjust(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: foundation::EForcedColorAdjust,
    );
    fn ViewportStyleColorSchemeFlagsIsNormal(&self, style: &Self::ComputedStyle) -> bool;
    fn ViewportBuilderColorSchemeFlagsIsNormal(&self, builder: &Self::ComputedStyleBuilder)
        -> bool;
    fn SetViewportColorSchemeFlagsIsNormal(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: bool,
    );
    fn ViewportStyleScrollSnapType(&self, style: &Self::ComputedStyle) -> Self::ScrollSnapType;
    fn ViewportBuilderScrollSnapType(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> Self::ScrollSnapType;
    fn SetViewportScrollSnapType(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: Self::ScrollSnapType,
    );
    fn ViewportStyleScrollPaddingTop(&self, style: &Self::ComputedStyle) -> foundation::Length;
    fn ViewportBuilderScrollPaddingTop(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> foundation::Length;
    fn SetViewportScrollPaddingTop(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: foundation::Length,
    );
    fn ViewportStyleScrollPaddingRight(&self, style: &Self::ComputedStyle) -> foundation::Length;
    fn ViewportBuilderScrollPaddingRight(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> foundation::Length;
    fn SetViewportScrollPaddingRight(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: foundation::Length,
    );
    fn ViewportStyleScrollPaddingBottom(&self, style: &Self::ComputedStyle) -> foundation::Length;
    fn ViewportBuilderScrollPaddingBottom(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> foundation::Length;
    fn SetViewportScrollPaddingBottom(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: foundation::Length,
    );
    fn ViewportStyleScrollPaddingLeft(&self, style: &Self::ComputedStyle) -> foundation::Length;
    fn ViewportBuilderScrollPaddingLeft(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> foundation::Length;
    fn SetViewportScrollPaddingLeft(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: foundation::Length,
    );
}

// cpp: style_resolver.cc:2145-2180. A real scope restores the saved document
// size, including when an operation exits early or unwinds.
struct ViewportSizeChangeScopeForPrinting<'a, B: StyleResolverCompleteBackend> {
    backend: &'a B,
    document: &'a B::Document,
    saved_size: Option<B::PrintingPhysicalSize>,
}
impl<'a, B: StyleResolverCompleteBackend> ViewportSizeChangeScopeForPrinting<'a, B> {
    fn new(backend: &'a B, document: &'a B::Document) -> Self {
        let saved_size = if backend.Printing(document) {
            let view = backend.GetLayoutView(document);
            let saved = backend.InitialContainingBlockSizeForPrinting(view);
            let frame = backend
                .GetFrame(document)
                .expect("printing document has a frame");
            backend.SetInitialContainingBlockSizeForPrinting(
                view,
                backend.DefaultPageDescriptionRoundedPhysicalSize(frame),
            );
            backend.UpdateViewportSize(backend.GetStyleEngine(document));
            Some(saved)
        } else {
            None
        };
        Self {
            backend,
            document,
            saved_size,
        }
    }
}
impl<B: StyleResolverCompleteBackend> Drop for ViewportSizeChangeScopeForPrinting<'_, B> {
    fn drop(&mut self) {
        if let Some(size) = self.saved_size.take() {
            self.backend.SetInitialContainingBlockSizeForPrinting(
                self.backend.GetLayoutView(self.document),
                size,
            );
            self.backend
                .UpdateViewportSize(self.backend.GetStyleEngine(self.document));
        }
    }
}
impl<'a, B: StyleResolverCompleteBackend> StyleResolver<'a, B> {
    // This helper expands the source's two-argument StyleResolverState
    // constructor; the external state class retains the supplied operands.
    fn NewDefaultState(
        &self,
        document: &B::Document,
        element: &B::Element,
    ) -> B::StyleResolverState {
        self.backend
            .NewStyleResolverState(document, element, None, &self.backend.NewStyleRequest())
    }
    fn ParentOverrideRequest(&self, parent: Option<&B::ComputedStyle>) -> B::StyleRequest {
        let mut request = self.backend.NewStyleRequest();
        self.backend.SetRequestParentOverride(&mut request, parent);
        request
    }
    // cpp: style_resolver.cc:2097-2124; h:108-114
    pub fn CreateCompositorKeyframeValueSnapshot(
        &self,
        element: &B::Element,
        base: &B::ComputedStyle,
        parent: Option<&B::ComputedStyle>,
        property: &B::PropertyHandle,
        value: Option<&B::CSSValue>,
        offset: f64,
    ) -> B::CompositorKeyframeValue {
        let b = self.backend;
        let mut state = b.NewStyleResolverState(
            b.ElementDocument(element),
            element,
            None,
            &self.ParentOverrideRequest(parent),
        );
        b.StateCreateNewClonedStyle(&mut state, base);
        if let Some(value) = value {
            let mut cascade = b.NewStyleCascade(&mut state);
            let set = b.NewMutableCSSPropertyValueSet(b.StateParserMode(&state));
            b.SetNamedProperty(&set, &b.PropertyHandleName(property), value);
            b.BeginCascadeAuthorRulesForTreeScope(&mut cascade, b.ElementTreeScope(element));
            b.AddCascadeMatchedProperties(&mut cascade, &set, CascadeOrigin::kAuthor);
            b.CascadeApply(&mut cascade, CascadeFilter::new());
        }
        b.NewCompositorKeyframeValue(property, &b.StateTakeStyle(&mut state), offset)
    }
    // cpp: style_resolver.cc:2182-2270; h:126-129
    pub fn StyleForPage(
        &self,
        index: u32,
        name: &AtomicString,
        fitting_scale: f32,
        ignore_author_style: bool,
    ) -> Rc<B::ComputedStyle> {
        use crate::parser::css_at_rule_id::CSSAtRuleID;
        let b = self.backend;
        let Some(root) = b.DocumentElement(self.GetDocument()) else {
            return Rc::new(self.InitialStyleForElement());
        };
        let Some(parent) = b.ElementComputedStyle(&root).filter(|s| !b.IsEnsured(s)) else {
            return Rc::new(self.InitialStyleForElement());
        };
        let mut state = b.NewStyleResolverState(
            self.GetDocument(),
            &root,
            None,
            &self.ParentOverrideRequest(Some(&parent)),
        );
        b.StateCreateNewStyle(&mut state, &self.InitialStyleForElement(), &parent, false);
        b.SetDisplay(b.StateStyleBuilderMut(&mut state), EDisplay::kBlock);
        let _viewport_scope = ViewportSizeChangeScopeForPrinting::new(b, self.document_);
        let mut cascade = b.NewStyleCascade(&mut state);
        let mut collector = b.NewPageRuleCollector(
            &parent,
            CSSAtRuleID::kCSSAtRulePage,
            index,
            name,
            &mut cascade,
        );
        b.PageCollectorMatchUARules(
            &mut collector,
            &b.DefaultHtmlStyle(),
            CascadeOrigin::kUserAgent,
            None,
        );
        if b.Printing(self.GetDocument()) {
            b.ApplyScopedProperty(
                crate::css_property_name::CSSPropertyName::new(CSSPropertyID::kZoom),
                &mut state,
                &b.NewNumberValue(fitting_scale as f64),
                b.DocumentTreeScope(self.GetDocument()),
            );
            let margins = b.PrintMargins(
                b.GetFrame(self.GetDocument())
                    .expect("printing document has a frame"),
            );
            let set = b.NewMutableCSSPropertyValueSet(CSSParserMode::kHTMLStandardMode);
            for (property, value) in [
                (CSSPropertyID::kMarginTop, margins.top),
                (CSSPropertyID::kMarginRight, margins.right),
                (CSSPropertyID::kMarginBottom, margins.bottom),
                (CSSPropertyID::kMarginLeft, margins.left),
            ] {
                b.SetProperty(
                    &set,
                    property,
                    &b.NewPixelValue(value),
                    margins.ignore_css_margins,
                );
            }
            b.AddCascadeMatchedProperties(&mut cascade, &set, CascadeOrigin::kUserAgent);
        }
        if !ignore_author_style {
            if let Some(resolver) = b.TreeScopeResolver(b.DocumentTreeScope(self.GetDocument())) {
                b.ScopedMatchPageRules(resolver, &mut collector);
            }
        }
        b.CascadeApply(&mut cascade, CascadeFilter::new());
        b.StateLoadPendingResources(&mut state);
        b.StateTakeStyle(&mut state)
    }
    // cpp: style_resolver.cc:2272-2337; h:135-138
    pub fn StyleForPageMargins(
        &self,
        page_style: &B::ComputedStyle,
        index: u32,
        name: &AtomicString,
        margins: &mut B::PageMarginsStyle,
    ) {
        use crate::parser::css_at_rule_id::CSSAtRuleID::*;
        use PageMarginSlot::*;
        let b = self.backend;
        let Some(root) = b.DocumentElement(self.GetDocument()) else {
            return;
        };
        let table = [
            (TopLeft, kCSSAtRuleTopLeft),
            (TopCenter, kCSSAtRuleTopCenter),
            (TopRight, kCSSAtRuleTopRight),
            (RightTop, kCSSAtRuleRightTop),
            (RightMiddle, kCSSAtRuleRightMiddle),
            (RightBottom, kCSSAtRuleRightBottom),
            (BottomLeft, kCSSAtRuleBottomLeft),
            (BottomCenter, kCSSAtRuleBottomCenter),
            (BottomRight, kCSSAtRuleBottomRight),
            (LeftTop, kCSSAtRuleLeftTop),
            (LeftMiddle, kCSSAtRuleLeftMiddle),
            (LeftBottom, kCSSAtRuleLeftBottom),
            (TopLeftCorner, kCSSAtRuleTopLeftCorner),
            (TopRightCorner, kCSSAtRuleTopRightCorner),
            (BottomRightCorner, kCSSAtRuleBottomRightCorner),
            (BottomLeftCorner, kCSSAtRuleBottomLeftCorner),
        ];
        let _viewport_scope = ViewportSizeChangeScopeForPrinting::new(b, self.document_);
        for (slot, at_rule) in table {
            let mut state = b.NewStyleResolverState(
                self.GetDocument(),
                &root,
                None,
                &self.ParentOverrideRequest(Some(page_style)),
            );
            b.StateCreateNewStyle(
                &mut state,
                &self.InitialStyleForElement(),
                page_style,
                false,
            );
            b.SetDisplay(b.StateStyleBuilderMut(&mut state), EDisplay::kBlock);
            b.SetIsPageMarginBox(b.StateStyleBuilderMut(&mut state), true);
            let mut cascade = b.NewStyleCascade(&mut state);
            let mut collector =
                b.NewPageRuleCollector(page_style, at_rule, index, name, &mut cascade);
            b.PageCollectorMatchUARules(
                &mut collector,
                &b.DefaultHtmlStyle(),
                CascadeOrigin::kUserAgent,
                None,
            );
            if let Some(resolver) = b.TreeScopeResolver(b.DocumentTreeScope(self.GetDocument())) {
                b.ScopedMatchPageRules(resolver, &mut collector);
            }
            b.CascadeApply(&mut cascade, CascadeFilter::new());
            b.StateLoadPendingResources(&mut state);
            b.SetPageMarginStyle(margins, slot, b.StateTakeStyle(&mut state));
        }
    }
    // cpp: style_resolver.cc:2339-2350; h:142
    pub fn LoadPaginationResources(&self) {
        let name = &foundation::g_null_atom;
        let page_style = self.StyleForPage(0, name, 1.0, false);
        let mut ignored = self.backend.NewPageMarginsStyle();
        self.StyleForPageMargins(&page_style, 0, name, &mut ignored);
    }
    // cpp: style_resolver.cc:2429-2438; h:144
    pub fn StyleForText(&self, text: &B::Text) -> Option<Rc<B::ComputedStyle>> {
        self.backend
            .TextLayoutParentElement(text)
            .and_then(|parent| self.backend.ElementComputedStyle(&parent))
            .filter(|style| !self.backend.IsEnsuredInDisplayNone(style))
    }
    // cpp: style_resolver.cc:2440-2443; h:331
    fn AddMatchedRulesToTracker(&self, collector: &B::ElementRuleCollector) {
        self.backend
            .CollectorAddMatchedRulesToTracker(collector, self.tracker_);
    }
    // cpp: style_resolver.cc:2445-2458; h:204
    pub fn StyleRulesForElement(
        &mut self,
        element: &B::Element,
        include: u32,
    ) -> Option<Vec<Rc<B::StyleRule>>> {
        let b = self.backend;
        let state = self.NewDefaultState(self.GetDocument(), element);
        let mut result = b.NewMatchResult();
        let mut collector = b.NewCollectorForResult(
            &state,
            &b.RecalcContextFromAncestors(element),
            &mut self.selector_filter_,
            &mut result,
        );
        b.SetCollectorInsideLink(&mut collector, EInsideLink::kNotInsideLink);
        b.SetCollectingStyleRulesMode(&mut collector);
        b.SetSuppressVisited(&mut collector, true);
        self.CollectPseudoRulesForElement(
            element,
            &mut collector,
            PseudoId::kPseudoIdNone,
            &foundation::g_null_atom,
            include,
        );
        b.MatchedStyleRuleList(&collector)
    }
    // cpp: style_resolver.cc:2460-2475; h:205-207
    pub fn CascadedValuesForElement(
        &mut self,
        element: &B::Element,
        pseudo: PseudoId,
    ) -> B::CascadedValues {
        let b = self.backend;
        let mut state = self.NewDefaultState(self.GetDocument(), element);
        b.StateCreateNewClonedStyle(&mut state, self.InitialStyle());
        let mut cascade = b.NewStyleCascade(&mut state);
        let mut collector = b.NewCollectorForCascade(
            &state,
            &b.RecalcContextFromAncestors(element),
            &mut self.selector_filter_,
            &mut cascade,
        );
        b.SetCollectorInsideLink(&mut collector, EInsideLink::kNotInsideLink);
        let mut request = b.NewStyleRequest();
        b.SetRequestPseudo(&mut request, pseudo, &foundation::g_null_atom);
        b.SetPseudoElementStyleRequest(&mut collector, &request);
        self.MatchAllRules(&mut state, &mut collector, false);
        b.CascadeApply(&mut cascade, CascadeFilter::new());
        b.CascadeGetCascadedValues(&cascade)
    }
    // cpp: style_resolver.cc:2477-2492; h:209-211
    pub fn FindContainerForElement(
        &self,
        element: &B::Element,
        selector: &B::ContainerSelector,
        scope: Option<&B::TreeScope>,
    ) -> Option<Rc<B::Element>> {
        let b = self.backend;
        let mut start = b.FlatTreeParentElement(element);
        if b.IsPseudoElement(element)
            && b.PseudoIsLayoutSiblingOfOriginatingElement(element)
            && (b.ContainerSelectorSelectsSizeContainers(selector)
                || b.ElementPseudoIdForStyling(element) == PseudoId::kPseudoIdSkeleton)
        {
            start = b.FlatTreeParentElement(
                start
                    .as_deref()
                    .expect("layout-sibling pseudo has a parent"),
            );
        }
        b.ContainerQueryFindContainer(start.as_deref(), selector, scope)
    }
    // cpp: style_resolver.cc:2494-2517; h:197-201
    pub fn PseudoCSSRulesForElement(
        &mut self,
        element: Option<&B::Element>,
        pseudo: PseudoId,
        argument: &AtomicString,
        include: u32,
    ) -> Option<B::RuleIndexList> {
        let element = element.filter(|element| self.backend.ElementIsConnected(element))?;
        let b = self.backend;
        let state = self.NewDefaultState(self.GetDocument(), element);
        let mut result = b.NewMatchResult();
        let mut collector = b.NewCollectorForResult(
            &state,
            &b.RecalcContextFromAncestors(element),
            &mut self.selector_filter_,
            &mut result,
        );
        b.SetCollectorInsideLink(&mut collector, b.StateElementLinkState(&state));
        b.SetCollectingCSSRulesMode(&mut collector);
        self.CollectPseudoRulesForElement(element, &mut collector, pseudo, argument, include);
        if self.tracker_.is_some() {
            self.AddMatchedRulesToTracker(&collector);
        }
        b.MatchedCSSRuleList(&collector)
    }
    // cpp: style_resolver.cc:2519-2523; h:195-196
    pub fn CssRulesForElement(
        &mut self,
        element: Option<&B::Element>,
        include: u32,
    ) -> Option<B::RuleIndexList> {
        self.PseudoCSSRulesForElement(
            element,
            PseudoId::kPseudoIdNone,
            &foundation::g_null_atom,
            include,
        )
    }
    // cpp: style_resolver.cc:2525-2570; h:333-337
    fn CollectPseudoRulesForElement(
        &mut self,
        element: &B::Element,
        collector: &mut B::ElementRuleCollector,
        pseudo: PseudoId,
        argument: &AtomicString,
        include: u32,
    ) {
        let b = self.backend;
        let mut request = b.NewStyleRequest();
        b.SetRequestPseudo(&mut request, pseudo, argument);
        if pseudo == PseudoId::kPseudoIdSearchText {
            b.SetSearchTextRequestNotCurrent(&mut request);
        }
        if b.IsTransitionPseudoElement(pseudo) && pseudo != PseudoId::kPseudoIdViewTransition {
            if let Some(transition) = b.GetPseudoElement(element, PseudoId::kPseudoIdViewTransition)
            {
                if let Some(group) = b.FindViewTransitionGroupPseudoElement(&transition, argument) {
                    b.SetRequestPseudoIdentList(&mut request, b.ViewTransitionClassList(&group));
                }
            }
        }
        b.SetPseudoElementStyleRequest(collector, &request);
        if include & CSSRuleFilter::kUACSSRules as u32 != 0 {
            self.MatchUARules(element, collector);
        }
        if include & CSSRuleFilter::kUserCSSRules as u32 != 0 {
            self.MatchUserRules(collector);
        }
        if include & CSSRuleFilter::kAuthorCSSRules as u32 != 0 {
            self.MatchAuthorRules(element, collector);
        }
    }
    // cpp: style_resolver.cc:631-643
    fn CollectScopedResolversForHostedShadowTrees<'e>(
        &self,
        element: &'e B::Element,
        resolvers: &mut Vec<&'e B::ScopedStyleResolver>,
    ) where
        B::ShadowRoot: 'e,
    {
        if let Some(root) = self.backend.GetShadowRoot(element) {
            if let Some(resolver) = self.backend.ShadowRootResolver(root) {
                resolvers.push(resolver);
            }
        }
    }
    // cpp: style_resolver.cc:2694-2751; h:175-183
    pub fn FindKeyframesRule<'e>(
        &self,
        element: &'e B::Element,
        animating: &B::Element,
        name: &AtomicString,
        name_scope: Option<&'e B::TreeScope>,
    ) -> FindKeyframesRuleResult<'e, B::StyleRuleKeyframes, B::TreeScope>
    where
        B::ShadowRoot: 'e,
        B::TreeScope: 'e,
        B::ScopedStyleResolver: 'e,
    {
        let b = self.backend;
        let mut resolvers = Vec::new();
        if let Some(scope) = name_scope {
            if let Some(resolver) = b.TreeScopeResolver(scope) {
                resolvers.push(resolver);
            }
        }
        if resolvers.is_empty() {
            self.CollectScopedResolversForHostedShadowTrees(element, &mut resolvers);
            if let Some(resolver) = self.ScopedResolverFor(element) {
                resolvers.push(resolver);
            }
        }
        for &resolver in &resolvers {
            if let Some(rule) = b.ScopedKeyframeStylesForAnimation(resolver, name) {
                return FindKeyframesRuleResult {
                    rule: Some(rule),
                    tree_scope: Some(b.ResolverTreeScope(resolver)),
                };
            }
        }
        if let Some(rule) =
            b.EngineKeyframeStylesForAnimation(b.GetStyleEngine(self.GetDocument()), name)
        {
            return FindKeyframesRuleResult {
                rule: Some(rule),
                tree_scope: None,
            };
        }
        let mut matched = None;
        self.ForEachUARulesForElement(animating, None, |rules, _index| {
            for rule in b.KeyframesRules(&rules) {
                if b.KeyframesName(&rule) == name {
                    matched = Some(rule);
                }
            }
        });
        if matched.is_some() {
            return FindKeyframesRuleResult {
                rule: matched,
                tree_scope: None,
            };
        }
        for resolver in resolvers {
            b.SetHasUnresolvedKeyframesRule(resolver);
        }
        FindKeyframesRuleResult {
            rule: None,
            tree_scope: None,
        }
    }
    // cpp: style_resolver.cc:3003-3009; h:242-244
    pub fn ComputeValue(
        &self,
        element: &B::Element,
        property: &crate::css_property_name::CSSPropertyName,
        value: &B::CSSValue,
    ) -> Option<Rc<B::CSSValue>> {
        let mut flags = self.backend.NewEmptyLengthConversionFlags();
        self.ComputeValueWithFlags(element, property, value, &mut flags)
    }
    // cpp: style_resolver.cc:3011-3018; h:236-239
    pub fn ComputeValueWithFlags(
        &self,
        element: &B::Element,
        property: &crate::css_property_name::CSSPropertyName,
        value: &B::CSSValue,
        flags: &mut B::LengthConversionFlags,
    ) -> Option<Rc<B::CSSValue>> {
        let mut has_random = false;
        self.ComputeValueWithFlagsAndRandom(element, property, value, flags, &mut has_random)
    }
    // cpp: style_resolver.cc:3020-3086; h:231-235
    pub fn ComputeValueWithFlagsAndRandom(
        &self,
        element: &B::Element,
        property: &crate::css_property_name::CSSPropertyName,
        value: &B::CSSValue,
        flags: &mut B::LengthConversionFlags,
        has_random: &mut bool,
    ) -> Option<Rc<B::CSSValue>> {
        let b = self.backend;
        let document = b.ElementDocument(element);
        b.UpdateViewportSize(b.GetStyleEngine(document));
        let base = b
            .ElementComputedStyle(element)
            .expect("ComputeValue requires a computed element style");
        let mut state = self.NewDefaultState(document, element);
        b.StateEnsureParentStyle(&mut state);
        let mut cascade = b.NewStyleCascade(&mut state);
        b.StateCreateNewClonedStyle(&mut state, &base);
        b.SetIsEnsuredInDisplayNone(b.StateStyleBuilderMut(&mut state));
        let mut resolved = b.RetainCSSValue(value);
        if property.IsCustomProperty()
            && b.ValueIsUnparsedDeclaration(value)
            && b.UnparsedVariableDataNeedsResolution(value)
        {
            if let Some(substituted) =
                b.ResolveSubstitutions(&mut state, value, b.ElementTreeScope(element))
            {
                resolved = substituted;
            }
        }
        if b.HasRandomFunctions(&resolved) {
            *has_random = true;
        }
        let set = b.NewMutableCSSPropertyValueSet(b.StateParserMode(&state));
        b.SetNamedProperty(&set, property, &resolved);
        b.BeginCascadeAuthorRulesForTreeScope(&mut cascade, b.ElementTreeScope(element));
        b.AddCascadeMatchedProperties(&mut cascade, &set, CascadeOrigin::kAuthor);
        b.CascadeApply(&mut cascade, CascadeFilter::new());
        if b.StateHasUnsupportedGuaranteedInvalid(&state) {
            return None;
        }
        *flags = b.TakeLengthConversionFlags(&mut state);
        let style = b.StateTakeStyle(&mut state);
        let computed = b.ComputedPropertyValue(property, document, &style);
        if computed
            .as_deref()
            .is_some_and(|value| b.HasRandomFunctions(value))
        {
            *has_random = true;
        }
        computed
    }
    // cpp: style_resolver.cc:3088-3100; h:252-255
    pub fn ResolveValue(
        &self,
        element: &B::Element,
        style: &B::ComputedStyle,
        property: &crate::css_property_name::CSSPropertyName,
        value: &B::CSSValue,
    ) -> Option<Rc<B::CSSValue>> {
        let b = self.backend;
        let document = b.ElementDocument(element);
        b.UpdateViewportSize(b.GetStyleEngine(document));
        let mut state = self.NewDefaultState(document, element);
        b.StateCreateNewClonedStyle(&mut state, style);
        b.CascadeResolve(&mut state, property, value, b.DocumentTreeScope(document))
    }
    // cpp: style_resolver.cc:3102-3124; h:261-263
    pub fn ComputeFilterOperations(
        &self,
        element: &B::Element,
        font: &B::Font,
        value: &B::CSSValue,
    ) -> B::FilterOperations {
        let b = self.backend;
        let mut parent_builder = self.CreateComputedStyleBuilder();
        b.SetFont(&mut parent_builder, b.RetainFont(font));
        let parent = b.TakeStyle(parent_builder);
        let mut state = b.NewStyleResolverState(
            self.GetDocument(),
            element,
            None,
            &self.ParentOverrideRequest(Some(&parent)),
        );
        b.UpdateViewportSize(b.GetStyleEngine(self.GetDocument()));
        b.StateCreateNewClonedStyle(&mut state, &parent);
        b.ApplyPropertyEnsuredScoped(
            CSSPropertyID::kFilter,
            &mut state,
            value,
            b.DocumentTreeScope(self.GetDocument()),
        );
        b.StateLoadPendingResources(&mut state);
        b.StyleFilter(&b.StateTakeStyle(&mut state))
    }
    // cpp: style_resolver.cc:3126-3142; h:265-267
    pub fn StyleForInterpolations(
        &mut self,
        element: &B::Element,
        interpolations: &B::ActiveInterpolationsMap,
    ) -> Rc<B::ComputedStyle> {
        let b = self.backend;
        let context = b.RecalcContextFromAncestors(element);
        let request = b.NewStyleRequest();
        let mut state =
            b.NewStyleResolverState(self.GetDocument(), element, Some(&context), &request);
        let mut cascade = b.NewStyleCascade(&mut state);
        self.ApplyBaseStyle(element, &context, &request, &mut state, &mut cascade);
        let base = b.BuilderCloneStyle(b.StateStyleBuilder(&state));
        let important = b.CascadeReleaseImportantSet(&mut cascade);
        let data = b.NewStyleBaseData(base, important);
        b.SetBaseData(b.StateStyleBuilderMut(&mut state), Some(data));
        self.ApplyInterpolations(&mut state, &mut cascade, interpolations);
        b.StateTakeStyle(&mut state)
    }
    // cpp: style_resolver.cc:3144-3150; h:327-329
    fn ApplyInterpolations(
        &self,
        _state: &mut B::StyleResolverState,
        cascade: &mut B::StyleCascade,
        interpolations: &B::ActiveInterpolationsMap,
    ) {
        self.backend
            .CascadeAddInterpolations(cascade, interpolations, CascadeOrigin::kAnimation);
        self.backend.CascadeApply(cascade, CascadeFilter::new());
    }
    // cpp: style_resolver.cc:3152-3179; h:274-277
    pub fn BeforeChangeStyleForTransitionUpdate(
        &self,
        element: &B::Element,
        base: &B::ComputedStyle,
        interpolations: &B::ActiveInterpolationsMap,
    ) -> Rc<B::ComputedStyle> {
        let b = self.backend;
        let mut state = self.NewDefaultState(self.GetDocument(), element);
        let mut cascade = b.NewStyleCascade(&mut state);
        b.StateCreateNewClonedStyle(&mut state, base);
        if b.StateParentStyle(&state).is_none() {
            if !b
                .DocumentElement(self.GetDocument())
                .is_some_and(|root| std::ptr::eq(root.as_ref(), element))
            {
                return b.StateTakeStyle(&mut state);
            }
            b.StateSetParentStyle(&mut state, Rc::new(self.InitialStyleForElement()));
            let parent = b.StateParentStyle(&state).expect("initial parent assigned");
            b.StateSetLayoutParentStyle(&mut state, parent);
        }
        let data = b.NewStyleBaseData(b.RetainComputedStyle(base), None);
        b.SetBaseData(b.StateStyleBuilderMut(&mut state), Some(data));
        self.ApplyInterpolations(&mut state, &mut cascade, interpolations);
        b.StateTakeStyle(&mut state)
    }
    // cpp: style_resolver.cc:3282-3313; h:213
    pub fn ComputeFont(
        &self,
        element: &B::Element,
        style: &B::ComputedStyle,
        properties: &B::CSSPropertyValueSet,
    ) -> Rc<B::Font> {
        let b = self.backend;
        let mut state = b.NewStyleResolverState(
            self.GetDocument(),
            element,
            None,
            &self.ParentOverrideRequest(Some(style)),
        );
        b.UpdateViewportSize(b.GetStyleEngine(self.GetDocument()));
        b.StateCreateNewClonedStyle(&mut state, style);
        if let Some(parent) = b.ElementComputedStyle(element) {
            b.StateSetParentStyle(&mut state, parent);
        }
        for property in [
            CSSPropertyID::kFontSize,
            CSSPropertyID::kFontFamily,
            CSSPropertyID::kFontStretch,
            CSSPropertyID::kFontStyle,
            CSSPropertyID::kFontVariantCaps,
            CSSPropertyID::kFontWeight,
        ] {
            b.ApplyPropertyEnsuredScoped(
                property,
                &mut state,
                &b.PropertySetCSSValue(properties, property),
                b.DocumentTreeScope(self.GetDocument()),
            );
        }
        b.StateUpdateFont(&mut state);
        b.StyleFont(&b.StateTakeStyle(&mut state))
    }
    // cpp: style_resolver.cc:3369-3377; h:158-160
    pub fn CreateAnonymousStyleBuilderWithDisplay(
        &self,
        parent: &B::ComputedStyle,
        display: EDisplay,
    ) -> B::ComputedStyleBuilder {
        let b = self.backend;
        let mut builder = self.CreateComputedStyleBuilderInheritingFrom(parent);
        b.SetUnicodeBidi(&mut builder, b.StyleUnicodeBidi(parent));
        b.SetBaseTextDecorationData(&mut builder, b.AppliedTextDecorationData(parent));
        b.SetDisplay(&mut builder, display);
        builder
    }
    // cpp: style_resolver.cc:3379-3384; h:161-163
    pub fn CreateAnonymousStyleWithDisplay(
        &self,
        parent: &B::ComputedStyle,
        display: EDisplay,
    ) -> B::ComputedStyle {
        self.backend
            .TakeStyle(self.CreateAnonymousStyleBuilderWithDisplay(parent, display))
    }
    // cpp: style_resolver.cc:3386-3393; h:167-169
    pub fn CreateInheritedDisplayContentsStyleIfNeeded(
        &self,
        parent: &B::ComputedStyle,
        layout_parent: &B::ComputedStyle,
    ) -> Option<B::ComputedStyle> {
        if self.backend.InheritedEqual(parent, layout_parent) {
            return None;
        }
        Some(self.CreateAnonymousStyleWithDisplay(parent, EDisplay::kInline))
    }
    // cpp: style_resolver.cc:3710-3744
    fn ComputeInitialLetterFont(
        &self,
        style: &B::ComputedStyle,
        paragraph: &B::ComputedStyle,
    ) -> Rc<B::Font> {
        let b = self.backend;
        let font = b.StyleFont(style);
        let cap_height = b.FontCapHeight(&font);
        let desired_cap_height = b.ComputedLineHeight(paragraph)
            * (b.InitialLetterSize(style) - 1.0)
            + b.FontCapHeight(&b.StyleFont(paragraph));
        let mut adjusted_size = desired_cap_height * b.StyleFontComputedSize(style) / cap_height;
        let mut description = b.StyleFontDescription(style);
        b.SetFontComputedSize(&mut description, adjusted_size);
        b.SetFontSpecifiedSize(&mut description, adjusted_size);
        while adjusted_size > 1.0 {
            let actual_font = b.NewFont(&description, &b.FontSelector(&font));
            if b.FontCapHeight(&actual_font) <= desired_cap_height {
                return actual_font;
            }
            adjusted_size -= 1.0;
            b.SetFontComputedSize(&mut description, adjusted_size);
            b.SetFontSpecifiedSize(&mut description, adjusted_size);
        }
        font
    }
    // cpp: style_resolver.cc:3757-3771; h:149-151
    pub fn StyleForInitialLetterText(
        &self,
        box_style: &B::ComputedStyle,
        paragraph: &B::ComputedStyle,
    ) -> B::ComputedStyle {
        let b = self.backend;
        let mut builder = self.CreateComputedStyleBuilderInheritingFrom(box_style);
        b.SetFont(
            &mut builder,
            self.ComputeInitialLetterFont(box_style, paragraph),
        );
        let height = b.BuilderFontHeight(&builder);
        b.SetLineHeight(&mut builder, foundation::Length::Fixed(height));
        b.SetVerticalAlignBaseline(&mut builder);
        b.SetBaseTextDecorationData(&mut builder, b.AppliedTextDecorationData(box_style));
        b.TakeStyle(builder)
    }
    // cpp: style_resolver.cc:3773-3804; h:279-280
    pub fn ResolvePositionTryRule(
        &self,
        scope: Option<&B::TreeScope>,
        name: &AtomicString,
    ) -> Option<Rc<B::StyleRulePositionTry>> {
        let b = self.backend;
        let mut current = Some(scope.unwrap_or_else(|| b.DocumentTreeScope(self.GetDocument())));
        while let Some(scope) = current {
            if let Some(resolver) = b.TreeScopeResolver(scope) {
                if let Some(rule) = b.PositionTryForName(resolver, name) {
                    return Some(rule);
                }
            }
            current = b.ParentTreeScope(scope);
        }
        for rule in b.PositionTryRules(&b.DefaultHtmlStyle()) {
            if b.PositionTryName(&rule) == name {
                return Some(rule);
            }
        }
        None
    }

    // cpp: style_resolver.cc:3395-3402,3406-3425. The source macros expand into
    // typed comparisons/setters here; changed stays under resolver control.
    fn PropagateScrollSnapStyleToViewport(
        &self,
        source: Option<&B::ComputedStyle>,
        builder: &mut B::ComputedStyleBuilder,
    ) -> bool {
        let b = self.backend;
        let mut changed = false;
        macro_rules! propagate {
            ($get:ident, $set:ident, $initial:expr) => {{
                let value = source.map_or_else(|| $initial, |style| b.$get(style));
                // Getter names for styles and builders are different at the
                // mandatory external storage boundary, but their value types match.
                value
            }};
        }
        let value = propagate!(
            ViewportStyleScrollSnapType,
            SetViewportScrollSnapType,
            b.InitialScrollSnapType()
        );
        if b.ViewportBuilderScrollSnapType(builder) != value {
            b.SetViewportScrollSnapType(builder, value);
            changed = true;
        }
        let value = source.map_or_else(foundation::Length::default, |style| {
            b.ViewportStyleScrollPaddingTop(style)
        });
        if b.ViewportBuilderScrollPaddingTop(builder) != value {
            b.SetViewportScrollPaddingTop(builder, value);
            changed = true;
        }
        let value = source.map_or_else(foundation::Length::default, |style| {
            b.ViewportStyleScrollPaddingRight(style)
        });
        if b.ViewportBuilderScrollPaddingRight(builder) != value {
            b.SetViewportScrollPaddingRight(builder, value);
            changed = true;
        }
        let value = source.map_or_else(foundation::Length::default, |style| {
            b.ViewportStyleScrollPaddingBottom(style)
        });
        if b.ViewportBuilderScrollPaddingBottom(builder) != value {
            b.SetViewportScrollPaddingBottom(builder, value);
            changed = true;
        }
        let value = source.map_or_else(foundation::Length::default, |style| {
            b.ViewportStyleScrollPaddingLeft(style)
        });
        if b.ViewportBuilderScrollPaddingLeft(builder) != value {
            b.SetViewportScrollPaddingLeft(builder, value);
            changed = true;
        }
        changed
    }
    // cpp: style_resolver.cc:3429-3449; h:284
    pub fn ShouldStopBodyPropagation(&self, element: &B::Element) -> bool {
        let Some(object) = self.backend.ElementLayoutObject(element) else {
            return true;
        };
        self.backend.ShouldApplyAnyContainment(&object)
    }
    // cpp: style_resolver.cc:3451-3706; h:155
    pub fn PropagateStyleToViewport(&self) {
        let b = self.backend;
        let root = b.DocumentElement(self.GetDocument());
        let root_style = root
            .as_deref()
            .filter(|root| b.ElementLayoutObject(root).is_some())
            .and_then(|root| b.ElementComputedStyle(root));
        let body_style = b.FirstBodyElement(self.GetDocument()).and_then(|body| {
            if !self
                .ShouldStopBodyPropagation(root.as_deref().expect("body has a document element"))
                && !self.ShouldStopBodyPropagation(&body)
            {
                b.ElementComputedStyle(&body)
            } else {
                None
            }
        });
        let root_style = root_style.as_deref();
        let body_style = body_style.as_deref();
        let viewport = b.LayoutViewStyle(b.GetLayoutView(self.GetDocument()));
        let mut builder = b.NewComputedStyleBuilder(&viewport);
        let mut changed = false;
        let mut update_scrollbar_style = false;
        macro_rules! propagate_from {
            ($source:expr, $get:ident, $builder_get:ident, $set:ident, $initial:expr) => {{
                let value = $source.map_or_else(|| $initial, |style| b.$get(style));
                if b.$builder_get(&builder) != value {
                    b.$set(&mut builder, value);
                    changed = true;
                }
            }};
        }
        macro_rules! propagate_value {
            ($value:expr, $get:ident, $set:ident) => {{
                let value = $value;
                if b.$get(&builder) != value {
                    b.$set(&mut builder, value);
                    changed = true;
                }
            }};
        }
        let direction_style = body_style.or(root_style);
        propagate_from!(
            direction_style,
            ViewportStyleWritingMode,
            ViewportBuilderWritingMode,
            SetViewportWritingMode,
            foundation::WritingMode::kHorizontalTb
        );
        propagate_from!(
            direction_style,
            ViewportStyleDirection,
            ViewportBuilderDirection,
            SetViewportDirection,
            TextDirection::kLtr
        );
        let mut background_style = root_style;
        if body_style.is_some()
            && !b.HasBackground(background_style.expect("propagated body has root style"))
        {
            background_style = body_style;
        }
        let mut background_color = foundation::Color::kTransparent;
        let mut layers = b.NewBackgroundFillLayers();
        let mut rendering = foundation::EImageRendering::kAuto;
        if let Some(style) = background_style {
            if let Some(root) = root_style.filter(|root| {
                b.StyleImageAnimation(root) != foundation::ImageAnimationEnum::kNormal
            }) {
                b.SetImageAnimation(&mut builder, b.StyleImageAnimation(root));
                changed = true;
            }
            background_color = b.VisitedDependentColor(style, CSSPropertyID::kBackgroundColor);
            layers = b.BackgroundLayers(style);
            for index in 0..b.FillLayerCount(&layers) {
                b.SetLayerClipBorder(&mut layers, index);
                if b.LayerAttachmentIsScroll(&layers, index) {
                    b.SetLayerAttachmentLocal(&mut layers, index);
                }
            }
            rendering = b.ImageRendering(style);
        }
        if b.VisitedDependentColor(&viewport, CSSPropertyID::kBackgroundColor) != background_color
            || b.BackgroundLayers(&viewport) != layers
            || b.ImageRendering(&viewport) != rendering
        {
            changed = true;
            b.SetStyleColor(
                &mut builder,
                CSSPropertyID::kBackgroundColor,
                background_color,
            );
            b.SetBackgroundLayers(&mut builder, layers);
            b.SetImageRendering(&mut builder, rendering);
        }
        if self.IsForcedColorsModeEnabled() {
            let color = root_style.map_or(foundation::Color::kTransparent, |style| {
                b.VisitedDependentColor(style, CSSPropertyID::kInternalForcedBackgroundColor)
            });
            if b.VisitedDependentColor(&viewport, CSSPropertyID::kInternalForcedBackgroundColor)
                != color
            {
                changed = true;
                b.SetStyleColor(
                    &mut builder,
                    CSSPropertyID::kInternalForcedBackgroundColor,
                    color,
                );
            }
        }
        let mut overflow_style = root_style;
        if let Some(body) = body_style {
            if b.IsOverflowVisibleAlongBothAxes(root_style.expect("propagated body has root style"))
            {
                overflow_style = Some(body);
            }
        }
        let overscroll_style = if b.PropagateOverscrollBehaviorFromRootEnabled() {
            root_style
        } else {
            overflow_style
        };
        propagate_from!(
            overscroll_style,
            ViewportStyleOverscrollBehaviorX,
            ViewportBuilderOverscrollBehaviorX,
            SetViewportOverscrollBehaviorX,
            foundation::EOverscrollBehavior::kAuto
        );
        propagate_from!(
            overscroll_style,
            ViewportStyleOverscrollBehaviorY,
            ViewportBuilderOverscrollBehaviorY,
            SetViewportOverscrollBehaviorY,
            foundation::EOverscrollBehavior::kAuto
        );
        if let Some(style) = overscroll_style {
            if b.IsInOutermostMainFrame(self.GetDocument()) {
                b.ChromeClientSetOverscrollBehavior(
                    self.GetDocument(),
                    b.ViewportStyleOverscrollBehaviorX(style),
                    b.ViewportStyleOverscrollBehaviorY(style),
                );
            }
        }
        let mut overflow_x = EOverflow::kAuto;
        let mut overflow_y = EOverflow::kAuto;
        let mut overflow_anchor = foundation::EOverflowAnchor::kAuto;
        if let Some(style) = overflow_style {
            overflow_x = b.ViewportStyleOverflowX(style);
            overflow_y = b.ViewportStyleOverflowY(style);
            overflow_anchor = b.ViewportStyleOverflowAnchor(style);
            overflow_x = match overflow_x {
                EOverflow::kVisible => EOverflow::kAuto,
                EOverflow::kClip => EOverflow::kHidden,
                value => value,
            };
            overflow_y = match overflow_y {
                EOverflow::kVisible => EOverflow::kAuto,
                EOverflow::kClip => EOverflow::kHidden,
                value => value,
            };
            if overflow_anchor == foundation::EOverflowAnchor::kVisible {
                overflow_anchor = foundation::EOverflowAnchor::kAuto;
            }
            if b.HasCustomScrollbarStyle(style, root.as_deref()) {
                update_scrollbar_style = true;
            }
        }
        propagate_value!(overflow_x, ViewportBuilderOverflowX, SetViewportOverflowX);
        propagate_value!(overflow_y, ViewportBuilderOverflowY, SetViewportOverflowY);
        propagate_value!(
            overflow_anchor,
            ViewportBuilderOverflowAnchor,
            SetViewportOverflowAnchor
        );
        let color = root_style.map_or_else(
            || b.CanvasTextColor(),
            |style| b.VisitedDependentColor(style, CSSPropertyID::kColor),
        );
        if b.VisitedDependentColor(&viewport, CSSPropertyID::kColor) != color {
            changed = true;
            b.SetStyleColor(&mut builder, CSSPropertyID::kColor, color);
        }
        propagate_from!(
            root_style,
            ViewportStyleEffectiveTouchAction,
            ViewportBuilderEffectiveTouchAction,
            SetViewportEffectiveTouchAction,
            foundation::TouchAction::kAuto
        );
        propagate_from!(
            root_style,
            ViewportStyleScrollBehavior,
            ViewportBuilderScrollBehavior,
            SetViewportScrollBehavior,
            b.AutoScrollBehavior()
        );
        propagate_from!(
            root_style,
            ViewportStyleDarkColorScheme,
            ViewportBuilderDarkColorScheme,
            SetViewportDarkColorScheme,
            false
        );
        propagate_from!(
            root_style,
            ViewportStyleColorSchemeForced,
            ViewportBuilderColorSchemeForced,
            SetViewportColorSchemeForced,
            false
        );
        propagate_from!(
            root_style,
            ViewportStyleScrollbarGutter,
            ViewportBuilderScrollbarGutter,
            SetViewportScrollbarGutter,
            b.AutoScrollbarGutter()
        );
        propagate_from!(
            root_style,
            ViewportStyleScrollbarWidth,
            ViewportBuilderScrollbarWidth,
            SetViewportScrollbarWidth,
            foundation::EScrollbarWidth::kAuto
        );
        // ScrollbarColor is a pointer-valued getter in the source macro.
        // Preserve its identity comparison, including nullptr.
        let scrollbar_color = root_style.and_then(|style| b.ViewportStyleScrollbarColor(style));
        let old_scrollbar_color = b.ViewportBuilderScrollbarColor(&builder);
        let color_unchanged = match (&old_scrollbar_color, &scrollbar_color) {
            (None, None) => true,
            (Some(old), Some(new)) => Rc::ptr_eq(old, new),
            _ => false,
        };
        if !color_unchanged {
            b.SetViewportScrollbarColor(&mut builder, scrollbar_color);
            changed = true;
        }
        propagate_from!(
            root_style,
            ViewportStyleForcedColorAdjust,
            ViewportBuilderForcedColorAdjust,
            SetViewportForcedColorAdjust,
            foundation::EForcedColorAdjust::kAuto
        );
        propagate_from!(
            root_style,
            ViewportStyleColorSchemeFlagsIsNormal,
            ViewportBuilderColorSchemeFlagsIsNormal,
            SetViewportColorSchemeFlagsIsNormal,
            false
        );
        changed |= self.PropagateScrollSnapStyleToViewport(root_style, &mut builder);
        if changed {
            b.UpdateFontOrientation(&mut builder);
            b.CreateInitialFont(self.GetDocument(), &mut builder);
        }
        if changed || update_scrollbar_style {
            b.SetLayoutViewStyle(b.GetLayoutView(self.GetDocument()), b.TakeStyle(builder));
        }
    }
}
