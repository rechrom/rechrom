// Fixture parser injects an actual CSSSelector vector into the translated
// CSSParser facade. It asserts the query cache's parser context and arguments;
// it does not implement an alternate CSS parser.
#![allow(unused_variables)]
use super::*;
use crate::css_property_value_set::SetResult;
use crate::parser::allowed_rules::AllowedRules;
use crate::parser::css_parser::{ParseColorResult, SupportsResult};
use crate::parser::css_parser_context::*;
use crate::parser::css_parser_local_context::CSSParserLocalContext;
use crate::parser::css_parser_mode::{CSSDeferPropertyParsing, CSSParserMode};
use crate::parser::css_parser_token::CSSParserToken;
use crate::parser::css_parser_token_stream::{CSSParserTokenStream, TokenStreamTokenizer};
use foundation::{CSSPropertyID, CSSValueID, StringView};
struct Parser;
struct Tokenizer;
struct Document;
type Context<P> = CSSParserContext<P>;
type Stream<P> = CSSParserTokenStream<'static, <P as CSSParserBackend>::Tokenizer>;
thread_local! { static PARSE_CALLS: Cell<usize> = const { Cell::new(0) }; static REJECT: Cell<bool> = const { Cell::new(false) }; }
impl CSSParserContextDocument<Parser> for Document {
    fn CountUse(&self, feature: ()) {
        panic!("unexpected counter");
    }
    fn CountWebDXFeature(&self, feature: ()) {
        panic!("unexpected counter");
    }
    fn CountProperty(&self, property: CSSPropertyID) {
        panic!("unexpected counter");
    }
    fn GetExecutionContext(&self) -> Option<Rc<()>> {
        None
    }
    fn IsForMarkupSanitization(&self) -> bool {
        false
    }
}
impl CSSParserContextPlatform for Parser {
    type URL = u8;
    type TextEncoding = ();
    type Referrer = ();
    type ReferrerPolicy = ();
    type DOMWrapperWorld = ();
    type ExecutionContext = ();
    type WebFeature = ();
    type WebDXFeature = ();
    fn NullURL() -> Self::URL {
        panic!("unexpected cache test dependency: NullURL")
    }
    fn EmptyURL() -> Self::URL {
        panic!("unexpected cache test dependency: EmptyURL")
    }
    fn EmptyTextEncoding() -> Self::TextEncoding {
        ()
    }
    fn IsEncodingValid(encoding: &Self::TextEncoding) -> bool {
        panic!("unexpected cache test dependency: IsEncodingValid")
    }
    fn EmptyReferrer() -> Self::Referrer {
        ()
    }
    fn MakeReferrer(referrer: String, policy: Self::ReferrerPolicy) -> Self::Referrer {
        panic!("unexpected cache test dependency: MakeReferrer")
    }
    fn StrippedForUseAsReferrer(url: &Self::URL) -> String {
        panic!("unexpected cache test dependency: StrippedForUseAsReferrer")
    }
    fn ResolveURL(
        base: &Self::URL,
        url: &String,
        encoding: Option<&Self::TextEncoding>,
    ) -> Self::URL {
        panic!("unexpected cache test dependency: ResolveURL")
    }
    fn CSSParserIgnoreCharsetForURLsEnabled() -> bool {
        false
    }
    fn CountDeprecation(context: Option<&Rc<Self::ExecutionContext>>, feature: Self::WebFeature) {
        panic!("unexpected cache test dependency: CountDeprecation")
    }
}
impl CSSParserBackend for Parser {
    type Tokenizer = Tokenizer;
    type StyleRule = ();
    type StyleRuleBase = ();
    type StyleRuleKeyframe = ();
    type StyleSheetContents = ();
    type CSSParserObserver = ();
    type CSSSelector = CSSSelector;
    type CSSSelectorList = ();
    type MutablePropertySet = ();
    type ImmutablePropertySet = ();
    type CSSValue = ();
    type CSSPrimitiveValue = ();
    type CSSPropertyValue = ();
    type Element = ();
    type KeyframeOffset = ();
    type ParseSheetResult = ();
    type Color = ();
    type ColorScheme = ();
    type ColorProvider = ();
    type ValueRange = ();
    type ParserImpl = ();
    fn IsAllocationAllowed() -> bool {
        true
    }
    fn TopLevelRules() -> AllowedRules {
        panic!("unexpected cache test dependency: TopLevelRules")
    }
    fn NestedGroupRules() -> AllowedRules {
        panic!("unexpected cache test dependency: NestedGroupRules")
    }
    fn PageMarginRules() -> AllowedRules {
        panic!("unexpected cache test dependency: PageMarginRules")
    }
    fn KeyframeRules() -> AllowedRules {
        panic!("unexpected cache test dependency: KeyframeRules")
    }
    fn StyleSheetParserContext(sheet: &Self::StyleSheetContents) -> Rc<Context<Self>> {
        panic!("unexpected cache test dependency: StyleSheetParserContext")
    }
    fn LocalDOMWindowDocument(context: &Self::ExecutionContext) -> Option<DocumentSnapshot<Self>> {
        panic!("unexpected cache test dependency: LocalDOMWindowDocument")
    }
    fn ExecutionSecureContextMode(context: &Self::ExecutionContext) -> SecureContextMode {
        panic!("unexpected cache test dependency: ExecutionSecureContextMode")
    }
    fn MutableSetParserMode(set: &Self::MutablePropertySet) -> CSSParserMode {
        panic!("unexpected cache test dependency: MutableSetParserMode")
    }
    fn NewMutablePropertySet(mode: CSSParserMode) -> Self::MutablePropertySet {
        panic!("unexpected cache test dependency: NewMutablePropertySet")
    }
    fn IsPropertySetEmpty(set: &Self::MutablePropertySet) -> bool {
        panic!("unexpected cache test dependency: IsPropertySetEmpty")
    }
    fn GetPropertyCSSValue(
        set: &Self::MutablePropertySet,
        property: CSSPropertyID,
    ) -> Option<Rc<Self::CSSValue>> {
        panic!("unexpected cache test dependency: GetPropertyCSSValue")
    }
    fn SetLonghandProperty(
        set: &mut Self::MutablePropertySet,
        resolved: CSSPropertyID,
        value: Rc<Self::CSSValue>,
        important: bool,
    ) -> SetResult {
        panic!("unexpected cache test dependency: SetLonghandProperty")
    }
    fn MakePropertyValue(
        resolved: CSSPropertyID,
        value: Rc<Self::CSSValue>,
        important: bool,
    ) -> Self::CSSPropertyValue {
        panic!("unexpected cache test dependency: MakePropertyValue")
    }
    fn IsCSSWideKeyword(value: &Self::CSSValue) -> bool {
        panic!("unexpected cache test dependency: IsCSSWideKeyword")
    }
    fn IsPendingSubstitutionValue(value: &Self::CSSValue) -> bool {
        panic!("unexpected cache test dependency: IsPendingSubstitutionValue")
    }
    fn IsValidVariableName(name: &AtomicString) -> bool {
        panic!("unexpected cache test dependency: IsValidVariableName")
    }
    fn ParseDeclarationListImpl(
        set: &mut Self::MutablePropertySet,
        text: &String,
        context: &Context<Self>,
    ) -> bool {
        panic!("unexpected cache test dependency: ParseDeclarationListImpl")
    }
    fn ParseNestedDeclarationsRuleImpl(
        context: &Context<Self>,
        nesting: CSSNestingType,
        parent: Option<&Self::StyleRule>,
        text: StringView,
    ) -> Option<Rc<Self::StyleRuleBase>> {
        panic!("unexpected cache test dependency: ParseNestedDeclarationsRuleImpl")
    }
    fn ParseDeclarationListForInspectorImpl(
        text: &String,
        context: &Context<Self>,
        observer: &mut Self::CSSParserObserver,
    ) {
        panic!("unexpected cache test dependency: ParseDeclarationListForInspectorImpl")
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
        assert_eq!(nesting, CSSNestingType::kNone);
        assert!(parent.is_none());
        assert!(sheet.is_none());
        assert!(!semicolon_aborts_nested_selector);
        assert!(context.IsOriginClean());
        assert_eq!(*context.BaseURL(), 7);
        assert_eq!(context.Mode(), CSSParserMode::kHTMLStandardMode);
        assert!(context.GetDocument().is_some());
        PARSE_CALLS.with(|c| c.set(c.get() + 1));
        if !REJECT.with(Cell::get) {
            let mut selector = class("a", RelationType::kSubSelector);
            selector.SetLastInComplexSelector(true);
            arena.push(selector);
        }
        arena.as_mut_slice()
    }
    fn ParsePageSelectorImpl(
        stream: &mut Stream<Self>,
        sheet: Option<&mut Self::StyleSheetContents>,
        context: &Context<Self>,
    ) -> Option<Rc<Self::CSSSelectorList>> {
        panic!("unexpected cache test dependency: ParsePageSelectorImpl")
    }
    fn ParseRuleImpl(
        text: &String,
        context: &Context<Self>,
        nesting: CSSNestingType,
        parent: Option<&Self::StyleRule>,
        sheet: Option<&mut Self::StyleSheetContents>,
        rules: AllowedRules,
    ) -> Option<Rc<Self::StyleRuleBase>> {
        panic!("unexpected cache test dependency: ParseRuleImpl")
    }
    fn ParseStyleSheetImpl(
        text: &String,
        context: &Context<Self>,
        sheet: &mut Self::StyleSheetContents,
        defer: CSSDeferPropertyParsing,
        allow_import_rules: bool,
    ) -> Self::ParseSheetResult {
        panic!("unexpected cache test dependency: ParseStyleSheetImpl")
    }
    fn ParseStyleSheetForInspectorImpl(
        text: &String,
        context: &Context<Self>,
        sheet: &mut Self::StyleSheetContents,
        observer: &mut Self::CSSParserObserver,
    ) {
        panic!("unexpected cache test dependency: ParseStyleSheetForInspectorImpl")
    }
    fn MaybeParseValueFast(
        property: CSSPropertyID,
        text: StringView,
        context: &Context<Self>,
    ) -> Option<Rc<Self::CSSValue>> {
        panic!("unexpected cache test dependency: MaybeParseValueFast")
    }
    fn ParseSingleValueImpl(
        property: CSSPropertyID,
        stream: &mut Stream<Self>,
        context: &Context<Self>,
    ) -> Option<Rc<Self::CSSValue>> {
        panic!("unexpected cache test dependency: ParseSingleValueImpl")
    }
    fn ParseValueImpl(
        set: &mut Self::MutablePropertySet,
        property: CSSPropertyID,
        text: StringView,
        important: bool,
        context: &Context<Self>,
    ) -> SetResult {
        panic!("unexpected cache test dependency: ParseValueImpl")
    }
    fn ParseValueVectorImpl(
        result: &mut Vec<Self::CSSPropertyValue>,
        resolved: CSSPropertyID,
        text: StringView,
        context: &Context<Self>,
    ) -> u32 {
        panic!("unexpected cache test dependency: ParseValueVectorImpl")
    }
    fn ParseVariableValueImpl(
        set: &mut Self::MutablePropertySet,
        name: &AtomicString,
        text: StringView,
        important: bool,
        context: &Context<Self>,
        animation_tainted: bool,
    ) -> SetResult {
        panic!("unexpected cache test dependency: ParseVariableValueImpl")
    }
    fn ParseInlineStyleDeclarationForElementImpl(
        text: &String,
        element: Option<&Self::Element>,
    ) -> Rc<Self::ImmutablePropertySet> {
        panic!("unexpected cache test dependency: ParseInlineStyleDeclarationForElementImpl")
    }
    fn ParseInlineStyleDeclarationImpl(
        text: &String,
        mode: CSSParserMode,
        secure: SecureContextMode,
        document: Option<&DocumentHandle<Self>>,
    ) -> Rc<Self::ImmutablePropertySet> {
        panic!("unexpected cache test dependency: ParseInlineStyleDeclarationImpl")
    }
    fn ParseKeyframeKeyListImpl(
        context: &Context<Self>,
        text: &String,
    ) -> Option<Vec<Self::KeyframeOffset>> {
        panic!("unexpected cache test dependency: ParseKeyframeKeyListImpl")
    }
    fn ToKeyframeRule(
        rule: Option<Rc<Self::StyleRuleBase>>,
    ) -> Option<Rc<Self::StyleRuleKeyframe>> {
        panic!("unexpected cache test dependency: ToKeyframeRule")
    }
    fn ParseCustomPropertyNameImpl(text: &String) -> String {
        panic!("unexpected cache test dependency: ParseCustomPropertyNameImpl")
    }
    fn NewParserImpl(context: &Context<Self>) -> Self::ParserImpl {
        panic!("unexpected cache test dependency: NewParserImpl")
    }
    fn ConsumeSupportsCondition(
        stream: &mut Stream<Self>,
        parser: &mut Self::ParserImpl,
    ) -> SupportsResult {
        panic!("unexpected cache test dependency: ConsumeSupportsCondition")
    }
    fn NamedColor(text: &String) -> Option<Self::Color> {
        panic!("unexpected cache test dependency: NamedColor")
    }
    fn ParseColorFast(
        text: &String,
        mode: CSSParserMode,
        color: &mut Self::Color,
    ) -> ParseColorResult {
        panic!("unexpected cache test dependency: ParseColorFast")
    }
    fn CSSColorValue(value: &Self::CSSValue) -> Option<Self::Color> {
        panic!("unexpected cache test dependency: CSSColorValue")
    }
    fn IsSystemColorIncludingDeprecated(id: CSSValueID) -> bool {
        panic!("unexpected cache test dependency: IsSystemColorIncludingDeprecated")
    }
    fn SystemColor(
        id: CSSValueID,
        scheme: Self::ColorScheme,
        provider: Option<&Self::ColorProvider>,
        expose_accent: bool,
    ) -> Self::Color {
        panic!("unexpected cache test dependency: SystemColor")
    }
    fn ConsumeLengthOrPercent(
        stream: &mut Stream<Self>,
        context: &Context<Self>,
        local: &mut CSSParserLocalContext,
        range: Self::ValueRange,
    ) -> Option<Rc<Self::CSSPrimitiveValue>> {
        panic!("unexpected cache test dependency: ConsumeLengthOrPercent")
    }
}
impl TokenStreamTokenizer for Tokenizer {
    fn new(text: StringView, offset: u32) -> Self {
        Self
    }
    fn TokenizeSingle(&mut self) -> CSSParserToken {
        panic!("unexpected cache test dependency: TokenizeSingle")
    }
    fn TokenizeSingleWithComments(&mut self) -> CSSParserToken {
        panic!("unexpected cache test dependency: TokenizeSingleWithComments")
    }
    fn Offset(&self) -> u32 {
        panic!("unexpected cache test dependency: Offset")
    }
    fn PreviousOffset(&self) -> u32 {
        panic!("unexpected cache test dependency: PreviousOffset")
    }
    fn StringRangeAt(&self, start: u32, length: u32) -> StringView {
        panic!("unexpected cache test dependency: StringRangeAt")
    }
    fn StringRangeFrom(&self, start: u32) -> StringView {
        panic!("unexpected cache test dependency: StringRangeFrom")
    }
    fn SkipToEndOfBlock(&mut self, offset: u32) {
        panic!("unexpected cache test dependency: SkipToEndOfBlock")
    }
    fn Restore(&mut self, next: &CSSParserToken, offset: u32) -> CSSParserToken {
        panic!("unexpected cache test dependency: Restore")
    }
    fn TokenCount(&self) -> u32 {
        panic!("unexpected cache test dependency: TokenCount")
    }
    fn UnicodeRangesAllowed(&self) -> bool {
        panic!("unexpected cache test dependency: UnicodeRangesAllowed")
    }
    fn SetUnicodeRangesAllowed(&mut self, allowed: bool) {
        panic!("unexpected cache test dependency: SetUnicodeRangesAllowed")
    }
    fn PopBlockStack(&mut self) {
        panic!("unexpected cache test dependency: PopBlockStack")
    }
}

#[test]
fn cache_parses_once_invalidates_and_reports_source_syntax_errors() {
    PARSE_CALLS.with(|c| c.set(0));
    REJECT.with(|c| c.set(false));
    let b = Backend::new(&[None], &[""], &[""]);
    let document = DocumentSnapshot::<Parser> {
        document: Rc::new(Document),
        base_url: 7,
        in_quirks_mode: false,
        is_html_document: true,
        referrer_policy: (),
        execution_context: None,
    };
    let mut cache = SelectorQueryCache::new();
    let selectors = AtomicString::from_str(".a");
    let first = cache.Add::<Parser, _>(&selectors, &document, &*b).unwrap();
    assert!(Rc::ptr_eq(
        &first,
        &cache.Add::<Parser, _>(&selectors, &document, &*b).unwrap()
    ));
    assert_eq!(PARSE_CALLS.with(Cell::get), 1);
    cache.Invalidate();
    let second = cache.Add::<Parser, _>(&selectors, &document, &*b).unwrap();
    assert!(!Rc::ptr_eq(&first, &second));
    assert_eq!(PARSE_CALLS.with(Cell::get), 2);
    assert_eq!(
        cache
            .Add::<Parser, _>(&AtomicString::from_str(""), &document, &*b)
            .err()
            .unwrap()
            .message
            .Utf8(),
        "The provided selector is empty."
    );
    assert_eq!(PARSE_CALLS.with(Cell::get), 2);
    REJECT.with(|c| c.set(true));
    assert_eq!(
        cache
            .Add::<Parser, _>(&AtomicString::from_str("["), &document, &*b)
            .err()
            .unwrap()
            .message
            .Utf8(),
        "'[' is not a valid selector."
    );
    let surrogate = cache
        .Add::<Parser, _>(&AtomicString::from_utf16(&[0xd800]), &document, &*b)
        .err()
        .unwrap();
    assert_eq!(surrogate.message.CodeUnitAt(1), 0xd800);
    assert_eq!(cache.entries_.len(), 1);
    REJECT.with(|c| c.set(false));
    for i in 0..300 {
        cache
            .Add::<Parser, _>(
                &AtomicString::from_str(&format!(".fixture{i}")),
                &document,
                &*b,
            )
            .unwrap();
    }
    assert_eq!(cache.entries_.len(), 256);
    cache.Invalidate();
    assert!(cache.entries_.is_empty());
}
