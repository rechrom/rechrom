/*
 * Copyright (C) 2012 Adobe Systems Incorporated. All rights reserved.
 * Copyright (C) 2012 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 *
 * 1. Redistributions of source code must retain the above
 *    copyright notice, this list of conditions and the following
 *    disclaimer.
 * 2. Redistributions in binary form must reproduce the above
 *    copyright notice, this list of conditions and the following
 *    disclaimer in the documentation and/or other materials
 *    provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDER "AS IS" AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY,
 * OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR
 * TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF
 * THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 */
// cpp: third_party/blink/renderer/core/css/parser/css_parser_mode.h:38-90

#![allow(non_camel_case_types)]

// Must not grow beyond 4 bits, due to packing in CSSPropertyValueSet.
// cpp: third_party/blink/renderer/core/css/parser/css_parser_mode.h:38-72
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSParserMode {
    kHTMLStandardMode,
    kHTMLQuirksMode,
    // SVG attributes are parsed in quirks mode but rules differ slightly.
    kSVGAttributeMode,
    // @font-face rules are specially tagged in CSSPropertyValueSet so
    // CSSOM modifications don't treat them as style rules.
    kCSSFontFaceRuleMode,
    // @keyframes rules are specially tagged in CSSPropertyValueSet so CSSOM
    // modifications don't allow setting animation-* in their keyframes.
    kCSSKeyframeRuleMode,
    // @property rules are specially tagged so modifications through the
    // inspector don't treat them as style rules.
    kCSSPropertyRuleMode,
    // @font-palette-values rules are specially tagged so modifications through
    // the inspector don't treat them as style rules.
    kCSSFontPaletteValuesRuleMode,
    // @position-try rules have limitations on what they allow, also through
    // mutations in CSSOM.
    // https://drafts.csswg.org/css-anchor-position-1/#om-position-try
    kCSSPositionTryRuleMode,
    // Within @function rules, only the 'result' descriptor and local variables
    // (i.e. "custom properties") are allowed.
    // https://drafts.csswg.org/css-mixins-1/#the-function-declarations-interface
    kCSSFunctionDescriptorsMode,
    // @counter-style rules are specially tagged so modifications through
    // the inspector don't treat them as style rules.
    kCSSCounterStyleRuleMode,
    // User agent stylesheets are parsed in standards mode but also allows
    // internal properties and values.
    kUASheetMode,
    // This should always be the last entry.
    kNumCSSParserModes,
}

// cpp: third_party/blink/renderer/core/css/parser/css_parser_mode.h:74-76
#[inline]
pub fn IsQuirksModeBehavior(mode: CSSParserMode) -> bool {
    mode == CSSParserMode::kHTMLQuirksMode
}

// cpp: third_party/blink/renderer/core/css/parser/css_parser_mode.h:78-80
#[inline]
pub fn IsUASheetBehavior(mode: CSSParserMode) -> bool {
    mode == CSSParserMode::kUASheetMode
}

// cpp: third_party/blink/renderer/core/css/parser/css_parser_mode.h:82-85
#[inline]
pub fn IsUseCounterEnabledForMode(mode: CSSParserMode) -> bool {
    // We don't count the UA style sheet in our statistics.
    mode != CSSParserMode::kUASheetMode
}

// Used in CSSParser APIs to say if we should defer parsing of declaration lists
// in style rules until we need them for CSSOM access, or for applying matched
// rules to computed style.
// cpp: third_party/blink/renderer/core/css/parser/css_parser_mode.h:87-90
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSDeferPropertyParsing {
    kNo,
    kYes,
}
