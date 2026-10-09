use foundation::{CSSPropertyID as P, CSSValueID as V, ETextAlign, String, TextDirection};
use layoutng_style::style::{
    computed_style::{ComputedStyle, ComputedStyleBuilder},
    computed_style_initial_values::ComputedStyleInitialValues,
};
use style::{
    media_queries::MediaValuesCachedData,
    parser::{
        css_parser_mode::CSSParserMode as Mode,
        production_property_parser::{ParseDeclarationList, ParseProperty},
    },
    production_css_value as values,
    properties::longhand_dispatch::LonghandApplicationError,
    resolver::production_style_builder::Apply,
    StyleEngine,
};

fn initial() -> &'static ComputedStyle {
    unsafe { &*ComputedStyle::GetInitialStyleSingleton() }
}
fn apply(b: &mut ComputedStyleBuilder, parent: Option<&ComputedStyle>, css: &str, mode: Mode) {
    let parsed = ParseDeclarationList(&String::from(css), mode);
    assert!(parsed.errors.is_empty(), "{css}: {:?}", parsed.errors);
    assert!(!parsed.properties.is_empty(), "{css}");
    for p in parsed.properties {
        Apply(
            p.PropertyID(),
            b,
            parent,
            p.Value(),
            16.,
            &MediaValuesCachedData::default(),
        )
        .unwrap_or_else(|e| panic!("{css}: {e:?}"));
    }
}
fn native<'a>(owner: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
    let d = owner.GetDocument();
    let node = (0..d.NodeCount())
        .find(|&n| d.Node(n).FindAttribute("id").is_some_and(|a| a.value == id))
        .unwrap();
    unsafe { &*d.ResolvedStyleFor(node).unwrap().native_style.Get() }
}

#[test]
fn text_align_match_parent_uses_parent_direction_and_root_computes_start() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<html id=root style='text-align:-webkit-match-parent'><body><div id=p style='direction:rtl;text-align:start'><div id=c style='direction:ltr;text-align:-webkit-match-parent'></div><div id=inherit style='text-align:inherit'></div></div><div style='direction:rtl;text-align:end'><div id=end style='text-align:-webkit-match-parent'></div></div></body></html>");
    let mut engine = StyleEngine::new(&owner);
    engine
        .Update(&mut owner, &MediaValuesCachedData::default(), &[])
        .unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    assert_eq!(native(&owner, "root").GetTextAlign(), ETextAlign::kStart);
    assert_eq!(native(&owner, "c").GetTextAlign(), ETextAlign::kRight);
    assert_eq!(native(&owner, "inherit").GetTextAlign(), ETextAlign::kStart);
    assert_eq!(native(&owner, "end").GetTextAlign(), ETextAlign::kLeft);
}

#[test]
fn text_align_keywords_and_ua_center_follow_stable_chromium_semantics() {
    let _heap = foundation::LayoutHeapScope::new();
    for (keyword, expected) in [
        ("left", ETextAlign::kLeft),
        ("right", ETextAlign::kRight),
        ("center", ETextAlign::kCenter),
        ("justify", ETextAlign::kJustify),
        ("start", ETextAlign::kStart),
        ("end", ETextAlign::kEnd),
        ("-webkit-auto", ETextAlign::kStart),
        ("-webkit-left", ETextAlign::kWebkitLeft),
        ("-webkit-right", ETextAlign::kWebkitRight),
        ("-webkit-center", ETextAlign::kWebkitCenter),
    ] {
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            None,
            &format!("text-align:{keyword}"),
            Mode::kHTMLStandardMode,
        );
        assert_eq!(b.GetTextAlign(), expected, "{keyword}");
    }
    for forbidden in ["match-parent", "-internal-center"] {
        assert!(ParseProperty(
            P::kTextAlign,
            &String::from(forbidden),
            false,
            Mode::kHTMLStandardMode
        )
        .is_err());
    }
    for parent_align in [
        ETextAlign::kStart,
        ETextAlign::kJustify,
        ETextAlign::kWebkitCenter,
    ] {
        let mut p = ComputedStyleBuilder::from_style(initial());
        p.SetTextAlign(parent_align);
        p.SetDirection(TextDirection::kRtl);
        let p = unsafe { &*p.TakeStyle() };
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(
            &mut b,
            Some(p),
            "text-align:-internal-center",
            Mode::kUASheetMode,
        );
        assert_eq!(
            b.GetTextAlign(),
            if parent_align == ETextAlign::kStart {
                ETextAlign::kCenter
            } else {
                parent_align
            }
        );
        apply(&mut b, Some(p), "text-align:unset", Mode::kHTMLStandardMode);
        assert_eq!(b.GetTextAlign(), parent_align);
        apply(
            &mut b,
            Some(p),
            "text-align:initial",
            Mode::kHTMLStandardMode,
        );
        assert_eq!(b.GetTextAlign(), ETextAlign::kStart);
    }
}

#[test]
fn shape_margin_inherits_computed_length_and_keeps_zoom_boundary() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut p = ComputedStyleBuilder::from_style(initial());
    p.SetEffectiveZoom(2.);
    apply(
        &mut p,
        None,
        "shape-margin:calc(3px + 4px)",
        Mode::kHTMLStandardMode,
    );
    let p = unsafe { &*p.TakeStyle() };
    assert_eq!(p.ShapeMargin().Pixels(), 14.);
    let mut b = ComputedStyleBuilder::from_style(initial());
    b.SetEffectiveZoom(2.);
    apply(
        &mut b,
        Some(p),
        "shape-margin:inherit",
        Mode::kHTMLStandardMode,
    );
    assert_eq!(b.ShapeMargin(), p.ShapeMargin());
    assert!(b.HasExplicitInheritance());
    assert!(p.ChildHasExplicitInheritance());
    apply(
        &mut b,
        Some(p),
        "shape-margin:unset",
        Mode::kHTMLStandardMode,
    );
    assert_eq!(b.ShapeMargin(), initial().ShapeMargin());
    b.SetEffectiveZoom(1.);
    assert_eq!(
        Apply(
            P::kShapeMargin,
            &mut b,
            Some(p),
            &values::wide(V::kInherit).unwrap(),
            16.,
            &MediaValuesCachedData::default()
        ),
        Err(LonghandApplicationError::Unsupported(P::kShapeMargin))
    );
}

#[test]
fn public_color_initials_reset_existing_fields_and_inherit_still_works() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut p = ComputedStyleBuilder::from_style(initial());
    apply(
        &mut p,
        None,
        "-webkit-tap-highlight-color:red;-webkit-text-fill-color:blue",
        Mode::kHTMLStandardMode,
    );
    let p = unsafe { &*p.TakeStyle() };
    let mut b = ComputedStyleBuilder::from_style(p);
    apply(
        &mut b,
        Some(p),
        "-webkit-tap-highlight-color:initial;-webkit-text-fill-color:initial",
        Mode::kHTMLStandardMode,
    );
    assert!(b.TapHighlightColor() == &ComputedStyleInitialValues::InitialTapHighlightColor());
    assert!(b.TextFillColor().IsCurrentColor());
    apply(
        &mut b,
        Some(p),
        "-webkit-tap-highlight-color:inherit;-webkit-text-fill-color:unset",
        Mode::kHTMLStandardMode,
    );
    assert!(b.TapHighlightColor() == p.TapHighlightColor());
    assert!(b.TextFillColor() == p.TextFillColor());
    apply(
        &mut b,
        None,
        "-webkit-tap-highlight-color:inherit;-webkit-text-fill-color:inherit",
        Mode::kHTMLStandardMode,
    );
    assert!(b.TapHighlightColor() == &ComputedStyleInitialValues::InitialTapHighlightColor());
    assert!(b.TextFillColor().IsCurrentColor());
}
