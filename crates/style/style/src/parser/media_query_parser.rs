pub use style_parser::media_query_parser::*;
#[cfg(test)]
mod tests {
    use super::*;
    use crate::kleene_value::KleeneValue;
    use crate::media_queries::conditional_exp_node::ConditionalExpNode;
    use crate::media_queries::media_query_evaluator::{
        MediaQueryEvaluationBackend, MediaQueryEvaluator,
    };
    use crate::media_queries::media_query_exp::{
        MediaQueryExp, MediaQueryExpSerialization, MediaQueryExpValue, MediaQueryOperator,
    };
    use crate::media_queries::media_query_set::MediaQuerySet;
    use crate::parser::css_parser_token::CSSParserTokenType::*;
    use crate::parser::css_parser_token_stream::{CSSParserTokenStream, TokenStreamTokenizer};
    use crate::parser::css_tokenizer::CSSTokenizer;
    use foundation::{AtomicString, CSSValueID, String, StringView};
    use std::cell::Cell;
    use std::rc::Rc;

    // A constrained CSSValue fixture, used only to isolate the source parser
    // state machine. Production callers must implement the real CSS consumer.
    struct FixtureValue(String);
    impl MediaQueryExpSerialization for FixtureValue {
        fn CssText(&self) -> String {
            self.0.clone()
        }
    }
    struct FixtureBackend {
        range_count: usize,
        eval_count: Cell<usize>,
    }
    impl FixtureBackend {
        fn new() -> Self {
            Self {
                range_count: 0,
                eval_count: Cell::new(0),
            }
        }
    }
    impl MediaQueryRuntimeFeatures for FixtureBackend {
        fn CSSCustomMediaEnabled(&self) -> bool {
            true
        }
        fn CSSScrolledContainerQueriesEnabled(&self) -> bool {
            true
        }
        fn PrefersReducedDataEnabled(&self) -> bool {
            true
        }
        fn ForcedColorsEnabled(&self) -> bool {
            true
        }
        fn MediaQueryNavigationControlsEnabled(&self) -> bool {
            true
        }
        fn OriginTrialsSampleAPIEnabled(&self) -> bool {
            true
        }
        fn ViewportSegmentsEnabled(&self) -> bool {
            true
        }
        fn DevicePostureEnabled(&self) -> bool {
            true
        }
        fn InvertedColorsEnabled(&self) -> bool {
            true
        }
        fn DesktopPWAsAdditionalWindowingControlsEnabled(&self) -> bool {
            true
        }
    }
    impl MediaQueryParserBackend for FixtureBackend {
        type Value = FixtureValue;
        type UnparsedValue = FixtureValue;
        fn ConsumeValue<T: TokenStreamTokenizer>(
            &mut self,
            feature: &AtomicString,
            stream: &mut CSSParserTokenStream<'_, T>,
            element_dependent: bool,
        ) -> Option<MediaQueryExpValue<FixtureValue>> {
            assert!(!element_dependent);
            if feature == "prefers-color-scheme"
                && matches!(stream.Peek().Id(), CSSValueID::kLight | CSSValueID::kDark)
            {
                return Some(MediaQueryExpValue::FromId(
                    stream.ConsumeIncludingWhitespace().Id(),
                ));
            }
            if feature != "width" || stream.Peek().GetType() != kDimensionToken {
                return None;
            }
            if stream.Peek().GetUnitType() != crate::css_primitive_value::UnitType::kPixels {
                return None;
            }
            let value = stream.ConsumeIncludingWhitespace().NumericValue();
            Some(MediaQueryExpValue::FromValue(Rc::new(FixtureValue(
                String::from(format!("{value}px").as_str()),
            ))))
        }
        fn UseCountRangeSyntax(&mut self) {
            self.range_count += 1;
        }
    }
    impl MediaQueryEvaluationBackend for FixtureBackend {
        type Document = ();
        type Value = FixtureValue;
        type UnparsedValue = FixtureValue;
        type ResultFlags = usize;
        type CustomMediaRulesMap = ();
        fn EvalFeature(
            &self,
            feature: &MediaQueryExp<FixtureValue>,
            flags: Option<&mut usize>,
            _: Option<&()>,
        ) -> KleeneValue {
            self.eval_count.set(self.eval_count.get() + 1);
            if let Some(flags) = flags {
                *flags += 1;
            }
            match feature.MediaFeature().Utf8().as_str() {
                "width" => KleeneValue::kFalse,
                "prefers-color-scheme" => KleeneValue::kTrue,
                "--fixture" => KleeneValue::kUnknown,
                _ => panic!("test leaf not specified"),
            }
        }
    }
    fn parse(text: &str) -> MediaQuerySet<FixtureValue> {
        MediaQueryParser::new(FixtureBackend::new()).ParseMediaQuerySet(StringView::from(text))
    }
    #[test]
    fn media_query_parser_lists_recover_at_top_level_commas() {
        for (input, expected) in [
            ("", ""),
            ("SCREEN, only print", "screen, only print"),
            ("screen and, print", "not all, print"),
            ("screen,", "screen, not all"),
            (",screen", "not all, screen"),
            ("layer, all", "not all, all"),
            ("(unknown: 1, 2), screen", "(unknown: 1, 2), screen"),
            (
                "screen and (width: 1px) or (width: 2px), print",
                "not all, print",
            ),
            ("only (width: 1px), print", "not all, print"),
            ("(width < = 1px), print", "(width < = 1px), print"),
        ] {
            assert_eq!(parse(input).MediaText().Utf8(), expected, "{input}");
        }
    }
    #[test]
    fn media_query_parser_range_bounds_and_unknown_fallback() {
        for (input, expected) in [
            ("(WIDTH<=10px)", "(width <= 10px)"),
            ("(10px < width <= 20px)", "(10px < width <= 20px)"),
            ("(20px >= width > 10px)", "(20px >= width > 10px)"),
            ("(width)", "(width)"),
            ("(--fixture)", "(--fixture)"),
            (
                "not (prefers-color-scheme:dark)",
                "not (prefers-color-scheme: dark)",
            ),
            ("(10px < width > 20px)", "(10px < width > 20px)"),
        ] {
            assert_eq!(parse(input).MediaText().Utf8(), expected, "{input}");
        }
        let range = parse("(10px < width <= 20px)");
        let mut expressions = Vec::new();
        range.QueryVector()[0].CollectExpressions(&mut expressions);
        assert_eq!(expressions.len(), 1);
        assert_eq!(expressions[0].Bounds().left.op, MediaQueryOperator::kLt);
        assert_eq!(expressions[0].Bounds().right.op, MediaQueryOperator::kLe);
        let mut expressions = Vec::new();
        parse("(10px < width > 20px)").QueryVector()[0].CollectExpressions(&mut expressions);
        assert!(expressions.is_empty()); // general-enclosed retains Unknown, never a false feature.
    }
    #[test]
    fn media_query_parser_tri_state_short_circuit_and_restrictors() {
        let evaluator =
            MediaQueryEvaluator::FromMediaType(String::from("screen"), FixtureBackend::new());
        for (input, expected) in [
            ("", true),
            ("screen", true),
            ("only print", false),
            ("not print", true),
            ("not all", false),
            ("not (unknown: x)", false),
            ("not (--fixture)", false),
            ("not (width)", true),
            ("(unknown: x) or (prefers-color-scheme: dark)", true),
            ("(width) and (prefers-color-scheme: dark)", false),
            ("print, (prefers-color-scheme: dark)", true),
        ] {
            assert_eq!(evaluator.Eval(&parse(input)), expected, "{input}");
        }
        let mut visits = 0;
        evaluator.EvalSet(
            &parse("(width) and (prefers-color-scheme: dark)"),
            Some(&mut visits),
            None,
        );
        assert_eq!(visits, 1);
        visits = 0;
        evaluator.EvalSet(
            &parse("(prefers-color-scheme: dark) or (width)"),
            Some(&mut visits),
            None,
        );
        assert_eq!(visits, 1);
        let mut expressions = Vec::new();
        parse("(width) and (prefers-color-scheme: dark)").QueryVector()[0]
            .CollectExpressions(&mut expressions);
        assert_eq!(expressions.len(), 2); // collector returns Unknown to prevent short-circuiting.
    }
    #[test]
    fn media_query_parser_custom_definition_boundary_and_set_copy_identity() {
        let mut parser = MediaQueryParser::new(FixtureBackend::new());
        let mut stream =
            CSSParserTokenStream::<CSSTokenizer>::new(StringView::from("screen; print"), 0);
        assert_eq!(
            parser
                .ParseCustomMediaDefinition(&mut stream)
                .MediaText()
                .Utf8(),
            "screen"
        );
        assert_eq!(stream.Peek().GetType(), kSemicolonToken);
        let queries = Rc::new(parse("screen, screen, print"));
        assert!(queries
            .CopyAndAdd(&String::from("SCREEN"), &mut parser)
            .is_none());
        let unchanged = queries
            .CopyAndRemove(&String::from("screen, print"), &mut parser)
            .unwrap();
        assert!(Rc::ptr_eq(&queries, &unchanged));
        assert_eq!(
            queries
                .CopyAndRemove(&String::from("SCREEN"), &mut parser)
                .unwrap()
                .MediaText()
                .Utf8(),
            "print"
        );
        assert!(queries
            .CopyAndRemove(&String::from("speech"), &mut parser)
            .is_none());
    }
}
