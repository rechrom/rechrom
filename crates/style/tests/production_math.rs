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
        "calc(1px / 1px)",
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
