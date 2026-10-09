use foundation::{CSSPropertyID as P, Color, EBorderStyle as S, String as CSSString};
use layoutng_style::style::computed_style::ComputedStyle;
use style::{
    media_queries::MediaValuesCachedData,
    parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseProperty, PropertyParseErrorKind},
    },
    StyleEngine,
};
fn native<'a>(owner: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
    let d = owner.GetDocument();
    let node = (0..d.NodeCount())
        .find(|&i| d.Node(i).FindAttribute("id").is_some_and(|a| a.value == id))
        .unwrap();
    unsafe { &*d.ResolvedStyleFor(node).unwrap().native_style.Get() }
}
fn update(owner: &mut dom::DOM) {
    let mut engine = StyleEngine::new(owner);
    engine
        .Update(owner, &MediaValuesCachedData::default(), &[])
        .unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
}
fn specified(s: &ComputedStyle) -> [i32; 4] {
    [
        *s.SpecifiedBorderTopWidth(),
        *s.SpecifiedBorderRightWidth(),
        *s.SpecifiedBorderBottomWidth(),
        *s.SpecifiedBorderLeftWidth(),
    ]
}
fn styles(s: &ComputedStyle) -> [S; 4] {
    [
        s.BorderTopStyle(),
        s.BorderRightStyle(),
        s.BorderBottomStyle(),
        s.BorderLeftStyle(),
    ]
}

#[test]
fn all_logical_side_shorthands_map_to_native_for_every_writing_direction() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut markup = std::string::String::new();
    let cases = [
        ("horizontal-tb", "ltr", [1, 4, 2, 3]),
        ("horizontal-tb", "rtl", [1, 3, 2, 4]),
        ("vertical-rl", "ltr", [3, 1, 4, 2]),
        ("vertical-rl", "rtl", [4, 1, 3, 2]),
        ("vertical-lr", "ltr", [3, 2, 4, 1]),
        ("vertical-lr", "rtl", [4, 2, 3, 1]),
        ("sideways-rl", "ltr", [3, 1, 4, 2]),
        ("sideways-rl", "rtl", [4, 1, 3, 2]),
        ("sideways-lr", "ltr", [4, 2, 3, 1]),
        ("sideways-lr", "rtl", [3, 2, 4, 1]),
    ];
    for (i, (mode, dir, _)) in cases.iter().enumerate() {
        markup.push_str(&format!("<div id=n{i} style='border-block-start:solid red 1px;border-block-end:2px blue dashed;border-inline-start:green dotted calc(1px + 2px);border-inline-end:double 4px black;writing-mode:{mode};direction:{dir}'></div>"));
    }
    let mut owner = html::html_parser::ParseHTML(&markup);
    update(&mut owner);
    let colors = [
        Color::FromRGB(255, 0, 0),
        Color::FromRGB(0, 0, 255),
        Color::FromRGB(0, 128, 0),
        Color::FromRGB(0, 0, 0),
    ];
    let expected_styles = [S::kSolid, S::kDashed, S::kDotted, S::kDouble];
    for (i, (_, _, expected)) in cases.iter().enumerate() {
        let s = native(&owner, &format!("n{i}"));
        assert_eq!(specified(s), *expected, "case {i}");
        let actual_colors = [
            s.BorderTopColor(),
            s.BorderRightColor(),
            s.BorderBottomColor(),
            s.BorderLeftColor(),
        ];
        for side in 0..4 {
            assert_eq!(
                styles(s)[side],
                expected_styles[expected[side] as usize - 1]
            );
            assert_eq!(
                actual_colors[side].GetColor(),
                colors[expected[side] as usize - 1]
            );
        }
    }
}

#[test]
fn axis_and_single_side_resets_share_existing_border_fields() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<div id=a style='border-block-width:1px 2px;border-block-style:solid dashed;border-block-color:red blue;border-inline-width:3px 4px;border-inline-style:dotted double;border-inline-color:green black;border-block:red'></div><div id=b style='border-block:9px double black;border-inline:8px dashed blue;border-block:dotted;border-inline:green'></div><div id=c style='border-block:9px solid blue;border-block-start:red'></div>");
    update(&mut owner);
    let a = native(&owner, "a");
    assert_eq!(specified(a), [3, 4, 3, 3]);
    assert_eq!(styles(a), [S::kNone, S::kDouble, S::kNone, S::kDotted]);
    assert_eq!(a.BorderTopColor().GetColor(), Color::FromRGB(255, 0, 0));
    assert_eq!(a.BorderBottomColor().GetColor(), Color::FromRGB(255, 0, 0));
    assert_eq!(a.BorderRightColor().GetColor(), Color::FromRGB(0, 0, 0));
    let b = native(&owner, "b");
    assert_eq!(specified(b), [3, 3, 3, 3]);
    assert_eq!(styles(b), [S::kDotted, S::kNone, S::kDotted, S::kNone]);
    assert!(b.BorderTopColor().IsCurrentColor());
    assert!(b.BorderBottomColor().IsCurrentColor());
    assert_eq!(b.BorderLeftColor().GetColor(), Color::FromRGB(0, 128, 0));
    let c = native(&owner, "c");
    assert_eq!(specified(c), [3, 3, 9, 3]);
    assert_eq!(c.BorderTopStyle(), S::kNone);
    assert_eq!(c.BorderBottomStyle(), S::kSolid);
    assert_eq!(c.BorderTopColor().GetColor(), Color::FromRGB(255, 0, 0));
    assert_eq!(c.BorderBottomColor().GetColor(), Color::FromRGB(0, 0, 255));
}

#[test]
fn logical_border_css_wide_inherits_and_initializes_native_physical_sides() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<div id=p style='direction:rtl;writing-mode:vertical-rl;border-block:2px solid red;border-inline:4px double blue'><i id=c style='border-block:inherit;border-inline:inherit'></i><i id=r style='border-block:initial;border-inline:initial'></i><i id=u style='border-block:unset;border-inline:unset'></i><i id=s style='border-block-start:inherit;border-block-end:inherit;border-inline-start:inherit;border-inline-end:inherit'></i></div>");
    update(&mut owner);
    let p = native(&owner, "p");
    for id in ["c", "s"] {
        let s = native(&owner, id);
        assert_eq!(specified(s), specified(p));
        assert_eq!(styles(s), styles(p));
        assert!(s.BorderRightColor() == p.BorderRightColor());
        assert!(s.BorderTopColor() == p.BorderTopColor());
    }
    for id in ["r", "u"] {
        let s = native(&owner, id);
        assert_eq!(specified(s), [3; 4]);
        assert_eq!(styles(s), [S::kNone; 4]);
        assert!(s.BorderRightColor().IsCurrentColor());
        assert!(s.BorderTopColor().IsCurrentColor());
    }
    for id in [
        P::kBorderBlock,
        P::kBorderInline,
        P::kBorderBlockStart,
        P::kBorderBlockEnd,
        P::kBorderInlineStart,
        P::kBorderInlineEnd,
    ] {
        for keyword in ["initial", "inherit", "unset", "revert", "revert-layer"] {
            let parsed = ParseProperty(
                id,
                &CSSString::from(keyword),
                true,
                CSSParserMode::kHTMLStandardMode,
            )
            .unwrap();
            assert_eq!(
                parsed.len(),
                if matches!(id, P::kBorderBlock | P::kBorderInline) {
                    6
                } else {
                    3
                }
            );
            for p in parsed {
                assert_eq!(p.ValueRef().CssText().Utf8(), keyword);
                assert!(p.IsImportant());
                assert!(!p.IsImplicit());
            }
        }
    }
}

#[test]
fn source_expansion_metadata_defaults_and_invalid_declarations_are_atomic() {
    let axis = ParseProperty(
        P::kBorderBlock,
        &CSSString::from("solid"),
        true,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap();
    assert_eq!(axis.len(), 6);
    for p in &axis {
        assert!(p.IsImportant());
        assert!(!p.IsImplicit());
        assert!(matches!(
            p.ShorthandID(),
            P::kBorderBlockWidth | P::kBorderBlockStyle | P::kBorderBlockColor
        ));
    }
    assert_eq!(axis[0].ValueRef().CssText().Utf8(), "medium");
    assert_eq!(axis[2].ValueRef().CssText().Utf8(), "solid");
    assert_eq!(axis[4].ValueRef().CssText().Utf8(), "currentcolor");
    assert!(std::rc::Rc::ptr_eq(
        &axis[0].ValueRef(),
        &axis[1].ValueRef()
    ));
    let side = ParseProperty(
        P::kBorderInlineEnd,
        &CSSString::from("solid"),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap();
    assert_eq!(side[0].ValueRef().CssText().Utf8(), "initial");
    assert_eq!(side[1].ValueRef().CssText().Utf8(), "solid");
    assert_eq!(side[2].ValueRef().CssText().Utf8(), "initial");
    // These commas come from the actual ConsumeBorderShorthand source; the
    // greedy side shorthand uses whitespace and rejects a comma.
    assert!(ParseProperty(
        P::kBorderBlock,
        &CSSString::from("2px,solid,red,"),
        false,
        CSSParserMode::kHTMLStandardMode
    )
    .is_ok());
    for id in [
        P::kBorderBlock,
        P::kBorderInline,
        P::kBorderBlockStart,
        P::kBorderBlockEnd,
        P::kBorderInlineStart,
        P::kBorderInlineEnd,
    ] {
        for text in [
            "",
            "1px 2px solid red",
            "solid dashed",
            "red blue",
            "-1px solid",
            "10% solid",
            "solid bogus",
            "inherit red",
        ] {
            let e = ParseProperty(
                id,
                &CSSString::from(text),
                false,
                CSSParserMode::kHTMLStandardMode,
            )
            .err()
            .unwrap_or_else(|| panic!("accepted {id:?} {text}"));
            assert_eq!(
                e.kind,
                PropertyParseErrorKind::Invalid,
                "{id:?} {text}: {e:?}"
            );
        }
        for text in ["solid 112233", "2 solid red"] {
            assert!(ParseProperty(
                id,
                &CSSString::from(text),
                false,
                CSSParserMode::kHTMLQuirksMode
            )
            .is_err());
        }
    }
    assert!(ParseProperty(
        P::kBorderBlockStart,
        &CSSString::from("2px, solid"),
        false,
        CSSParserMode::kHTMLStandardMode
    )
    .is_err());
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<div id=x style='border-block:5px solid red;border-block:1px 2px dashed blue;border-inline:7px dotted blue;border-inline-start:solid dashed'></div>");
    update(&mut owner);
    let x = native(&owner, "x");
    assert_eq!(specified(x), [5, 7, 5, 7]);
    assert_eq!(styles(x), [S::kSolid, S::kDotted, S::kSolid, S::kDotted]);
}
