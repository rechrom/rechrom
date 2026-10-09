/*
 * CSS Media Query
 *
 * Copyright (C) 2006 Kimmo Kinnunen <kimmo.t.kinnunen@nokia.com>.
 * Copyright (C) 2010 Nokia Corporation and/or its subsidiary(-ies).
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
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Ledger uses C++ physical / effective / mapped / omitted / pending lines.
// Effective excludes comments, blanks, preprocessor/includes, namespaces and
// lines consisting only of braces/parentheses/semicolons.
// media_query_exp.cc: 834 / 535 / 167 / 10 / 358 (cumulative, partial file).
// Mapped: 373-488,610-638,656-699,712-731,820-827 (latter AST dispatch
// in conditional_exp_node.rs; plain factory in parser/media_query_parser.rs).
// Omitted: 54-56,608,630,649-654,829-832 (using/namespace/default dtor/GC).
// Pending: 58-371,491-606,640-647,701-710,733-819: value grammar/type
// dispatch, style-range factory, unit flags and feature dependency wrappers.
// Header runtime still pending: 79-95,102-110,127-141,156,179-183,
// 288-289,327,350-385; this batch maps serialization declarations 158,324.
// This batch adds .cc:662-699,712-731 serialization. CSSValue::CssText is
// required from its real owning backend; the numeric/value grammar stays
// in the precise pending ranges above.
use foundation::CSSValueID;
use std::rc::Rc;

// CSSValue serialization ownership boundary; implementations must call the
// supplied CSS object's source-aligned CssText, without substitute formatting.
pub trait MediaQueryExpSerialization {
    fn CssText(&self) -> foundation::String;
}

impl<V: MediaQueryExpSerialization> MediaQueryExpValue<V> {
    // cpp: third_party/blink/renderer/core/css/media_query_exp.cc:712-731
    pub fn CssText(&self) -> foundation::String {
        match self {
            Self::Invalid => foundation::String::from(""),
            Self::Id(id) => {
                foundation::String::from(crate::css_value_keywords::GetCSSValueName(*id))
            }
            Self::Value(value) => value.CssText(),
            Self::Ratio(ratio) => {
                let mut output = ratio.0.CssText();
                output.push_str(" / ");
                output.push_string(&ratio.1.CssText());
                output
            }
        }
    }
}

impl<V: MediaQueryExpSerialization, U: MediaQueryExpSerialization> MediaQueryExp<V, U> {
    // cpp: third_party/blink/renderer/core/css/media_query_exp.cc:662-699
    pub fn Serialize(&self) -> foundation::String {
        let mut result = foundation::String::from("");
        let bounds = &self.bounds_;
        if !bounds.IsRange() {
            if self.HasMediaFeature() || self.IsCustomMedia() {
                let name =
                    foundation::String::from_utf16(self.media_feature_.utf16_units().unwrap());
                result.push_string(&crate::css_markup::SerializeIdentifier(&name, false));
            } else {
                result.push_string(&self.ReferenceValue().CssText());
            }
            if bounds.right.IsValid() {
                debug_assert!(!self.IsCustomMedia());
                result.push_str(": ");
                result.push_string(&bounds.right.value.CssText());
            }
        } else {
            debug_assert!(!self.IsCustomMedia());
            if bounds.left.IsValid() {
                result.push_string(&bounds.left.value.CssText());
                result.push_str(" ");
                result.push_str(MediaQueryOperatorToString(bounds.left.op));
                result.push_str(" ");
            }
            if self.HasMediaFeature() {
                let name =
                    foundation::String::from_utf16(self.media_feature_.utf16_units().unwrap());
                result.push_string(&crate::css_markup::SerializeIdentifier(&name, false));
            } else {
                result.push_string(&self.ReferenceValue().CssText());
            }
            if bounds.right.IsValid() {
                result.push_str(" ");
                result.push_str(MediaQueryOperatorToString(bounds.right.op));
                result.push_str(" ");
                result.push_string(&bounds.right.value.CssText());
            }
        }
        result
    }
}

// cpp: third_party/blink/renderer/core/css/media_query_exp.h:54-68,186-193
// V is the existing CSSValue object at the ownership boundary. This module
// ports its tagged storage without introducing a substitute CSSValue parser.
pub enum MediaQueryExpValue<V> {
    Invalid,
    Id(CSSValueID),
    Value(Rc<V>),
    Ratio(Rc<(Rc<V>, Rc<V>)>),
}
impl<V> Default for MediaQueryExpValue<V> {
    fn default() -> Self {
        Self::Invalid
    }
}
impl<V> Clone for MediaQueryExpValue<V> {
    fn clone(&self) -> Self {
        match self {
            Self::Invalid => Self::Invalid,
            Self::Id(id) => Self::Id(*id),
            Self::Value(value) => Self::Value(value.clone()),
            Self::Ratio(ratio) => Self::Ratio(ratio.clone()),
        }
    }
}
impl<V> MediaQueryExpValue<V> {
    // cpp: third_party/blink/renderer/core/css/media_query_exp.h:61-68
    pub fn FromId(id: CSSValueID) -> Self {
        Self::Id(id)
    }
    pub fn FromValue(value: Rc<V>) -> Self {
        Self::Value(value)
    }
    pub fn FromRatio(numerator: Rc<V>, denominator: Rc<V>) -> Self {
        Self::Ratio(Rc::new((numerator, denominator)))
    }
    // cpp: third_party/blink/renderer/core/css/media_query_exp.h:74-77
    pub fn IsValid(&self) -> bool {
        !matches!(self, Self::Invalid)
    }
    pub fn IsId(&self) -> bool {
        matches!(self, Self::Id(_))
    }
    pub fn IsRatio(&self) -> bool {
        matches!(self, Self::Ratio(_))
    }
    pub fn IsValue(&self) -> bool {
        matches!(self, Self::Value(_))
    }
    // cpp: third_party/blink/renderer/core/css/media_query_exp.h:97-100
    pub fn Id(&self) -> CSSValueID {
        debug_assert!(self.IsId());
        match self {
            Self::Id(id) => *id,
            _ => unreachable!("not an identifier value"),
        }
    }
    // cpp: third_party/blink/renderer/core/css/media_query_exp.h:112-125
    pub fn GetCSSValue(&self) -> &V {
        debug_assert!(self.IsValue());
        match self {
            Self::Value(value) => value,
            _ => unreachable!("not a CSS value"),
        }
    }
    pub fn Numerator(&self) -> &V {
        debug_assert!(self.IsRatio());
        match self {
            Self::Ratio(ratio) => &ratio.0,
            _ => unreachable!("not a ratio value"),
        }
    }
    pub fn Denominator(&self) -> &V {
        debug_assert!(self.IsRatio());
        match self {
            Self::Ratio(ratio) => &ratio.1,
            _ => unreachable!("not a ratio value"),
        }
    }
    // cpp: third_party/blink/renderer/core/css/media_query_exp.h:143-154
    pub const kNone: u32 = 0;
    pub const kFontRelative: u32 = 1 << 0;
    pub const kRootRelative: u32 = 1 << 1;
    pub const kDynamicViewport: u32 = 1 << 2;
    pub const kStaticViewport: u32 = 1 << 3;
    pub const kContainer: u32 = 1 << 4;
    pub const kTreeCounting: u32 = 1 << 5;
    pub const kLineHeightRelative: u32 = 1 << 6;
    pub const kUnitFlagsBits: u32 = 7;
}
fn ValuesEquivalent<V: PartialEq>(a: &Rc<V>, b: &Rc<V>) -> bool {
    Rc::ptr_eq(a, b) || **a == **b
}
impl<V: PartialEq> PartialEq for MediaQueryExpValue<V> {
    // cpp: third_party/blink/renderer/core/css/media_query_exp.h:159-173
    // Ratio equality dependency: css_ratio_value.cc:24-27.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Invalid, Self::Invalid) => true,
            (Self::Id(a), Self::Id(b)) => a == b,
            (Self::Value(a), Self::Value(b)) => ValuesEquivalent(a, b),
            (Self::Ratio(a), Self::Ratio(b)) => {
                Rc::ptr_eq(a, b) || (ValuesEquivalent(&a.0, &b.0) && ValuesEquivalent(&a.1, &b.1))
            }
            _ => false,
        }
    }
}

// cpp: third_party/blink/renderer/core/css/media_query_exp.h:196-206
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MediaQueryOperator {
    #[default]
    kNone,
    kEq,
    kLt,
    kLe,
    kGt,
    kGe,
}
// cpp: third_party/blink/renderer/core/css/media_query_exp.cc:610-627
pub const fn MediaQueryOperatorToString(op: MediaQueryOperator) -> &'static str {
    match op {
        MediaQueryOperator::kNone => "",
        MediaQueryOperator::kEq => "=",
        MediaQueryOperator::kLt => "<",
        MediaQueryOperator::kLe => "<=",
        MediaQueryOperator::kGt => ">",
        MediaQueryOperator::kGe => ">=",
    }
}

// cpp: third_party/blink/renderer/core/css/media_query_exp.h:213-230
pub struct MediaQueryExpComparison<V> {
    pub value: MediaQueryExpValue<V>,
    pub op: MediaQueryOperator,
}
impl<V> Default for MediaQueryExpComparison<V> {
    fn default() -> Self {
        Self {
            value: MediaQueryExpValue::default(),
            op: MediaQueryOperator::kNone,
        }
    }
}
impl<V> Clone for MediaQueryExpComparison<V> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            op: self.op,
        }
    }
}
impl<V: PartialEq> PartialEq for MediaQueryExpComparison<V> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.op == other.op
    }
}
impl<V> MediaQueryExpComparison<V> {
    pub fn FromValue(value: &MediaQueryExpValue<V>) -> Self {
        Self {
            value: value.clone(),
            op: MediaQueryOperator::kNone,
        }
    }
    pub fn new(value: &MediaQueryExpValue<V>, op: MediaQueryOperator) -> Self {
        Self {
            value: value.clone(),
            op,
        }
    }
    pub fn IsValid(&self) -> bool {
        self.value.IsValid()
    }
}

// cpp: third_party/blink/renderer/core/css/media_query_exp.h:250-274
pub struct MediaQueryExpBounds<V> {
    pub left: MediaQueryExpComparison<V>,
    pub right: MediaQueryExpComparison<V>,
}
impl<V> Default for MediaQueryExpBounds<V> {
    fn default() -> Self {
        Self {
            left: MediaQueryExpComparison::default(),
            right: MediaQueryExpComparison::default(),
        }
    }
}
impl<V> Clone for MediaQueryExpBounds<V> {
    fn clone(&self) -> Self {
        Self {
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}
impl<V: PartialEq> PartialEq for MediaQueryExpBounds<V> {
    fn eq(&self, other: &Self) -> bool {
        self.left == other.left && self.right == other.right
    }
}
impl<V> MediaQueryExpBounds<V> {
    pub fn FromRight(right: &MediaQueryExpComparison<V>) -> Self {
        Self {
            left: MediaQueryExpComparison::default(),
            right: right.clone(),
        }
    }
    pub fn new(left: &MediaQueryExpComparison<V>, right: &MediaQueryExpComparison<V>) -> Self {
        Self {
            left: left.clone(),
            right: right.clone(),
        }
    }
    pub fn IsRange(&self) -> bool {
        self.left.op != MediaQueryOperator::kNone || self.right.op != MediaQueryOperator::kNone
    }
}
// cpp: third_party/blink/renderer/core/css/media_query_exp.h:290-322,330-348
// U preserves the distinct CSSUnparsedDeclarationValue reference-value type.
// V and U remain provided CSS objects, not a replacement value implementation.
#[derive(Clone, Copy, PartialEq, Eq)]
enum MediaQueryExpType {
    MediaFeature,
    CustomMedia,
    StyleRange,
    Invalid,
}
pub struct MediaQueryExp<V, U = V> {
    type_: MediaQueryExpType,
    media_feature_: foundation::AtomicString,
    reference_value_: Option<Rc<U>>,
    bounds_: MediaQueryExpBounds<V>,
}
impl<V, U> Default for MediaQueryExp<V, U> {
    fn default() -> Self {
        Self {
            type_: MediaQueryExpType::Invalid,
            media_feature_: foundation::AtomicString::default(),
            reference_value_: None,
            bounds_: MediaQueryExpBounds::default(),
        }
    }
}
impl<V, U> Clone for MediaQueryExp<V, U> {
    // cpp: third_party/blink/renderer/core/css/media_query_exp.cc:456-460
    fn clone(&self) -> Self {
        Self {
            type_: self.type_,
            media_feature_: self.media_feature_.clone(),
            reference_value_: self.reference_value_.clone(),
            bounds_: self.bounds_.clone(),
        }
    }
}
impl<V: PartialEq, U> PartialEq for MediaQueryExp<V, U> {
    // cpp: third_party/blink/renderer/core/css/media_query_exp.cc:656-660
    fn eq(&self, other: &Self) -> bool {
        let same_reference = match (&self.reference_value_, &other.reference_value_) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            _ => false,
        };
        self.type_ == other.type_
            && self.media_feature_ == other.media_feature_
            && same_reference
            && self.bounds_ == other.bounds_
    }
}
impl<V, U> MediaQueryExp<V, U> {
    // cpp: third_party/blink/renderer/core/css/media_query_exp.h:290
    pub fn Invalid() -> Self {
        Self::default()
    }
    // cpp: third_party/blink/renderer/core/css/media_query_exp.cc:462-466
    #[allow(dead_code)]
    fn FromFeatureValue(media_feature: &foundation::String, value: &MediaQueryExpValue<V>) -> Self {
        let atom = if media_feature.IsNull() {
            foundation::AtomicString::default()
        } else {
            foundation::AtomicString::from_utf16(media_feature.Span16().unwrap_or_default())
        };
        Self::CreateWithBounds(
            &atom,
            &MediaQueryExpBounds::FromRight(&MediaQueryExpComparison::FromValue(value)),
        )
    }
    // cpp: third_party/blink/renderer/core/css/media_query_exp.cc:468-477,631-638
    pub fn CreateWithBounds(
        media_feature: &foundation::AtomicString,
        bounds: &MediaQueryExpBounds<V>,
    ) -> Self {
        Self {
            type_: MediaQueryExpType::MediaFeature,
            media_feature_: media_feature.clone(),
            reference_value_: None,
            bounds_: bounds.clone(),
        }
    }
    pub fn CreateCustomMedia(custom_media: &foundation::AtomicString) -> Self {
        Self {
            type_: MediaQueryExpType::CustomMedia,
            media_feature_: custom_media.clone(),
            reference_value_: None,
            bounds_: MediaQueryExpBounds::default(),
        }
    }
    // Typed style-range construction keeps unparsed CSSValue ownership intact.
    // The caller supplies the same runtime gate as the Chromium factory.
    pub fn CreateStyleRange(
        reference_value: Rc<U>,
        bounds: &MediaQueryExpBounds<V>,
        enabled: bool,
    ) -> Option<Self> {
        enabled.then(|| Self::FromReferenceValue(reference_value, bounds))
    }
    fn FromReferenceValue(reference_value: Rc<U>, bounds: &MediaQueryExpBounds<V>) -> Self {
        Self {
            type_: MediaQueryExpType::StyleRange,
            reference_value_: Some(reference_value),
            media_feature_: foundation::AtomicString::default(),
            bounds_: bounds.clone(),
        }
    }
    // cpp: third_party/blink/renderer/core/css/media_query_exp.h:296-314
    pub fn IsValid(&self) -> bool {
        self.type_ != MediaQueryExpType::Invalid
    }
    pub fn HasMediaFeature(&self) -> bool {
        self.type_ == MediaQueryExpType::MediaFeature
    }
    pub fn HasStyleRange(&self) -> bool {
        self.type_ == MediaQueryExpType::StyleRange
    }
    pub fn IsCustomMedia(&self) -> bool {
        self.type_ == MediaQueryExpType::CustomMedia
    }
    pub fn MediaFeature(&self) -> &foundation::AtomicString {
        debug_assert!(self.HasMediaFeature() || self.IsCustomMedia());
        &self.media_feature_
    }
    pub fn ReferenceValue(&self) -> &U {
        debug_assert!(self.HasStyleRange());
        self.reference_value_.as_ref().unwrap()
    }
    pub fn Bounds(&self) -> &MediaQueryExpBounds<V> {
        &self.bounds_
    }
    // cpp: third_party/blink/renderer/core/css/media_query_exp.cc:373-392
    // Literal atom names: core/css/media_feature_names.json5.
    pub fn IsViewportDependent(&self) -> bool {
        if !self.HasMediaFeature() {
            return false;
        }
        self.media_feature_ == "width"
            || self.media_feature_ == "height"
            || self.media_feature_ == "min-width"
            || self.media_feature_ == "min-height"
            || self.media_feature_ == "max-width"
            || self.media_feature_ == "max-height"
            || self.media_feature_ == "orientation"
            || self.media_feature_ == "aspect-ratio"
            || self.media_feature_ == "min-aspect-ratio"
            || self.media_feature_ == "-webkit-device-pixel-ratio"
            || self.media_feature_ == "resolution"
            || self.media_feature_ == "max-aspect-ratio"
            || self.media_feature_ == "-webkit-max-device-pixel-ratio"
            || self.media_feature_ == "-webkit-min-device-pixel-ratio"
    }
    // cpp: third_party/blink/renderer/core/css/media_query_exp.cc:394-410
    // Literal atom names: core/css/media_feature_names.json5.
    pub fn IsDeviceDependent(&self) -> bool {
        if !self.HasMediaFeature() {
            return false;
        }
        self.media_feature_ == "device-aspect-ratio"
            || self.media_feature_ == "device-width"
            || self.media_feature_ == "device-height"
            || self.media_feature_ == "min-device-aspect-ratio"
            || self.media_feature_ == "min-device-width"
            || self.media_feature_ == "min-device-height"
            || self.media_feature_ == "max-device-aspect-ratio"
            || self.media_feature_ == "max-device-width"
            || self.media_feature_ == "max-device-height"
            || self.media_feature_ == "dynamic-range"
            || self.media_feature_ == "video-dynamic-range"
    }
    // cpp: third_party/blink/renderer/core/css/media_query_exp.cc:412-423
    // Literal atom names: core/css/media_feature_names.json5.
    pub fn IsWidthDependent(&self) -> bool {
        if !self.HasMediaFeature() {
            return false;
        }
        self.media_feature_ == "width"
            || self.media_feature_ == "min-width"
            || self.media_feature_ == "max-width"
            || self.media_feature_ == "aspect-ratio"
            || self.media_feature_ == "min-aspect-ratio"
            || self.media_feature_ == "max-aspect-ratio"
            || self.media_feature_ == "orientation"
    }
    // cpp: third_party/blink/renderer/core/css/media_query_exp.cc:425-436
    // Literal atom names: core/css/media_feature_names.json5.
    pub fn IsHeightDependent(&self) -> bool {
        if !self.HasMediaFeature() {
            return false;
        }
        self.media_feature_ == "height"
            || self.media_feature_ == "min-height"
            || self.media_feature_ == "max-height"
            || self.media_feature_ == "aspect-ratio"
            || self.media_feature_ == "min-aspect-ratio"
            || self.media_feature_ == "max-aspect-ratio"
            || self.media_feature_ == "orientation"
    }
    // cpp: third_party/blink/renderer/core/css/media_query_exp.cc:438-445
    // Literal atom names: core/css/media_feature_names.json5.
    pub fn IsInlineSizeDependent(&self) -> bool {
        if !self.HasMediaFeature() {
            return false;
        }
        self.media_feature_ == "inline-size"
            || self.media_feature_ == "min-inline-size"
            || self.media_feature_ == "max-inline-size"
    }
    // cpp: third_party/blink/renderer/core/css/media_query_exp.cc:447-454
    // Literal atom names: core/css/media_feature_names.json5.
    pub fn IsBlockSizeDependent(&self) -> bool {
        if !self.HasMediaFeature() {
            return false;
        }
        self.media_feature_ == "block-size"
            || self.media_feature_ == "min-block-size"
            || self.media_feature_ == "max-block-size"
    }
}
// Deliberately unported: primitive/numeric CSSValue dispatch, canonical-unit
// resolution, Consume(), the style-range public factory, accumulated-unit
// dispatch, and the feature dependency wrappers.

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_tags_and_pointer_equality_are_distinct() {
        let invalid = MediaQueryExpValue::<f64>::default();
        let invalid_id = MediaQueryExpValue::<f64>::FromId(CSSValueID::kInvalid);
        assert!(!invalid.IsValid());
        assert!(invalid_id.IsValid());
        assert!(invalid_id.IsId());
        assert!(invalid != invalid_id);
        let nan = Rc::new(f64::NAN);
        let a = MediaQueryExpValue::FromValue(nan.clone());
        assert!(a == a.clone());
        assert!(a != MediaQueryExpValue::FromValue(Rc::new(f64::NAN)));
        let ratio = MediaQueryExpValue::FromRatio(nan.clone(), Rc::new(1.0));
        assert!(ratio == ratio.clone());
        assert!(ratio == MediaQueryExpValue::FromRatio(nan, Rc::new(1.0)));
        assert!(ratio != MediaQueryExpValue::FromRatio(Rc::new(f64::NAN), Rc::new(1.0)));
        assert_eq!(*ratio.Denominator(), 1.0);
        let comparison = MediaQueryExpComparison::new(&invalid, MediaQueryOperator::kLt);
        let bounds = MediaQueryExpBounds::FromRight(&comparison);
        assert!(!bounds.right.IsValid());
        assert!(bounds.IsRange());
    }
    #[test]
    fn dependency_classification_and_reference_identity_match_source() {
        use foundation::{AtomicString, String};
        let bounds = MediaQueryExpBounds::<f64>::default();
        let width = MediaQueryExp::<f64, String>::CreateWithBounds(
            &AtomicString::from_str("min-width"),
            &bounds,
        );
        assert!(width.IsViewportDependent());
        assert!(width.IsWidthDependent());
        assert!(!width.IsHeightDependent());
        assert!(!width.IsDeviceDependent());
        let device = MediaQueryExp::<f64, String>::CreateWithBounds(
            &AtomicString::from_str("device-width"),
            &bounds,
        );
        assert!(device.IsDeviceDependent());
        assert!(!device.IsViewportDependent());
        assert!(!device.IsWidthDependent());
        let prefixed = MediaQueryExp::<f64, String>::CreateWithBounds(
            &AtomicString::from_str("-webkit-device-pixel-ratio"),
            &bounds,
        );
        assert!(prefixed.IsViewportDependent());
        let max_resolution = MediaQueryExp::<f64, String>::CreateWithBounds(
            &AtomicString::from_str("max-resolution"),
            &bounds,
        );
        // The exact source predicate includes resolution but does not include
        // min/max-resolution; do not expand this list by inferred semantics.
        assert!(!max_resolution.IsViewportDependent());
        let custom =
            MediaQueryExp::<f64, String>::CreateCustomMedia(&AtomicString::from_str("width"));
        assert!(custom.IsCustomMedia());
        assert!(!custom.HasMediaFeature());
        assert!(!custom.IsViewportDependent());
        let reference = Rc::new(String::from("--foo"));
        let style = MediaQueryExp::FromReferenceValue(reference.clone(), &bounds);
        assert!(style.HasStyleRange());
        assert!(!style.IsDeviceDependent());
        assert!(style == MediaQueryExp::FromReferenceValue(reference, &bounds));
        assert!(
            style != MediaQueryExp::FromReferenceValue(Rc::new(String::from("--foo")), &bounds)
        );
    }
}
