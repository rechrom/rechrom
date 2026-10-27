use crate::css_parser::{CSSParser, CSSParserBackend};
use crate::css_parser_context::{CSSParserContext, SecureContextMode};
use foundation::{AtomicString, String, StringView};

// The actual CSSParser is used directly. This optional extension constrains its
// associated property/value types to the shared model store; no parallel parser
// API or copied declaration storage is introduced.
use style_model::css_property_value_set::{
    CSSPropertyValueSetBackend, MutableCSSPropertyValueSet, SetResult,
};

pub trait CSSPropertyValueSetParserBackend: CSSPropertyValueSetBackend {
    type Parser: CSSParserBackend<
        MutablePropertySet = MutableCSSPropertyValueSet<Self>,
        CSSValue = style_model::css_value::CSSValue<Self>,
        CSSPropertyValue = style_model::css_property_value::CSSPropertyValue<Self>,
    >;

    // Source sheet constructor also preserves its SingleOwnerDocument.
    fn DeclarationParserContext(
        sheet: &<Self::Parser as CSSParserBackend>::StyleSheetContents,
    ) -> CSSParserContext<Self::Parser>;
}

// Rust cannot add inherent methods to a type owned by `style_model`; this trait
// is the parser-side half of Blink's MutableCSSPropertyValueSet API. Call sites
// importing this module's public surface retain method-call syntax.
pub trait MutableCSSPropertyValueSetParserExt<D: CSSPropertyValueSetParserBackend> {
    fn ParseAndSetProperty(
        &mut self,
        id: style_model::css_property_names::CSSPropertyID,
        value: StringView,
        important: bool,
        secure: SecureContextMode,
        sheet: Option<&<D::Parser as CSSParserBackend>::StyleSheetContents>,
    ) -> SetResult;

    fn ParseAndSetCustomProperty(
        &mut self,
        name: &AtomicString,
        value: StringView,
        important: bool,
        secure: SecureContextMode,
        sheet: Option<&<D::Parser as CSSParserBackend>::StyleSheetContents>,
        animation_tainted: bool,
    ) -> SetResult;

    fn ParseDeclarationList(
        &mut self,
        text: &String,
        secure: SecureContextMode,
        sheet: Option<&<D::Parser as CSSParserBackend>::StyleSheetContents>,
    );
}

#[allow(non_snake_case)]
impl<D: CSSPropertyValueSetParserBackend> MutableCSSPropertyValueSetParserExt<D>
    for MutableCSSPropertyValueSet<D>
{
    // cpp: css_property_value_set.cc:484-520
    fn ParseAndSetProperty(
        &mut self,
        id: style_model::css_property_names::CSSPropertyID,
        value: StringView,
        important: bool,
        secure: SecureContextMode,
        sheet: Option<&<D::Parser as CSSParserBackend>::StyleSheetContents>,
    ) -> SetResult {
        if value.IsEmpty() {
            return if self.RemoveProperty(
                style_model::css_property_names::ResolveCSSPropertyID(id),
                None,
            ) {
                SetResult::kChangedPropertySet
            } else {
                SetResult::kUnchanged
            };
        }
        CSSParser::<D::Parser>::ParseValueWithSheet(self, id, value, important, secure, sheet, None)
    }

    fn ParseAndSetCustomProperty(
        &mut self,
        name: &AtomicString,
        value: StringView,
        important: bool,
        secure: SecureContextMode,
        sheet: Option<&<D::Parser as CSSParserBackend>::StyleSheetContents>,
        animation_tainted: bool,
    ) -> SetResult {
        if value.IsEmpty() {
            return if self.RemoveProperty(name, None) {
                SetResult::kChangedPropertySet
            } else {
                SetResult::kUnchanged
            };
        }
        CSSParser::<D::Parser>::ParseValueForCustomProperty(
            self,
            name,
            value,
            important,
            secure,
            sheet,
            animation_tainted,
        )
    }

    // cpp: css_property_value_set.cc:662-678
    fn ParseDeclarationList(
        &mut self,
        text: &String,
        secure: SecureContextMode,
        sheet: Option<&<D::Parser as CSSParserBackend>::StyleSheetContents>,
    ) {
        self.Clear();
        let context = if let Some(sheet) = sheet {
            let context = D::DeclarationParserContext(sheet);
            context.SetMode(self.CssParserMode());
            context
        } else {
            CSSParserContext::<D::Parser>::FromMode(self.CssParserMode(), secure, None)
        };
        CSSParser::<D::Parser>::ParseDeclarationList(&context, self, text);
    }
}
