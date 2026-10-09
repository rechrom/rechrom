use foundation::{CSSPropertyID, ECursor, String, WindRule};
use layoutng_style::style::{
    basic_shapes::{BasicShapePolygon, ShapeType},
    clip_path_operation::OperationType as ClipType,
    computed_style::ComputedStyle,
    computed_style_constants::{GeometryBox, ShapeBox},
    filter_operation::*,
    geometry_box_clip_path_operation::GeometryBoxClipPathOperation,
    shape_clip_path_operation::ShapeClipPathOperation,
    shape_value::ShapeValueType,
};
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
#[test]
fn all_css_filter_functions_reach_native_operations_with_defaults_clamping_and_math() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<div id=p style='filter:grayscale(200%) sepia() saturate(2) hue-rotate(.5turn) invert(calc(2)) opacity(25%) brightness(250%) contrast(0.5) blur(calc(1px + 2px)) drop-shadow(2px 3px 4px red);backdrop-filter:blur() hue-rotate()'><div id=c style='filter:inherit;backdrop-filter:inherit'></div><div id=i style='filter:initial;backdrop-filter:unset'></div></div>");
    update(&mut owner);
    let p = native(&owner, "p");
    let filters = p.Filter();
    assert_eq!(filters.size(), 10);
    for (index, kind, amount) in [
        (0, OperationType::kGrayscale, 1.0),
        (1, OperationType::kSepia, 1.0),
        (2, OperationType::kSaturate, 2.0),
        (3, OperationType::kHueRotate, 180.0),
    ] {
        let operation = unsafe { &*filters.at(index) };
        assert_eq!(operation.GetType(), kind);
        assert_eq!(
            unsafe {
                &*(operation as *const FilterOperation as *const BasicColorMatrixFilterOperation)
            }
            .Amount(),
            amount
        );
    }
    for (index, kind, amount) in [
        (4, OperationType::kInvert, 1.0),
        (5, OperationType::kOpacity, 0.25),
        (6, OperationType::kBrightness, 2.5),
        (7, OperationType::kContrast, 0.5),
    ] {
        let operation = unsafe { &*filters.at(index) };
        assert_eq!(operation.GetType(), kind);
        assert_eq!(
            unsafe {
                &*(operation as *const FilterOperation
                    as *const BasicComponentTransferFilterOperation)
            }
            .Amount(),
            amount
        );
    }
    let blur = unsafe { &*(filters.at(8) as *const BlurFilterOperation) };
    assert_eq!(blur.StdDeviation().Pixels(), 3.0);
    let shadow = unsafe { &*(filters.at(9) as *const DropShadowFilterOperation) }.Shadow();
    assert_eq!(
        (shadow.X(), shadow.Y(), shadow.BlurRadius(), shadow.Spread()),
        (2.0, 3.0, 4.0, 0.0)
    );
    assert!(p.Filter().HasFilterThatMovesPixels());
    assert!(p.Filter() == native(&owner, "c").Filter());
    assert!(p.BackdropFilter() == native(&owner, "c").BackdropFilter());
    assert_eq!(
        unsafe { &*(p.BackdropFilter().at(0) as *const BlurFilterOperation) }
            .StdDeviation()
            .Pixels(),
        0.0
    );
    assert_eq!(
        unsafe { &*(p.BackdropFilter().at(1) as *const BasicColorMatrixFilterOperation) }.Amount(),
        0.0
    );
    assert!(
        native(&owner, "i").Filter().IsEmpty() && native(&owner, "i").BackdropFilter().IsEmpty()
    );
}
#[test]
fn polygon_and_box_clipping_and_shape_outside_preserve_native_owners() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<div id=p style='clip-path:content-box polygon(evenodd round 2px,0 0,100% 0,calc(50% + 1px) 100%);shape-outside:polygon(0 0,10px 0,10px 20px) padding-box;shape-image-threshold:calc(3 / 4)'><div id=c style='clip-path:inherit;shape-outside:inherit;shape-image-threshold:inherit'></div><div id=i style='clip-path:initial;shape-outside:unset;shape-image-threshold:initial'></div></div><div id=b style='clip-path:fill-box;shape-outside:margin-box;shape-image-threshold:200%'></div><div id=n style='shape-image-threshold:-1'></div>");
    update(&mut owner);
    let p = native(&owner, "p");
    let clip = unsafe { &*p.ClipPath().unwrap() };
    assert_eq!(clip.GetType(), ClipType::kShape);
    let clip = unsafe {
        &*(clip as *const dyn layoutng_style::style::clip_path_operation::ClipPathOperation
            as *const ShapeClipPathOperation)
    };
    assert_eq!(clip.GetGeometryBox(), GeometryBox::kContentBox);
    let shape = clip.GetBasicShape();
    assert_eq!(shape.GetType(), ShapeType::kBasicShapePolygonType);
    let polygon = unsafe {
        &*(shape as *const dyn layoutng_style::style::basic_shapes::BasicShape
            as *const BasicShapePolygon)
    };
    assert_eq!(polygon.Values().len(), 6);
    assert_eq!(polygon.GetWindRule(), WindRule::RULE_EVENODD);
    assert_eq!(polygon.RoundingRadius().Pixels(), 2.0);
    assert!(polygon.Values()[4].IsCalculated());
    let outside = unsafe { &*p.ShapeOutside() };
    assert_eq!(outside.GetType(), ShapeValueType::kShape);
    assert_eq!(outside.CssBox(), ShapeBox::kPaddingBox);
    assert_eq!(p.ShapeImageThreshold(), 0.75);
    let c = native(&owner, "c");
    assert!(p.ClipPath().unwrap() == c.ClipPath().unwrap());
    assert_eq!(p.ShapeOutside(), c.ShapeOutside());
    assert_eq!(c.ShapeImageThreshold(), 0.75);
    let i = native(&owner, "i");
    assert!(i.ClipPath().is_none() && i.ShapeOutside().is_null());
    assert_eq!(i.ShapeImageThreshold(), 0.0);
    let b = native(&owner, "b");
    let clip = unsafe { &*b.ClipPath().unwrap() };
    assert_eq!(clip.GetType(), ClipType::kGeometryBox);
    assert_eq!(
        unsafe {
            &*(clip as *const dyn layoutng_style::style::clip_path_operation::ClipPathOperation
                as *const GeometryBoxClipPathOperation)
        }
        .GetGeometryBox(),
        GeometryBox::kFillBox
    );
    assert_eq!(
        unsafe { &*b.ShapeOutside() }.GetType(),
        ShapeValueType::kBox
    );
    assert_eq!(b.ShapeImageThreshold(), 1.0);
    assert_eq!(native(&owner, "n").ShapeImageThreshold(), 0.0);
}
#[test]
fn cursor_keywords_clear_images_and_follow_source_inheritance_flags() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<div id=p style='cursor:-webkit-grab'><div id=c style='cursor:inherit'></div><div id=i style='cursor:initial'></div><div id=v style='cursor:nwse-resize'></div><div id=u style='cursor:unset'></div></div>");
    update(&mut owner);
    assert_eq!(native(&owner, "p").Cursor(), ECursor::kGrab);
    assert_eq!(native(&owner, "c").Cursor(), ECursor::kGrab);
    assert!(native(&owner, "c").CursorIsInherited());
    assert!(native(&owner, "c").Cursors().is_null());
    assert_eq!(native(&owner, "i").Cursor(), ECursor::kAuto);
    assert!(!native(&owner, "i").CursorIsInherited());
    assert_eq!(native(&owner, "v").Cursor(), ECursor::kNwseResize);
    assert_eq!(native(&owner, "u").Cursor(), ECursor::kGrab);
    let property = ParseProperty(
        CSSPropertyID::kCursor,
        &String::from("hand"),
        false,
        CSSParserMode::kHTMLQuirksMode,
    )
    .unwrap();
    assert_eq!(property[0].Value().CssText().Utf8(), "pointer");
}
#[test]
fn invalid_supported_effect_values_and_missing_native_owners_are_typed() {
    for (id, text) in [
        (CSSPropertyID::kFilter, "blur(-1px)"),
        (CSSPropertyID::kFilter, "blur(10%)"),
        (CSSPropertyID::kFilter, "opacity(-.5)"),
        (CSSPropertyID::kFilter, "hue-rotate(1)"),
        (CSSPropertyID::kFilter, "drop-shadow(1px 2px 3px 4px)"),
        (CSSPropertyID::kFilter, "drop-shadow(inset 1px 2px)"),
        (CSSPropertyID::kFilter, "grayscale(1,2)"),
        (CSSPropertyID::kBackdropFilter, "unknown(1)"),
        (CSSPropertyID::kClipPath, "polygon(0 0,)"),
        (CSSPropertyID::kClipPath, "polygon(evenodd 0 0)"),
        (CSSPropertyID::kClipPath, "polygon(round -1px,0 0)"),
        (CSSPropertyID::kClipPath, "padding-box content-box"),
        (CSSPropertyID::kShapeOutside, "fill-box"),
        (CSSPropertyID::kCursor, "hand"),
        (CSSPropertyID::kShapeImageThreshold, "1px"),
    ] {
        let error = ParseProperty(
            id,
            &String::from(text),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .err()
        .unwrap();
        assert_eq!(error.kind, PropertyParseErrorKind::Invalid, "{text}");
    }
    for (id, text) in [
        (CSSPropertyID::kClipPath, "circle(10px)"),
        (CSSPropertyID::kClipPath, "ellipse()"),
        (CSSPropertyID::kClipPath, "inset(2px)"),
        (CSSPropertyID::kShapeOutside, "path('M0 0')"),
        (CSSPropertyID::kCursor, "url(cursor.png),pointer"),
    ] {
        let error = ParseProperty(
            id,
            &String::from(text),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .err()
        .unwrap();
        assert_eq!(error.kind, PropertyParseErrorKind::Unsupported, "{text}");
    }
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    for (id, text) in [
        (CSSPropertyID::kFilter, "url('#filter')"),
        (CSSPropertyID::kClipPath, "url('#clip')"),
        (CSSPropertyID::kShapeOutside, "url(shape.png)"),
    ] {
        let mut builder =
            layoutng_style::style::computed_style::ComputedStyleBuilder::from_style(initial);
        let property = ParseProperty(
            id,
            &String::from(text),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        assert!(matches!(
            style::resolver::production_style_builder::Apply(
                id,
                &mut builder,
                None,
                property[0].Value(),
                16.0,
                &MediaValuesCachedData::default()
            ),
            Err(style::properties::longhand_dispatch::LonghandApplicationError::Unsupported(_))
        ));
    }
}
