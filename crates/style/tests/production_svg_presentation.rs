use foundation::{CSSPropertyID as P, Length, LineCap, LineJoin, String as CSSString};
use layoutng_style::style::{
    computed_style::{ComputedStyle, ComputedStyleBuilder},
    computed_style_constants::EBaselineShiftType,
};
use style::{
    css_value::CSSValuePayload,
    media_queries::MediaValuesCachedData,
    parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseProperty, PropertyParseErrorKind},
    },
    properties::longhand_dispatch::LonghandApplicationError,
    resolver::production_style_builder::Apply,
    StyleEngine,
};

fn parse(id: P, text: &str) -> std::rc::Rc<style::production_css_value::Value> {
    ParseProperty(
        id,
        &CSSString::from(text),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap()
    .remove(0)
    .ValueRef()
}
fn media() -> MediaValuesCachedData {
    MediaValuesCachedData {
        em_size: 16.,
        media_type: CSSString::from("screen"),
        viewport_width: 800.,
        viewport_height: 600.,
        ..Default::default()
    }
}
fn native<'a>(owner: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
    let doc = owner.GetDocument();
    let node = (0..doc.NodeCount())
        .find(|&i| {
            doc.Node(i)
                .FindAttribute("id")
                .is_some_and(|a| a.value == id)
        })
        .unwrap();
    unsafe { &*doc.ResolvedStyleFor(node).unwrap().native_style.Get() }
}

#[test]
fn svg_presentation_production_cascade_reaches_native_fields_and_css_wide() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<svg id=p><g id=c></g><g id=r></g><g id=b></g><g id=s></g><g id=bad></g></svg>",
    );
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("#p{baseline-shift:super;stroke-linecap:square;stroke-linejoin:bevel;stroke-miterlimit:calc(2 + 3);marker:none;d:none} #c{baseline-shift:inherit;marker:inherit;d:inherit} #r{baseline-shift:initial;stroke-linecap:initial;stroke-linejoin:initial;stroke-miterlimit:initial;marker:initial;d:initial} #b{baseline-shift:baseline;stroke-linecap:round;stroke-linejoin:round;stroke-miterlimit:0} #s{baseline-shift:calc(2px + 3%);stroke-linecap:unset;stroke-linejoin:unset;stroke-miterlimit:unset;marker:unset;d:unset} #bad{stroke-linecap:round;stroke-linecap:bevel;stroke-linejoin:miter;stroke-linejoin:square;stroke-miterlimit:7;stroke-miterlimit:-1;baseline-shift:-4;baseline-shift:none}"));
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let p = native(&owner, "p");
    assert_eq!(p.BaselineShiftType(), EBaselineShiftType::kSuper);
    assert_eq!(p.BaselineShift(), &Length::Fixed(0));
    assert_eq!(p.CapStyle(), LineCap::kSquareCap);
    assert_eq!(p.JoinStyle(), LineJoin::kBevelJoin);
    assert_eq!(p.StrokeMiterLimit(), 5.);
    for id in ["p", "c", "r", "s"] {
        let style = native(&owner, id);
        assert!(
            style.MarkerStartResource().is_null()
                && style.MarkerMidResource().is_null()
                && style.MarkerEndResource().is_null()
        );
        assert!(style.D().is_null());
    }
    let c = native(&owner, "c");
    assert_eq!(c.BaselineShiftType(), EBaselineShiftType::kSuper);
    assert_eq!(c.CapStyle(), p.CapStyle());
    assert_eq!(c.JoinStyle(), p.JoinStyle());
    assert_eq!(c.StrokeMiterLimit(), 5.);
    let r = native(&owner, "r");
    assert_eq!(r.BaselineShift(), &Length::Fixed(0));
    // Baseline shift is non-inherited; its native child type starts at length.
    assert_eq!(r.BaselineShiftType(), EBaselineShiftType::kLength);
    assert_eq!(r.CapStyle(), LineCap::kButtCap);
    assert_eq!(r.JoinStyle(), LineJoin::kMiterJoin);
    assert_eq!(r.StrokeMiterLimit(), 4.);
    assert_eq!(
        native(&owner, "b").BaselineShiftType(),
        EBaselineShiftType::kLength
    );
    assert_eq!(
        native(&owner, "s").BaselineShiftType(),
        EBaselineShiftType::kLength
    );
    assert!(native(&owner, "s").BaselineShift().IsCalculated());
    assert_eq!(native(&owner, "bad").BaselineShift(), &Length::Fixed(-4));
    assert_eq!(native(&owner, "bad").CapStyle(), LineCap::kRoundCap);
    assert_eq!(native(&owner, "bad").JoinStyle(), LineJoin::kMiterJoin);
    assert_eq!(native(&owner, "bad").StrokeMiterLimit(), 7.);
}

#[test]
fn text_stroke_shorthand_resets_omitted_slots_and_uses_native_font_length() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p><i id=c></i><i id=red></i><i id=w></i><i id=r></i><i id=bad></i></div>",
    );
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("#p{font-size:16px;-webkit-text-stroke:blue medium} #c{-webkit-text-stroke:inherit} #red{-webkit-text-stroke:red} #w{-webkit-text-stroke:calc(1px + 1px)} #r{-webkit-text-stroke:initial} #bad{-webkit-text-stroke:4px red;-webkit-text-stroke:1px 2px;-webkit-text-stroke:blue blue}"));
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let p = native(&owner, "p");
    assert_eq!(p.TextStrokeWidth(), 1.);
    assert_eq!(p.TextStrokeColor().GetColor().Param2(), 255.);
    let c = native(&owner, "c");
    assert_eq!(c.TextStrokeWidth(), p.TextStrokeWidth());
    assert!(c.TextStrokeColor() == p.TextStrokeColor());
    let red = native(&owner, "red");
    assert_eq!(red.TextStrokeWidth(), 0.);
    assert_eq!(red.TextStrokeColor().GetColor().Param0(), 255.);
    assert_eq!(native(&owner, "w").TextStrokeWidth(), 2.);
    assert!(native(&owner, "w").TextStrokeColor().IsCurrentColor());
    assert_eq!(native(&owner, "r").TextStrokeWidth(), 0.);
    assert!(native(&owner, "r").TextStrokeColor().IsCurrentColor());
    assert_eq!(native(&owner, "bad").TextStrokeWidth(), 4.);
}

#[test]
fn marker_urls_keep_typed_fragment_and_path_owner_is_explicit() {
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut builder = ComputedStyleBuilder::from_style(initial);
    let expansion = ParseProperty(
        P::kMarker,
        &CSSString::from("url(\"#marker\")"),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap();
    assert_eq!(expansion.len(), 3);
    for item in expansion {
        let value = item.ValueRef();
        let CSSValuePayload::kURIClass(uri) = value.Payload() else {
            panic!("typed URI required")
        };
        assert_eq!(uri.url.Utf8(), "#marker");
        assert_eq!(
            Apply(item.PropertyID(), &mut builder, None, &value, 16., &media()),
            Err(LonghandApplicationError::Unsupported(item.PropertyID()))
        );
    }
    assert!(builder.MarkerStartResource().is_null());
    assert_eq!(
        ParseProperty(
            P::kD,
            &CSSString::from("path(\"M0 0L1 1\")"),
            false,
            CSSParserMode::kHTMLStandardMode
        )
        .err()
        .unwrap()
        .kind,
        PropertyParseErrorKind::Unsupported
    );
    Apply(
        P::kD,
        &mut builder,
        None,
        &parse(P::kD, "none"),
        16.,
        &media(),
    )
    .unwrap();
    assert!(builder.D().is_null());
    assert_eq!(
        Apply(
            P::kBaselineShift,
            &mut builder,
            None,
            &parse(P::kBaselineShift, "sub"),
            16.,
            &media()
        ),
        Ok(())
    );
    assert_eq!(builder.BaselineShiftType(), EBaselineShiftType::kSub);
    Apply(
        P::kBaselineShift,
        &mut builder,
        None,
        &parse(P::kBaselineShift, "initial"),
        16.,
        &media(),
    )
    .unwrap();
    assert_eq!(builder.BaselineShiftType(), EBaselineShiftType::kSub);
}

#[test]
fn svg_presentation_invalid_values_are_rejected() {
    for (id, text) in [
        (P::kD, "url(#p)"),
        (P::kD, "path()"),
        (P::kD, "path(1)"),
        (P::kD, "path(evenodd, \"M0 0\")"),
        (P::kMarker, "none none"),
        (P::kMarkerStart, "red"),
        (P::kMarkerMid, "url(#p) red"),
        (P::kBaselineShift, "none"),
        (P::kBaselineShift, "sub 1px"),
        (P::kStrokeLinecap, "bevel"),
        (P::kStrokeLinejoin, "square"),
        (P::kStrokeMiterlimit, "-1"),
        (P::kStrokeMiterlimit, "1px"),
        (P::kWebkitTextStroke, "-1px red"),
        (P::kWebkitTextStroke, "1px 2px"),
        (P::kWebkitTextStrokeWidth, "2%"),
        (P::kWebkitTextStrokeWidth, "2"),
    ] {
        assert_eq!(
            ParseProperty(
                id,
                &CSSString::from(text),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .err()
            .unwrap()
            .kind,
            PropertyParseErrorKind::Invalid,
            "{id:?}: {text}"
        );
    }
}
