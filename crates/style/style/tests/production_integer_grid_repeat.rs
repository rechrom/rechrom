//! Integer grid repetition retains typed counts and compact canonical tracks.
use foundation::CSSPropertyID as P;
use style::css_value::CSSValuePayload;
use style::parser::{
    css_parser_mode::CSSParserMode,
    production_property_parser::{ParseProperty, PropertyParseErrorKind},
};
use style::{media_queries::MediaValuesCachedData, StyleEngine};

#[test]
fn repeat_parser_retains_typed_count_names_and_track_sizes() {
    for (property, value) in [
        (
            P::kGridTemplateColumns,
            "repeat(3,[a] minmax(0,1fr) [b] fit-content(20px))",
        ),
        (P::kGridTemplateRows, "repeat(calc(1 + 2),20px 1fr)"),
        (P::kGridTemplate, "repeat(2,1fr) / repeat(3,20px)"),
        (P::kGrid, "repeat(2,1fr) / repeat(3,20px)"),
    ] {
        let parsed = ParseProperty(
            property,
            &foundation::String::from(value),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap_or_else(|e| panic!("{value}: {e:?}"));
        assert!(!parsed.is_empty());
        let has_repeat = parsed.iter().any(|p| matches!(p.Value().Payload(),CSSValuePayload::kValueListClass(list) if list.values.iter().any(|v|matches!(v.Payload(),CSSValuePayload::kGridIntegerRepeatClass(_)))));
        assert!(has_repeat, "{value}");
    }
    let parsed = ParseProperty(
        P::kGridTemplateColumns,
        &foundation::String::from("repeat(3, [a] 10px [b])"),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap();
    assert_eq!(
        parsed[0].Value().CssText().Utf8(),
        "repeat(3, [a] 10px [b])"
    );
}

#[test]
fn repeat_parser_rejects_bad_counts_and_nested_repeat() {
    for value in [
        "repeat(0,1fr)",
        "repeat(-1,1fr)",
        "repeat(1.5,1fr)",
        "repeat(2px,1fr)",
        "repeat(2)",
        "repeat(2,)",
        "repeat(2,[a])",
        "repeat(2,repeat(2,1fr))",
        "repeat(2,10px,20px)",
        "repeat(2,[a][b] 10px)",
        "repeat(2,minmax(1fr,2fr))",
        "repeat(2,[auto] 10px)",
        "repeat(2,[span] 10px)",
    ] {
        let e = ParseProperty(
            P::kGridTemplateColumns,
            &foundation::String::from(value),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .err()
        .unwrap_or_else(|| panic!("accepted {value}"));
        assert_eq!(e.kind, PropertyParseErrorKind::Invalid, "{value}");
    }
    for p in [P::kGridAutoColumns, P::kGridAutoRows] {
        assert_eq!(
            ParseProperty(
                p,
                &foundation::String::from("repeat(2,10px)"),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .err()
            .unwrap()
            .kind,
            PropertyParseErrorKind::Invalid
        );
    }
    for (value, kind) in [
        ("repeat(auto-fill,20px)", foundation::CSSValueID::kAutoFill),
        ("repeat(auto-fit,20px)", foundation::CSSValueID::kAutoFit),
    ] {
        let parsed = ParseProperty(
            P::kGridTemplateColumns,
            &foundation::String::from(value),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap_or_else(|error| panic!("{value}: {error:?}"));
        let CSSValuePayload::kValueListClass(list) = parsed[0].Value().Payload() else {
            panic!("not a track list: {value}")
        };
        let CSSValuePayload::kGridAutoRepeatClass(repeat) = list.values[0].Payload() else {
            panic!("not an auto repeat: {value}")
        };
        assert_eq!(repeat.kind, kind);
    }
}

#[test]
fn repeat_count_clamps_to_the_native_track_limit_without_expanding_sizes() {
    let parsed = ParseProperty(
        P::kGridTemplateColumns,
        &foundation::String::from("repeat(999999999,1fr 20px 30px)"),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap();
    let CSSValuePayload::kValueListClass(list) = parsed[0].Value().Payload() else {
        panic!()
    };
    let CSSValuePayload::kGridIntegerRepeatClass(repeat) = list.values[0].Payload() else {
        panic!()
    };
    assert_eq!(repeat.extra_clamp, 3_333_333);
    assert_eq!(repeat.ClampRepetitions(u32::MAX), 3_333_333);
    assert_eq!(repeat.list.values.len(), 3);
}

#[test]
fn repeat_native_named_lines_and_ordered_flags_use_expanded_indices() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<div id=g style='grid-template-columns:[outer] 5px repeat(2,[a] minmax(0,1fr) [b] 10px [end]) [tail] 7px'></div>");
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
    let i = (0..doc.NodeCount())
        .find(|&i| {
            doc.Node(i)
                .FindAttribute("id")
                .is_some_and(|a| a.value == "g")
        })
        .unwrap();
    let native = unsafe { &*doc.ResolvedStyleFor(i).unwrap().native_style.Get() };
    let columns = unsafe { &*native.SpecifiedGridTemplateColumns().Get() };
    let tracks = columns.GetTrackList();
    assert_eq!(tracks.TrackCountWithoutAutoRepeat(), 6);
    assert_eq!(tracks.RepeaterCount(), 3);
    assert_eq!(tracks.RepeatCount(1, 0), 2);
    assert_eq!(tracks.RepeatSize(1), 2);
    assert_eq!(
        columns.GetNamedGridLines()[&foundation::String::from("a")],
        [1, 3]
    );
    assert_eq!(
        columns.GetNamedGridLines()[&foundation::String::from("end")],
        [3, 5]
    );
    assert_eq!(
        columns.GetNamedGridLines()[&foundation::String::from("tail")],
        [5]
    );
    let ordered = columns.GetOrderedNamedGridLines();
    assert!(ordered[&1][0].is_in_repeat && ordered[&1][0].is_first_repeat);
    assert!(ordered[&3][1].is_in_repeat && !ordered[&3][1].is_first_repeat);
    assert!(!ordered[&5][1].is_in_repeat);
}
