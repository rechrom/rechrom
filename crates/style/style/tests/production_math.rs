use foundation::{CSSPropertyID, String, StringView};
use style::{
    css_math_expression_node::{
        CalculationResultCategory as Category, ConsumeMathFunction, MathError,
    },
    css_math_function_value::{CSSMathFunctionValue, ValueRange},
    css_primitive_value::UnitType,
    css_value::CSSValuePayload,
    parser::{
        css_parser_mode::CSSParserMode, css_parser_token_stream::CSSParserTokenStream,
        css_tokenizer::CSSTokenizer, production_property_parser::ParseProperty,
    },
};
fn expression(text: &str) -> style::css_math_expression_node::CSSMathExpressionNode {
    let text = String::from(text);
    let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(&text), 0);
    let expression = ConsumeMathFunction(&mut stream).unwrap();
    assert!(stream.AtEnd());
    expression
}
fn absent_length(_: f64, _: UnitType) -> Result<f64, MathError> {
    Err(MathError::MissingLengthContext)
}
#[test]
fn typed_arithmetic_precedence_categories_and_comparison_functions() {
    for (text, category, value) in [
        ("calc(2 + 3 * 4)", Category::Number, 14.0),
        ("calc((2 + 3) * 4)", Category::Number, 20.0),
        ("calc(min(9, 3) + max(2, 4))", Category::Number, 7.0),
        ("clamp(10, 2, 5)", Category::Number, 10.0),
        ("clamp(none, 2, 5)", Category::Number, 2.0),
        ("clamp(10, 2, none)", Category::Number, 10.0),
        ("calc(0.25turn + 90deg)", Category::Angle, 180.0),
        ("max(500ms, calc(1s / 2))", Category::Time, 0.5),
        ("min(40%, 25%)", Category::Percent, 25.0),
    ] {
        let expression = expression(text);
        assert_eq!(expression.Category(), category, "{text}");
        assert_eq!(
            expression.ComputeValue(&mut absent_length, None).unwrap(),
            value,
            "{text}"
        )
    }
    for text in [
        "calc(1px + 1s)",
        "calc(1px + 1)",
        "calc(1px * 1px)",
        "calc(1px+ 2px)",
        "calc(1px +2px)",
        "min()",
        "clamp(1px,2px)",
        "clamp(1px,none,2px)",
        "calc(1, 2)",
        "calc(min(1, 2)+ 3)",
    ] {
        let text = String::from(text);
        let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(&text), 0);
        assert!(ConsumeMathFunction(&mut stream).is_err(), "{}", text.Utf8());
    }
    let mut resolver = |value, unit| match unit {
        UnitType::kPixels => Ok(value),
        UnitType::kViewportWidth => Ok(value * 8.0),
        _ => Err(MathError::MissingLengthContext),
    };
    let quotient = expression("min(calc(100vw / 1600px), calc(1080px / 1600px))");
    assert_eq!(quotient.Category(), Category::Number);
    assert_eq!(quotient.ComputeValue(&mut resolver, None).unwrap(), 0.5);
}
#[test]
fn level_four_math_functions_keep_chromium_categories_and_evaluation() {
    let cases = [
        ("round(up, 5, 2)", Category::Number, 6.0),
        ("round(5)", Category::Number, 5.0),
        ("mod(-5, 3)", Category::Number, 1.0),
        ("rem(-5, 3)", Category::Number, -2.0),
        ("hypot(3, 4)", Category::Number, 5.0),
        ("abs(-2)", Category::Number, 2.0),
        ("sign(-2px)", Category::Number, -1.0),
        ("log(8, 2)", Category::Number, 3.0),
        ("sqrt(9)", Category::Number, 3.0),
        ("pow(2, 3)", Category::Number, 8.0),
        ("sin(calc(pi / 2))", Category::Number, 1.0),
        ("cos(180deg)", Category::Number, -1.0),
        ("asin(1)", Category::Angle, 90.0),
        ("atan2(1, 1)", Category::Angle, 45.0),
    ];
    let mut resolver = |value, unit| match unit {
        UnitType::kPixels => Ok(value),
        _ => Err(MathError::MissingLengthContext),
    };
    for (text, category, expected) in cases {
        let expression = expression(text);
        assert_eq!(expression.Category(), category, "{text}");
        let actual = expression.ComputeValue(&mut resolver, None).unwrap();
        assert!((actual - expected).abs() < 0.0001, "{text}: {actual}");
    }
    assert_eq!(
        expression("hypot(3px, 4px)")
            .ComputeValue(&mut resolver, None)
            .unwrap(),
        5.0
    );
    assert_eq!(
        expression("round(down, 5px, 2px)").CssText(),
        "round(down, 5px, 2px)"
    );
    for text in [
        "round(1px)",
        "round(up 5, 2)",
        "mod(1px, 2s)",
        "sqrt(4px)",
        "pow(2px, 2)",
        "sin(1px)",
        "asin(1deg)",
        "atan2(1, 1px)",
        "log(1, 2, 3)",
    ] {
        let text = String::from(text);
        let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(&text), 0);
        assert!(ConsumeMathFunction(&mut stream).is_err(), "{}", text.Utf8());
    }
}
#[test]
fn mixed_percentages_retain_native_used_value_basis_and_clamp_at_computation() {
    let mut resolver = |value, unit| match unit {
        UnitType::kPixels => Ok(value),
        UnitType::kEms => Ok(value * 20.0),
        UnitType::kViewportWidth => Ok(value * 8.0),
        _ => Err(MathError::MissingLengthContext),
    };
    let math = CSSMathFunctionValue::Create(
        expression("clamp(20px, calc(50% - 2em), 10vw)"),
        ValueRange::NonNegative,
    );
    assert_eq!(
        math.ComputeValue(&mut resolver, None),
        Err(MathError::MissingPercentageBasis)
    );
    let length = math.ConvertToLength(&mut resolver).unwrap();
    assert!(length.IsCalculated());
    for (basis, wanted) in [(100.0, 20.0), (200.0, 60.0), (400.0, 80.0)] {
        assert_eq!(
            length
                .GetCalculationValue()
                .Evaluate(basis, &foundation::EvaluationInput::default()),
            wanted
        );
        assert_eq!(
            math.ComputeValue(&mut resolver, Some(basis as f64))
                .unwrap(),
            wanted as f64
        );
    }
    let math =
        CSSMathFunctionValue::Create(expression("calc(50% - 100px)"), ValueRange::NonNegative);
    assert_eq!(
        math.ConvertToLength(&mut resolver)
            .unwrap()
            .GetCalculationValue()
            .Evaluate(100.0, &foundation::EvaluationInput::default()),
        0.0
    );
    let infinite =
        CSSMathFunctionValue::Create(expression("calc(infinity * 1px)"), ValueRange::All);
    assert!(infinite
        .ConvertToLength(&mut resolver)
        .unwrap()
        .Pixels()
        .is_finite());
    let integer = CSSMathFunctionValue::Create(expression("calc(-1.5)"), ValueRange::Integer);
    assert_eq!(
        integer.ComputeValue(&mut absent_length, None).unwrap(),
        -1.0
    );
    let nan = CSSMathFunctionValue::Create(expression("min(NaN, 2)"), ValueRange::All);
    assert_eq!(nan.ComputeValue(&mut absent_length, None).unwrap(), 0.0);
}
#[test]
fn property_consumers_accept_typed_math_and_keep_invalid_categories_out() {
    for (id, text) in [
        (CSSPropertyID::kWidth, "calc(50% - 10px)"),
        (CSSPropertyID::kMargin, "min(1em, 20px)"),
        (CSSPropertyID::kOpacity, "max(25%, 50%)"),
        (CSSPropertyID::kZIndex, "calc(1 + 0.5)"),
        (CSSPropertyID::kTransitionDuration, "max(100ms, 0.2s)"),
        (CSSPropertyID::kAnimationIterationCount, "min(2, 3)"),
    ] {
        let values = ParseProperty(
            id,
            &String::from(text),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        let value = values[0].Value();
        assert!(
            matches!(
                value.Payload(),
                CSSValuePayload::kMathFunctionClass(_) | CSSValuePayload::kValueListClass(_)
            ),
            "{id:?}"
        );
    }
    for (id, text) in [
        (CSSPropertyID::kWidth, "calc(1s + 2s)"),
        (CSSPropertyID::kOpacity, "calc(1px + 2px)"),
        (CSSPropertyID::kTransitionDuration, "calc(1deg + 2deg)"),
    ] {
        assert!(ParseProperty(
            id,
            &String::from(text),
            false,
            CSSParserMode::kHTMLStandardMode
        )
        .is_err())
    }
}
fn node(owner: &dom::DOM, id: &str) -> usize {
    (0..owner.GetDocument().NodeCount())
        .find(|&n| {
            owner
                .GetDocument()
                .Node(n)
                .FindAttribute("id")
                .is_some_and(|a| a.value == id)
        })
        .unwrap()
}
#[test]
fn variables_and_container_numeric_ranges_compute_through_production_style() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<div id=outer style='font-size:20px;--base:10px;--gap:calc(var(--base) + 1em);--angle:calc(.25turn + 90deg);--time:calc(100ms + .2s)'><span id=target></span></div>");
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("#target{width:calc(var(--gap) * 2);height:clamp(10px,calc(50% - 2px),100px);opacity:max(25%,50%);z-index:calc(-1.5);transition-duration:max(100ms,.2s)}@container style(calc(1em + 5px) < --gap < max(35px, 2em)){#target{margin-left:calc(5px + 1em)}}@container style(170deg < --angle < 200deg){#target{padding-top:5px}}@container style(200ms < --time < .4s){#target{padding-bottom:7px}}"));
    let target = node(&owner, "target");
    let outer = node(&owner, "outer");
    let mut engine = style::StyleEngine::new(&owner);
    let media = style::media_queries::MediaValuesCachedData {
        media_type: String::from("screen"),
        large_viewport_width: 800.0,
        large_viewport_height: 600.0,
        ..Default::default()
    };
    engine.Update(&mut owner, &media, &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let native = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(native.Width().Pixels(), 60.0);
    assert!(native.Height().IsCalculated());
    assert_eq!(
        native
            .Height()
            .GetCalculationValue()
            .Evaluate(100.0, &foundation::EvaluationInput::default()),
        48.0
    );
    assert_eq!(native.MarginLeft().Pixels(), 25.0);
    assert_eq!(native.PaddingTop().Pixels(), 5.0);
    assert_eq!(native.PaddingBottom().Pixels(), 7.0);
    assert_eq!(native.Opacity(), 0.5);
    assert_eq!(native.ZIndex(), -1);
    owner.GetDocumentMut().SetAttribute(
        outer,
        dom::persistent_document::DOMAttribute {
            local_name: "style".into(),
            value: "font-size:20px;--base:30px;--gap:calc(var(--base) + 1em)".into(),
            ..Default::default()
        },
    );
    engine.Update(&mut owner, &media, &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let native = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(native.Width().Pixels(), 100.0);
    assert_eq!(native.MarginLeft().Pixels(), 0.0);
}

#[test]
fn css_math_keeps_double_precision_until_native_lowering() {
    for (text, expected) in [
        ("sign(1e-50)", 1.0),
        ("sign(-1e-50)", -1.0),
        ("mod(16777217, 2)", 1.0),
        ("rem(16777217, 2)", 1.0),
        ("round(pow(1e25, 2), 1)", 1e25f64.powf(2.0)),
    ] {
        assert_eq!(
            expression(text)
                .ComputeValue(&mut absent_length, None)
                .unwrap(),
            expected,
            "{text}"
        );
    }
    let actual = expression("atan2(pow(1e25, 4), pow(1e33, 3))")
        .ComputeValue(&mut absent_length, None)
        .unwrap();
    assert!((actual - 10.0f64.atan().to_degrees()).abs() < 1e-12);
    let actual = expression("tan(90.000001deg)")
        .ComputeValue(&mut absent_length, None)
        .unwrap();
    assert!(actual.is_finite() && actual < -1e7, "{actual}");
    for text in [
        "round(1, 0)",
        "mod(infinity, 2)",
        "round(NaN, infinity)",
        "atan2(NaN, 1)",
    ] {
        assert!(
            expression(text)
                .ComputeValue(&mut absent_length, None)
                .unwrap()
                .is_nan(),
            "{text}"
        );
    }
    assert!(expression("round(up, 1, infinity)")
        .ComputeValue(&mut absent_length, None)
        .unwrap()
        .is_infinite());
    let minus_zero = expression("sign(-0)")
        .ComputeValue(&mut absent_length, None)
        .unwrap();
    assert!(minus_zero == 0.0 && minus_zero.is_sign_negative());
}

#[test]
fn number_and_angle_results_retain_nested_percentage_dependencies() {
    let mut resolver = |value, unit| match unit {
        UnitType::kPixels => Ok(value),
        UnitType::kEms => Ok(value * 20.0),
        _ => Err(MathError::MissingLengthContext),
    };
    let sign = expression("sign(10px - 50%)");
    assert_eq!(sign.Category(), Category::Number);
    assert_eq!(
        sign.ComputeValue(&mut resolver, None),
        Err(MathError::MissingPercentageBasis)
    );
    assert_eq!(sign.ComputeValue(&mut resolver, Some(10.0)).unwrap(), 1.0);
    assert_eq!(sign.ComputeValue(&mut resolver, Some(100.0)).unwrap(), -1.0);
    let atan = expression("atan2(10px + 10%, 10px - 10%)");
    assert_eq!(atan.Category(), Category::Angle);
    assert_eq!(
        atan.ComputeValue(&mut resolver, None),
        Err(MathError::MissingPercentageBasis)
    );
    for basis in [10.0, 200.0] {
        let expected = (10.0f64 + basis / 10.0)
            .atan2(10.0 - basis / 10.0)
            .to_degrees();
        assert!((atan.ComputeValue(&mut resolver, Some(basis)).unwrap() - expected).abs() < 1e-12);
    }
    assert!(expression("hypot(NaN * 1em, infinity * 1px)")
        .ComputeValue(&mut resolver, None)
        .unwrap()
        .is_nan());
}

#[test]
fn typed_math_leaves_lower_to_native_calculations_in_canonical_units() {
    let mut resolver = |value, unit| match unit {
        UnitType::kPixels => Ok(value),
        _ => Err(MathError::MissingLengthContext),
    };
    for (text, basis, expected) in [
        ("calc(50% + sin(.25turn) * 10px)", 100.0, 60.0),
        ("calc(50% + sign(1e-50) * 10px)", 100.0, 60.0),
        ("calc(50% + mod(16777217, 2) * 10px)", 100.0, 60.0),
        ("calc(50% + cos(0rad) * 10px)", 100.0, 60.0),
        ("calc(50% + sign(-1ms) * 2px)", 100.0, 48.0),
        ("calc(50% + sign(-1kHz) * 2px)", 100.0, 48.0),
        ("calc(50% + sign(-1dpi) * 2px)", 100.0, 48.0),
        (
            "calc(50% + sin(90deg) * 10px + sign(10px - 50%) * 2px)",
            100.0,
            58.0,
        ),
        (
            "calc(50% + sin(90deg) * 10px + sign(10px - 50%) * 2px)",
            10.0,
            17.0,
        ),
    ] {
        let expression = expression(text);
        let direct = expression.ComputeValue(&mut resolver, Some(basis)).unwrap();
        let native = expression
            .ToCalculationExpression(&mut resolver)
            .unwrap()
            .Evaluate(basis as f32, &foundation::EvaluationInput::default());
        assert!((direct - expected).abs() < 1e-12, "{text}: {direct}");
        assert!((native as f64 - expected).abs() < 0.001, "{text}: {native}");
    }
    for (text, expected) in [
        ("calc(500ms)", 0.5),
        ("calc(1kHz)", 1000.0),
        ("calc(96dpi)", 1.0),
        ("calc(.25turn)", 90.0),
    ] {
        let node = expression(text)
            .ToCalculationExpression(&mut resolver)
            .unwrap();
        assert_eq!(
            node.Evaluate(0.0, &foundation::EvaluationInput::default()),
            expected,
            "{text}"
        );
    }
}
