use foundation::{CSSPropertyID as P, CSSValueID as V, EResize, String, UnicodeBidi, WindRule};
use layoutng_style::style::{
    computed_style::{ComputedStyle, ComputedStyleBuilder},
    grid_area::K_GRID_MAX_TRACKS,
    scroll_enums::mojom::blink::ScrollBehavior,
    style_overflow_clip_margin::ReferenceBox,
};
use style::{
    media_queries::MediaValuesCachedData,
    parser::{
        css_parser_mode::CSSParserMode as Mode, production_property_parser::ParseDeclarationList,
    },
    production_css_value as values,
    resolver::production_style_builder::Apply,
    StyleEngine,
};
const BOX_PROPERTIES: [P; 18] = [
    P::kWidth,
    P::kHeight,
    P::kMinWidth,
    P::kMinHeight,
    P::kMaxWidth,
    P::kMaxHeight,
    P::kMarginTop,
    P::kMarginRight,
    P::kMarginBottom,
    P::kMarginLeft,
    P::kPaddingTop,
    P::kPaddingRight,
    P::kPaddingBottom,
    P::kPaddingLeft,
    P::kTop,
    P::kRight,
    P::kBottom,
    P::kLeft,
];
fn initial() -> &'static ComputedStyle {
    unsafe { &*ComputedStyle::GetInitialStyleSingleton() }
}
fn apply(b: &mut ComputedStyleBuilder, p: Option<&ComputedStyle>, css: &str) {
    let parsed = ParseDeclarationList(&String::from(css), Mode::kHTMLStandardMode);
    assert!(parsed.errors.is_empty(), "{css}: {:?}", parsed.errors);
    assert!(!parsed.properties.is_empty(), "{css}");
    for value in parsed.properties {
        Apply(
            value.PropertyID(),
            b,
            p,
            value.Value(),
            16.,
            &MediaValuesCachedData::default(),
        )
        .unwrap_or_else(|e| panic!("{css}: {e:?}"));
    }
}
fn lengths(s: &ComputedStyle) -> Vec<&foundation::Length> {
    vec![
        s.Width(),
        s.Height(),
        s.MinWidth(),
        s.MinHeight(),
        s.MaxWidth(),
        s.MaxHeight(),
        s.MarginTop(),
        s.MarginRight(),
        s.MarginBottom(),
        s.MarginLeft(),
        s.PaddingTop(),
        s.PaddingRight(),
        s.PaddingBottom(),
        s.PaddingLeft(),
        s.Top(),
        s.Right(),
        s.Bottom(),
        s.Left(),
    ]
}
fn native<'a>(owner: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
    let d = owner.GetDocument();
    let n = (0..d.NodeCount())
        .find(|&n| d.Node(n).FindAttribute("id").is_some_and(|a| a.value == id))
        .unwrap();
    unsafe { &*d.ResolvedStyleFor(n).unwrap().native_style.Get() }
}
#[test]
fn physical_box_lengths_inherit_computed_units_and_reset_all_eighteen_fields() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut b = ComputedStyleBuilder::from_style(initial());
    b.SetEffectiveZoom(2.);
    apply(&mut b, None, "width:11px;height:12%;min-width:13px;min-height:14%;max-width:15px;max-height:none;margin:1px 2% auto -4px;padding:5px 6% 7px 8px;top:9px;right:auto;bottom:-10px;left:calc(11px + 12%)");
    let parent = unsafe { &*b.TakeStyle() };
    assert_eq!(parent.Width().Pixels(), 22.);
    assert_eq!(parent.Height().PercentValue(), 12.);
    assert_eq!(parent.MarginLeft().Pixels(), -8.);
    assert!(parent.MarginBottom().IsAuto());
    assert!(parent.Left().IsCalculated());
    for wide in [V::kInherit, V::kInitial, V::kUnset] {
        let mut child = ComputedStyleBuilder::from_style(initial());
        child.SetEffectiveZoom(2.);
        for id in BOX_PROPERTIES {
            Apply(
                id,
                &mut child,
                Some(parent),
                &values::wide(wide).unwrap(),
                16.,
                &MediaValuesCachedData::default(),
            )
            .unwrap();
        }
        let result = unsafe { &*child.TakeStyle() };
        let expected = if wide == V::kInherit {
            parent
        } else {
            initial()
        };
        assert_eq!(lengths(result), lengths(expected));
    }
    let mut child = ComputedStyleBuilder::from_style(initial());
    for id in BOX_PROPERTIES {
        assert!(Apply(
            id,
            &mut child,
            Some(parent),
            &values::wide(V::kInherit).unwrap(),
            16.,
            &MediaValuesCachedData::default()
        )
        .is_err());
    }
}

#[test]
fn gaps_clip_margin_grid_positions_and_simple_enums_use_native_storage() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut b = ComputedStyleBuilder::from_style(initial());
    apply(&mut b, None, "column-gap:calc(3px + 4%);row-gap:7%;overflow-clip-margin:content-box -2.25px;grid-column:calc(0) / span calc(0);grid-row:-999999999 / span 999999999;scroll-behavior:smooth;resize:inline;unicode-bidi:isolate-override");
    let p = unsafe { &*b.TakeStyle() };
    assert!(p.ColumnGap().as_ref().unwrap().IsCalculated());
    assert_eq!(p.RowGap().as_ref().unwrap().PercentValue(), 7.);
    let margin = p.OverflowClipMargin().unwrap();
    assert_eq!(margin.GetReferenceBox(), ReferenceBox::kContentBox);
    assert_eq!(margin.GetMargin().ToDouble(), -2.25);
    assert_eq!(p.GridColumnStart().IntegerPosition(), 1);
    assert_eq!(p.GridColumnEnd().SpanPosition(), 1);
    assert_eq!(p.GridRowStart().IntegerPosition(), -K_GRID_MAX_TRACKS);
    assert_eq!(p.GridRowEnd().SpanPosition(), K_GRID_MAX_TRACKS);
    assert_eq!(p.GetScrollBehavior(), ScrollBehavior::kSmooth);
    assert_eq!(p.Resize(), EResize::kInline);
    assert_eq!(p.GetUnicodeBidi(), UnicodeBidi::kIsolateOverride);
    let ids = [
        P::kColumnGap,
        P::kRowGap,
        P::kOverflowClipMargin,
        P::kGridColumnStart,
        P::kGridColumnEnd,
        P::kGridRowStart,
        P::kGridRowEnd,
        P::kScrollBehavior,
        P::kResize,
        P::kUnicodeBidi,
    ];
    let mut b = ComputedStyleBuilder::from_style(initial());
    for id in ids {
        Apply(
            id,
            &mut b,
            Some(p),
            &values::wide(V::kInherit).unwrap(),
            16.,
            &MediaValuesCachedData::default(),
        )
        .unwrap();
    }
    let c = unsafe { &*b.TakeStyle() };
    assert_eq!(c.ColumnGap(), p.ColumnGap());
    assert_eq!(c.RowGap(), p.RowGap());
    assert_eq!(c.OverflowClipMargin(), p.OverflowClipMargin());
    assert!(c.GridColumnStart() == p.GridColumnStart());
    assert!(c.GridColumnEnd() == p.GridColumnEnd());
    assert!(c.GridRowStart() == p.GridRowStart());
    assert!(c.GridRowEnd() == p.GridRowEnd());
    assert_eq!(c.GetScrollBehavior(), p.GetScrollBehavior());
    assert_eq!(c.Resize(), p.Resize());
    assert_eq!(c.GetUnicodeBidi(), p.GetUnicodeBidi());
    let mut b = ComputedStyleBuilder::from_style(c);
    for id in ids {
        Apply(
            id,
            &mut b,
            Some(p),
            &values::wide(V::kUnset).unwrap(),
            16.,
            &MediaValuesCachedData::default(),
        )
        .unwrap();
    }
    assert!(b.ColumnGap().is_none());
    assert!(b.RowGap().is_none());
    assert!(b.OverflowClipMargin().is_none());
    assert!(b.GridColumnStart().IsAuto());
    assert_eq!(b.GetScrollBehavior(), ScrollBehavior::kAuto);
    assert_eq!(b.Resize(), EResize::kNone);
    assert_eq!(b.GetUnicodeBidi(), UnicodeBidi::kNormal);
    apply(
        &mut b,
        None,
        "overflow-clip-margin:padding-box;resize:block",
    );
    assert_eq!(
        b.OverflowClipMargin().unwrap().GetReferenceBox(),
        ReferenceBox::kPaddingBox
    );
    assert_eq!(b.Resize(), EResize::kBlock);
}

#[test]
fn svg_alpha_math_percent_clamping_and_winding_rules_survive_inheritance() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut b = ComputedStyleBuilder::from_style(initial());
    apply(&mut b, None, "fill-opacity:calc(25% + 25%);flood-opacity:-2;stop-opacity:250%;stroke-opacity:calc(.2 + .1);fill-rule:evenodd;clip-rule:evenodd");
    let p = unsafe { &*b.TakeStyle() };
    assert_eq!(p.FillOpacity(), 0.5);
    assert_eq!(p.FloodOpacity(), 0.);
    assert_eq!(p.StopOpacity(), 1.);
    assert!((p.StrokeOpacity() - 0.3).abs() < 1e-6);
    assert_eq!(p.FillRule(), WindRule::RULE_EVENODD);
    assert_eq!(p.ClipRule(), WindRule::RULE_EVENODD);
    let mut b = ComputedStyleBuilder::from_style(initial());
    // These scalar/enum fields are independent of the length zoom policy.
    b.SetEffectiveZoom(2.);
    apply(&mut b, Some(p), "fill-opacity:inherit;flood-opacity:inherit;stop-opacity:inherit;stroke-opacity:inherit;fill-rule:inherit;clip-rule:inherit");
    assert_eq!(b.FillOpacity(), 0.5);
    assert_eq!(b.FloodOpacity(), 0.);
    assert_eq!(b.StopOpacity(), 1.);
    assert!((b.StrokeOpacity() - 0.3).abs() < 1e-6);
    assert_eq!(b.FillRule(), WindRule::RULE_EVENODD);
    assert_eq!(b.ClipRule(), WindRule::RULE_EVENODD);
    apply(&mut b, Some(p), "fill-opacity:initial;flood-opacity:initial;stop-opacity:initial;stroke-opacity:initial;fill-rule:initial;clip-rule:initial");
    assert_eq!(b.FillOpacity(), 1.);
    assert_eq!(b.FloodOpacity(), 1.);
    assert_eq!(b.StopOpacity(), 1.);
    assert_eq!(b.StrokeOpacity(), 1.);
    assert_eq!(b.FillRule(), WindRule::RULE_NONZERO);
    assert_eq!(b.ClipRule(), WindRule::RULE_NONZERO);
}

#[test]
fn style_engine_production_logical_mapping_and_css_wide_cascade_reach_real_fields() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=p style='width:21px;height:22px;min-width:23px;max-height:24px;margin:1px 2px 3px 4px;padding:5px 6px 7px 8px;top:9px;column-gap:11px;row-gap:12%;grid-column:2 / span 3;grid-row:-4 / 5;fill-opacity:40%;fill-rule:evenodd;scroll-behavior:smooth;resize:both;unicode-bidi:embed'><div id=c style='direction:rtl;inline-size:inherit;block-size:inherit;min-inline-size:inherit;max-block-size:inherit;margin-inline-start:inherit;padding-inline-start:inherit;inset-block-start:inherit;gap:inherit;grid-column:inherit;grid-row:inherit;fill-opacity:inherit;fill-rule:inherit;scroll-behavior:inherit;resize:inherit;unicode-bidi:inherit'></div><div id=u style='width:unset;gap:unset;grid-column:unset;scroll-behavior:unset;fill-opacity:unset;fill-rule:unset'></div></div>");
    let mut engine = StyleEngine::new(&owner);
    engine
        .Update(&mut owner, &MediaValuesCachedData::default(), &[])
        .unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let c = native(&owner, "c");
    assert_eq!(c.Width().Pixels(), 21.);
    assert_eq!(c.Height().Pixels(), 22.);
    assert_eq!(c.MinWidth().Pixels(), 23.);
    assert_eq!(c.MaxHeight().Pixels(), 24.);
    assert_eq!(c.MarginRight().Pixels(), 2.);
    assert_eq!(c.PaddingRight().Pixels(), 6.);
    assert_eq!(c.Top().Pixels(), 9.);
    assert_eq!(c.ColumnGap().as_ref().unwrap().Pixels(), 11.);
    assert_eq!(c.RowGap().as_ref().unwrap().PercentValue(), 12.);
    assert_eq!(c.GridColumnStart().IntegerPosition(), 2);
    assert_eq!(c.GridColumnEnd().SpanPosition(), 3);
    assert_eq!(c.GridRowStart().IntegerPosition(), -4);
    assert_eq!(c.GridRowEnd().IntegerPosition(), 5);
    assert_eq!(c.FillOpacity(), 0.4);
    assert_eq!(c.FillRule(), WindRule::RULE_EVENODD);
    assert_eq!(c.GetScrollBehavior(), ScrollBehavior::kSmooth);
    assert_eq!(c.Resize(), EResize::kBoth);
    assert_eq!(c.GetUnicodeBidi(), UnicodeBidi::kEmbed);
    let u = native(&owner, "u");
    assert!(u.Width().IsAuto());
    assert!(u.ColumnGap().is_none());
    assert!(u.GridColumnStart().IsAuto());
    assert_eq!(u.GetScrollBehavior(), ScrollBehavior::kAuto);
    assert_eq!(u.FillOpacity(), 0.4);
    assert_eq!(u.FillRule(), WindRule::RULE_EVENODD);
}
