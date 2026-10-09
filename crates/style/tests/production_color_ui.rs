use foundation::{CSSPropertyID as P, String};
use layoutng_style::style::computed_style::ComputedStyle;
use style::{
    media_queries::{MediaValuesCachedData, PreferredColorScheme},
    parser::{
        css_parser_mode::CSSParserMode as Mode,
        production_property_parser::{ParseProperty, PropertyParseErrorKind},
    },
    resolver::production_style_builder::ColorSchemeSettings,
    StyleEngine,
};
fn native<'a>(owner: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
    let document = owner.GetDocument();
    let index = (0..document.NodeCount())
        .find(|&n| {
            document
                .Node(n)
                .FindAttribute("id")
                .is_some_and(|a| a.value == id)
        })
        .unwrap();
    unsafe { &*document.ResolvedStyleFor(index).unwrap().native_style.Get() }
}
fn color(r: i32, g: i32, b: i32, a: i32) -> foundation::Color {
    foundation::Color::FromRGBA(r, g, b, a)
}
fn schemes(style: &ComputedStyle) -> Vec<std::string::String> {
    style
        .ColorScheme()
        .iter()
        .map(|s| String::from_utf16(s.utf16_units().unwrap_or_default()).Utf8())
        .collect()
}
#[test]
fn ordinary_auto_current_and_literal_colors_reach_native_and_css_wide() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=p style='accent-color:currentColor;caret-color:rgba(10,20,30,.5)'><div id=c style='accent-color:inherit;caret-color:unset'></div><div id=i style='accent-color:initial;caret-color:initial'></div><div id=v style='accent-color:#1234;caret-color:currentColor'></div><div id=a style='accent-color:auto;caret-color:auto'></div></div>");
    let mut engine = StyleEngine::new(&owner);
    engine
        .Update(&mut owner, &MediaValuesCachedData::default(), &[])
        .unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let p = native(&owner, "p");
    assert!(p.AccentColor().IsCurrentColor());
    assert_eq!(p.CaretColor().ToStyleColor().GetColor(), {
        let mut rgba = color(10, 20, 30, 255);
        rgba.SetAlpha(0.5);
        rgba
    });
    assert!(p.CaretColor().TextColor().IsAutoColor());
    let c = native(&owner, "c");
    assert!(c.AccentColor() == p.AccentColor());
    assert!(c.CaretColor() == p.CaretColor());
    assert!(c.HasExplicitInheritance());
    let i = native(&owner, "i");
    assert!(i.AccentColor().IsAutoColor() && i.CaretColor().IsAutoColor());
    let v = native(&owner, "v");
    assert_eq!(
        v.AccentColor().ToStyleColor().GetColor(),
        color(17, 34, 51, 68)
    );
    assert!(v.CaretColor().IsCurrentColor());
    let a = native(&owner, "a");
    assert!(a.AccentColor().IsAutoColor() && a.CaretColor().IsAutoColor());
}
#[test]
fn visited_native_slots_are_separate_and_inherit_parent_unvisited_colors() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p><div id=c></div><div id=i></div><div id=v></div><div id=l></div></div>",
    );
    // Internal properties are UA-only in Chromium. This exercises the real UA
    // declaration/cascade/builder path without inventing a visited-history owner.
    let parsed = style::production_style_sheet::ParseStyleSheet(&String::from("#p{caret-color:red;outline-color:#00ff00;border-top-color:red;border-right-color:green;border-bottom-color:blue;border-left-color:yellow;-internal-visited-caret-color:blue;-internal-visited-outline-color:red;-internal-visited-border-top-color:blue;-internal-visited-border-right-color:blue;-internal-visited-border-bottom-color:red;-internal-visited-border-left-color:blue}#c{caret-color:inherit;outline-color:inherit;-internal-visited-caret-color:inherit;-internal-visited-outline-color:inherit;-internal-visited-border-top-color:inherit;-internal-visited-border-right-color:inherit;-internal-visited-border-bottom-color:inherit;-internal-visited-border-left-color:inherit}#i{-internal-visited-caret-color:initial;-internal-visited-outline-color:unset;-internal-visited-border-top-color:initial}#v{-internal-visited-caret-color:currentColor;-internal-visited-outline-color:currentColor;-internal-visited-border-top-color:currentColor}#l{direction:rtl;-internal-visited-border-inline-start-color:#010203}"), Mode::kUASheetMode);
    assert!(parsed.diagnostics.is_empty());
    let ua = style::production_style_sheet_projection::ProjectStyleSheet(&parsed);
    let mut engine = StyleEngine::new(&owner);
    engine
        .Update(&mut owner, &MediaValuesCachedData::default(), &[ua])
        .unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let p = native(&owner, "p");
    assert_eq!(
        p.CaretColor().ToStyleColor().GetColor(),
        color(255, 0, 0, 255)
    );
    assert_eq!(
        p.InternalVisitedCaretColor().ToStyleColor().GetColor(),
        color(0, 0, 255, 255)
    );
    assert_eq!(p.OutlineColor().GetColor(), color(0, 255, 0, 255));
    assert_eq!(
        p.InternalVisitedOutlineColor().GetColor(),
        color(255, 0, 0, 255)
    );
    let c = native(&owner, "c");
    assert!(c.InternalVisitedCaretColor() == p.CaretColor());
    assert!(c.InternalVisitedOutlineColor() == p.OutlineColor());
    assert!(c.InternalVisitedBorderTopColor() == p.BorderTopColor());
    assert!(c.InternalVisitedBorderRightColor() == p.BorderRightColor());
    assert!(c.InternalVisitedBorderBottomColor() == p.BorderBottomColor());
    assert!(c.InternalVisitedBorderLeftColor() == p.BorderLeftColor());
    let i = native(&owner, "i");
    assert!(i.InternalVisitedCaretColor().IsAutoColor());
    assert!(i.InternalVisitedOutlineColor().IsCurrentColor());
    assert!(i.InternalVisitedBorderTopColor().IsCurrentColor());
    let v = native(&owner, "v");
    assert!(v.InternalVisitedCaretColor().IsCurrentColor());
    assert!(v.InternalVisitedOutlineColor().IsCurrentColor());
    assert!(v.InternalVisitedBorderTopColor().IsCurrentColor());
    assert_eq!(
        native(&owner, "l")
            .InternalVisitedBorderRightColor()
            .GetColor(),
        color(1, 2, 3, 255)
    );
}
#[test]
fn color_scheme_native_flags_follow_preference_only_page_settings_and_force_dark() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=p style='color-scheme:only light'><div id=c style='color-scheme:inherit'></div></div><div id=d style='color-scheme:dark'></div><div id=b style='color-scheme:light dark'></div><div id=n style='color-scheme:normal'></div><div id=i style='color-scheme:initial'></div><div id=u style='color-scheme:MyTheme LIGHT dark light only'></div>");
    let mut media = MediaValuesCachedData::default();
    media.preferred_color_scheme = PreferredColorScheme::kDark;
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media, &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    assert_eq!(schemes(native(&owner, "p")), vec!["light", "only"]);
    assert!(!native(&owner, "p").DarkColorScheme());
    assert!(native(&owner, "d").DarkColorScheme() && native(&owner, "b").DarkColorScheme());
    assert!(native(&owner, "n").ColorSchemeFlagsIsNormal());
    assert_eq!(
        schemes(native(&owner, "u")),
        vec!["MyTheme", "light", "dark", "light", "only"]
    );
    assert!(!native(&owner, "u").ColorSchemeFlagsIsNormal());
    assert_eq!(schemes(native(&owner, "c")), schemes(native(&owner, "p")));
    engine
        .SetColorSchemeSettings(
            &mut owner,
            ColorSchemeSettings {
                page_color_schemes: 1,
                force_dark: true,
            },
        )
        .unwrap();
    engine.Update(&mut owner, &media, &[]).unwrap();
    assert!(!native(&owner, "p").DarkColorScheme());
    assert!(native(&owner, "n").DarkColorScheme() && native(&owner, "i").DarkColorScheme());
    assert!(!native(&owner, "n").ColorSchemeForced());
    assert!(!native(&owner, "n").ColorSchemeFlagsIsNormal());
    assert!(schemes(native(&owner, "n")).is_empty());
    media.preferred_color_scheme = PreferredColorScheme::kLight;
    engine.Update(&mut owner, &media, &[]).unwrap();
    assert!(native(&owner, "p").DarkColorScheme() && native(&owner, "p").ColorSchemeForced());
    assert!(native(&owner, "d").ColorSchemeForced());
    assert_eq!(
        native(&owner, "c").DarkColorScheme(),
        native(&owner, "p").DarkColorScheme()
    );
    assert_eq!(
        native(&owner, "c").ColorSchemeForced(),
        native(&owner, "p").ColorSchemeForced()
    );
    engine
        .SetColorSchemeSettings(&mut owner, ColorSchemeSettings::default())
        .unwrap();
    engine.Update(&mut owner, &media, &[]).unwrap();
    assert!(!native(&owner, "b").DarkColorScheme());
    assert!(native(&owner, "d").DarkColorScheme());
    assert!(!native(&owner, "p").ColorSchemeForced());
    assert!(engine.Diagnostics().is_empty());
}
#[test]
fn invalid_stable_ui_grammar_and_unavailable_color_contexts_remain_typed() {
    for (id, text) in [
        (P::kAccentColor, "none"),
        (P::kAccentColor, "-internal-active-list-box-selection"),
        (P::kCaretColor, "red blue"),
        (P::kInternalVisitedCaretColor, "auto red"),
        (P::kColorScheme, "only"),
        (P::kColorScheme, "only only light"),
        (P::kColorScheme, "light only dark"),
        (P::kColorScheme, "normal dark"),
        (P::kColorScheme, "light normal"),
        (P::kColorScheme, "light initial"),
        (P::kColorScheme, "default"),
        (P::kColorScheme, "ident('dark')"),
    ] {
        assert_eq!(
            ParseProperty(id, &String::from(text), false, Mode::kHTMLStandardMode)
                .err()
                .unwrap()
                .kind,
            PropertyParseErrorKind::Invalid,
            "{text}"
        );
    }
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    for id in [
        P::kAccentColor,
        P::kCaretColor,
        P::kInternalVisitedCaretColor,
        P::kInternalVisitedOutlineColor,
        P::kInternalVisitedBorderTopColor,
    ] {
        assert_eq!(
            ParseProperty(
                id,
                &String::from("color(display-p3 1 0 0)"),
                false,
                Mode::kHTMLStandardMode
            )
            .err()
            .unwrap()
            .kind,
            PropertyParseErrorKind::Unsupported
        );
        let value = ParseProperty(
            id,
            &String::from("CanvasText"),
            false,
            Mode::kHTMLStandardMode,
        )
        .unwrap();
        let mut builder =
            layoutng_style::style::computed_style::ComputedStyleBuilder::from_style(initial);
        assert!(
            matches!(style::resolver::production_style_builder::Apply(id,&mut builder,None,value[0].Value(),16.0,&MediaValuesCachedData::default()),Err(style::properties::longhand_dispatch::LonghandApplicationError::Unsupported(p)) if p==id)
        );
    }
    let parsed = ParseProperty(
        P::kColorScheme,
        &String::from("only MyTheme LIGHT"),
        false,
        Mode::kHTMLStandardMode,
    )
    .unwrap();
    assert_eq!(parsed[0].Value().CssText().Utf8(), "MyTheme light only");
}
