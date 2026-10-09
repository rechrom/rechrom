use foundation::{CSSPropertyID as P, CSSValueID as V, Color, String};
use layoutng_style::{
    css::style_color::StyleColor,
    style::computed_style::{ComputedStyle, ComputedStyleBuilder},
};
use style::{
    StyleEngine,
    media_queries::MediaValuesCachedData,
    parser::{css_parser_mode::CSSParserMode, production_property_parser::ParseProperty},
    production_css_value as values,
};
fn native<'a>(owner: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
    let d = owner.GetDocument();
    let n = (0..d.NodeCount())
        .find(|&n| d.Node(n).FindAttribute("id").is_some_and(|a| a.value == id))
        .unwrap();
    unsafe { &*d.ResolvedStyleFor(n).unwrap().native_style.Get() }
}
fn update(owner: &mut dom::DOM) {
    let mut e = StyleEngine::new(owner);
    e.Update(owner, &MediaValuesCachedData::default(), &[])
        .unwrap();
    assert!(e.Diagnostics().is_empty(), "{:?}", e.Diagnostics());
}
fn parse(id: P, text: &str) -> Vec<values::PropertyValue> {
    ParseProperty(
        id,
        &String::from(text),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap()
}
use layoutng_style::{
    css::css_reflection_direction::CSSReflectionDirection as D,
    style::nine_piece_image::ENinePieceImageRule as R,
};
fn initial() -> &'static ComputedStyle {
    unsafe { &*ComputedStyle::GetInitialStyleSingleton() }
}
fn apply(b: &mut ComputedStyleBuilder, parent: Option<&ComputedStyle>, id: P, text: &str) {
    let v = parse(id, text);
    style::resolver::production_style_builder::Apply(
        id,
        b,
        parent,
        v[0].Value(),
        16.0,
        &MediaValuesCachedData::default(),
    )
    .unwrap();
}
#[test]
fn internal_visited_svg_paint_keeps_slots_separate_and_inherits_parent_ordinary_paint() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut p = ComputedStyleBuilder::from_style(initial());
    apply(&mut p, None, P::kFill, "red");
    apply(&mut p, None, P::kStroke, "blue");
    apply(&mut p, None, P::kInternalVisitedFill, "green");
    apply(&mut p, None, P::kInternalVisitedStroke, "yellow");
    assert!(p.FillPaint().GetColor() == &StyleColor::from_color(Color::FromRGB(255, 0, 0)));
    assert!(
        p.InternalVisitedFillPaint().GetColor()
            == &StyleColor::from_color(Color::FromRGB(0, 128, 0))
    );
    let parent = unsafe { &*p.TakeStyle() };
    let mut b = ComputedStyleBuilder::from_style(initial());
    apply(&mut b, Some(parent), P::kInternalVisitedFill, "inherit");
    apply(&mut b, Some(parent), P::kInternalVisitedStroke, "inherit");
    assert!(b.InternalVisitedFillPaint() == parent.FillPaint());
    assert!(b.InternalVisitedStrokePaint() == parent.StrokePaint());
    assert!(b.FillPaint() == initial().FillPaint());
    assert!(b.StrokePaint() == initial().StrokePaint());
    apply(
        &mut b,
        Some(parent),
        P::kInternalVisitedFill,
        "currentcolor",
    );
    assert!(b.InternalVisitedFillPaint().HasCurrentColor());
    assert!(!b.FillPaint().HasCurrentColor());
    apply(&mut b, Some(parent), P::kInternalVisitedStroke, "none");
    assert!(b.InternalVisitedStrokePaint().IsNone());
    apply(&mut b, Some(parent), P::kInternalVisitedFill, "initial");
    assert!(b.InternalVisitedFillPaint() == initial().FillPaint());
    apply(&mut b, Some(parent), P::kInternalVisitedFill, "unset");
    assert!(b.InternalVisitedFillPaint() == parent.FillPaint());
    for id in [P::kInternalVisitedFill, P::kInternalVisitedStroke] {
        for wide in ["initial", "inherit", "unset", "revert", "revert-layer"] {
            assert_eq!(parse(id, wide).len(), 1);
        }
        let value = parse(id, "url(#real-paint-owner-required) currentcolor");
        assert!(
            matches!(value[0].Value().Payload(),style::css_value::CSSValuePayload::kValueListClass(l) if l.values[0].IsURIValue())
        );
        assert!(
            style::resolver::production_style_builder::Apply(
                id,
                &mut b,
                Some(parent),
                value[0].Value(),
                16.0,
                &MediaValuesCachedData::default()
            )
            .is_err()
        );
    }
}
#[test]
fn reflection_direction_offset_mask_and_css_wide_reach_native_reflection() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p style='-webkit-box-reflect:below 8px linear-gradient(to bottom,red,blue) 20 fill / 2 / 3px round space'><div id=c style='-webkit-box-reflect:inherit'></div><div id=i style='-webkit-box-reflect:initial'></div><div id=u style='-webkit-box-reflect:unset'></div></div><div id=a style='-webkit-box-reflect:above'></div><div id=l style='-webkit-box-reflect:left -2px'></div><div id=r style='-webkit-box-reflect:right 15%'></div><div id=m style='border-top:7px solid;-webkit-box-reflect:above 0 none 20 fill / 40px'></div>",
    );
    update(&mut owner);
    let p = native(&owner, "p");
    let reflection = unsafe { &*p.BoxReflect() };
    assert_eq!(reflection.Direction(), D::kReflectionBelow);
    assert_eq!(reflection.Offset().Pixels(), 8.0);
    assert!(reflection.Mask().HasImage());
    assert!(unsafe { &*reflection.Mask().GetImage() }.IsGeneratedImage());
    assert!(reflection.Mask().Fill());
    assert_eq!(reflection.Mask().ImageSlices().Top().Pixels(), 20.0);
    assert_eq!(reflection.Mask().BorderSlices().Top().Number(), 2.0);
    assert_eq!(reflection.Mask().Outset().Top().length().Pixels(), 3.0);
    assert!(reflection.Mask().HorizontalRule() == R::kRoundImageRule);
    assert!(reflection.Mask().VerticalRule() == R::kSpaceImageRule);
    assert_eq!(native(&owner, "c").BoxReflect(), p.BoxReflect());
    assert!(native(&owner, "i").BoxReflect().is_null());
    assert!(native(&owner, "u").BoxReflect().is_null());
    for (id, direction) in [
        ("a", D::kReflectionAbove),
        ("l", D::kReflectionLeft),
        ("r", D::kReflectionRight),
    ] {
        assert_eq!(
            unsafe { &*native(&owner, id).BoxReflect() }.Direction(),
            direction
        );
    }
    assert_eq!(
        unsafe { &*native(&owner, "a").BoxReflect() }
            .Offset()
            .Pixels(),
        0.0
    );
    assert_eq!(
        unsafe { &*native(&owner, "l").BoxReflect() }
            .Offset()
            .Pixels(),
        -2.0
    );
    assert_eq!(
        unsafe { &*native(&owner, "r").BoxReflect() }
            .Offset()
            .PercentValue(),
        15.0
    );
    assert_eq!(native(&owner, "m").BorderTopWidth(), 7);
    assert_eq!(
        unsafe { &*native(&owner, "m").BoxReflect() }
            .Mask()
            .BorderSlices()
            .Top()
            .length()
            .Pixels(),
        40.0
    );
    assert_eq!(
        parse(P::kWebkitBoxReflect, "above")[0]
            .Value()
            .CssText()
            .Utf8(),
        "above 0px"
    );
}
#[test]
fn reflection_source_invalid_none_converter_branch_url_binding_and_zoom_boundary() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut b = ComputedStyleBuilder::from_style(initial());
    apply(&mut b, None, P::kWebkitBoxReflect, "below calc(3px + 2px)");
    assert_eq!(unsafe { &*b.BoxReflect() }.Offset().Pixels(), 5.0);
    let ptr = b.BoxReflect();
    let url = parse(
        P::kWebkitBoxReflect,
        "above 10px url(real-fetched-adapter-required.png) 20 / 2",
    );
    assert!(
        matches!(url[0].Value().Payload(),style::css_value::CSSValuePayload::kReflectClass(r) if r.mask.is_some())
    );
    assert!(
        style::resolver::production_style_builder::Apply(
            P::kWebkitBoxReflect,
            &mut b,
            None,
            url[0].Value(),
            16.0,
            &MediaValuesCachedData::default()
        )
        .is_err()
    );
    assert_eq!(b.BoxReflect(), ptr);
    style::resolver::production_style_builder::Apply(
        P::kWebkitBoxReflect,
        &mut b,
        None,
        &values::identifier(V::kNone),
        16.0,
        &MediaValuesCachedData::default(),
    )
    .unwrap();
    assert!(b.BoxReflect().is_null());
    for text in [
        "none",
        "below none",
        "below 1",
        "above 2px invalid-mask",
        "above 2px 3 / -2",
        "center 0",
    ] {
        assert!(
            ParseProperty(
                P::kWebkitBoxReflect,
                &String::from(text),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .is_err(),
            "{text}"
        );
    }
    for wide in ["initial", "inherit", "unset", "revert", "revert-layer"] {
        assert_eq!(parse(P::kWebkitBoxReflect, wide).len(), 1);
    }
    b.SetEffectiveZoom(2.0);
    apply(&mut b, None, P::kWebkitBoxReflect, "left 3px");
    assert_eq!(unsafe { &*b.BoxReflect() }.Offset().Pixels(), 6.0);
    let p = unsafe { &*b.TakeStyle() };
    let mut child = ComputedStyleBuilder::from_style(initial());
    assert!(
        style::resolver::production_style_builder::Apply(
            P::kWebkitBoxReflect,
            &mut child,
            Some(p),
            &values::wide(V::kInherit).unwrap(),
            16.0,
            &MediaValuesCachedData::default()
        )
        .is_err()
    );
}
