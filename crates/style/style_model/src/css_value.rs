/*
 * (C) 1999-2003 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2004, 2005, 2006, 2007, 2008 Apple Inc. All rights reserved.
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
 */

/*
 * Copyright (C) 2011 Andreas Kling (kling@webkit.org)
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
 * THIS SOFTWARE IS PROVIDED BY APPLE COMPUTER, INC. ``AS IS'' AND ANY
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
 *
 */

// cpp: third_party/blink/renderer/core/css/css_value.h:40-270,275-277,281-478
//! CSSValue's complete base-level runtime logic over typed concrete payloads.
//! The required traits below are dependency boundaries for the original derived
//! methods; no default implementation, placeholder value, or class-only equality
//! is provided. Each associated payload must implement its real source behavior.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
use foundation::{HashInt, HashInts, Length, LengthType, String};
use std::rc::Rc;

// cpp: third_party/blink/renderer/core/css/css_value.h:285-401
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClassType {
    kNumericLiteralClass = 0,
    kMathFunctionClass = 1,
    kIdentifierClass = 2,
    kScopedKeywordClass = 3,
    kColorClass = 4,
    kUnresolvedColorClass = 5,
    kColorMixClass = 6,
    kAlphaColorClass = 7,
    kContrastColorClass = 8,
    kCounterClass = 9,
    kCounterContentClass = 10,
    kQuadClass = 11,
    kCustomIdentClass = 12,
    kStringClass = 13,
    kURIClass = 14,
    kURLPatternClass = 15,
    kValuePairClass = 16,
    kLightDarkValuePairClass = 17,
    kParamValuePairClass = 18,
    kScrollClass = 19,
    kViewClass = 20,
    kRatioClass = 21,
    kRelativeColorClass = 22,
    kBasicShapeCircleClass = 23,
    kBasicShapeEllipseClass = 24,
    kBasicShapePolygonClass = 25,
    kBasicShapeInsetClass = 26,
    kBasicShapeRectClass = 27,
    kBasicShapeXYWHClass = 28,
    kBasicShapePathClass = 29,
    kBasicShapeShapeClass = 30,
    kImageClass = 31,
    kCursorImageClass = 32,
    kCrossfadeClass = 33,
    kPaintClass = 34,
    kLinearGradientClass = 35,
    kRadialGradientClass = 36,
    kConicGradientClass = 37,
    kConstantGradientClass = 38,
    kColorImageClass = 39,
    kLinearTimingFunctionClass = 40,
    kCubicBezierTimingFunctionClass = 41,
    kStepsTimingFunctionClass = 42,
    kProgressClass = 43,
    kBorderImageSliceClass = 44,
    kDynamicRangeLimitMixClass = 45,
    kFontFeatureClass = 46,
    kFontFaceSrcClass = 47,
    kFontFamilyClass = 48,
    kFontStyleRangeClass = 49,
    kFontVariationClass = 50,
    kAlternateClass = 51,
    kInheritedClass = 52,
    kInitialClass = 53,
    kUnsetClass = 54,
    kRevertClass = 55,
    kRevertLayerClass = 56,
    kRevertRuleClass = 57,
    kReflectClass = 58,
    kShadowClass = 59,
    kUnicodeRangeClass = 60,
    kGridTemplateAreasClass = 61,
    kPaletteMixClass = 62,
    kRayClass = 63,
    kUnparsedDeclarationClass = 64,
    kPendingSubstitutionValueClass = 65,
    kPendingSystemFontValueClass = 66,
    kInvalidVariableValueClass = 67,
    kCyclicVariableValueClass = 68,
    kFlipRevertClass = 69,
    kLayoutFunctionClass = 70,
    kCSSContentDistributionClass = 71,
    kKeyframeShorthandClass = 72,
    kInitialColorValueClass = 73,
    kImageSetOptionClass = 74,
    kImageSetTypeClass = 75,
    kRepeatStyleClass = 76,
    kSuperellipseClass = 77,
    kSymbolsClass = 78,
    kTriggerAttachmentClass = 79,
    kRepeatClass = 80,
    kValueListClass = 81,
    kFunctionClass = 82,
    kImageSetClass = 83,
    kGridLineNamesClass = 84,
    kGridAutoRepeatClass = 85,
    kGridIntegerRepeatClass = 86,
    kAxisClass = 87,
}

// Required methods on the actual derived values. These do not manufacture
// missing CSS derived classes or accept a free-standing class tag as a value.
pub trait CSSValueSubclass {
    fn CustomCSSText(&self) -> String;
    fn Equals(&self, other: &Self) -> bool;
}
pub trait CSSValueCustomHash {
    fn CustomHash(&self) -> u32;
}
pub trait CSSValueSubresources {
    fn HasFailedOrCanceledSubresources(&self) -> bool;
}
pub trait CSSValueRandom {
    fn HasRandomFunctions(&self) -> bool;
}
pub trait CSSValueUrl<Document: ?Sized> {
    fn ReResolveUrl(&self, document: &Document);
}
pub trait CSSValueListUrls {
    fn MayContainUrl(&self) -> bool;
}
pub trait CSSValueListSubclass<D: CSSValueDispatch> {
    fn AsValueList(&self) -> &D::CSSValueList;
}
pub trait CSSValuePairSubclass<D: CSSValueDispatch> {
    fn AsValuePair(&self) -> &D::CSSValuePair;
}
pub trait CSSValueTreeScope<D: CSSValueDispatch> {
    fn PopulateWithTreeScope<'a>(&'a self, scope: Option<&'a D::TreeScope>) -> &'a CSSValue<D>;
}

// Every associated type is the real source cast target's payload. Bounds are
// generated from the concrete methods called by css_value.cc, rather than a
// catch-all interface with default values. Document/TreeScope remain opaque
// required application-owned types and do not depend on renderer/browser/page.
pub trait CSSValueDispatch: Sized {
    type Document: ?Sized;
    type TreeScope: ?Sized;
    type CSSNumericLiteralValue: CSSValueSubclass + CSSValueCustomHash;
    type CSSMathFunctionValue: CSSValueSubclass + CSSValueRandom + CSSValueTreeScope<Self>;
    type CSSIdentifierValue: CSSValueSubclass + CSSValueCustomHash;
    type CSSScopedKeywordValue: CSSValueSubclass + CSSValueTreeScope<Self>;
    type CSSColor: CSSValueSubclass + CSSValueCustomHash;
    type CSSUnresolvedColorValue: CSSValueSubclass + CSSValueRandom;
    type CSSColorMixValue: CSSValueSubclass + CSSValueRandom;
    type CSSAlphaColorValue: CSSValueSubclass + CSSValueRandom;
    type CSSContrastColorValue: CSSValueSubclass + CSSValueRandom;
    type CSSCounterValue: CSSValueSubclass + CSSValueRandom;
    type CSSCounterContentValue: CSSValueSubclass + CSSValueRandom + CSSValueTreeScope<Self>;
    type CSSQuadValue: CSSValueSubclass + CSSValueRandom;
    type CSSCustomIdentValue: CSSValueSubclass
        + CSSValueCustomHash
        + CSSValueRandom
        + CSSValueTreeScope<Self>;
    type CSSStringValue: CSSValueSubclass + CSSValueCustomHash;
    type CSSURIValue: CSSValueSubclass + CSSValueUrl<Self::Document>;
    type CSSURLPatternValue: CSSValueSubclass;
    type CSSValuePair: CSSValueSubclass + CSSValueCustomHash + CSSValueRandom;
    type CSSLightDarkValuePair: CSSValueSubclass + CSSValuePairSubclass<Self>;
    type CSSParamValuePair: CSSValueSubclass + CSSValuePairSubclass<Self>;
    type CSSScrollValue: CSSValueSubclass + CSSValueRandom;
    type CSSViewValue: CSSValueSubclass + CSSValueRandom;
    type CSSRatioValue: CSSValueSubclass + CSSValueRandom;
    type CSSRelativeColorValue: CSSValueSubclass + CSSValueRandom;
    type CSSBasicShapeCircleValue: CSSValueSubclass + CSSValueRandom;
    type CSSBasicShapeEllipseValue: CSSValueSubclass + CSSValueRandom;
    type CSSBasicShapePolygonValue: CSSValueSubclass + CSSValueRandom;
    type CSSBasicShapeInsetValue: CSSValueSubclass + CSSValueRandom;
    type CSSBasicShapeRectValue: CSSValueSubclass + CSSValueRandom;
    type CSSBasicShapeXYWHValue: CSSValueSubclass + CSSValueRandom;
    type CSSPathValue: CSSValueSubclass + CSSValueCustomHash;
    type CSSShapeValue: CSSValueSubclass + CSSValueRandom;
    type CSSImageValue: CSSValueSubclass + CSSValueSubresources + CSSValueUrl<Self::Document>;
    type CSSCursorImageValue: CSSValueSubclass;
    type CSSCrossfadeValue: CSSValueSubclass + CSSValueRandom + CSSValueSubresources;
    type CSSPaintValue: CSSValueSubclass + CSSValueRandom;
    type CSSLinearGradientValue: CSSValueSubclass + CSSValueRandom;
    type CSSRadialGradientValue: CSSValueSubclass + CSSValueRandom;
    type CSSConicGradientValue: CSSValueSubclass + CSSValueRandom;
    type CSSConstantGradientValue: CSSValueSubclass + CSSValueRandom;
    type CSSColorImageValue: CSSValueSubclass + CSSValueRandom;
    type CSSLinearTimingFunctionValue: CSSValueSubclass;
    type CSSCubicBezierTimingFunctionValue: CSSValueSubclass;
    type CSSStepsTimingFunctionValue: CSSValueSubclass + CSSValueRandom;
    type CSSProgressValue: CSSValueSubclass;
    type CSSBorderImageSliceValue: CSSValueSubclass + CSSValueRandom;
    type CSSDynamicRangeLimitMixValue: CSSValueSubclass + CSSValueRandom;
    type CSSFontFeatureValue: CSSValueSubclass + CSSValueRandom;
    type CSSFontFaceSrcValue: CSSValueSubclass + CSSValueSubresources;
    type CSSFontFamilyValue: CSSValueSubclass;
    type CSSFontStyleRangeValue: CSSValueSubclass + CSSValueRandom;
    type CSSFontVariationValue: CSSValueSubclass + CSSValueRandom;
    type CSSAlternateValue: CSSValueSubclass + CSSValueRandom;
    type CSSInheritedValue: CSSValueSubclass;
    type CSSInitialValue: CSSValueSubclass;
    type CSSUnsetValue: CSSValueSubclass;
    type CSSRevertValue: CSSValueSubclass;
    type CSSRevertLayerValue: CSSValueSubclass;
    type CSSRevertRuleValue: CSSValueSubclass;
    type CSSReflectValue: CSSValueSubclass + CSSValueRandom;
    type CSSShadowValue: CSSValueSubclass + CSSValueRandom;
    type CSSUnicodeRangeValue: CSSValueSubclass;
    type CSSGridTemplateAreasValue: CSSValueSubclass;
    type CSSPaletteMixValue: CSSValueSubclass + CSSValueRandom;
    type CSSRayValue: CSSValueSubclass + CSSValueRandom;
    type CSSUnparsedDeclarationValue: CSSValueSubclass + CSSValueCustomHash + CSSValueRandom;
    type CSSPendingSubstitutionValue: CSSValueSubclass;
    type CSSPendingSystemFontValue: CSSValueSubclass;
    type CSSInvalidVariableValue: CSSValueSubclass;
    type CSSCyclicVariableValue: CSSValueSubclass;
    type CSSFlipRevertValue: CSSValueSubclass;
    type CSSLayoutFunctionValue: CSSValueSubclass + CSSValueRandom;
    type CSSContentDistributionValue: CSSValueSubclass + CSSValueCustomHash;
    type CSSKeyframeShorthandValue: CSSValueSubclass;
    type CSSInitialColorValue: CSSValueSubclass;
    type CSSImageSetOptionValue: CSSValueSubclass + CSSValueRandom;
    type CSSImageSetTypeValue: CSSValueSubclass;
    type CSSRepeatStyleValue: CSSValueSubclass + CSSValueRandom;
    type CSSSuperellipseValue: CSSValueSubclass + CSSValueCustomHash + CSSValueRandom;
    type CSSSymbolsValue: CSSValueSubclass;
    type CSSTriggerAttachmentValue: CSSValueSubclass + CSSValueRandom + CSSValueTreeScope<Self>;
    type CSSRepeatValue: CSSValueSubclass + CSSValueRandom;
    type CSSValueList: CSSValueSubclass
        + CSSValueCustomHash
        + CSSValueRandom
        + CSSValueSubresources
        + CSSValueUrl<Self::Document>
        + CSSValueTreeScope<Self>
        + CSSValueListUrls;
    type CSSFunctionValue: CSSValueSubclass + CSSValueListSubclass<Self>;
    type CSSImageSetValue: CSSValueSubclass
        + CSSValueRandom
        + CSSValueSubresources
        + CSSValueListSubclass<Self>;
    type CSSBracketedValueList: CSSValueSubclass + CSSValueListSubclass<Self>;
    type CSSGridAutoRepeatValue: CSSValueSubclass + CSSValueListSubclass<Self>;
    type CSSGridIntegerRepeatValue: CSSValueSubclass + CSSValueRandom + CSSValueListSubclass<Self>;
    type CSSAxisValue: CSSValueSubclass + CSSValueListSubclass<Self>;
    fn CreateIdentifierFromLength(value: &Length) -> Rc<CSSValue<Self>>;
    fn CreatePrimitiveFromLength(value: &Length, zoom: f32) -> Rc<CSSValue<Self>>;
}

// The enum variant and its concrete associated type are inseparable. All
// casts in the source switch therefore require the correct typed payload.
pub enum CSSValuePayload<D: CSSValueDispatch> {
    kNumericLiteralClass(D::CSSNumericLiteralValue),
    kMathFunctionClass(D::CSSMathFunctionValue),
    kIdentifierClass(D::CSSIdentifierValue),
    kScopedKeywordClass(D::CSSScopedKeywordValue),
    kColorClass(D::CSSColor),
    kUnresolvedColorClass(D::CSSUnresolvedColorValue),
    kColorMixClass(D::CSSColorMixValue),
    kAlphaColorClass(D::CSSAlphaColorValue),
    kContrastColorClass(D::CSSContrastColorValue),
    kCounterClass(D::CSSCounterValue),
    kCounterContentClass(D::CSSCounterContentValue),
    kQuadClass(D::CSSQuadValue),
    kCustomIdentClass(D::CSSCustomIdentValue),
    kStringClass(D::CSSStringValue),
    kURIClass(D::CSSURIValue),
    kURLPatternClass(D::CSSURLPatternValue),
    kValuePairClass(D::CSSValuePair),
    kLightDarkValuePairClass(D::CSSLightDarkValuePair),
    kParamValuePairClass(D::CSSParamValuePair),
    kScrollClass(D::CSSScrollValue),
    kViewClass(D::CSSViewValue),
    kRatioClass(D::CSSRatioValue),
    kRelativeColorClass(D::CSSRelativeColorValue),
    kBasicShapeCircleClass(D::CSSBasicShapeCircleValue),
    kBasicShapeEllipseClass(D::CSSBasicShapeEllipseValue),
    kBasicShapePolygonClass(D::CSSBasicShapePolygonValue),
    kBasicShapeInsetClass(D::CSSBasicShapeInsetValue),
    kBasicShapeRectClass(D::CSSBasicShapeRectValue),
    kBasicShapeXYWHClass(D::CSSBasicShapeXYWHValue),
    kBasicShapePathClass(D::CSSPathValue),
    kBasicShapeShapeClass(D::CSSShapeValue),
    kImageClass(D::CSSImageValue),
    kCursorImageClass(D::CSSCursorImageValue),
    kCrossfadeClass(D::CSSCrossfadeValue),
    kPaintClass(D::CSSPaintValue),
    kLinearGradientClass(D::CSSLinearGradientValue),
    kRadialGradientClass(D::CSSRadialGradientValue),
    kConicGradientClass(D::CSSConicGradientValue),
    kConstantGradientClass(D::CSSConstantGradientValue),
    kColorImageClass(D::CSSColorImageValue),
    kLinearTimingFunctionClass(D::CSSLinearTimingFunctionValue),
    kCubicBezierTimingFunctionClass(D::CSSCubicBezierTimingFunctionValue),
    kStepsTimingFunctionClass(D::CSSStepsTimingFunctionValue),
    kProgressClass(D::CSSProgressValue),
    kBorderImageSliceClass(D::CSSBorderImageSliceValue),
    kDynamicRangeLimitMixClass(D::CSSDynamicRangeLimitMixValue),
    kFontFeatureClass(D::CSSFontFeatureValue),
    kFontFaceSrcClass(D::CSSFontFaceSrcValue),
    kFontFamilyClass(D::CSSFontFamilyValue),
    kFontStyleRangeClass(D::CSSFontStyleRangeValue),
    kFontVariationClass(D::CSSFontVariationValue),
    kAlternateClass(D::CSSAlternateValue),
    kInheritedClass(D::CSSInheritedValue),
    kInitialClass(D::CSSInitialValue),
    kUnsetClass(D::CSSUnsetValue),
    kRevertClass(D::CSSRevertValue),
    kRevertLayerClass(D::CSSRevertLayerValue),
    kRevertRuleClass(D::CSSRevertRuleValue),
    kReflectClass(D::CSSReflectValue),
    kShadowClass(D::CSSShadowValue),
    kUnicodeRangeClass(D::CSSUnicodeRangeValue),
    kGridTemplateAreasClass(D::CSSGridTemplateAreasValue),
    kPaletteMixClass(D::CSSPaletteMixValue),
    kRayClass(D::CSSRayValue),
    kUnparsedDeclarationClass(D::CSSUnparsedDeclarationValue),
    kPendingSubstitutionValueClass(D::CSSPendingSubstitutionValue),
    kPendingSystemFontValueClass(D::CSSPendingSystemFontValue),
    kInvalidVariableValueClass(D::CSSInvalidVariableValue),
    kCyclicVariableValueClass(D::CSSCyclicVariableValue),
    kFlipRevertClass(D::CSSFlipRevertValue),
    kLayoutFunctionClass(D::CSSLayoutFunctionValue),
    kCSSContentDistributionClass(D::CSSContentDistributionValue),
    kKeyframeShorthandClass(D::CSSKeyframeShorthandValue),
    kInitialColorValueClass(D::CSSInitialColorValue),
    kImageSetOptionClass(D::CSSImageSetOptionValue),
    kImageSetTypeClass(D::CSSImageSetTypeValue),
    kRepeatStyleClass(D::CSSRepeatStyleValue),
    kSuperellipseClass(D::CSSSuperellipseValue),
    kSymbolsClass(D::CSSSymbolsValue),
    kTriggerAttachmentClass(D::CSSTriggerAttachmentValue),
    kRepeatClass(D::CSSRepeatValue),
    kValueListClass(D::CSSValueList),
    kFunctionClass(D::CSSFunctionValue),
    kImageSetClass(D::CSSImageSetValue),
    kGridLineNamesClass(D::CSSBracketedValueList),
    kGridAutoRepeatClass(D::CSSGridAutoRepeatValue),
    kGridIntegerRepeatClass(D::CSSGridIntegerRepeatValue),
    kAxisClass(D::CSSAxisValue),
}
impl<D: CSSValueDispatch> CSSValuePayload<D> {
    fn GetClassType(&self) -> ClassType {
        match self {
            Self::kNumericLiteralClass(..) => ClassType::kNumericLiteralClass,
            Self::kMathFunctionClass(..) => ClassType::kMathFunctionClass,
            Self::kIdentifierClass(..) => ClassType::kIdentifierClass,
            Self::kScopedKeywordClass(..) => ClassType::kScopedKeywordClass,
            Self::kColorClass(..) => ClassType::kColorClass,
            Self::kUnresolvedColorClass(..) => ClassType::kUnresolvedColorClass,
            Self::kColorMixClass(..) => ClassType::kColorMixClass,
            Self::kAlphaColorClass(..) => ClassType::kAlphaColorClass,
            Self::kContrastColorClass(..) => ClassType::kContrastColorClass,
            Self::kCounterClass(..) => ClassType::kCounterClass,
            Self::kCounterContentClass(..) => ClassType::kCounterContentClass,
            Self::kQuadClass(..) => ClassType::kQuadClass,
            Self::kCustomIdentClass(..) => ClassType::kCustomIdentClass,
            Self::kStringClass(..) => ClassType::kStringClass,
            Self::kURIClass(..) => ClassType::kURIClass,
            Self::kURLPatternClass(..) => ClassType::kURLPatternClass,
            Self::kValuePairClass(..) => ClassType::kValuePairClass,
            Self::kLightDarkValuePairClass(..) => ClassType::kLightDarkValuePairClass,
            Self::kParamValuePairClass(..) => ClassType::kParamValuePairClass,
            Self::kScrollClass(..) => ClassType::kScrollClass,
            Self::kViewClass(..) => ClassType::kViewClass,
            Self::kRatioClass(..) => ClassType::kRatioClass,
            Self::kRelativeColorClass(..) => ClassType::kRelativeColorClass,
            Self::kBasicShapeCircleClass(..) => ClassType::kBasicShapeCircleClass,
            Self::kBasicShapeEllipseClass(..) => ClassType::kBasicShapeEllipseClass,
            Self::kBasicShapePolygonClass(..) => ClassType::kBasicShapePolygonClass,
            Self::kBasicShapeInsetClass(..) => ClassType::kBasicShapeInsetClass,
            Self::kBasicShapeRectClass(..) => ClassType::kBasicShapeRectClass,
            Self::kBasicShapeXYWHClass(..) => ClassType::kBasicShapeXYWHClass,
            Self::kBasicShapePathClass(..) => ClassType::kBasicShapePathClass,
            Self::kBasicShapeShapeClass(..) => ClassType::kBasicShapeShapeClass,
            Self::kImageClass(..) => ClassType::kImageClass,
            Self::kCursorImageClass(..) => ClassType::kCursorImageClass,
            Self::kCrossfadeClass(..) => ClassType::kCrossfadeClass,
            Self::kPaintClass(..) => ClassType::kPaintClass,
            Self::kLinearGradientClass(..) => ClassType::kLinearGradientClass,
            Self::kRadialGradientClass(..) => ClassType::kRadialGradientClass,
            Self::kConicGradientClass(..) => ClassType::kConicGradientClass,
            Self::kConstantGradientClass(..) => ClassType::kConstantGradientClass,
            Self::kColorImageClass(..) => ClassType::kColorImageClass,
            Self::kLinearTimingFunctionClass(..) => ClassType::kLinearTimingFunctionClass,
            Self::kCubicBezierTimingFunctionClass(..) => ClassType::kCubicBezierTimingFunctionClass,
            Self::kStepsTimingFunctionClass(..) => ClassType::kStepsTimingFunctionClass,
            Self::kProgressClass(..) => ClassType::kProgressClass,
            Self::kBorderImageSliceClass(..) => ClassType::kBorderImageSliceClass,
            Self::kDynamicRangeLimitMixClass(..) => ClassType::kDynamicRangeLimitMixClass,
            Self::kFontFeatureClass(..) => ClassType::kFontFeatureClass,
            Self::kFontFaceSrcClass(..) => ClassType::kFontFaceSrcClass,
            Self::kFontFamilyClass(..) => ClassType::kFontFamilyClass,
            Self::kFontStyleRangeClass(..) => ClassType::kFontStyleRangeClass,
            Self::kFontVariationClass(..) => ClassType::kFontVariationClass,
            Self::kAlternateClass(..) => ClassType::kAlternateClass,
            Self::kInheritedClass(..) => ClassType::kInheritedClass,
            Self::kInitialClass(..) => ClassType::kInitialClass,
            Self::kUnsetClass(..) => ClassType::kUnsetClass,
            Self::kRevertClass(..) => ClassType::kRevertClass,
            Self::kRevertLayerClass(..) => ClassType::kRevertLayerClass,
            Self::kRevertRuleClass(..) => ClassType::kRevertRuleClass,
            Self::kReflectClass(..) => ClassType::kReflectClass,
            Self::kShadowClass(..) => ClassType::kShadowClass,
            Self::kUnicodeRangeClass(..) => ClassType::kUnicodeRangeClass,
            Self::kGridTemplateAreasClass(..) => ClassType::kGridTemplateAreasClass,
            Self::kPaletteMixClass(..) => ClassType::kPaletteMixClass,
            Self::kRayClass(..) => ClassType::kRayClass,
            Self::kUnparsedDeclarationClass(..) => ClassType::kUnparsedDeclarationClass,
            Self::kPendingSubstitutionValueClass(..) => ClassType::kPendingSubstitutionValueClass,
            Self::kPendingSystemFontValueClass(..) => ClassType::kPendingSystemFontValueClass,
            Self::kInvalidVariableValueClass(..) => ClassType::kInvalidVariableValueClass,
            Self::kCyclicVariableValueClass(..) => ClassType::kCyclicVariableValueClass,
            Self::kFlipRevertClass(..) => ClassType::kFlipRevertClass,
            Self::kLayoutFunctionClass(..) => ClassType::kLayoutFunctionClass,
            Self::kCSSContentDistributionClass(..) => ClassType::kCSSContentDistributionClass,
            Self::kKeyframeShorthandClass(..) => ClassType::kKeyframeShorthandClass,
            Self::kInitialColorValueClass(..) => ClassType::kInitialColorValueClass,
            Self::kImageSetOptionClass(..) => ClassType::kImageSetOptionClass,
            Self::kImageSetTypeClass(..) => ClassType::kImageSetTypeClass,
            Self::kRepeatStyleClass(..) => ClassType::kRepeatStyleClass,
            Self::kSuperellipseClass(..) => ClassType::kSuperellipseClass,
            Self::kSymbolsClass(..) => ClassType::kSymbolsClass,
            Self::kTriggerAttachmentClass(..) => ClassType::kTriggerAttachmentClass,
            Self::kRepeatClass(..) => ClassType::kRepeatClass,
            Self::kValueListClass(..) => ClassType::kValueListClass,
            Self::kFunctionClass(..) => ClassType::kFunctionClass,
            Self::kImageSetClass(..) => ClassType::kImageSetClass,
            Self::kGridLineNamesClass(..) => ClassType::kGridLineNamesClass,
            Self::kGridAutoRepeatClass(..) => ClassType::kGridAutoRepeatClass,
            Self::kGridIntegerRepeatClass(..) => ClassType::kGridIntegerRepeatClass,
            Self::kAxisClass(..) => ClassType::kAxisClass,
        }
    }
}

// cpp: third_party/blink/renderer/core/css/css_value.h:281-282
pub const kValueListSeparatorBits: usize = 2;
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueListSeparator {
    kSpaceSeparator,
    kCommaSeparator,
    kSlashSeparator,
}

// cpp: third_party/blink/renderer/core/css/css_value.h:424-443
// cpp: third_party/blink/renderer/core/css/css_value.h:446
// Four-byte base state from the original class, independent of payload size.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CSSValueState {
    pub numeric_literal_unit_type_: u8,
    pub value_list_separator_: u8,
    flags_: u8,
    class_type_: ClassType,
}
const _: () = assert!(std::mem::size_of::<CSSValueState>() == 4);
impl CSSValueState {
    // cpp: third_party/blink/renderer/core/css/css_value.h:407-410
    fn new(class_type: ClassType) -> Self {
        Self {
            numeric_literal_unit_type_: 0,
            value_list_separator_: ValueListSeparator::kSpaceSeparator as u8,
            flags_: 0,
            class_type_: class_type,
        }
    }
    // Rust accessors to the same protected one-bit subclass fields.
    pub fn AllowsNegativePercentageReference(&self) -> bool {
        self.flags_ & 1 != 0
    }
    pub fn SetAllowsNegativePercentageReference(&mut self, value: bool) {
        self.SetFlag(1, value);
    }
    pub fn NeedsTreeScopePopulation(&self) -> bool {
        self.flags_ & 2 != 0
    }
    pub fn SetNeedsTreeScopePopulation(&mut self, value: bool) {
        self.SetFlag(2, value);
    }
    pub fn WasQuirky(&self) -> bool {
        self.flags_ & 4 != 0
    }
    pub fn SetWasQuirky(&mut self, value: bool) {
        self.SetFlag(4, value);
    }
    fn SetFlag(&mut self, mask: u8, value: bool) {
        self.flags_ = (self.flags_ & !mask) | if value { mask } else { 0 };
    }
}

pub struct CSSValue<D: CSSValueDispatch> {
    state_: CSSValueState,
    payload_: CSSValuePayload<D>,
}
impl<D: CSSValueDispatch> CSSValue<D> {
    pub fn new(payload: CSSValuePayload<D>) -> Self {
        Self {
            state_: CSSValueState::new(payload.GetClassType()),
            payload_: payload,
        }
    }
    pub fn State(&self) -> &CSSValueState {
        &self.state_
    }
    pub fn StateMut(&mut self) -> &mut CSSValueState {
        &mut self.state_
    }
    pub fn Payload(&self) -> &CSSValuePayload<D> {
        &self.payload_
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:403
    pub fn GetClassType(&self) -> ClassType {
        self.state_.class_type_
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:48-50
    pub fn IsNumericLiteralValue(&self) -> bool {
        self.GetClassType() == ClassType::kNumericLiteralClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:51-51
    pub fn IsMathFunctionValue(&self) -> bool {
        self.GetClassType() == ClassType::kMathFunctionClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:52-54
    pub fn IsPrimitiveValue(&self) -> bool {
        self.IsNumericLiteralValue() || self.IsMathFunctionValue()
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:55-55
    pub fn IsIdentifierValue(&self) -> bool {
        self.GetClassType() == ClassType::kIdentifierClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:56-58
    pub fn IsScopedKeywordValue(&self) -> bool {
        self.GetClassType() == ClassType::kScopedKeywordClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:59-59
    pub fn IsValuePair(&self) -> bool {
        self.GetClassType() == ClassType::kValuePairClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:60-60
    pub fn IsValueList(&self) -> bool {
        self.GetClassType() >= ClassType::kValueListClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:62-62
    pub fn IsBaseValueList(&self) -> bool {
        self.GetClassType() == ClassType::kValueListClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:66-69
    pub fn IsBasicShapeValue(&self) -> bool {
        self.GetClassType() >= ClassType::kBasicShapeCircleClass
            && self.GetClassType() <= ClassType::kBasicShapeShapeClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:70-72
    pub fn IsBasicShapeCircleValue(&self) -> bool {
        self.GetClassType() == ClassType::kBasicShapeCircleClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:73-75
    pub fn IsBasicShapeEllipseValue(&self) -> bool {
        self.GetClassType() == ClassType::kBasicShapeEllipseClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:76-78
    pub fn IsBasicShapePolygonValue(&self) -> bool {
        self.GetClassType() == ClassType::kBasicShapePolygonClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:79-81
    pub fn IsBasicShapeInsetValue(&self) -> bool {
        self.GetClassType() == ClassType::kBasicShapeInsetClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:82-84
    pub fn IsBasicShapeRectValue(&self) -> bool {
        self.GetClassType() == ClassType::kBasicShapeRectClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:85-87
    pub fn IsBasicShapeXYWHValue(&self) -> bool {
        self.GetClassType() == ClassType::kBasicShapeXYWHClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:89-91
    pub fn IsBorderImageSliceValue(&self) -> bool {
        self.GetClassType() == ClassType::kBorderImageSliceClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:92-92
    pub fn IsAlphaColorValue(&self) -> bool {
        self.GetClassType() == ClassType::kAlphaColorClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:93-93
    pub fn IsColorValue(&self) -> bool {
        self.GetClassType() == ClassType::kColorClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:94-94
    pub fn IsColorMixValue(&self) -> bool {
        self.GetClassType() == ClassType::kColorMixClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:95-97
    pub fn IsContrastColorValue(&self) -> bool {
        self.GetClassType() == ClassType::kContrastColorClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:98-98
    pub fn IsCounterValue(&self) -> bool {
        self.GetClassType() == ClassType::kCounterClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:99-101
    pub fn IsCounterContentValue(&self) -> bool {
        self.GetClassType() == ClassType::kCounterContentClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:102-102
    pub fn IsCursorImageValue(&self) -> bool {
        self.GetClassType() == ClassType::kCursorImageClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:103-103
    pub fn IsCrossfadeValue(&self) -> bool {
        self.GetClassType() == ClassType::kCrossfadeClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:104-106
    pub fn IsDynamicRangeLimitMixValue(&self) -> bool {
        self.GetClassType() == ClassType::kDynamicRangeLimitMixClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:107-107
    pub fn IsPaintValue(&self) -> bool {
        self.GetClassType() == ClassType::kPaintClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:108-108
    pub fn IsFontFeatureValue(&self) -> bool {
        self.GetClassType() == ClassType::kFontFeatureClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:109-109
    pub fn IsFontFamilyValue(&self) -> bool {
        self.GetClassType() == ClassType::kFontFamilyClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:110-110
    pub fn IsFontFaceSrcValue(&self) -> bool {
        self.GetClassType() == ClassType::kFontFaceSrcClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:111-113
    pub fn IsFontStyleRangeValue(&self) -> bool {
        self.GetClassType() == ClassType::kFontStyleRangeClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:114-116
    pub fn IsFontVariationValue(&self) -> bool {
        self.GetClassType() == ClassType::kFontVariationClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:117-117
    pub fn IsFunctionValue(&self) -> bool {
        self.GetClassType() == ClassType::kFunctionClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:118-118
    pub fn IsCustomIdentValue(&self) -> bool {
        self.GetClassType() == ClassType::kCustomIdentClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:119-121
    pub fn IsImageGeneratorValue(&self) -> bool {
        self.GetClassType() >= ClassType::kCrossfadeClass
            && self.GetClassType() <= ClassType::kColorImageClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:122-125
    pub fn IsGradientValue(&self) -> bool {
        self.GetClassType() >= ClassType::kLinearGradientClass
            && self.GetClassType() <= ClassType::kColorImageClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:126-128
    pub fn IsImageSetOptionValue(&self) -> bool {
        self.GetClassType() == ClassType::kImageSetOptionClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:129-129
    pub fn IsImageSetTypeValue(&self) -> bool {
        self.GetClassType() == ClassType::kImageSetTypeClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:130-130
    pub fn IsImageSetValue(&self) -> bool {
        self.GetClassType() == ClassType::kImageSetClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:131-131
    pub fn IsImageValue(&self) -> bool {
        self.GetClassType() == ClassType::kImageClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:132-132
    pub fn IsInheritedValue(&self) -> bool {
        self.GetClassType() == ClassType::kInheritedClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:133-133
    pub fn IsInitialValue(&self) -> bool {
        self.GetClassType() == ClassType::kInitialClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:134-134
    pub fn IsUnsetValue(&self) -> bool {
        self.GetClassType() == ClassType::kUnsetClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:135-135
    pub fn IsRevertValue(&self) -> bool {
        self.GetClassType() == ClassType::kRevertClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:136-136
    pub fn IsRevertLayerValue(&self) -> bool {
        self.GetClassType() == ClassType::kRevertLayerClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:137-137
    pub fn IsRevertRuleValue(&self) -> bool {
        self.GetClassType() == ClassType::kRevertRuleClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:139-141
    pub fn IsCascadeDependentKeyword(&self) -> bool {
        self.IsRevertValue() || self.IsRevertLayerValue() || self.IsRevertRuleValue()
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:142-144
    pub fn IsCSSWideKeyword(&self) -> bool {
        self.GetClassType() >= ClassType::kInheritedClass
            && self.GetClassType() <= ClassType::kRevertRuleClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:145-147
    pub fn IsLayoutFunctionValue(&self) -> bool {
        self.GetClassType() == ClassType::kLayoutFunctionClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:148-148
    pub fn IsParamValuePair(&self) -> bool {
        self.GetClassType() == ClassType::kParamValuePairClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:149-151
    pub fn IsLinearGradientValue(&self) -> bool {
        self.GetClassType() == ClassType::kLinearGradientClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:152-152
    pub fn IsPaletteMixValue(&self) -> bool {
        self.GetClassType() == ClassType::kPaletteMixClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:153-153
    pub fn IsPathValue(&self) -> bool {
        self.GetClassType() == ClassType::kBasicShapePathClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:154-154
    pub fn IsShapeValue(&self) -> bool {
        self.GetClassType() == ClassType::kBasicShapeShapeClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:155-155
    pub fn IsQuadValue(&self) -> bool {
        self.GetClassType() == ClassType::kQuadClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:156-156
    pub fn IsRayValue(&self) -> bool {
        self.GetClassType() == ClassType::kRayClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:157-159
    pub fn IsRadialGradientValue(&self) -> bool {
        self.GetClassType() == ClassType::kRadialGradientClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:160-162
    pub fn IsConicGradientValue(&self) -> bool {
        self.GetClassType() == ClassType::kConicGradientClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:163-166
    pub fn IsConstantGradientValue(&self) -> bool {
        self.GetClassType() == ClassType::kConstantGradientClass
            || self.GetClassType() == ClassType::kColorImageClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:167-167
    pub fn IsColorImageValue(&self) -> bool {
        self.GetClassType() == ClassType::kColorImageClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:168-168
    pub fn IsProgressValue(&self) -> bool {
        self.GetClassType() == ClassType::kProgressClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:169-169
    pub fn IsReflectValue(&self) -> bool {
        self.GetClassType() == ClassType::kReflectClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:170-170
    pub fn IsShadowValue(&self) -> bool {
        self.GetClassType() == ClassType::kShadowClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:171-171
    pub fn IsStringValue(&self) -> bool {
        self.GetClassType() == ClassType::kStringClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:172-172
    pub fn IsSuperellipseValue(&self) -> bool {
        self.GetClassType() == ClassType::kSuperellipseClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:173-173
    pub fn IsSymbolsValue(&self) -> bool {
        self.GetClassType() == ClassType::kSymbolsClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:174-174
    pub fn IsURIValue(&self) -> bool {
        self.GetClassType() == ClassType::kURIClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:175-175
    pub fn IsURLPatternValue(&self) -> bool {
        self.GetClassType() == ClassType::kURLPatternClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:176-178
    pub fn IsLinearTimingFunctionValue(&self) -> bool {
        self.GetClassType() == ClassType::kLinearTimingFunctionClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:179-181
    pub fn IsCubicBezierTimingFunctionValue(&self) -> bool {
        self.GetClassType() == ClassType::kCubicBezierTimingFunctionClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:182-184
    pub fn IsStepsTimingFunctionValue(&self) -> bool {
        self.GetClassType() == ClassType::kStepsTimingFunctionClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:185-187
    pub fn IsGridTemplateAreasValue(&self) -> bool {
        self.GetClassType() == ClassType::kGridTemplateAreasClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:188-190
    pub fn IsContentDistributionValue(&self) -> bool {
        self.GetClassType() == ClassType::kCSSContentDistributionClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:191-191
    pub fn IsUnicodeRangeValue(&self) -> bool {
        self.GetClassType() == ClassType::kUnicodeRangeClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:192-194
    pub fn IsGridLineNamesValue(&self) -> bool {
        self.GetClassType() == ClassType::kGridLineNamesClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:195-197
    pub fn IsUnparsedDeclaration(&self) -> bool {
        self.GetClassType() == ClassType::kUnparsedDeclarationClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:198-200
    pub fn IsGridAutoRepeatValue(&self) -> bool {
        self.GetClassType() == ClassType::kGridAutoRepeatClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:201-203
    pub fn IsGridIntegerRepeatValue(&self) -> bool {
        self.GetClassType() == ClassType::kGridIntegerRepeatClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:204-206
    pub fn IsGridRepeatValue(&self) -> bool {
        self.IsGridAutoRepeatValue() || self.IsGridIntegerRepeatValue()
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:207-209
    pub fn IsPendingSubstitutionValue(&self) -> bool {
        self.GetClassType() == ClassType::kPendingSubstitutionValueClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:210-212
    pub fn IsPendingSystemFontValue(&self) -> bool {
        self.GetClassType() == ClassType::kPendingSystemFontValueClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:213-216
    pub fn IsInvalidVariableValue(&self) -> bool {
        self.GetClassType() == ClassType::kInvalidVariableValueClass
            || self.GetClassType() == ClassType::kCyclicVariableValueClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:217-219
    pub fn IsCyclicVariableValue(&self) -> bool {
        self.GetClassType() == ClassType::kCyclicVariableValueClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:220-220
    pub fn IsFlipRevertValue(&self) -> bool {
        self.GetClassType() == ClassType::kFlipRevertClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:221-221
    pub fn IsAlternateValue(&self) -> bool {
        self.GetClassType() == ClassType::kAlternateClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:222-222
    pub fn IsAxisValue(&self) -> bool {
        self.GetClassType() == ClassType::kAxisClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:223-225
    pub fn IsShorthandWrapperValue(&self) -> bool {
        self.GetClassType() == ClassType::kKeyframeShorthandClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:226-228
    pub fn IsInitialColorValue(&self) -> bool {
        self.GetClassType() == ClassType::kInitialColorValueClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:229-231
    pub fn IsLightDarkValuePair(&self) -> bool {
        self.GetClassType() == ClassType::kLightDarkValuePairClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:233-233
    pub fn IsScrollValue(&self) -> bool {
        self.GetClassType() == ClassType::kScrollClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:234-234
    pub fn IsViewValue(&self) -> bool {
        self.GetClassType() == ClassType::kViewClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:235-235
    pub fn IsRatioValue(&self) -> bool {
        self.GetClassType() == ClassType::kRatioClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:237-237
    pub fn IsRepeatStyleValue(&self) -> bool {
        self.GetClassType() == ClassType::kRepeatStyleClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:239-241
    pub fn IsRelativeColorValue(&self) -> bool {
        self.GetClassType() == ClassType::kRelativeColorClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:245-247
    pub fn IsUnresolvedColorValue(&self) -> bool {
        self.GetClassType() == ClassType::kUnresolvedColorClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:249-249
    pub fn IsRepeatValue(&self) -> bool {
        self.GetClassType() == ClassType::kRepeatClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:265-265
    pub fn IsScopedValue(&self) -> bool {
        !self.state_.NeedsTreeScopePopulation()
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:267-269
    pub fn IsTriggerAttachmentValue(&self) -> bool {
        self.GetClassType() == ClassType::kTriggerAttachmentClass
    }
    // cpp: third_party/blink/renderer/core/css/css_value.cc:119-138
    pub fn Create(value: &Length, zoom: f32) -> Rc<Self> {
        match value.GetType() {
            LengthType::kAuto
            | LengthType::kMinContent
            | LengthType::kMaxContent
            | LengthType::kStretch
            | LengthType::kFitContent
            | LengthType::kContent => D::CreateIdentifierFromLength(value),
            LengthType::kPercent
            | LengthType::kFixed
            | LengthType::kCalculated
            | LengthType::kFlex => D::CreatePrimitiveFromLength(value, zoom),
            LengthType::kMinIntrinsic | LengthType::kNone | LengthType::kOverlapJoin => {
                unreachable!("NOTREACHED: length type has no CSSValue representation")
            }
        }
    }
    fn ValueListView(&self) -> &D::CSSValueList {
        match &self.payload_ {
            CSSValuePayload::kValueListClass(v) => v,
            CSSValuePayload::kFunctionClass(v) => v.AsValueList(),
            CSSValuePayload::kImageSetClass(v) => v.AsValueList(),
            CSSValuePayload::kGridLineNamesClass(v) => v.AsValueList(),
            CSSValuePayload::kGridAutoRepeatClass(v) => v.AsValueList(),
            CSSValuePayload::kGridIntegerRepeatClass(v) => v.AsValueList(),
            CSSValuePayload::kAxisClass(v) => v.AsValueList(),
            _ => unreachable!("CSSValueList cast requires IsValueList"),
        }
    }
    // cpp: third_party/blink/renderer/core/css/css_value.cc:140-159
    pub fn HasFailedOrCanceledSubresources(&self) -> bool {
        if self.IsValueList() {
            return self.ValueListView().HasFailedOrCanceledSubresources();
        }
        match &self.payload_ {
            CSSValuePayload::kFontFaceSrcClass(v) => v.HasFailedOrCanceledSubresources(),
            CSSValuePayload::kImageClass(v) => v.HasFailedOrCanceledSubresources(),
            CSSValuePayload::kCrossfadeClass(v) => v.HasFailedOrCanceledSubresources(),
            CSSValuePayload::kImageSetClass(v) => v.HasFailedOrCanceledSubresources(),
            _ => false,
        }
    }
    // cpp: third_party/blink/renderer/core/css/css_value.cc:161-166
    pub fn MayContainUrl(&self) -> bool {
        if self.IsValueList() {
            return self.ValueListView().MayContainUrl();
        }
        self.IsImageValue() || self.IsURIValue()
    }
    // cpp: third_party/blink/renderer/core/css/css_value.cc:168-182
    pub fn ReResolveUrl(&self, document: &D::Document) {
        match &self.payload_ {
            // The image adapter binds source ReResolveURL (uppercase URL).
            CSSValuePayload::kImageClass(v) => v.ReResolveUrl(document),
            CSSValuePayload::kURIClass(v) => v.ReResolveUrl(document),
            _ if self.IsValueList() => self.ValueListView().ReResolveUrl(document),
            _ => {}
        }
    }
    // cpp: third_party/blink/renderer/core/css/css_value.cc:393-574
    pub fn CssText(&self) -> String {
        match &self.payload_ {
            CSSValuePayload::kAxisClass(v) => v.CustomCSSText(),
            CSSValuePayload::kBasicShapeCircleClass(v) => v.CustomCSSText(),
            CSSValuePayload::kBasicShapeEllipseClass(v) => v.CustomCSSText(),
            CSSValuePayload::kBasicShapePolygonClass(v) => v.CustomCSSText(),
            CSSValuePayload::kBasicShapeInsetClass(v) => v.CustomCSSText(),
            CSSValuePayload::kBasicShapeRectClass(v) => v.CustomCSSText(),
            CSSValuePayload::kBasicShapeXYWHClass(v) => v.CustomCSSText(),
            CSSValuePayload::kBorderImageSliceClass(v) => v.CustomCSSText(),
            CSSValuePayload::kAlphaColorClass(v) => v.CustomCSSText(),
            CSSValuePayload::kColorClass(v) => v.CustomCSSText(),
            CSSValuePayload::kColorMixClass(v) => v.CustomCSSText(),
            CSSValuePayload::kContrastColorClass(v) => v.CustomCSSText(),
            CSSValuePayload::kUnresolvedColorClass(v) => v.CustomCSSText(),
            CSSValuePayload::kCounterClass(v) => v.CustomCSSText(),
            CSSValuePayload::kCounterContentClass(v) => v.CustomCSSText(),
            CSSValuePayload::kCursorImageClass(v) => v.CustomCSSText(),
            CSSValuePayload::kDynamicRangeLimitMixClass(v) => v.CustomCSSText(),
            CSSValuePayload::kFontFaceSrcClass(v) => v.CustomCSSText(),
            CSSValuePayload::kFontFamilyClass(v) => v.CustomCSSText(),
            CSSValuePayload::kFontFeatureClass(v) => v.CustomCSSText(),
            CSSValuePayload::kFontStyleRangeClass(v) => v.CustomCSSText(),
            CSSValuePayload::kFontVariationClass(v) => v.CustomCSSText(),
            CSSValuePayload::kAlternateClass(v) => v.CustomCSSText(),
            CSSValuePayload::kFunctionClass(v) => v.CustomCSSText(),
            CSSValuePayload::kLayoutFunctionClass(v) => v.CustomCSSText(),
            CSSValuePayload::kLinearGradientClass(v) => v.CustomCSSText(),
            CSSValuePayload::kRadialGradientClass(v) => v.CustomCSSText(),
            CSSValuePayload::kConicGradientClass(v) => v.CustomCSSText(),
            CSSValuePayload::kConstantGradientClass(v) => v.CustomCSSText(),
            CSSValuePayload::kColorImageClass(v) => v.CustomCSSText(),
            CSSValuePayload::kCrossfadeClass(v) => v.CustomCSSText(),
            CSSValuePayload::kPaintClass(v) => v.CustomCSSText(),
            CSSValuePayload::kCustomIdentClass(v) => v.CustomCSSText(),
            CSSValuePayload::kImageClass(v) => v.CustomCSSText(),
            CSSValuePayload::kInheritedClass(v) => v.CustomCSSText(),
            CSSValuePayload::kUnsetClass(v) => v.CustomCSSText(),
            CSSValuePayload::kRevertClass(v) => v.CustomCSSText(),
            CSSValuePayload::kRevertLayerClass(v) => v.CustomCSSText(),
            CSSValuePayload::kRevertRuleClass(v) => v.CustomCSSText(),
            CSSValuePayload::kInitialClass(v) => v.CustomCSSText(),
            CSSValuePayload::kGridAutoRepeatClass(v) => v.CustomCSSText(),
            CSSValuePayload::kGridIntegerRepeatClass(v) => v.CustomCSSText(),
            CSSValuePayload::kGridLineNamesClass(v) => v.CustomCSSText(),
            CSSValuePayload::kGridTemplateAreasClass(v) => v.CustomCSSText(),
            CSSValuePayload::kBasicShapePathClass(v) => v.CustomCSSText(),
            CSSValuePayload::kBasicShapeShapeClass(v) => v.CustomCSSText(),
            CSSValuePayload::kSuperellipseClass(v) => v.CustomCSSText(),
            CSSValuePayload::kNumericLiteralClass(v) => v.CustomCSSText(),
            CSSValuePayload::kMathFunctionClass(v) => v.CustomCSSText(),
            CSSValuePayload::kRayClass(v) => v.CustomCSSText(),
            CSSValuePayload::kIdentifierClass(v) => v.CustomCSSText(),
            CSSValuePayload::kScopedKeywordClass(v) => v.CustomCSSText(),
            CSSValuePayload::kKeyframeShorthandClass(v) => v.CustomCSSText(),
            CSSValuePayload::kInitialColorValueClass(v) => v.CustomCSSText(),
            CSSValuePayload::kQuadClass(v) => v.CustomCSSText(),
            CSSValuePayload::kReflectClass(v) => v.CustomCSSText(),
            CSSValuePayload::kShadowClass(v) => v.CustomCSSText(),
            CSSValuePayload::kStringClass(v) => v.CustomCSSText(),
            CSSValuePayload::kSymbolsClass(v) => v.CustomCSSText(),
            CSSValuePayload::kProgressClass(v) => v.CustomCSSText(),
            CSSValuePayload::kLinearTimingFunctionClass(v) => v.CustomCSSText(),
            CSSValuePayload::kCubicBezierTimingFunctionClass(v) => v.CustomCSSText(),
            CSSValuePayload::kStepsTimingFunctionClass(v) => v.CustomCSSText(),
            CSSValuePayload::kUnicodeRangeClass(v) => v.CustomCSSText(),
            CSSValuePayload::kURIClass(v) => v.CustomCSSText(),
            CSSValuePayload::kURLPatternClass(v) => v.CustomCSSText(),
            CSSValuePayload::kValuePairClass(v) => v.CustomCSSText(),
            CSSValuePayload::kValueListClass(v) => v.CustomCSSText(),
            CSSValuePayload::kImageSetTypeClass(v) => v.CustomCSSText(),
            CSSValuePayload::kImageSetOptionClass(v) => v.CustomCSSText(),
            CSSValuePayload::kImageSetClass(v) => v.CustomCSSText(),
            CSSValuePayload::kCSSContentDistributionClass(v) => v.CustomCSSText(),
            CSSValuePayload::kUnparsedDeclarationClass(v) => v.CustomCSSText(),
            CSSValuePayload::kPendingSubstitutionValueClass(v) => v.CustomCSSText(),
            CSSValuePayload::kPendingSystemFontValueClass(v) => v.CustomCSSText(),
            CSSValuePayload::kInvalidVariableValueClass(v) => v.CustomCSSText(),
            CSSValuePayload::kCyclicVariableValueClass(v) => v.CustomCSSText(),
            CSSValuePayload::kFlipRevertClass(v) => v.CustomCSSText(),
            CSSValuePayload::kLightDarkValuePairClass(v) => v.CustomCSSText(),
            CSSValuePayload::kParamValuePairClass(v) => v.CustomCSSText(),
            CSSValuePayload::kScrollClass(v) => v.CustomCSSText(),
            CSSValuePayload::kViewClass(v) => v.CustomCSSText(),
            CSSValuePayload::kRatioClass(v) => v.CustomCSSText(),
            CSSValuePayload::kPaletteMixClass(v) => v.CustomCSSText(),
            CSSValuePayload::kRepeatStyleClass(v) => v.CustomCSSText(),
            CSSValuePayload::kRelativeColorClass(v) => v.CustomCSSText(),
            CSSValuePayload::kRepeatClass(v) => v.CustomCSSText(),
            CSSValuePayload::kTriggerAttachmentClass(v) => v.CustomCSSText(),
        }
    }
    // cpp: third_party/blink/renderer/core/css/css_value.cc:576-694
    pub fn Hash(&self) -> u32 {
        match &self.payload_ {
            CSSValuePayload::kColorClass(v) => HashInts(self.GetClassType() as u32, v.CustomHash()),
            CSSValuePayload::kCSSContentDistributionClass(v) => {
                HashInts(self.GetClassType() as u32, v.CustomHash())
            }
            CSSValuePayload::kCustomIdentClass(v) => {
                HashInts(self.GetClassType() as u32, v.CustomHash())
            }
            CSSValuePayload::kIdentifierClass(v) => {
                HashInts(self.GetClassType() as u32, v.CustomHash())
            }
            CSSValuePayload::kNumericLiteralClass(v) => {
                HashInts(self.GetClassType() as u32, v.CustomHash())
            }
            CSSValuePayload::kBasicShapePathClass(v) => {
                HashInts(self.GetClassType() as u32, v.CustomHash())
            }
            CSSValuePayload::kStringClass(v) => {
                HashInts(self.GetClassType() as u32, v.CustomHash())
            }
            CSSValuePayload::kUnparsedDeclarationClass(v) => {
                HashInts(self.GetClassType() as u32, v.CustomHash())
            }
            CSSValuePayload::kValueListClass(v) => {
                HashInts(self.GetClassType() as u32, v.CustomHash())
            }
            CSSValuePayload::kValuePairClass(v) => {
                HashInts(self.GetClassType() as u32, v.CustomHash())
            }
            CSSValuePayload::kSuperellipseClass(v) => {
                HashInts(self.GetClassType() as u32, v.CustomHash())
            }
            CSSValuePayload::kInheritedClass(..)
            | CSSValuePayload::kInitialClass(..)
            | CSSValuePayload::kUnsetClass(..)
            | CSSValuePayload::kRevertClass(..)
            | CSSValuePayload::kRevertLayerClass(..)
            | CSSValuePayload::kRevertRuleClass(..) => HashInt(self.GetClassType() as u32),
            CSSValuePayload::kMathFunctionClass(..)
            | CSSValuePayload::kScopedKeywordClass(..)
            | CSSValuePayload::kAlphaColorClass(..)
            | CSSValuePayload::kColorMixClass(..)
            | CSSValuePayload::kContrastColorClass(..)
            | CSSValuePayload::kCounterClass(..)
            | CSSValuePayload::kCounterContentClass(..)
            | CSSValuePayload::kSymbolsClass(..)
            | CSSValuePayload::kQuadClass(..)
            | CSSValuePayload::kURIClass(..)
            | CSSValuePayload::kURLPatternClass(..)
            | CSSValuePayload::kLightDarkValuePairClass(..)
            | CSSValuePayload::kParamValuePairClass(..)
            | CSSValuePayload::kScrollClass(..)
            | CSSValuePayload::kViewClass(..)
            | CSSValuePayload::kRatioClass(..)
            | CSSValuePayload::kRelativeColorClass(..)
            | CSSValuePayload::kBasicShapeCircleClass(..)
            | CSSValuePayload::kBasicShapeEllipseClass(..)
            | CSSValuePayload::kBasicShapePolygonClass(..)
            | CSSValuePayload::kBasicShapeInsetClass(..)
            | CSSValuePayload::kBasicShapeRectClass(..)
            | CSSValuePayload::kBasicShapeXYWHClass(..)
            | CSSValuePayload::kImageClass(..)
            | CSSValuePayload::kCursorImageClass(..)
            | CSSValuePayload::kCrossfadeClass(..)
            | CSSValuePayload::kPaintClass(..)
            | CSSValuePayload::kLinearGradientClass(..)
            | CSSValuePayload::kRadialGradientClass(..)
            | CSSValuePayload::kConicGradientClass(..)
            | CSSValuePayload::kConstantGradientClass(..)
            | CSSValuePayload::kColorImageClass(..)
            | CSSValuePayload::kProgressClass(..)
            | CSSValuePayload::kLinearTimingFunctionClass(..)
            | CSSValuePayload::kCubicBezierTimingFunctionClass(..)
            | CSSValuePayload::kStepsTimingFunctionClass(..)
            | CSSValuePayload::kBorderImageSliceClass(..)
            | CSSValuePayload::kDynamicRangeLimitMixClass(..)
            | CSSValuePayload::kFontFeatureClass(..)
            | CSSValuePayload::kFontFaceSrcClass(..)
            | CSSValuePayload::kFontFamilyClass(..)
            | CSSValuePayload::kFontStyleRangeClass(..)
            | CSSValuePayload::kFontVariationClass(..)
            | CSSValuePayload::kAlternateClass(..)
            | CSSValuePayload::kReflectClass(..)
            | CSSValuePayload::kShadowClass(..)
            | CSSValuePayload::kBasicShapeShapeClass(..)
            | CSSValuePayload::kUnicodeRangeClass(..)
            | CSSValuePayload::kGridTemplateAreasClass(..)
            | CSSValuePayload::kPaletteMixClass(..)
            | CSSValuePayload::kRayClass(..)
            | CSSValuePayload::kPendingSubstitutionValueClass(..)
            | CSSValuePayload::kPendingSystemFontValueClass(..)
            | CSSValuePayload::kInvalidVariableValueClass(..)
            | CSSValuePayload::kCyclicVariableValueClass(..)
            | CSSValuePayload::kFlipRevertClass(..)
            | CSSValuePayload::kLayoutFunctionClass(..)
            | CSSValuePayload::kKeyframeShorthandClass(..)
            | CSSValuePayload::kInitialColorValueClass(..)
            | CSSValuePayload::kImageSetOptionClass(..)
            | CSSValuePayload::kImageSetTypeClass(..)
            | CSSValuePayload::kRepeatStyleClass(..)
            | CSSValuePayload::kFunctionClass(..)
            | CSSValuePayload::kImageSetClass(..)
            | CSSValuePayload::kGridLineNamesClass(..)
            | CSSValuePayload::kGridAutoRepeatClass(..)
            | CSSValuePayload::kGridIntegerRepeatClass(..)
            | CSSValuePayload::kAxisClass(..)
            | CSSValuePayload::kRepeatClass(..)
            | CSSValuePayload::kUnresolvedColorClass(..)
            | CSSValuePayload::kTriggerAttachmentClass(..) => self as *const Self as usize as u32,
        }
    }
    // cpp: third_party/blink/renderer/core/css/css_value.h:259-264
    pub fn EnsureScopedValue<'a>(&'a self, tree_scope: Option<&'a D::TreeScope>) -> &'a Self {
        if !self.state_.NeedsTreeScopePopulation() {
            return self;
        }
        self.PopulateWithTreeScope(tree_scope)
    }
    // cpp: third_party/blink/renderer/core/css/css_value.cc:696-717
    pub fn PopulateWithTreeScope<'a>(&'a self, scope: Option<&'a D::TreeScope>) -> &'a Self {
        match &self.payload_ {
            CSSValuePayload::kScopedKeywordClass(v) => v.PopulateWithTreeScope(scope),
            CSSValuePayload::kCounterContentClass(v) => v.PopulateWithTreeScope(scope),
            CSSValuePayload::kCustomIdentClass(v) => v.PopulateWithTreeScope(scope),
            CSSValuePayload::kMathFunctionClass(v) => v.PopulateWithTreeScope(scope),
            CSSValuePayload::kValueListClass(v) => v.PopulateWithTreeScope(scope),
            CSSValuePayload::kTriggerAttachmentClass(v) => v.PopulateWithTreeScope(scope),
            _ => unreachable!("NOTREACHED: class cannot populate with tree scope"),
        }
    }
    // cpp: third_party/blink/renderer/core/css/css_value.cc:1185-1334
    pub fn HasRandomFunctions(&self) -> bool {
        match &self.payload_ {
            CSSValuePayload::kMathFunctionClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kUnresolvedColorClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kColorMixClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kRelativeColorClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kAlphaColorClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kContrastColorClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kPaletteMixClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kCustomIdentClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kValueListClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kAxisClass(v) => v.AsValueList().HasRandomFunctions(),
            CSSValuePayload::kFunctionClass(v) => v.AsValueList().HasRandomFunctions(),
            CSSValuePayload::kGridLineNamesClass(v) => v.AsValueList().HasRandomFunctions(),
            CSSValuePayload::kRepeatClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kValuePairClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kLightDarkValuePairClass(v) => v.AsValuePair().HasRandomFunctions(),
            CSSValuePayload::kParamValuePairClass(v) => v.AsValuePair().HasRandomFunctions(),
            CSSValuePayload::kGridIntegerRepeatClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kFontFeatureClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kFontStyleRangeClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kFontVariationClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kLayoutFunctionClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kAlternateClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kTriggerAttachmentClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kSuperellipseClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kPaintClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kCounterClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kCounterContentClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kQuadClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kScrollClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kViewClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kRatioClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kCrossfadeClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kBorderImageSliceClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kReflectClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kImageSetOptionClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kImageSetClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kRepeatStyleClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kDynamicRangeLimitMixClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kBasicShapeCircleClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kBasicShapeEllipseClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kBasicShapePolygonClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kBasicShapeInsetClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kBasicShapeRectClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kBasicShapeXYWHClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kBasicShapeShapeClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kLinearGradientClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kRadialGradientClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kConicGradientClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kConstantGradientClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kColorImageClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kStepsTimingFunctionClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kShadowClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kRayClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kUnparsedDeclarationClass(v) => v.HasRandomFunctions(),
            CSSValuePayload::kInheritedClass(..) => false,
            CSSValuePayload::kInitialClass(..) => false,
            CSSValuePayload::kUnsetClass(..) => false,
            CSSValuePayload::kRevertClass(..) => false,
            CSSValuePayload::kRevertLayerClass(..) => false,
            CSSValuePayload::kRevertRuleClass(..) => false,
            CSSValuePayload::kURIClass(..) => false,
            CSSValuePayload::kURLPatternClass(..) => false,
            CSSValuePayload::kColorClass(..) => false,
            CSSValuePayload::kStringClass(..) => false,
            CSSValuePayload::kBasicShapePathClass(..) => false,
            CSSValuePayload::kCSSContentDistributionClass(..) => false,
            CSSValuePayload::kImageClass(..) => false,
            CSSValuePayload::kCursorImageClass(..) => false,
            CSSValuePayload::kProgressClass(..) => false,
            CSSValuePayload::kLinearTimingFunctionClass(..) => false,
            CSSValuePayload::kCubicBezierTimingFunctionClass(..) => false,
            CSSValuePayload::kFontFaceSrcClass(..) => false,
            CSSValuePayload::kFontFamilyClass(..) => false,
            CSSValuePayload::kUnicodeRangeClass(..) => false,
            CSSValuePayload::kGridTemplateAreasClass(..) => false,
            CSSValuePayload::kPendingSubstitutionValueClass(..) => false,
            CSSValuePayload::kPendingSystemFontValueClass(..) => false,
            CSSValuePayload::kInvalidVariableValueClass(..) => false,
            CSSValuePayload::kCyclicVariableValueClass(..) => false,
            CSSValuePayload::kFlipRevertClass(..) => false,
            CSSValuePayload::kKeyframeShorthandClass(..) => false,
            CSSValuePayload::kInitialColorValueClass(..) => false,
            CSSValuePayload::kImageSetTypeClass(..) => false,
            CSSValuePayload::kGridAutoRepeatClass(..) => false,
            CSSValuePayload::kScopedKeywordClass(..) => false,
            CSSValuePayload::kNumericLiteralClass(..) => false,
            CSSValuePayload::kSymbolsClass(..) => false,
            CSSValuePayload::kIdentifierClass(..) => false,
        }
    }
}
// cpp: third_party/blink/renderer/core/css/css_value.cc:183-391
impl<D: CSSValueDispatch> PartialEq for CSSValue<D> {
    fn eq(&self, other: &Self) -> bool {
        if self.GetClassType() != other.GetClassType() {
            return false;
        }
        match (&self.payload_, &other.payload_) {
            (CSSValuePayload::kAxisClass(a), CSSValuePayload::kAxisClass(b)) => a.Equals(b),
            (
                CSSValuePayload::kBasicShapeCircleClass(a),
                CSSValuePayload::kBasicShapeCircleClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kBasicShapeEllipseClass(a),
                CSSValuePayload::kBasicShapeEllipseClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kBasicShapePolygonClass(a),
                CSSValuePayload::kBasicShapePolygonClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kBasicShapeInsetClass(a),
                CSSValuePayload::kBasicShapeInsetClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kBasicShapeRectClass(a),
                CSSValuePayload::kBasicShapeRectClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kBasicShapeXYWHClass(a),
                CSSValuePayload::kBasicShapeXYWHClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kBorderImageSliceClass(a),
                CSSValuePayload::kBorderImageSliceClass(b),
            ) => a.Equals(b),
            (CSSValuePayload::kAlphaColorClass(a), CSSValuePayload::kAlphaColorClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kColorClass(a), CSSValuePayload::kColorClass(b)) => a.Equals(b),
            (CSSValuePayload::kColorMixClass(a), CSSValuePayload::kColorMixClass(b)) => a.Equals(b),
            (CSSValuePayload::kContrastColorClass(a), CSSValuePayload::kContrastColorClass(b)) => {
                a.Equals(b)
            }
            (
                CSSValuePayload::kUnresolvedColorClass(a),
                CSSValuePayload::kUnresolvedColorClass(b),
            ) => a.Equals(b),
            (CSSValuePayload::kCounterClass(a), CSSValuePayload::kCounterClass(b)) => a.Equals(b),
            (
                CSSValuePayload::kCounterContentClass(a),
                CSSValuePayload::kCounterContentClass(b),
            ) => a.Equals(b),
            (CSSValuePayload::kCursorImageClass(a), CSSValuePayload::kCursorImageClass(b)) => {
                a.Equals(b)
            }
            (
                CSSValuePayload::kDynamicRangeLimitMixClass(a),
                CSSValuePayload::kDynamicRangeLimitMixClass(b),
            ) => a.Equals(b),
            (CSSValuePayload::kFontFaceSrcClass(a), CSSValuePayload::kFontFaceSrcClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kFontFamilyClass(a), CSSValuePayload::kFontFamilyClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kFontFeatureClass(a), CSSValuePayload::kFontFeatureClass(b)) => {
                a.Equals(b)
            }
            (
                CSSValuePayload::kFontStyleRangeClass(a),
                CSSValuePayload::kFontStyleRangeClass(b),
            ) => a.Equals(b),
            (CSSValuePayload::kFontVariationClass(a), CSSValuePayload::kFontVariationClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kAlternateClass(a), CSSValuePayload::kAlternateClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kFunctionClass(a), CSSValuePayload::kFunctionClass(b)) => a.Equals(b),
            (
                CSSValuePayload::kLayoutFunctionClass(a),
                CSSValuePayload::kLayoutFunctionClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kLinearGradientClass(a),
                CSSValuePayload::kLinearGradientClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kRadialGradientClass(a),
                CSSValuePayload::kRadialGradientClass(b),
            ) => a.Equals(b),
            (CSSValuePayload::kConicGradientClass(a), CSSValuePayload::kConicGradientClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kCrossfadeClass(a), CSSValuePayload::kCrossfadeClass(b)) => {
                a.Equals(b)
            }
            (
                CSSValuePayload::kConstantGradientClass(a),
                CSSValuePayload::kConstantGradientClass(b),
            ) => a.Equals(b),
            (CSSValuePayload::kColorImageClass(a), CSSValuePayload::kColorImageClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kPaintClass(a), CSSValuePayload::kPaintClass(b)) => a.Equals(b),
            (CSSValuePayload::kCustomIdentClass(a), CSSValuePayload::kCustomIdentClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kImageClass(a), CSSValuePayload::kImageClass(b)) => a.Equals(b),
            (CSSValuePayload::kInheritedClass(a), CSSValuePayload::kInheritedClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kInitialClass(a), CSSValuePayload::kInitialClass(b)) => a.Equals(b),
            (CSSValuePayload::kUnsetClass(a), CSSValuePayload::kUnsetClass(b)) => a.Equals(b),
            (CSSValuePayload::kRevertClass(a), CSSValuePayload::kRevertClass(b)) => a.Equals(b),
            (CSSValuePayload::kRevertLayerClass(a), CSSValuePayload::kRevertLayerClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kRevertRuleClass(a), CSSValuePayload::kRevertRuleClass(b)) => {
                a.Equals(b)
            }
            (
                CSSValuePayload::kGridAutoRepeatClass(a),
                CSSValuePayload::kGridAutoRepeatClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kGridIntegerRepeatClass(a),
                CSSValuePayload::kGridIntegerRepeatClass(b),
            ) => a.Equals(b),
            (CSSValuePayload::kGridLineNamesClass(a), CSSValuePayload::kGridLineNamesClass(b)) => {
                a.Equals(b)
            }
            (
                CSSValuePayload::kGridTemplateAreasClass(a),
                CSSValuePayload::kGridTemplateAreasClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kBasicShapePathClass(a),
                CSSValuePayload::kBasicShapePathClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kBasicShapeShapeClass(a),
                CSSValuePayload::kBasicShapeShapeClass(b),
            ) => a.Equals(b),
            (CSSValuePayload::kSuperellipseClass(a), CSSValuePayload::kSuperellipseClass(b)) => {
                a.Equals(b)
            }
            (
                CSSValuePayload::kNumericLiteralClass(a),
                CSSValuePayload::kNumericLiteralClass(b),
            ) => a.Equals(b),
            (CSSValuePayload::kMathFunctionClass(a), CSSValuePayload::kMathFunctionClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kRayClass(a), CSSValuePayload::kRayClass(b)) => a.Equals(b),
            (CSSValuePayload::kIdentifierClass(a), CSSValuePayload::kIdentifierClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kScopedKeywordClass(a), CSSValuePayload::kScopedKeywordClass(b)) => {
                a.Equals(b)
            }
            (
                CSSValuePayload::kKeyframeShorthandClass(a),
                CSSValuePayload::kKeyframeShorthandClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kInitialColorValueClass(a),
                CSSValuePayload::kInitialColorValueClass(b),
            ) => a.Equals(b),
            (CSSValuePayload::kQuadClass(a), CSSValuePayload::kQuadClass(b)) => a.Equals(b),
            (CSSValuePayload::kReflectClass(a), CSSValuePayload::kReflectClass(b)) => a.Equals(b),
            (CSSValuePayload::kShadowClass(a), CSSValuePayload::kShadowClass(b)) => a.Equals(b),
            (CSSValuePayload::kStringClass(a), CSSValuePayload::kStringClass(b)) => a.Equals(b),
            (CSSValuePayload::kSymbolsClass(a), CSSValuePayload::kSymbolsClass(b)) => a.Equals(b),
            (CSSValuePayload::kProgressClass(a), CSSValuePayload::kProgressClass(b)) => a.Equals(b),
            (
                CSSValuePayload::kLinearTimingFunctionClass(a),
                CSSValuePayload::kLinearTimingFunctionClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kCubicBezierTimingFunctionClass(a),
                CSSValuePayload::kCubicBezierTimingFunctionClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kStepsTimingFunctionClass(a),
                CSSValuePayload::kStepsTimingFunctionClass(b),
            ) => a.Equals(b),
            (CSSValuePayload::kUnicodeRangeClass(a), CSSValuePayload::kUnicodeRangeClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kURIClass(a), CSSValuePayload::kURIClass(b)) => a.Equals(b),
            (CSSValuePayload::kURLPatternClass(a), CSSValuePayload::kURLPatternClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kValueListClass(a), CSSValuePayload::kValueListClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kValuePairClass(a), CSSValuePayload::kValuePairClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kImageSetTypeClass(a), CSSValuePayload::kImageSetTypeClass(b)) => {
                a.Equals(b)
            }
            (
                CSSValuePayload::kImageSetOptionClass(a),
                CSSValuePayload::kImageSetOptionClass(b),
            ) => a.Equals(b),
            (CSSValuePayload::kImageSetClass(a), CSSValuePayload::kImageSetClass(b)) => a.Equals(b),
            (
                CSSValuePayload::kCSSContentDistributionClass(a),
                CSSValuePayload::kCSSContentDistributionClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kUnparsedDeclarationClass(a),
                CSSValuePayload::kUnparsedDeclarationClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kPendingSubstitutionValueClass(a),
                CSSValuePayload::kPendingSubstitutionValueClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kPendingSystemFontValueClass(a),
                CSSValuePayload::kPendingSystemFontValueClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kInvalidVariableValueClass(a),
                CSSValuePayload::kInvalidVariableValueClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kCyclicVariableValueClass(a),
                CSSValuePayload::kCyclicVariableValueClass(b),
            ) => a.Equals(b),
            (CSSValuePayload::kFlipRevertClass(a), CSSValuePayload::kFlipRevertClass(b)) => {
                a.Equals(b)
            }
            (
                CSSValuePayload::kLightDarkValuePairClass(a),
                CSSValuePayload::kLightDarkValuePairClass(b),
            ) => a.Equals(b),
            (
                CSSValuePayload::kParamValuePairClass(a),
                CSSValuePayload::kParamValuePairClass(b),
            ) => a.Equals(b),
            (CSSValuePayload::kScrollClass(a), CSSValuePayload::kScrollClass(b)) => a.Equals(b),
            (
                CSSValuePayload::kTriggerAttachmentClass(a),
                CSSValuePayload::kTriggerAttachmentClass(b),
            ) => a.Equals(b),
            (CSSValuePayload::kViewClass(a), CSSValuePayload::kViewClass(b)) => a.Equals(b),
            (CSSValuePayload::kRatioClass(a), CSSValuePayload::kRatioClass(b)) => a.Equals(b),
            (CSSValuePayload::kPaletteMixClass(a), CSSValuePayload::kPaletteMixClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kRepeatStyleClass(a), CSSValuePayload::kRepeatStyleClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kRelativeColorClass(a), CSSValuePayload::kRelativeColorClass(b)) => {
                a.Equals(b)
            }
            (CSSValuePayload::kRepeatClass(a), CSSValuePayload::kRepeatClass(b)) => a.Equals(b),
            _ => unreachable!("same ClassType must contain matching typed payloads"),
        }
    }
}

fn ValuesEquivalent<V: PartialEq>(first: Option<&V>, second: Option<&V>) -> bool {
    match (first, second) {
        (Some(a), Some(b)) => std::ptr::eq(a, b) || a == b,
        (None, None) => true,
        _ => false,
    }
}
// cpp: third_party/blink/renderer/core/css/css_value.h:449-464
pub fn CompareCSSValueVector<V: PartialEq>(
    first: &[Option<Rc<V>>],
    second: &[Option<Rc<V>>],
) -> bool {
    if first.len() != second.len() {
        return false;
    }
    for (a, b) in first.iter().zip(second) {
        if !ValuesEquivalent(a.as_deref(), b.as_deref()) {
            return false;
        }
    }
    true
}
// cpp: third_party/blink/renderer/core/css/css_value.h:467-478
// Source implementation skips leading nulls, despite its stronger comment.
// Once a non-null first value exists, any subsequent null is unequal.
pub fn AllCSSValuesEqual<'a, D: CSSValueDispatch + 'a>(
    values: impl IntoIterator<Item = Option<&'a CSSValue<D>>>,
) -> bool {
    let mut first = None;
    for value in values {
        if first.is_none() {
            first = value;
        } else if !ValuesEquivalent(first, value) {
            return false;
        }
    }
    first.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    // Fixtures verify only the base dispatcher and expose which required child
    // operation was called. They are not implementations of CSS derived values.
    struct Fixture {
        label: &'static str,
        text: String,
        number: f64,
        hash: u32,
        failed: bool,
        url: bool,
        random: bool,
        list_base: Option<Box<Fixture>>,
        pair_base: Option<Box<Fixture>>,
        scoped: Option<Rc<CSSValue<Dispatch>>>,
        scope_arguments: RefCell<Vec<Option<u32>>>,
        factory_length_type: Option<LengthType>,
    }
    impl Fixture {
        fn new(label: &'static str) -> Self {
            Self {
                label,
                text: String::FromUtf8(label.as_bytes()),
                number: 1.0,
                hash: 0x12345678,
                failed: false,
                url: false,
                random: false,
                list_base: None,
                pair_base: None,
                scoped: None,
                scope_arguments: RefCell::new(Vec::new()),
                factory_length_type: None,
            }
        }
    }
    impl CSSValueSubclass for Fixture {
        fn CustomCSSText(&self) -> String {
            self.text.clone()
        }
        fn Equals(&self, other: &Self) -> bool {
            self.text == other.text && self.number == other.number
        }
    }
    impl CSSValueCustomHash for Fixture {
        fn CustomHash(&self) -> u32 {
            self.hash
        }
    }
    impl CSSValueSubresources for Fixture {
        fn HasFailedOrCanceledSubresources(&self) -> bool {
            self.failed
        }
    }
    impl CSSValueRandom for Fixture {
        fn HasRandomFunctions(&self) -> bool {
            self.random
        }
    }
    impl CSSValueUrl<RefCell<Vec<&'static str>>> for Fixture {
        fn ReResolveUrl(&self, document: &RefCell<Vec<&'static str>>) {
            document.borrow_mut().push(self.label);
        }
    }
    impl CSSValueListUrls for Fixture {
        fn MayContainUrl(&self) -> bool {
            self.url
        }
    }
    impl CSSValueListSubclass<Dispatch> for Fixture {
        fn AsValueList(&self) -> &Fixture {
            self.list_base
                .as_deref()
                .expect("fixture needs the real base-list projection")
        }
    }
    impl CSSValuePairSubclass<Dispatch> for Fixture {
        fn AsValuePair(&self) -> &Fixture {
            self.pair_base
                .as_deref()
                .expect("fixture needs the real base-pair projection")
        }
    }
    impl CSSValueTreeScope<Dispatch> for Fixture {
        fn PopulateWithTreeScope<'a>(&'a self, scope: Option<&'a u32>) -> &'a CSSValue<Dispatch> {
            self.scope_arguments.borrow_mut().push(scope.copied());
            self.scoped
                .as_deref()
                .expect("fixture must supply the actual scoped value")
        }
    }
    struct Dispatch;
    macro_rules! fixture_payloads { ($($name:ident),+ $(,)?) => { $(type $name = Fixture;)+ }; }
    impl CSSValueDispatch for Dispatch {
        type Document = RefCell<Vec<&'static str>>;
        type TreeScope = u32;
        fixture_payloads!(
            CSSNumericLiteralValue,
            CSSMathFunctionValue,
            CSSIdentifierValue,
            CSSScopedKeywordValue,
            CSSColor,
            CSSUnresolvedColorValue,
            CSSColorMixValue,
            CSSAlphaColorValue,
            CSSContrastColorValue,
            CSSCounterValue,
            CSSCounterContentValue,
            CSSQuadValue,
            CSSCustomIdentValue,
            CSSStringValue,
            CSSURIValue,
            CSSURLPatternValue,
            CSSValuePair,
            CSSLightDarkValuePair,
            CSSParamValuePair,
            CSSScrollValue,
            CSSViewValue,
            CSSRatioValue,
            CSSRelativeColorValue,
            CSSBasicShapeCircleValue,
            CSSBasicShapeEllipseValue,
            CSSBasicShapePolygonValue,
            CSSBasicShapeInsetValue,
            CSSBasicShapeRectValue,
            CSSBasicShapeXYWHValue,
            CSSPathValue,
            CSSShapeValue,
            CSSImageValue,
            CSSCursorImageValue,
            CSSCrossfadeValue,
            CSSPaintValue,
            CSSLinearGradientValue,
            CSSRadialGradientValue,
            CSSConicGradientValue,
            CSSConstantGradientValue,
            CSSColorImageValue,
            CSSLinearTimingFunctionValue,
            CSSCubicBezierTimingFunctionValue,
            CSSStepsTimingFunctionValue,
            CSSProgressValue,
            CSSBorderImageSliceValue,
            CSSDynamicRangeLimitMixValue,
            CSSFontFeatureValue,
            CSSFontFaceSrcValue,
            CSSFontFamilyValue,
            CSSFontStyleRangeValue,
            CSSFontVariationValue,
            CSSAlternateValue,
            CSSInheritedValue,
            CSSInitialValue,
            CSSUnsetValue,
            CSSRevertValue,
            CSSRevertLayerValue,
            CSSRevertRuleValue,
            CSSReflectValue,
            CSSShadowValue,
            CSSUnicodeRangeValue,
            CSSGridTemplateAreasValue,
            CSSPaletteMixValue,
            CSSRayValue,
            CSSUnparsedDeclarationValue,
            CSSPendingSubstitutionValue,
            CSSPendingSystemFontValue,
            CSSInvalidVariableValue,
            CSSCyclicVariableValue,
            CSSFlipRevertValue,
            CSSLayoutFunctionValue,
            CSSContentDistributionValue,
            CSSKeyframeShorthandValue,
            CSSInitialColorValue,
            CSSImageSetOptionValue,
            CSSImageSetTypeValue,
            CSSRepeatStyleValue,
            CSSSuperellipseValue,
            CSSSymbolsValue,
            CSSTriggerAttachmentValue,
            CSSRepeatValue,
            CSSValueList,
            CSSFunctionValue,
            CSSImageSetValue,
            CSSBracketedValueList,
            CSSGridAutoRepeatValue,
            CSSGridIntegerRepeatValue,
            CSSAxisValue
        );
        fn CreateIdentifierFromLength(value: &Length) -> Rc<CSSValue<Self>> {
            let mut payload = Fixture::new("identifier factory");
            payload.factory_length_type = Some(value.GetType());
            Rc::new(CSSValue::new(CSSValuePayload::kIdentifierClass(payload)))
        }
        fn CreatePrimitiveFromLength(value: &Length, zoom: f32) -> Rc<CSSValue<Self>> {
            let mut payload = Fixture::new("primitive factory");
            payload.factory_length_type = Some(value.GetType());
            payload.number = f64::from(zoom);
            Rc::new(CSSValue::new(CSSValuePayload::kNumericLiteralClass(
                payload,
            )))
        }
    }
    fn value(payload: CSSValuePayload<Dispatch>) -> CSSValue<Dispatch> {
        CSSValue::new(payload)
    }

    #[test]
    fn class_ranges_flags_and_child_text_preserve_source_semantics() {
        let number = value(CSSValuePayload::kNumericLiteralClass(Fixture::new(
            "numeric text",
        )));
        assert!(number.IsNumericLiteralValue() && number.IsPrimitiveValue());
        assert!(!number.IsIdentifierValue() && !number.IsValueList());
        assert_eq!(number.CssText().Utf8(), "numeric text");
        assert_eq!(number.State().numeric_literal_unit_type_, 0);
        assert_eq!(
            number.State().value_list_separator_,
            ValueListSeparator::kSpaceSeparator as u8
        );
        assert!(!number.State().AllowsNegativePercentageReference());
        assert!(!number.State().WasQuirky());
        assert!(number.IsScopedValue());
        let shape = value(CSSValuePayload::kBasicShapeShapeClass(Fixture::new(
            "shape",
        )));
        assert!(shape.IsBasicShapeValue() && shape.IsShapeValue());
        let ray = value(CSSValuePayload::kRayClass(Fixture::new("ray")));
        assert!(!ray.IsBasicShapeValue());
        let color_image = value(CSSValuePayload::kColorImageClass(Fixture::new("image")));
        assert!(
            color_image.IsGradientValue()
                && color_image.IsConstantGradientValue()
                && color_image.IsImageGeneratorValue()
        );
        let crossfade = value(CSSValuePayload::kCrossfadeClass(Fixture::new("crossfade")));
        assert!(crossfade.IsImageGeneratorValue() && !crossfade.IsGradientValue());
        let revert = value(CSSValuePayload::kRevertRuleClass(Fixture::new(
            "revert-rule",
        )));
        assert!(revert.IsCSSWideKeyword() && revert.IsCascadeDependentKeyword());
        let initial = value(CSSValuePayload::kInitialClass(Fixture::new("initial")));
        assert!(initial.IsCSSWideKeyword() && !initial.IsCascadeDependentKeyword());
        let cyclic = value(CSSValuePayload::kCyclicVariableValueClass(Fixture::new(
            "cyclic",
        )));
        assert!(cyclic.IsInvalidVariableValue() && cyclic.IsCyclicVariableValue());
        let mut math = value(CSSValuePayload::kMathFunctionClass(Fixture::new("math")));
        math.StateMut().SetAllowsNegativePercentageReference(true);
        math.StateMut().SetWasQuirky(true);
        math.StateMut().SetNeedsTreeScopePopulation(true);
        assert_eq!(math.State().flags_, 7);
        math.StateMut().SetWasQuirky(false);
        assert_eq!(math.State().flags_, 3);
        assert!(!math.IsScopedValue());
        assert_eq!(math.GetClassType(), ClassType::kMathFunctionClass);
    }
    #[test]
    fn equals_and_pointer_helpers_preserve_payload_comparison_and_null_precedence() {
        let a = value(CSSValuePayload::kStringClass(Fixture::new("a")));
        let same = value(CSSValuePayload::kStringClass(Fixture::new("a")));
        let different = value(CSSValuePayload::kStringClass(Fixture::new("b")));
        let other_class = value(CSSValuePayload::kIdentifierClass(Fixture::new("a")));
        assert!(a == same);
        assert!(a != different);
        assert!(a != other_class);
        assert!(AllCSSValuesEqual([None, Some(&a), Some(&same)]));
        assert!(!AllCSSValuesEqual([Some(&a), None]));
        assert!(!AllCSSValuesEqual::<Dispatch>([None, None]));
        assert!(!AllCSSValuesEqual::<Dispatch>([]));
        let mut nan = Fixture::new("nan");
        nan.number = f64::NAN;
        let nan = Rc::new(value(CSSValuePayload::kNumericLiteralClass(nan)));
        // CSSValue::operator== delegates even for self; ValuesEquivalent adds
        // the pointer fast path only in the collection helpers.
        assert!(*nan != *nan);
        assert!(CompareCSSValueVector(
            &[Some(nan.clone())],
            &[Some(nan.clone())]
        ));
        assert!(AllCSSValuesEqual([Some(&*nan), Some(&*nan)]));
        assert!(CompareCSSValueVector::<CSSValue<Dispatch>>(
            &[None],
            &[None]
        ));
        assert!(!CompareCSSValueVector::<CSSValue<Dispatch>>(&[None], &[]));
        assert!(!CompareCSSValueVector(&[None], &[Some(nan)]));
    }
    #[test]
    fn hash_dispatch_uses_custom_singleton_and_object_pointer_paths() {
        let number = value(CSSValuePayload::kNumericLiteralClass(Fixture::new(
            "number",
        )));
        let text = value(CSSValuePayload::kStringClass(Fixture::new("text")));
        assert_eq!(
            number.Hash(),
            HashInts(ClassType::kNumericLiteralClass as u32, 0x12345678)
        );
        assert_eq!(
            text.Hash(),
            HashInts(ClassType::kStringClass as u32, 0x12345678)
        );
        let initial = value(CSSValuePayload::kInitialClass(Fixture::new("initial")));
        assert_eq!(initial.Hash(), HashInt(ClassType::kInitialClass as u32));
        let other_initial = value(CSSValuePayload::kInitialClass(Fixture::new(
            "different payload fixture",
        )));
        assert_eq!(initial.Hash(), other_initial.Hash());
        let math = Box::new(value(CSSValuePayload::kMathFunctionClass(Fixture::new(
            "math",
        ))));
        assert_eq!(
            math.Hash(),
            (&*math as *const CSSValue<Dispatch> as usize) as u32
        );
        let function = Box::new(value(CSSValuePayload::kFunctionClass(Fixture::new(
            "function",
        ))));
        assert_eq!(
            function.Hash(),
            (&*function as *const CSSValue<Dispatch> as usize) as u32
        );
    }
    #[test]
    fn url_subresource_and_random_dispatch_preserve_list_and_pair_projections() {
        let mut list_base = Fixture::new("base list");
        list_base.failed = true;
        list_base.url = true;
        list_base.random = true;
        let mut image_set_payload = Fixture::new("image set");
        image_set_payload.failed = false;
        image_set_payload.random = false;
        image_set_payload.list_base = Some(Box::new(list_base));
        let image_set = value(CSSValuePayload::kImageSetClass(image_set_payload));
        assert!(image_set.IsValueList() && !image_set.IsBaseValueList());
        // IsValueList() runs before the image-set-specific resource branch.
        assert!(image_set.HasFailedOrCanceledSubresources());
        assert!(image_set.MayContainUrl());
        // The random-function switch explicitly chooses the image-set child.
        assert!(!image_set.HasRandomFunctions());
        let document = RefCell::new(Vec::new());
        image_set.ReResolveUrl(&document);
        assert_eq!(*document.borrow(), ["base list"]);
        let mut axis_payload = Fixture::new("axis");
        let mut base = Fixture::new("axis base");
        base.random = true;
        axis_payload.list_base = Some(Box::new(base));
        let axis = value(CSSValuePayload::kAxisClass(axis_payload));
        assert!(axis.HasRandomFunctions());
        let mut pair_payload = Fixture::new("light dark child");
        let mut pair_base = Fixture::new("pair base");
        pair_base.random = true;
        pair_payload.pair_base = Some(Box::new(pair_base));
        let pair = value(CSSValuePayload::kLightDarkValuePairClass(pair_payload));
        assert!(pair.HasRandomFunctions());
        assert!(!pair.IsValuePair());
        let mut payload = Fixture::new("numeric fixture");
        payload.random = true;
        let number = value(CSSValuePayload::kNumericLiteralClass(payload));
        assert!(!number.HasRandomFunctions());
        assert!(!number.MayContainUrl());
        number.ReResolveUrl(&document);
        assert_eq!(document.borrow().len(), 1);
        let mut image_payload = Fixture::new("image");
        image_payload.failed = true;
        let image = value(CSSValuePayload::kImageClass(image_payload));
        assert!(image.MayContainUrl() && image.HasFailedOrCanceledSubresources());
        image.ReResolveUrl(&document);
        assert_eq!(*document.borrow(), ["base list", "image"]);
        let uri = value(CSSValuePayload::kURIClass(Fixture::new("uri")));
        assert!(uri.MayContainUrl());
        uri.ReResolveUrl(&document);
        let mut crossfade_payload = Fixture::new("crossfade");
        crossfade_payload.failed = true;
        let crossfade = value(CSSValuePayload::kCrossfadeClass(crossfade_payload));
        assert!(!crossfade.MayContainUrl());
        assert!(crossfade.HasFailedOrCanceledSubresources());
    }
    #[test]
    fn scope_population_preserves_self_fast_path_and_nullable_scope_argument() {
        let scoped = Rc::new(value(CSSValuePayload::kScopedKeywordClass(Fixture::new(
            "scoped",
        ))));
        let mut payload = Fixture::new("unscoped");
        payload.scoped = Some(scoped.clone());
        let mut unscoped = value(CSSValuePayload::kScopedKeywordClass(payload));
        let scope = 123;
        assert!(std::ptr::eq(
            unscoped.EnsureScopedValue(Some(&scope)),
            &unscoped
        ));
        unscoped.StateMut().SetNeedsTreeScopePopulation(true);
        assert!(std::ptr::eq(
            unscoped.EnsureScopedValue(Some(&scope)),
            &*scoped
        ));
        assert!(std::ptr::eq(unscoped.EnsureScopedValue(None), &*scoped));
        match unscoped.Payload() {
            CSSValuePayload::kScopedKeywordClass(payload) => {
                assert_eq!(*payload.scope_arguments.borrow(), [Some(123), None])
            }
            _ => unreachable!(),
        }
        assert!(!unscoped.IsScopedValue());
        assert!(scoped.IsScopedValue());
    }
    #[test]
    fn create_routes_length_kind_and_zoom_to_the_original_factory_targets() {
        let auto = Length::new(0.0, LengthType::kAuto);
        let fixed = Length::Fixed(17.0);
        let percent = Length::Percent(33.0);
        let ident = CSSValue::<Dispatch>::Create(&auto, 2.0);
        match ident.Payload() {
            CSSValuePayload::kIdentifierClass(payload) => {
                assert_eq!(payload.factory_length_type, Some(LengthType::kAuto))
            }
            _ => panic!("auto length must use the identifier factory"),
        }
        for length in [&fixed, &percent] {
            let primitive = CSSValue::<Dispatch>::Create(length, 2.5);
            match primitive.Payload() {
                CSSValuePayload::kNumericLiteralClass(payload) => {
                    assert_eq!(payload.factory_length_type, Some(length.GetType()));
                    assert_eq!(payload.number, 2.5);
                }
                _ => panic!("fixed/percent length must use the primitive factory"),
            }
        }
    }
}
