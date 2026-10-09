use foundation::{CSSPropertyID as P, CSSValueID, Length, String as CSSString};
use layoutng_style::style::{
    computed_style::ComputedStyle, computed_style_constants::EPaintOrder, svg_paint::SVGPaintType,
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
fn native<'a>(owner: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
    let doc = owner.GetDocument();
    let node = (0..doc.NodeCount())
        .find(|&i| {
            doc.Node(i)
                .FindAttribute("id")
                .is_some_and(|attribute| attribute.value == id)
        })
        .unwrap();
    unsafe { &*doc.ResolvedStyleFor(node).unwrap().native_style.Get() }
}
fn media() -> MediaValuesCachedData {
    MediaValuesCachedData {
        em_size: 16.,
        media_type: CSSString::from("screen"),
        viewport_width: 800.,
        viewport_height: 600.,
        large_viewport_width: 800.,
        large_viewport_height: 600.,
        ..Default::default()
    }
}

#[test]
fn svg_typed_grammar_handles_user_units_dash_separators_and_canonical_order() {
    for (id, value) in [
        (P::kCx, "-2"),
        (P::kCy, "10%"),
        (P::kR, "calc(2 + 3)"),
        (P::kRx, "auto"),
        (P::kRy, "calc(2px + 3%)"),
        (P::kX, "-3em"),
        (P::kY, "4"),
        (P::kStrokeWidth, "3"),
        (P::kStrokeDashoffset, "-2"),
        (P::kStrokeDasharray, "1, 2 3%, 4px"),
        (P::kPathLength, "3px"),
        (P::kFill, "currentColor"),
        (P::kStroke, "none"),
    ] {
        parse(id, value);
    }
    let order = parse(P::kPaintOrder, "stroke markers fill");
    assert_eq!(order.CssText().Utf8(), "stroke markers");
    assert_eq!(
        parse(P::kPaintOrder, "fill stroke markers")
            .CssText()
            .Utf8(),
        "fill"
    );
    assert!(
        matches!(parse(P::kStrokeWidth,"3").Payload(), CSSValuePayload::kNumericLiteralClass(n)
        if n.GetType() == style::css_primitive_value::UnitType::kUserUnits)
    );
    for (id, value) in [
        (P::kR, "-1"),
        (P::kRx, "-1%"),
        (P::kStrokeWidth, "-1"),
        (P::kStrokeDasharray, "1,"),
        (P::kStrokeDasharray, "1,-2"),
        (P::kPathLength, "2"),
        (P::kPathLength, "10%"),
        (P::kPaintOrder, "fill fill"),
        (P::kPaintOrder, "normal fill"),
        (P::kFill, "url(#paint) context-fill"),
    ] {
        assert_eq!(
            ParseProperty(
                id,
                &CSSString::from(value),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .err()
            .unwrap()
            .kind,
            PropertyParseErrorKind::Invalid,
            "{id:?}: {value}"
        );
    }
}

#[test]
fn production_svg_values_inheritance_and_initial_reach_native_fields() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<svg id=parent><g id=child></g><g id=reset></g><g id=invalid></g></svg>",
    );
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS(
        "#parent{color:blue;fill:currentColor;stroke:#123456;stroke-width:3;stroke-dashoffset:-2;stroke-dasharray:1,2 3%;cx:-4;cy:5%;r:6;rx:auto;ry:7px;x:8;y:-9px;path-length:10px;paint-order:stroke markers fill} #child{cx:inherit;cy:inherit;r:inherit;rx:inherit;ry:inherit;x:inherit;y:inherit;path-length:inherit} #reset{fill:initial;stroke:initial;stroke-width:initial;stroke-dasharray:initial;stroke-dashoffset:initial;paint-order:initial;cx:initial;cy:initial;r:initial;rx:initial;ry:initial;x:initial;y:initial;path-length:initial} #invalid{stroke-width:4;stroke-width:-1;r:5;r:-1;paint-order:markers stroke;paint-order:fill fill}"));
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let parent = native(&owner, "parent");
    assert!(parent.FillPaint().HasCurrentColor());
    assert_eq!(parent.StrokePaint().paint_type, SVGPaintType::kColor);
    assert_eq!(parent.StrokePaint().GetColor().GetColor().Param0(), 18.);
    assert_eq!(parent.StrokeWidth().length(), &Length::Fixed(3));
    assert_eq!(parent.StrokeDashOffset(), &Length::Fixed(-2));
    let dashes = unsafe { &*parent.StrokeDashArray() };
    assert_eq!(dashes.len(), 3);
    assert_eq!(dashes[0], Length::Fixed(1));
    assert_eq!(dashes[2], Length::Percent(3.));
    assert_eq!(parent.Cx(), &Length::Fixed(-4));
    assert_eq!(parent.Cy(), &Length::Percent(5.));
    assert_eq!(parent.R(), &Length::Fixed(6));
    assert!(parent.Rx().IsAuto());
    assert_eq!(parent.Ry(), &Length::Fixed(7));
    assert_eq!(parent.X(), &Length::Fixed(8));
    assert_eq!(parent.Y(), &Length::Fixed(-9));
    assert_eq!(parent.PathLength(), &Length::Fixed(10));
    assert_eq!(
        parent.PaintOrder(),
        EPaintOrder::kPaintOrderStrokeMarkersFill
    );
    let child = native(&owner, "child");
    assert!(child.FillPaint().HasCurrentColor());
    assert!(child.StrokePaint() == parent.StrokePaint());
    assert_eq!(child.StrokeWidth(), parent.StrokeWidth());
    assert_eq!(child.StrokeDashArray(), parent.StrokeDashArray());
    assert_eq!(child.StrokeDashOffset(), parent.StrokeDashOffset());
    assert_eq!(child.Cx(), parent.Cx());
    assert_eq!(child.Cy(), parent.Cy());
    assert_eq!(child.R(), parent.R());
    assert_eq!(child.Rx(), parent.Rx());
    assert_eq!(child.Ry(), parent.Ry());
    assert_eq!(child.X(), parent.X());
    assert_eq!(child.Y(), parent.Y());
    assert_eq!(child.PathLength(), parent.PathLength());
    assert_eq!(child.PaintOrder(), parent.PaintOrder());
    let reset = native(&owner, "reset");
    assert!(reset.FillPaint().IsInitial());
    assert!(reset.FillPaint().IsColor());
    assert!(reset.StrokePaint().IsInitial());
    assert!(reset.StrokePaint().IsNone());
    assert!(reset.StrokeDashArray().is_null());
    assert_eq!(reset.StrokeDashOffset(), &Length::Fixed(0));
    assert_eq!(reset.StrokeWidth().length(), &Length::Fixed(1));
    assert_eq!(reset.PaintOrder(), EPaintOrder::kPaintOrderNormal);
    assert_eq!(reset.Cx(), &Length::Fixed(0));
    assert_eq!(reset.Cy(), &Length::Fixed(0));
    assert_eq!(reset.R(), &Length::Fixed(0));
    assert!(reset.Rx().IsAuto());
    assert!(reset.Ry().IsAuto());
    assert_eq!(reset.X(), &Length::Fixed(0));
    assert_eq!(reset.Y(), &Length::Fixed(0));
    assert!(reset.PathLength().IsNone());
    let invalid = native(&owner, "invalid");
    assert_eq!(invalid.StrokeWidth().length(), &Length::Fixed(4));
    assert_eq!(invalid.R(), &Length::Fixed(5));
    assert_eq!(
        invalid.PaintOrder(),
        EPaintOrder::kPaintOrderMarkersStrokeFill
    );
}

#[test]
fn svg_url_paint_keeps_typed_fallback_and_requires_real_resource_binding() {
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut builder =
        layoutng_style::style::computed_style::ComputedStyleBuilder::from_style(initial);
    Apply(
        P::kFill,
        &mut builder,
        None,
        &parse(P::kFill, "red"),
        16.,
        &media(),
    )
    .unwrap();
    for text in [
        "url(#paint)",
        "url('#paint') none",
        "url(#paint) currentColor",
        "url(#paint) #123456",
    ] {
        let value = parse(P::kFill, text);
        let url = match value.Payload() {
            CSSValuePayload::kURIClass(url) => url,
            CSSValuePayload::kValueListClass(list) => {
                assert_eq!(list.values.len(), 2);
                match list.values[0].Payload() {
                    CSSValuePayload::kURIClass(url) => url,
                    _ => panic!(),
                }
            }
            _ => panic!(),
        };
        assert_eq!(url.url.Utf8(), "#paint");
        assert_eq!(
            Apply(P::kFill, &mut builder, None, &value, 16., &media()),
            Err(LonghandApplicationError::Unsupported(P::kFill))
        );
        assert!(builder.FillPaint().IsColor());
        assert!(builder.FillPaint().Resource().is_null());
    }
    Apply(
        P::kFill,
        &mut builder,
        None,
        &parse(P::kFill, "context-fill"),
        16.,
        &media(),
    )
    .unwrap();
    assert_eq!(builder.FillPaint().paint_type, SVGPaintType::kContextFill);
    Apply(
        P::kStroke,
        &mut builder,
        None,
        &parse(P::kStroke, "context-stroke"),
        16.,
        &media(),
    )
    .unwrap();
    assert_eq!(
        builder.StrokePaint().paint_type,
        SVGPaintType::kContextStroke
    );
    assert!(
        matches!(parse(P::kFill,"none").Payload(), CSSValuePayload::kIdentifierClass(k) if k.0 == CSSValueID::kNone)
    );
}

#[test]
fn svg_geometry_uses_zoomed_lengths_and_stroke_width_uses_unzoomed_lengths() {
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut builder =
        layoutng_style::style::computed_style::ComputedStyleBuilder::from_style(initial);
    builder.SetEffectiveZoom(2.);
    let mut font = builder.GetFontDescription().clone();
    font.SetSpecifiedSize(20.);
    font.SetComputedSize(40.);
    style::resolver::production_style_builder::StageFontDescription(&mut builder, &font);
    for (id, text) in [
        (P::kCx, "1em"),
        (P::kCy, "1rem"),
        (P::kR, "calc(2 + 3)"),
        (P::kRx, "calc(3px + 10%)"),
        (P::kStrokeWidth, "1em"),
        (P::kStrokeDasharray, "2,1em"),
        (P::kStrokeDashoffset, "-3"),
    ] {
        Apply(id, &mut builder, None, &parse(id, text), 16., &media()).unwrap();
    }
    let target = unsafe { &*builder.TakeStyle() };
    assert_eq!(target.Cx(), &Length::Fixed(40));
    assert_eq!(target.Cy(), &Length::Fixed(32));
    assert_eq!(target.R(), &Length::Fixed(10));
    assert!(target.Rx().IsCalculated());
    assert_eq!(
        target
            .Rx()
            .GetCalculationValue()
            .Evaluate(100., &foundation::EvaluationInput::default()),
        16.
    );
    assert_eq!(target.StrokeWidth().length(), &Length::Fixed(20));
    let dashes = unsafe { &*target.StrokeDashArray() };
    assert_eq!(dashes[0], Length::Fixed(4));
    assert_eq!(dashes[1], Length::Fixed(40));
    assert_eq!(target.StrokeDashOffset(), &Length::Fixed(-6));
}
