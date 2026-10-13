//! Source-aligned minmax grammar and native track values through production Style.
use foundation::CSSPropertyID as P;
use style::css_value::CSSValuePayload;
use style::parser::{
    css_parser_mode::CSSParserMode,
    production_property_parser::{ParseProperty, PropertyParseErrorKind},
};
use style::{media_queries::MediaValuesCachedData, StyleEngine};

#[test]
fn minmax_parser_accepts_breadths_and_retains_typed_function_arguments() {
    for property in [
        P::kGridTemplateColumns,
        P::kGridTemplateRows,
        P::kGridAutoColumns,
        P::kGridAutoRows,
    ] {
        for value in [
            "minmax(0,48rem)",
            "minmax(10%,1fr)",
            "minmax(auto,max-content)",
            "minmax(min-content,auto)",
            "minmax(max-content,min-content)",
            "minmax(calc(1px - 2px),max(20px,30%))",
            "minmax(100px,20px)",
            "minmax(0,0fr)",
        ] {
            let parsed = ParseProperty(
                property,
                &foundation::String::from(value),
                false,
                CSSParserMode::kHTMLStandardMode,
            )
            .unwrap_or_else(|e| panic!("{property:?} {value}: {e:?}"));
            let CSSValuePayload::kValueListClass(list) = parsed[0].Value().Payload() else {
                panic!("not a typed track list")
            };
            let CSSValuePayload::kFunctionClass(function) = list.values[0].Payload() else {
                panic!("not a typed function")
            };
            assert_eq!(function.function_id, foundation::CSSValueID::kMinmax);
            assert_eq!(function.arguments.values.len(), 2);
        }
    }
}

#[test]
fn minmax_parser_rejects_flex_minimum_and_nested_or_malformed_functions() {
    for value in [
        "minmax(1fr,20px)",
        "minmax(0fr,1fr)",
        "minmax(-1px,20px)",
        "minmax(0,-1fr)",
        "minmax(0,-5%)",
        "minmax(1,20px)",
        "minmax(0 20px)",
        "minmax(0,)",
        "minmax(,20px)",
        "minmax(0,20px,30px)",
        "minmax(0,minmax(10px,20px))",
        "minmax(fit-content(10px),20px)",
        "minmax(auto,none)",
    ] {
        let error = ParseProperty(
            P::kGridTemplateColumns,
            &foundation::String::from(value),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .err()
        .unwrap_or_else(|| panic!("accepted {value}"));
        assert_eq!(error.kind, PropertyParseErrorKind::Invalid, "{value}");
    }
}

#[test]
fn minmax_shorthands_share_the_track_consumer_without_admitting_repeat() {
    for (property, value) in [
        (P::kGridTemplate, "minmax(10px,1fr) / minmax(0,40px)"),
        (P::kGrid, "auto-flow minmax(10px,auto) / minmax(0,1fr)"),
        (P::kGrid, "minmax(0,1fr) / auto-flow minmax(0,20px)"),
    ] {
        ParseProperty(
            property,
            &foundation::String::from(value),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap_or_else(|e| panic!("{value}: {e:?}"));
    }
    for value in ["repeat(2,minmax(0,1fr))", "fit-content(20px)", "subgrid"] {
        let error = ParseProperty(
            P::kGridTemplateColumns,
            &foundation::String::from(value),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .err()
        .unwrap();
        assert_eq!(error.kind, PropertyParseErrorKind::Unsupported, "{value}");
    }
}

#[test]
fn minmax_native_values_preserve_math_flex_intrinsic_and_minimum_greater_than_maximum() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<div id=g style='font-size:16px;grid-template-columns:minmax(calc(1px - 2px),1fr) minmax(min-content,auto) minmax(100px,20px);grid-auto-rows:minmax(10%,2fr);grid-template-rows:minmax(10px,30px)'></div>");
    let mut engine = StyleEngine::new(&owner);
    engine
        .Update(&mut owner, &MediaValuesCachedData::default(), &[])
        .unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let doc = owner.GetDocument();
    let index = (0..doc.NodeCount())
        .find(|&i| {
            doc.Node(i)
                .FindAttribute("id")
                .is_some_and(|a| a.value == "g")
        })
        .unwrap();
    let native = unsafe { &*doc.ResolvedStyleFor(index).unwrap().native_style.Get() };
    let columns = unsafe { &*native.SpecifiedGridTemplateColumns().Get() }.GetTrackList();
    let first = columns.RepeatTrackSize(0, 0);
    assert_eq!(first.MinTrackBreadth().Pixels(), 0.0);
    assert_eq!(first.MaxTrackBreadth().FlexValue(), 1.0);
    let second = columns.RepeatTrackSize(1, 0);
    assert!(second.MinTrackBreadth().IsMinContent());
    assert!(second.MaxTrackBreadth().IsAuto());
    let third = columns.RepeatTrackSize(2, 0);
    assert_eq!(third.MinTrackBreadth().Pixels(), 100.0);
    assert_eq!(third.MaxTrackBreadth().Pixels(), 20.0);
    let auto = native.GridAutoRows().RepeatTrackSize(0, 0);
    assert_eq!(auto.MinTrackBreadth().PercentValue(), 10.0);
    assert_eq!(auto.MaxTrackBreadth().FlexValue(), 2.0);
    assert_eq!(
        unsafe { &*native.SpecifiedGridTemplateRows().Get() }
            .GetTrackList()
            .RepeatTrackSize(0, 0)
            .MaxTrackBreadth()
            .Pixels(),
        30.0
    );
}
