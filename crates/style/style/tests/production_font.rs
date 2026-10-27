use font_engine::fonts::{
    font_description::{FontVariantCaps, FontVariantPosition, LigaturesState, VariantLigatures},
    font_size_adjust::Metric,
    font_variant_east_asian::{EastAsianForm, EastAsianWidth},
    font_variant_numeric::{NumericFigure, NumericFraction, NumericSpacing, Ordinal, SlashedZero},
};
use foundation::{CSSPropertyID, String};
use layoutng_style::style::computed_style::ComputedStyle;
use style::{
    media_queries::MediaValuesCachedData,
    parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseProperty, PropertyParseErrorKind},
    },
    StyleEngine,
};
fn native<'a>(owner: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
    let document = owner.GetDocument();
    let index = (0..document.NodeCount())
        .find(|&i| {
            document
                .Node(i)
                .FindAttribute("id")
                .is_some_and(|a| a.value == id)
        })
        .unwrap();
    unsafe { &*document.ResolvedStyleFor(index).unwrap().native_style.Get() }
}
fn update(owner: &mut dom::DOM) {
    let mut engine = StyleEngine::new(owner);
    engine
        .Update(owner, &MediaValuesCachedData::default(), &[])
        .unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
}
#[test]
fn font_variants_and_adjustment_reach_native_description_and_inherit() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        r#"<div id=p style="font-variant:small-caps no-common-ligatures discretionary-ligatures historical-ligatures no-contextual oldstyle-nums tabular-nums stacked-fractions ordinal slashed-zero ruby proportional-width jis04 stylistic(fancy) styleset(one,two) historical-forms swash(flow) super emoji;font-language-override:'TRK ';font-size-adjust:cap-height calc(0.3 + 0.2)"><div id=c style="font-variant:inherit;font-language-override:inherit;font-size-adjust:inherit"></div><div id=i style="font-variant:initial;font-language-override:initial;font-size-adjust:initial"></div></div>"#,
    );
    update(&mut owner);
    let (p, c, i) = (
        native(&owner, "p").GetFontDescription(),
        native(&owner, "c").GetFontDescription(),
        native(&owner, "i").GetFontDescription(),
    );
    assert_eq!(p.VariantCaps(), FontVariantCaps::kSmallCaps);
    assert_eq!(
        p.GetVariantLigatures(),
        VariantLigatures {
            common: LigaturesState::kDisabledLigaturesState,
            discretionary: LigaturesState::kEnabledLigaturesState,
            historical: LigaturesState::kEnabledLigaturesState,
            contextual: LigaturesState::kDisabledLigaturesState
        }
    );
    let numeric = p.VariantNumeric();
    assert_eq!(numeric.NumericFigureValue(), NumericFigure::kOldstyleNums);
    assert_eq!(numeric.NumericSpacingValue(), NumericSpacing::kTabularNums);
    assert_eq!(
        numeric.NumericFractionValue(),
        NumericFraction::kStackedFractions
    );
    assert_eq!(numeric.OrdinalValue(), Ordinal::kOrdinalOn);
    assert_eq!(numeric.SlashedZeroValue(), SlashedZero::kSlashedZeroOn);
    let east = p.VariantEastAsian();
    assert_eq!(east.Form(), EastAsianForm::kJis04);
    assert_eq!(east.Width(), EastAsianWidth::kProportionalWidth);
    assert!(east.Ruby());
    let alternates = p.FontVariantAlternatesValue().unwrap();
    assert!(alternates.HistoricalForms());
    assert_eq!(alternates.Styleset().len(), 2);
    assert_eq!(unsafe { &*alternates.Stylistic() }.Utf8(), "fancy");
    assert_eq!(unsafe { &*alternates.Swash() }.Utf8(), "flow");
    assert_eq!(
        p.VariantPosition(),
        FontVariantPosition::kSuperVariantPosition
    );
    assert_eq!(p.FontLanguageOverride().Utf8(), "TRK");
    assert_eq!(p.SizeAdjust().GetMetric(), Metric::kCapHeight);
    assert_eq!(p.SizeAdjust().Value(), 0.5);
    assert_eq!(p.VariantCaps(), c.VariantCaps());
    assert_eq!(p.GetVariantLigatures(), c.GetVariantLigatures());
    assert_eq!(p.VariantNumeric(), c.VariantNumeric());
    assert_eq!(p.VariantEastAsian(), c.VariantEastAsian());
    assert!(std::sync::Arc::ptr_eq(
        p.FontVariantAlternatesValue().unwrap(),
        c.FontVariantAlternatesValue().unwrap()
    ));
    assert_eq!(p.SizeAdjust(), c.SizeAdjust());
    assert_eq!(p.FontLanguageOverride(), c.FontLanguageOverride());
    assert_eq!(i.VariantCaps(), FontVariantCaps::kCapsNormal);
    assert_eq!(i.GetVariantLigatures(), VariantLigatures::default());
    assert!(i.VariantNumeric().IsAllNormal() && i.VariantEastAsian().IsAllNormal());
    assert!(i.FontVariantAlternatesValue().is_none());
    assert!(!i.HasSizeAdjust());
    assert!(i.FontLanguageOverride().empty());
}
#[test]
fn font_shorthand_resets_every_stable_font_longhand() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        r#"<div id=a style="font-variant:all-small-caps no-common-ligatures tabular-nums jis78 historical-forms sub emoji;font-size-adjust:from-font;font-language-override:'ENG';font-feature-settings:'liga' 0;font-variation-settings:'wght' 620;font-kerning:none;font-optical-sizing:none;font:italic small-caps bold condensed 20px/1.5 'Example',serif"></div><div id=p style="font:16px monospace"><div id=c style="font:inherit"></div><div id=i style="font:initial"></div></div>"#,
    );
    update(&mut owner);
    let a = native(&owner, "a").GetFontDescription();
    assert_eq!(a.ComputedSize(), 20.0);
    assert_eq!(a.Weight().ToFloat(), 700.0);
    assert_eq!(a.Stretch().ToFloat(), 75.0);
    assert_eq!(a.Style().ToFloat(), 14.0);
    assert_eq!(
        a.GetStyleSyntax(),
        font_engine::fonts::font_description::StyleSyntax::kItalicKeyword
    );
    assert_eq!(native(&owner, "a").LineHeight().PercentValue(), 150.0);
    assert_eq!(a.Family().FamilyName().Utf8(), "Example");
    assert_eq!(a.VariantCaps(), FontVariantCaps::kSmallCaps);
    assert_eq!(a.GetVariantLigatures(), VariantLigatures::default());
    assert!(a.VariantNumeric().IsAllNormal() && a.VariantEastAsian().IsAllNormal());
    assert!(a.FontVariantAlternatesValue().is_none());
    assert_eq!(
        a.VariantPosition(),
        FontVariantPosition::kNormalVariantPosition
    );
    assert_eq!(
        a.VariantEmoji(),
        font_engine::fonts::font_variant_emoji::FontVariantEmoji::kNormalVariantEmoji
    );
    assert!(!a.HasSizeAdjust());
    assert!(a.FontLanguageOverride().empty());
    assert_eq!(a.FeatureSettings().unwrap().size(), 0);
    assert!(a.VariationSettings().is_none());
    assert_eq!(
        a.GetKerning(),
        font_engine::fonts::font_description::Kerning::kAutoKerning
    );
    assert_eq!(
        a.FontOpticalSizing(),
        font_engine::fonts::font_optical_sizing::OpticalSizing::kAutoOpticalSizing
    );
    assert!(native(&owner, "p").GetFontDescription() == native(&owner, "c").GetFontDescription());
    assert_eq!(
        native(&owner, "i").GetFontDescription().VariantCaps(),
        FontVariantCaps::kCapsNormal
    );
}
#[test]
fn invalid_font_grammar_is_rejected_atomically_and_canonicalized() {
    for (id, text) in [
        (
            CSSPropertyID::kFontVariantLigatures,
            "common-ligatures no-common-ligatures",
        ),
        (
            CSSPropertyID::kFontVariantNumeric,
            "lining-nums oldstyle-nums",
        ),
        (CSSPropertyID::kFontVariantEastAsian, "jis78 jis90"),
        (CSSPropertyID::kFontVariantCaps, "small-caps all-small-caps"),
        (
            CSSPropertyID::kFontVariantAlternates,
            "stylistic(a) stylistic(b)",
        ),
        (CSSPropertyID::kFontVariantAlternates, "swash(a,b)"),
        (CSSPropertyID::kFontVariantAlternates, "styleset()"),
        (
            CSSPropertyID::kFontVariantAlternates,
            "styleset(ident(foo))",
        ),
        (CSSPropertyID::kFontVariantAlternates, "annotation(initial)"),
        (CSSPropertyID::kFontLanguageOverride, "'ABCDE'"),
        (CSSPropertyID::kFontLanguageOverride, "'é'"),
        (CSSPropertyID::kFontLanguageOverride, "'   '"),
        (CSSPropertyID::kFontSizeAdjust, "-0.5"),
        (CSSPropertyID::kFont, "all-small-caps 12px serif"),
        (CSSPropertyID::kFont, "12px"),
        (CSSPropertyID::kFont, "12px/ serif"),
        (CSSPropertyID::kFontVariant, "normal ordinal"),
    ] {
        let err = ParseProperty(
            id,
            &String::from(text),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .err()
        .unwrap();
        assert_eq!(err.kind, PropertyParseErrorKind::Invalid, "{text}");
    }
    let parsed = ParseProperty(
        CSSPropertyID::kFontVariantEastAsian,
        &String::from("ruby full-width jis90"),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap();
    assert_eq!(parsed[0].Value().CssText().Utf8(), "jis90 full-width ruby");
    let parsed = ParseProperty(
        CSSPropertyID::kFontVariantAlternates,
        &String::from(
            "swash(flow) historical-forms historical-forms stylistic(fancy) styleset(one,two)",
        ),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap();
    assert_eq!(
        parsed[0].Value().CssText().Utf8(),
        "stylistic(fancy) historical-forms styleset(one, two) swash(flow)"
    );
    let parsed = ParseProperty(
        CSSPropertyID::kFont,
        &String::from("0 serif"),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap();
    assert_eq!(parsed.len(), 19);
}

#[test]
fn core_font_and_text_longhands_reach_native_and_inherit() {
    use font_engine::fonts::{
        font_description::{GenericFamilyType, StyleSyntax},
        font_smoothing_mode::FontSmoothingMode,
        text_rendering_mode::TextRenderingMode,
    };
    use font_engine::FontOrientation;
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        r#"<div id=p style="font-family:'Example',serif,monospace;font-size:large;font-weight:550;font-style:oblique 0.125turn;text-rendering:optimizeLegibility;-webkit-font-smoothing:antialiased;writing-mode:vertical-rl;text-orientation:upright;word-break:auto-phrase;letter-spacing:10%;word-spacing:calc(2px + 20%)"><div id=c style="font-size:200%;font-weight:bolder"></div><div id=i style="font-family:initial;font-size:initial;font-style:initial;text-rendering:initial;-webkit-font-smoothing:initial;text-orientation:initial;word-break:initial;letter-spacing:initial;word-spacing:normal"></div></div>"#,
    );
    update(&mut owner);
    let (p, c, i) = (
        native(&owner, "p"),
        native(&owner, "c"),
        native(&owner, "i"),
    );
    let (pd, cd, id) = (
        p.GetFontDescription(),
        c.GetFontDescription(),
        i.GetFontDescription(),
    );
    assert_eq!(pd.Family().ToString(), "Example, serif, monospace");
    assert_eq!(pd.GenericFamily(), GenericFamilyType::kMonospaceFamily);
    assert_eq!(cd.GenericFamily(), pd.GenericFamily());
    assert_eq!(pd.SpecifiedSize(), 18.0);
    assert_eq!(pd.KeywordSize(), 5);
    assert!(!pd.IsAbsoluteSize());
    assert_eq!(cd.SpecifiedSize(), 36.0);
    assert_eq!(cd.KeywordSize(), 0);
    assert!(!cd.IsAbsoluteSize());
    assert_eq!(cd.Weight().ToFloat(), 900.0);
    assert_eq!(pd.Style().ToFloat(), 45.0);
    assert_eq!(pd.GetStyleSyntax(), StyleSyntax::kExplicitAngle);
    assert_eq!(cd.GetStyleSyntax(), StyleSyntax::kExplicitAngle);
    assert_eq!(pd.TextRendering(), TextRenderingMode::kOptimizeLegibility);
    assert_eq!(cd.TextRendering(), pd.TextRendering());
    assert_eq!(pd.FontSmoothing(), FontSmoothingMode::kAntialiased);
    assert_eq!(cd.FontSmoothing(), pd.FontSmoothing());
    assert_eq!(pd.Orientation(), FontOrientation::kVerticalUpright);
    assert_eq!(cd.Orientation(), FontOrientation::kVerticalUpright);
    assert_eq!(p.WordBreak(), foundation::EWordBreak::kAutoPhrase);
    assert_eq!(c.WordBreak(), p.WordBreak());
    assert_eq!(p.ComputedLetterSpacing().PercentValue(), 10.0);
    assert!((p.LetterSpacing() - 1.8).abs() < 0.001);
    assert!((c.LetterSpacing() - 3.6).abs() < 0.001);
    assert!((p.WordSpacing() - 5.6).abs() < 0.001);
    assert!((c.WordSpacing() - 9.2).abs() < 0.001);
    assert_eq!(id.GenericFamily(), GenericFamilyType::kStandardFamily);
    assert_eq!(id.SpecifiedSize(), 16.0);
    assert_eq!(id.KeywordSize(), 4);
    assert_eq!(id.Style().ToFloat(), 0.0);
    assert_eq!(id.TextRendering(), TextRenderingMode::kAutoTextRendering);
    assert_eq!(id.FontSmoothing(), FontSmoothingMode::kAutoSmoothing);
    assert_eq!(id.Orientation(), FontOrientation::kVerticalMixed);
    assert_eq!(i.WordBreak(), foundation::EWordBreak::kNormal);
    assert_eq!(i.LetterSpacing(), 0.0);
    assert_eq!(i.WordSpacing(), 0.0);
}

#[test]
fn font_size_keywords_use_chromium_table_and_preserve_absolute_metadata() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        r#"<div id=xs style="font-size:x-small"></div><div id=xxl style="font-size:xx-large"></div><div id=xxxl style="font-size:-webkit-xxx-large"></div><div id=p style="font-size:20px;font-style:italic"><div id=c style="font-size:smaller;font-style:inherit;font-weight:lighter"></div><div id=r style="font-size:2em"></div></div>"#,
    );
    update(&mut owner);
    for (name, size, keyword) in [("xs", 10.0, 2), ("xxl", 32.0, 7), ("xxxl", 48.0, 8)] {
        let d = native(&owner, name).GetFontDescription();
        assert_eq!(d.SpecifiedSize(), size);
        assert_eq!(d.KeywordSize(), keyword);
        assert!(!d.IsAbsoluteSize());
    }
    for name in ["p", "c", "r"] {
        assert!(native(&owner, name).GetFontDescription().IsAbsoluteSize());
    }
    assert!((native(&owner, "c").GetFontDescription().SpecifiedSize() - 20.0 / 1.2).abs() < 0.001);
    assert_eq!(
        native(&owner, "r").GetFontDescription().SpecifiedSize(),
        40.0
    );
    assert_eq!(
        native(&owner, "c").GetFontDescription().GetStyleSyntax(),
        font_engine::fonts::font_description::StyleSyntax::kItalicKeyword
    );
    assert_eq!(
        native(&owner, "c").GetFontDescription().Weight().ToFloat(),
        100.0
    );
}

#[test]
fn oblique_angles_are_typed_and_invalid_ranges_are_rejected() {
    for text in [
        "oblique 91deg",
        "oblique -91deg",
        "oblique .5turn",
        "oblique 5px",
        "oblique 1deg 2deg",
    ] {
        assert!(
            ParseProperty(
                CSSPropertyID::kFontStyle,
                &String::from(text),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .is_err(),
            "{text}"
        );
    }
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        r#"<div id=a style="font-style:oblique calc(10deg + 20deg)"></div><div id=b style="font-style:oblique"></div>"#,
    );
    update(&mut owner);
    assert_eq!(
        native(&owner, "a").GetFontDescription().Style().ToFloat(),
        30.0
    );
    assert_eq!(
        native(&owner, "a").GetFontDescription().GetStyleSyntax(),
        font_engine::fonts::font_description::StyleSyntax::kExplicitAngle
    );
    assert_eq!(
        native(&owner, "b").GetFontDescription().Style().ToFloat(),
        14.0
    );
    assert_eq!(
        native(&owner, "b").GetFontDescription().GetStyleSyntax(),
        font_engine::fonts::font_description::StyleSyntax::kImplicitAngle
    );
}
