/*
 * CSS Media Query Evaluator
 *
 * Copyright (C) 2006 Kimmo Kinnunen <kimmo.t.kinnunen@nokia.com>.
 * Copyright (C) 2013 Apple Inc. All rights reserved.
 * Copyright (C) 2013 Intel Corporation. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE AUTHOR ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE COMPUTER, INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
use super::color_space_gamut::ColorSpaceGamut;
use super::forced_colors::ForcedColors;
use super::media_query_exp::{MediaQueryExpValue, MediaQueryOperator};
use super::media_values::MediaValues;
use super::navigation_controls::NavigationControls;
use super::preferred_color_scheme::PreferredColorScheme;
use super::preferred_contrast::PreferredContrast;
use super::web_preferences::{HoverType, PointerType};
use foundation::{CSSValueID, LayoutUnit};

// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Ledger uses C++ physical / effective / mapped / omitted / pending lines.
// Effective excludes comments, blanks, preprocessor/includes, namespaces and
// lines consisting only of braces/parentheses/semicolons.
// media_query_evaluator.cc: 1964 / 1257 / 1211 / 46 / 0 (complete).
// media_query_evaluator.h: 154 / 61 / 43 / 18 / 0.
// Production document/ancestor owners, computed variable reads, unparsed typed
// downcasts, native font/line-height conversion contexts, position-area and
// fallback conversion/matching adapters live in production_container_query.rs
// and document_style_engine.rs. Layout/scroll/anchor snapshots are explicitly
// published by their owner and invalidate query matches; absent state is Unknown.
// Numeric math coercion is implemented by ContainerNumericContext through the
// shared CSSMathExpressionNode parser and records its conversion flags.
// PropertyRegistry implements RegisteredContainerPropertyResolver with the
// document's real syntax, initial value and computed registered-value storage.
// Omitted .cc:109,120-124 and UseCounter statements (42 effective lines):
// global table storage, GC and telemetry only, never observable matching.
use super::conditional_exp_node::{ConditionalExpNode, ConditionalExpNodeVisitor};
use super::media_query::{MediaQuery, RestrictorType};
use super::media_query_exp::MediaQueryExp;
use super::media_query_set::MediaQuerySet;
use crate::kleene_value::{KleeneNot, KleeneOr, KleeneValue};
use foundation::String;

fn EqualIgnoringAsciiCase(a: &String, b: &String) -> bool {
    fn lower(c: u16) -> u16 {
        if (65..=90).contains(&c) {
            c + 32
        } else {
            c
        }
    }
    a.length() == b.length()
        && (0..a.length()).all(|i| lower(a.CodeUnitAt(i)) == lower(b.CodeUnitAt(i)))
}

pub trait MediaQueryEvaluationBackend {
    type Document;
    type Value;
    type UnparsedValue;
    type ResultFlags;
    type CustomMediaRulesMap;
    fn EvalFeature(
        &self,
        feature: &MediaQueryExp<Self::Value, Self::UnparsedValue>,
        flags: Option<&mut Self::ResultFlags>,
        custom_medias: Option<&Self::CustomMediaRulesMap>,
    ) -> KleeneValue;
}
/// Frame ownership supplies the dynamic MediaValues reader; no synthetic
/// device values are created when the frame is absent.
pub trait MediaQueryFrameValues {
    type Document;
    fn CreateDynamicIfFrameExists(&self) -> Option<&dyn MediaValues<Document = Self::Document>>;
}
pub struct MediaQueryEvaluator<'a, B: MediaQueryEvaluationBackend> {
    media_type_: String,
    media_values_: Option<&'a dyn MediaValues<Document = B::Document>>,
    backend: B,
}
// cpp: media_query_evaluator.cc:91-97
fn ApplyRestrictor(restrictor: RestrictorType, value: KleeneValue) -> KleeneValue {
    if restrictor == RestrictorType::kNot {
        KleeneNot(value)
    } else {
        value
    }
}
impl<'a, B: MediaQueryEvaluationBackend> MediaQueryEvaluator<'a, B> {
    // cpp: media_query_evaluator.cc:111-112,117-118
    pub fn FromMediaType(media_type: String, backend: B) -> Self {
        Self {
            media_type_: media_type,
            media_values_: None,
            backend,
        }
    }
    pub fn FromMediaValues(
        values: &'a dyn MediaValues<Document = B::Document>,
        backend: B,
    ) -> Self {
        Self {
            media_type_: String::default(),
            media_values_: Some(values),
            backend,
        }
    }
    // cpp: media_query_evaluator.cc:114-115,126-132.
    pub fn FromFrame(
        frame: &'a impl MediaQueryFrameValues<Document = B::Document>,
        backend: B,
    ) -> Self {
        Self {
            media_type_: String::default(),
            media_values_: frame.CreateDynamicIfFrameExists(),
            backend,
        }
    }
    pub fn GetDocument(&self) -> Option<&B::Document> {
        self.media_values_.and_then(MediaValues::GetDocument)
    }
    pub fn GetMediaValues(&self) -> &dyn MediaValues<Document = B::Document> {
        self.media_values_
            .expect("feature evaluator requires MediaValues")
    }
    // cpp: media_query_evaluator.cc:1717-1741. Custom media retain Unknown
    // across disjunctions; the public bool set overload only admits true.
    pub fn EvalCustomMediaSet(
        &self,
        set: &MediaQuerySet<B::Value, B::UnparsedValue>,
        mut flags: Option<&mut B::ResultFlags>,
        custom_medias: Option<&B::CustomMediaRulesMap>,
    ) -> KleeneValue {
        if set.QueryVector().is_empty() {
            return KleeneValue::kTrue;
        }
        let mut result = KleeneValue::kFalse;
        for query in set.QueryVector() {
            if result == KleeneValue::kTrue {
                break;
            }
            result = KleeneOr(
                result,
                self.EvalQuery(query, flags.as_deref_mut(), custom_medias),
            );
        }
        result
    }
    // cpp: media_query_evaluator.cc:133-150
    pub fn MediaType(&self) -> String {
        if !self.media_type_.empty() {
            return self.media_type_.clone();
        }
        if let Some(values) = self.media_values_ {
            return values.MediaType();
        }
        String::default()
    }
    pub fn MediaTypeMatch(&self, media_type: &String) -> bool {
        media_type.empty()
            || EqualIgnoringAsciiCase(media_type, &String::from("all"))
            || EqualIgnoringAsciiCase(media_type, &self.MediaType())
    }
    // cpp: media_query_evaluator.cc:152-174
    pub fn EvalQuery(
        &self,
        query: &MediaQuery<B::Value, B::UnparsedValue>,
        flags: Option<&mut B::ResultFlags>,
        custom_medias: Option<&B::CustomMediaRulesMap>,
    ) -> KleeneValue {
        if !self.MediaTypeMatch(query.MediaType()) {
            return ApplyRestrictor(query.Restrictor(), KleeneValue::kFalse);
        }
        let Some(node) = query.ExpNode() else {
            return ApplyRestrictor(query.Restrictor(), KleeneValue::kTrue);
        };
        ApplyRestrictor(
            query.Restrictor(),
            self.EvalNode(node, flags, custom_medias),
        )
    }
    // cpp: media_query_evaluator.cc:176-197
    pub fn Eval(&self, set: &MediaQuerySet<B::Value, B::UnparsedValue>) -> bool {
        self.EvalSet(set, None, None)
    }
    pub fn EvalSet(
        &self,
        set: &MediaQuerySet<B::Value, B::UnparsedValue>,
        mut flags: Option<&mut B::ResultFlags>,
        custom_medias: Option<&B::CustomMediaRulesMap>,
    ) -> bool {
        if set.QueryVector().is_empty() {
            return true;
        }
        let mut result = KleeneValue::kFalse;
        for query in set.QueryVector() {
            if result == KleeneValue::kTrue {
                break;
            }
            result = self.EvalQuery(query, flags.as_deref_mut(), custom_medias);
        }
        result == KleeneValue::kTrue
    }
    // cpp: media_query_evaluator.cc:199-230
    pub fn EvalNode(
        &self,
        node: &ConditionalExpNode<B::Value, B::UnparsedValue>,
        flags: Option<&mut B::ResultFlags>,
        custom_medias: Option<&B::CustomMediaRulesMap>,
    ) -> KleeneValue {
        struct Handler<'a, B: MediaQueryEvaluationBackend> {
            backend: &'a B,
            flags: Option<&'a mut B::ResultFlags>,
            custom_medias: Option<&'a B::CustomMediaRulesMap>,
        }
        impl<B: MediaQueryEvaluationBackend> ConditionalExpNodeVisitor<B::Value, B::UnparsedValue>
            for Handler<'_, B>
        {
            fn EvaluateMediaQueryFeatureExpNode(
                &mut self,
                feature: &MediaQueryExp<B::Value, B::UnparsedValue>,
            ) -> KleeneValue {
                self.backend
                    .EvalFeature(feature, self.flags.as_deref_mut(), self.custom_medias)
            }
        }
        node.Evaluate(&mut Handler {
            backend: &self.backend,
            flags,
            custom_medias,
        })
    }
    // cpp: media_query_evaluator.cc:232-250; the two source collections share this traversal.
    pub fn DidResultsChange<'b>(
        &self,
        results: impl IntoIterator<Item = (&'b MediaQuerySet<B::Value, B::UnparsedValue>, bool)>,
    ) -> bool
    where
        B::Value: 'b,
        B::UnparsedValue: 'b,
    {
        results
            .into_iter()
            .any(|(queries, result)| result != self.Eval(queries))
    }
}

// Source comparison/preference functions share typed production dispatch with
// media_query_backend.rs. UseCounter statements are explicit ledger omissions;
// concrete container ownership/coercion remains in the pending ranges above.

// cpp: third_party/blink/renderer/core/css/media_query_evaluator.cc:253-265
pub const fn HandleNegativeMediaFeatureValue(op: MediaQueryOperator) -> bool {
    matches!(op, MediaQueryOperator::kGt | MediaQueryOperator::kGe)
}
// cpp: third_party/blink/renderer/core/css/media_query_evaluator.cc:267-286
pub fn CompareValue<T: Copy + PartialOrd + From<u8>>(
    actual_value: T,
    query_value: T,
    op: MediaQueryOperator,
) -> bool {
    if query_value < T::from(0) {
        return HandleNegativeMediaFeatureValue(op);
    }
    match op {
        MediaQueryOperator::kGe => actual_value >= query_value,
        MediaQueryOperator::kLe => actual_value <= query_value,
        MediaQueryOperator::kEq | MediaQueryOperator::kNone => actual_value == query_value,
        MediaQueryOperator::kLt => actual_value < query_value,
        MediaQueryOperator::kGt => actual_value > query_value,
    }
}
// cpp: third_party/blink/renderer/core/css/media_query_evaluator.cc:288-309
pub fn CompareDoubleValue(actual_value: f64, query_value: f64, op: MediaQueryOperator) -> bool {
    if query_value < 0.0 {
        return HandleNegativeMediaFeatureValue(op);
    }
    let precision = f64::from(LayoutUnit::Epsilon());
    match op {
        MediaQueryOperator::kGe => actual_value >= query_value - precision,
        MediaQueryOperator::kLe => actual_value <= query_value + precision,
        MediaQueryOperator::kEq | MediaQueryOperator::kNone => {
            (actual_value - query_value).abs() <= precision
        }
        MediaQueryOperator::kLt => actual_value < query_value,
        MediaQueryOperator::kGt => actual_value > query_value,
    }
}
// cpp: third_party/blink/renderer/core/css/media_query_evaluator.cc:1608-1624
pub const fn ReverseOperator(op: MediaQueryOperator) -> MediaQueryOperator {
    match op {
        MediaQueryOperator::kNone | MediaQueryOperator::kEq => op,
        MediaQueryOperator::kLt => MediaQueryOperator::kGt,
        MediaQueryOperator::kLe => MediaQueryOperator::kGe,
        MediaQueryOperator::kGt => MediaQueryOperator::kLt,
        MediaQueryOperator::kGe => MediaQueryOperator::kLe,
    }
}

// cpp: third_party/blink/renderer/core/css/media_query_evaluator.cc:930-946
pub fn HoverMediaFeatureEval<V>(
    value: &MediaQueryExpValue<V>,
    _op: MediaQueryOperator,
    media_values: &(impl MediaValues + ?Sized),
) -> bool {
    let hover = media_values.PrimaryHoverType();
    if !value.IsValid() {
        return hover != HoverType::kHoverNone;
    }
    if !value.IsId() {
        return false;
    }
    (hover == HoverType::kHoverNone && value.Id() == CSSValueID::kNone)
        || (hover == HoverType::kHoverHoverType && value.Id() == CSSValueID::kHover)
}
// cpp: third_party/blink/renderer/core/css/media_query_evaluator.cc:948-970
pub fn AnyHoverMediaFeatureEval<V>(
    value: &MediaQueryExpValue<V>,
    _op: MediaQueryOperator,
    media_values: &(impl MediaValues + ?Sized),
) -> bool {
    let available_hover_types = media_values.AvailableHoverTypes();
    if !value.IsValid() {
        return available_hover_types & !(HoverType::kHoverNone as i32) != 0;
    }
    if !value.IsId() {
        return false;
    }
    match value.Id() {
        CSSValueID::kNone => available_hover_types & HoverType::kHoverNone as i32 != 0,
        CSSValueID::kHover => available_hover_types & HoverType::kHoverHoverType as i32 != 0,
        _ => unreachable!("invalid any-hover identifier"),
    }
}
// cpp: third_party/blink/renderer/core/css/media_query_evaluator.cc:972-979
pub fn OriginTrialTestMediaFeatureEval<V>(
    value: &MediaQueryExpValue<V>,
    _op: MediaQueryOperator,
    _media_values: &(impl MediaValues + ?Sized),
) -> bool {
    debug_assert!(!value.IsValid());
    true
}
// cpp: third_party/blink/renderer/core/css/media_query_evaluator.cc:981-1000
pub fn PointerMediaFeatureEval<V>(
    value: &MediaQueryExpValue<V>,
    _op: MediaQueryOperator,
    media_values: &(impl MediaValues + ?Sized),
) -> bool {
    let pointer = media_values.PrimaryPointerType();
    if !value.IsValid() {
        return pointer != PointerType::kPointerNone;
    }
    if !value.IsId() {
        return false;
    }
    (pointer == PointerType::kPointerNone && value.Id() == CSSValueID::kNone)
        || (pointer == PointerType::kPointerCoarseType && value.Id() == CSSValueID::kCoarse)
        || (pointer == PointerType::kPointerFineType && value.Id() == CSSValueID::kFine)
}
// cpp: third_party/blink/renderer/core/css/media_query_evaluator.cc:1010-1021
// Omitted source lines 1006-1007: UseCounter telemetry.
pub fn PrefersReducedMotionMediaFeatureEval<V>(
    value: &MediaQueryExpValue<V>,
    _op: MediaQueryOperator,
    media_values: &(impl MediaValues + ?Sized),
) -> bool {
    if !value.IsValid() {
        return media_values.PrefersReducedMotion();
    }
    if !value.IsId() {
        return false;
    }
    (value.Id() == CSSValueID::kNoPreference) ^ media_values.PrefersReducedMotion()
}
// cpp: third_party/blink/renderer/core/css/media_query_evaluator.cc:1030-1040
// Omitted source lines 1027-1028: UseCounter telemetry.
pub fn PrefersReducedDataMediaFeatureEval<V>(
    value: &MediaQueryExpValue<V>,
    _op: MediaQueryOperator,
    media_values: &(impl MediaValues + ?Sized),
) -> bool {
    if !value.IsValid() {
        return media_values.PrefersReducedData();
    }
    if !value.IsId() {
        return false;
    }
    (value.Id() == CSSValueID::kNoPreference) ^ media_values.PrefersReducedData()
}
// cpp: third_party/blink/renderer/core/css/media_query_evaluator.cc:1049-1059
// Omitted source lines 1046-1047: UseCounter telemetry.
pub fn PrefersReducedTransparencyMediaFeatureEval<V>(
    value: &MediaQueryExpValue<V>,
    _op: MediaQueryOperator,
    media_values: &(impl MediaValues + ?Sized),
) -> bool {
    if !value.IsValid() {
        return media_values.PrefersReducedTransparency();
    }
    if !value.IsId() {
        return false;
    }
    (value.Id() == CSSValueID::kNoPreference) ^ media_values.PrefersReducedTransparency()
}
// cpp: third_party/blink/renderer/core/css/media_query_evaluator.cc:1061-1086
pub fn AnyPointerMediaFeatureEval<V>(
    value: &MediaQueryExpValue<V>,
    _op: MediaQueryOperator,
    media_values: &(impl MediaValues + ?Sized),
) -> bool {
    let available_pointers = media_values.AvailablePointerTypes();
    if !value.IsValid() {
        return available_pointers & !(PointerType::kPointerNone as i32) != 0;
    }
    if !value.IsId() {
        return false;
    }
    match value.Id() {
        CSSValueID::kCoarse => available_pointers & PointerType::kPointerCoarseType as i32 != 0,
        CSSValueID::kFine => available_pointers & PointerType::kPointerFineType as i32 != 0,
        CSSValueID::kNone => available_pointers & PointerType::kPointerNone as i32 != 0,
        _ => unreachable!("invalid any-pointer identifier"),
    }
}
// cpp: third_party/blink/renderer/core/css/media_query_evaluator.cc:1111-1141
pub fn ColorGamutMediaFeatureEval<V>(
    value: &MediaQueryExpValue<V>,
    _op: MediaQueryOperator,
    media_values: &(impl MediaValues + ?Sized),
) -> bool {
    if !value.IsValid() {
        return true;
    }
    if !value.IsId() {
        return false;
    }
    debug_assert!(matches!(
        value.Id(),
        CSSValueID::kSRGB | CSSValueID::kP3 | CSSValueID::kRec2020
    ));
    match media_values.ColorGamut() {
        ColorSpaceGamut::SRGB => value.Id() == CSSValueID::kSRGB,
        ColorSpaceGamut::P3 => value.Id() == CSSValueID::kSRGB || value.Id() == CSSValueID::kP3,
        ColorSpaceGamut::BT2020 => {
            value.Id() == CSSValueID::kSRGB
                || value.Id() == CSSValueID::kP3
                || value.Id() == CSSValueID::kRec2020
        }
    }
}
// cpp: third_party/blink/renderer/core/css/media_query_evaluator.cc:1150-1164
// Omitted source lines 1147-1148: UseCounter telemetry.
pub fn PrefersColorSchemeMediaFeatureEval<V>(
    value: &MediaQueryExpValue<V>,
    _op: MediaQueryOperator,
    media_values: &(impl MediaValues + ?Sized),
) -> bool {
    let preferred_scheme = media_values.GetPreferredColorScheme();
    if !value.IsValid() {
        return true;
    }
    if !value.IsId() {
        return false;
    }
    (preferred_scheme == PreferredColorScheme::kDark && value.Id() == CSSValueID::kDark)
        || (preferred_scheme == PreferredColorScheme::kLight && value.Id() == CSSValueID::kLight)
}
// cpp: third_party/blink/renderer/core/css/media_query_evaluator.cc:1172-1195
// Omitted source lines 1169-1170: UseCounter telemetry.
pub fn PrefersContrastMediaFeatureEval<V>(
    value: &MediaQueryExpValue<V>,
    _op: MediaQueryOperator,
    media_values: &(impl MediaValues + ?Sized),
) -> bool {
    let preferred_contrast = media_values.GetPreferredContrast();
    if !value.IsValid() {
        return preferred_contrast != PreferredContrast::kNoPreference;
    }
    if !value.IsId() {
        return false;
    }
    match value.Id() {
        CSSValueID::kMore => preferred_contrast == PreferredContrast::kMore,
        CSSValueID::kLess => preferred_contrast == PreferredContrast::kLess,
        CSSValueID::kNoPreference => preferred_contrast == PreferredContrast::kNoPreference,
        CSSValueID::kCustom => preferred_contrast == PreferredContrast::kCustom,
        _ => unreachable!("invalid preferred-contrast identifier"),
    }
}
// cpp: third_party/blink/renderer/core/css/media_query_evaluator.cc:1203-1218
// Omitted source lines 1200-1201: UseCounter telemetry.
pub fn ForcedColorsMediaFeatureEval<V>(
    value: &MediaQueryExpValue<V>,
    _op: MediaQueryOperator,
    media_values: &(impl MediaValues + ?Sized),
) -> bool {
    let forced_colors = media_values.GetForcedColors();
    if !value.IsValid() {
        return forced_colors != ForcedColors::kNone;
    }
    if !value.IsId() {
        return false;
    }
    (forced_colors == ForcedColors::kNone && value.Id() == CSSValueID::kNone)
        || (forced_colors != ForcedColors::kNone && value.Id() == CSSValueID::kActive)
}
// cpp: third_party/blink/renderer/core/css/media_query_evaluator.cc:1220-1239
pub fn NavigationControlsMediaFeatureEval<V>(
    value: &MediaQueryExpValue<V>,
    _op: MediaQueryOperator,
    media_values: &(impl MediaValues + ?Sized),
) -> bool {
    let navigation_controls = media_values.GetNavigationControls();
    if !value.IsValid() {
        return navigation_controls != NavigationControls::kNone;
    }
    if !value.IsId() {
        return false;
    }
    (navigation_controls == NavigationControls::kNone && value.Id() == CSSValueID::kNone)
        || (navigation_controls == NavigationControls::kBackButton
            && value.Id() == CSSValueID::kBackButton)
}

// cpp: media_query_evaluator.cc:1386-1523.
pub fn StuckMediaFeatureEval<V>(
    value: &MediaQueryExpValue<V>,
    _op: MediaQueryOperator,
    values: &(impl MediaValues + ?Sized),
) -> bool {
    use super::container_state::{
        ContainerStuckLogical as Logical, ContainerStuckPhysical as Physical,
    };
    use CSSValueID::*;
    if !value.IsValid() {
        return values.Stuck();
    }
    match value.Id() {
        kNone => {
            values.StuckHorizontal() == Physical::kNo && values.StuckVertical() == Physical::kNo
        }
        kTop => values.StuckVertical() == Physical::kTop,
        kLeft => values.StuckHorizontal() == Physical::kLeft,
        kBottom => values.StuckVertical() == Physical::kBottom,
        kRight => values.StuckHorizontal() == Physical::kRight,
        kBlockStart => values.StuckBlock() == Logical::kStart,
        kBlockEnd => values.StuckBlock() == Logical::kEnd,
        kInlineStart => values.StuckInline() == Logical::kStart,
        kInlineEnd => values.StuckInline() == Logical::kEnd,
        _ => unreachable!("invalid stuck identifier"),
    }
}
pub fn SnappedMediaFeatureEval<V>(
    value: &MediaQueryExpValue<V>,
    _op: MediaQueryOperator,
    values: &(impl MediaValues + ?Sized),
) -> bool {
    use CSSValueID::*;
    if !value.IsValid() {
        return values.Snapped();
    }
    match value.Id() {
        kNone => !values.Snapped(),
        kX => values.SnappedX(),
        kY => values.SnappedY(),
        kBlock => values.SnappedBlock(),
        kInline => values.SnappedInline(),
        kBoth => values.SnappedX() && values.SnappedY(),
        _ => unreachable!("invalid snapped identifier"),
    }
}
pub fn ScrollableMediaFeatureEval<V>(
    value: &MediaQueryExpValue<V>,
    _op: MediaQueryOperator,
    values: &(impl MediaValues + ?Sized),
) -> bool {
    use super::container_state::ContainerScrollable;
    use CSSValueID::*;
    if !value.IsValid() {
        return values.Scrollable();
    }
    let start = ContainerScrollable::kStart as u32;
    let end = ContainerScrollable::kEnd as u32;
    match value.Id() {
        kNone => !values.Scrollable(),
        kTop => values.ScrollableVertical() & start != 0,
        kLeft => values.ScrollableHorizontal() & start != 0,
        kBottom => values.ScrollableVertical() & end != 0,
        kRight => values.ScrollableHorizontal() & end != 0,
        kBlockStart => values.ScrollableBlock() & start != 0,
        kBlockEnd => values.ScrollableBlock() & end != 0,
        kInlineStart => values.ScrollableInline() & start != 0,
        kInlineEnd => values.ScrollableInline() & end != 0,
        kX => values.ScrollableHorizontal() != 0,
        kY => values.ScrollableVertical() != 0,
        kBlock => values.ScrollableBlock() != 0,
        kInline => values.ScrollableInline() != 0,
        _ => unreachable!("invalid scrollable identifier"),
    }
}
pub fn ScrolledMediaFeatureEval<V>(
    value: &MediaQueryExpValue<V>,
    _op: MediaQueryOperator,
    values: &(impl MediaValues + ?Sized),
) -> bool {
    use super::container_state::ContainerScrolled;
    use CSSValueID::*;
    if !value.IsValid() {
        return values.Scrolled();
    }
    match value.Id() {
        kNone => !values.Scrolled(),
        kTop => values.ScrolledVertical() == ContainerScrolled::kStart,
        kLeft => values.ScrolledHorizontal() == ContainerScrolled::kStart,
        kBottom => values.ScrolledVertical() == ContainerScrolled::kEnd,
        kRight => values.ScrolledHorizontal() == ContainerScrolled::kEnd,
        kBlockStart => values.ScrolledBlock() == ContainerScrolled::kStart,
        kBlockEnd => values.ScrolledBlock() == ContainerScrolled::kEnd,
        kInlineStart => values.ScrolledInline() == ContainerScrolled::kStart,
        kInlineEnd => values.ScrolledInline() == ContainerScrolled::kEnd,
        kX => values.ScrolledHorizontal() != ContainerScrolled::kNone,
        kY => values.ScrolledVertical() != ContainerScrolled::kNone,
        kBlock => values.ScrolledBlock() != ContainerScrolled::kNone,
        kInline => values.ScrolledInline() != ContainerScrolled::kNone,
        _ => unreachable!("invalid scrolled identifier"),
    }
}

// cpp: media_query_evaluator.cc:83-88,1928-1962. Numeric coercion belongs to
// StyleCascade; this comparison receives its already resolved literal values.
pub fn EvalStyleRangeValues(
    reference: &crate::production_css_value::Value,
    query: &crate::production_css_value::Value,
    op: MediaQueryOperator,
    reverse_op: bool,
) -> KleeneValue {
    use crate::css_value::CSSValuePayload;
    match (reference.Payload(), query.Payload()) {
        (
            CSSValuePayload::kNumericLiteralClass(reference),
            CSSValuePayload::kNumericLiteralClass(query),
        ) => EvalStyleRange(reference, query, op, reverse_op),
        _ => KleeneValue::kFalse,
    }
}
pub fn EvalStyleRange(
    reference: &crate::css_numeric_literal_value::CSSNumericLiteralValue,
    query: &crate::css_numeric_literal_value::CSSNumericLiteralValue,
    mut op: MediaQueryOperator,
    reverse_op: bool,
) -> KleeneValue {
    use crate::css_numeric_literal_value::CSSNumericLiteralValue;
    use crate::css_primitive_value::UnitType::*;
    let mut reference = reference.clone();
    let mut query = query.clone();
    if reference.IsNumber() && reference.DoubleValue() == 0.0 && query.IsLength() {
        reference = CSSNumericLiteralValue::Create(0.0, query.GetType());
    }
    if query.IsNumber() && query.DoubleValue() == 0.0 && reference.IsLength() {
        query = CSSNumericLiteralValue::Create(0.0, reference.GetType());
    }
    let percentage = |v: &CSSNumericLiteralValue| v.GetType() == kPercentage;
    let angle = |v: &CSSNumericLiteralValue| {
        matches!(v.GetType(), kDegrees | kRadians | kGradians | kTurns)
    };
    let time = |v: &CSSNumericLiteralValue| matches!(v.GetType(), kMilliseconds | kSeconds);
    let types_match = reference.IsNumber() && query.IsNumber()
        || percentage(&reference) && percentage(&query)
        || reference.IsLength() && query.IsLength()
        || angle(&reference) && angle(&query)
        || time(&reference) && time(&query)
        || reference.IsResolution() && query.IsResolution();
    if !types_match {
        return KleeneValue::kFalse;
    }
    if reverse_op {
        op = ReverseOperator(op);
    }
    if CompareDoubleValue(reference.DoubleValue(), query.DoubleValue(), op) {
        KleeneValue::kTrue
    } else {
        KleeneValue::kFalse
    }
}
