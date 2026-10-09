use foundation::{
    CSSPropertyID, EAspectRatioType, String, TextDirection, WritingDirectionMode, WritingMode,
};
use layoutng_style::style::{computed_style::ComputedStyle, computed_style_constants::Containment};
use style::{
    media_queries::MediaValuesCachedData,
    parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseProperty, PropertyParseErrorKind},
    },
    properties::longhand_dispatch::ResolvePhysical,
    StyleEngine,
};

fn native<'a>(owner: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
    let document = owner.GetDocument();
    let index = (0..document.NodeCount())
        .find(|&i| {
            document
                .Node(i)
                .FindAttribute("id")
                .is_some_and(|a| a.value == id)
        })
        .unwrap();
    unsafe { &*document.ResolvedStyleFor(index).unwrap().native_style.Get() }
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

#[test]
fn native_layout_initial_inherit_and_typed_math_values() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=p style='aspect-ratio:auto calc(12 / 2) / 4;column-count:calc(2 + 2.5);column-width:calc(10px + 2px);column-height:30px;contain:paint layout size;contain-intrinsic-size:auto calc(8px + 2px) auto none;orphans:calc(2 + 2);widows:99999'><div id=c style='aspect-ratio:inherit;column-count:inherit;column-width:inherit;column-height:inherit;contain:inherit;contain-intrinsic-size:inherit;orphans:inherit;widows:inherit'></div><div id=i style='aspect-ratio:initial;column-count:initial;column-width:initial;column-height:initial;contain:initial;contain-intrinsic-size:initial;orphans:initial;widows:initial'></div></div>");
    update(&mut owner);
    let (p, c, i) = (
        native(&owner, "p"),
        native(&owner, "c"),
        native(&owner, "i"),
    );
    assert_eq!(
        p.AspectRatio().GetTypeForComputedStyle(),
        EAspectRatioType::kAutoAndRatio
    );
    assert_eq!(
        p.AspectRatio().GetRatio(),
        foundation::gfx::SizeF::new(6.0, 4.0)
    );
    assert_eq!(p.AspectRatio(), c.AspectRatio());
    assert_eq!((p.ColumnCount(), c.ColumnCount()), (5, 5));
    assert!(!p.HasAutoColumnCount());
    assert_eq!((p.ColumnWidth(), c.ColumnWidth()), (12.0, 12.0));
    assert_eq!((p.ColumnHeight(), c.ColumnHeight()), (30.0, 30.0));
    assert_eq!(
        p.Contain(),
        (Containment::kContainsPaint | Containment::kContainsLayout | Containment::kContainsSize)
            .value() as u32
    );
    assert_eq!(p.Contain(), c.Contain());
    assert!(p.ContainIntrinsicWidth().HasAuto());
    assert_eq!(
        p.ContainIntrinsicWidth()
            .GetLength()
            .as_ref()
            .unwrap()
            .Pixels(),
        10.0
    );
    assert!(p.ContainIntrinsicHeight().HasAuto());
    assert!(p.ContainIntrinsicHeight().GetLength().is_none());
    assert_eq!(p.ContainIntrinsicWidth(), c.ContainIntrinsicWidth());
    assert_eq!(p.ContainIntrinsicHeight(), c.ContainIntrinsicHeight());
    assert_eq!(
        (p.Orphans(), c.Orphans(), p.Widows(), c.Widows()),
        (4, 4, i16::MAX, i16::MAX)
    );
    assert!(i.AspectRatio().IsAuto());
    assert!(i.HasAutoColumnCount() && i.HasAutoColumnWidth() && i.HasAutoColumnHeight());
    assert_eq!(i.Contain(), 0);
    assert!(i.ContainIntrinsicWidth().IsNoOp() && i.ContainIntrinsicHeight().IsNoOp());
    assert_eq!((i.Orphans(), i.Widows()), (2, 2));
}

#[test]
fn columns_expands_height_and_wrap_and_auto_restores_native_flags() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=a style='column-wrap:wrap;columns:3 20px / calc(25px * 2)'></div><div id=b style='column-count:4;column-width:10px;column-height:15px;column-wrap:nowrap;columns:auto auto'></div><div id=c style='columns:0 2'></div><div id=d style='columns:calc(3 + 1) 12px'></div>");
    update(&mut owner);
    let (a, b, c, d) = (
        native(&owner, "a"),
        native(&owner, "b"),
        native(&owner, "c"),
        native(&owner, "d"),
    );
    assert_eq!(
        (a.ColumnCount(), a.ColumnWidth(), a.ColumnHeight()),
        (3, 20.0, 50.0)
    );
    assert_eq!(a.ColumnWrap(), b.ColumnWrap());
    assert!(b.HasAutoColumnCount() && b.HasAutoColumnWidth() && b.HasAutoColumnHeight());
    assert_eq!((c.ColumnWidth(), c.ColumnCount()), (0.0, 2));
    assert!(!c.HasAutoColumnWidth());
    assert_eq!((d.ColumnCount(), d.ColumnWidth()), (4, 12.0));
}

#[test]
fn intrinsic_logical_fields_follow_all_writing_modes_and_cascade_order() {
    // Regression for the generator's multi-line PhysicalMapping declaration:
    // css_direction_aware_resolver.cc:157-163; generated longhands.cc:6657-6661,6738-6742.
    for mode in [
        WritingMode::kHorizontalTb,
        WritingMode::kVerticalRl,
        WritingMode::kVerticalLr,
        WritingMode::kSidewaysRl,
        WritingMode::kSidewaysLr,
    ] {
        let direction = WritingDirectionMode::new(mode, TextDirection::kRtl);
        assert_eq!(
            ResolvePhysical(CSSPropertyID::kContainIntrinsicInlineSize, direction),
            if mode == WritingMode::kHorizontalTb {
                CSSPropertyID::kContainIntrinsicWidth
            } else {
                CSSPropertyID::kContainIntrinsicHeight
            }
        );
        assert_eq!(
            ResolvePhysical(CSSPropertyID::kContainIntrinsicBlockSize, direction),
            if mode == WritingMode::kHorizontalTb {
                CSSPropertyID::kContainIntrinsicHeight
            } else {
                CSSPropertyID::kContainIntrinsicWidth
            }
        );
    }
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=v style='writing-mode:vertical-rl;contain-intrinsic-size:1px 2px;contain-intrinsic-inline-size:auto 30px;contain-intrinsic-block-size:40px'></div><div id=h style='contain-intrinsic-size:auto 7px'></div><div id=s style='contain:strict'></div><div id=n style='contain:content'></div><div id=z style='aspect-ratio:0 / 2'></div><div id=o style='column-count:99999;column-width:1e100px;contain-intrinsic-width:1e100px'></div>");
    update(&mut owner);
    let (v, h, s, n, z) = (
        native(&owner, "v"),
        native(&owner, "h"),
        native(&owner, "s"),
        native(&owner, "n"),
        native(&owner, "z"),
    );
    assert_eq!(
        v.ContainIntrinsicWidth()
            .GetLength()
            .as_ref()
            .unwrap()
            .Pixels(),
        40.0
    );
    assert_eq!(
        v.ContainIntrinsicHeight()
            .GetLength()
            .as_ref()
            .unwrap()
            .Pixels(),
        30.0
    );
    assert!(v.ContainIntrinsicHeight().HasAuto());
    assert_eq!(h.ContainIntrinsicWidth(), h.ContainIntrinsicHeight());
    assert_eq!(s.Contain(), Containment::kContainsStrict.value() as u32);
    assert_eq!(n.Contain(), Containment::kContainsContent.value() as u32);
    assert_eq!(
        z.AspectRatio().GetTypeForComputedStyle(),
        EAspectRatioType::kRatio
    );
    assert!(z.AspectRatio().IsAuto());
    let o = native(&owner, "o");
    assert_eq!(o.ColumnCount(), u16::MAX);
    assert_eq!(o.ColumnWidth(), f32::MAX);
    assert_eq!(
        o.ContainIntrinsicWidth()
            .GetLength()
            .as_ref()
            .unwrap()
            .Pixels(),
        (foundation::LayoutUnit::Max().ToInt() - 2) as f32
    );
}

#[test]
fn invalid_layout_grammar_and_unavailable_math_are_typed() {
    for (id, text) in [
        (CSSPropertyID::kAspectRatio, "auto 1 /"),
        (CSSPropertyID::kAspectRatio, "1 / -2"),
        (CSSPropertyID::kAspectRatio, "auto auto"),
        (CSSPropertyID::kContain, "size inline-size"),
        (CSSPropertyID::kContain, "paint paint"),
        (CSSPropertyID::kContain, "strict paint"),
        (CSSPropertyID::kColumnCount, "0"),
        (CSSPropertyID::kColumnCount, "1.5"),
        (CSSPropertyID::kColumnWidth, "-2px"),
        (CSSPropertyID::kColumnWidth, "10%"),
        (CSSPropertyID::kColumns, "2 3"),
        (CSSPropertyID::kColumns, "12px 14px"),
        (CSSPropertyID::kColumns, "1 /"),
        (CSSPropertyID::kContainIntrinsicSize, "auto"),
        (CSSPropertyID::kContainIntrinsicSize, "none auto"),
        (CSSPropertyID::kContainIntrinsicWidth, "-1px"),
        (CSSPropertyID::kOrphans, "0"),
        (CSSPropertyID::kWidows, "1.2"),
    ] {
        let error = ParseProperty(
            id,
            &String::from(text),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .err()
        .unwrap();
        assert_eq!(
            error.kind,
            PropertyParseErrorKind::Invalid,
            "{id:?}: {text}"
        );
    }
    for id in [CSSPropertyID::kColumnWidth, CSSPropertyID::kColumnHeight] {
        assert!(ParseProperty(
            id,
            &String::from("2"),
            false,
            CSSParserMode::kHTMLQuirksMode
        )
        .is_err());
    }
    let error = ParseProperty(
        CSSPropertyID::kAspectRatio,
        &String::from("calc(sin(30deg))"),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .err()
    .unwrap();
    assert_eq!(error.kind, PropertyParseErrorKind::Unsupported);
}

#[test]
fn invalid_declarations_preserve_valid_native_values() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<div id=x style='aspect-ratio:4/3;aspect-ratio:2 / -1;column-count:3;column-count:0;column-width:8px;column-width:-1px;contain:paint;contain:paint paint;contain-intrinsic-width:auto 10px;contain-intrinsic-width:auto;orphans:3;orphans:0'></div>");
    let mut engine = StyleEngine::new(&mut owner);
    engine
        .Update(&mut owner, &MediaValuesCachedData::default(), &[])
        .unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let x = native(&owner, "x");
    assert_eq!(
        x.AspectRatio().GetRatio(),
        foundation::gfx::SizeF::new(4.0, 3.0)
    );
    assert_eq!((x.ColumnCount(), x.ColumnWidth(), x.Orphans()), (3, 8.0, 3));
    assert_eq!(x.Contain(), Containment::kContainsPaint.value() as u32);
    assert!(x.ContainIntrinsicWidth().HasAuto());
    assert_eq!(
        x.ContainIntrinsicWidth()
            .GetLength()
            .as_ref()
            .unwrap()
            .Pixels(),
        10.0
    );
}
