//! Omitted grid placement components retain source custom-ident semantics.
use foundation::{CSSPropertyID as P, String};
use style::parser::{css_parser_mode::CSSParserMode, production_property_parser::ParseProperty};
use style::{media_queries::MediaValuesCachedData, StyleEngine};

fn expanded(property: P, value: &str) -> Vec<std::string::String> {
    let parsed = ParseProperty(
        property,
        &String::from(value),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap();
    assert!(parsed
        .iter()
        .all(|v| v.ShorthandID() == property && !v.IsImplicit()));
    parsed.iter().map(|v| v.Value().CssText().Utf8()).collect()
}

#[test]
fn grid_area_omissions_copy_only_the_corresponding_custom_ident() {
    for (value, expected) in [
        ("zone", ["zone", "zone", "zone", "zone"]),
        ("zone / column", ["zone", "column", "zone", "column"]),
        ("zone / column / end", ["zone", "column", "end", "column"]),
        (
            "zone / column / end / far",
            ["zone", "column", "end", "far"],
        ),
        ("zone / auto", ["zone", "auto", "zone", "auto"]),
        ("auto / column", ["auto", "column", "auto", "column"]),
        ("auto / column / end", ["auto", "column", "end", "column"]),
        ("zone / 2", ["zone", "2", "zone", "auto"]),
        ("2 / column", ["2", "column", "auto", "column"]),
        ("zone / column / auto", ["zone", "column", "auto", "column"]),
        (
            "zone / auto / auto / auto",
            ["zone", "auto", "auto", "auto"],
        ),
        ("2", ["2", "auto", "auto", "auto"]),
        ("span zone", ["span zone", "auto", "auto", "auto"]),
        ("2 zone", ["2 zone", "auto", "auto", "auto"]),
        ("span 2 zone", ["span 2 zone", "auto", "auto", "auto"]),
        (
            "zone / span column",
            ["zone", "span column", "zone", "auto"],
        ),
        ("zone / 2 column", ["zone", "2 column", "zone", "auto"]),
        ("auto", ["auto", "auto", "auto", "auto"]),
        ("ZONE", ["ZONE", "ZONE", "ZONE", "ZONE"]),
    ] {
        assert_eq!(expanded(P::kGridArea, value), expected, "{value}");
    }
}

#[test]
fn grid_axis_omissions_copy_only_bare_names_and_preserve_explicit_ends() {
    for property in [P::kGridRow, P::kGridColumn] {
        for (value, expected) in [
            ("zone", ["zone", "zone"]),
            ("zone / end", ["zone", "end"]),
            ("zone / auto", ["zone", "auto"]),
            ("2", ["2", "auto"]),
            ("auto", ["auto", "auto"]),
            ("-1", ["-1", "auto"]),
            ("2 zone", ["2 zone", "auto"]),
            ("span zone", ["span zone", "auto"]),
            ("span 2 zone", ["span 2 zone", "auto"]),
            ("zone / span 2", ["zone", "span 2"]),
        ] {
            assert_eq!(expanded(property, value), expected, "{property:?}: {value}");
        }
    }
}

#[test]
fn omitted_grid_names_reach_canonical_native_positions_after_substitution() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner =
        html::html_parser::ParseHTML("<div id=g style='--area:zone;grid-area:var(--area)'></div>");
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
    for position in [
        native.GridRowStart(),
        native.GridColumnStart(),
        native.GridRowEnd(),
        native.GridColumnEnd(),
    ] {
        assert!(position.IsNamedGridArea());
        assert_eq!(position.NamedGridLine().Utf8(), "zone");
    }
}
