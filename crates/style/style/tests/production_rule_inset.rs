use foundation::{CSSPropertyID as P, CSSValueID as V, Length, String};
use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};
use style::{
    media_queries::MediaValuesCachedData,
    parser::{
        css_parser_mode::CSSParserMode as Mode,
        production_property_parser::{
            ParseDeclarationList, ParseProperty, ParsePropertyTokens, PropertyParseErrorKind,
        },
    },
    production_css_value as values,
    resolver::production_style_builder::Apply,
    StyleEngine,
};

const COLUMN: [P; 4] = [
    P::kColumnRuleInsetCapStart,
    P::kColumnRuleInsetCapEnd,
    P::kColumnRuleInsetJunctionStart,
    P::kColumnRuleInsetJunctionEnd,
];
const ROW: [P; 4] = [
    P::kRowRuleInsetCapStart,
    P::kRowRuleInsetCapEnd,
    P::kRowRuleInsetJunctionStart,
    P::kRowRuleInsetJunctionEnd,
];
const SHORTHANDS: [P; 15] = [
    P::kColumnRuleInset,
    P::kRowRuleInset,
    P::kRuleInset,
    P::kColumnRuleInsetCap,
    P::kRowRuleInsetCap,
    P::kRuleInsetCap,
    P::kColumnRuleInsetJunction,
    P::kRowRuleInsetJunction,
    P::kRuleInsetJunction,
    P::kColumnRuleInsetStart,
    P::kRowRuleInsetStart,
    P::kRuleInsetStart,
    P::kColumnRuleInsetEnd,
    P::kRowRuleInsetEnd,
    P::kRuleInsetEnd,
];
fn parse(id: P, text: &str) -> Vec<values::PropertyValue> {
    ParseProperty(id, &String::from(text), false, Mode::kHTMLStandardMode).unwrap()
}
// Chromium css_property_value.h:94 retains a two-bit shorthand index.
// The generated inset candidate order has six entries; index 4 and 5
// truncate to RuleInset and the directional Inset respectively. JunctionEnd
// places directional End before Junction, unlike the other three longhands.
fn stored_identity(field: P, requested: P) -> P {
    match (field, requested) {
        (P::kColumnRuleInsetJunctionEnd, P::kColumnRuleInsetJunction) => P::kColumnRuleInset,
        (P::kRowRuleInsetJunctionEnd, P::kRowRuleInsetJunction) => P::kRowRuleInset,
        (P::kColumnRuleInsetJunctionEnd, P::kColumnRuleInsetEnd)
        | (P::kRowRuleInsetJunctionEnd, P::kRowRuleInsetEnd) => P::kRuleInset,
        (
            _,
            P::kColumnRuleInsetCap
            | P::kRowRuleInsetCap
            | P::kColumnRuleInsetJunction
            | P::kRowRuleInsetJunction,
        ) => P::kRuleInset,
        (_, P::kColumnRuleInsetStart | P::kColumnRuleInsetEnd) => P::kColumnRuleInset,
        (_, P::kRowRuleInsetStart | P::kRowRuleInsetEnd) => P::kRowRuleInset,
        _ => requested,
    }
}
fn insets(s: &ComputedStyle) -> [&Length; 8] {
    [
        s.ColumnRuleInsetCapStart(),
        s.ColumnRuleInsetCapEnd(),
        s.ColumnRuleInsetJunctionStart(),
        s.ColumnRuleInsetJunctionEnd(),
        s.RowRuleInsetCapStart(),
        s.RowRuleInsetCapEnd(),
        s.RowRuleInsetJunctionStart(),
        s.RowRuleInsetJunctionEnd(),
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
fn all_fifteen_shorthands_match_chromium_order_defaults_and_identity() {
    let _heap = foundation::LayoutHeapScope::new();
    for (id, expected, identity) in [
        (
            P::kColumnRuleInset,
            COLUMN.to_vec(),
            vec![P::kColumnRuleInset; 4],
        ),
        (P::kRowRuleInset, ROW.to_vec(), vec![P::kRowRuleInset; 4]),
        (
            P::kRuleInset,
            [COLUMN, ROW].concat(),
            vec![P::kRuleInset; 8],
        ),
    ] {
        for (text, texts) in [
            ("1px", ["1px", "1px", "1px", "1px"]),
            ("1px 2%", ["1px", "2%", "1px", "2%"]),
            (
                "1px / overlap-join",
                ["1px", "1px", "overlap-join", "overlap-join"],
            ),
            (
                "-1px 2% / 3px overlap-join !important",
                ["-1px", "2%", "3px", "overlap-join"],
            ),
        ] {
            let p = parse(id, text);
            assert_eq!(
                p.iter().map(|p| p.PropertyID()).collect::<Vec<_>>(),
                expected
            );
            assert_eq!(
                p.iter().map(|p| p.ShorthandID()).collect::<Vec<_>>(),
                expected
                    .iter()
                    .zip(&identity)
                    .map(|(&field, &id)| stored_identity(field, id))
                    .collect::<Vec<_>>()
            );
            for (index, p) in p.iter().enumerate() {
                assert_eq!(p.Value().CssText().Utf8(), texts[index % 4]);
                assert!(!p.IsImplicit());
                assert_eq!(p.IsImportant(), text.contains("!important"));
            }
        }
    }
    for (id, expected, identity, pair) in [
        (
            P::kColumnRuleInsetCap,
            vec![COLUMN[0], COLUMN[1]],
            vec![P::kColumnRuleInsetCap; 2],
            true,
        ),
        (
            P::kRowRuleInsetCap,
            vec![ROW[0], ROW[1]],
            vec![P::kRowRuleInsetCap; 2],
            true,
        ),
        (
            P::kRuleInsetCap,
            vec![ROW[0], ROW[1], COLUMN[0], COLUMN[1]],
            vec![
                P::kRowRuleInsetCap,
                P::kRowRuleInsetCap,
                P::kColumnRuleInsetCap,
                P::kColumnRuleInsetCap,
            ],
            true,
        ),
        (
            P::kColumnRuleInsetJunction,
            vec![COLUMN[2], COLUMN[3]],
            vec![P::kColumnRuleInsetJunction; 2],
            true,
        ),
        (
            P::kRowRuleInsetJunction,
            vec![ROW[2], ROW[3]],
            vec![P::kRowRuleInsetJunction; 2],
            true,
        ),
        (
            P::kRuleInsetJunction,
            vec![ROW[2], ROW[3], COLUMN[2], COLUMN[3]],
            vec![
                P::kRowRuleInsetJunction,
                P::kRowRuleInsetJunction,
                P::kColumnRuleInsetJunction,
                P::kColumnRuleInsetJunction,
            ],
            true,
        ),
        (
            P::kColumnRuleInsetStart,
            vec![COLUMN[0], COLUMN[2]],
            vec![P::kColumnRuleInsetStart; 2],
            false,
        ),
        (
            P::kRowRuleInsetStart,
            vec![ROW[0], ROW[2]],
            vec![P::kRowRuleInsetStart; 2],
            false,
        ),
        (
            P::kRuleInsetStart,
            vec![COLUMN[0], COLUMN[2], ROW[0], ROW[2]],
            vec![P::kRuleInsetStart; 4],
            false,
        ),
        (
            P::kColumnRuleInsetEnd,
            vec![COLUMN[1], COLUMN[3]],
            vec![P::kColumnRuleInsetEnd; 2],
            false,
        ),
        (
            P::kRowRuleInsetEnd,
            vec![ROW[1], ROW[3]],
            vec![P::kRowRuleInsetEnd; 2],
            false,
        ),
        (
            P::kRuleInsetEnd,
            vec![COLUMN[1], COLUMN[3], ROW[1], ROW[3]],
            vec![P::kRuleInsetEnd; 4],
            false,
        ),
    ] {
        for text in [
            "overlap-join",
            if pair {
                "overlap-join -2% !important"
            } else {
                "-2% !important"
            },
        ] {
            let p = parse(id, text);
            assert_eq!(
                p.iter().map(|p| p.PropertyID()).collect::<Vec<_>>(),
                expected
            );
            assert_eq!(
                p.iter().map(|p| p.ShorthandID()).collect::<Vec<_>>(),
                expected
                    .iter()
                    .zip(&identity)
                    .map(|(&field, &id)| stored_identity(field, id))
                    .collect::<Vec<_>>()
            );
            for (index, p) in p.iter().enumerate() {
                assert_eq!(
                    p.Value().CssText().Utf8(),
                    if text == "overlap-join" || pair && index % 2 == 0 {
                        "overlap-join"
                    } else {
                        "-2%"
                    }
                );
                assert!(!p.IsImplicit());
            }
        }
    }
    let declarations =
        ParseDeclarationList(&String::from("--inset:1px / -20%"), Mode::kHTMLStandardMode);
    let style::css_value::CSSValuePayload::kUnparsedDeclarationClass(data) =
        declarations.properties[0].Value().Payload()
    else {
        panic!("custom tokens");
    };
    assert_eq!(
        ParsePropertyTokens(P::kRuleInset, &data.data, Mode::kHTMLStandardMode)
            .unwrap()
            .len(),
        8
    );
    for id in SHORTHANDS {
        let len = parse(id, "0").len();
        for wide in ["initial", "inherit", "unset", "revert", "revert-layer"] {
            let p = parse(id, wide);
            assert_eq!(p.len(), len);
            assert!(p
                .iter()
                .all(|p| p.ShorthandID() == stored_identity(p.PropertyID(), id)));
        }
    }
}

#[test]
fn eight_longhands_and_shorthands_reject_invalid_grammar_without_partial_acceptance() {
    let _heap = foundation::LayoutHeapScope::new();
    for id in [COLUMN, ROW].concat() {
        for text in ["0", "-3px", "-20%", "overlap-join", "calc(3px - 2%)"] {
            assert_eq!(parse(id, text).len(), 1);
        }
        for text in [
            "auto",
            "normal",
            "1",
            "2px 3px",
            "2px/3px",
            "repeat(2, 1px)",
        ] {
            assert_eq!(
                ParseProperty(id, &String::from(text), false, Mode::kHTMLStandardMode)
                    .err()
                    .unwrap()
                    .kind,
                if text.starts_with("repeat(") {
                    PropertyParseErrorKind::Unsupported
                } else {
                    PropertyParseErrorKind::Invalid
                },
                "{id:?} {text}"
            );
        }
        assert!(ParseProperty(id, &String::from("3"), false, Mode::kHTMLQuirksMode).is_err());
    }
    for id in SHORTHANDS {
        for text in [
            "",
            "/ 1px",
            "1px /",
            "1px / 2px / 3px",
            "1px 2px 3px",
            "1px, 2px",
            "auto",
            "1",
        ] {
            assert_eq!(
                ParseProperty(id, &String::from(text), false, Mode::kHTMLStandardMode)
                    .err()
                    .unwrap()
                    .kind,
                if text.starts_with("repeat(") {
                    PropertyParseErrorKind::Unsupported
                } else {
                    PropertyParseErrorKind::Invalid
                },
                "{id:?} {text}"
            );
        }
    }
    for id in [
        P::kColumnRuleInsetStart,
        P::kColumnRuleInsetEnd,
        P::kRowRuleInsetStart,
        P::kRowRuleInsetEnd,
        P::kRuleInsetStart,
        P::kRuleInsetEnd,
    ] {
        assert!(
            ParseProperty(id, &String::from("1px 2px"), false, Mode::kHTMLStandardMode).is_err()
        );
    }
}

#[test]
fn style_engine_cascade_variables_css_wide_and_native_length_conversion() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p style='font-size:20px;rule-inset:-2em 20% / overlap-join calc(3px + 4%);column-rule-inset-cap-end:7px'><div id=c style='font-size:10px;rule-inset:inherit'></div><div id=i style='rule-inset:initial'></div><div id=u style='rule-inset:unset'></div><div id=none></div></div><div id=v style='--insets:1px 2px / 3px 4px;rule-inset:var(--insets);rule-inset-start:9%;column-rule-inset-end:overlap-join;row-rule-inset-cap:5px !important;row-rule-inset-cap:6px;rule:2px solid red;direction:rtl;writing-mode:vertical-rl'></div><div id=layer></div>");
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS(
        "@layer early,late;@layer early{#layer{rule-inset:8px}}@layer late{#layer{rule-inset:10px;rule-inset:revert-layer}}"));
    let mut engine = StyleEngine::new(&owner);
    engine
        .Update(&mut owner, &MediaValuesCachedData::default(), &[])
        .unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    for id in ["p", "c"] {
        let a = insets(native(&owner, id));
        assert_eq!(a[0].Pixels(), -40.);
        assert_eq!(a[1].Pixels(), 7.);
        assert!(a[2].IsOverlapJoin());
        assert!(a[3].IsCalculated());
        assert_eq!(a[4].Pixels(), -40.);
        assert_eq!(a[5].PercentValue(), 20.);
        assert!(a[6].IsOverlapJoin());
        assert!(a[7].IsCalculated());
    }
    for id in ["i", "u", "none"] {
        assert!(insets(native(&owner, id))
            .iter()
            .all(|l| l.IsFixed() && l.Pixels() == 0.));
    }
    assert!(insets(native(&owner, "layer"))
        .iter()
        .all(|l| l.Pixels() == 8.));
    let a = insets(native(&owner, "v"));
    assert_eq!(a[0].PercentValue(), 9.);
    assert!(a[1].IsOverlapJoin());
    assert_eq!(a[2].PercentValue(), 9.);
    assert!(a[3].IsOverlapJoin());
    assert_eq!(a[4].Pixels(), 5.);
    assert_eq!(a[5].Pixels(), 5.);
    assert_eq!(a[6].PercentValue(), 9.);
    assert_eq!(a[7].Pixels(), 4.);
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let media = MediaValuesCachedData::default();
    let mut b = ComputedStyleBuilder::from_style(initial);
    b.SetEffectiveZoom(2.);
    for id in [COLUMN, ROW].concat() {
        let p = parse(id, "-3px");
        Apply(id, &mut b, None, p[0].Value(), 16., &media).unwrap();
    }
    let parent = unsafe { &*b.TakeStyle() };
    assert!(insets(parent).iter().all(|l| l.Pixels() == -6.));
    let mut child = ComputedStyleBuilder::from_style(initial);
    child.SetEffectiveZoom(2.);
    for id in [COLUMN, ROW].concat() {
        Apply(
            id,
            &mut child,
            Some(parent),
            &values::wide(V::kInherit).unwrap(),
            16.,
            &media,
        )
        .unwrap();
    }
    let inherited = unsafe { &*child.TakeStyle() };
    assert!(insets(inherited).iter().all(|l| l.Pixels() == -6.));
    child = ComputedStyleBuilder::from_style(initial);
    for id in [COLUMN, ROW].concat() {
        assert!(Apply(
            id,
            &mut child,
            Some(parent),
            &values::wide(V::kInherit).unwrap(),
            16.,
            &media
        )
        .is_err());
    }
}
