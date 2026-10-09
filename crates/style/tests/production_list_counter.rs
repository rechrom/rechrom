use foundation::{AtomicString, CSSPropertyID, EListStylePosition, String};
use layoutng_style::style::computed_style::ComputedStyle;
use style::{
    css_value::CSSValuePayload,
    media_queries::MediaValuesCachedData,
    parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseProperty, PropertyParseErrorKind},
    },
    properties::longhand_dispatch::LonghandApplicationError,
    resolver::production_style_builder,
    StyleEngine,
};
fn native<'a>(owner: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
    let document = owner.GetDocument();
    let index = (0..document.NodeCount())
        .find(|&index| {
            document
                .Node(index)
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
fn counters_merge_duplicate_names_preserve_lists_and_inherit_each_directive() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=p style='counter-increment:n 2 n 3 extra;counter-reset:n 7 n 9;counter-set:n 4 n calc(5 + 1)'><div id=c style='counter-increment:inherit;counter-reset:inherit;counter-set:inherit'></div><div id=i style='counter-increment:initial;counter-reset:initial;counter-set:initial'></div><div id=clear style='counter-increment:none;counter-reset:none;counter-set:none'></div><div id=keep style='counter-increment:inherit;counter-reset:none;counter-set:inherit'></div><div id=sat style='counter-increment:n 2147483647 n 1 negative -9999999999;counter-reset:n 9999999999;counter-set:n -9999999999'></div></div>");
    update(&mut owner);
    let p = native(&owner, "p");
    let c = native(&owner, "c");
    for style in [p, c] {
        let n = style.GetCounterDirectivesForIdentifier(&AtomicString::from_str("n"));
        assert_eq!(n.IncrementValue(), 5);
        assert_eq!(n.ResetValueInt64(), Some(9));
        assert_eq!(n.SetValue(), 6);
        assert_eq!(
            style
                .GetCounterDirectivesForIdentifier(&AtomicString::from_str("extra"))
                .IncrementValue(),
            1
        );
        let increments = unsafe { &*style.CounterIncrementList() };
        assert_eq!(increments.len(), 3);
        assert_eq!(
            (
                increments[0].value,
                increments[1].value,
                increments[2].value
            ),
            (Some(2), Some(3), Some(1))
        );
        assert_eq!(unsafe { &*style.CounterResetList() }[1].value, Some(9));
        assert_eq!(unsafe { &*style.CounterSetList() }[1].value, Some(6));
    }
    assert_ne!(p.CounterIncrementList(), c.CounterIncrementList());
    for style in [native(&owner, "i"), native(&owner, "clear")] {
        assert!(!style
            .GetCounterDirectivesForIdentifier(&AtomicString::from_str("n"))
            .IsDefined());
        assert!(
            style.CounterIncrementList().is_null()
                && style.CounterResetList().is_null()
                && style.CounterSetList().is_null()
        );
    }
    let keep =
        native(&owner, "keep").GetCounterDirectivesForIdentifier(&AtomicString::from_str("n"));
    assert_eq!((keep.IncrementValue(), keep.SetValue()), (5, 6));
    assert!(!keep.IsReset());
    let sat = native(&owner, "sat");
    let n = sat.GetCounterDirectivesForIdentifier(&AtomicString::from_str("n"));
    assert_eq!(n.IncrementValue(), i32::MAX);
    assert_eq!(n.ResetValueInt64(), Some(9999999999));
    assert_eq!(n.ResetValue(), Some(i32::MAX));
    assert_eq!(n.SetValue(), i32::MIN);
    assert_eq!(
        sat.GetCounterDirectivesForIdentifier(&AtomicString::from_str("negative"))
            .IncrementValue(),
        i32::MIN
    );
    assert!(c.HasExplicitInheritance() && p.ChildHasExplicitInheritance());
}
#[test]
fn quotes_native_pairs_none_auto_initial_and_inheritance() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=p style='quotes:&quot;\\ab &quot; &quot;\\bb &quot; &quot;\\2039 &quot; &quot;\\203a &quot;'><div id=c style='quotes:inherit'></div><div id=n style='quotes:none'></div><div id=a style='quotes:auto'></div><div id=i style='quotes:initial'></div></div>");
    update(&mut owner);
    let p = native(&owner, "p");
    let c = native(&owner, "c");
    assert_eq!(p.Quotes().size(), 2);
    assert!(p.Quotes() == c.Quotes());
    assert_eq!(p.Quotes().GetOpenQuote(0).Utf8(), "«");
    assert_eq!(p.Quotes().GetCloseQuote(1).Utf8(), "›");
    assert_eq!(p.Quotes().GetOpenQuote(10).Utf8(), "‹");
    assert_eq!(p.Quotes().GetCloseQuote(-1).Utf8(), "");
    assert_eq!(native(&owner, "n").Quotes().size(), 0);
    assert!(!native(&owner, "n").Quotes().GetOpenQuote(0).IsNull());
    assert!(!p.Quotes().GetCloseQuote(-1).IsNull());
    assert!(!native(&owner, "n").Quotes().IsNull());
    assert!(native(&owner, "a").Quotes().IsNull() && native(&owner, "i").Quotes().IsNull());
}
#[test]
fn list_shorthand_resets_omitted_fields_and_inherits_native_values() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=p style='list-style:inside &quot;\\2192 &quot;'><div id=c style='list-style:inherit'></div><div id=i style='list-style:initial'></div><div id=reset style='list-style-position:inside;list-style-type:&quot;old&quot;;list-style-image:linear-gradient(red,blue);list-style:decimal'></div><div id=n style='list-style:none none inside'></div><div id=g style='list-style-image:linear-gradient(red,blue)'></div></div>");
    update(&mut owner);
    let p = native(&owner, "p");
    let c = native(&owner, "c");
    assert_eq!(p.ListStylePosition(), EListStylePosition::kInside);
    assert!(unsafe { &*p.ListStyleType().Get() }.IsString());
    assert_eq!(
        unsafe { &*p.ListStyleType().Get() }.GetStringValue().Utf8(),
        "→"
    );
    assert_eq!(p.ListStyleType().Get(), c.ListStyleType().Get());
    assert_eq!(c.ListStylePosition(), EListStylePosition::kInside);
    assert!(c.ListStylePositionIsInherited());
    for style in [native(&owner, "i"), native(&owner, "reset")] {
        assert_eq!(style.ListStylePosition(), EListStylePosition::kOutside);
        assert!(style.ListStyleImage().Get().is_null());
        assert!(!style.ListStylePositionIsInherited());
    }
    assert_eq!(
        unsafe { &*native(&owner, "i").ListStyleType().Get() }
            .GetCounterStyleName()
            .Utf8(),
        "disc"
    );
    assert_eq!(
        unsafe { &*native(&owner, "reset").ListStyleType().Get() }
            .GetCounterStyleName()
            .Utf8(),
        "decimal"
    );
    let g = native(&owner, "g");
    assert!(unsafe { &*g.ListStyleImage().Get() }.IsGeneratedImage());
    assert!(g.BackgroundLayers().GetImage().is_null());
    let n = native(&owner, "n");
    assert!(n.ListStyleType().Get().is_null() && n.ListStyleImage().Get().is_null());
    assert_eq!(n.ListStylePosition(), EListStylePosition::kInside);
}
#[test]
fn invalid_values_and_resource_scope_boundaries_remain_typed() {
    let _heap = foundation::LayoutHeapScope::new();
    for (id, text) in [
        (CSSPropertyID::kCounterIncrement, "none n"),
        (CSSPropertyID::kCounterReset, "n 1.5"),
        (CSSPropertyID::kCounterSet, "default"),
        (CSSPropertyID::kCounterIncrement, "reversed(n)"),
        (CSSPropertyID::kCounterSet, "reversed(n)"),
        (CSSPropertyID::kQuotes, "'odd'"),
        (CSSPropertyID::kQuotes, "'a' 1"),
        (CSSPropertyID::kListStylePosition, "sideways"),
        (CSSPropertyID::kListStyle, "none none none"),
        (CSSPropertyID::kListStyle, "inside outside disc"),
    ] {
        assert_eq!(
            ParseProperty(
                id,
                &String::from(text),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .err()
            .unwrap_or_else(|| panic!("accepted invalid value: {text}"))
            .kind,
            PropertyParseErrorKind::Invalid,
            "{text}"
        );
    }
    for (id, text) in [
        (CSSPropertyID::kCounterReset, "reversed(n)"),
        (CSSPropertyID::kListStyleType, "symbols('a')"),
    ] {
        assert_eq!(
            ParseProperty(
                id,
                &String::from(text),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .err()
            .unwrap_or_else(|| panic!("accepted invalid value: {text}"))
            .kind,
            PropertyParseErrorKind::Unsupported
        );
    }
    let mut b = layoutng_style::style::computed_style::ComputedStyleBuilder::from_style(unsafe {
        &*ComputedStyle::GetInitialStyleSingleton()
    });
    for (id, text) in [
        (CSSPropertyID::kListStyleImage, "url(marker.png)"),
        (CSSPropertyID::kListStyleType, "CustomName"),
        (CSSPropertyID::kListStyleType, "upper-roman"),
    ] {
        let parsed = ParseProperty(
            id,
            &String::from(text),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        if id == CSSPropertyID::kListStyleImage {
            assert!(matches!(
                parsed[0].Value().Payload(),
                CSSValuePayload::kImageClass(_)
            ));
        } else {
            assert!(matches!(
                parsed[0].Value().Payload(),
                CSSValuePayload::kCustomIdentClass(_)
            ));
        }
        assert_eq!(
            production_style_builder::Apply(
                id,
                &mut b,
                None,
                parsed[0].Value(),
                16.0,
                &MediaValuesCachedData::default()
            ),
            Err(LonghandApplicationError::Unsupported(id))
        );
    }
    struct MissingFetchedResource(std::cell::Cell<usize>);
    impl production_style_builder::URLImageResolver for MissingFetchedResource {
        fn ResolveImage(
            &self,
            value: &style::production_css_value::CSSImageValue,
        ) -> Result<production_style_builder::FetchedImageBinding, LonghandApplicationError>
        {
            assert_eq!(value.url.Utf8(), "marker.png");
            self.0.set(self.0.get() + 1);
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kListStyleImage,
            ))
        }
    }
    let resolver = MissingFetchedResource(std::cell::Cell::new(0));
    let image = ParseProperty(
        CSSPropertyID::kListStyleImage,
        &String::from("url(marker.png)"),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap();
    assert_eq!(
        production_style_builder::ApplyWithImageResolver(
            CSSPropertyID::kListStyleImage,
            &mut b,
            None,
            image[0].Value(),
            16.0,
            &MediaValuesCachedData::default(),
            &resolver
        ),
        Err(LonghandApplicationError::Unsupported(
            CSSPropertyID::kListStyleImage
        ))
    );
    assert_eq!(resolver.0.get(), 1);
    let upper = ParseProperty(
        CSSPropertyID::kListStyleType,
        &String::from("UPPER-ROMAN"),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap();
    assert!(
        matches!(upper[0].Value().Payload(), CSSValuePayload::kCustomIdentClass(name) if name.name.Utf8() == "upper-roman")
    );
    assert!(b.ListStyleImage().Get().is_null());
}
