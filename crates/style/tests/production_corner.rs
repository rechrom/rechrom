use foundation::{CSSPropertyID, String, TextDirection, WritingDirectionMode, WritingMode};
use layoutng_style::style::{
    basic_shapes::{BasicShapePolygon, ShapeType},
    computed_style::ComputedStyle,
    computed_style_constants::GeometryBox,
    style_border_shape::StyleBorderShape,
};
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
    let node = (0..document.NodeCount())
        .find(|&i| {
            document
                .Node(i)
                .FindAttribute("id")
                .is_some_and(|a| a.value == id)
        })
        .unwrap();
    unsafe { &*document.ResolvedStyleFor(node).unwrap().native_style.Get() }
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
fn border_shape(style: &ComputedStyle) -> &StyleBorderShape {
    unsafe { &*style.BorderShape().Get() }
}

#[test]
fn corner_shapes_reach_native_fields_with_wide_keywords_and_unclamped_math() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=p style='corner-shape:bevel notch squircle scoop'><div id=c style='corner-shape:inherit'></div><div id=i style='corner-shape:initial'></div><div id=u style='corner-shape:unset'></div></div><div id=m style='corner-shape:superellipse(infinity) superellipse(-infinity) superellipse(calc(3 / 2)) superellipse(calc(infinity))'></div>");
    update(&mut owner);
    for id in ["p", "c"] {
        let style = native(&owner, id);
        assert_eq!(style.CornerTopLeftShape().Parameter(), 0.0);
        assert_eq!(style.CornerTopRightShape().Parameter(), f64::NEG_INFINITY);
        assert_eq!(style.CornerBottomRightShape().Parameter(), 2.0);
        assert_eq!(style.CornerBottomLeftShape().Parameter(), -1.0);
    }
    for id in ["i", "u"] {
        let style = native(&owner, id);
        assert_eq!(style.CornerTopLeftShape().Parameter(), 1.0);
        assert_eq!(style.CornerTopRightShape().Parameter(), 1.0);
        assert_eq!(style.CornerBottomRightShape().Parameter(), 1.0);
        assert_eq!(style.CornerBottomLeftShape().Parameter(), 1.0);
    }
    let math = native(&owner, "m");
    assert_eq!(math.CornerTopLeftShape().Parameter(), f64::INFINITY);
    assert_eq!(math.CornerTopRightShape().Parameter(), f64::NEG_INFINITY);
    assert_eq!(math.CornerBottomRightShape().Parameter(), 1.5);
    assert_eq!(math.CornerBottomLeftShape().Parameter(), f64::INFINITY);
}

#[test]
fn logical_corner_shapes_follow_writing_direction_and_cascade_order() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=x style='corner-shape:round;corner-block-start-shape:bevel squircle;corner-inline-end-shape:scoop square;writing-mode:vertical-rl;direction:rtl'></div>");
    update(&mut owner);
    let style = native(&owner, "x");
    assert_eq!(style.CornerBottomRightShape().Parameter(), 0.0);
    assert_eq!(style.CornerTopRightShape().Parameter(), -1.0);
    assert_eq!(style.CornerTopLeftShape().Parameter(), f64::INFINITY);
    assert_eq!(style.CornerBottomLeftShape().Parameter(), 1.0);
    for mode in [
        WritingMode::kHorizontalTb,
        WritingMode::kVerticalRl,
        WritingMode::kVerticalLr,
        WritingMode::kSidewaysRl,
        WritingMode::kSidewaysLr,
    ] {
        for direction in [TextDirection::kLtr, TextDirection::kRtl] {
            let direction = WritingDirectionMode::new(mode, direction);
            for (shape, radius) in [
                (
                    CSSPropertyID::kCornerStartStartShape,
                    CSSPropertyID::kBorderStartStartRadius,
                ),
                (
                    CSSPropertyID::kCornerStartEndShape,
                    CSSPropertyID::kBorderStartEndRadius,
                ),
                (
                    CSSPropertyID::kCornerEndStartShape,
                    CSSPropertyID::kBorderEndStartRadius,
                ),
                (
                    CSSPropertyID::kCornerEndEndShape,
                    CSSPropertyID::kBorderEndEndRadius,
                ),
            ] {
                let expected = match ResolvePhysical(radius, direction) {
                    CSSPropertyID::kBorderTopLeftRadius => CSSPropertyID::kCornerTopLeftShape,
                    CSSPropertyID::kBorderTopRightRadius => CSSPropertyID::kCornerTopRightShape,
                    CSSPropertyID::kBorderBottomRightRadius => {
                        CSSPropertyID::kCornerBottomRightShape
                    }
                    CSSPropertyID::kBorderBottomLeftRadius => CSSPropertyID::kCornerBottomLeftShape,
                    _ => unreachable!(),
                };
                assert_eq!(ResolvePhysical(shape, direction), expected);
            }
        }
    }
}

#[test]
fn border_shape_preserves_default_boxes_pair_equality_and_native_ownership() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=p style='border-shape:polygon(0 0,10px 0,10px 20px)'><div id=c style='border-shape:inherit'></div><div id=i style='border-shape:initial'></div></div><div id=two style='border-shape:polygon(0 0,10px 0,10px 20px) polygon(0 0,10px 0,10px 20px)'></div><div id=equal style='border-shape:polygon(0 0,10px 0,10px 20px) content-box polygon(0 0,10px 0,10px 20px) content-box'></div><div id=boxes style='border-shape:polygon(0 0,10px 0,10px 20px) border-box polygon(0 0,10px 0,10px 20px) half-border-box'></div>");
    update(&mut owner);
    let single = border_shape(native(&owner, "p"));
    assert_eq!(single.OuterBox(), GeometryBox::kHalfBorderBox);
    assert_eq!(single.InnerBox(), GeometryBox::kHalfBorderBox);
    assert!(!single.HasSeparateInnerShape());
    assert!(single == border_shape(native(&owner, "c")));
    assert!(native(&owner, "i").BorderShape().GetNonNull().is_none());
    assert_eq!(
        single.OuterShape().GetType(),
        ShapeType::kBasicShapePolygonType
    );
    let polygon = unsafe {
        &*(single.OuterShape() as *const dyn layoutng_style::style::basic_shapes::BasicShape
            as *const BasicShapePolygon)
    };
    assert_eq!(polygon.Values()[2].Pixels(), 10.0);
    assert_eq!(polygon.Values()[5].Pixels(), 20.0);
    let two = border_shape(native(&owner, "two"));
    assert_eq!(
        (two.OuterBox(), two.InnerBox()),
        (GeometryBox::kBorderBox, GeometryBox::kPaddingBox)
    );
    assert!(two.HasSeparateInnerShape());
    let equal = border_shape(native(&owner, "equal"));
    assert_eq!(
        (equal.OuterBox(), equal.InnerBox()),
        (GeometryBox::kContentBox, GeometryBox::kContentBox)
    );
    assert!(!equal.HasSeparateInnerShape());
    let boxes = border_shape(native(&owner, "boxes"));
    assert_eq!(
        (boxes.OuterBox(), boxes.InnerBox()),
        (GeometryBox::kBorderBox, GeometryBox::kHalfBorderBox)
    );
    assert!(boxes.HasSeparateInnerShape());
}

#[test]
fn corner_and_border_shape_reject_invalid_syntax_and_keep_runtime_gaps_explicit() {
    for (id, text) in [
        (CSSPropertyID::kCornerTopLeftShape, "normal"),
        (CSSPropertyID::kCornerTopLeftShape, "superellipse()"),
        (CSSPropertyID::kCornerTopLeftShape, "superellipse(1px)"),
        (CSSPropertyID::kCornerTopLeftShape, "superellipse(1 2)"),
        (
            CSSPropertyID::kCornerShape,
            "round scoop square bevel notch",
        ),
        (CSSPropertyID::kCornerTopShape, "round scoop square"),
        (
            CSSPropertyID::kBorderShape,
            "border-box polygon(0 0,1px 0,0 1px)",
        ),
        (CSSPropertyID::kBorderShape, "none polygon(0 0,1px 0,0 1px)"),
    ] {
        assert!(
            ParseProperty(
                id,
                &String::from(text),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .is_err(),
            "{id:?}: {text}"
        );
    }
    for (id, text) in [
        (CSSPropertyID::kCorner, "10px bevel"),
        (CSSPropertyID::kCornerTopLeft, "normal"),
        (CSSPropertyID::kBorderShape, "circle()"),
    ] {
        assert_eq!(
            ParseProperty(
                id,
                &String::from(text),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .err()
            .unwrap()
            .kind,
            PropertyParseErrorKind::Unsupported
        );
    }
}
