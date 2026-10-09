// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/core/css/parser/css_parser.h:49-183
// cpp: third_party/blink/renderer/core/css/parser/css_parser.cc:33-500
// Complete public facade and local-context dispatch. The required backend
// operations are the untranslated CSSParserImpl/selector/property/fast-path/
// supports/variable parsers and CSSRule/CSSValue/property-store/platform types.
// There is no fallback parser, resolver, browser behavior, or default backend.

#![allow(non_snake_case, non_camel_case_types)]

use super::allowed_rules::AllowedRules;
use super::css_at_rule_id::CSSAtRuleID;
use super::css_nesting_type::CSSNestingType;
use super::css_parser_context::{
    CSSParserContext, CSSParserContextPlatform, DocumentHandle, DocumentSnapshot,
    SecureContextMode, StrictCSSParserContext,
};
use super::css_parser_local_context::CSSParserLocalContext;
use super::css_parser_mode::{CSSDeferPropertyParsing, CSSParserMode};
use super::css_parser_token_stream::{CSSParserTokenStream, TokenStreamTokenizer};
use super::css_property_parser::CssValueKeywordID;
use crate::css_property_names::ResolveCSSPropertyID;
use crate::css_property_value_set::SetResult;
use crate::properties::css_property::CSSProperty;
use foundation::{AtomicString, CSSPropertyID, CSSValueID, String, StringView};
use std::marker::PhantomData;
use std::rc::Rc;

type Context<B> = CSSParserContext<B>;
type Stream<B> = CSSParserTokenStream<'static, <B as CSSParserBackend>::Tokenizer>;

// cpp: css_parser_fast_paths.h:22-31; css_supports_parser.h Result
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseColorResult {
    kFailure,
    kKeyword,
    kColor,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupportsResult {
    kUnsupported,
    kSupported,
    kParseFailure,
}

// Each operation corresponds to an actual dependency call in css_parser.cc.
// All operations are required. Rc represents a source GC value/rule reference;
// selector slices retain the source arena lifetime. No concrete CSS AST/value
// is manufactured by this facade.
pub trait CSSParserBackend: CSSParserContextPlatform + Sized {
    type Tokenizer: TokenStreamTokenizer;
    type StyleRule;
    type StyleRuleBase;
    type StyleRuleKeyframe;
    type StyleSheetContents;
    type CSSParserObserver;
    type CSSSelector;
    type CSSSelectorList;
    type MutablePropertySet;
    type ImmutablePropertySet;
    type CSSValue;
    type CSSPrimitiveValue;
    type CSSPropertyValue;
    type Element;
    type KeyframeOffset;
    type ParseSheetResult;
    type Color;
    type ColorScheme: Copy;
    type ColorProvider;
    type ValueRange: Copy;
    type ParserImpl;

    fn IsAllocationAllowed() -> bool;
    fn TopLevelRules() -> AllowedRules;
    fn NestedGroupRules() -> AllowedRules;
    fn PageMarginRules() -> AllowedRules;
    fn KeyframeRules() -> AllowedRules;
    fn StyleSheetParserContext(sheet: &Self::StyleSheetContents) -> Rc<Context<Self>>;
    fn LocalDOMWindowDocument(context: &Self::ExecutionContext) -> Option<DocumentSnapshot<Self>>;
    fn ExecutionSecureContextMode(context: &Self::ExecutionContext) -> SecureContextMode;
    fn MutableSetParserMode(set: &Self::MutablePropertySet) -> CSSParserMode;
    fn NewMutablePropertySet(mode: CSSParserMode) -> Self::MutablePropertySet;
    fn IsPropertySetEmpty(set: &Self::MutablePropertySet) -> bool;
    fn GetPropertyCSSValue(
        set: &Self::MutablePropertySet,
        property: CSSPropertyID,
    ) -> Option<Rc<Self::CSSValue>>;
    fn SetLonghandProperty(
        set: &mut Self::MutablePropertySet,
        resolved: CSSPropertyID,
        value: Rc<Self::CSSValue>,
        important: bool,
    ) -> SetResult;
    fn MakePropertyValue(
        resolved: CSSPropertyID,
        value: Rc<Self::CSSValue>,
        important: bool,
    ) -> Self::CSSPropertyValue;
    fn IsCSSWideKeyword(value: &Self::CSSValue) -> bool;
    fn IsPendingSubstitutionValue(value: &Self::CSSValue) -> bool;
    fn IsValidVariableName(name: &AtomicString) -> bool;

    fn ParseDeclarationListImpl(
        set: &mut Self::MutablePropertySet,
        text: &String,
        context: &Context<Self>,
    ) -> bool;
    fn ParseNestedDeclarationsRuleImpl(
        context: &Context<Self>,
        nesting: CSSNestingType,
        parent: Option<&Self::StyleRule>,
        text: StringView,
    ) -> Option<Rc<Self::StyleRuleBase>>;
    fn ParseDeclarationListForInspectorImpl(
        text: &String,
        context: &Context<Self>,
        observer: &mut Self::CSSParserObserver,
    );
    fn ParseSelectorImpl<'a>(
        stream: &mut Stream<Self>,
        context: &Context<Self>,
        nesting: CSSNestingType,
        parent: Option<&Self::StyleRule>,
        semicolon_aborts_nested_selector: bool,
        sheet: Option<&mut Self::StyleSheetContents>,
        arena: &'a mut Vec<Self::CSSSelector>,
    ) -> &'a mut [Self::CSSSelector];
    fn ParsePageSelectorImpl(
        stream: &mut Stream<Self>,
        sheet: Option<&mut Self::StyleSheetContents>,
        context: &Context<Self>,
    ) -> Option<Rc<Self::CSSSelectorList>>;
    fn ParseRuleImpl(
        text: &String,
        context: &Context<Self>,
        nesting: CSSNestingType,
        parent: Option<&Self::StyleRule>,
        sheet: Option<&mut Self::StyleSheetContents>,
        rules: AllowedRules,
    ) -> Option<Rc<Self::StyleRuleBase>>;
    fn ParseStyleSheetImpl(
        text: &String,
        context: &Context<Self>,
        sheet: &mut Self::StyleSheetContents,
        defer: CSSDeferPropertyParsing,
        allow_import_rules: bool,
    ) -> Self::ParseSheetResult;
    fn ParseStyleSheetForInspectorImpl(
        text: &String,
        context: &Context<Self>,
        sheet: &mut Self::StyleSheetContents,
        observer: &mut Self::CSSParserObserver,
    );
    fn MaybeParseValueFast(
        property: CSSPropertyID,
        text: StringView,
        context: &Context<Self>,
    ) -> Option<Rc<Self::CSSValue>>;
    fn ParseSingleValueImpl(
        property: CSSPropertyID,
        stream: &mut Stream<Self>,
        context: &Context<Self>,
    ) -> Option<Rc<Self::CSSValue>>;
    fn ParseValueImpl(
        set: &mut Self::MutablePropertySet,
        property: CSSPropertyID,
        text: StringView,
        important: bool,
        context: &Context<Self>,
    ) -> SetResult;
    fn ParseValueVectorImpl(
        result: &mut Vec<Self::CSSPropertyValue>,
        resolved: CSSPropertyID,
        text: StringView,
        context: &Context<Self>,
    ) -> u32;
    fn ParseVariableValueImpl(
        set: &mut Self::MutablePropertySet,
        name: &AtomicString,
        text: StringView,
        important: bool,
        context: &Context<Self>,
        animation_tainted: bool,
    ) -> SetResult;
    fn ParseInlineStyleDeclarationForElementImpl(
        text: &String,
        element: Option<&Self::Element>,
    ) -> Rc<Self::ImmutablePropertySet>;
    fn ParseInlineStyleDeclarationImpl(
        text: &String,
        mode: CSSParserMode,
        secure: SecureContextMode,
        document: Option<&DocumentHandle<Self>>,
    ) -> Rc<Self::ImmutablePropertySet>;
    fn ParseKeyframeKeyListImpl(
        context: &Context<Self>,
        text: &String,
    ) -> Option<Vec<Self::KeyframeOffset>>;
    fn ToKeyframeRule(rule: Option<Rc<Self::StyleRuleBase>>)
        -> Option<Rc<Self::StyleRuleKeyframe>>;
    fn ParseCustomPropertyNameImpl(text: &String) -> String;
    fn NewParserImpl(context: &Context<Self>) -> Self::ParserImpl;
    fn ConsumeSupportsCondition(
        stream: &mut Stream<Self>,
        parser: &mut Self::ParserImpl,
    ) -> SupportsResult;
    fn NamedColor(text: &String) -> Option<Self::Color>;
    // This dependency may change color only when returning kColor.
    fn ParseColorFast(
        text: &String,
        mode: CSSParserMode,
        color: &mut Self::Color,
    ) -> ParseColorResult;
    fn CSSColorValue(value: &Self::CSSValue) -> Option<Self::Color>;
    fn IsSystemColorIncludingDeprecated(id: CSSValueID) -> bool;
    fn SystemColor(
        id: CSSValueID,
        scheme: Self::ColorScheme,
        provider: Option<&Self::ColorProvider>,
        expose_accent: bool,
    ) -> Self::Color;
    fn ConsumeLengthOrPercent(
        stream: &mut Stream<Self>,
        context: &Context<Self>,
        local: &mut CSSParserLocalContext,
        range: Self::ValueRange,
    ) -> Option<Rc<Self::CSSPrimitiveValue>>;
}

// cpp: css_parser.cc LocalCSSParserContext
// An owning restoration guard avoids a self-reference to the Rc context while
// retaining the source scope's restore-after-parse behavior, including unwind.
struct LocalCSSParserContext<B: CSSParserBackend> {
    context: Rc<Context<B>>,
    previous_mode: Option<CSSParserMode>,
}
impl<B: CSSParserBackend> LocalCSSParserContext<B> {
    fn new(
        secure: SecureContextMode,
        sheet: Option<&B::StyleSheetContents>,
        execution: Option<&B::ExecutionContext>,
        mode: CSSParserMode,
    ) -> Self {
        if let Some(sheet) = sheet {
            let context = B::StyleSheetParserContext(sheet);
            let previous_mode = if context.GetMode() != mode {
                let old = context.GetMode();
                context.SetMode(mode);
                Some(old)
            } else {
                None
            };
            Self {
                context,
                previous_mode,
            }
        } else if let Some(document) = execution.and_then(B::LocalDOMWindowDocument) {
            let context = Rc::new(Context::<B>::FromDocument(&document));
            context.SetMode(mode);
            Self {
                context,
                previous_mode: None,
            }
        } else {
            Self {
                context: Rc::new(Context::<B>::FromMode(mode, secure, None)),
                previous_mode: None,
            }
        }
    }
}
impl<B: CSSParserBackend> Drop for LocalCSSParserContext<B> {
    fn drop(&mut self) {
        if let Some(mode) = self.previous_mode {
            self.context.SetMode(mode);
        }
    }
}

// cpp: css_parser.h CSSParser; static-only class becomes associated functions.
// References encode source non-null prerequisites. Nullable nesting parents,
// execution contexts, sheets, providers and the guarded length-parser context
// remain explicit Option arguments. Source default arguments are not silently
// inferred: callers supply defer=kNo, allow_import_rules=true, strict=true, etc.
pub struct CSSParser<B: CSSParserBackend>(PhantomData<B>);
impl<B: CSSParserBackend> CSSParser<B> {
    // cpp: css_parser.cc ParseDeclarationList
    pub fn ParseDeclarationList(
        context: &Context<B>,
        set: &mut B::MutablePropertySet,
        text: &String,
    ) -> bool {
        B::ParseDeclarationListImpl(set, text, context)
    }
    // cpp: css_parser.cc ParseNestedDeclarationsRule
    pub fn ParseNestedDeclarationsRule(
        context: &Context<B>,
        nesting: CSSNestingType,
        parent: Option<&B::StyleRule>,
        text: StringView,
    ) -> Option<Rc<B::StyleRuleBase>> {
        B::ParseNestedDeclarationsRuleImpl(context, nesting, parent, text)
    }
    // cpp: css_parser.cc ParseDeclarationListForInspector
    pub fn ParseDeclarationListForInspector(
        context: &Context<B>,
        text: &String,
        observer: &mut B::CSSParserObserver,
    ) {
        B::ParseDeclarationListForInspectorImpl(text, context, observer);
    }
    // cpp: css_parser.cc ParseSelector
    pub fn ParseSelector<'a>(
        context: &Context<B>,
        nesting: CSSNestingType,
        parent: Option<&B::StyleRule>,
        sheet: Option<&mut B::StyleSheetContents>,
        text: &String,
        arena: &'a mut Vec<B::CSSSelector>,
    ) -> &'a mut [B::CSSSelector] {
        let mut stream = Stream::<B>::new(StringView::from(text), 0);
        B::ParseSelectorImpl(&mut stream, context, nesting, parent, false, sheet, arena)
    }
    // cpp: css_parser.cc ParsePageSelector
    pub fn ParsePageSelector(
        context: &Context<B>,
        sheet: Option<&mut B::StyleSheetContents>,
        text: &String,
    ) -> Option<Rc<B::CSSSelectorList>> {
        let mut stream = Stream::<B>::new(StringView::from(text), 0);
        let list = B::ParsePageSelectorImpl(&mut stream, sheet, context);
        if !stream.AtEnd() {
            return None;
        }
        list
    }
    // cpp: css_parser.cc ParseMarginRule
    pub fn ParseMarginRule(
        context: &Context<B>,
        sheet: Option<&mut B::StyleSheetContents>,
        text: &String,
    ) -> Option<Rc<B::StyleRuleBase>> {
        B::ParseRuleImpl(
            text,
            context,
            CSSNestingType::kNone,
            None,
            sheet,
            B::PageMarginRules(),
        )
    }
    // cpp: css_parser.cc ParseRule
    pub fn ParseRule(
        context: &Context<B>,
        sheet: Option<&mut B::StyleSheetContents>,
        nesting: CSSNestingType,
        parent: Option<&B::StyleRule>,
        text: &String,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let mut rules = B::TopLevelRules();
        rules.Remove(CSSAtRuleID::kCSSAtRuleCharset);
        if parent.is_some() {
            rules = rules | B::NestedGroupRules();
        }
        B::ParseRuleImpl(text, context, nesting, parent, sheet, rules)
    }
    // cpp: css_parser.cc ParseSheet
    pub fn ParseSheet(
        context: &Context<B>,
        sheet: &mut B::StyleSheetContents,
        text: &String,
        defer: CSSDeferPropertyParsing,
        allow_import_rules: bool,
    ) -> B::ParseSheetResult {
        B::ParseStyleSheetImpl(text, context, sheet, defer, allow_import_rules)
    }
    // cpp: css_parser.cc ParseSheetForInspector
    pub fn ParseSheetForInspector(
        context: &Context<B>,
        sheet: &mut B::StyleSheetContents,
        text: &String,
        observer: &mut B::CSSParserObserver,
    ) {
        B::ParseStyleSheetForInspectorImpl(text, context, sheet, observer);
    }
    // cpp: css_parser.cc ParseValue execution-context overload
    pub fn ParseValue(
        set: &mut B::MutablePropertySet,
        unresolved: CSSPropertyID,
        text: StringView,
        important: bool,
        execution: Option<&B::ExecutionContext>,
    ) -> SetResult {
        let secure = execution.map_or(
            SecureContextMode::kInsecureContext,
            B::ExecutionSecureContextMode,
        );
        Self::ParseValueWithSheet(set, unresolved, text, important, secure, None, execution)
    }
    // cpp: css_parser.cc ParseValue secure-context/sheet overload
    pub fn ParseValueWithSheet(
        set: &mut B::MutablePropertySet,
        unresolved: CSSPropertyID,
        text: StringView,
        important: bool,
        secure: SecureContextMode,
        sheet: Option<&B::StyleSheetContents>,
        execution: Option<&B::ExecutionContext>,
    ) -> SetResult {
        debug_assert!(B::IsAllocationAllowed());
        if text.IsEmpty() {
            return SetResult::kParseError;
        }
        let mode = B::MutableSetParserMode(set);
        let local = LocalCSSParserContext::<B>::new(secure, sheet, execution, mode);
        let context = &local.context;
        let resolved = ResolveCSSPropertyID(unresolved);
        if let Some(value) = B::MaybeParseValueFast(resolved, text.clone(), context) {
            context.CountProperty(unresolved);
            return B::SetLonghandProperty(set, resolved, value, important);
        }
        let property = CSSProperty::Get(resolved);
        if mode == CSSParserMode::kHTMLStandardMode
            && property.IsProperty()
            && !property.IsShorthand()
        {
            let mut stream = Stream::<B>::new(text.clone(), 0);
            if let Some(value) = B::ParseSingleValueImpl(unresolved, &mut stream, context) {
                context.CountProperty(unresolved);
                return B::SetLonghandProperty(set, resolved, value, important);
            }
        }
        Self::ParseValueWithContext(set, unresolved, text, important, context)
    }
    // cpp: css_parser.cc ParseForPresentationStyle
    pub fn ParseForPresentationStyle(
        result: &mut Vec<B::CSSPropertyValue>,
        resolved: CSSPropertyID,
        text: StringView,
        mode: CSSParserMode,
        sheet: Option<&B::StyleSheetContents>,
        execution: Option<&B::ExecutionContext>,
    ) -> u32 {
        debug_assert!(B::IsAllocationAllowed());
        if text.IsEmpty() {
            return 0;
        }
        let secure = execution.map_or(
            SecureContextMode::kInsecureContext,
            B::ExecutionSecureContextMode,
        );
        let local = LocalCSSParserContext::<B>::new(secure, sheet, execution, mode);
        let context = &local.context;
        if let Some(value) = B::MaybeParseValueFast(resolved, text.clone(), context) {
            result.push(B::MakePropertyValue(resolved, value, false));
            return 1;
        }
        let property = CSSProperty::Get(resolved);
        if mode == CSSParserMode::kHTMLStandardMode
            && property.IsProperty()
            && !property.IsShorthand()
        {
            let mut stream = Stream::<B>::new(text.clone(), 0);
            if let Some(value) = B::ParseSingleValueImpl(resolved, &mut stream, context) {
                result.push(B::MakePropertyValue(resolved, value, false));
                return 1;
            } else {
                return 0;
            }
        }
        B::ParseValueVectorImpl(result, resolved, text, context)
    }
    // cpp: css_parser.cc ParseValueForCustomProperty
    pub fn ParseValueForCustomProperty(
        set: &mut B::MutablePropertySet,
        name: &AtomicString,
        text: StringView,
        important: bool,
        secure: SecureContextMode,
        sheet: Option<&B::StyleSheetContents>,
        animation_tainted: bool,
    ) -> SetResult {
        debug_assert!(B::IsAllocationAllowed());
        debug_assert!(B::IsValidVariableName(name));
        if text.IsEmpty() {
            return SetResult::kParseError;
        }
        let mode = B::MutableSetParserMode(set);
        let context = if let Some(sheet) = sheet {
            let original = B::StyleSheetParserContext(sheet);
            let copy = Context::<B>::CopyWithDocument(&original, None);
            copy.SetMode(mode);
            copy
        } else {
            Context::<B>::FromMode(mode, secure, None)
        };
        B::ParseVariableValueImpl(set, name, text, important, &context, animation_tainted)
    }
    // cpp: css_parser.cc private ParseValue context overload
    fn ParseValueWithContext(
        set: &mut B::MutablePropertySet,
        unresolved: CSSPropertyID,
        text: StringView,
        important: bool,
        context: &Context<B>,
    ) -> SetResult {
        debug_assert!(B::IsAllocationAllowed());
        B::ParseValueImpl(set, unresolved, text, important, context)
    }
    // cpp: css_parser.cc ParseSingleValue (does not resolve aliases here)
    pub fn ParseSingleValue(
        unresolved: CSSPropertyID,
        text: &String,
        context: &Context<B>,
    ) -> Option<Rc<B::CSSValue>> {
        debug_assert!(B::IsAllocationAllowed());
        if text.empty() {
            return None;
        }
        if let Some(value) = B::MaybeParseValueFast(unresolved, StringView::from(text), context) {
            return Some(value);
        }
        let mut stream = Stream::<B>::new(StringView::from(text), 0);
        B::ParseSingleValueImpl(unresolved, &mut stream, context)
    }
    // cpp: css_parser.cc ParseInlineStyleDeclaration overloads
    pub fn ParseInlineStyleDeclarationForElement(
        text: &String,
        element: Option<&B::Element>,
    ) -> Rc<B::ImmutablePropertySet> {
        B::ParseInlineStyleDeclarationForElementImpl(text, element)
    }
    pub fn ParseInlineStyleDeclaration(
        text: &String,
        mode: CSSParserMode,
        secure: SecureContextMode,
        document: Option<&DocumentHandle<B>>,
    ) -> Rc<B::ImmutablePropertySet> {
        B::ParseInlineStyleDeclarationImpl(text, mode, secure, document)
    }
    // cpp: css_parser.cc ParseKeyframeKeyList/ParseKeyframeRule
    pub fn ParseKeyframeKeyList(
        context: &Context<B>,
        text: &String,
    ) -> Option<Vec<B::KeyframeOffset>> {
        B::ParseKeyframeKeyListImpl(context, text)
    }
    pub fn ParseKeyframeRule(
        context: &Context<B>,
        text: &String,
    ) -> Option<Rc<B::StyleRuleKeyframe>> {
        let rule = B::ParseRuleImpl(
            text,
            context,
            CSSNestingType::kNone,
            None,
            None,
            B::KeyframeRules(),
        );
        B::ToKeyframeRule(rule)
    }
    // cpp: css_parser.cc ParseCustomPropertyName
    pub fn ParseCustomPropertyName(text: &String) -> String {
        B::ParseCustomPropertyNameImpl(text)
    }
    // cpp: css_parser.cc ParseSupportsCondition
    pub fn ParseSupportsCondition(condition: &String, execution: &B::ExecutionContext) -> bool {
        let wrapped = WrapSupportsCondition(condition);
        let mut stream = Stream::<B>::new(StringView::from(&wrapped), 0);
        let document = B::LocalDOMWindowDocument(execution)
            .expect("CSS.supports requires a LocalDOMWindow document");
        let context = Context::<B>::FromDocument(&document);
        context.SetMode(CSSParserMode::kHTMLStandardMode);
        let mut parser = B::NewParserImpl(&context);
        let mut result = B::ConsumeSupportsCondition(&mut stream, &mut parser);
        if !stream.AtEnd() {
            result = SupportsResult::kParseFailure;
        }
        result == SupportsResult::kSupported
    }
    // cpp: css_parser.cc ParseColor
    pub fn ParseColor(color: &mut B::Color, text: &String, strict: bool) -> bool {
        debug_assert!(B::IsAllocationAllowed());
        if text.empty() {
            return false;
        }
        if let Some(named) = B::NamedColor(text) {
            *color = named;
            return true;
        }
        match B::ParseColorFast(
            text,
            if strict {
                CSSParserMode::kHTMLStandardMode
            } else {
                CSSParserMode::kHTMLQuirksMode
            },
            color,
        ) {
            ParseColorResult::kFailure => {}
            ParseColorResult::kKeyword => return false,
            ParseColorResult::kColor => return true,
        }
        let context = StrictCSSParserContext::<B>(SecureContextMode::kInsecureContext);
        let value = Self::ParseSingleValue(CSSPropertyID::kColor, text, &context);
        let Some(parsed) = value.as_deref().and_then(B::CSSColorValue) else {
            return false;
        };
        *color = parsed;
        true
    }
    // cpp: css_parser.cc ParseSystemColor
    pub fn ParseSystemColor(
        color: &mut B::Color,
        text: &String,
        scheme: B::ColorScheme,
        provider: Option<&B::ColorProvider>,
        expose_accent: bool,
    ) -> bool {
        let id = CssValueKeywordID(&StringView::from(text));
        if !B::IsSystemColorIncludingDeprecated(id) {
            return false;
        }
        *color = B::SystemColor(id, scheme, provider, expose_accent);
        true
    }
    // cpp: css_parser.cc ParseFontFaceDescriptor
    pub fn ParseFontFaceDescriptor(
        property: CSSPropertyID,
        text: &String,
        context: &Context<B>,
    ) -> Option<Rc<B::CSSValue>> {
        let mut set = B::NewMutablePropertySet(CSSParserMode::kCSSFontFaceRuleMode);
        Self::ParseValueWithContext(&mut set, property, StringView::from(text), true, context);
        B::GetPropertyCSSValue(&set, property)
    }
    // cpp: css_parser.cc ParseLengthPercentage
    pub fn ParseLengthPercentage(
        text: &String,
        context: Option<&Context<B>>,
        local: &mut CSSParserLocalContext,
        range: B::ValueRange,
    ) -> Option<Rc<B::CSSPrimitiveValue>> {
        if text.empty() {
            return None;
        }
        let context = context?;
        let mut stream = Stream::<B>::new(StringView::from(text), 0);
        stream.ConsumeWhitespace();
        let value = B::ConsumeLengthOrPercent(&mut stream, context, local, range);
        if stream.AtEnd() {
            value
        } else {
            None
        }
    }
    // cpp: css_parser.cc ParseFont
    pub fn ParseFont(
        text: &String,
        execution: Option<&B::ExecutionContext>,
    ) -> Option<B::MutablePropertySet> {
        debug_assert!(B::IsAllocationAllowed());
        let mut set = B::NewMutablePropertySet(CSSParserMode::kHTMLStandardMode);
        Self::ParseValue(
            &mut set,
            CSSPropertyID::kFont,
            StringView::from(text),
            true,
            execution,
        );
        if B::IsPropertySetEmpty(&set) {
            return None;
        }
        let font_size = B::GetPropertyCSSValue(&set, CSSPropertyID::kFontSize)?;
        if B::IsCSSWideKeyword(&font_size) {
            return None;
        }
        if B::IsPendingSubstitutionValue(&font_size) {
            return None;
        }
        Some(set)
    }
}

// cpp: css_parser.cc StrCat({"(", condition, ")"})
fn WrapSupportsCondition(condition: &String) -> String {
    if let Some(bytes) = condition.Span8() {
        let mut wrapped = Vec::with_capacity(bytes.len() + 2);
        wrapped.push(b'(');
        wrapped.extend_from_slice(bytes);
        wrapped.push(b')');
        String::from_latin1(&wrapped)
    } else {
        let units = condition.Span16().unwrap_or_default();
        let mut wrapped = Vec::with_capacity(units.len() + 2);
        wrapped.push(b'(' as u16);
        wrapped.extend_from_slice(units);
        wrapped.push(b')' as u16);
        String::from_utf16(&wrapped)
    }
}

#[cfg(test)]
mod tests {
    use super::super::css_parser_context::{
        CSSParserContextDocument, DocumentExecutionContextSnapshot,
    };
    use super::super::css_parser_token::{BlockType, CSSParserToken, CSSParserTokenType};
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    struct Backend;
    #[derive(Clone, Debug)]
    struct Value {
        kind: u8,
    }
    struct Set {
        mode: CSSParserMode,
        values: Vec<(CSSPropertyID, Rc<Value>, bool)>,
    }
    struct Execution {
        window: bool,
        secure: SecureContextMode,
    }
    struct Document;
    struct Plan {
        fast: Option<Value>,
        single: Option<Value>,
        full: Option<Value>,
        supports: SupportsResult,
        consume_supports: bool,
        named: Option<u32>,
        color_fast: ParseColorResult,
    }
    impl Plan {
        fn empty() -> Self {
            Self {
                fast: None,
                single: None,
                full: None,
                supports: SupportsResult::kUnsupported,
                consume_supports: false,
                named: None,
                color_fast: ParseColorResult::kFailure,
            }
        }
    }
    thread_local! {
        static PLAN: RefCell<Plan> = RefCell::new(Plan::empty());
        static EVENTS: RefCell<Vec<std::string::String>> = const { RefCell::new(Vec::new()) };
        static TOKENS: RefCell<Vec<CSSParserToken>> = const { RefCell::new(Vec::new()) };
        static DOCUMENT: Rc<Document> = Rc::new(Document);
    }
    fn event(text: impl Into<std::string::String>) {
        EVENTS.with(|events| events.borrow_mut().push(text.into()));
    }
    fn reset() {
        PLAN.with(|p| *p.borrow_mut() = Plan::empty());
        EVENTS.with(|e| e.borrow_mut().clear());
        TOKENS.with(|t| t.borrow_mut().clear());
    }
    fn events() -> Vec<std::string::String> {
        EVENTS.with(|e| e.borrow().clone())
    }
    fn context(mode: CSSParserMode) -> Rc<Context<Backend>> {
        Rc::new(Context::<Backend>::FromMode(
            mode,
            SecureContextMode::kInsecureContext,
            None,
        ))
    }
    fn set(mode: CSSParserMode) -> Set {
        Set {
            mode,
            values: vec![],
        }
    }
    fn tokens(types: &[CSSParserTokenType]) {
        TOKENS.with(|t| {
            *t.borrow_mut() = types
                .iter()
                .map(|&kind| CSSParserToken::new(kind, BlockType::kNotBlock))
                .collect()
        });
    }

    impl CSSParserContextPlatform for Backend {
        type URL = u8;
        type TextEncoding = bool;
        type Referrer = (String, u8);
        type ReferrerPolicy = u8;
        type DOMWrapperWorld = ();
        type ExecutionContext = Execution;
        type WebFeature = u8;
        type WebDXFeature = u8;
        fn NullURL() -> u8 {
            0
        }
        fn EmptyURL() -> u8 {
            1
        }
        fn EmptyTextEncoding() -> bool {
            false
        }
        fn IsEncodingValid(value: &bool) -> bool {
            *value
        }
        fn EmptyReferrer() -> Self::Referrer {
            (String::default(), 0)
        }
        fn MakeReferrer(value: String, policy: u8) -> Self::Referrer {
            (value, policy)
        }
        fn StrippedForUseAsReferrer(_: &u8) -> String {
            panic!("unexpected URL dependency")
        }
        fn ResolveURL(_: &u8, _: &String, _: Option<&bool>) -> u8 {
            panic!("unexpected URL dependency")
        }
        fn CSSParserIgnoreCharsetForURLsEnabled() -> bool {
            false
        }
        fn CountDeprecation(_: Option<&Rc<Execution>>, _: u8) {
            panic!("unexpected counter")
        }
    }
    impl CSSParserContextDocument<Backend> for Document {
        fn CountUse(&self, _: u8) {
            panic!("unexpected counter")
        }
        fn CountWebDXFeature(&self, _: u8) {
            panic!("unexpected counter")
        }
        fn CountProperty(&self, property: CSSPropertyID) {
            event(format!("count:{property:?}"));
        }
        fn GetExecutionContext(&self) -> Option<Rc<Execution>> {
            None
        }
        fn IsForMarkupSanitization(&self) -> bool {
            false
        }
    }
    // A token fixture scripts the dependency output; it does not tokenize CSS.
    struct Tokenizer {
        tokens: VecDeque<CSSParserToken>,
        offset: u32,
        previous: u32,
    }
    impl TokenStreamTokenizer for Tokenizer {
        fn new(text: StringView, offset: u32) -> Self {
            event(format!("stream:{}", text.ToString().Utf8()));
            Self {
                tokens: TOKENS.with(|t| t.borrow().clone().into()),
                offset,
                previous: offset,
            }
        }
        fn TokenizeSingle(&mut self) -> CSSParserToken {
            self.previous = self.offset;
            self.offset += 1;
            self.tokens.pop_front().unwrap_or_else(|| {
                CSSParserToken::new(CSSParserTokenType::kEOFToken, BlockType::kNotBlock)
            })
        }
        fn TokenizeSingleWithComments(&mut self) -> CSSParserToken {
            self.TokenizeSingle()
        }
        fn Offset(&self) -> u32 {
            self.offset
        }
        fn PreviousOffset(&self) -> u32 {
            self.previous
        }
        fn StringRangeAt(&self, _: u32, _: u32) -> StringView {
            panic!("unexpected range access")
        }
        fn StringRangeFrom(&self, _: u32) -> StringView {
            panic!("unexpected range access")
        }
        fn SkipToEndOfBlock(&mut self, _: u32) {
            panic!("unexpected block skip")
        }
        fn Restore(&mut self, _: &CSSParserToken, _: u32) -> CSSParserToken {
            panic!("unexpected restore")
        }
        fn TokenCount(&self) -> u32 {
            self.offset
        }
        fn UnicodeRangesAllowed(&self) -> bool {
            false
        }
        fn SetUnicodeRangesAllowed(&mut self, _: bool) {
            panic!("unexpected Unicode scope")
        }
        fn PopBlockStack(&mut self) {
            panic!("unexpected block stack")
        }
    }
    impl CSSParserBackend for Backend {
        type Tokenizer = Tokenizer;
        type StyleRule = u8;
        type StyleRuleBase = u8;
        type StyleRuleKeyframe = u8;
        type StyleSheetContents = Rc<Context<Backend>>;
        type CSSParserObserver = u8;
        type CSSSelector = u8;
        type CSSSelectorList = u8;
        type MutablePropertySet = Set;
        type ImmutablePropertySet = u8;
        type CSSValue = Value;
        type CSSPrimitiveValue = u8;
        type CSSPropertyValue = (CSSPropertyID, Rc<Value>, bool);
        type Element = u8;
        type KeyframeOffset = u8;
        type ParseSheetResult = u8;
        type Color = u32;
        type ColorScheme = u8;
        type ColorProvider = u8;
        type ValueRange = u8;
        type ParserImpl = CSSParserMode;
        fn IsAllocationAllowed() -> bool {
            true
        }
        fn TopLevelRules() -> AllowedRules {
            AllowedRules::FromAtRules(&[
                CSSAtRuleID::kCSSAtRuleCharset,
                CSSAtRuleID::kCSSAtRuleImport,
                CSSAtRuleID::kCSSAtRuleNamespace,
            ])
        }
        fn NestedGroupRules() -> AllowedRules {
            AllowedRules::FromAtRules(&[CSSAtRuleID::kCSSAtRuleApplyMixin])
        }
        fn PageMarginRules() -> AllowedRules {
            AllowedRules::FromAtRules(&[CSSAtRuleID::kCSSAtRuleTopLeft])
        }
        fn KeyframeRules() -> AllowedRules {
            AllowedRules::FromQualifiedRules(&[
                super::super::allowed_rules::QualifiedRuleType::kKeyframe,
            ])
        }
        fn StyleSheetParserContext(sheet: &Self::StyleSheetContents) -> Rc<Context<Self>> {
            sheet.clone()
        }
        fn LocalDOMWindowDocument(
            context: &Self::ExecutionContext,
        ) -> Option<DocumentSnapshot<Self>> {
            if !context.window {
                return None;
            }
            Some(DocumentSnapshot {
                document: DOCUMENT.with(|d| d.clone() as DocumentHandle<Self>),
                base_url: 2,
                in_quirks_mode: true,
                is_html_document: true,
                referrer_policy: 0,
                execution_context: Some(DocumentExecutionContextSnapshot {
                    outgoing_referrer: String::default(),
                    secure_context_mode: context.secure,
                    world: None,
                }),
            })
        }
        fn ExecutionSecureContextMode(context: &Self::ExecutionContext) -> SecureContextMode {
            context.secure
        }
        fn MutableSetParserMode(set: &Self::MutablePropertySet) -> CSSParserMode {
            set.mode
        }
        fn NewMutablePropertySet(mode: CSSParserMode) -> Self::MutablePropertySet {
            event(format!("new-set:{mode:?}"));
            Set {
                mode,
                values: vec![],
            }
        }
        fn IsPropertySetEmpty(set: &Self::MutablePropertySet) -> bool {
            set.values.is_empty()
        }
        fn GetPropertyCSSValue(
            set: &Self::MutablePropertySet,
            property: CSSPropertyID,
        ) -> Option<Rc<Self::CSSValue>> {
            set.values
                .iter()
                .find(|(id, _, _)| *id == property)
                .map(|(_, value, _)| value.clone())
        }
        fn SetLonghandProperty(
            set: &mut Self::MutablePropertySet,
            resolved: CSSPropertyID,
            value: Rc<Self::CSSValue>,
            important: bool,
        ) -> SetResult {
            event(format!("set:{resolved:?}:{important}"));
            set.values.push((resolved, value, important));
            SetResult::kChangedPropertySet
        }
        fn MakePropertyValue(
            resolved: CSSPropertyID,
            value: Rc<Self::CSSValue>,
            important: bool,
        ) -> Self::CSSPropertyValue {
            (resolved, value, important)
        }
        fn IsCSSWideKeyword(value: &Self::CSSValue) -> bool {
            value.kind == 1
        }
        fn IsPendingSubstitutionValue(value: &Self::CSSValue) -> bool {
            value.kind == 2
        }
        fn IsValidVariableName(name: &AtomicString) -> bool {
            name.Utf8().starts_with("--")
        }
        fn ParseDeclarationListImpl(
            set: &mut Self::MutablePropertySet,
            text: &String,
            context: &Context<Self>,
        ) -> bool {
            panic!("unexpected ParseDeclarationListImpl dependency")
        }
        fn ParseNestedDeclarationsRuleImpl(
            context: &Context<Self>,
            nesting: CSSNestingType,
            parent: Option<&Self::StyleRule>,
            text: StringView,
        ) -> Option<Rc<Self::StyleRuleBase>> {
            panic!("unexpected ParseNestedDeclarationsRuleImpl dependency")
        }
        fn ParseDeclarationListForInspectorImpl(
            text: &String,
            context: &Context<Self>,
            observer: &mut Self::CSSParserObserver,
        ) {
            panic!("unexpected ParseDeclarationListForInspectorImpl dependency")
        }
        fn ParseSelectorImpl<'a>(
            stream: &mut Stream<Self>,
            context: &Context<Self>,
            nesting: CSSNestingType,
            parent: Option<&Self::StyleRule>,
            semicolon_aborts_nested_selector: bool,
            sheet: Option<&mut Self::StyleSheetContents>,
            arena: &'a mut Vec<Self::CSSSelector>,
        ) -> &'a mut [Self::CSSSelector] {
            panic!("unexpected ParseSelectorImpl dependency")
        }
        fn ParsePageSelectorImpl(
            stream: &mut Stream<Self>,
            sheet: Option<&mut Self::StyleSheetContents>,
            context: &Context<Self>,
        ) -> Option<Rc<Self::CSSSelectorList>> {
            Some(Rc::new(1))
        }
        fn ParseRuleImpl(
            text: &String,
            context: &Context<Self>,
            nesting: CSSNestingType,
            parent: Option<&Self::StyleRule>,
            sheet: Option<&mut Self::StyleSheetContents>,
            rules: AllowedRules,
        ) -> Option<Rc<Self::StyleRuleBase>> {
            event(format!(
                "rule:{}:{}:{}:{}",
                rules.Has(CSSAtRuleID::kCSSAtRuleCharset),
                rules.Has(CSSAtRuleID::kCSSAtRuleImport),
                rules.Has(CSSAtRuleID::kCSSAtRuleApplyMixin),
                parent.is_some()
            ));
            Some(Rc::new(1))
        }
        fn ParseStyleSheetImpl(
            text: &String,
            context: &Context<Self>,
            sheet: &mut Self::StyleSheetContents,
            defer: CSSDeferPropertyParsing,
            allow_import_rules: bool,
        ) -> Self::ParseSheetResult {
            panic!("unexpected ParseStyleSheetImpl dependency")
        }
        fn ParseStyleSheetForInspectorImpl(
            text: &String,
            context: &Context<Self>,
            sheet: &mut Self::StyleSheetContents,
            observer: &mut Self::CSSParserObserver,
        ) {
            panic!("unexpected ParseStyleSheetForInspectorImpl dependency")
        }
        fn MaybeParseValueFast(
            property: CSSPropertyID,
            text: StringView,
            context: &Context<Self>,
        ) -> Option<Rc<Self::CSSValue>> {
            event(format!("fast:{property:?}:{:?}", context.Mode()));
            PLAN.with(|p| p.borrow().fast.clone().map(Rc::new))
        }
        fn ParseSingleValueImpl(
            property: CSSPropertyID,
            stream: &mut Stream<Self>,
            context: &Context<Self>,
        ) -> Option<Rc<Self::CSSValue>> {
            event(format!("single:{property:?}"));
            PLAN.with(|p| p.borrow().single.clone().map(Rc::new))
        }
        fn ParseValueImpl(
            set: &mut Self::MutablePropertySet,
            property: CSSPropertyID,
            text: StringView,
            important: bool,
            context: &Context<Self>,
        ) -> SetResult {
            event(format!(
                "full:{property:?}:{:?}:{important}",
                context.Mode()
            ));
            let value = PLAN.with(|p| p.borrow().full.clone());
            if let Some(value) = value {
                set.values.push((
                    if property == CSSPropertyID::kFont {
                        CSSPropertyID::kFontSize
                    } else {
                        property
                    },
                    Rc::new(value),
                    important,
                ));
                SetResult::kChangedPropertySet
            } else {
                SetResult::kParseError
            }
        }
        fn ParseValueVectorImpl(
            result: &mut Vec<Self::CSSPropertyValue>,
            resolved: CSSPropertyID,
            text: StringView,
            context: &Context<Self>,
        ) -> u32 {
            event("vector-full");
            0
        }
        fn ParseVariableValueImpl(
            set: &mut Self::MutablePropertySet,
            name: &AtomicString,
            text: StringView,
            important: bool,
            context: &Context<Self>,
            animation_tainted: bool,
        ) -> SetResult {
            event(format!(
                "variable:{}:{:?}:{important}:{animation_tainted}:{}",
                name.Utf8(),
                context.Mode(),
                context.GetDocument().is_some()
            ));
            SetResult::kUnchanged
        }
        fn ParseInlineStyleDeclarationForElementImpl(
            text: &String,
            element: Option<&Self::Element>,
        ) -> Rc<Self::ImmutablePropertySet> {
            panic!("unexpected ParseInlineStyleDeclarationForElementImpl dependency")
        }
        fn ParseInlineStyleDeclarationImpl(
            text: &String,
            mode: CSSParserMode,
            secure: SecureContextMode,
            document: Option<&DocumentHandle<Self>>,
        ) -> Rc<Self::ImmutablePropertySet> {
            panic!("unexpected ParseInlineStyleDeclarationImpl dependency")
        }
        fn ParseKeyframeKeyListImpl(
            context: &Context<Self>,
            text: &String,
        ) -> Option<Vec<Self::KeyframeOffset>> {
            panic!("unexpected ParseKeyframeKeyListImpl dependency")
        }
        fn ToKeyframeRule(
            rule: Option<Rc<Self::StyleRuleBase>>,
        ) -> Option<Rc<Self::StyleRuleKeyframe>> {
            rule
        }
        fn ParseCustomPropertyNameImpl(text: &String) -> String {
            panic!("unexpected ParseCustomPropertyNameImpl dependency")
        }
        fn NewParserImpl(context: &Context<Self>) -> Self::ParserImpl {
            event(format!("supports-mode:{:?}", context.Mode()));
            context.Mode()
        }
        fn ConsumeSupportsCondition(
            stream: &mut Stream<Self>,
            parser: &mut Self::ParserImpl,
        ) -> SupportsResult {
            let (consume, result) =
                PLAN.with(|p| (p.borrow().consume_supports, p.borrow().supports));
            if consume {
                while !stream.AtEnd() {
                    stream.Peek();
                    stream.Consume();
                }
            }
            result
        }
        fn NamedColor(text: &String) -> Option<Self::Color> {
            PLAN.with(|p| p.borrow().named)
        }
        fn ParseColorFast(
            text: &String,
            mode: CSSParserMode,
            color: &mut Self::Color,
        ) -> ParseColorResult {
            event(format!("color-fast:{mode:?}"));
            let result = PLAN.with(|p| p.borrow().color_fast);
            if result == ParseColorResult::kColor {
                *color = 20;
            }
            result
        }
        fn CSSColorValue(value: &Self::CSSValue) -> Option<Self::Color> {
            if value.kind == 3 {
                Some(30)
            } else {
                None
            }
        }
        fn IsSystemColorIncludingDeprecated(id: CSSValueID) -> bool {
            id == CSSValueID::kCanvas
        }
        fn SystemColor(
            id: CSSValueID,
            scheme: Self::ColorScheme,
            provider: Option<&Self::ColorProvider>,
            expose_accent: bool,
        ) -> Self::Color {
            event(format!(
                "system:{id:?}:{scheme}:{}:{expose_accent}",
                provider.is_some()
            ));
            40
        }
        fn ConsumeLengthOrPercent(
            stream: &mut Stream<Self>,
            context: &Context<Self>,
            local: &mut CSSParserLocalContext,
            range: Self::ValueRange,
        ) -> Option<Rc<Self::CSSPrimitiveValue>> {
            event(format!("length-range:{range}"));
            if stream.Peek().GetType() == CSSParserTokenType::kNumberToken {
                stream.Consume();
                stream.ConsumeWhitespace();
                Some(Rc::new(1))
            } else {
                None
            }
        }
    }

    #[test]
    fn value_paths_keep_alias_counting_and_restore_sheet_mode() {
        reset();
        let sheet = context(CSSParserMode::kHTMLQuirksMode);
        let mut values = set(CSSParserMode::kHTMLStandardMode);
        PLAN.with(|p| p.borrow_mut().single = Some(Value { kind: 0 }));
        assert_eq!(
            CSSParser::<Backend>::ParseValueWithSheet(
                &mut values,
                CSSPropertyID::kAliasWebkitAppearance,
                StringView::from("value"),
                true,
                SecureContextMode::kSecureContext,
                Some(&sheet),
                None
            ),
            SetResult::kChangedPropertySet
        );
        let log = events();
        assert!(log
            .iter()
            .any(|e| e == "fast:kAppearance:kHTMLStandardMode"));
        assert!(log.iter().any(|e| e == "single:kAliasWebkitAppearance"));
        assert!(log.iter().any(|e| e == "set:kAppearance:true"));
        assert!(!log.iter().any(|e| e.starts_with("full:")));
        assert_eq!(sheet.Mode(), CSSParserMode::kHTMLQuirksMode);
        reset();
        PLAN.with(|p| p.borrow_mut().fast = Some(Value { kind: 0 }));
        let execution = Execution {
            window: true,
            secure: SecureContextMode::kSecureContext,
        };
        CSSParser::<Backend>::ParseValue(
            &mut values,
            CSSPropertyID::kAliasWebkitAppearance,
            StringView::from("value"),
            false,
            Some(&execution),
        );
        assert!(events().iter().any(|e| e == "count:kAliasWebkitAppearance"));
        assert!(!events().iter().any(|e| e.starts_with("single:")));
        reset();
        assert_eq!(
            CSSParser::<Backend>::ParseValue(
                &mut values,
                CSSPropertyID::kColor,
                StringView::from(""),
                false,
                None
            ),
            SetResult::kParseError
        );
        assert!(events().is_empty());
    }
    #[test]
    fn failed_longhand_falls_back_only_for_regular_declarations() {
        reset();
        let mut values = set(CSSParserMode::kHTMLStandardMode);
        CSSParser::<Backend>::ParseValue(
            &mut values,
            CSSPropertyID::kColor,
            StringView::from("bad"),
            false,
            None,
        );
        assert!(events().iter().any(|e| e.starts_with("full:kColor:")));
        reset();
        let mut result = vec![];
        assert_eq!(
            CSSParser::<Backend>::ParseForPresentationStyle(
                &mut result,
                CSSPropertyID::kColor,
                StringView::from("bad"),
                CSSParserMode::kHTMLStandardMode,
                None,
                None
            ),
            0
        );
        assert!(!events().iter().any(|e| e == "vector-full"));
        reset();
        CSSParser::<Backend>::ParseForPresentationStyle(
            &mut result,
            CSSPropertyID::kMargin,
            StringView::from("bad"),
            CSSParserMode::kHTMLStandardMode,
            None,
            None,
        );
        assert!(events().iter().any(|e| e == "vector-full"));
        reset();
        PLAN.with(|p| p.borrow_mut().fast = Some(Value { kind: 0 }));
        assert_eq!(
            CSSParser::<Backend>::ParseForPresentationStyle(
                &mut result,
                CSSPropertyID::kColor,
                StringView::from("value"),
                CSSParserMode::kSVGAttributeMode,
                None,
                None
            ),
            1
        );
        assert_eq!(result[0].0, CSSPropertyID::kColor);
        assert!(!result[0].2);
    }
    #[test]
    fn rules_selectors_and_supports_keep_rejection_boundaries() {
        reset();
        let context = context(CSSParserMode::kHTMLQuirksMode);
        CSSParser::<Backend>::ParseRule(
            &context,
            None,
            CSSNestingType::kNone,
            None,
            &String::from("rule"),
        );
        assert!(events().iter().any(|e| e == "rule:false:true:false:false"));
        CSSParser::<Backend>::ParseRule(
            &context,
            None,
            CSSNestingType::kNesting,
            Some(&1),
            &String::from("rule"),
        );
        assert!(events().iter().any(|e| e == "rule:false:true:true:true"));
        tokens(&[CSSParserTokenType::kIdentToken]);
        assert!(
            CSSParser::<Backend>::ParsePageSelector(&context, None, &String::from("extra"))
                .is_none()
        );
        tokens(&[]);
        assert!(
            CSSParser::<Backend>::ParsePageSelector(&context, None, &String::from("valid"))
                .is_some()
        );
        let execution = Execution {
            window: true,
            secure: SecureContextMode::kSecureContext,
        };
        PLAN.with(|p| p.borrow_mut().supports = SupportsResult::kSupported);
        tokens(&[CSSParserTokenType::kIdentToken]);
        assert!(!CSSParser::<Backend>::ParseSupportsCondition(
            &String::from_latin1(b"condition"),
            &execution
        ));
        PLAN.with(|p| p.borrow_mut().consume_supports = true);
        assert!(CSSParser::<Backend>::ParseSupportsCondition(
            &String::from_latin1(b"condition"),
            &execution
        ));
        assert!(events().iter().any(|e| e == "stream:(condition)"));
        assert!(events()
            .iter()
            .any(|e| e == "supports-mode:kHTMLStandardMode"));
    }
    #[test]
    fn custom_values_copy_context_and_length_rejects_trailing_tokens() {
        reset();
        let sheet = context(CSSParserMode::kHTMLQuirksMode);
        let mut values = set(CSSParserMode::kSVGAttributeMode);
        assert_eq!(
            CSSParser::<Backend>::ParseValueForCustomProperty(
                &mut values,
                &AtomicString::from_str("--x"),
                StringView::from("v"),
                true,
                SecureContextMode::kSecureContext,
                Some(&sheet),
                true
            ),
            SetResult::kUnchanged
        );
        assert_eq!(sheet.Mode(), CSSParserMode::kHTMLQuirksMode);
        assert!(events()
            .iter()
            .any(|e| e == "variable:--x:kSVGAttributeMode:true:true:false"));
        let mut local = CSSParserLocalContext::CreateWithoutPropertyForTest();
        tokens(&[
            CSSParserTokenType::kWhitespaceToken,
            CSSParserTokenType::kNumberToken,
            CSSParserTokenType::kWhitespaceToken,
        ]);
        assert!(CSSParser::<Backend>::ParseLengthPercentage(
            &String::from("value"),
            Some(&sheet),
            &mut local,
            7
        )
        .is_some());
        tokens(&[
            CSSParserTokenType::kNumberToken,
            CSSParserTokenType::kIdentToken,
        ]);
        assert!(CSSParser::<Backend>::ParseLengthPercentage(
            &String::from("extra"),
            Some(&sheet),
            &mut local,
            7
        )
        .is_none());
        assert!(CSSParser::<Backend>::ParseLengthPercentage(
            &String::from("value"),
            None,
            &mut local,
            7
        )
        .is_none());
    }
    #[test]
    fn font_filters_css_wide_and_pending_substitution() {
        reset();
        assert!(CSSParser::<Backend>::ParseFont(&String::from("invalid"), None).is_none());
        for kind in [1, 2] {
            PLAN.with(|p| p.borrow_mut().full = Some(Value { kind }));
            assert!(CSSParser::<Backend>::ParseFont(&String::from("font"), None).is_none());
        }
        PLAN.with(|p| p.borrow_mut().full = Some(Value { kind: 0 }));
        let parsed = CSSParser::<Backend>::ParseFont(&String::from("font"), None).unwrap();
        assert_eq!(parsed.mode, CSSParserMode::kHTMLStandardMode);
        assert!(parsed.values[0].2);
        let context = context(CSSParserMode::kHTMLStandardMode);
        assert!(CSSParser::<Backend>::ParseFontFaceDescriptor(
            CSSPropertyID::kFontFamily,
            &String::from("family"),
            &context
        )
        .is_some());
        assert!(events().iter().any(|e| e == "new-set:kCSSFontFaceRuleMode"));
    }
    #[test]
    fn colors_change_output_only_on_success_and_keep_fast_keyword_rejection() {
        reset();
        let mut color = 99;
        PLAN.with(|p| p.borrow_mut().color_fast = ParseColorResult::kKeyword);
        assert!(!CSSParser::<Backend>::ParseColor(
            &mut color,
            &String::from("keyword"),
            false
        ));
        assert_eq!(color, 99);
        assert!(!events().iter().any(|e| e.starts_with("fast:")));
        PLAN.with(|p| {
            let mut p = p.borrow_mut();
            p.color_fast = ParseColorResult::kFailure;
            p.single = Some(Value { kind: 3 });
        });
        assert!(CSSParser::<Backend>::ParseColor(
            &mut color,
            &String::from("complex"),
            false
        ));
        assert_eq!(color, 30);
        assert!(events()
            .iter()
            .any(|e| e == "fast:kColor:kHTMLStandardMode"));
        assert!(!CSSParser::<Backend>::ParseSystemColor(
            &mut color,
            &String::from("unknown"),
            1,
            None,
            false
        ));
        assert_eq!(color, 30);
        assert!(CSSParser::<Backend>::ParseSystemColor(
            &mut color,
            &String::from("CANVAS"),
            1,
            Some(&2),
            true
        ));
        assert_eq!(color, 40);
        assert!(events().iter().any(|e| e == "system:kCanvas:1:true:true"));
    }
}
